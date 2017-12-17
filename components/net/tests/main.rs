/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![cfg(test)]

extern crate cookie as cookie_rs;
extern crate crossbeam_channel;
extern crate devtools_traits;
extern crate embedder_traits;
extern crate flate2;
extern crate hyper;
extern crate hyper_openssl;
extern crate hyper_serde;
extern crate ipc_channel;
extern crate msg;
extern crate net;
extern crate net_traits;
extern crate profile_traits;
extern crate servo_config;
extern crate servo_url;
extern crate time;
extern crate unicase;
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

use crossbeam_channel::Sender;
use devtools_traits::DevtoolsControlMsg;
use embedder_traits::{EmbedderProxy, EventLoopWaker};
use embedder_traits::resources::{self, Resource};
use hyper::server::{Handler, Listening, Server};
use net::connector::create_ssl_client;
use net::fetch::cors_cache::CorsCache;
use net::fetch::methods::{self, CancellationListener, FetchContext};
use net::filemanager_thread::FileManager;
use net::test::HttpState;
use net_traits::FetchTaskTarget;
use net_traits::request::Request;
use net_traits::response::Response;
use servo_url::ServoUrl;
use std::sync::{Arc, Mutex};

const DEFAULT_USER_AGENT: &'static str = "Such Browser. Very Layout. Wow.";

struct FetchResponseCollector {
    sender: Sender<Response>,
}

fn create_embedder_proxy() -> EmbedderProxy {
    let (sender, _) = crossbeam_channel::unbounded();
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
    let ssl_client = create_ssl_client(&resources::read_string(Resource::SSLCertificates));
    let sender = fc.unwrap_or_else(|| create_embedder_proxy());
    FetchContext {
        state: Arc::new(HttpState::new(ssl_client)),
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
    let (sender, receiver) = crossbeam_channel::unbounded();
    let mut target = FetchResponseCollector {
        sender: sender,
    };

    methods::fetch(request, &mut target, context);

    receiver.recv().unwrap()
}

fn fetch_with_cors_cache(request: &mut Request, cache: &mut CorsCache) -> Response {
    let (sender, receiver) = crossbeam_channel::unbounded();
    let mut target = FetchResponseCollector {
        sender: sender,
    };

    methods::fetch_with_cors_cache(request, cache, &mut target, &new_fetch_context(None, None));

    receiver.recv().unwrap()
}

fn make_server<H: Handler + 'static>(handler: H) -> (Listening, ServoUrl) {
    // this is a Listening server because of handle_threads()
    let server = Server::http("0.0.0.0:0").unwrap().handle_threads(handler, 2).unwrap();
    let url_string = format!("http://localhost:{}", server.socket.port());
    let url = ServoUrl::parse(&url_string).unwrap();
    (server, url)
}
