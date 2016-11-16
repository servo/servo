/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use {DEFAULT_USER_AGENT, FetchResponseCollector, new_fetch_context, fetch_async, fetch_sync, make_server};
use devtools_traits::DevtoolsControlMsg;
use devtools_traits::HttpRequest as DevtoolsHttpRequest;
use devtools_traits::HttpResponse as DevtoolsHttpResponse;
use http_loader::{expect_devtools_http_request, expect_devtools_http_response};
use hyper::LanguageTag;
use hyper::header::{Accept, AccessControlAllowCredentials, AccessControlAllowHeaders, AccessControlAllowOrigin};
use hyper::header::{AcceptEncoding, AcceptLanguage, AccessControlAllowMethods, AccessControlMaxAge};
use hyper::header::{AccessControlRequestHeaders, AccessControlRequestMethod, Date, UserAgent};
use hyper::header::{CacheControl, ContentLanguage, ContentLength, ContentType, Expires, LastModified};
use hyper::header::{Encoding, Location, Pragma, Quality, QualityItem, SetCookie, qitem};
use hyper::header::{Headers, Host, HttpDate, Referer as HyperReferer};
use hyper::method::Method;
use hyper::mime::{Mime, SubLevel, TopLevel};
use hyper::server::{Request as HyperRequest, Response as HyperResponse};
use hyper::status::StatusCode;
use hyper::uri::RequestUri;
use msg::constellation_msg::TEST_PIPELINE_ID;
use net::fetch::cors_cache::CorsCache;
use net::fetch::methods::{fetch, fetch_with_cors_cache};
use net_traits::ReferrerPolicy;
use net_traits::request::{Origin, RedirectMode, Referrer, Request, RequestMode};
use net_traits::response::{CacheState, Response, ResponseBody, ResponseType};
use servo_url::ServoUrl;
use std::fs::File;
use std::io::Read;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{Sender, channel};
use time::{self, Duration};
use unicase::UniCase;
use url::Origin as UrlOrigin;
use util::resource_files::resources_dir_path;

// TODO write a struct that impls Handler for storing test values

#[test]
fn test_fetch_response_is_not_network_error() {
    static MESSAGE: &'static [u8] = b"";
    let handler = move |_: HyperRequest, response: HyperResponse| {
        response.send(MESSAGE).unwrap();
    };
    let (mut server, url) = make_server(handler);

    let origin = Origin::Origin(url.origin());
    let request = Request::new(url, Some(origin), false, None);
    *request.referrer.borrow_mut() = Referrer::NoReferrer;
    let fetch_response = fetch_sync(request, None);
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
    let request = Request::new(url, Some(origin), false, None);
    *request.referrer.borrow_mut() = Referrer::NoReferrer;
    let fetch_response = fetch_sync(request, None);
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
    let url = ServoUrl::parse("about:blank").unwrap();
    let origin = Origin::Origin(url.origin());
    let request = Request::new(url, Some(origin), false, None);
    *request.referrer.borrow_mut() = Referrer::NoReferrer;
    let fetch_response = fetch_sync(request, None);
    assert!(!fetch_response.is_network_error());
    assert!(*fetch_response.body.lock().unwrap() == ResponseBody::Done(vec![]));
}

