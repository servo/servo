/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(plugin)]
#![plugin(plugins)]

extern crate content_blocker;
extern crate cookie as cookie_rs;
extern crate devtools_traits;
extern crate flate2;
extern crate hyper;
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

#[cfg(test)] mod chrome_loader;
#[cfg(test)] mod cookie;
#[cfg(test)] mod cookie_http_state;
#[cfg(test)] mod data_loader;
#[cfg(test)] mod file_loader;
#[cfg(test)] mod fetch;
#[cfg(test)] mod mime_classifier;
#[cfg(test)] mod resource_thread;
#[cfg(test)] mod hsts;
#[cfg(test)] mod http_loader;
#[cfg(test)] mod filemanager_thread;

use devtools_traits::DevtoolsControlMsg;
use hyper::server::{Handler, Listening, Server};
use net::fetch::cors_cache::CorsCache;
use net::fetch::methods::{self, FetchContext};
use net::filemanager_thread::FileManager;
use net::test::HttpState;
use net_traits::FetchTaskTarget;
use net_traits::request::Request;
use net_traits::response::Response;
use servo_url::ServoUrl;
use std::rc::Rc;
use std::sync::mpsc::{Sender, channel};

const DEFAULT_USER_AGENT: &'static str = "Such Browser. Very Layout. Wow.";

struct FetchResponseCollector {
    sender: Sender<Response>,
}

fn new_fetch_context(dc: Option<Sender<DevtoolsControlMsg>>) -> FetchContext {
    FetchContext {
        state: HttpState::new("certs"),
        user_agent: DEFAULT_USER_AGENT.into(),
        devtools_chan: dc,
        filemanager: FileManager::new(),
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

fn fetch(request: Request, dc: Option<Sender<DevtoolsControlMsg>>) -> Response {
    fetch_with_context(request, &new_fetch_context(dc))
}

fn fetch_with_context(request: Request, context: &FetchContext) -> Response {
    let (sender, receiver) = channel();
    let mut target = FetchResponseCollector {
        sender: sender,
    };

    methods::fetch(Rc::new(request), &mut target, context);

    receiver.recv().unwrap()
}

fn fetch_with_cors_cache(request: Rc<Request>, cache: &mut CorsCache) -> Response {
    let (sender, receiver) = channel();
    let mut target = FetchResponseCollector {
        sender: sender,
    };

    methods::fetch_with_cors_cache(request, cache, &mut target, &new_fetch_context(None));

    receiver.recv().unwrap()
}

fn make_server<H: Handler + 'static>(handler: H) -> (Listening, ServoUrl) {
    // this is a Listening server because of handle_threads()
    let server = Server::http("0.0.0.0:0").unwrap().handle_threads(handler, 1).unwrap();
    let url_string = format!("http://localhost:{}", server.socket.port());
    let url = ServoUrl::parse(&url_string).unwrap();
    (server, url)
}
