/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use hyper::server::{Listening, Server};
use hyper::server::{Request as HyperRequest, Response as HyperResponse};
use net::fetch::request::{Context, fetch, Referer, Request};
use net_traits::response::{Response};
use std::rc::Rc;
use url::Url;

fn make_server(message: &'static [u8]) -> (Listening, Url) {

    let handler = move | _: HyperRequest, response: HyperResponse | {
        response.send(message).unwrap();
    };

    // this is a Listening server because of handle_threads()
    let server = Server::http("0.0.0.0:0").unwrap().handle_threads(handler, 1).unwrap();
    let port = server.socket.port().to_string();
    let mut url_string = "http://localhost:".to_owned();
    url_string.push_str(&port);
    let url = Url::parse(&url_string).unwrap();
    (server, url)
}


#[test]
fn test_fetch_response_is_not_network_error() {

    static MESSAGE: &'static [u8] = b"";
    let (mut server, url) = make_server(MESSAGE);

    let mut request = Request::new(url, Context::Fetch, false);
    request.referer = Referer::NoReferer;
    let wrapped_request = Rc::new(request);

    let fetch_response = fetch(wrapped_request, false);
    let _ = server.close();

    if Response::is_network_error(&fetch_response) {
        panic!("fetch response shouldn't be a network error");
    }
}