#[test]
fn test_fetch_blob() {
    use ipc_channel::ipc;
    use net_traits::blob_url_store::BlobBuf;
    use net_traits::filemanager_thread::FileManagerThreadMsg;

    let context = new_fetch_context(None);

    let bytes = b"content";
    let blob_buf = BlobBuf {
        filename: Some("test.txt".into()),
        type_string: "text/plain".into(),
        size: bytes.len() as u64,
        bytes: bytes.to_vec(),
    };

    let origin = ServoUrl::parse("http://www.example.org/").unwrap();

    let (sender, receiver) = ipc::channel().unwrap();
    let message = FileManagerThreadMsg::PromoteMemory(blob_buf, true, sender, "http://www.example.org".into());
    context.filemanager.handle(message, None);
    let id = receiver.recv().unwrap().unwrap();
    let url = ServoUrl::parse(&format!("blob:{}{}", origin.as_str(), id.simple())).unwrap();


    let request = Request::new(url, Some(Origin::Origin(origin.origin())), false, None);
    let fetch_response = fetch(Rc::new(request), &mut None, &context);

    assert!(!fetch_response.is_network_error());

    assert_eq!(fetch_response.headers.len(), 2);

    let content_type: &ContentType = fetch_response.headers.get().unwrap();
    assert_eq!(**content_type, Mime(TopLevel::Text, SubLevel::Plain, vec![]));

    let content_length: &ContentLength = fetch_response.headers.get().unwrap();
    assert_eq!(**content_length, bytes.len() as u64);

    assert_eq!(*fetch_response.body.lock().unwrap(),
               ResponseBody::Done(bytes.to_vec()));
}

#[test]
fn test_fetch_file() {
    let mut path = resources_dir_path().expect("Cannot find resource dir");
    path.push("servo.css");

    let url = ServoUrl::from_file_path(path.clone()).unwrap();
    let origin = Origin::Origin(url.origin());
    let request = Request::new(url, Some(origin), false, None);

    let fetch_response = fetch_sync(request, None);
    assert!(!fetch_response.is_network_error());
    assert_eq!(fetch_response.headers.len(), 1);
    let content_type: &ContentType = fetch_response.headers.get().unwrap();
    assert!(**content_type == Mime(TopLevel::Text, SubLevel::Css, vec![]));

    let resp_body = fetch_response.body.lock().unwrap();
    let mut file = File::open(path).unwrap();
    let mut bytes = vec![];
    let _ = file.read_to_end(&mut bytes);

    match *resp_body {
        ResponseBody::Done(ref val) => {
            assert_eq!(val, &bytes);
        },
        _ => panic!()
    }
}

#[test]
fn test_cors_preflight_fetch() {
    static ACK: &'static [u8] = b"ACK";
    let state = Arc::new(AtomicUsize::new(0));
    let handler = move |request: HyperRequest, mut response: HyperResponse| {
        if request.method == Method::Options && state.clone().fetch_add(1, Ordering::SeqCst) == 0 {
            assert!(request.headers.has::<AccessControlRequestMethod>());
            assert!(request.headers.has::<AccessControlRequestHeaders>());
            assert!(!request.headers.get::<HyperReferer>().unwrap().contains("a.html"));
            response.headers_mut().set(AccessControlAllowOrigin::Any);
            response.headers_mut().set(AccessControlAllowCredentials);
            response.headers_mut().set(AccessControlAllowMethods(vec![Method::Get]));
        } else {
            response.headers_mut().set(AccessControlAllowOrigin::Any);
            response.send(ACK).unwrap();
        }
    };
    let (mut server, url) = make_server(handler);

    let target_url = url.clone().join("a.html").unwrap();

    let origin = Origin::Origin(UrlOrigin::new_opaque());
    let mut request = Request::new(url.clone(), Some(origin), false, None);
    *request.referrer.borrow_mut() = Referrer::ReferrerUrl(target_url);
    *request.referrer_policy.get_mut() = Some(ReferrerPolicy::Origin);
    request.use_cors_preflight = true;
    request.mode = RequestMode::CorsMode;
    let fetch_response = fetch_sync(request, None);
    let _ = server.close();

    assert!(!fetch_response.is_network_error());

    match *fetch_response.body.lock().unwrap() {
        ResponseBody::Done(ref body) => assert_eq!(&**body, ACK),
        _ => panic!()
    };
}

