/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

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

#[test]
#[should_panic]
fn test_fetch_redirect_count() {
    
    static MESSAGE: &'static [u8] = b"no more redirects";
    static SERVER_HOST: &'static str = "http://localhost:8000";
    let redirect_cap = 2;

    let handler = move |request: HyperRequest, mut response: HyperResponse| {
        
        let redirects = match request.uri {
            RequestUri::AbsolutePath(url) =>
                url.split("/").collect::<String>().parse::<u32>().unwrap_or(0) + 1,
            RequestUri::AbsoluteUri(url) =>
                url.path().unwrap().last().unwrap().split("/").collect::<String>().parse::<u32>().unwrap_or(0) + 1,
            _ => panic!()
        };

        if redirects == redirect_cap {
            response.send(MESSAGE).unwrap();
        } else {
            *response.status_mut() = StatusCode::Found;
            let url = format!("{host}/{redirects}", host = SERVER_HOST, redirects = redirects);
            // set response Location header to url
        }
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
