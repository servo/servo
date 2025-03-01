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
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufReader};
use std::net::TcpListener as StdTcpListener;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock, Mutex, RwLock, Weak};

use crossbeam_channel::{unbounded, Receiver, Sender};
use devtools_traits::DevtoolsControlMsg;
use embedder_traits::{AuthenticationResponse, EmbedderMsg, EmbedderProxy, EventLoopWaker};
use futures::future::ready;
use http_body_util::combinators::BoxBody;
use http_body_util::{BodyExt, Empty, Full};
use hyper::body::{Bytes, Incoming};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request as HyperRequest, Response as HyperResponse};
use hyper_util::rt::tokio::TokioIo;
use net::connector::{create_http_client, create_tls_config};
use net::fetch::cors_cache::CorsCache;
use net::fetch::methods::{self, FetchContext};
use net::filemanager_thread::FileManager;
use net::protocols::ProtocolRegistry;
use net::request_interceptor::RequestInterceptor;
use net::resource_thread::CoreResourceThreadPool;
use net::test::HttpState;
use net_traits::filemanager_thread::FileTokenCheck;
use net_traits::request::Request;
use net_traits::response::Response;
use net_traits::{FetchTaskTarget, ResourceFetchTiming, ResourceTimingType};
use rustls_pemfile::{certs, pkcs8_private_keys};
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use servo_arc::Arc as ServoArc;
use servo_url::ServoUrl;
use tokio::net::{TcpListener, TcpStream};
use tokio::runtime::{Builder, Runtime};
use tokio_rustls::{self, TlsAcceptor};

pub static HANDLE: LazyLock<Runtime> = LazyLock::new(|| {
    Builder::new_multi_thread()
        .enable_io()
        .worker_threads(10)
        .build()
        .unwrap()
});

const DEFAULT_USER_AGENT: &'static str = "Such Browser. Very Layout. Wow.";

struct FetchResponseCollector {
    sender: Option<tokio::sync::oneshot::Sender<Response>>,
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

fn create_embedder_proxy_and_receiver() -> (EmbedderProxy, Receiver<EmbedderMsg>) {
    let (sender, receiver) = unbounded();
    let event_loop_waker = || {
        struct DummyEventLoopWaker {}
        impl DummyEventLoopWaker {
            fn new() -> DummyEventLoopWaker {
                DummyEventLoopWaker {}
            }
        }
        impl embedder_traits::EventLoopWaker for DummyEventLoopWaker {
            fn wake(&self) {}
            fn clone_box(&self) -> Box<dyn embedder_traits::EventLoopWaker> {
                Box::new(DummyEventLoopWaker {})
            }
        }

        Box::new(DummyEventLoopWaker::new())
    };

    let embedder_proxy = embedder_traits::EmbedderProxy {
        sender: sender.clone(),
        event_loop_waker: event_loop_waker(),
    };

    (embedder_proxy, receiver)
}

fn receive_credential_prompt_msgs(
    embedder_receiver: Receiver<EmbedderMsg>,
    response: Option<AuthenticationResponse>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || loop {
        let embedder_msg = embedder_receiver.recv().unwrap();
        match embedder_msg {
            embedder_traits::EmbedderMsg::RequestAuthentication(_, _, _, response_sender) => {
                let _ = response_sender.send(response);
                break;
            },
            embedder_traits::EmbedderMsg::WebResourceRequested(..) => {},
            _ => unreachable!(),
        }
    })
}

fn create_http_state(fc: Option<EmbedderProxy>) -> HttpState {
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    let override_manager = net::connector::CertificateErrorOverrideManager::new();
    HttpState {
        hsts_list: RwLock::new(net::hsts::HstsList::default()),
        cookie_jar: RwLock::new(net::cookie_storage::CookieStorage::new(150)),
        auth_cache: RwLock::new(net::resource_thread::AuthCache::default()),
        history_states: RwLock::new(HashMap::new()),
        http_cache: RwLock::new(net::http_cache::HttpCache::default()),
        http_cache_state: Mutex::new(HashMap::new()),
        client: create_http_client(create_tls_config(
            net::connector::CACertificates::Default,
            false, /* ignore_certificate_errors */
            override_manager.clone(),
        )),
        override_manager,
        embedder_proxy: Mutex::new(fc.unwrap_or_else(|| create_embedder_proxy())),
    }
}

