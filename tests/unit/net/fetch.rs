/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use {DEFAULT_USER_AGENT, new_fetch_context, fetch, make_server};
use devtools_traits::DevtoolsControlMsg;
use devtools_traits::HttpRequest as DevtoolsHttpRequest;
use devtools_traits::HttpResponse as DevtoolsHttpResponse;
use fetch_with_context;
use fetch_with_cors_cache;
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
use hyper::server::{Request as HyperRequest, Response as HyperResponse, Server};
use hyper::status::StatusCode;
use hyper::uri::RequestUri;
use hyper_openssl;
use msg::constellation_msg::TEST_PIPELINE_ID;
use net::connector::create_ssl_client;
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
use servo_config::resource_files::resources_dir_path;
use servo_url::{ImmutableOrigin, ServoUrl};
use std::fs::File;
use std::io::Read;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{Sender, channel};
use time::{self, Duration};
use unicase::UniCase;

// TODO write a struct that impls Handler for storing test values

#[test]
fn test_fetch_response_is_not_network_error() {
    static MESSAGE: &'static [u8] = b"";
    let handler = move |_: HyperRequest, response: HyperResponse| {
        response.send(MESSAGE).unwrap();
    };
    let (mut server, url) = make_server(handler);

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
    assert!(fetch_error == &NetworkError::Internal("Request attempted on bad port".into()))
}

#[test]
fn test_fetch_response_body_matches_const_message() {
    static MESSAGE: &'static [u8] = b"Hello World!";
    let handler = move |_: HyperRequest, response: HyperResponse| {
        response.send(MESSAGE).unwrap();
    };
    let (mut server, url) = make_server(handler);

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
    assert!(*fetch_response.body.lock().unwrap() == ResponseBody::Done(vec![]));
}

