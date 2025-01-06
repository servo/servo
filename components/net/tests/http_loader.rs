/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![cfg(not(target_os = "windows"))]

use std::collections::HashMap;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

use base::id::TEST_PIPELINE_ID;
use cookie::Cookie as CookiePair;
use crossbeam_channel::{unbounded, Receiver};
use devtools_traits::{
    ChromeToDevtoolsControlMsg, DevtoolsControlMsg, HttpRequest as DevtoolsHttpRequest,
    HttpResponse as DevtoolsHttpResponse, NetworkEvent,
};
use flate2::write::{GzEncoder, ZlibEncoder};
use flate2::Compression;
use headers::authorization::Basic;
use headers::{
    Authorization, ContentLength, Date, HeaderMapExt, Host, StrictTransportSecurity, UserAgent,
};
use http::header::{self, HeaderMap, HeaderValue};
use http::uri::Authority;
use http::{HeaderName, Method, StatusCode};
use http_body_util::combinators::BoxBody;
use hyper::body::{Body, Bytes, Incoming};
use hyper::{Request as HyperRequest, Response as HyperResponse};
use ipc_channel::ipc;
use ipc_channel::router::ROUTER;
use net::cookie::ServoCookie;
use net::cookie_storage::CookieStorage;
use net::fetch::methods::{self};
use net::http_loader::{determine_requests_referrer, serialize_origin};
use net::resource_thread::AuthCacheEntry;
use net::test::{replace_host_table, DECODER_BUFFER_SIZE};
use net_traits::http_status::HttpStatus;
use net_traits::request::{
    BodyChunkRequest, BodyChunkResponse, BodySource, CredentialsMode, Destination, Referrer,
    Request, RequestBody, RequestBuilder,
};
use net_traits::response::{Response, ResponseBody};
use net_traits::{CookieSource, FetchTaskTarget, NetworkError, ReferrerPolicy};
use servo_url::{ImmutableOrigin, ServoUrl};
use url::Url;

use crate::{
    create_embedder_proxy_and_receiver, fetch, fetch_with_context, make_body, make_server,
    new_fetch_context, receive_credential_prompt_msgs,
};

fn mock_origin() -> ImmutableOrigin {
    ServoUrl::parse("http://servo.org").unwrap().origin()
}

fn assert_cookie_for_domain(
    cookie_jar: &RwLock<CookieStorage>,
    domain: &str,
    cookie: Option<&str>,
) {
    let mut cookie_jar = cookie_jar.write().unwrap();
    let url = ServoUrl::parse(&*domain).unwrap();
    let cookies = cookie_jar.cookies_for_url(&url, CookieSource::HTTP);
    assert_eq!(cookies.as_ref().map(|c| &**c), cookie);
}

pub fn expect_devtools_http_request(
    devtools_port: &Receiver<DevtoolsControlMsg>,
) -> DevtoolsHttpRequest {
    match devtools_port.recv().unwrap() {
        DevtoolsControlMsg::FromChrome(ChromeToDevtoolsControlMsg::NetworkEvent(_, net_event)) => {
            match net_event {
                NetworkEvent::HttpRequest(httprequest) => httprequest,

                _ => panic!("No HttpRequest Received"),
            }
        },
        _ => panic!("No HttpRequest Received"),
    }
}

pub fn expect_devtools_http_response(
    devtools_port: &Receiver<DevtoolsControlMsg>,
) -> DevtoolsHttpResponse {
    match devtools_port.recv().unwrap() {
        DevtoolsControlMsg::FromChrome(ChromeToDevtoolsControlMsg::NetworkEvent(
            _,
            net_event_response,
        )) => match net_event_response {
            NetworkEvent::HttpResponse(httpresponse) => httpresponse,

            _ => panic!("No HttpResponse Received"),
        },
        _ => panic!("No HttpResponse Received"),
    }
}

fn create_request_body_with_content(content: Vec<u8>) -> RequestBody {
    let content_len = content.len();

    let (chunk_request_sender, chunk_request_receiver) = ipc::channel().unwrap();
    ROUTER.add_typed_route(
        chunk_request_receiver,
        Box::new(move |message| {
            let request = message.unwrap();
            if let BodyChunkRequest::Connect(sender) = request {
                let _ = sender.send(BodyChunkResponse::Chunk(content.clone()));
                let _ = sender.send(BodyChunkResponse::Done);
            }
        }),
    );

    RequestBody::new(chunk_request_sender, BodySource::Object, Some(content_len))
}

