/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use {DEFAULT_USER_AGENT, new_fetch_context, create_embedder_proxy, fetch, make_server, make_ssl_server};
use devtools_traits::HttpRequest as DevtoolsHttpRequest;
use devtools_traits::HttpResponse as DevtoolsHttpResponse;
use fetch_with_context;
use fetch_with_cors_cache;
use headers_core::HeaderMapExt;
use headers_ext::{AccessControlAllowCredentials, AccessControlAllowHeaders, AccessControlAllowOrigin};
use headers_ext::{AccessControlAllowMethods, AccessControlMaxAge};
use headers_ext::{CacheControl, ContentLength, ContentType, Expires, Host, LastModified, Pragma, UserAgent};
use http::{Method, StatusCode};
use http::header::{self, HeaderMap, HeaderName, HeaderValue};
use http::uri::Authority;
use http_loader::{expect_devtools_http_request, expect_devtools_http_response};
use hyper::{Request as HyperRequest, Response as HyperResponse};
use hyper::body::Body;
use mime::{self, Mime};
use msg::constellation_msg::TEST_PIPELINE_ID;
use net::connector::create_ssl_connector_builder;
use net::fetch::cors_cache::CorsCache;
use net::fetch::methods::{CancellationListener, FetchContext};
use net::filemanager_thread::FileManager;
use net::hsts::HstsEntry;
use net::test::HttpState;
use net_traits::IncludeSubdomains;
use net_traits::NetworkError;
use net_traits::ReferrerPolicy;
use net_traits::request::{Destination, Origin, RedirectMode, Referrer, Request, RequestMode};
use net_traits::response::{CacheState, Response, ResponseBody, ResponseType};
use servo_channel::{channel, Sender};
use servo_url::{ImmutableOrigin, ServoUrl};
use std::fs::File;
use std::io::Read;
use std::iter::FromIterator;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, Duration};

// TODO write a struct that impls Handler for storing test values

#[test]
fn test_fetch_response_is_not_network_error() {
    static MESSAGE: &'static [u8] = b"";
    let handler = move |_: HyperRequest<Body>, response: &mut HyperResponse<Body>| {
        *response.body_mut() = MESSAGE.to_vec().into();
    };
    let (server, url) = make_server(handler);

    let origin = Origin::Origin(url.origin());
    let mut request = Request::new(url, Some(origin), None);
    request.referrer = Referrer::NoReferrer;
    let fetch_response = fetch(&mut request, None);
    let _ = server.close();

    if fetch_response.is_network_error() {
        panic!("fetch response shouldn't be a network error");
    }
}

#[test]
fn test_fetch_on_bad_port_is_network_error() {
    let url = ServoUrl::parse("http://www.example.org:6667").unwrap();
    let origin = Origin::Origin(url.origin());
    let mut request = Request::new(url, Some(origin), None);
    request.referrer = Referrer::NoReferrer;
    let fetch_response = fetch(&mut request, None);
    assert!(fetch_response.is_network_error());
    let fetch_error = fetch_response.get_network_error().unwrap();
    assert_eq!(fetch_error, &NetworkError::Internal("Request attempted on bad port".into()))
}

#[test]
fn test_fetch_response_body_matches_const_message() {
    static MESSAGE: &'static [u8] = b"Hello World!";
    let handler = move |_: HyperRequest<Body>, response: &mut HyperResponse<Body>| {
        *response.body_mut() = MESSAGE.to_vec().into();
    };
    let (server, url) = make_server(handler);

    let origin = Origin::Origin(url.origin());
    let mut request = Request::new(url, Some(origin), None);
    request.referrer = Referrer::NoReferrer;
    let fetch_response = fetch(&mut request, None);
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
    let mut request = Request::new(url, Some(origin), None);
    request.referrer = Referrer::NoReferrer;
    let fetch_response = fetch(&mut request, None);
    assert!(!fetch_response.is_network_error());
    assert_eq!(*fetch_response.body.lock().unwrap(), ResponseBody::Done(vec![]));
}