fn new_fetch_context(
    dc: Option<Sender<DevtoolsControlMsg>>,
    fc: Option<EmbedderProxy>,
    pool_handle: Option<Weak<CoreResourceThreadPool>>,
) -> FetchContext {
    let sender = fc.unwrap_or_else(|| create_embedder_proxy());

    FetchContext {
        state: Arc::new(create_http_state(Some(sender.clone()))),
        user_agent: DEFAULT_USER_AGENT.into(),
        devtools_chan: dc.map(|dc| Arc::new(Mutex::new(dc))),
        filemanager: Arc::new(Mutex::new(FileManager::new(
            sender.clone(),
            pool_handle.unwrap_or_else(|| Weak::new()),
        ))),
        file_token: FileTokenCheck::NotRequired,
        request_interceptor: Arc::new(Mutex::new(RequestInterceptor::new(sender))),
        cancellation_listener: Arc::new(Default::default()),
        timing: ServoArc::new(Mutex::new(ResourceFetchTiming::new(
            ResourceTimingType::Navigation,
        ))),
        protocols: Arc::new(ProtocolRegistry::with_internal_protocols()),
    }
}
impl FetchTaskTarget for FetchResponseCollector {
    fn process_request_body(&mut self, _: &Request) {}
    fn process_request_eof(&mut self, _: &Request) {}
    fn process_response(&mut self, _: &Request, _: &Response) {}
    fn process_response_chunk(&mut self, _: &Request, _: Vec<u8>) {}
    /// Fired when the response is fully fetched
    fn process_response_eof(&mut self, _: &Request, response: &Response) {
        let _ = self.sender.take().unwrap().send(response.clone());
    }
}

fn fetch(request: Request, dc: Option<Sender<DevtoolsControlMsg>>) -> Response {
    fetch_with_context(request, &mut new_fetch_context(dc, None, None))
}

fn fetch_with_context(request: Request, mut context: &mut FetchContext) -> Response {
    let (sender, receiver) = tokio::sync::oneshot::channel();
    let mut target = FetchResponseCollector {
        sender: Some(sender),
    };
    HANDLE.block_on(async move {
        methods::fetch(request, &mut target, &mut context).await;
        receiver.await.unwrap()
    })
}

fn fetch_with_cors_cache(request: Request, cache: &mut CorsCache) -> Response {
    let (sender, receiver) = tokio::sync::oneshot::channel();
    let mut target = FetchResponseCollector {
        sender: Some(sender),
    };
    HANDLE.block_on(async move {
        methods::fetch_with_cors_cache(
            request,
            cache,
            &mut target,
            &mut new_fetch_context(None, None, None),
        )
        .await;
        receiver.await.unwrap()
    })
}

pub(crate) struct Server {
    pub close_channel: tokio::sync::oneshot::Sender<()>,
    pub certificates: Option<Vec<CertificateDer<'static>>>,
}

impl Server {
    fn close(self) {
        self.close_channel.send(()).expect("err closing server:");
    }
}

