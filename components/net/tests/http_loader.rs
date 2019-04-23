/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::fetch;
use crate::fetch_with_context;
use crate::make_server;
use crate::new_fetch_context;
use cookie_rs::Cookie as CookiePair;
use crossbeam_channel::{unbounded, Receiver};
use devtools_traits::HttpRequest as DevtoolsHttpRequest;
use devtools_traits::HttpResponse as DevtoolsHttpResponse;
use devtools_traits::{ChromeToDevtoolsControlMsg, DevtoolsControlMsg, NetworkEvent};
use flate2::write::{DeflateEncoder, GzEncoder};
use flate2::Compression;
use futures::{self, Future, Stream};
use headers_core::HeaderMapExt;
use headers_ext::{
    AccessControlAllowOrigin, Authorization, Basic, ContentLength, Date, Host, Origin,
};
use headers_ext::{StrictTransportSecurity, UserAgent};
use http::header::{self, HeaderMap, HeaderValue};
use http::uri::Authority;
use http::{Method, StatusCode};
use hyper::body::Body;
use hyper::{Request as HyperRequest, Response as HyperResponse};
use msg::constellation_msg::TEST_PIPELINE_ID;
use net::cookie::Cookie;
use net::cookie_storage::CookieStorage;
use net::resource_thread::AuthCacheEntry;
use net::test::replace_host_table;
use net_traits::request::{CredentialsMode, Destination, RequestBuilder, RequestMode};
use net_traits::response::ResponseBody;
use net_traits::{CookieSource, NetworkError};
use servo_url::{ImmutableOrigin, ServoUrl};
use std::collections::HashMap;
use std::io::Write;
use std::str;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

fn mock_origin() -> ImmutableOrigin {
    ServoUrl::parse("http://servo.org").unwrap().origin()
}