#[test]
fn test_fetch_blob() {
    use ipc_channel::ipc;
    use net_traits::blob_url_store::BlobBuf;

    let context = new_fetch_context(None, None);

    let bytes = b"content";
    let blob_buf = BlobBuf {
        filename: Some("test.txt".into()),
        type_string: "text/plain".into(),
        size: bytes.len() as u64,
        bytes: bytes.to_vec(),
    };

    let origin = ServoUrl::parse("http://www.example.org/").unwrap();

    let (sender, receiver) = ipc::channel().unwrap();
    context.filemanager.promote_memory(blob_buf, true, sender, "http://www.example.org".into());
    let id = receiver.recv().unwrap().unwrap();
    let url = ServoUrl::parse(&format!("blob:{}{}", origin.as_str(), id.simple())).unwrap();


    let mut request = Request::new(url, Some(Origin::Origin(origin.origin())), None);
    let fetch_response = fetch_with_context(&mut request, &context);

    assert!(!fetch_response.is_network_error());

    assert_eq!(fetch_response.headers.len(), 2);

    let content_type: Mime = fetch_response.headers.typed_get::<ContentType>().unwrap().into();
    assert_eq!(content_type, mime::TEXT_PLAIN);

    let content_length: ContentLength = fetch_response.headers.typed_get().unwrap();
    assert_eq!(content_length.0, bytes.len() as u64);

    assert_eq!(*fetch_response.body.lock().unwrap(),
               ResponseBody::Done(bytes.to_vec()));
}