#[test]
fn test_fetch_blob() {
    use ipc_channel::ipc;
    use net_traits::blob_url_store::BlobBuf;

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
    context.filemanager.promote_memory(blob_buf, true, sender, "http://www.example.org".into());
    let id = receiver.recv().unwrap().unwrap();
    let url = ServoUrl::parse(&format!("blob:{}{}", origin.as_str(), id.simple())).unwrap();


    let mut request = Request::new(url, Some(Origin::Origin(origin.origin())), None);
    let fetch_response = fetch_with_context(&mut request, &context);

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
    let mut request = Request::new(url, Some(origin), None);

    let fetch_response = fetch(&mut request, None);
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
    let handler = move |request: HyperRequest, mut response: HyperResponse| {
        if request.method == Method::Options && state.clone().fetch_add(1, Ordering::SeqCst) == 0 {
            assert!(request.headers.has::<AccessControlRequestMethod>());
            assert!(!request.headers.has::<AccessControlRequestHeaders>());
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
    let handler = move |request: HyperRequest, mut response: HyperResponse| {
        if request.method == Method::Options && state.clone().fetch_add(1, Ordering::SeqCst) == 0 {
            assert!(request.headers.has::<AccessControlRequestMethod>());
            assert!(!request.headers.has::<AccessControlRequestHeaders>());
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
    assert_eq!(true, cache.match_method(&wrapped_request0, Method::Get));
    assert_eq!(true, cache.match_method(&wrapped_request1, Method::Get));

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
            assert!(!request.headers.has::<AccessControlRequestHeaders>());
            response.headers_mut().set(AccessControlAllowOrigin::Any);
            response.headers_mut().set(AccessControlAllowCredentials);
            response.headers_mut().set(AccessControlAllowMethods(vec![Method::Get]));
        } else {
            response.headers_mut().set(AccessControlAllowOrigin::Any);
            response.send(ACK).unwrap();
        }
    };
    let (mut server, url) = make_server(handler);

    let origin = Origin::Origin(ImmutableOrigin::new_opaque());
    let mut request = Request::new(url, Some(origin), None);
    request.method = Method::Extension("CHICKEN".to_owned());
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
    let handler = move |_: HyperRequest, mut response: HyperResponse| {
        response.headers_mut().set(SetCookie(vec![]));
        // this header is obsoleted, so hyper doesn't implement it, but it's still covered by the spec
        response.headers_mut().set_raw("Set-Cookie2", vec![]);

        response.send(MESSAGE).unwrap();
    };
    let (mut server, url) = make_server(handler);

    let origin = Origin::Origin(url.origin());
    let mut request = Request::new(url, Some(origin), None);
    request.referrer = Referrer::NoReferrer;
    let fetch_response = fetch(&mut request, None);
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
    let origin = Origin::Origin(ImmutableOrigin::new_opaque());
    let mut request = Request::new(url, Some(origin), None);
    request.referrer = Referrer::NoReferrer;
    request.mode = RequestMode::CorsMode;
    let fetch_response = fetch(&mut request, None);
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
    let mut request = Request::new(url, Some(origin), None);
    request.referrer = Referrer::NoReferrer;
    request.redirect_mode = RedirectMode::Manual;
    let fetch_response = fetch(&mut request, None);
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
    let handler = move |_: HyperRequest, response: HyperResponse| {
        response.send(MESSAGE).unwrap();
    };

    let path = resources_dir_path().expect("Cannot find resource dir");
    let mut cert_path = path.clone();
    cert_path.push("self_signed_certificate_for_testing.crt");

    let mut key_path = path.clone();
    key_path.push("privatekey_for_testing.key");

    let ssl = hyper_openssl::OpensslServer::from_files(key_path, cert_path)
        .unwrap();

    //takes an address and something that implements hyper::net::Ssl
    let mut server = Server::https("0.0.0.0:0", ssl).unwrap().handle_threads(handler, 1).unwrap();

    let ca_file = resources_dir_path().unwrap().join("self_signed_certificate_for_testing.crt");
    let ssl_client = create_ssl_client(&ca_file);

    let context =  FetchContext {
        state: Arc::new(HttpState::new(ssl_client)),
        user_agent: DEFAULT_USER_AGENT.into(),
        devtools_chan: None,
        filemanager: FileManager::new(),
        cancellation_listener: Arc::new(Mutex::new(CancellationListener::new(None))),
    };

    {
        let mut list = context.state.hsts_list.write().unwrap();
        list.push(HstsEntry::new("localhost".to_owned(), IncludeSubdomains::NotIncluded, None)
            .unwrap());
    }
    let url_string = format!("http://localhost:{}", server.socket.port());
    let url = ServoUrl::parse(&url_string).unwrap();
    let origin = Origin::Origin(url.origin());
    let mut request = Request::new(url, Some(origin), None);
    request.referrer = Referrer::NoReferrer;
    // Set the flag.
    request.local_urls_only = false;
    let response = fetch_with_context(&mut request, &context);
    let _ = server.close();
    assert_eq!(response.internal_response.unwrap().url().unwrap().scheme(),
               "https");
}

#[test]
fn test_fetch_with_sri_network_error() {
    static MESSAGE: &'static [u8] = b"alert('Hello, Network Error');";
    let handler = move |_: HyperRequest, response: HyperResponse| {
        response.send(MESSAGE).unwrap();
    };
    let (mut server, url) = make_server(handler);

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
    let handler = move |_: HyperRequest, response: HyperResponse| {
        response.send(MESSAGE).unwrap();
    };
    let (mut server, url) = make_server(handler);

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
        const HEADER: &'static str = "X-Content-Type-Options";
        const VALUE: &'static [u8] = b"nosniff";

        let handler = move |_: HyperRequest, mut response: HyperResponse| {
            let mime_header = ContentType(mime.clone());
            response.headers_mut().set(mime_header);
            assert!(response.headers().has::<ContentType>());
            // Add the nosniff header
            response.headers_mut().set_raw(HEADER, vec![VALUE.to_vec()]);

            response.send(MESSAGE).unwrap();
        };

        let (mut server, url) = make_server(handler);

        let origin = Origin::Origin(url.origin());
        let mut request = Request::new(url, Some(origin), None);
        request.destination = destination;
        let fetch_response = fetch(&mut request, None);
        let _ = server.close();

        assert_eq!(fetch_response.is_network_error(), should_error);
    }

    let tests = vec![
        (Destination::Script, Mime(TopLevel::Text, SubLevel::Javascript, vec![]), false),
        (Destination::Script, Mime(TopLevel::Text, SubLevel::Css, vec![]), true),
        (Destination::Style,  Mime(TopLevel::Text, SubLevel::Css, vec![]), false),
    ];

    for test in tests {
        let (destination, mime, should_error) = test;
        test_nosniff_request(destination, mime, should_error);
    }
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
    let mut request = Request::new(url, Some(origin), None);
    request.referrer = Referrer::NoReferrer;
    request.method = method;

    let _ = fetch(&mut request, None);
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
    let mut request = Request::new(url, Some(origin), None);
    request.referrer = Referrer::NoReferrer;

    let fetch_response = fetch(&mut request, None);

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
    let handler = move |_: HyperRequest, response: HyperResponse| {
        response.send(MESSAGE).unwrap();
    };

    let (mut server, url) = make_server(handler);

    let origin = Origin::Origin(url.origin());
    let mut request = Request::new(url.clone(), Some(origin), Some(TEST_PIPELINE_ID));
    request.referrer = Referrer::NoReferrer;

    let (devtools_chan, devtools_port) = channel::<DevtoolsControlMsg>();

    let _ = fetch(&mut request, Some(devtools_chan));
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