#[test]
fn test_cors_preflight_cache_fetch() {
    static ACK: &'static [u8] = b"ACK";
    let state = Arc::new(AtomicUsize::new(0));
    let counter = state.clone();
    let mut cache = CorsCache::new();
    let handler = move |request: HyperRequest, mut response: HyperResponse| {
        if request.method == Method::Options && state.clone().fetch_add(1, Ordering::SeqCst) == 0 {
            assert!(request.headers.has::<AccessControlRequestMethod>());
            assert!(request.headers.has::<AccessControlRequestHeaders>());
            response.headers_mut().set(AccessControlAllowOrigin::Any);
            response.headers_mut().set(AccessControlAllowCredentials);
            response.headers_mut().set(AccessControlAllowMethods(vec![Method::Get]));
            response.headers_mut().set(AccessControlMaxAge(6000));
        } else {
            response.headers_mut().set(AccessControlAllowOrigin::Any);
            response.send(ACK).unwrap();
        }
    };
    let (mut server, url) = make_server(handler);

    let origin = Origin::Origin(UrlOrigin::new_opaque());
    let mut request = Request::new(url.clone(), Some(origin.clone()), false, None);
    *request.referrer.borrow_mut() = Referrer::NoReferrer;
    request.use_cors_preflight = true;
    request.mode = RequestMode::CorsMode;
    let wrapped_request0 = Rc::new(request.clone());
    let wrapped_request1 = Rc::new(request);

    let fetch_response0 = fetch_with_cors_cache(wrapped_request0.clone(), &mut cache,
                                                &mut None, &new_fetch_context(None));
    let fetch_response1 = fetch_with_cors_cache(wrapped_request1.clone(), &mut cache,
                                                &mut None, &new_fetch_context(None));
    let _ = server.close();

    assert!(!fetch_response0.is_network_error() && !fetch_response1.is_network_error());

    // The response from the CORS-preflight cache was used
    assert_eq!(1, counter.load(Ordering::SeqCst));

    // The entry exists in the CORS-preflight cache
    assert_eq!(true, cache.match_method(&*wrapped_request0, Method::Get));
    assert_eq!(true, cache.match_method(&*wrapped_request1, Method::Get));

    match *fetch_response0.body.lock().unwrap() {
        ResponseBody::Done(ref body) => assert_eq!(&**body, ACK),
        _ => panic!()
    };
    match *fetch_response1.body.lock().unwrap() {
        ResponseBody::Done(ref body) => assert_eq!(&**body, ACK),
        _ => panic!()
    };
}