fn make_server<H>(handler: H) -> (Server, ServoUrl)
where
    H: Fn(HyperRequest<Incoming>, &mut HyperResponse<BoxBody<Bytes, hyper::Error>>)
        + Send
        + Sync
        + 'static,
{
    let handler = Arc::new(handler);

    let listener = StdTcpListener::bind("0.0.0.0:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let listener = HANDLE.block_on(async move { TcpListener::from_std(listener).unwrap() });

    let url_string = format!("http://localhost:{}", listener.local_addr().unwrap().port());
    let url = ServoUrl::parse(&url_string).unwrap();

    let graceful = hyper_util::server::graceful::GracefulShutdown::new();

    let (tx, mut rx) = tokio::sync::oneshot::channel::<()>();
    let server = async move {
        loop {
            let stream = tokio::select! {
                stream = listener.accept() => stream.unwrap().0,
                _val = &mut rx => {
                    let _ = graceful.shutdown();
                    break;
                }
            };

            let handler = handler.clone();

            let stream = stream.into_std().unwrap();
            stream
                .set_read_timeout(Some(std::time::Duration::new(5, 0)))
                .unwrap();
            let stream = TcpStream::from_std(stream).unwrap();

            let http = http1::Builder::new();
            let conn = http.serve_connection(
                TokioIo::new(stream),
                service_fn(move |req: HyperRequest<Incoming>| {
                    let mut response =
                        HyperResponse::new(Empty::new().map_err(|_| unreachable!()).boxed());
                    handler(req, &mut response);
                    ready(Ok::<_, Infallible>(response))
                }),
            );
            let conn = graceful.watch(conn);
            HANDLE.spawn(async move {
                let _ = conn.await;
            });
        }
    };

    let _ = HANDLE.spawn(server);
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
fn load_certificates_from_pem(path: &PathBuf) -> std::io::Result<Vec<CertificateDer<'static>>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    certs(&mut reader).collect::<Result<Vec<_>, _>>()
}

/// Given a path to a file containing PEM keys, load and parse them into
/// a vector of RusTLS [PrivateKey]s.
fn load_private_key_from_file(
    path: &PathBuf,
) -> Result<PrivateKeyDer<'static>, Box<dyn std::error::Error>> {
    let file = File::open(&path)?;
    let mut reader = BufReader::new(file);
    let mut keys = pkcs8_private_keys(&mut reader).collect::<Result<Vec<_>, _>>()?;

    match keys.len() {
        0 => Err(format!("No PKCS8-encoded private key found in {path:?}").into()),
        1 => Ok(PrivateKeyDer::try_from(keys.remove(0))?),
        _ => Err(format!("More than one PKCS8-encoded private key found in {path:?}").into()),
    }
}

fn make_ssl_server<H>(handler: H) -> (Server, ServoUrl)
where
    H: Fn(HyperRequest<Incoming>, &mut HyperResponse<BoxBody<Bytes, hyper::Error>>)
        + Send
        + Sync
        + 'static,
{
    let handler = Arc::new(handler);
    let listener = StdTcpListener::bind("[::0]:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let listener = HANDLE.block_on(async move { TcpListener::from_std(listener).unwrap() });

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
        .with_no_client_auth()
        .with_single_cert(certificates.clone(), key)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))
        .expect("Could not create rustls ServerConfig");
    let acceptor = TlsAcceptor::from(Arc::new(config));

    let (tx, mut rx) = tokio::sync::oneshot::channel::<()>();
    let server = async move {
        loop {
            let stream = tokio::select! {
                stream = listener.accept() => stream.unwrap().0,
                _ = &mut rx => break
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

            let _ = http1::Builder::new()
                .serve_connection(
                    TokioIo::new(stream),
                    service_fn(move |req: HyperRequest<Incoming>| {
                        let mut response =
                            HyperResponse::new(Empty::new().map_err(|_| unreachable!()).boxed());
                        handler(req, &mut response);
                        ready(Ok::<_, Infallible>(response))
                    }),
                )
                .await;
        }
    };

    HANDLE.spawn(server);

    (
        Server {
            close_channel: tx,
            certificates: Some(certificates),
        },
        url,
    )
}

pub fn make_body(bytes: Vec<u8>) -> BoxBody<Bytes, hyper::Error> {
    Full::new(Bytes::from(bytes))
        .map_err(|_| unreachable!())
        .boxed()
}
