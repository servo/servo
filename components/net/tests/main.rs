/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![cfg(test)]

extern crate cookie as cookie_rs;
extern crate devtools_traits;
extern crate embedder_traits;
extern crate flate2;
extern crate futures;
extern crate headers_core;
extern crate headers_ext;
extern crate http;
extern crate hyper;
extern crate hyper_serde;
extern crate ipc_channel;
#[macro_use]
extern crate lazy_static;
extern crate mime;
extern crate msg;
extern crate net;
extern crate net_traits;
extern crate openssl;
extern crate profile_traits;
extern crate servo_channel;
extern crate servo_config;
extern crate servo_url;
extern crate time;
extern crate tokio;
extern crate tokio_openssl;
extern crate url;

mod cookie;
mod cookie_http_state;
mod data_loader;
mod fetch;
mod file_loader;
mod filemanager_thread;
mod hsts;
mod http_loader;
mod mime_classifier;
mod resource_thread;
mod subresource_integrity;

use devtools_traits::DevtoolsControlMsg;
use embedder_traits::{EmbedderProxy, EventLoopWaker};
use embedder_traits::resources::{self, Resource};
use futures::{Future, Stream};
use hyper::{Body, Request as HyperRequest, Response as HyperResponse};
use hyper::server::Server as HyperServer;
use hyper::server::conn::Http;
use hyper::service::service_fn_ok;
use net::connector::create_ssl_connector_builder;
use net::fetch::cors_cache::CorsCache;
use net::fetch::methods::{self, CancellationListener, FetchContext};
use net::filemanager_thread::FileManager;
use net::test::HttpState;
use net_traits::FetchTaskTarget;
use net_traits::request::Request;
use net_traits::response::Response;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use servo_channel::{channel, Sender};
use servo_url::ServoUrl;
use std::net::TcpListener as StdTcpListener;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::runtime::Runtime;
use tokio_openssl::SslAcceptorExt;

lazy_static! {
        pub static ref HANDLE: Mutex<Runtime> = {
            Mutex::new(Runtime::new().unwrap())
        };
}

const DEFAULT_USER_AGENT: &'static str = "Such Browser. Very Layout. Wow.";

struct FetchResponseCollector {
    sender: Sender<Response>,
}

fn create_embedder_proxy() -> EmbedderProxy {
    let (sender, _) = channel();
    let event_loop_waker = | | {
        struct DummyEventLoopWaker {
        }
        impl DummyEventLoopWaker {
            fn new() -> DummyEventLoopWaker {
                DummyEventLoopWaker { }
            }
        }
        impl EventLoopWaker for DummyEventLoopWaker {
            fn wake(&self) { }
            fn clone(&self) -> Box<EventLoopWaker + Send> {
                Box::new(DummyEventLoopWaker { })
            }
        }

        Box::new(DummyEventLoopWaker::new())
    };

    EmbedderProxy {
        sender: sender,
        event_loop_waker: event_loop_waker()
    }
}

fn new_fetch_context(dc: Option<Sender<DevtoolsControlMsg>>, fc: Option<EmbedderProxy>) -> FetchContext {
    let ssl_connector = create_ssl_connector_builder(&resources::read_string(Resource::SSLCertificates));
    let sender = fc.unwrap_or_else(|| create_embedder_proxy());
    FetchContext {
        state: Arc::new(HttpState::new(ssl_connector)),
        user_agent: DEFAULT_USER_AGENT.into(),
        devtools_chan: dc,
        filemanager: FileManager::new(sender),
        cancellation_listener: Arc::new(Mutex::new(CancellationListener::new(None))),
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
    fetch_with_context(request, &new_fetch_context(dc, None))
}

fn fetch_with_context(request: &mut Request, context: &FetchContext) -> Response {
    let (sender, receiver) = channel();
    let mut target = FetchResponseCollector {
        sender: sender,
    };

    methods::fetch(request, &mut target, context);

    receiver.recv().unwrap()
}

fn fetch_with_cors_cache(request: &mut Request, cache: &mut CorsCache) -> Response {
    let (sender, receiver) = channel();
    let mut target = FetchResponseCollector {
        sender: sender,
    };

    methods::fetch_with_cors_cache(request, cache, &mut target, &new_fetch_context(None, None));

    receiver.recv().unwrap()
}

pub(crate) struct Server {
    pub close_channel: futures::sync::oneshot::Sender<()>,
}

impl Server {
    fn close(self) {
        self.close_channel.send(()).unwrap();
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
    let (tx, rx) = futures::sync::oneshot::channel::<()>();
    let server = HyperServer::from_tcp(listener).unwrap().serve(
        move || {
            let handler = handler.clone();
            service_fn_ok(move |req: HyperRequest<Body>| {
                let mut response = HyperResponse::new(Vec::<u8>::new().into());
                handler(req, &mut response);
                response
            })
        }
    )
    .with_graceful_shutdown(rx)
    .map_err(|_|());

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
    let listener = TcpListener::from_std(listener, &HANDLE.lock().unwrap().reactor()).unwrap();
    let url_string = format!("http://localhost:{}", listener.local_addr().unwrap().port());
    let url = ServoUrl::parse(&url_string).unwrap();

    let server = listener.incoming()
        .map_err(|_| ())
        .for_each(move |sock| {
            let mut ssl_builder = SslAcceptor::mozilla_modern(SslMethod::tls()).unwrap();
            ssl_builder.set_certificate_file(&cert_path, SslFiletype::PEM).unwrap();
            ssl_builder.set_private_key_file(&key_path, SslFiletype::PEM).unwrap();

            let handler = handler.clone();
            ssl_builder.build().accept_async(sock).map_err(|_| ()).and_then(move |ssl| {
                Http::new().serve_connection(ssl,
                        service_fn_ok(move |req: HyperRequest<Body>| {
                            let mut response = HyperResponse::new(Vec::<u8>::new().into());
                            handler(req, &mut response);
                            response
                        })
                )
                .map_err(|_|())
            })
        });

    let (tx, rx) = futures::sync::oneshot::channel::<()>();
    let server = server.select(rx.map_err(|_| ())).map(|_| ()).map_err(|_| ());

    HANDLE.lock().unwrap().spawn(server);

    let server = Server { close_channel: tx };
    (server, url)
}