#[test]
fn test_cors_preflight_fetch_network_error() {
    static ACK: &'static [u8] = b"ACK";
    let state = Arc::new(AtomicUsize::new(0));
    let handler = move |request: HyperRequest, mut response: HyperResponse| {
        if request.method == Method::Options && state.clone().fetch_add(1, Ordering::SeqCst) == 0 {
            assert!(request.headers.has::<AccessControlRequestMethod>());
            assert!(request.headers.has::<AccessControlRequestHeaders>());
            response.headers_mut().set(AccessControlAllowOrigin::Any);
            response.headers_mut().set(AccessControlAllowCredentials);
            response.headers_mut().set(AccessControlAllowMethods(vec![Method::Get]));
        } else {
            response.headers_mut().set(AccessControlAllowOrigin::Any);
            response.send(ACK).unwrap();
        }
    };
    let (mut server, url) = make_server(handler);

    let origin = Origin::Origin(UrlOrigin::new_opaque());
    let mut request = Request::new(url, Some(origin), false, None);
    *request.method.borrow_mut() = Method::Extension("CHICKEN".to_owned());
    *request.referrer.borrow_mut() = Referrer::NoReferrer;
    request.use_cors_preflight = true;
    request.mode = RequestMode::CorsMode;
    let fetch_response = fetch_sync(request, None);
    let _ = server.close();

    assert!(fetch_response.is_network_error());
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
    let request = Request::new(url, Some(origin), false, None);
    *request.referrer.borrow_mut() = Referrer::NoReferrer;
    let fetch_response = fetch_sync(request, None);
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
    let origin = Origin::Origin(UrlOrigin::new_opaque());
    let mut request = Request::new(url, Some(origin), false, None);
    *request.referrer.borrow_mut() = Referrer::NoReferrer;
    request.mode = RequestMode::CorsMode;
    let fetch_response = fetch_sync(request, None);
    let _ = server.close();

    assert!(!fetch_response.is_network_error());
    assert_eq!(fetch_response.response_type, ResponseType::Cors);

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
    let origin = Origin::Origin(UrlOrigin::new_opaque());
    let request = Request::new(url, Some(origin), false, None);
    *request.referrer.borrow_mut() = Referrer::NoReferrer;
    let fetch_response = fetch_sync(request, None);
    let _ = server.close();

    assert!(!fetch_response.is_network_error());
    assert_eq!(fetch_response.response_type, ResponseType::Opaque);

    assert!(fetch_response.url().is_none());
    assert!(fetch_response.url_list.into_inner().len() == 0);
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
                url.path_segments().unwrap().next_back().unwrap().parse::<u32>().unwrap_or(0),
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
    let request = Request::new(url, Some(origin), false, None);
    *request.referrer.borrow_mut() = Referrer::NoReferrer;
    request.redirect_mode.set(RedirectMode::Manual);
    let fetch_response = fetch_sync(request, None);
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

    let do_fetch = |url: ServoUrl| {
        let origin = Origin::Origin(url.origin());
        let mut request = Request::new(url, Some(origin), false, None);
        *request.referrer.borrow_mut() = Referrer::NoReferrer;

        // Set the flag.
        request.local_urls_only = true;

        fetch_sync(request, None)
    };

    let local_url = ServoUrl::parse("about:blank").unwrap();
    let local_response = do_fetch(local_url);
    let server_response = do_fetch(server_url);

    let _ = server.close();

    assert!(!local_response.is_network_error());
    assert!(server_response.is_network_error());
}