#[test]
fn test_check_default_headers_loaded_in_every_request() {
    let expected_headers = Arc::new(Mutex::new(None));
    let expected_headers_clone = expected_headers.clone();
    let handler = move |request: HyperRequest<Incoming>,
                        _: &mut HyperResponse<BoxBody<Bytes, hyper::Error>>| {
        assert_eq!(
            request.headers().clone(),
            expected_headers_clone.lock().unwrap().take().unwrap()
        );
    };
    let (server, url) = make_server(handler);

    let mut headers = HeaderMap::new();

    headers.insert(
        header::ACCEPT_ENCODING,
        HeaderValue::from_static("gzip, deflate, br"),
    );

    headers.typed_insert(Host::from(
        format!("{}:{}", url.host_str().unwrap(), url.port().unwrap())
            .parse::<Authority>()
            .unwrap(),
    ));

    headers.insert(
        header::ACCEPT,
        HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"),
    );

    headers.insert(
        header::ACCEPT_LANGUAGE,
        HeaderValue::from_static("en-US,en;q=0.5"),
    );

    headers.typed_insert::<UserAgent>(crate::DEFAULT_USER_AGENT.parse().unwrap());

    // Append fetch metadata headers
    headers.insert(
        HeaderName::from_static("sec-fetch-dest"),
        HeaderValue::from_static("document"),
    );
    headers.insert(
        HeaderName::from_static("sec-fetch-mode"),
        HeaderValue::from_static("no-cors"),
    );
    headers.insert(
        HeaderName::from_static("sec-fetch-site"),
        HeaderValue::from_static("same-origin"),
    );
    headers.insert(
        HeaderName::from_static("sec-fetch-user"),
        HeaderValue::from_static("?1"),
    );
    headers.insert("Upgrade-Insecure-Requests", HeaderValue::from_static("1"));

    *expected_headers.lock().unwrap() = Some(headers.clone());

    // Testing for method.GET
    let request = RequestBuilder::new(url.clone(), Referrer::NoReferrer)
        .method(Method::GET)
        .destination(Destination::Document)
        .origin(url.clone().origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .build();

    let response = dbg!(fetch(request, None));
    assert!(response
        .internal_response
        .unwrap()
        .status
        .code()
        .is_success());

    // Testing for method.POST
    let mut post_headers = headers.clone();
    post_headers.typed_insert(ContentLength(0 as u64));
    let url_str = url.as_str();
    // request gets header "Origin: http://example.com" but expected_headers has
    // "Origin: http://example.com/" which do not match for equality so strip trailing '/'
    post_headers.insert(
        header::ORIGIN,
        HeaderValue::from_str(&url_str[..url_str.len() - 1]).unwrap(),
    );
    *expected_headers.lock().unwrap() = Some(post_headers);
    let request = RequestBuilder::new(url.clone(), Referrer::NoReferrer)
        .method(Method::POST)
        .destination(Destination::Document)
        .origin(url.clone().origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .build();

    let response = fetch(request, None);
    assert!(response
        .internal_response
        .unwrap()
        .status
        .code()
        .is_success());

    let _ = server.close();
}

#[test]
fn test_load_when_request_is_not_get_or_head_and_there_is_no_body_content_length_should_be_set_to_0(
) {
    let handler = move |request: HyperRequest<Incoming>,
                        _: &mut HyperResponse<BoxBody<Bytes, hyper::Error>>| {
        assert_eq!(
            request.headers().typed_get::<ContentLength>(),
            Some(ContentLength(0))
        );
    };
    let (server, url) = make_server(handler);

    let request = RequestBuilder::new(url.clone(), Referrer::NoReferrer)
        .method(Method::POST)
        .body(None)
        .destination(Destination::Document)
        .origin(mock_origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .build();

    let response = fetch(request, None);
    assert!(response
        .internal_response
        .unwrap()
        .status
        .code()
        .is_success());

    let _ = server.close();
}

#[test]
fn test_request_and_response_data_with_network_messages() {
    let handler =
        move |_: HyperRequest<Incoming>,
              response: &mut HyperResponse<BoxBody<Bytes, hyper::Error>>| {
            response
                .headers_mut()
                .typed_insert(Host::from("foo.bar".parse::<Authority>().unwrap()));
            *response.body_mut() = make_body(b"Yay!".to_vec());
        };
    let (server, url) = make_server(handler);

    let mut request_headers = HeaderMap::new();
    request_headers.typed_insert(Host::from("bar.foo".parse::<Authority>().unwrap()));
    let request = RequestBuilder::new(url.clone(), Referrer::NoReferrer)
        .method(Method::GET)
        .headers(request_headers)
        .body(None)
        .destination(Destination::Document)
        .origin(mock_origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .build();

    let (devtools_chan, devtools_port) = unbounded();
    let response = fetch(request, Some(devtools_chan));
    assert!(response
        .internal_response
        .unwrap()
        .status
        .code()
        .is_success());

    let _ = server.close();

    // notification received from devtools
    let devhttprequest = expect_devtools_http_request(&devtools_port);
    let devhttpresponse = expect_devtools_http_response(&devtools_port);

    //Creating default headers for request
    let mut headers = HeaderMap::new();

    headers.insert(
        header::ACCEPT,
        HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"),
    );

    headers.insert(
        header::ACCEPT_LANGUAGE,
        HeaderValue::from_static("en-US,en;q=0.5"),
    );

    headers.typed_insert::<UserAgent>(crate::DEFAULT_USER_AGENT.parse().unwrap());

    headers.insert(
        header::ACCEPT_ENCODING,
        HeaderValue::from_static("gzip, deflate, br"),
    );

    // Append fetch metadata headers
    headers.insert(
        HeaderName::from_static("sec-fetch-dest"),
        HeaderValue::from_static("document"),
    );
    headers.insert(
        HeaderName::from_static("sec-fetch-mode"),
        HeaderValue::from_static("no-cors"),
    );
    headers.insert(
        HeaderName::from_static("sec-fetch-site"),
        HeaderValue::from_static("same-site"),
    );
    headers.insert(
        HeaderName::from_static("sec-fetch-user"),
        HeaderValue::from_static("?1"),
    );
    headers.insert("Upgrade-Insecure-Requests", HeaderValue::from_static("1"));

    let httprequest = DevtoolsHttpRequest {
        url: url,
        method: Method::GET,
        headers: headers,
        body: Some(vec![]),
        pipeline_id: TEST_PIPELINE_ID,
        started_date_time: devhttprequest.started_date_time,
        time_stamp: devhttprequest.time_stamp,
        connect_time: devhttprequest.connect_time,
        send_time: devhttprequest.send_time,
        is_xhr: false,
    };

    let content = "Yay!";
    let mut response_headers = HeaderMap::new();
    response_headers.typed_insert(ContentLength(content.len() as u64));
    response_headers.typed_insert(Host::from("foo.bar".parse::<Authority>().unwrap()));
    response_headers.typed_insert(
        devhttpresponse
            .headers
            .as_ref()
            .unwrap()
            .typed_get::<Date>()
            .unwrap()
            .clone(),
    );

    let httpresponse = DevtoolsHttpResponse {
        headers: Some(response_headers),
        status: HttpStatus::default(),
        body: None,
        pipeline_id: TEST_PIPELINE_ID,
    };

    assert_eq!(devhttprequest, httprequest);
    assert_eq!(devhttpresponse, httpresponse);
}

#[test]
#[cfg(not(target_os = "windows"))]
fn test_request_and_response_message_from_devtool_without_pipeline_id() {
    let handler =
        move |_: HyperRequest<Incoming>,
              response: &mut HyperResponse<BoxBody<Bytes, hyper::Error>>| {
            response
                .headers_mut()
                .typed_insert(Host::from("foo.bar".parse::<Authority>().unwrap()));
            *response.body_mut() = make_body(b"Yay!".to_vec());
        };
    let (server, url) = make_server(handler);

    let request = RequestBuilder::new(url.clone(), Referrer::NoReferrer)
        .method(Method::GET)
        .destination(Destination::Document)
        .origin(mock_origin())
        .pipeline_id(None)
        .build();

    let (devtools_chan, devtools_port) = unbounded();
    let response = fetch(request, Some(devtools_chan));
    assert!(response.actual_response().status.code().is_success());

    let _ = server.close();

    // notification received from devtools
    assert!(devtools_port.try_recv().is_err());
}

#[test]
fn test_redirected_request_to_devtools() {
    let post_handler =
        move |request: HyperRequest<Incoming>,
              response: &mut HyperResponse<BoxBody<Bytes, hyper::Error>>| {
            assert_eq!(request.method(), Method::GET);
            *response.body_mut() = make_body(b"Yay!".to_vec());
        };
    let (post_server, post_url) = make_server(post_handler);

    let post_redirect_url = post_url.clone();
    let pre_handler =
        move |request: HyperRequest<Incoming>,
              response: &mut HyperResponse<BoxBody<Bytes, hyper::Error>>| {
            assert_eq!(request.method(), Method::POST);
            response.headers_mut().insert(
                header::LOCATION,
                HeaderValue::from_str(&post_redirect_url.to_string()).unwrap(),
            );
            *response.status_mut() = StatusCode::MOVED_PERMANENTLY;
        };
    let (pre_server, pre_url) = make_server(pre_handler);

    let request = RequestBuilder::new(pre_url.clone(), Referrer::NoReferrer)
        .method(Method::POST)
        .destination(Destination::Document)
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .build();

    let (devtools_chan, devtools_port) = unbounded();
    fetch(request, Some(devtools_chan));

    let _ = pre_server.close();
    let _ = post_server.close();

    let devhttprequest = expect_devtools_http_request(&devtools_port);
    let devhttpresponse = expect_devtools_http_response(&devtools_port);

    assert_eq!(devhttprequest.method, Method::POST);
    assert_eq!(devhttprequest.url, pre_url);
    assert_eq!(
        devhttpresponse.status,
        HttpStatus::from(StatusCode::MOVED_PERMANENTLY)
    );

    let devhttprequest = expect_devtools_http_request(&devtools_port);
    let devhttpresponse = expect_devtools_http_response(&devtools_port);

    assert_eq!(devhttprequest.method, Method::GET);
    assert_eq!(devhttprequest.url, post_url);
    assert_eq!(devhttpresponse.status, HttpStatus::default());
}

#[test]
fn test_load_when_redirecting_from_a_post_should_rewrite_next_request_as_get() {
    let post_handler =
        move |request: HyperRequest<Incoming>,
              response: &mut HyperResponse<BoxBody<Bytes, hyper::Error>>| {
            assert_eq!(request.method(), Method::GET);
            *response.body_mut() = make_body(b"Yay!".to_vec());
        };
    let (post_server, post_url) = make_server(post_handler);

    let post_redirect_url = post_url.clone();
    let pre_handler =
        move |request: HyperRequest<Incoming>,
              response: &mut HyperResponse<BoxBody<Bytes, hyper::Error>>| {
            assert_eq!(request.method(), Method::POST);
            response.headers_mut().insert(
                header::LOCATION,
                HeaderValue::from_str(&post_redirect_url.to_string()).unwrap(),
            );
            *response.status_mut() = StatusCode::MOVED_PERMANENTLY;
        };
    let (pre_server, pre_url) = make_server(pre_handler);

    let request = RequestBuilder::new(pre_url.clone(), Referrer::NoReferrer)
        .method(Method::POST)
        .destination(Destination::Document)
        .origin(mock_origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .build();

    let response = fetch(request, None);

    let _ = pre_server.close();
    let _ = post_server.close();

    assert!(response.to_actual().status.code().is_success());
}

#[test]
fn test_load_should_decode_the_response_as_deflate_when_response_headers_have_content_encoding_deflate(
) {
    let handler =
        move |_: HyperRequest<Incoming>,
              response: &mut HyperResponse<BoxBody<Bytes, hyper::Error>>| {
            response.headers_mut().insert(
                header::CONTENT_ENCODING,
                HeaderValue::from_static("deflate"),
            );
            let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
            e.write(b"Yay!").unwrap();
            let encoded_content = e.finish().unwrap();
            *response.body_mut() = make_body(encoded_content);
        };
    let (server, url) = make_server(handler);

    let request = RequestBuilder::new(url.clone(), Referrer::NoReferrer)
        .method(Method::GET)
        .body(None)
        .destination(Destination::Document)
        .origin(mock_origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .build();

    let response = fetch(request, None);

    let _ = server.close();

    let internal_response = response.internal_response.unwrap();
    assert!(internal_response.status.clone().code().is_success());
    assert_eq!(
        *internal_response.body.lock().unwrap(),
        ResponseBody::Done(b"Yay!".to_vec())
    );
}

#[test]
fn test_load_should_decode_the_response_as_gzip_when_response_headers_have_content_encoding_gzip() {
    let handler =
        move |_: HyperRequest<Incoming>,
              response: &mut HyperResponse<BoxBody<Bytes, hyper::Error>>| {
            response
                .headers_mut()
                .insert(header::CONTENT_ENCODING, HeaderValue::from_static("gzip"));
            let mut e = GzEncoder::new(Vec::new(), Compression::default());
            e.write(b"Yay!").unwrap();
            let encoded_content = e.finish().unwrap();
            *response.body_mut() = make_body(encoded_content);
        };
    let (server, url) = make_server(handler);

    let request = RequestBuilder::new(url.clone(), Referrer::NoReferrer)
        .method(Method::GET)
        .body(None)
        .destination(Destination::Document)
        .origin(mock_origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .build();

    let response = fetch(request, None);

    let _ = server.close();

    let internal_response = response.internal_response.unwrap();
    assert!(internal_response.status.clone().code().is_success());
    assert_eq!(
        *internal_response.body.lock().unwrap(),
        ResponseBody::Done(b"Yay!".to_vec())
    );
}

#[test]
fn test_load_doesnt_send_request_body_on_any_redirect() {
    let post_handler =
        move |request: HyperRequest<Incoming>,
              response: &mut HyperResponse<BoxBody<Bytes, hyper::Error>>| {
            assert_eq!(request.method(), Method::GET);
            assert_eq!(request.size_hint().exact(), Some(0));
            *response.body_mut() = make_body(b"Yay!".to_vec());
        };
    let (post_server, post_url) = make_server(post_handler);

    let post_redirect_url = post_url.clone();
    let pre_handler =
        move |request: HyperRequest<Incoming>,
              response: &mut HyperResponse<BoxBody<Bytes, hyper::Error>>| {
            assert_eq!(request.size_hint().exact(), Some(13));
            response.headers_mut().insert(
                header::LOCATION,
                HeaderValue::from_str(&post_redirect_url.to_string()).unwrap(),
            );
            *response.status_mut() = StatusCode::MOVED_PERMANENTLY;
        };
    let (pre_server, pre_url) = make_server(pre_handler);

    let content = b"Body on POST!";
    let request_body = create_request_body_with_content(content.to_vec());

    let request = RequestBuilder::new(pre_url.clone(), Referrer::NoReferrer)
        .body(Some(request_body))
        .method(Method::POST)
        .destination(Destination::Document)
        .origin(mock_origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .build();

    let response = fetch(request, None);

    let _ = pre_server.close();
    let _ = post_server.close();

    assert!(response.to_actual().status.code().is_success());
}

#[test]
fn test_load_doesnt_add_host_to_hsts_list_when_url_is_http_even_if_hsts_headers_are_present() {
    let handler =
        move |_: HyperRequest<Incoming>,
              response: &mut HyperResponse<BoxBody<Bytes, hyper::Error>>| {
            response
                .headers_mut()
                .typed_insert(StrictTransportSecurity::excluding_subdomains(
                    Duration::from_secs(31536000),
                ));
            *response.body_mut() = make_body(b"Yay!".to_vec());
        };
    let (server, url) = make_server(handler);

    let request = RequestBuilder::new(url.clone(), Referrer::NoReferrer)
        .method(Method::GET)
        .body(None)
        .destination(Destination::Document)
        .origin(mock_origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .build();

    let mut context = new_fetch_context(None, None, None);
    let response = fetch_with_context(request, &mut context);

    let _ = server.close();

    assert!(response
        .internal_response
        .unwrap()
        .status
        .code()
        .is_success());
    assert_eq!(
        context
            .state
            .hsts_list
            .read()
            .unwrap()
            .is_host_secure(url.host_str().unwrap()),
        false
    );
}

#[test]
fn test_load_sets_cookies_in_the_resource_manager_when_it_get_set_cookie_header_in_response() {
    let handler =
        move |_: HyperRequest<Incoming>,
              response: &mut HyperResponse<BoxBody<Bytes, hyper::Error>>| {
            response.headers_mut().insert(
                header::SET_COOKIE,
                HeaderValue::from_static("mozillaIs=theBest"),
            );
            *response.body_mut() = make_body(b"Yay!".to_vec());
        };
    let (server, url) = make_server(handler);

    let mut context = new_fetch_context(None, None, None);

    assert_cookie_for_domain(&context.state.cookie_jar, url.as_str(), None);

    let request = RequestBuilder::new(url.clone(), Referrer::NoReferrer)
        .method(Method::GET)
        .body(None)
        .destination(Destination::Document)
        .origin(mock_origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .credentials_mode(CredentialsMode::Include)
        .build();

    let response = fetch_with_context(request, &mut context);

    let _ = server.close();

    assert!(response
        .internal_response
        .unwrap()
        .status
        .code()
        .is_success());

    assert_cookie_for_domain(
        &context.state.cookie_jar,
        url.as_str(),
        Some("mozillaIs=theBest"),
    );
}

#[test]
fn test_load_sets_requests_cookies_header_for_url_by_getting_cookies_from_the_resource_manager() {
    let handler =
        move |request: HyperRequest<Incoming>,
              response: &mut HyperResponse<BoxBody<Bytes, hyper::Error>>| {
            assert_eq!(
                request.headers().get(header::COOKIE).unwrap().as_bytes(),
                b"mozillaIs=theBest"
            );
            *response.body_mut() = make_body(b"Yay!".to_vec());
        };
    let (server, url) = make_server(handler);

    let mut context = new_fetch_context(None, None, None);

    {
        let mut cookie_jar = context.state.cookie_jar.write().unwrap();
        let cookie = ServoCookie::new_wrapped(
            CookiePair::new("mozillaIs".to_owned(), "theBest".to_owned()),
            &url,
            CookieSource::HTTP,
        )
        .unwrap();
        cookie_jar.push(cookie, &url, CookieSource::HTTP);
    }

    let request = RequestBuilder::new(url.clone(), Referrer::NoReferrer)
        .method(Method::GET)
        .body(None)
        .destination(Destination::Document)
        .origin(mock_origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .credentials_mode(CredentialsMode::Include)
        .build();

    let response = fetch_with_context(request, &mut context);

    let _ = server.close();

    assert!(response
        .internal_response
        .unwrap()
        .status
        .code()
        .is_success());
}

#[test]
fn test_load_sends_cookie_if_nonhttp() {
    let handler =
        move |request: HyperRequest<Incoming>,
              response: &mut HyperResponse<BoxBody<Bytes, hyper::Error>>| {
            assert_eq!(
                request.headers().get(header::COOKIE).unwrap().as_bytes(),
                b"mozillaIs=theBest"
            );
            *response.body_mut() = make_body(b"Yay!".to_vec());
        };
    let (server, url) = make_server(handler);

    let mut context = new_fetch_context(None, None, None);

    {
        let mut cookie_jar = context.state.cookie_jar.write().unwrap();
        let cookie = ServoCookie::new_wrapped(
            CookiePair::new("mozillaIs".to_owned(), "theBest".to_owned()),
            &url,
            CookieSource::NonHTTP,
        )
        .unwrap();
        cookie_jar.push(cookie, &url, CookieSource::HTTP);
    }

    let request = RequestBuilder::new(url.clone(), Referrer::NoReferrer)
        .method(Method::GET)
        .body(None)
        .destination(Destination::Document)
        .origin(mock_origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .credentials_mode(CredentialsMode::Include)
        .build();

    let response = fetch_with_context(request, &mut context);

    let _ = server.close();

    assert!(response
        .internal_response
        .unwrap()
        .status
        .code()
        .is_success());
}

#[test]
#[cfg(not(target_os = "windows"))]
fn test_cookie_set_with_httponly_should_not_be_available_using_getcookiesforurl() {
    let handler =
        move |_: HyperRequest<Incoming>,
              response: &mut HyperResponse<BoxBody<Bytes, hyper::Error>>| {
            response.headers_mut().insert(
                header::SET_COOKIE,
                HeaderValue::from_static("mozillaIs=theBest; HttpOnly"),
            );
            *response.body_mut() = make_body(b"Yay!".to_vec());
        };
    let (server, url) = make_server(handler);

    let mut context = new_fetch_context(None, None, None);

    assert_cookie_for_domain(&context.state.cookie_jar, url.as_str(), None);

    let request = RequestBuilder::new(url.clone(), Referrer::NoReferrer)
        .method(Method::GET)
        .body(None)
        .destination(Destination::Document)
        .origin(mock_origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .credentials_mode(CredentialsMode::Include)
        .build();

    let response = fetch_with_context(request, &mut context);

    let _ = server.close();

    assert!(response
        .internal_response
        .unwrap()
        .status
        .code()
        .is_success());

    assert_cookie_for_domain(
        &context.state.cookie_jar,
        url.as_str(),
        Some("mozillaIs=theBest"),
    );
    let mut cookie_jar = context.state.cookie_jar.write().unwrap();
    assert!(cookie_jar
        .cookies_for_url(&url, CookieSource::NonHTTP)
        .is_none());
}

#[test]
#[cfg(not(target_os = "windows"))]
fn test_when_cookie_received_marked_secure_is_ignored_for_http() {
    let handler =
        move |_: HyperRequest<Incoming>,
              response: &mut HyperResponse<BoxBody<Bytes, hyper::Error>>| {
            response.headers_mut().insert(
                header::SET_COOKIE,
                HeaderValue::from_static("mozillaIs=theBest; Secure"),
            );
            *response.body_mut() = make_body(b"Yay!".to_vec());
        };
    let (server, url) = make_server(handler);

    let mut context = new_fetch_context(None, None, None);

    assert_cookie_for_domain(&context.state.cookie_jar, url.as_str(), None);

    let request = RequestBuilder::new(url.clone(), Referrer::NoReferrer)
        .method(Method::GET)
        .body(None)
        .destination(Destination::Document)
        .origin(mock_origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .credentials_mode(CredentialsMode::Include)
        .build();

    let response = fetch_with_context(request, &mut context);

    let _ = server.close();

    assert!(response.actual_response().status.code().is_success());

    assert_cookie_for_domain(&context.state.cookie_jar, url.as_str(), None);
}

#[test]
fn test_load_sets_content_length_to_length_of_request_body() {
    let content = b"This is a request body";
    let handler =
        move |request: HyperRequest<Incoming>,
              response: &mut HyperResponse<BoxBody<Bytes, hyper::Error>>| {
            let content_length = ContentLength(content.len() as u64);
            assert_eq!(
                request.headers().typed_get::<ContentLength>(),
                Some(content_length)
            );
            *response.body_mut() = make_body(content.to_vec());
        };
    let (server, url) = make_server(handler);

    let request_body = create_request_body_with_content(content.to_vec());

    let request = RequestBuilder::new(url.clone(), Referrer::NoReferrer)
        .method(Method::POST)
        .body(Some(request_body))
        .destination(Destination::Document)
        .origin(mock_origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .build();

    let response = fetch(request, None);

    let _ = server.close();

    assert!(response
        .internal_response
        .unwrap()
        .status
        .code()
        .is_success());
}

#[test]
fn test_load_uses_explicit_accept_from_headers_in_load_data() {
    let handler =
        move |request: HyperRequest<Incoming>,
              response: &mut HyperResponse<BoxBody<Bytes, hyper::Error>>| {
            assert_eq!(
                request
                    .headers()
                    .get(header::ACCEPT)
                    .unwrap()
                    .to_str()
                    .unwrap(),
                "text/html"
            );
            *response.body_mut() = make_body(b"Yay!".to_vec());
        };
    let (server, url) = make_server(handler);

    let mut accept_headers = HeaderMap::new();
    accept_headers.insert(header::ACCEPT, HeaderValue::from_static("text/html"));
    let request = RequestBuilder::new(url.clone(), Referrer::NoReferrer)
        .method(Method::GET)
        .headers(accept_headers)
        .destination(Destination::Document)
        .origin(mock_origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .build();

    let response = fetch(request, None);

    let _ = server.close();

    assert!(response
        .internal_response
        .unwrap()
        .status
        .code()
        .is_success());
}

#[test]
fn test_load_sets_default_accept_to_html_xhtml_xml_and_then_anything_else() {
    let handler =
        move |request: HyperRequest<Incoming>,
              response: &mut HyperResponse<BoxBody<Bytes, hyper::Error>>| {
            assert_eq!(
                request
                    .headers()
                    .get(header::ACCEPT)
                    .unwrap()
                    .to_str()
                    .unwrap(),
                "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"
            );
            *response.body_mut() = make_body(b"Yay!".to_vec());
        };
    let (server, url) = make_server(handler);

    let request = RequestBuilder::new(url.clone(), Referrer::NoReferrer)
        .method(Method::GET)
        .destination(Destination::Document)
        .origin(mock_origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .build();

    let response = fetch(request, None);

    let _ = server.close();

    assert!(response
        .internal_response
        .unwrap()
        .status
        .code()
        .is_success());
}

#[test]
fn test_load_uses_explicit_accept_encoding_from_load_data_headers() {
    let handler =
        move |request: HyperRequest<Incoming>,
              response: &mut HyperResponse<BoxBody<Bytes, hyper::Error>>| {
            assert_eq!(
                request
                    .headers()
                    .get(header::ACCEPT_ENCODING)
                    .unwrap()
                    .to_str()
                    .unwrap(),
                "chunked"
            );
            *response.body_mut() = make_body(b"Yay!".to_vec());
        };
    let (server, url) = make_server(handler);

    let mut accept_encoding_headers = HeaderMap::new();
    accept_encoding_headers.insert(header::ACCEPT_ENCODING, HeaderValue::from_static("chunked"));
    let request = RequestBuilder::new(url.clone(), Referrer::NoReferrer)
        .method(Method::GET)
        .headers(accept_encoding_headers)
        .destination(Destination::Document)
        .origin(mock_origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .build();

    let response = fetch(request, None);

    let _ = server.close();

    assert!(response
        .internal_response
        .unwrap()
        .status
        .code()
        .is_success());
}

#[test]
fn test_load_sets_default_accept_encoding_to_gzip_and_deflate() {
    let handler =
        move |request: HyperRequest<Incoming>,
              response: &mut HyperResponse<BoxBody<Bytes, hyper::Error>>| {
            assert_eq!(
                request
                    .headers()
                    .get(header::ACCEPT_ENCODING)
                    .unwrap()
                    .to_str()
                    .unwrap(),
                "gzip, deflate, br"
            );
            *response.body_mut() = make_body(b"Yay!".to_vec());
        };
    let (server, url) = make_server(handler);

    let request = RequestBuilder::new(url.clone(), Referrer::NoReferrer)
        .method(Method::GET)
        .destination(Destination::Document)
        .origin(mock_origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .build();

    let response = fetch(request, None);

    let _ = server.close();

    assert!(response
        .internal_response
        .unwrap()
        .status
        .code()
        .is_success());
}

#[test]
fn test_load_errors_when_there_a_redirect_loop() {
    let url_b_for_a = Arc::new(Mutex::new(None::<ServoUrl>));
    let url_b_for_a_clone = url_b_for_a.clone();
    let handler_a =
        move |_: HyperRequest<Incoming>,
              response: &mut HyperResponse<BoxBody<Bytes, hyper::Error>>| {
            response.headers_mut().insert(
                header::LOCATION,
                HeaderValue::from_str(
                    &url_b_for_a_clone
                        .lock()
                        .unwrap()
                        .as_ref()
                        .unwrap()
                        .to_string(),
                )
                .unwrap(),
            );
            *response.status_mut() = StatusCode::MOVED_PERMANENTLY;
        };
    let (server_a, url_a) = make_server(handler_a);

    let url_a_for_b = url_a.clone();
    let handler_b =
        move |_: HyperRequest<Incoming>,
              response: &mut HyperResponse<BoxBody<Bytes, hyper::Error>>| {
            response.headers_mut().insert(
                header::LOCATION,
                HeaderValue::from_str(&url_a_for_b.to_string()).unwrap(),
            );
            *response.status_mut() = StatusCode::MOVED_PERMANENTLY;
        };
    let (server_b, url_b) = make_server(handler_b);

    *url_b_for_a.lock().unwrap() = Some(url_b.clone());

    let request = RequestBuilder::new(url_a.clone(), Referrer::NoReferrer)
        .method(Method::GET)
        .destination(Destination::Document)
        .origin(mock_origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .build();

    let response = fetch(request, None);

    let _ = server_a.close();
    let _ = server_b.close();

    assert_eq!(
        response.get_network_error(),
        Some(&NetworkError::Internal("Too many redirects".to_owned()))
    );
}

#[test]
fn test_load_succeeds_with_a_redirect_loop() {
    let url_b_for_a = Arc::new(Mutex::new(None::<ServoUrl>));
    let url_b_for_a_clone = url_b_for_a.clone();
    let handled_a = AtomicBool::new(false);
    let handler_a =
        move |_: HyperRequest<Incoming>,
              response: &mut HyperResponse<BoxBody<Bytes, hyper::Error>>| {
            if !handled_a.swap(true, Ordering::SeqCst) {
                response.headers_mut().insert(
                    header::LOCATION,
                    HeaderValue::from_str(
                        &url_b_for_a_clone
                            .lock()
                            .unwrap()
                            .as_ref()
                            .unwrap()
                            .to_string(),
                    )
                    .unwrap(),
                );
                *response.status_mut() = StatusCode::MOVED_PERMANENTLY;
            } else {
                *response.body_mut() = make_body(b"Success".to_vec());
            }
        };
    let (server_a, url_a) = make_server(handler_a);

    let url_a_for_b = url_a.clone();
    let handler_b =
        move |_: HyperRequest<Incoming>,
              response: &mut HyperResponse<BoxBody<Bytes, hyper::Error>>| {
            response.headers_mut().insert(
                header::LOCATION,
                HeaderValue::from_str(&url_a_for_b.to_string()).unwrap(),
            );
            *response.status_mut() = StatusCode::MOVED_PERMANENTLY;
        };
    let (server_b, url_b) = make_server(handler_b);

    *url_b_for_a.lock().unwrap() = Some(url_b.clone());

    let request = RequestBuilder::new(url_a.clone(), Referrer::NoReferrer)
        .method(Method::GET)
        .destination(Destination::Document)
        .origin(mock_origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .build();

    let response = fetch(request, None);

    let _ = server_a.close();
    let _ = server_b.close();

    let response = response.to_actual();
    assert_eq!(response.url_list, [url_a.clone(), url_b, url_a]);
    assert_eq!(
        *response.body.lock().unwrap(),
        ResponseBody::Done(b"Success".to_vec())
    );
}

#[test]
fn test_load_follows_a_redirect() {
    let post_handler =
        move |request: HyperRequest<Incoming>,
              response: &mut HyperResponse<BoxBody<Bytes, hyper::Error>>| {
            assert_eq!(request.method(), Method::GET);
            *response.body_mut() = make_body(b"Yay!".to_vec());
        };
    let (post_server, post_url) = make_server(post_handler);

    let post_redirect_url = post_url.clone();
    let pre_handler =
        move |request: HyperRequest<Incoming>,
              response: &mut HyperResponse<BoxBody<Bytes, hyper::Error>>| {
            assert_eq!(request.method(), Method::GET);
            response.headers_mut().insert(
                header::LOCATION,
                HeaderValue::from_str(&post_redirect_url.to_string()).unwrap(),
            );
            *response.status_mut() = StatusCode::MOVED_PERMANENTLY;
        };
    let (pre_server, pre_url) = make_server(pre_handler);

    let request = RequestBuilder::new(pre_url.clone(), Referrer::NoReferrer)
        .method(Method::GET)
        .destination(Destination::Document)
        .origin(mock_origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .build();

    let response = fetch(request, None);

    let _ = pre_server.close();
    let _ = post_server.close();

    let internal_response = response.internal_response.unwrap();
    assert!(internal_response.status.clone().code().is_success());
    assert_eq!(
        *internal_response.body.lock().unwrap(),
        ResponseBody::Done(b"Yay!".to_vec())
    );
}

#[test]
fn test_redirect_from_x_to_y_provides_y_cookies_from_y() {
    let shared_url_y = Arc::new(Mutex::new(None::<ServoUrl>));
    let shared_url_y_clone = shared_url_y.clone();
    let handler =
        move |request: HyperRequest<Incoming>,
              response: &mut HyperResponse<BoxBody<Bytes, hyper::Error>>| {
            let path = request.uri().path();
            if path == "/com/" {
                assert_eq!(
                    request.headers().get(header::COOKIE).unwrap().as_bytes(),
                    b"mozillaIsNot=dotOrg"
                );
                let location = shared_url_y.lock().unwrap().as_ref().unwrap().to_string();
                response.headers_mut().insert(
                    header::LOCATION,
                    HeaderValue::from_str(&location.to_string()).unwrap(),
                );
                *response.status_mut() = StatusCode::MOVED_PERMANENTLY;
            } else if path == "/org/" {
                assert_eq!(
                    request.headers().get(header::COOKIE).unwrap().as_bytes(),
                    b"mozillaIs=theBest"
                );
                *response.body_mut() = make_body(b"Yay!".to_vec());
            } else {
                panic!("unexpected path {:?}", path)
            }
        };
    let (server, url) = make_server(handler);
    let port = url.port().unwrap();

    assert_eq!(url.host_str(), Some("localhost"));
    let ip = "127.0.0.1".parse().unwrap();
    let mut host_table = HashMap::new();
    host_table.insert("mozilla.com".to_owned(), ip);
    host_table.insert("mozilla.org".to_owned(), ip);

    replace_host_table(host_table);

    let url_x = ServoUrl::parse(&format!("http://mozilla.com:{}/com/", port)).unwrap();
    let url_y = ServoUrl::parse(&format!("http://mozilla.org:{}/org/", port)).unwrap();
    *shared_url_y_clone.lock().unwrap() = Some(url_y.clone());

    let mut context = new_fetch_context(None, None, None);
    {
        let mut cookie_jar = context.state.cookie_jar.write().unwrap();
        let cookie_x = ServoCookie::new_wrapped(
            CookiePair::new("mozillaIsNot".to_owned(), "dotOrg".to_owned()),
            &url_x,
            CookieSource::HTTP,
        )
        .unwrap();

        cookie_jar.push(cookie_x, &url_x, CookieSource::HTTP);

        let cookie_y = ServoCookie::new_wrapped(
            CookiePair::new("mozillaIs".to_owned(), "theBest".to_owned()),
            &url_y,
            CookieSource::HTTP,
        )
        .unwrap();
        cookie_jar.push(cookie_y, &url_y, CookieSource::HTTP);
    }

    let request = RequestBuilder::new(url_x.clone(), Referrer::NoReferrer)
        .method(Method::GET)
        .destination(Destination::Document)
        .origin(mock_origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .credentials_mode(CredentialsMode::Include)
        .build();

    let response = fetch_with_context(request, &mut context);

    let _ = server.close();

    let internal_response = response.internal_response.unwrap();
    assert!(internal_response.status.clone().code().is_success());
    assert_eq!(
        *internal_response.body.lock().unwrap(),
        ResponseBody::Done(b"Yay!".to_vec())
    );
}

#[test]
fn test_redirect_from_x_to_x_provides_x_with_cookie_from_first_response() {
    let handler =
        move |request: HyperRequest<Incoming>,
              response: &mut HyperResponse<BoxBody<Bytes, hyper::Error>>| {
            let path = request.uri().path();
            if path == "/initial/" {
                response.headers_mut().insert(
                    header::SET_COOKIE,
                    HeaderValue::from_static("mozillaIs=theBest; path=/;"),
                );
                let location = "/subsequent/".to_string();
                response.headers_mut().insert(
                    header::LOCATION,
                    HeaderValue::from_str(&location.to_string()).unwrap(),
                );
                *response.status_mut() = StatusCode::MOVED_PERMANENTLY;
            } else if path == "/subsequent/" {
                assert_eq!(
                    request.headers().get(header::COOKIE).unwrap().as_bytes(),
                    b"mozillaIs=theBest"
                );
                *response.body_mut() = make_body(b"Yay!".to_vec());
            } else {
                panic!("unexpected path {:?}", path)
            }
        };
    let (server, url) = make_server(handler);

    let url = url.join("/initial/").unwrap();

    let request = RequestBuilder::new(url.clone(), Referrer::NoReferrer)
        .method(Method::GET)
        .destination(Destination::Document)
        .origin(mock_origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .credentials_mode(CredentialsMode::Include)
        .build();

    let response = fetch(request, None);

    let _ = server.close();

    let internal_response = response.internal_response.unwrap();
    assert!(internal_response.status.clone().code().is_success());
    assert_eq!(
        *internal_response.body.lock().unwrap(),
        ResponseBody::Done(b"Yay!".to_vec())
    );
}

#[test]
fn test_if_auth_creds_not_in_url_but_in_cache_it_sets_it() {
    let handler =
        move |request: HyperRequest<Incoming>,
              _response: &mut HyperResponse<BoxBody<Bytes, hyper::Error>>| {
            let expected = Authorization::basic("username", "test");
            assert_eq!(
                request.headers().typed_get::<Authorization<Basic>>(),
                Some(expected)
            );
        };
    let (server, url) = make_server(handler);

    let request = RequestBuilder::new(url.clone(), Referrer::NoReferrer)
        .method(Method::GET)
        .body(None)
        .destination(Destination::Document)
        .origin(mock_origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .credentials_mode(CredentialsMode::Include)
        .build();

    let mut context = new_fetch_context(None, None, None);

    let auth_entry = AuthCacheEntry {
        user_name: "username".to_owned(),
        password: "test".to_owned(),
    };

    context
        .state
        .auth_cache
        .write()
        .unwrap()
        .entries
        .insert(url.origin().clone().ascii_serialization(), auth_entry);

    let response = fetch_with_context(request, &mut context);

    let _ = server.close();

    assert!(response
        .internal_response
        .unwrap()
        .status
        .code()
        .is_success());
}

#[test]
fn test_auth_ui_needs_www_auth() {
    let handler =
        move |_: HyperRequest<Incoming>,
              response: &mut HyperResponse<BoxBody<Bytes, hyper::Error>>| {
            *response.status_mut() = StatusCode::UNAUTHORIZED;
        };
    let (server, url) = make_server(handler);

    let request = RequestBuilder::new(url.clone(), Referrer::NoReferrer)
        .method(Method::GET)
        .body(None)
        .destination(Destination::Document)
        .origin(mock_origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .credentials_mode(CredentialsMode::Include)
        .build();

    let response = fetch(request, None);

    let _ = server.close();

    assert_eq!(
        response.internal_response.unwrap().status,
        StatusCode::UNAUTHORIZED
    );
}

#[test]
fn test_determine_requests_referrer_shorter_than_4k() {
    let url_str = "http://username:password@example.com/such/short/referer?query#fragment";
    let referrer_source = ServoUrl::parse(url_str).unwrap();
    let current_url = ServoUrl::parse("http://example.com/current/url").unwrap();
    let referrer_policy = ReferrerPolicy::UnsafeUrl;

    let referer = determine_requests_referrer(referrer_policy, referrer_source, current_url);

    assert_eq!(
        referer.unwrap().as_str(),
        "http://example.com/such/short/referer?query"
    );
}

#[test]
fn test_determine_requests_referrer_longer_than_4k() {
    let long_url_str = format!(
        "http://username:password@example.com/such/{}/referer?query#fragment",
        "long".repeat(1024)
    );
    let referrer_source = ServoUrl::parse(&long_url_str).unwrap();
    let current_url = ServoUrl::parse("http://example.com/current/url").unwrap();
    let referrer_policy = ReferrerPolicy::UnsafeUrl;

    let referer = determine_requests_referrer(referrer_policy, referrer_source, current_url);

    assert_eq!(referer.unwrap().as_str(), "http://example.com/");
}

#[test]
fn test_fetch_compressed_response_update_count() {
    // contents of ../../tests/wpt/tests/fetch/content-encoding/br/resources/foo.text.br
    const DATA_BROTLI_COMPRESSED: [u8; 15] = [
        0xe1, 0x18, 0x48, 0xc1, 0x2f, 0x65, 0xf6, 0x16, 0x9f, 0x05, 0x01, 0xbb, 0x20, 0x00, 0x06,
    ];
    const DATA_DECOMPRESSED_LEN: usize = 10500;

    let handler =
        move |_: HyperRequest<Incoming>,
              response: &mut HyperResponse<BoxBody<Bytes, hyper::Error>>| {
            response
                .headers_mut()
                .insert(header::CONTENT_ENCODING, HeaderValue::from_static("br"));
            *response.body_mut() = make_body(DATA_BROTLI_COMPRESSED.to_vec());
        };
    let (server, url) = make_server(handler);

    let request = RequestBuilder::new(url.clone(), Referrer::NoReferrer)
        .method(Method::GET)
        .body(None)
        .destination(Destination::Document)
        .origin(mock_origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .build();

    struct FetchResponseCollector {
        sender: Option<tokio::sync::oneshot::Sender<usize>>,
        update_count: usize,
    }
    impl FetchTaskTarget for FetchResponseCollector {
        fn process_request_body(&mut self, _: &Request) {}
        fn process_request_eof(&mut self, _: &Request) {}
        fn process_response(&mut self, _: &Request, _: &Response) {}
        fn process_response_chunk(&mut self, _: &Request, _: Vec<u8>) {
            self.update_count += 1;
        }
        /// Fired when the response is fully fetched
        fn process_response_eof(&mut self, _: &Request, _: &Response) {
            let _ = self.sender.take().unwrap().send(self.update_count);
        }
    }

    let (sender, receiver) = tokio::sync::oneshot::channel();
    let mut target = FetchResponseCollector {
        sender: Some(sender),
        update_count: 0,
    };
    let response_update_count = crate::HANDLE.block_on(async move {
        methods::fetch(
            request,
            &mut target,
            &mut new_fetch_context(None, None, None),
        )
        .await;
        receiver.await.unwrap()
    });

    server.close();

    const EXPECTED_UPDATE_COUNT: usize =
        (DATA_DECOMPRESSED_LEN + DECODER_BUFFER_SIZE - 1) / DECODER_BUFFER_SIZE;
    assert_eq!(response_update_count, EXPECTED_UPDATE_COUNT);
}

#[test]
fn test_origin_serialization_compatability() {
    let ensure_serialiations_match = |url_string| {
        let url = Url::parse(url_string).unwrap();
        let origin = ImmutableOrigin::new(url.origin());
        let serialized = format!("{}", serialize_origin(&origin));
        assert_eq!(serialized, origin.ascii_serialization());
    };

    ensure_serialiations_match("https://example.com");
    ensure_serialiations_match("https://example.com:443");

    ensure_serialiations_match("http://example.com");
    ensure_serialiations_match("http://example.com:80");

    ensure_serialiations_match("https://example.com:1234");
    ensure_serialiations_match("http://example.com:1234");

    ensure_serialiations_match("data:,dataurltexta");
}

#[test]
fn test_user_credentials_prompt_when_proxy_authentication_is_required() {
    let handler =
        move |request: HyperRequest<Incoming>,
              response: &mut HyperResponse<BoxBody<Bytes, hyper::Error>>| {
            let expected = Authorization::basic("username", "test");
            if let Some(credentials) = request.headers().typed_get::<Authorization<Basic>>() {
                if credentials == expected {
                    *response.status_mut() = StatusCode::OK;
                } else {
                    *response.status_mut() = StatusCode::UNAUTHORIZED;
                }
            } else {
                *response.status_mut() = StatusCode::PROXY_AUTHENTICATION_REQUIRED;
            }
        };
    let (server, url) = make_server(handler);

    let request = RequestBuilder::new(url.clone(), Referrer::NoReferrer)
        .method(Method::GET)
        .body(None)
        .destination(Destination::Document)
        .origin(mock_origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .credentials_mode(CredentialsMode::Include)
        .build();

    let (embedder_proxy, embedder_receiver) = create_embedder_proxy_and_receiver();
    let _ = receive_credential_prompt_msgs(
        embedder_receiver,
        Some("username".to_string()),
        Some("test".to_string()),
    );

    let mut context = new_fetch_context(None, Some(embedder_proxy), None);

    let response = fetch_with_context(request, &mut context);

    let _ = server.close();

    assert!(response
        .internal_response
        .unwrap()
        .status
        .code()
        .is_success());
}

#[test]
fn test_prompt_credentials_when_client_receives_unauthorized_response() {
    let handler =
        move |request: HyperRequest<Incoming>,
              response: &mut HyperResponse<BoxBody<Bytes, hyper::Error>>| {
            let expected = Authorization::basic("username", "test");
            if let Some(credentials) = request.headers().typed_get::<Authorization<Basic>>() {
                if credentials == expected {
                    *response.status_mut() = StatusCode::OK;
                } else {
                    *response.status_mut() = StatusCode::UNAUTHORIZED;
                }
            } else {
                *response.status_mut() = StatusCode::UNAUTHORIZED;
            }
        };
    let (server, url) = make_server(handler);

    let request = RequestBuilder::new(url.clone(), Referrer::NoReferrer)
        .method(Method::GET)
        .body(None)
        .destination(Destination::Document)
        .origin(mock_origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .credentials_mode(CredentialsMode::Include)
        .build();

    let (embedder_proxy, embedder_receiver) = create_embedder_proxy_and_receiver();
    let _ = receive_credential_prompt_msgs(
        embedder_receiver,
        Some("username".to_string()),
        Some("test".to_string()),
    );
    let mut context = new_fetch_context(None, Some(embedder_proxy), None);

    let response = fetch_with_context(request, &mut context);

    server.close();

    assert!(response
        .internal_response
        .unwrap()
        .status
        .code()
        .is_success());
}

#[test]
fn test_prompt_credentials_user_cancels_dialog_input() {
    let handler =
        move |request: HyperRequest<Incoming>,
              response: &mut HyperResponse<BoxBody<Bytes, hyper::Error>>| {
            let expected = Authorization::basic("username", "test");
            if let Some(credentials) = request.headers().typed_get::<Authorization<Basic>>() {
                if credentials == expected {
                    *response.status_mut() = StatusCode::OK;
                } else {
                    *response.status_mut() = StatusCode::UNAUTHORIZED;
                }
            } else {
                *response.status_mut() = StatusCode::UNAUTHORIZED;
            }
        };
    let (server, url) = make_server(handler);

    let request = RequestBuilder::new(url.clone(), Referrer::NoReferrer)
        .method(Method::GET)
        .body(None)
        .destination(Destination::Document)
        .origin(mock_origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .credentials_mode(CredentialsMode::Include)
        .build();

    let (embedder_proxy, embedder_receiver) = create_embedder_proxy_and_receiver();
    let _ = receive_credential_prompt_msgs(embedder_receiver, None, None);
    let mut context = new_fetch_context(None, Some(embedder_proxy), None);

    let response = fetch_with_context(request, &mut context);

    server.close();

    assert!(response
        .internal_response
        .unwrap()
        .status
        .code()
        .is_client_error());
}

#[test]
fn test_prompt_credentials_user_input_incorrect_credentials() {
    let handler =
        move |request: HyperRequest<Incoming>,
              response: &mut HyperResponse<BoxBody<Bytes, hyper::Error>>| {
            let expected = Authorization::basic("username", "test");
            if let Some(credentials) = request.headers().typed_get::<Authorization<Basic>>() {
                if credentials == expected {
                    *response.status_mut() = StatusCode::OK;
                } else {
                    *response.status_mut() = StatusCode::UNAUTHORIZED;
                }
            } else {
                *response.status_mut() = StatusCode::UNAUTHORIZED;
            }
        };
    let (server, url) = make_server(handler);

    let request = RequestBuilder::new(url.clone(), Referrer::NoReferrer)
        .method(Method::GET)
        .body(None)
        .destination(Destination::Document)
        .origin(mock_origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .credentials_mode(CredentialsMode::Include)
        .build();

    let (embedder_proxy, embedder_receiver) = create_embedder_proxy_and_receiver();
    let _ = receive_credential_prompt_msgs(
        embedder_receiver,
        Some("test".to_string()),
        Some("test".to_string()),
    );
    let mut context = new_fetch_context(None, Some(embedder_proxy), None);

    let response = fetch_with_context(request, &mut context);

    server.close();

    assert!(response
        .internal_response
        .unwrap()
        .status
        .code()
        .is_client_error());
}
