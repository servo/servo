/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use hyper::header::{Location};
use hyper::server::{Handler, Listening, Server};
use hyper::server::{Request as HyperRequest, Response as HyperResponse};
use hyper::status::StatusCode;
use hyper::uri::RequestUri;
use net::fetch::methods::fetch;
use net_traits::request::{Context, Referer, Request};
use net_traits::response::{Response, ResponseBody};
use std::rc::Rc;
use url::Url;

// TODO write a struct that impls Handler for storing test values

fn make_server<H: Handler + 'static>(handler: H) -> (Listening, Url) {

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
    let handler = move |_: HyperRequest, response: HyperResponse| {
        response.send(MESSAGE).unwrap();
    };
    let (mut server, url) = make_server(handler);

    let mut request = Request::new(url, Context::Fetch, false);
    request.referer = Referer::NoReferer;
    let wrapped_request = Rc::new(request);

    let fetch_response = fetch(wrapped_request, false);
    let _ = server.close();

    if Response::is_network_error(&fetch_response) {
        panic!("fetch response shouldn't be a network error");
    }
}

#[test]
fn test_fetch_response_body_matches_const_message() {

    static MESSAGE: &'static [u8] = b"Hello World!";
    let handler = move |_: HyperRequest, response: HyperResponse| {
        response.send(MESSAGE).unwrap();
    };
    let (mut server, url) = make_server(handler);

    let mut request = Request::new(url, Context::Fetch, false);
    request.referer = Referer::NoReferer;
    let wrapped_request = Rc::new(request);

    let fetch_response = fetch(wrapped_request, false);
    let _ = server.close();

    match fetch_response.body {
        ResponseBody::Done(body) => {
            assert_eq!(body, MESSAGE);
        },
        _ => panic!()
    };
}

fn test_fetch_redirect_count(message: &'static [u8], redirect_cap: u32) -> Response {

    let handler = move |request: HyperRequest, mut response: HyperResponse| {

        let redirects = match request.uri {
            RequestUri::AbsolutePath(url) =>
                url.split("/").collect::<String>().parse::<u32>().unwrap_or(0),
            RequestUri::AbsoluteUri(url) =>
                url.path().unwrap().last().unwrap().split("/").collect::<String>().parse::<u32>().unwrap_or(0),
            _ => panic!()
        };

        if redirects >= redirect_cap {
            response.send(message).unwrap();
        } else {
            *response.status_mut() = StatusCode::Found;
            let url = format!("{redirects}", redirects = redirects + 1);
            response.headers_mut().set(Location(url.to_owned()));
        }
    };

    let (mut server, url) = make_server(handler);

    let mut request = Request::new(url, Context::Fetch, false);
    request.referer = Referer::NoReferer;
    let wrapped_request = Rc::new(request);

    let fetch_response = fetch(wrapped_request, false);
    let _ = server.close();
    fetch_response
}

#[test]
fn test_fetch_redirect_count_ceiling() {

    static MESSAGE: &'static [u8] = b"no more redirects";
    // how many redirects to cause
    let redirect_cap = 20;

    let fetch_response = test_fetch_redirect_count(MESSAGE, redirect_cap);

    assert_eq!(Response::is_network_error(&fetch_response), false);
    match fetch_response.body {
        ResponseBody::Done(body) => {
            assert_eq!(body, MESSAGE);
        },
        _ => panic!()
    };
}

#[test]
fn test_fetch_redirect_count_failure() {

    static MESSAGE: &'static [u8] = b"this message shouldn't be reachable";
    // how many redirects to cause
    let redirect_cap = 21;

    let fetch_response = test_fetch_redirect_count(MESSAGE, redirect_cap);

    assert_eq!(Response::is_network_error(&fetch_response), true);
    match fetch_response.body {
        ResponseBody::Done(_) => panic!(),
        _ => { }
    };
}
