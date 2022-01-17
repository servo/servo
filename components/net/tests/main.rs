/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![cfg(test)]
#![allow(dead_code)]

#[macro_use]
extern crate lazy_static;

mod cookie;
mod cookie_http_state;
mod data_loader;
mod fetch;
mod file_loader;
mod filemanager_thread;
mod hsts;
mod http_cache;
mod http_loader;
mod mime_classifier;
mod resource_thread;
mod subresource_integrity;

use core::convert::Infallible;
use core::pin::Pin;
use crossbeam_channel::{unbounded, Sender};
use devtools_traits::DevtoolsControlMsg;
use embedder_traits::resources::{self, Resource};
use embedder_traits::{EmbedderProxy, EventLoopWaker};
use futures::future::ready;
use futures::StreamExt;
use hyper::server::conn::Http;
use hyper::server::Server as HyperServer;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request as HyperRequest, Response as HyperResponse};
use net::connector::{create_tls_config, ConnectionCerts, ExtraCerts, ALPN_H2_H1};
use net::fetch::cors_cache::CorsCache;
use net::fetch::methods::{self, CancellationListener, FetchContext};
use net::filemanager_thread::FileManager;
use net::resource_thread::CoreResourceThreadPool;
use net::test::HttpState;
use net_traits::filemanager_thread::FileTokenCheck;
use net_traits::request::Request;
use net_traits::response::Response;
use net_traits::{FetchTaskTarget, ResourceFetchTiming, ResourceTimingType};
use openssl::ssl::{Ssl, SslAcceptor, SslFiletype, SslMethod};
use servo_arc::Arc as ServoArc;
use servo_url::ServoUrl;
use std::net::TcpListener as StdTcpListener;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, Weak};
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::runtime::{Builder, Runtime};
use tokio_openssl::SslStream;
use tokio_stream::wrappers::TcpListenerStream;
use tokio_test::block_on;

lazy_static! {
    pub static ref HANDLE: Mutex<Runtime> = Mutex::new(
        Builder::new_multi_thread()
            .enable_io()
            .worker_threads(10)
            .build()
            .unwrap()
    );
}

const DEFAULT_USER_AGENT: &'static str = "Such Browser. Very Layout. Wow.";

struct FetchResponseCollector {
    sender: Sender<Response>,
}

fn create_embedder_proxy() -> EmbedderProxy {
    let (sender, _) = unbounded();
    let event_loop_waker = || {
        struct DummyEventLoopWaker {}
        impl DummyEventLoopWaker {
            fn new() -> DummyEventLoopWaker {
                DummyEventLoopWaker {}
            }
        }
        impl EventLoopWaker for DummyEventLoopWaker {
            fn wake(&self) {}
            fn clone_box(&self) -> Box<dyn EventLoopWaker> {
                Box::new(DummyEventLoopWaker {})
            }
        }

        Box::new(DummyEventLoopWaker::new())
    };

    EmbedderProxy {
        sender: sender,
        event_loop_waker: event_loop_waker(),
    }
}

fn new_fetch_context(
    dc: Option<Sender<DevtoolsControlMsg>>,
    fc: Option<EmbedderProxy>,
    pool_handle: Option<Weak<CoreResourceThreadPool>>,
) -> FetchContext {
    let certs = resources::read_string(Resource::SSLCertificates);
    let tls_config = create_tls_config(
        &certs,
        ALPN_H2_H1,
        ExtraCerts::new(),
        ConnectionCerts::new(),
    );
    let sender = fc.unwrap_or_else(|| create_embedder_proxy());

    FetchContext {
        state: Arc::new(HttpState::new(tls_config)),
        user_agent: DEFAULT_USER_AGENT.into(),
        devtools_chan: dc.map(|dc| Arc::new(Mutex::new(dc))),
        filemanager: Arc::new(Mutex::new(FileManager::new(
            sender,
            pool_handle.unwrap_or_else(|| Weak::new()),
        ))),
        file_token: FileTokenCheck::NotRequired,
        cancellation_listener: Arc::new(Mutex::new(CancellationListener::new(None))),
        timing: ServoArc::new(Mutex::new(ResourceFetchTiming::new(
            ResourceTimingType::Navigation,
        ))),
    }
}
impl FetchTaskTarget for FetchResponseCollector {
    fn process_request_body(&mut self, _: &Request) {}
    fn process_request_eof(&mut self, _: &Request) {}
    fn process_response(&mut self, _: &Response) {}
    fn process_response_chunk(&mut self, _: Vec<u8>) {}
    /// Fired when the response is fully fetched
    fn process_response_eof(&mut self, response: &Response) {
        let _ = self.sender.send(response.clone());
    }
}

