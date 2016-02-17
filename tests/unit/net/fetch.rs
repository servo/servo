/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use hyper::header::{AccessControlAllowHeaders, AccessControlAllowOrigin};
use hyper::header::{CacheControl, ContentLanguage, ContentType, Expires, LastModified};
use hyper::header::{Headers, HttpDate, Location, SetCookie, Pragma};
use hyper::server::{Handler, Listening, Server};
use hyper::server::{Request as HyperRequest, Response as HyperResponse};
use hyper::status::StatusCode;
use hyper::uri::RequestUri;
use net::fetch::methods::fetch;
use net_traits::request::{RedirectMode, Referer, Request, RequestMode};
use net_traits::response::{CacheState, Response, ResponseBody, ResponseType};
use std::cell::Cell;
use std::rc::Rc;
use time::{self, Duration};
use unicase::UniCase;
use url::{Origin, OpaqueOrigin, Url};

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

    let mut request = Request::new(url, false);
    request.referer = Referer::NoReferer;
    let wrapped_request = Rc::new(request);

    let fetch_response = fetch(wrapped_request);
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

    let origin = url.origin();
    let mut request = Request::new(url, false);
    request.referer = Referer::NoReferer;
    let wrapped_request = Rc::new(request);

    let fetch_response = fetch(wrapped_request);
    let _ = server.close();

    assert!(!Response::is_network_error(&fetch_response));
    assert_eq!(fetch_response.response_type, ResponseType::Basic);

    match *fetch_response.body.borrow() {
        ResponseBody::Done(ref body) => {
            assert_eq!(&**body, MESSAGE);
        },
        _ => panic!()
    };
}

#[test]
fn test_fetch_response_is_basic_filtered() {

    static MESSAGE: &'static [u8] = b"";
    let handler = move |_: HyperRequest, mut response: HyperResponse| {

        response.headers_mut().set(SetCookie(vec![]));
        // this header is obsoleted, so hyper doesn't implement it, but it's still covered by the spec
        response.headers_mut().set_raw("Set-Cookie2", vec![]);

        response.send(MESSAGE).unwrap();
    };
    let (mut server, url) = make_server(handler);

    let origin = url.origin();
    let mut request = Request::new(url, false);
    request.referer = Referer::NoReferer;
    let wrapped_request = Rc::new(request);

    let fetch_response = fetch(wrapped_request);
    let _ = server.close();

    assert!(!Response::is_network_error(&fetch_response));
    assert_eq!(fetch_response.response_type, ResponseType::Basic);

    let headers = fetch_response.headers;
    assert!(!headers.has::<SetCookie>());
    assert!(headers.get_raw("Set-Cookie2").is_none());
}

#[test]
fn test_fetch_response_is_cors_filtered() {

    static MESSAGE: &'static [u8] = b"";
    let handler = move |_: HyperRequest, mut response: HyperResponse| {

        // this is mandatory for the Cors Check to pass
        // TODO test using different url encodings with this value ie. punycode
        response.headers_mut().set(AccessControlAllowOrigin::Any);

        // these are the headers that should be kept after filtering
        response.headers_mut().set(CacheControl(vec![]));
        response.headers_mut().set(ContentLanguage(vec![]));
        response.headers_mut().set(ContentType::html());
        response.headers_mut().set(Expires(HttpDate(time::now() + Duration::days(1))));
        response.headers_mut().set(LastModified(HttpDate(time::now())));
        response.headers_mut().set(Pragma::NoCache);

        // these headers should not be kept after filtering, even though they are given a pass
        response.headers_mut().set(SetCookie(vec![]));
        response.headers_mut().set_raw("Set-Cookie2", vec![]);
        response.headers_mut().set(
            AccessControlAllowHeaders(vec![
                UniCase("set-cookie".to_owned()),
                UniCase("set-cookie2".to_owned())
            ])
        );

        response.send(MESSAGE).unwrap();
    };
    let (mut server, url) = make_server(handler);

    // an origin mis-match will stop it from defaulting to a basic filtered response
    let origin = Origin::UID(OpaqueOrigin::new());
    let mut request = Request::new(url, false);
    request.referer = Referer::NoReferer;
    request.mode = RequestMode::CORSMode;
    let wrapped_request = Rc::new(request);

    let fetch_response = fetch(wrapped_request);
    let _ = server.close();

    assert!(!Response::is_network_error(&fetch_response));
    assert_eq!(fetch_response.response_type, ResponseType::CORS);

    let headers = fetch_response.headers;
    assert!(headers.has::<CacheControl>());
    assert!(headers.has::<ContentLanguage>());
    assert!(headers.has::<ContentType>());
    assert!(headers.has::<Expires>());
    assert!(headers.has::<LastModified>());
    assert!(headers.has::<Pragma>());

    assert!(!headers.has::<AccessControlAllowOrigin>());
    assert!(!headers.has::<SetCookie>());
    assert!(headers.get_raw("Set-Cookie2").is_none());
}