fn read_response(req: HyperRequest<Body>) -> impl Future<Item = String, Error = ()> {
    req.into_body()
        .concat2()
        .and_then(|body| futures::future::ok(str::from_utf8(&body).unwrap().to_owned()))
        .map_err(|_| ())
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

#[test]
fn test_check_default_headers_loaded_in_every_request() {
    let expected_headers = Arc::new(Mutex::new(None));
    let expected_headers_clone = expected_headers.clone();
    let handler = move |request: HyperRequest<Body>, _: &mut HyperResponse<Body>| {
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
        HeaderValue::from_static(
            "text/html, application/xhtml+xml, application/xml; q=0.9, */*; q=0.8",
        ),
    );

    headers.insert(
        header::ACCEPT_LANGUAGE,
        HeaderValue::from_static("en-US, en; q=0.5"),
    );

    headers.typed_insert::<UserAgent>(crate::DEFAULT_USER_AGENT.parse().unwrap());

    *expected_headers.lock().unwrap() = Some(headers.clone());

    // Testing for method.GET
    let mut request = RequestBuilder::new(url.clone())
        .method(Method::GET)
        .destination(Destination::Document)
        .origin(url.clone().origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .build();

    let response = fetch(&mut request, None);
    assert!(response
        .internal_response
        .unwrap()
        .status
        .unwrap()
        .0
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
    let mut request = RequestBuilder::new(url.clone())
        .method(Method::POST)
        .destination(Destination::Document)
        .origin(url.clone().origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .build();

    let response = fetch(&mut request, None);
    assert!(response
        .internal_response
        .unwrap()
        .status
        .unwrap()
        .0
        .is_success());

    let _ = server.close();
}

#[test]
fn test_load_when_request_is_not_get_or_head_and_there_is_no_body_content_length_should_be_set_to_0(
) {
    let handler = move |request: HyperRequest<Body>, _: &mut HyperResponse<Body>| {
        assert_eq!(
            request.headers().typed_get::<ContentLength>(),
            Some(ContentLength(0))
        );
    };
    let (server, url) = make_server(handler);

    let mut request = RequestBuilder::new(url.clone())
        .method(Method::POST)
        .body(None)
        .destination(Destination::Document)
        .origin(mock_origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .build();

    let response = fetch(&mut request, None);
    assert!(response
        .internal_response
        .unwrap()
        .status
        .unwrap()
        .0
        .is_success());

    let _ = server.close();
}

#[test]
fn test_request_and_response_data_with_network_messages() {
    let handler = move |_: HyperRequest<Body>, response: &mut HyperResponse<Body>| {
        response
            .headers_mut()
            .typed_insert(Host::from("foo.bar".parse::<Authority>().unwrap()));
        *response.body_mut() = b"Yay!".to_vec().into();
    };
    let (server, url) = make_server(handler);

    let mut request_headers = HeaderMap::new();
    request_headers.typed_insert(Host::from("bar.foo".parse::<Authority>().unwrap()));
    let mut request = RequestBuilder::new(url.clone())
        .method(Method::GET)
        .headers(request_headers)
        .body(None)
        .destination(Destination::Document)
        .origin(mock_origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .build();

    let (devtools_chan, devtools_port) = unbounded();
    let response = fetch(&mut request, Some(devtools_chan));
    assert!(response
        .internal_response
        .unwrap()
        .status
        .unwrap()
        .0
        .is_success());

    let _ = server.close();

    // notification received from devtools
    let devhttprequest = expect_devtools_http_request(&devtools_port);
    let devhttpresponse = expect_devtools_http_response(&devtools_port);

    //Creating default headers for request
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
        HeaderValue::from_static(
            "text/html, application/xhtml+xml, application/xml; q=0.9, */*; q=0.8",
        ),
    );

    headers.insert(
        header::ACCEPT_LANGUAGE,
        HeaderValue::from_static("en-US, en; q=0.5"),
    );

    headers.typed_insert::<UserAgent>(crate::DEFAULT_USER_AGENT.parse().unwrap());

    let httprequest = DevtoolsHttpRequest {
        url: url,
        method: Method::GET,
        headers: headers,
        body: Some(b"".to_vec()),
        pipeline_id: TEST_PIPELINE_ID,
        startedDateTime: devhttprequest.startedDateTime,
        timeStamp: devhttprequest.timeStamp,
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
        status: Some((200, b"OK".to_vec())),
        body: None,
        pipeline_id: TEST_PIPELINE_ID,
    };

    assert_eq!(devhttprequest, httprequest);
    assert_eq!(devhttpresponse, httpresponse);
}

#[test]
fn test_request_and_response_message_from_devtool_without_pipeline_id() {
    let handler = move |_: HyperRequest<Body>, response: &mut HyperResponse<Body>| {
        response
            .headers_mut()
            .typed_insert(Host::from("foo.bar".parse::<Authority>().unwrap()));
        *response.body_mut() = b"Yay!".to_vec().into();
    };
    let (server, url) = make_server(handler);

    let mut request = RequestBuilder::new(url.clone())
        .method(Method::GET)
        .destination(Destination::Document)
        .origin(mock_origin())
        .pipeline_id(None)
        .build();

    let (devtools_chan, devtools_port) = unbounded();
    let response = fetch(&mut request, Some(devtools_chan));
    assert!(response
        .internal_response
        .unwrap()
        .status
        .unwrap()
        .0
        .is_success());

    let _ = server.close();

    // notification received from devtools
    assert!(devtools_port.try_recv().is_err());
}

#[test]
fn test_redirected_request_to_devtools() {
    let post_handler = move |request: HyperRequest<Body>, response: &mut HyperResponse<Body>| {
        assert_eq!(request.method(), Method::GET);
        *response.body_mut() = b"Yay!".to_vec().into();
    };
    let (post_server, post_url) = make_server(post_handler);

    let post_redirect_url = post_url.clone();
    let pre_handler = move |request: HyperRequest<Body>, response: &mut HyperResponse<Body>| {
        assert_eq!(request.method(), Method::POST);
        response.headers_mut().insert(
            header::LOCATION,
            HeaderValue::from_str(&post_redirect_url.to_string()).unwrap(),
        );
        *response.status_mut() = StatusCode::MOVED_PERMANENTLY;
    };
    let (pre_server, pre_url) = make_server(pre_handler);

    let mut request = RequestBuilder::new(pre_url.clone())
        .method(Method::POST)
        .destination(Destination::Document)
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .build();

    let (devtools_chan, devtools_port) = unbounded();
    fetch(&mut request, Some(devtools_chan));

    let _ = pre_server.close();
    let _ = post_server.close();

    let devhttprequest = expect_devtools_http_request(&devtools_port);
    let devhttpresponse = expect_devtools_http_response(&devtools_port);

    assert_eq!(devhttprequest.method, Method::POST);
    assert_eq!(devhttprequest.url, pre_url);
    assert_eq!(
        devhttpresponse.status,
        Some((301, b"Moved Permanently".to_vec()))
    );

    let devhttprequest = expect_devtools_http_request(&devtools_port);
    let devhttpresponse = expect_devtools_http_response(&devtools_port);

    assert_eq!(devhttprequest.method, Method::GET);
    assert_eq!(devhttprequest.url, post_url);
    assert_eq!(devhttpresponse.status, Some((200, b"OK".to_vec())));
}

#[test]
fn test_load_when_redirecting_from_a_post_should_rewrite_next_request_as_get() {
    let post_handler = move |request: HyperRequest<Body>, response: &mut HyperResponse<Body>| {
        assert_eq!(request.method(), Method::GET);
        *response.body_mut() = b"Yay!".to_vec().into();
    };
    let (post_server, post_url) = make_server(post_handler);

    let post_redirect_url = post_url.clone();
    let pre_handler = move |request: HyperRequest<Body>, response: &mut HyperResponse<Body>| {
        assert_eq!(request.method(), Method::POST);
        response.headers_mut().insert(
            header::LOCATION,
            HeaderValue::from_str(&post_redirect_url.to_string()).unwrap(),
        );
        *response.status_mut() = StatusCode::MOVED_PERMANENTLY;
    };
    let (pre_server, pre_url) = make_server(pre_handler);

    let mut request = RequestBuilder::new(pre_url.clone())
        .method(Method::POST)
        .destination(Destination::Document)
        .origin(mock_origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .build();

    let response = fetch(&mut request, None);

    let _ = pre_server.close();
    let _ = post_server.close();

    assert!(response.to_actual().status.unwrap().0.is_success());
}

#[test]
fn test_load_should_decode_the_response_as_deflate_when_response_headers_have_content_encoding_deflate(
) {
    let handler = move |_: HyperRequest<Body>, response: &mut HyperResponse<Body>| {
        response.headers_mut().insert(
            header::CONTENT_ENCODING,
            HeaderValue::from_static("deflate"),
        );
        let mut e = DeflateEncoder::new(Vec::new(), Compression::default());
        e.write(b"Yay!").unwrap();
        let encoded_content = e.finish().unwrap();
        *response.body_mut() = encoded_content.into();
    };
    let (server, url) = make_server(handler);

    let mut request = RequestBuilder::new(url.clone())
        .method(Method::GET)
        .body(None)
        .destination(Destination::Document)
        .origin(mock_origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .build();

    let response = fetch(&mut request, None);

    let _ = server.close();

    let internal_response = response.internal_response.unwrap();
    assert!(internal_response.status.clone().unwrap().0.is_success());
    assert_eq!(
        *internal_response.body.lock().unwrap(),
        ResponseBody::Done(b"Yay!".to_vec())
    );
}

#[test]
fn test_load_should_decode_the_response_as_gzip_when_response_headers_have_content_encoding_gzip() {
    let handler = move |_: HyperRequest<Body>, response: &mut HyperResponse<Body>| {
        response
            .headers_mut()
            .insert(header::CONTENT_ENCODING, HeaderValue::from_static("gzip"));
        let mut e = GzEncoder::new(Vec::new(), Compression::default());
        e.write(b"Yay!").unwrap();
        let encoded_content = e.finish().unwrap();
        *response.body_mut() = encoded_content.into();
    };
    let (server, url) = make_server(handler);

    let mut request = RequestBuilder::new(url.clone())
        .method(Method::GET)
        .body(None)
        .destination(Destination::Document)
        .origin(mock_origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .build();

    let response = fetch(&mut request, None);

    let _ = server.close();

    let internal_response = response.internal_response.unwrap();
    assert!(internal_response.status.clone().unwrap().0.is_success());
    assert_eq!(
        *internal_response.body.lock().unwrap(),
        ResponseBody::Done(b"Yay!".to_vec())
    );
}

#[test]
fn test_load_doesnt_send_request_body_on_any_redirect() {
    let post_handler = move |request: HyperRequest<Body>, response: &mut HyperResponse<Body>| {
        assert_eq!(request.method(), Method::GET);
        read_response(request)
            .and_then(|data| {
                assert_eq!(data, "");
                futures::future::ok(())
            })
            .poll()
            .unwrap();
        *response.body_mut() = b"Yay!".to_vec().into();
    };
    let (post_server, post_url) = make_server(post_handler);

    let post_redirect_url = post_url.clone();
    let pre_handler = move |request: HyperRequest<Body>, response: &mut HyperResponse<Body>| {
        read_response(request)
            .and_then(|data| {
                assert_eq!(data, "Body on POST");
                futures::future::ok(())
            })
            .poll()
            .unwrap();
        response.headers_mut().insert(
            header::LOCATION,
            HeaderValue::from_str(&post_redirect_url.to_string()).unwrap(),
        );
        *response.status_mut() = StatusCode::MOVED_PERMANENTLY;
    };
    let (pre_server, pre_url) = make_server(pre_handler);

    let mut request = RequestBuilder::new(pre_url.clone())
        .body(Some(b"Body on POST!".to_vec()))
        .method(Method::POST)
        .destination(Destination::Document)
        .origin(mock_origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .build();

    let response = fetch(&mut request, None);

    let _ = pre_server.close();
    let _ = post_server.close();

    assert!(response.to_actual().status.unwrap().0.is_success());
}

#[test]
fn test_load_doesnt_add_host_to_sts_list_when_url_is_http_even_if_sts_headers_are_present() {
    let handler = move |_: HyperRequest<Body>, response: &mut HyperResponse<Body>| {
        response
            .headers_mut()
            .typed_insert(StrictTransportSecurity::excluding_subdomains(
                Duration::from_secs(31536000),
            ));
        *response.body_mut() = b"Yay!".to_vec().into();
    };
    let (server, url) = make_server(handler);

    let mut request = RequestBuilder::new(url.clone())
        .method(Method::GET)
        .body(None)
        .destination(Destination::Document)
        .origin(mock_origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .build();

    let mut context = new_fetch_context(None, None);
    let response = fetch_with_context(&mut request, &mut context);

    let _ = server.close();

    assert!(response
        .internal_response
        .unwrap()
        .status
        .unwrap()
        .0
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
    let handler = move |_: HyperRequest<Body>, response: &mut HyperResponse<Body>| {
        response.headers_mut().insert(
            header::SET_COOKIE,
            HeaderValue::from_static("mozillaIs=theBest"),
        );
        *response.body_mut() = b"Yay!".to_vec().into();
    };
    let (server, url) = make_server(handler);

    let mut context = new_fetch_context(None, None);

    assert_cookie_for_domain(&context.state.cookie_jar, url.as_str(), None);

    let mut request = RequestBuilder::new(url.clone())
        .method(Method::GET)
        .body(None)
        .destination(Destination::Document)
        .origin(mock_origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .credentials_mode(CredentialsMode::Include)
        .build();

    let response = fetch_with_context(&mut request, &mut context);

    let _ = server.close();

    assert!(response
        .internal_response
        .unwrap()
        .status
        .unwrap()
        .0
        .is_success());

    assert_cookie_for_domain(
        &context.state.cookie_jar,
        url.as_str(),
        Some("mozillaIs=theBest"),
    );
}

#[test]
fn test_load_sets_requests_cookies_header_for_url_by_getting_cookies_from_the_resource_manager() {
    let handler = move |request: HyperRequest<Body>, response: &mut HyperResponse<Body>| {
        assert_eq!(
            request.headers().get(header::COOKIE).unwrap().as_bytes(),
            b"mozillaIs=theBest"
        );
        *response.body_mut() = b"Yay!".to_vec().into();
    };
    let (server, url) = make_server(handler);

    let mut context = new_fetch_context(None, None);

    {
        let mut cookie_jar = context.state.cookie_jar.write().unwrap();
        let cookie = Cookie::new_wrapped(
            CookiePair::new("mozillaIs".to_owned(), "theBest".to_owned()),
            &url,
            CookieSource::HTTP,
        )
        .unwrap();
        cookie_jar.push(cookie, &url, CookieSource::HTTP);
    }

    let mut request = RequestBuilder::new(url.clone())
        .method(Method::GET)
        .body(None)
        .destination(Destination::Document)
        .origin(mock_origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .credentials_mode(CredentialsMode::Include)
        .build();

    let response = fetch_with_context(&mut request, &mut context);

    let _ = server.close();

    assert!(response
        .internal_response
        .unwrap()
        .status
        .unwrap()
        .0
        .is_success());
}

#[test]
fn test_load_sends_cookie_if_nonhttp() {
    let handler = move |request: HyperRequest<Body>, response: &mut HyperResponse<Body>| {
        assert_eq!(
            request.headers().get(header::COOKIE).unwrap().as_bytes(),
            b"mozillaIs=theBest"
        );
        *response.body_mut() = b"Yay!".to_vec().into();
    };
    let (server, url) = make_server(handler);

    let mut context = new_fetch_context(None, None);

    {
        let mut cookie_jar = context.state.cookie_jar.write().unwrap();
        let cookie = Cookie::new_wrapped(
            CookiePair::new("mozillaIs".to_owned(), "theBest".to_owned()),
            &url,
            CookieSource::NonHTTP,
        )
        .unwrap();
        cookie_jar.push(cookie, &url, CookieSource::HTTP);
    }

    let mut request = RequestBuilder::new(url.clone())
        .method(Method::GET)
        .body(None)
        .destination(Destination::Document)
        .origin(mock_origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .credentials_mode(CredentialsMode::Include)
        .build();

    let response = fetch_with_context(&mut request, &mut context);

    let _ = server.close();

    assert!(response
        .internal_response
        .unwrap()
        .status
        .unwrap()
        .0
        .is_success());
}

#[test]
fn test_cookie_set_with_httponly_should_not_be_available_using_getcookiesforurl() {
    let handler = move |_: HyperRequest<Body>, response: &mut HyperResponse<Body>| {
        response.headers_mut().insert(
            header::SET_COOKIE,
            HeaderValue::from_static("mozillaIs=theBest; HttpOnly"),
        );
        *response.body_mut() = b"Yay!".to_vec().into();
    };
    let (server, url) = make_server(handler);

    let mut context = new_fetch_context(None, None);

    assert_cookie_for_domain(&context.state.cookie_jar, url.as_str(), None);

    let mut request = RequestBuilder::new(url.clone())
        .method(Method::GET)
        .body(None)
        .destination(Destination::Document)
        .origin(mock_origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .credentials_mode(CredentialsMode::Include)
        .build();

    let response = fetch_with_context(&mut request, &mut context);

    let _ = server.close();

    assert!(response
        .internal_response
        .unwrap()
        .status
        .unwrap()
        .0
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
fn test_when_cookie_received_marked_secure_is_ignored_for_http() {
    let handler = move |_: HyperRequest<Body>, response: &mut HyperResponse<Body>| {
        response.headers_mut().insert(
            header::SET_COOKIE,
            HeaderValue::from_static("mozillaIs=theBest; Secure"),
        );
        *response.body_mut() = b"Yay!".to_vec().into();
    };
    let (server, url) = make_server(handler);

    let mut context = new_fetch_context(None, None);

    assert_cookie_for_domain(&context.state.cookie_jar, url.as_str(), None);

    let mut request = RequestBuilder::new(url.clone())
        .method(Method::GET)
        .body(None)
        .destination(Destination::Document)
        .origin(mock_origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .credentials_mode(CredentialsMode::Include)
        .build();

    let response = fetch_with_context(&mut request, &mut context);

    let _ = server.close();

    assert!(response
        .internal_response
        .unwrap()
        .status
        .unwrap()
        .0
        .is_success());

    assert_cookie_for_domain(&context.state.cookie_jar, url.as_str(), None);
}

#[test]
fn test_load_sets_content_length_to_length_of_request_body() {
    let content = b"This is a request body";
    let handler = move |request: HyperRequest<Body>, response: &mut HyperResponse<Body>| {
        let content_length = ContentLength(content.len() as u64);
        assert_eq!(
            request.headers().typed_get::<ContentLength>(),
            Some(content_length)
        );
        *response.body_mut() = content.to_vec().into();
    };
    let (server, url) = make_server(handler);

    let mut request = RequestBuilder::new(url.clone())
        .method(Method::POST)
        .body(Some(content.to_vec()))
        .destination(Destination::Document)
        .origin(mock_origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .build();

    let response = fetch(&mut request, None);

    let _ = server.close();

    assert!(response
        .internal_response
        .unwrap()
        .status
        .unwrap()
        .0
        .is_success());
}

#[test]
fn test_load_uses_explicit_accept_from_headers_in_load_data() {
    let handler = move |request: HyperRequest<Body>, response: &mut HyperResponse<Body>| {
        assert_eq!(
            request
                .headers()
                .get(header::ACCEPT)
                .unwrap()
                .to_str()
                .unwrap(),
            "text/html"
        );
        *response.body_mut() = b"Yay!".to_vec().into();
    };
    let (server, url) = make_server(handler);

    let mut accept_headers = HeaderMap::new();
    accept_headers.insert(header::ACCEPT, HeaderValue::from_static("text/html"));
    let mut request = RequestBuilder::new(url.clone())
        .method(Method::GET)
        .headers(accept_headers)
        .destination(Destination::Document)
        .origin(mock_origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .build();

    let response = fetch(&mut request, None);

    let _ = server.close();

    assert!(response
        .internal_response
        .unwrap()
        .status
        .unwrap()
        .0
        .is_success());
}

#[test]
fn test_load_sets_default_accept_to_html_xhtml_xml_and_then_anything_else() {
    let handler = move |request: HyperRequest<Body>, response: &mut HyperResponse<Body>| {
        assert_eq!(
            request
                .headers()
                .get(header::ACCEPT)
                .unwrap()
                .to_str()
                .unwrap(),
            "text/html, application/xhtml+xml, application/xml; q=0.9, */*; q=0.8"
        );
        *response.body_mut() = b"Yay!".to_vec().into();
    };
    let (server, url) = make_server(handler);

    let mut request = RequestBuilder::new(url.clone())
        .method(Method::GET)
        .destination(Destination::Document)
        .origin(mock_origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .build();

    let response = fetch(&mut request, None);

    let _ = server.close();

    assert!(response
        .internal_response
        .unwrap()
        .status
        .unwrap()
        .0
        .is_success());
}

#[test]
fn test_load_uses_explicit_accept_encoding_from_load_data_headers() {
    let handler = move |request: HyperRequest<Body>, response: &mut HyperResponse<Body>| {
        assert_eq!(
            request
                .headers()
                .get(header::ACCEPT_ENCODING)
                .unwrap()
                .to_str()
                .unwrap(),
            "chunked"
        );
        *response.body_mut() = b"Yay!".to_vec().into();
    };
    let (server, url) = make_server(handler);

    let mut accept_encoding_headers = HeaderMap::new();
    accept_encoding_headers.insert(header::ACCEPT_ENCODING, HeaderValue::from_static("chunked"));
    let mut request = RequestBuilder::new(url.clone())
        .method(Method::GET)
        .headers(accept_encoding_headers)
        .destination(Destination::Document)
        .origin(mock_origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .build();

    let response = fetch(&mut request, None);

    let _ = server.close();

    assert!(response
        .internal_response
        .unwrap()
        .status
        .unwrap()
        .0
        .is_success());
}

#[test]
fn test_load_sets_default_accept_encoding_to_gzip_and_deflate() {
    let handler = move |request: HyperRequest<Body>, response: &mut HyperResponse<Body>| {
        assert_eq!(
            request
                .headers()
                .get(header::ACCEPT_ENCODING)
                .unwrap()
                .to_str()
                .unwrap(),
            "gzip, deflate, br"
        );
        *response.body_mut() = b"Yay!".to_vec().into();
    };
    let (server, url) = make_server(handler);

    let mut request = RequestBuilder::new(url.clone())
        .method(Method::GET)
        .destination(Destination::Document)
        .origin(mock_origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .build();

    let response = fetch(&mut request, None);

    let _ = server.close();

    assert!(response
        .internal_response
        .unwrap()
        .status
        .unwrap()
        .0
        .is_success());
}

#[test]
fn test_load_errors_when_there_a_redirect_loop() {
    let url_b_for_a = Arc::new(Mutex::new(None::<ServoUrl>));
    let url_b_for_a_clone = url_b_for_a.clone();
    let handler_a = move |_: HyperRequest<Body>, response: &mut HyperResponse<Body>| {
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
    let handler_b = move |_: HyperRequest<Body>, response: &mut HyperResponse<Body>| {
        response.headers_mut().insert(
            header::LOCATION,
            HeaderValue::from_str(&url_a_for_b.to_string()).unwrap(),
        );
        *response.status_mut() = StatusCode::MOVED_PERMANENTLY;
    };
    let (server_b, url_b) = make_server(handler_b);

    *url_b_for_a.lock().unwrap() = Some(url_b.clone());

    let mut request = RequestBuilder::new(url_a.clone())
        .method(Method::GET)
        .destination(Destination::Document)
        .origin(mock_origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .build();

    let response = fetch(&mut request, None);

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
    let handler_a = move |_: HyperRequest<Body>, response: &mut HyperResponse<Body>| {
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
            *response.body_mut() = b"Success".to_vec().into()
        }
    };
    let (server_a, url_a) = make_server(handler_a);

    let url_a_for_b = url_a.clone();
    let handler_b = move |_: HyperRequest<Body>, response: &mut HyperResponse<Body>| {
        response.headers_mut().insert(
            header::LOCATION,
            HeaderValue::from_str(&url_a_for_b.to_string()).unwrap(),
        );
        *response.status_mut() = StatusCode::MOVED_PERMANENTLY;
    };
    let (server_b, url_b) = make_server(handler_b);

    *url_b_for_a.lock().unwrap() = Some(url_b.clone());

    let mut request = RequestBuilder::new(url_a.clone())
        .method(Method::GET)
        .destination(Destination::Document)
        .origin(mock_origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .build();

    let response = fetch(&mut request, None);

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
    let post_handler = move |request: HyperRequest<Body>, response: &mut HyperResponse<Body>| {
        assert_eq!(request.method(), Method::GET);
        *response.body_mut() = b"Yay!".to_vec().into();
    };
    let (post_server, post_url) = make_server(post_handler);

    let post_redirect_url = post_url.clone();
    let pre_handler = move |request: HyperRequest<Body>, response: &mut HyperResponse<Body>| {
        assert_eq!(request.method(), Method::GET);
        response.headers_mut().insert(
            header::LOCATION,
            HeaderValue::from_str(&post_redirect_url.to_string()).unwrap(),
        );
        *response.status_mut() = StatusCode::MOVED_PERMANENTLY;
    };
    let (pre_server, pre_url) = make_server(pre_handler);

    let mut request = RequestBuilder::new(pre_url.clone())
        .method(Method::GET)
        .destination(Destination::Document)
        .origin(mock_origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .build();

    let response = fetch(&mut request, None);

    let _ = pre_server.close();
    let _ = post_server.close();

    let internal_response = response.internal_response.unwrap();
    assert!(internal_response.status.clone().unwrap().0.is_success());
    assert_eq!(
        *internal_response.body.lock().unwrap(),
        ResponseBody::Done(b"Yay!".to_vec())
    );
}

#[test]
fn test_redirect_from_x_to_y_provides_y_cookies_from_y() {
    let shared_url_y = Arc::new(Mutex::new(None::<ServoUrl>));
    let shared_url_y_clone = shared_url_y.clone();
    let handler = move |request: HyperRequest<Body>, response: &mut HyperResponse<Body>| {
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
            *response.body_mut() = b"Yay!".to_vec().into();
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

    let mut context = new_fetch_context(None, None);
    {
        let mut cookie_jar = context.state.cookie_jar.write().unwrap();
        let cookie_x = Cookie::new_wrapped(
            CookiePair::new("mozillaIsNot".to_owned(), "dotOrg".to_owned()),
            &url_x,
            CookieSource::HTTP,
        )
        .unwrap();

        cookie_jar.push(cookie_x, &url_x, CookieSource::HTTP);

        let cookie_y = Cookie::new_wrapped(
            CookiePair::new("mozillaIs".to_owned(), "theBest".to_owned()),
            &url_y,
            CookieSource::HTTP,
        )
        .unwrap();
        cookie_jar.push(cookie_y, &url_y, CookieSource::HTTP);
    }

    let mut request = RequestBuilder::new(url_x.clone())
        .method(Method::GET)
        .destination(Destination::Document)
        .origin(mock_origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .credentials_mode(CredentialsMode::Include)
        .build();

    let response = fetch_with_context(&mut request, &mut context);

    let _ = server.close();

    let internal_response = response.internal_response.unwrap();
    assert!(internal_response.status.clone().unwrap().0.is_success());
    assert_eq!(
        *internal_response.body.lock().unwrap(),
        ResponseBody::Done(b"Yay!".to_vec())
    );
}

#[test]
fn test_redirect_from_x_to_x_provides_x_with_cookie_from_first_response() {
    let handler = move |request: HyperRequest<Body>, response: &mut HyperResponse<Body>| {
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
            *response.body_mut() = b"Yay!".to_vec().into();
        } else {
            panic!("unexpected path {:?}", path)
        }
    };
    let (server, url) = make_server(handler);

    let url = url.join("/initial/").unwrap();

    let mut request = RequestBuilder::new(url.clone())
        .method(Method::GET)
        .destination(Destination::Document)
        .origin(mock_origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .credentials_mode(CredentialsMode::Include)
        .build();

    let response = fetch(&mut request, None);

    let _ = server.close();

    let internal_response = response.internal_response.unwrap();
    assert!(internal_response.status.clone().unwrap().0.is_success());
    assert_eq!(
        *internal_response.body.lock().unwrap(),
        ResponseBody::Done(b"Yay!".to_vec())
    );
}

#[test]
fn test_if_auth_creds_not_in_url_but_in_cache_it_sets_it() {
    let handler = move |request: HyperRequest<Body>, _response: &mut HyperResponse<Body>| {
        let expected = Authorization::basic("username", "test");
        assert_eq!(
            request.headers().typed_get::<Authorization<Basic>>(),
            Some(expected)
        );
    };
    let (server, url) = make_server(handler);

    let mut request = RequestBuilder::new(url.clone())
        .method(Method::GET)
        .body(None)
        .destination(Destination::Document)
        .origin(mock_origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .credentials_mode(CredentialsMode::Include)
        .build();

    let mut context = new_fetch_context(None, None);

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

    let response = fetch_with_context(&mut request, &mut context);

    let _ = server.close();

    assert!(response
        .internal_response
        .unwrap()
        .status
        .unwrap()
        .0
        .is_success());
}

#[test]
fn test_auth_ui_needs_www_auth() {
    let handler = move |_: HyperRequest<Body>, response: &mut HyperResponse<Body>| {
        *response.status_mut() = StatusCode::UNAUTHORIZED;
    };
    let (server, url) = make_server(handler);

    let mut request = RequestBuilder::new(url.clone())
        .method(Method::GET)
        .body(None)
        .destination(Destination::Document)
        .origin(mock_origin())
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .credentials_mode(CredentialsMode::Include)
        .build();

    let response = fetch(&mut request, None);

    let _ = server.close();

    assert_eq!(
        response.internal_response.unwrap().status.unwrap().0,
        StatusCode::UNAUTHORIZED
    );
}

#[test]
fn test_origin_set() {
    let origin_header = Arc::new(Mutex::new(None));
    let origin_header_clone = origin_header.clone();
    let handler = move |request: HyperRequest<Body>, resp: &mut HyperResponse<Body>| {
        let origin_header_clone = origin_header.clone();
        resp.headers_mut()
            .typed_insert(AccessControlAllowOrigin::ANY);
        match request.headers().typed_get::<Origin>() {
            None => assert_eq!(origin_header_clone.lock().unwrap().take(), None),
            Some(h) => assert_eq!(h, origin_header_clone.lock().unwrap().take().unwrap()),
        }
    };
    let (server, url) = make_server(handler);

    let mut origin =
        Origin::try_from_parts(url.scheme(), url.host_str().unwrap(), url.port()).unwrap();
    *origin_header_clone.lock().unwrap() = Some(origin.clone());
    let mut request = RequestBuilder::new(url.clone())
        .method(Method::POST)
        .body(None)
        .origin(url.clone().origin())
        .build();

    let response = fetch(&mut request, None);
    assert!(response
        .internal_response
        .unwrap()
        .status
        .unwrap()
        .0
        .is_success());

    let origin_url = ServoUrl::parse("http://example.com").unwrap();
    origin =
        Origin::try_from_parts(origin_url.scheme(), origin_url.host_str().unwrap(), None).unwrap();
    // Test Origin header is set on Get request with CORS mode
    let mut request = RequestBuilder::new(url.clone())
        .method(Method::GET)
        .mode(RequestMode::CorsMode)
        .body(None)
        .origin(origin_url.clone().origin())
        .build();

    *origin_header_clone.lock().unwrap() = Some(origin.clone());
    let response = fetch(&mut request, None);
    assert!(response
        .internal_response
        .unwrap()
        .status
        .unwrap()
        .0
        .is_success());

    // Test Origin header is not set on method Head
    let mut request = RequestBuilder::new(url.clone())
        .method(Method::HEAD)
        .body(None)
        .origin(url.clone().origin())
        .build();

    *origin_header_clone.lock().unwrap() = None;
    let response = fetch(&mut request, None);
    assert!(response
        .internal_response
        .unwrap()
        .status
        .unwrap()
        .0
        .is_success());

    let _ = server.close();
}
