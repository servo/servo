/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![cfg(test)]
#![expect(dead_code)]

mod cookie;
mod cookie_http_state;
mod data_loader;
mod fetch;

fn fetch(request: Request, dc: Option<Sender<DevtoolsControlMsg>>) -> Response {
    fetch_with_context(request, &mut new_fetch_context(dc, None, None))
}
mod file_loader;
mod filemanager_thread;
mod hsts;
mod http_cache;
mod http_loader;
mod image_cache;
mod resource_thread;
mod subresource_integrity;
use std::collections::HashMap;
use std::sync::{Arc, Weak};

use base::threadpool::ThreadPool;
use content_security_policy as csp;
use crossbeam_channel::{Receiver, Sender, unbounded};
use devtools_traits::DevtoolsControlMsg;
use embedder_traits::{AuthenticationResponse, EmbedderMsg, EmbedderProxy};
use net::async_runtime::spawn_blocking_task;
use net::connector::{CACertificates, create_http_client, create_tls_config};
use net::fetch::cors_cache::CorsCache;
use net::fetch::methods::{self, FetchContext};
use net::filemanager_thread::FileManager;
use net::protocols::ProtocolRegistry;
use net::request_interceptor::RequestInterceptor;
use net::test::HttpState;
use net::test_util::{create_embedder_proxy, make_body, make_server, make_ssl_server};
use net_traits::filemanager_thread::FileTokenCheck;
use net_traits::request::Request;
use net_traits::response::Response;
use net_traits::{FetchTaskTarget, ResourceFetchTiming, ResourceTimingType};
use parking_lot::{Mutex, RwLock};
use rustc_hash::FxHashMap;
use servo_arc::Arc as ServoArc;
use servo_url::{ImmutableOrigin, ServoUrl};

const DEFAULT_USER_AGENT: &'static str = "Such Browser. Very Layout. Wow.";

struct FetchResponseCollector {
    sender: Option<tokio::sync::oneshot::Sender<Response>>,
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
    std::thread::spawn(move || {
        loop {
            let embedder_msg = embedder_receiver.recv().unwrap();
            match embedder_msg {
                embedder_traits::EmbedderMsg::RequestAuthentication(_, _, _, response_sender) => {
                    let _ = response_sender.send(response);
                    break;
                },
                embedder_traits::EmbedderMsg::WebResourceRequested(..) => {},
                _ => unreachable!(),
            }
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
        history_states: RwLock::new(FxHashMap::default()),
        http_cache: net::http_cache::HttpCache::default(),
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
    pool_handle: Option<Weak<ThreadPool>>,
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
        websocket_chan: None,
        ca_certificates: CACertificates::Default,
        ignore_certificate_errors: false,
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
    fn process_csp_violations(&mut self, _: &Request, _: Vec<csp::Violation>) {}
}

fn fetch_with_context(request: Request, mut context: &mut FetchContext) -> Response {
    let (sender, receiver) = tokio::sync::oneshot::channel();
    let mut target = FetchResponseCollector {
        sender: Some(sender),
    };
    spawn_blocking_task::<_, Response>(async move {
        methods::fetch(request, &mut target, &mut context).await;
        receiver.await.unwrap()
    })
}

fn fetch_with_cors_cache(request: Request, cache: &mut CorsCache) -> Response {
    let (sender, receiver) = tokio::sync::oneshot::channel();
    let mut target = FetchResponseCollector {
        sender: Some(sender),
    };
    let mut fetch_context = new_fetch_context(None, None, None);
    spawn_blocking_task::<_, Response>(async move {
        methods::fetch_with_cors_cache(request, cache, &mut target, &mut fetch_context).await;
        receiver.await.unwrap()
    })
}

pub(crate) fn mock_origin() -> ImmutableOrigin {
    ServoUrl::parse("http://servo.org").unwrap().origin()
}