#[test]
fn test_fetch_file() {
    let path = Path::new("../../resources/servo.css").canonicalize().unwrap();
    let url = ServoUrl::from_file_path(path.clone()).unwrap();
    let origin = Origin::Origin(url.origin());
    let mut request = Request::new(url, Some(origin), None);

    let fetch_response = fetch(&mut request, None);
    assert!(!fetch_response.is_network_error());
    assert_eq!(fetch_response.headers.len(), 1);
    let content_type: Mime = fetch_response.headers.typed_get::<ContentType>().unwrap().into();
    assert_eq!(content_type, mime::TEXT_CSS);

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
fn test_fetch_ftp() {
    let url = ServoUrl::parse("ftp://not-supported").unwrap();
    let origin = Origin::Origin(url.origin());
    let mut request = Request::new(url, Some(origin), None);
    request.referrer = Referrer::NoReferrer;
    let fetch_response = fetch(&mut request, None);
    assert!(fetch_response.is_network_error());
}

#[test]
fn test_fetch_bogus_scheme() {
    let url = ServoUrl::parse("bogus://whatever").unwrap();
    let origin = Origin::Origin(url.origin());
    let mut request = Request::new(url, Some(origin), None);
    request.referrer = Referrer::NoReferrer;
    let fetch_response = fetch(&mut request, None);
    assert!(fetch_response.is_network_error());
}

#[test]
fn test_cors_preflight_fetch() {
    static ACK: &'static [u8] = b"ACK";
    let state = Arc::new(AtomicUsize::new(0));
    let handler = move |request: HyperRequest<Body>, response: &mut HyperResponse<Body>| {
        if request.method() == Method::OPTIONS && state.clone().fetch_add(1, Ordering::SeqCst) == 0 {
            assert!(request.headers().contains_key(header::ACCESS_CONTROL_REQUEST_METHOD));
            assert!(!request.headers().contains_key(header::ACCESS_CONTROL_REQUEST_HEADERS));
            assert!(!request.headers().get(header::REFERER).unwrap().to_str().unwrap().contains("a.html"));
            response.headers_mut().typed_insert(AccessControlAllowOrigin::ANY);
            response.headers_mut().typed_insert(AccessControlAllowCredentials);
            response.headers_mut().typed_insert(AccessControlAllowMethods::from_iter(vec![Method::GET]));
        } else {
            response.headers_mut().typed_insert(AccessControlAllowOrigin::ANY);
            *response.body_mut() = ACK.to_vec().into();
        }
    };
    let (server, url) = make_server(handler);

    let target_url = url.clone().join("a.html").unwrap();

    let origin = Origin::Origin(ImmutableOrigin::new_opaque());
    let mut request = Request::new(url.clone(), Some(origin), None);
    request.referrer = Referrer::ReferrerUrl(target_url);
    request.referrer_policy = Some(ReferrerPolicy::Origin);
    request.use_cors_preflight = true;
    request.mode = RequestMode::CorsMode;
    let fetch_response = fetch(&mut request, None);
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
    let handler = move |request: HyperRequest<Body>, response: &mut HyperResponse<Body>| {
        if request.method() == Method::OPTIONS && state.clone().fetch_add(1, Ordering::SeqCst) == 0 {
            assert!(request.headers().contains_key(header::ACCESS_CONTROL_REQUEST_METHOD));
            assert!(!request.headers().contains_key(header::ACCESS_CONTROL_REQUEST_HEADERS));
            response.headers_mut().typed_insert(AccessControlAllowOrigin::ANY);
            response.headers_mut().typed_insert(AccessControlAllowCredentials);
            response.headers_mut().typed_insert(AccessControlAllowMethods::from_iter(vec![Method::GET]));
            response.headers_mut().typed_insert(AccessControlMaxAge::from(Duration::new(6000, 0)));
        } else {
            response.headers_mut().typed_insert(AccessControlAllowOrigin::ANY);
            *response.body_mut() = ACK.to_vec().into();
        }
    };
    let (server, url) = make_server(handler);

    let origin = Origin::Origin(ImmutableOrigin::new_opaque());
    let mut request = Request::new(url.clone(), Some(origin.clone()), None);
    request.referrer = Referrer::NoReferrer;
    request.use_cors_preflight = true;
    request.mode = RequestMode::CorsMode;
    let mut wrapped_request0 = request.clone();
    let mut wrapped_request1 = request;

    let fetch_response0 = fetch_with_cors_cache(&mut wrapped_request0, &mut cache);
    let fetch_response1 = fetch_with_cors_cache(&mut wrapped_request1, &mut cache);
    let _ = server.close();

    assert!(!fetch_response0.is_network_error() && !fetch_response1.is_network_error());

    // The response from the CORS-preflight cache was used
    assert_eq!(1, counter.load(Ordering::SeqCst));

    // The entry exists in the CORS-preflight cache
    assert_eq!(true, cache.match_method(&wrapped_request0, Method::GET));
    assert_eq!(true, cache.match_method(&wrapped_request1, Method::GET));

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
    let handler = move |request: HyperRequest<Body>, response: &mut HyperResponse<Body>| {
        if request.method() == Method::OPTIONS && state.clone().fetch_add(1, Ordering::SeqCst) == 0 {
            assert!(request.headers().contains_key(header::ACCESS_CONTROL_REQUEST_METHOD));
            assert!(!request.headers().contains_key(header::ACCESS_CONTROL_REQUEST_HEADERS));
            response.headers_mut().typed_insert(AccessControlAllowOrigin::ANY);
            response.headers_mut().typed_insert(AccessControlAllowCredentials);
            response.headers_mut().typed_insert(AccessControlAllowMethods::from_iter(vec![Method::GET]));
        } else {
            response.headers_mut().typed_insert(AccessControlAllowOrigin::ANY);
            *response.body_mut() = ACK.to_vec().into();
        }
    };
    let (server, url) = make_server(handler);

    let origin = Origin::Origin(ImmutableOrigin::new_opaque());
    let mut request = Request::new(url, Some(origin), None);
    request.method = Method::from_bytes(b"CHICKEN").unwrap();
    request.referrer = Referrer::NoReferrer;
    request.use_cors_preflight = true;
    request.mode = RequestMode::CorsMode;
    let fetch_response = fetch(&mut request, None);
    let _ = server.close();

    assert!(fetch_response.is_network_error());
}

#[test]
fn test_fetch_response_is_basic_filtered() {
    static MESSAGE: &'static [u8] = b"";
    let handler = move |_: HyperRequest<Body>, response: &mut HyperResponse<Body>| {
        response.headers_mut().insert(header::SET_COOKIE, HeaderValue::from_static(""));
        // this header is obsoleted, so hyper doesn't implement it, but it's still covered by the spec
        response.headers_mut().insert(
            HeaderName::from_static("set-cookie2"),
            HeaderValue::from_bytes(&vec![]).unwrap()
        );

        *response.body_mut() = MESSAGE.to_vec().into();
    };
    let (server, url) = make_server(handler);

    let origin = Origin::Origin(url.origin());
    let mut request = Request::new(url, Some(origin), None);
    request.referrer = Referrer::NoReferrer;
    let fetch_response = fetch(&mut request, None);
    let _ = server.close();

    assert!(!fetch_response.is_network_error());
    assert_eq!(fetch_response.response_type, ResponseType::Basic);

    let headers = fetch_response.headers;
    assert!(!headers.contains_key(header::SET_COOKIE));
    assert!(headers.get(HeaderName::from_static("set-cookie2")).is_none());
}

#[test]
fn test_fetch_response_is_cors_filtered() {
    static MESSAGE: &'static [u8] = b"";
    let handler = move |_: HyperRequest<Body>, response: &mut HyperResponse<Body>| {
        // this is mandatory for the Cors Check to pass
        // TODO test using different url encodings with this value ie. punycode
        response.headers_mut().typed_insert(AccessControlAllowOrigin::ANY);

        // these are the headers that should be kept after filtering
        response.headers_mut().typed_insert(CacheControl::new());
        response.headers_mut().insert(header::CONTENT_LANGUAGE, HeaderValue::from_bytes(&vec![]).unwrap());
        response.headers_mut().typed_insert(ContentType::from(mime::TEXT_HTML));
        response.headers_mut().typed_insert(Expires::from(SystemTime::now() + Duration::new(86400, 0)));
        response.headers_mut().typed_insert(LastModified::from(SystemTime::now()));
        response.headers_mut().typed_insert(Pragma::no_cache());

        // these headers should not be kept after filtering, even though they are given a pass
        response.headers_mut().insert(header::SET_COOKIE, HeaderValue::from_static(""));
        response.headers_mut().insert(
            HeaderName::from_static("set-cookie2"),
            HeaderValue::from_bytes(&vec![]).unwrap()
        );
        response.headers_mut().typed_insert(
            AccessControlAllowHeaders::from_iter(vec![
                HeaderName::from_static("set-cookie"),
                HeaderName::from_static("set-cookie2")
            ])
        );

        *response.body_mut() = MESSAGE.to_vec().into();
    };
    let (server, url) = make_server(handler);

    // an origin mis-match will stop it from defaulting to a basic filtered response
    let origin = Origin::Origin(ImmutableOrigin::new_opaque());
    let mut request = Request::new(url, Some(origin), None);
    request.referrer = Referrer::NoReferrer;
    request.mode = RequestMode::CorsMode;
    let fetch_response = fetch(&mut request, None);
    let _ = server.close();

    assert!(!fetch_response.is_network_error());
    assert_eq!(fetch_response.response_type, ResponseType::Cors);

    let headers = fetch_response.headers;
    assert!(headers.contains_key(header::CACHE_CONTROL));
    assert!(headers.contains_key(header::CONTENT_LANGUAGE));
    assert!(headers.contains_key(header::CONTENT_TYPE));
    assert!(headers.contains_key(header::EXPIRES));
    assert!(headers.contains_key(header::LAST_MODIFIED));
    assert!(headers.contains_key(header::PRAGMA));

    assert!(!headers.contains_key(header::ACCESS_CONTROL_ALLOW_ORIGIN));
    assert!(!headers.contains_key(header::SET_COOKIE));
    assert!(headers.get(HeaderName::from_static("set-cookie2")).is_none());
}

#[test]
fn test_fetch_response_is_opaque_filtered() {
    static MESSAGE: &'static [u8] = b"";
    let handler = move |_: HyperRequest<Body>, response: &mut HyperResponse<Body>| {
        *response.body_mut() = MESSAGE.to_vec().into();
    };
    let (server, url) = make_server(handler);

    // an origin mis-match will fall through to an Opaque filtered response
    let origin = Origin::Origin(ImmutableOrigin::new_opaque());
    let mut request = Request::new(url, Some(origin), None);
    request.referrer = Referrer::NoReferrer;
    let fetch_response = fetch(&mut request, None);
    let _ = server.close();

    assert!(!fetch_response.is_network_error());
    assert_eq!(fetch_response.response_type, ResponseType::Opaque);

    assert!(fetch_response.url().is_none());
    assert!(fetch_response.url_list.is_empty());
    // this also asserts that status message is "the empty byte sequence"
    assert!(fetch_response.status.is_none());
    assert_eq!(fetch_response.headers, HeaderMap::new());
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
    let handler = move |request: HyperRequest<Body>, response: &mut HyperResponse<Body>| {
        let redirects = request.uri().path().split("/").collect::<String>().parse::<u32>().unwrap_or(0);

        if redirects == 1 {
            *response.body_mut() = MESSAGE.to_vec().into();
        } else {
            *response.status_mut() = StatusCode::FOUND;
            response.headers_mut().insert(header::LOCATION, HeaderValue::from_static("1"));
        }
    };

    let (server, url) = make_server(handler);

    let origin = Origin::Origin(url.origin());
    let mut request = Request::new(url, Some(origin), None);
    request.referrer = Referrer::NoReferrer;
    request.redirect_mode = RedirectMode::Manual;
    let fetch_response = fetch(&mut request, None);
    let _ = server.close();

    assert!(!fetch_response.is_network_error());
    assert_eq!(fetch_response.response_type, ResponseType::OpaqueRedirect);

    // this also asserts that status message is "the empty byte sequence"
    assert!(fetch_response.status.is_none());
    assert_eq!(fetch_response.headers, HeaderMap::new());
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
    let handler = move |_: HyperRequest<Body>, response: &mut HyperResponse<Body>| {
        *response.body_mut() = MESSAGE.to_vec().into();
    };
    let (server, server_url) = make_server(handler);

    let do_fetch = |url: ServoUrl| {
        let origin = Origin::Origin(url.origin());
        let mut request = Request::new(url, Some(origin), None);
        request.referrer = Referrer::NoReferrer;

        // Set the flag.
        request.local_urls_only = true;

        fetch(&mut request, None)
    };

    let local_url = ServoUrl::parse("about:blank").unwrap();
    let local_response = do_fetch(local_url);
    let server_response = do_fetch(server_url);

    let _ = server.close();

    assert!(!local_response.is_network_error());
    assert!(server_response.is_network_error());
}
// NOTE(emilio): If this test starts failing:
//
// openssl req -x509 -nodes -days 3650 -newkey rsa:2048 \
//   -keyout resources/privatekey_for_testing.key       \
//   -out resources/self_signed_certificate_for_testing.crt
//
// And make sure to specify `localhost` as the server name.
#[test]
fn test_fetch_with_hsts() {
    static MESSAGE: &'static [u8] = b"";
    let handler = move |_: HyperRequest<Body>, response: &mut HyperResponse<Body>| {
        *response.body_mut() = MESSAGE.to_vec().into();
    };

    let cert_path = Path::new("../../resources/self_signed_certificate_for_testing.crt").canonicalize().unwrap();
    let key_path = Path::new("../../resources/privatekey_for_testing.key").canonicalize().unwrap();
    let (server, url) = make_ssl_server(handler, cert_path.clone(), key_path.clone());

    let mut ca_content = String::new();
    File::open(cert_path).unwrap().read_to_string(&mut ca_content).unwrap();
    let ssl_client = create_ssl_connector_builder(&ca_content);

    let context = FetchContext {
        state: Arc::new(HttpState::new(ssl_client)),
        user_agent: DEFAULT_USER_AGENT.into(),
        devtools_chan: None,
        filemanager: FileManager::new(create_embedder_proxy()),
        cancellation_listener: Arc::new(Mutex::new(CancellationListener::new(None))),
    };

    {
        let mut list = context.state.hsts_list.write().unwrap();
        list.push(HstsEntry::new("localhost".to_owned(), IncludeSubdomains::NotIncluded, None)
            .unwrap());
    }
    let origin = Origin::Origin(url.origin());
    let mut request = Request::new(url, Some(origin), None);
    request.referrer = Referrer::NoReferrer;
    // Set the flag.
    request.local_urls_only = false;
    let response = fetch_with_context(&mut request, &context);
    server.close();
    assert_eq!(response.internal_response.unwrap().url().unwrap().scheme(),
               "https");
}

#[test]
fn test_fetch_with_sri_network_error() {
    static MESSAGE: &'static [u8] = b"alert('Hello, Network Error');";
    let handler = move |_: HyperRequest<Body>, response: &mut HyperResponse<Body>| {
        *response.body_mut() = MESSAGE.to_vec().into();
    };
    let (server, url) = make_server(handler);

    let origin = Origin::Origin(url.origin());
    let mut request = Request::new(url, Some(origin), None);
    request.referrer = Referrer::NoReferrer;
    // To calulate hash use :
    // echo -n "alert('Hello, Network Error');" | openssl dgst -sha384 -binary | openssl base64 -A
    request.integrity_metadata =
           "sha384-H8BRh8j48O9oYatfu5AZzq6A9RINhZO5H16dQZngK7T62em8MUt1FLm52t+eX6xO".to_owned();
    // Set the flag.
    request.local_urls_only = false;

    let response = fetch(&mut request, None);

    let _ = server.close();
    assert!(response.is_network_error());
}

#[test]
fn test_fetch_with_sri_sucess() {
    static MESSAGE: &'static [u8] = b"alert('Hello, world.');";
    let handler = move |_: HyperRequest<Body>, response: &mut HyperResponse<Body>| {
        *response.body_mut() = MESSAGE.to_vec().into();
    };
    let (server, url) = make_server(handler);

    let origin = Origin::Origin(url.origin());
    let mut request = Request::new(url, Some(origin), None);
    request.referrer = Referrer::NoReferrer;
    // To calulate hash use :
    // echo -n "alert('Hello, Network Error');" | openssl dgst -sha384 -binary | openssl base64 -A
    request.integrity_metadata =
            "sha384-H8BRh8j48O9oYatfu5AZzq6A9RINhZO5H16dQZngK7T62em8MUt1FLm52t+eX6xO".to_owned();
    // Set the flag.
    request.local_urls_only = false;

    let response = fetch(&mut request, None);

    let _ = server.close();
    assert_eq!(response_is_done(&response), true);
}

/// `fetch` should return a network error if there is a header `X-Content-Type-Options: nosniff`
#[test]
fn test_fetch_blocked_nosniff() {
    #[inline]
    fn test_nosniff_request(destination: Destination,
                            mime: Mime,
                            should_error: bool) {
        const MESSAGE: &'static [u8] = b"";
        const HEADER: &'static str = "x-content-type-options";
        const VALUE: &'static [u8] = b"nosniff";

        let handler = move |_: HyperRequest<Body>, response: &mut HyperResponse<Body>| {
            let mime_header = ContentType::from(mime.clone());
            response.headers_mut().typed_insert(mime_header);
            assert!(response.headers().contains_key(header::CONTENT_TYPE));
            // Add the nosniff header
            response.headers_mut().insert(HeaderName::from_static(HEADER), HeaderValue::from_bytes(VALUE).unwrap());

            *response.body_mut() = MESSAGE.to_vec().into();
        };

        let (server, url) = make_server(handler);

        let origin = Origin::Origin(url.origin());
        let mut request = Request::new(url, Some(origin), None);
        request.destination = destination;
        let fetch_response = fetch(&mut request, None);
        let _ = server.close();

        assert_eq!(fetch_response.is_network_error(), should_error);
    }

    let tests = vec![
        (Destination::Script, mime::TEXT_JAVASCRIPT, false),
        (Destination::Script, mime::TEXT_CSS, true),
        (Destination::Style,  mime::TEXT_CSS, false),
    ];

    for test in tests {
        let (destination, mime, should_error) = test;
        test_nosniff_request(destination, mime, should_error);
    }
}

fn setup_server_and_fetch(message: &'static [u8], redirect_cap: u32) -> Response {
    let handler = move |request: HyperRequest<Body>, response: &mut HyperResponse<Body>| {
        let redirects = request.uri().path().split("/").collect::<String>().parse::<u32>().unwrap_or(0);

        if redirects >= redirect_cap {
            *response.body_mut() = message.to_vec().into();
        } else {
            *response.status_mut() = StatusCode::FOUND;
            let url = format!("{redirects}", redirects = redirects + 1);
            response.headers_mut().insert(header::LOCATION, HeaderValue::from_str(&url).unwrap());
        }
    };

    let (server, url) = make_server(handler);

    let origin = Origin::Origin(url.origin());
    let mut request = Request::new(url, Some(origin), None);
    request.referrer = Referrer::NoReferrer;
    let fetch_response = fetch(&mut request, None);
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

    let handler = move |request: HyperRequest<Body>, response: &mut HyperResponse<Body>| {
        let redirects = request.uri().path().split("/").collect::<String>().parse::<u32>().unwrap_or(0);

        let mut test_pass = true;

        if redirects == 0 {
            *response.status_mut() = StatusCode::TEMPORARY_REDIRECT;
            response.headers_mut().insert(header::LOCATION, HeaderValue::from_static("1"));

        } else if redirects == 1 {
            // this makes sure that the request method does't change from the wrong status code
            if handler_method != Method::GET && request.method() == Method::GET {
                test_pass = false;
            }
            *response.status_mut() = status_code;
            response.headers_mut().insert(header::LOCATION, HeaderValue::from_static("2"));

        } else if request.method() != Method::GET {
            test_pass = false;
        }

        // the first time this handler is reached, nothing is being tested, so don't send anything
        if redirects > 0 {
            handler_tx.lock().unwrap().send(test_pass).unwrap();
        }

    };

    let (server, url) = make_server(handler);

    let origin = Origin::Origin(url.origin());
    let mut request = Request::new(url, Some(origin), None);
    request.referrer = Referrer::NoReferrer;
    request.method = method;

    let _ = fetch(&mut request, None);
    let _ = server.close();
}

#[test]
fn test_fetch_redirect_updates_method() {
    let (tx, rx) = channel();

    test_fetch_redirect_updates_method_runner(tx.clone(), StatusCode::MOVED_PERMANENTLY, Method::POST);
    assert_eq!(rx.recv().unwrap(), true);
    assert_eq!(rx.recv().unwrap(), true);
    // make sure the test doesn't send more data than expected
    assert_eq!(rx.try_recv().is_none(), true);

    test_fetch_redirect_updates_method_runner(tx.clone(), StatusCode::FOUND, Method::POST);
    assert_eq!(rx.recv().unwrap(), true);
    assert_eq!(rx.recv().unwrap(), true);
    assert_eq!(rx.try_recv().is_none(), true);

    test_fetch_redirect_updates_method_runner(tx.clone(), StatusCode::SEE_OTHER, Method::GET);
    assert_eq!(rx.recv().unwrap(), true);
    assert_eq!(rx.recv().unwrap(), true);
    assert_eq!(rx.try_recv().is_none(), true);

    let extension = Method::from_bytes(b"FOO").unwrap();

    test_fetch_redirect_updates_method_runner(tx.clone(), StatusCode::MOVED_PERMANENTLY, extension.clone());
    assert_eq!(rx.recv().unwrap(), true);
    // for MovedPermanently and Found, Method should only be changed if it was Post
    assert_eq!(rx.recv().unwrap(), false);
    assert_eq!(rx.try_recv().is_none(), true);

    test_fetch_redirect_updates_method_runner(tx.clone(), StatusCode::FOUND, extension.clone());
    assert_eq!(rx.recv().unwrap(), true);
    assert_eq!(rx.recv().unwrap(), false);
    assert_eq!(rx.try_recv().is_none(), true);

    test_fetch_redirect_updates_method_runner(tx.clone(), StatusCode::SEE_OTHER, extension.clone());
    assert_eq!(rx.recv().unwrap(), true);
    // for SeeOther, Method should always be changed, so this should be true
    assert_eq!(rx.recv().unwrap(), true);
    assert_eq!(rx.try_recv().is_none(), true);
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
    let handler = move |_: HyperRequest<Body>, response: &mut HyperResponse<Body>| {
        *response.body_mut() = MESSAGE.to_vec().into();
    };
    let (server, url) = make_server(handler);

    let origin = Origin::Origin(url.origin());
    let mut request = Request::new(url, Some(origin), None);
    request.referrer = Referrer::NoReferrer;

    let fetch_response = fetch(&mut request, None);

    let _ = server.close();
    assert_eq!(response_is_done(&fetch_response), true);
}

#[test]
fn test_opaque_filtered_fetch_async_returns_complete_response() {
    static MESSAGE: &'static [u8] = b"";
    let handler = move |_: HyperRequest<Body>, response: &mut HyperResponse<Body>| {
        *response.body_mut() = MESSAGE.to_vec().into();
    };
    let (server, url) = make_server(handler);

    // an origin mis-match will fall through to an Opaque filtered response
    let origin = Origin::Origin(ImmutableOrigin::new_opaque());
    let mut request = Request::new(url, Some(origin), None);
    request.referrer = Referrer::NoReferrer;

    let fetch_response = fetch(&mut request, None);

    let _ = server.close();

    assert_eq!(fetch_response.response_type, ResponseType::Opaque);
    assert_eq!(response_is_done(&fetch_response), true);
}

#[test]
fn test_opaque_redirect_filtered_fetch_async_returns_complete_response() {
    static MESSAGE: &'static [u8] = b"";
    let handler = move |request: HyperRequest<Body>, response: &mut HyperResponse<Body>| {
        let redirects = request.uri().path().split("/").collect::<String>().parse::<u32>().unwrap_or(0);

        if redirects == 1 {
            *response.body_mut() = MESSAGE.to_vec().into();
        } else {
            *response.status_mut() = StatusCode::FOUND;
            response.headers_mut().insert(header::LOCATION, HeaderValue::from_static("1"));
        }
    };

    let (server, url) = make_server(handler);

    let origin = Origin::Origin(url.origin());
    let mut request = Request::new(url, Some(origin), None);
    request.referrer = Referrer::NoReferrer;
    request.redirect_mode = RedirectMode::Manual;

    let fetch_response = fetch(&mut request, None);

    let _ = server.close();

    assert_eq!(fetch_response.response_type, ResponseType::OpaqueRedirect);
    assert_eq!(response_is_done(&fetch_response), true);
}

#[test]
fn test_fetch_with_devtools() {
    static MESSAGE: &'static [u8] = b"Yay!";
    let handler = move |_: HyperRequest<Body>, response: &mut HyperResponse<Body>| {
        *response.body_mut() = MESSAGE.to_vec().into();
    };

    let (server, url) = make_server(handler);

    let origin = Origin::Origin(url.origin());
    let mut request = Request::new(url.clone(), Some(origin), Some(TEST_PIPELINE_ID));
    request.referrer = Referrer::NoReferrer;

    let (devtools_chan, devtools_port) = channel();

    let _ = fetch(&mut request, Some(devtools_chan));
    let _ = server.close();

    // notification received from devtools
    let devhttprequest = expect_devtools_http_request(&devtools_port);
    let mut devhttpresponse = expect_devtools_http_response(&devtools_port);

    //Creating default headers for request
    let mut headers = HeaderMap::new();

    headers.insert(header::ACCEPT_ENCODING, HeaderValue::from_static("gzip, deflate, br"));
    headers.typed_insert(
        Host::from(format!("{}:{}", url.host_str().unwrap(), url.port().unwrap()).parse::<Authority>().unwrap()));

    headers.insert(header::ACCEPT, HeaderValue::from_static("*/*"));

    headers.insert(header::ACCEPT_LANGUAGE, HeaderValue::from_static("en-US, en; q=0.5"));

    headers.typed_insert::<UserAgent>(DEFAULT_USER_AGENT.parse().unwrap());

    let httprequest = DevtoolsHttpRequest {
        url: url,
        method: Method::GET,
        headers: headers,
        body: Some(vec![]),
        pipeline_id: TEST_PIPELINE_ID,
        startedDateTime: devhttprequest.startedDateTime,
        timeStamp: devhttprequest.timeStamp,
        connect_time: devhttprequest.connect_time,
        send_time: devhttprequest.send_time,
        is_xhr: true,
    };

    let content = "Yay!";
    let mut response_headers = HeaderMap::new();
    response_headers.typed_insert(ContentLength(content.len() as u64));
    devhttpresponse.headers.as_mut().unwrap().remove(header::DATE);

    let httpresponse = DevtoolsHttpResponse {
        headers: Some(response_headers),
        status: Some((200, b"OK".to_vec())),
        body: None,
        pipeline_id: TEST_PIPELINE_ID,
    };

    assert_eq!(devhttprequest, httprequest);
    assert_eq!(devhttpresponse, httpresponse);
}
