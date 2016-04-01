/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use hyper::header::{AccessControlAllowHeaders, AccessControlAllowOrigin};
use hyper::header::{CacheControl, ContentLanguage, ContentType, Expires, LastModified};
use hyper::header::{Headers, HttpDate, Location, SetCookie, Pragma};
use hyper::method::Method;
use hyper::mime::{Mime, TopLevel, SubLevel};
use hyper::server::{Handler, Listening, Server};
use hyper::server::{Request as HyperRequest, Response as HyperResponse};
use hyper::status::StatusCode;
use hyper::uri::RequestUri;
use net::fetch::methods::{fetch, fetch_async};
use net_traits::AsyncFetchListener;
use net_traits::request::{Origin, RedirectMode, Referer, Request, RequestMode};
use net_traits::response::{CacheState, Response, ResponseBody, ResponseType};
use std::rc::Rc;
use std::sync::mpsc::{Sender, channel};
use std::sync::{Arc, Mutex};
use time::{self, Duration};
use unicase::UniCase;
use url::{Origin as UrlOrigin, OpaqueOrigin, Url};

// TODO write a struct that impls Handler for storing test values

struct FetchResponseCollector {
    sender: Sender<Response>,
}

impl AsyncFetchListener for FetchResponseCollector {
    fn response_available(&self, response: Response) {
        let _ = self.sender.send(response);
    }
}

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

    let origin = Origin::Origin(url.origin());
    let mut request = Request::new(url, Some(origin), false);
    request.referer = Referer::NoReferer;
    let wrapped_request = Rc::new(request);

    let fetch_response = fetch(wrapped_request);
    let _ = server.close();

    if fetch_response.is_network_error() {
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

    let origin = Origin::Origin(url.origin());
    let mut request = Request::new(url, Some(origin), false);
    request.referer = Referer::NoReferer;
    let wrapped_request = Rc::new(request);

    let fetch_response = fetch(wrapped_request);
    let _ = server.close();

    assert!(!fetch_response.is_network_error());
    assert_eq!(fetch_response.response_type, ResponseType::Basic);

    match *fetch_response.body.lock().unwrap() {
        ResponseBody::Done(ref body) => {
            assert_eq!(&**body, MESSAGE);
        },
        _ => panic!()
    };
}

#[test]
fn test_fetch_aboutblank() {

    let url = Url::parse("about:blank").unwrap();
    let origin = Origin::Origin(url.origin());
    let mut request = Request::new(url, Some(origin), false);
    request.referer = Referer::NoReferer;
    let wrapped_request = Rc::new(request);

    let fetch_response = fetch(wrapped_request);
    assert!(!fetch_response.is_network_error());
    assert!(*fetch_response.body.lock().unwrap() == ResponseBody::Done(vec![]));
}