fn fetch(request: &mut Request, dc: Option<Sender<DevtoolsControlMsg>>) -> Response {
    fetch_with_context(request, &mut new_fetch_context(dc, None, None))
}

fn fetch_with_context(request: &mut Request, mut context: &mut FetchContext) -> Response {
    let (sender, receiver) = unbounded();
    let mut target = FetchResponseCollector { sender: sender };
    block_on(async move {
        methods::fetch(request, &mut target, &mut context).await;
        receiver.recv().unwrap()
    })
}

fn fetch_with_cors_cache(request: &mut Request, cache: &mut CorsCache) -> Response {
    let (sender, receiver) = unbounded();
    let mut target = FetchResponseCollector { sender: sender };
    block_on(async move {
        methods::fetch_with_cors_cache(
            request,
            cache,
            &mut target,
            &mut new_fetch_context(None, None, None),
        )
        .await;
        receiver.recv().unwrap()
    })
}

pub(crate) struct Server {
    pub close_channel: tokio::sync::oneshot::Sender<()>,
}

impl Server {
    fn close(self) {
        self.close_channel.send(()).expect("err closing server:");
    }
}

fn make_server<H>(handler: H) -> (Server, ServoUrl)
where
    H: Fn(HyperRequest<Body>, &mut HyperResponse<Body>) + Send + Sync + 'static,
{
    let handler = Arc::new(handler);
    let listener = StdTcpListener::bind("0.0.0.0:0").unwrap();
    let url_string = format!("http://localhost:{}", listener.local_addr().unwrap().port());
    let url = ServoUrl::parse(&url_string).unwrap();
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    let server = async move {
        HyperServer::from_tcp(listener)
            .unwrap()
            .serve(make_service_fn(move |_| {
                let handler = handler.clone();
                ready(Ok::<_, Infallible>(service_fn(
                    move |req: HyperRequest<Body>| {
                        let mut response = HyperResponse::new(Vec::<u8>::new().into());
                        handler(req, &mut response);
                        ready(Ok::<_, Infallible>(response))
                    },
                )))
            }))
            .with_graceful_shutdown(async move {
                rx.await.ok();
            })
            .await
            .expect("Could not start server");
    };

    HANDLE.lock().unwrap().spawn(server);
    let server = Server { close_channel: tx };
    (server, url)
}

fn make_ssl_server<H>(handler: H, cert_path: PathBuf, key_path: PathBuf) -> (Server, ServoUrl)
where
    H: Fn(HyperRequest<Body>, &mut HyperResponse<Body>) + Send + Sync + 'static,
{
    let handler = Arc::new(handler);
    let listener = StdTcpListener::bind("[::0]:0").unwrap();
    let listener = HANDLE
        .lock()
        .unwrap()
        .block_on(async move { TcpListener::from_std(listener).unwrap() });

    let url_string = format!("http://localhost:{}", listener.local_addr().unwrap().port());
    let mut listener = TcpListenerStream::new(listener);

    let url = ServoUrl::parse(&url_string).unwrap();
    let (tx, mut rx) = tokio::sync::oneshot::channel::<()>();

    let server = async move {
        loop {
            let stream = tokio::select! {
                stream = listener.next() => stream,
                _ = &mut rx => break
            };

            let stream = match stream {
                Some(stream) => stream.expect("Could not accept stream: "),
                _ => break,
            };

            let stream = stream.into_std().unwrap();
            stream
                .set_read_timeout(Some(std::time::Duration::new(5, 0)))
                .unwrap();
            let stream = TcpStream::from_std(stream).unwrap();

            let mut tls_server_config =
                SslAcceptor::mozilla_intermediate_v5(SslMethod::tls()).unwrap();
            tls_server_config
                .set_certificate_file(&cert_path, SslFiletype::PEM)
                .unwrap();
            tls_server_config
                .set_private_key_file(&key_path, SslFiletype::PEM)
                .unwrap();

            let tls_server_config = tls_server_config.build();
            let ssl = Ssl::new(tls_server_config.context()).unwrap();
            let mut stream = SslStream::new(ssl, stream).unwrap();

            let _ = Pin::new(&mut stream).accept().await;

            let handler = handler.clone();

            let _ = Http::new()
                .serve_connection(
                    stream,
                    service_fn(move |req: HyperRequest<Body>| {
                        let mut response = HyperResponse::new(Body::empty());
                        handler(req, &mut response);
                        ready(Ok::<_, Infallible>(response))
                    }),
                )
                .await;
        }
    };

    HANDLE.lock().unwrap().spawn(server);

    let server = Server { close_channel: tx };
    (server, url)
}