fn setup_server_and_fetch(message: &'static [u8], redirect_cap: u32) -> Response {
    let handler = move |request: HyperRequest, mut response: HyperResponse| {
        let redirects = match request.uri {
            RequestUri::AbsolutePath(url) =>
                url.split("/").collect::<String>().parse::<u32>().unwrap_or(0),
            RequestUri::AbsoluteUri(url) =>
                url.path_segments().unwrap().next_back().unwrap().parse::<u32>().unwrap_or(0),
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
    let request = Request::new(url, Some(origin), false, None);
    *request.referrer.borrow_mut() = Referrer::NoReferrer;
    let fetch_response = fetch_sync(request, None);
    let _ = server.close();
    fetch_response
}

#[test]
fn test_fetch_redirect_count_ceiling() {
    static MESSAGE: &'static [u8] = b"no more redirects";
    // how many redirects to cause
    let redirect_cap = 20;

    let fetch_response = setup_server_and_fetch(MESSAGE, redirect_cap);

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

    let fetch_response = setup_server_and_fetch(MESSAGE, redirect_cap);

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
                url.path_segments().unwrap().next_back().unwrap().parse::<u32>().unwrap_or(0),
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
    let request = Request::new(url, Some(origin), false, None);
    *request.referrer.borrow_mut() = Referrer::NoReferrer;
    *request.method.borrow_mut() = method;

    let _ = fetch_sync(request, None);
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
        ResponseType::Default | ResponseType::Basic | ResponseType::Cors => {
            (*response.body.lock().unwrap()).is_done()
        }
        // if the internal response cannot have a body, it shouldn't block the "done" state
        ResponseType::Opaque | ResponseType::OpaqueRedirect | ResponseType::Error(..) => true
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
    let request = Request::new(url, Some(origin), false, None);
    *request.referrer.borrow_mut() = Referrer::NoReferrer;

    let (tx, rx) = channel();
    let listener = Box::new(FetchResponseCollector {
        sender: tx.clone()
    });

    fetch_async(request, listener, None);
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
    let origin = Origin::Origin(UrlOrigin::new_opaque());
    let request = Request::new(url, Some(origin), false, None);
    *request.referrer.borrow_mut() = Referrer::NoReferrer;

    let (tx, rx) = channel();
    let listener = Box::new(FetchResponseCollector {
        sender: tx.clone()
    });

    fetch_async(request, listener, None);
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
                url.path_segments().unwrap().last().unwrap().parse::<u32>().unwrap_or(0),
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
    let request = Request::new(url, Some(origin), false, None);
    *request.referrer.borrow_mut() = Referrer::NoReferrer;
    request.redirect_mode.set(RedirectMode::Manual);

    let (tx, rx) = channel();
    let listener = Box::new(FetchResponseCollector {
        sender: tx.clone()
    });

    fetch_async(request, listener, None);
    let fetch_response = rx.recv().unwrap();
    let _ = server.close();

    assert_eq!(fetch_response.response_type, ResponseType::OpaqueRedirect);
    assert_eq!(response_is_done(&fetch_response), true);
}

#[test]
fn test_fetch_with_devtools() {
    static MESSAGE: &'static [u8] = b"Yay!";
    let handler = move |_: HyperRequest, response: HyperResponse| {
        response.send(MESSAGE).unwrap();
    };

    let (mut server, url) = make_server(handler);

    let origin = Origin::Origin(url.origin());
    let request = Request::new(url.clone(), Some(origin), false, Some(TEST_PIPELINE_ID));
    *request.referrer.borrow_mut() = Referrer::NoReferrer;

    let (devtools_chan, devtools_port) = channel::<DevtoolsControlMsg>();

    let _ = fetch_sync(request, Some(devtools_chan));
    let _ = server.close();

    // notification received from devtools
    let devhttprequest = expect_devtools_http_request(&devtools_port);
    let mut devhttpresponse = expect_devtools_http_response(&devtools_port);

    //Creating default headers for request
    let mut headers = Headers::new();

    headers.set(AcceptEncoding(vec![
                                   qitem(Encoding::Gzip),
                                   qitem(Encoding::Deflate),
                                   qitem(Encoding::EncodingExt("br".to_owned()))
                                   ]));

    headers.set(Host { hostname: url.host_str().unwrap().to_owned() , port: url.port().to_owned() });

    let accept = Accept(vec![qitem(Mime(TopLevel::Star, SubLevel::Star, vec![]))]);
    headers.set(accept);

    let mut en_us: LanguageTag = Default::default();
    en_us.language = Some("en".to_owned());
    en_us.region = Some("US".to_owned());
    let mut en: LanguageTag = Default::default();
    en.language = Some("en".to_owned());
    headers.set(AcceptLanguage(vec![
        qitem(en_us),
        QualityItem::new(en, Quality(500)),
    ]));

    headers.set(UserAgent(DEFAULT_USER_AGENT.to_owned()));

    let httprequest = DevtoolsHttpRequest {
        url: url,
        method: Method::Get,
        headers: headers,
        body: None,
        pipeline_id: TEST_PIPELINE_ID,
        startedDateTime: devhttprequest.startedDateTime,
        timeStamp: devhttprequest.timeStamp,
        connect_time: devhttprequest.connect_time,
        send_time: devhttprequest.send_time,
        is_xhr: true,
    };

    let content = "Yay!";
    let mut response_headers = Headers::new();
    response_headers.set(ContentLength(content.len() as u64));
    devhttpresponse.headers.as_mut().unwrap().remove::<Date>();

    let httpresponse = DevtoolsHttpResponse {
        headers: Some(response_headers),
        status: Some((200, b"OK".to_vec())),
        body: None,
        pipeline_id: TEST_PIPELINE_ID,
    };

    assert_eq!(devhttprequest, httprequest);
    assert_eq!(devhttpresponse, httpresponse);
}