#[test]
fn test_fetch_data() {

    let url = Url::parse("data:text/html,<p>Servo</p>").unwrap();
    let origin = Origin::Origin(url.origin());
    let request = Request::new(url, Some(origin), false);
    request.same_origin_data.set(true);
    let expected_resp_body = "<p>Servo</p>".to_owned();
    let fetch_response = fetch(Rc::new(request));

    assert!(!fetch_response.is_network_error());
    assert_eq!(fetch_response.headers.len(), 1);
    let content_type: &ContentType = fetch_response.headers.get().unwrap();
    assert!(**content_type == Mime(TopLevel::Text, SubLevel::Html, vec![]));
    let resp_body = fetch_response.body.lock().unwrap();

    match *resp_body {
        ResponseBody::Done(ref val) => {
            assert_eq!(val, &expected_resp_body.into_bytes());
        }
        ResponseBody::Receiving(_) => {
            panic!();
        },
        ResponseBody::Empty => panic!(),
    }
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

    let origin = Origin::Origin(url.origin());
    let mut request = Request::new(url, Some(origin), false);
    request.referer = Referer::NoReferer;
    let wrapped_request = Rc::new(request);

    let fetch_response = fetch(wrapped_request);
    let _ = server.close();

    assert!(!fetch_response.is_network_error());
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
    let origin = Origin::Origin(UrlOrigin::UID(OpaqueOrigin::new()));
    let mut request = Request::new(url, Some(origin), false);
    request.referer = Referer::NoReferer;
    request.mode = RequestMode::CORSMode;
    let wrapped_request = Rc::new(request);

    let fetch_response = fetch(wrapped_request);
    let _ = server.close();

    assert!(!fetch_response.is_network_error());
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
    let origin = Origin::Origin(UrlOrigin::UID(OpaqueOrigin::new()));
    let mut request = Request::new(url, Some(origin), false);
    request.referer = Referer::NoReferer;
    let wrapped_request = Rc::new(request);

    let fetch_response = fetch(wrapped_request);
    let _ = server.close();

    assert!(!fetch_response.is_network_error());
    assert_eq!(fetch_response.response_type, ResponseType::Opaque);

    assert!(fetch_response.url_list.into_inner().len() == 0);
    assert!(fetch_response.url.is_none());
    // this also asserts that status message is "the empty byte sequence"
    assert!(fetch_response.status.is_none());
    assert_eq!(fetch_response.headers, Headers::new());
    match *fetch_response.body.lock().unwrap() {
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

    let origin = Origin::Origin(url.origin());
    let mut request = Request::new(url, Some(origin), false);
    request.referer = Referer::NoReferer;
    request.redirect_mode.set(RedirectMode::Manual);
    let wrapped_request = Rc::new(request);

    let fetch_response = fetch(wrapped_request);
    let _ = server.close();

    assert!(!fetch_response.is_network_error());
    assert_eq!(fetch_response.response_type, ResponseType::OpaqueRedirect);

    // this also asserts that status message is "the empty byte sequence"
    assert!(fetch_response.status.is_none());
    assert_eq!(fetch_response.headers, Headers::new());
    match *fetch_response.body.lock().unwrap() {
        ResponseBody::Empty => { },
        _ => panic!()
    }
    match fetch_response.cache_state {
        CacheState::None => { },
        _ => panic!()
    }
}

#[test]
fn test_fetch_with_local_urls_only() {
    // If flag `local_urls_only` is set, fetching a non-local URL must result in network error.

    static MESSAGE: &'static [u8] = b"";
    let handler = move |_: HyperRequest, response: HyperResponse| {
        response.send(MESSAGE).unwrap();
    };
    let (mut server, server_url) = make_server(handler);

    let do_fetch = |url: Url| {
        let origin = Origin::Origin(url.origin());
        let mut request = Request::new(url, Some(origin), false);
        request.referer = Referer::NoReferer;

        // Set the flag.
        request.local_urls_only = true;

        let wrapped_request = Rc::new(request);
        fetch(wrapped_request)
    };

    let local_url = Url::parse("about:blank").unwrap();
    let local_response = do_fetch(local_url);
    let server_response = do_fetch(server_url);

    let _ = server.close();

    assert!(!local_response.is_network_error());
    assert!(server_response.is_network_error());
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

    let origin = Origin::Origin(url.origin());
    let mut request = Request::new(url, Some(origin), false);
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

    assert!(!fetch_response.is_network_error());
    assert_eq!(fetch_response.response_type, ResponseType::Basic);

    match *fetch_response.body.lock().unwrap() {
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

    assert!(fetch_response.is_network_error());

    match *fetch_response.body.lock().unwrap() {
        ResponseBody::Done(_) | ResponseBody::Receiving(_) => panic!(),
        _ => { }
    };
}

fn test_fetch_redirect_updates_method_runner(tx: Sender<bool>, status_code: StatusCode, method: Method) {

    let handler_method = method.clone();
    let handler_tx = Arc::new(Mutex::new(tx));

    let handler = move |request: HyperRequest, mut response: HyperResponse| {

        let redirects = match request.uri {
            RequestUri::AbsolutePath(url) =>
                url.split("/").collect::<String>().parse::<u32>().unwrap_or(0),
            RequestUri::AbsoluteUri(url) =>
                url.path().unwrap().last().unwrap().split("/").collect::<String>().parse::<u32>().unwrap_or(0),
            _ => panic!()
        };

        let mut test_pass = true;

        if redirects == 0 {

            *response.status_mut() = StatusCode::TemporaryRedirect;
            response.headers_mut().set(Location("1".to_owned()));

        } else if redirects == 1 {

            // this makes sure that the request method does't change from the wrong status code
            if handler_method != Method::Get && request.method == Method::Get {
                test_pass = false;
            }
            *response.status_mut() = status_code;
            response.headers_mut().set(Location("2".to_owned()));

        } else if request.method != Method::Get {
            test_pass = false;
        }

        // the first time this handler is reached, nothing is being tested, so don't send anything
        if redirects > 0 {
            handler_tx.lock().unwrap().send(test_pass).unwrap();
        }

    };

    let (mut server, url) = make_server(handler);

    let origin = Origin::Origin(url.origin());
    let mut request = Request::new(url, Some(origin), false);
    request.referer = Referer::NoReferer;
    *request.method.borrow_mut() = method;
    let wrapped_request = Rc::new(request);

    let _ = fetch(wrapped_request);
    let _ = server.close();
}

#[test]
fn test_fetch_redirect_updates_method() {

    let (tx, rx) = channel();

    test_fetch_redirect_updates_method_runner(tx.clone(), StatusCode::MovedPermanently, Method::Post);
    assert_eq!(rx.recv().unwrap(), true);
    assert_eq!(rx.recv().unwrap(), true);
    // make sure the test doesn't send more data than expected
    assert_eq!(rx.try_recv().is_err(), true);

    test_fetch_redirect_updates_method_runner(tx.clone(), StatusCode::Found, Method::Post);
    assert_eq!(rx.recv().unwrap(), true);
    assert_eq!(rx.recv().unwrap(), true);
    assert_eq!(rx.try_recv().is_err(), true);

    test_fetch_redirect_updates_method_runner(tx.clone(), StatusCode::SeeOther, Method::Get);
    assert_eq!(rx.recv().unwrap(), true);
    assert_eq!(rx.recv().unwrap(), true);
    assert_eq!(rx.try_recv().is_err(), true);

    let extension = Method::Extension("FOO".to_owned());

    test_fetch_redirect_updates_method_runner(tx.clone(), StatusCode::MovedPermanently, extension.clone());
    assert_eq!(rx.recv().unwrap(), true);
    // for MovedPermanently and Found, Method should only be changed if it was Post
    assert_eq!(rx.recv().unwrap(), false);
    assert_eq!(rx.try_recv().is_err(), true);

    test_fetch_redirect_updates_method_runner(tx.clone(), StatusCode::Found, extension.clone());
    assert_eq!(rx.recv().unwrap(), true);
    assert_eq!(rx.recv().unwrap(), false);
    assert_eq!(rx.try_recv().is_err(), true);

    test_fetch_redirect_updates_method_runner(tx.clone(), StatusCode::SeeOther, extension.clone());
    assert_eq!(rx.recv().unwrap(), true);
    // for SeeOther, Method should always be changed, so this should be true
    assert_eq!(rx.recv().unwrap(), true);
    assert_eq!(rx.try_recv().is_err(), true);
}

fn response_is_done(response: &Response) -> bool {

    let response_complete = match response.response_type {
        ResponseType::Default | ResponseType::Basic | ResponseType::CORS => {
            (*response.body.lock().unwrap()).is_done()
        }
        // if the internal response cannot have a body, it shouldn't block the "done" state
        ResponseType::Opaque | ResponseType::OpaqueRedirect | ResponseType::Error => true
    };

    let internal_complete = if let Some(ref res) = response.internal_response {
        res.body.lock().unwrap().is_done()
    } else {
        true
    };

    response_complete && internal_complete
}

#[test]
fn test_fetch_async_returns_complete_response() {

    static MESSAGE: &'static [u8] = b"this message should be retrieved in full";
    let handler = move |_: HyperRequest, response: HyperResponse| {
        response.send(MESSAGE).unwrap();
    };
    let (mut server, url) = make_server(handler);

    let origin = Origin::Origin(url.origin());
    let mut request = Request::new(url, Some(origin), false);
    request.referer = Referer::NoReferer;

    let (tx, rx) = channel();
    let listener = Box::new(FetchResponseCollector {
        sender: tx.clone()
    });

    fetch_async(request, listener);
    let fetch_response = rx.recv().unwrap();
    let _ = server.close();

    assert_eq!(response_is_done(&fetch_response), true);
}

#[test]
fn test_opaque_filtered_fetch_async_returns_complete_response() {

    static MESSAGE: &'static [u8] = b"";
    let handler = move |_: HyperRequest, response: HyperResponse| {
        response.send(MESSAGE).unwrap();
    };
    let (mut server, url) = make_server(handler);

    // an origin mis-match will fall through to an Opaque filtered response
    let origin = Origin::Origin(UrlOrigin::UID(OpaqueOrigin::new()));
    let mut request = Request::new(url, Some(origin), false);
    request.referer = Referer::NoReferer;

    let (tx, rx) = channel();
    let listener = Box::new(FetchResponseCollector {
        sender: tx.clone()
    });

    fetch_async(request, listener);
    let fetch_response = rx.recv().unwrap();
    let _ = server.close();

    assert_eq!(fetch_response.response_type, ResponseType::Opaque);
    assert_eq!(response_is_done(&fetch_response), true);
}

#[test]
fn test_opaque_redirect_filtered_fetch_async_returns_complete_response() {

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

    let origin = Origin::Origin(url.origin());
    let mut request = Request::new(url, Some(origin), false);
    request.referer = Referer::NoReferer;
    request.redirect_mode.set(RedirectMode::Manual);

    let (tx, rx) = channel();
    let listener = Box::new(FetchResponseCollector {
        sender: tx.clone()
    });

    fetch_async(request, listener);
    let fetch_response = rx.recv().unwrap();
    let _ = server.close();

    assert_eq!(fetch_response.response_type, ResponseType::OpaqueRedirect);
    assert_eq!(response_is_done(&fetch_response), true);
}
