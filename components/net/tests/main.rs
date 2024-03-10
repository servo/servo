/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![cfg(test)]
#![allow(dead_code)]

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
use std::fs::File;
use std::io::{self, BufReader};
use std::net::TcpListener as StdTcpListener;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, Weak};

use crossbeam_channel::{unbounded, Sender};
use devtools_traits::DevtoolsControlMsg;
use embedder_traits::{EmbedderProxy, EventLoopWaker};
use futures::future::ready;
use futures::StreamExt;
use hyper::server::conn::Http;
use hyper::server::Server as HyperServer;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request as HyperRequest, Response as HyperResponse};
use lazy_static::lazy_static;
use net::fetch::cors_cache::CorsCache;
use net::fetch::methods::{self, CancellationListener, FetchContext};
use net::filemanager_thread::FileManager;
use net::resource_thread::CoreResourceThreadPool;
use net::test::HttpState;
use net_traits::filemanager_thread::FileTokenCheck;
use net_traits::request::Request;
use net_traits::response::Response;
use net_traits::{FetchTaskTarget, ResourceFetchTiming, ResourceTimingType};
use rustls::{self, Certificate, PrivateKey};
use rustls_pemfile::{certs, pkcs8_private_keys};
use servo_arc::Arc as ServoArc;
use servo_url::ServoUrl;
use tokio::net::{TcpListener, TcpStream};
use tokio::runtime::{Builder, Runtime};
use tokio_rustls::{self, TlsAcceptor};
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
    let sender = fc.unwrap_or_else(|| create_embedder_proxy());

    FetchContext {
        state: Arc::new(HttpState::default()),
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
    pub certificates: Option<Vec<Certificate>>,
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
    (
        Server {
            close_channel: tx,
            certificates: None,
        },
        url,
    )
}

/// Given a path to a file containing PEM certificates, load and parse them into
/// a vector of RusTLS [Certificate]s.
fn load_certificates_from_pem(path: &PathBuf) -> std::io::Result<Vec<Certificate>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let certs = certs(&mut reader)?;
    Ok(certs.into_iter().map(Certificate).collect())
}

/// Given a path to a file containing PEM keys, load and parse them into
/// a vector of RusTLS [PrivateKey]s.
fn load_private_key_from_file(path: &PathBuf) -> Result<PrivateKey, Box<dyn std::error::Error>> {
    let file = File::open(&path)?;
    let mut reader = BufReader::new(file);
    let mut keys = pkcs8_private_keys(&mut reader)?;

    match keys.len() {
        0 => Err(format!("No PKCS8-encoded private key found in {path:?}").into()),
        1 => Ok(PrivateKey(keys.remove(0))),
        _ => Err(format!("More than one PKCS8-encoded private key found in {path:?}").into()),
    }
}

fn make_ssl_server<H>(handler: H) -> (Server, ServoUrl)
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
    let url = ServoUrl::parse(&url_string).unwrap();

    let cert_path = Path::new("../../resources/self_signed_certificate_for_testing.crt")
        .canonicalize()
        .unwrap();
    let key_path = Path::new("../../resources/privatekey_for_testing.key")
        .canonicalize()
        .unwrap();
    let certificates = load_certificates_from_pem(&cert_path).expect("Invalid certificate");
    let key = load_private_key_from_file(&key_path).expect("Invalid key");

    let config = rustls::ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certificates.clone(), key)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))
        .expect("Could not create rustls ServerConfig");
    let acceptor = TlsAcceptor::from(Arc::new(config));

    let mut listener = TcpListenerStream::new(listener);
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

            let handler = handler.clone();
            let acceptor = acceptor.clone();

            let stream = match acceptor.accept(stream).await {
                Ok(stream) => stream,
                Err(_) => {
                    eprintln!("Error handling TLS stream.");
                    continue;
                },
            };

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

    (
        Server {
            close_channel: tx,
            certificates: Some(certificates),
        },
        url,
    )
}