#[test]
fn test_fetch_response_is_opaque_filtered() {

    static MESSAGE: &'static [u8] = b"";
    let handler = move |_: HyperRequest, response: HyperResponse| {
        response.send(MESSAGE).unwrap();
    };
    let (mut server, url) = make_server(handler);

    // an origin mis-match will fall through to an Opaque filtered response
    let origin = Origin::UID(OpaqueOrigin::new());
    let mut request = Request::new(url, false);
    request.referer = Referer::NoReferer;
    let wrapped_request = Rc::new(request);

    let fetch_response = fetch(wrapped_request);
    let _ = server.close();

    assert!(!Response::is_network_error(&fetch_response));
    assert_eq!(fetch_response.response_type, ResponseType::Opaque);

    assert!(fetch_response.url_list.into_inner().len() == 0);
    assert!(fetch_response.url.is_none());
    // this also asserts that status message is "the empty byte sequence"
    assert!(fetch_response.status.is_none());
    assert_eq!(fetch_response.headers, Headers::new());
    match fetch_response.body.into_inner() {
        ResponseBody::Empty => { },
        _ => panic!()
    }
    match fetch_response.cache_state {
        CacheState::None => { },
        _ => panic!()
    }
}

#[test]
fn test_fetch_response_is_opaque_redirect_filtered() {

    static MESSAGE: &'static [u8] = b"";
    let handler = move |request: HyperRequest, mut response: HyperResponse| {

        let redirects = match request.uri {
            RequestUri::AbsolutePath(url) =>
                url.split("/").collect::<String>().parse::<u32>().unwrap_or(0),
            RequestUri::AbsoluteUri(url) =>
                url.path().unwrap().last().unwrap().split("/").collect::<String>().parse::<u32>().unwrap_or(0),
            _ => panic!()
        };

        if redirects == 1 {
            response.send(MESSAGE).unwrap();
        } else {
            *response.status_mut() = StatusCode::Found;
            let url = format!("{}", 1);
            response.headers_mut().set(Location(url.to_owned()));
        }
    };

    let (mut server, url) = make_server(handler);

    let origin = url.origin();
    let mut request = Request::new(url, false);
    request.referer = Referer::NoReferer;
    request.redirect_mode = Cell::new(RedirectMode::Manual);
    let wrapped_request = Rc::new(request);

    let fetch_response = fetch(wrapped_request);
    let _ = server.close();

    assert!(!Response::is_network_error(&fetch_response));
    assert_eq!(fetch_response.response_type, ResponseType::OpaqueRedirect);

    // this also asserts that status message is "the empty byte sequence"
    assert!(fetch_response.status.is_none());
    assert_eq!(fetch_response.headers, Headers::new());
    match fetch_response.body.into_inner() {
        ResponseBody::Empty => { },
        _ => panic!()
    }
    match fetch_response.cache_state {
        CacheState::None => { },
        _ => panic!()
    }
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

    let origin = url.origin();
    let mut request = Request::new(url, false);
    request.referer = Referer::NoReferer;
    let wrapped_request = Rc::new(request);

    let fetch_response = fetch(wrapped_request);
    let _ = server.close();
    fetch_response
}

#[test]
fn test_fetch_redirect_count_ceiling() {

    static MESSAGE: &'static [u8] = b"no more redirects";
    // how many redirects to cause
    let redirect_cap = 20;

    let fetch_response = test_fetch_redirect_count(MESSAGE, redirect_cap);

    assert!(!Response::is_network_error(&fetch_response));
    assert_eq!(fetch_response.response_type, ResponseType::Basic);

    match *fetch_response.body.borrow() {
        ResponseBody::Done(ref body) => {
            assert_eq!(&**body, MESSAGE);
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

    assert!(Response::is_network_error(&fetch_response));

    match *fetch_response.body.borrow() {
        ResponseBody::Done(_) | ResponseBody::Receiving(_) => panic!(),
        _ => { }
    };
}
