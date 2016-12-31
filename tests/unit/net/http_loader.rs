/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use content_blocker::parse_list;
use cookie_rs::Cookie as CookiePair;
use devtools_traits::{ChromeToDevtoolsControlMsg, DevtoolsControlMsg, NetworkEvent};
use devtools_traits::HttpRequest as DevtoolsHttpRequest;
use devtools_traits::HttpResponse as DevtoolsHttpResponse;
use fetch;
use fetch_with_context;
use flate2::Compression;
use flate2::write::{DeflateEncoder, GzEncoder};
use hyper::LanguageTag;
use hyper::header::{Accept, AcceptEncoding, ContentEncoding, ContentLength, Cookie as CookieHeader};
use hyper::header::{AcceptLanguage, Authorization, Basic, Date};
use hyper::header::{Encoding, Headers, Host, Location, Quality, QualityItem, SetCookie, qitem};
use hyper::header::{StrictTransportSecurity, UserAgent};
use hyper::method::Method;
use hyper::mime::{Mime, SubLevel, TopLevel};
use hyper::server::{Request as HyperRequest, Response as HyperResponse};
use hyper::status::StatusCode;
use hyper::uri::RequestUri;
use make_server;
use msg::constellation_msg::TEST_PIPELINE_ID;
use net::cookie::Cookie;
use net::cookie_storage::CookieStorage;
use net::resource_thread::AuthCacheEntry;
use net_traits::{CookieSource, NetworkError};
use net_traits::hosts::replace_host_table;
use net_traits::request::{Request, RequestInit, CredentialsMode, Destination};
use net_traits::response::ResponseBody;
use new_fetch_context;
use servo_url::ServoUrl;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex, RwLock, mpsc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;

fn read_response(reader: &mut Read) -> String {
    let mut buf = vec![0; 1024];
    match reader.read(&mut buf) {
        Ok(len) if len > 0 => {
            unsafe { buf.set_len(len); }
            String::from_utf8(buf).unwrap()
        },
        Ok(_) => "".to_owned(),
        Err(e) => panic!("problem reading response {}", e)
    }
}

fn assert_cookie_for_domain(cookie_jar: Arc<RwLock<CookieStorage>>, domain: &str, cookie: Option<&str>) {
    let mut cookie_jar = cookie_jar.write().unwrap();
    let url = ServoUrl::parse(&*domain).unwrap();
    let cookies = cookie_jar.cookies_for_url(&url, CookieSource::HTTP);
    assert_eq!(cookies.as_ref().map(|c| &**c), cookie);
}

pub fn expect_devtools_http_request(devtools_port: &Receiver<DevtoolsControlMsg>) -> DevtoolsHttpRequest {
    match devtools_port.recv().unwrap() {
        DevtoolsControlMsg::FromChrome(
        ChromeToDevtoolsControlMsg::NetworkEvent(_, net_event)) => {
            match net_event {
                NetworkEvent::HttpRequest(httprequest) => {
                    httprequest
                },

                _ => panic!("No HttpRequest Received"),
            }
        },
        _ => panic!("No HttpRequest Received"),
    }
}

pub fn expect_devtools_http_response(devtools_port: &Receiver<DevtoolsControlMsg>) -> DevtoolsHttpResponse {
    match devtools_port.recv().unwrap() {
        DevtoolsControlMsg::FromChrome(
            ChromeToDevtoolsControlMsg::NetworkEvent(_, net_event_response)) => {
            match net_event_response {
                NetworkEvent::HttpResponse(httpresponse) => {
                    httpresponse
                },

                _ => panic!("No HttpResponse Received"),
            }
        },
        _ => panic!("No HttpResponse Received"),
    }
}

#[test]
fn test_check_default_headers_loaded_in_every_request() {
    let expected_headers = Arc::new(Mutex::new(None));
    let expected_headers_clone = expected_headers.clone();
    let handler = move |request: HyperRequest, _: HyperResponse| {
        assert_eq!(request.headers, expected_headers_clone.lock().unwrap().take().unwrap());
    };
    let (mut server, url) = make_server(handler);

    let mut headers = Headers::new();

    headers.set(AcceptEncoding(vec![qitem(Encoding::Gzip),
                                    qitem(Encoding::Deflate),
                                    qitem(Encoding::EncodingExt("br".to_owned()))]));

    let hostname = match url.host_str() {
        Some(hostname) => hostname.to_owned(),
        _ => panic!()
    };

    headers.set(Host { hostname: hostname, port: url.port() });

    let accept = Accept(vec![
                            qitem(Mime(TopLevel::Text, SubLevel::Html, vec![])),
                            qitem(Mime(TopLevel::Application, SubLevel::Ext("xhtml+xml".to_owned()), vec![])),
                            QualityItem::new(Mime(TopLevel::Application, SubLevel::Xml, vec![]), Quality(900u16)),
                            QualityItem::new(Mime(TopLevel::Star, SubLevel::Star, vec![]), Quality(800u16)),
                            ]);
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

    headers.set(UserAgent(::DEFAULT_USER_AGENT.to_owned()));

    *expected_headers.lock().unwrap() = Some(headers.clone());

    // Testing for method.GET
    let request = Request::from_init(RequestInit {
        url: url.clone(),
        method: Method::Get,
        destination: Destination::Document,
        origin: url.clone(),
        pipeline_id: Some(TEST_PIPELINE_ID),
        .. RequestInit::default()
    });
    let response = fetch(request, None);
    assert!(response.status.unwrap().is_success());

    // Testing for method.POST
    headers.set(ContentLength(0 as u64));
    *expected_headers.lock().unwrap() = Some(headers.clone());
    let request = Request::from_init(RequestInit {
        url: url.clone(),
        method: Method::Post,
        destination: Destination::Document,
        origin: url.clone(),
        pipeline_id: Some(TEST_PIPELINE_ID),
        .. RequestInit::default()
    });
    let response = fetch(request, None);
    assert!(response.status.unwrap().is_success());

    let _ = server.close();
}

#[test]
fn test_load_when_request_is_not_get_or_head_and_there_is_no_body_content_length_should_be_set_to_0() {
    let handler = move |request: HyperRequest, _: HyperResponse| {
        assert_eq!(request.headers.get::<ContentLength>(), Some(&ContentLength(0)));
    };
    let (mut server, url) = make_server(handler);

    let request = Request::from_init(RequestInit {
        url: url.clone(),
        method: Method::Post,
        body: None,
        destination: Destination::Document,
        origin: url.clone(),
        pipeline_id: Some(TEST_PIPELINE_ID),
        .. RequestInit::default()
    });
    let response = fetch(request, None);
    assert!(response.status.unwrap().is_success());

    let _ = server.close();
}

#[test]
fn test_request_and_response_data_with_network_messages() {
    let handler = move |_: HyperRequest, mut response: HyperResponse| {
        response.headers_mut().set(Host { hostname: "foo.bar".to_owned(), port: None });
        response.send(b"Yay!").unwrap();
    };
    let (mut server, url) = make_server(handler);

    let mut request_headers = Headers::new();
    request_headers.set(Host { hostname: "bar.foo".to_owned(), port: None });
    let request = Request::from_init(RequestInit {
        url: url.clone(),
        method: Method::Get,
        headers: request_headers,
        body: None,
        destination: Destination::Document,
        origin: url.clone(),
        pipeline_id: Some(TEST_PIPELINE_ID),
        .. RequestInit::default()
    });
    let (devtools_chan, devtools_port) = mpsc::channel();
    let response = fetch(request, Some(devtools_chan));
    assert!(response.status.unwrap().is_success());

    let _ = server.close();

    // notification received from devtools
    let devhttprequest = expect_devtools_http_request(&devtools_port);
    let devhttpresponse = expect_devtools_http_response(&devtools_port);

    //Creating default headers for request
    let mut headers = Headers::new();

    headers.set(AcceptEncoding(vec![
                                   qitem(Encoding::Gzip),
                                   qitem(Encoding::Deflate),
                                   qitem(Encoding::EncodingExt("br".to_owned()))
                                   ]));

    headers.set(Host { hostname: url.host_str().unwrap().to_owned() , port: url.port() });

    let accept = Accept(vec![
                            qitem(Mime(TopLevel::Text, SubLevel::Html, vec![])),
                            qitem(Mime(TopLevel::Application, SubLevel::Ext("xhtml+xml".to_owned()), vec![])),
                            QualityItem::new(Mime(TopLevel::Application, SubLevel::Xml, vec![]), Quality(900u16)),
                            QualityItem::new(Mime(TopLevel::Star, SubLevel::Star, vec![]), Quality(800u16)),
                            ]);
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

    headers.set(UserAgent(::DEFAULT_USER_AGENT.to_owned()));

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
        is_xhr: false,
    };

    let content = "Yay!";
    let mut response_headers = Headers::new();
    response_headers.set(ContentLength(content.len() as u64));
    response_headers.set(Host { hostname: "foo.bar".to_owned(), port: None });
    response_headers.set(devhttpresponse.headers.as_ref().unwrap().get::<Date>().unwrap().clone());

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
    let handler = move |_: HyperRequest, mut response: HyperResponse| {
        response.headers_mut().set(Host { hostname: "foo.bar".to_owned(), port: None });
        response.send(b"Yay!").unwrap();
    };
    let (mut server, url) = make_server(handler);

    let request = Request::from_init(RequestInit {
        url: url.clone(),
        method: Method::Get,
        destination: Destination::Document,
        origin: url.clone(),
        pipeline_id: None,
        .. RequestInit::default()
    });
    let (devtools_chan, devtools_port) = mpsc::channel();
    let response = fetch(request, Some(devtools_chan));
    assert!(response.status.unwrap().is_success());

    let _ = server.close();

    // notification received from devtools
    assert!(devtools_port.try_recv().is_err());
}

#[test]
fn test_redirected_request_to_devtools() {
    let post_handler = move |request: HyperRequest, response: HyperResponse| {
        assert_eq!(request.method, Method::Get);
        response.send(b"Yay!").unwrap();
    };
    let (mut post_server, post_url) = make_server(post_handler);

    let post_redirect_url = post_url.clone();
    let pre_handler = move |request: HyperRequest, mut response: HyperResponse| {
        assert_eq!(request.method, Method::Post);
        response.headers_mut().set(Location(post_redirect_url.to_string()));
        *response.status_mut() = StatusCode::MovedPermanently;
        response.send(b"").unwrap();
    };
    let (mut pre_server, pre_url) = make_server(pre_handler);

    let request = Request::from_init(RequestInit {
        url: pre_url.clone(),
        method: Method::Post,
        destination: Destination::Document,
        origin: pre_url.clone(),
        pipeline_id: Some(TEST_PIPELINE_ID),
        .. RequestInit::default()
    });
    let (devtools_chan, devtools_port) = mpsc::channel();
    fetch(request, Some(devtools_chan));

    let _ = pre_server.close();
    let _ = post_server.close();

    let devhttprequest = expect_devtools_http_request(&devtools_port);
    let devhttpresponse = expect_devtools_http_response(&devtools_port);

    assert!(devhttprequest.method == Method::Post);
    assert!(devhttprequest.url == pre_url);
    assert!(devhttpresponse.status == Some((301, b"Moved Permanently".to_vec())));

    let devhttprequest = expect_devtools_http_request(&devtools_port);
    let devhttpresponse = expect_devtools_http_response(&devtools_port);

    assert!(devhttprequest.method == Method::Get);
    assert!(devhttprequest.url == post_url);
    assert!(devhttpresponse.status == Some((200, b"OK".to_vec())));
}



#[test]
fn test_load_when_redirecting_from_a_post_should_rewrite_next_request_as_get() {
    let post_handler = move |request: HyperRequest, response: HyperResponse| {
        assert_eq!(request.method, Method::Get);
        response.send(b"Yay!").unwrap();
    };
    let (mut post_server, post_url) = make_server(post_handler);

    let post_redirect_url = post_url.clone();
    let pre_handler = move |request: HyperRequest, mut response: HyperResponse| {
        assert_eq!(request.method, Method::Post);
        response.headers_mut().set(Location(post_redirect_url.to_string()));
        *response.status_mut() = StatusCode::MovedPermanently;
        response.send(b"").unwrap();
    };
    let (mut pre_server, pre_url) = make_server(pre_handler);

    let request = Request::from_init(RequestInit {
        url: pre_url.clone(),
        method: Method::Post,
        destination: Destination::Document,
        origin: pre_url.clone(),
        pipeline_id: Some(TEST_PIPELINE_ID),
        .. RequestInit::default()
    });
    let response = fetch(request, None);

    let _ = pre_server.close();
    let _ = post_server.close();

    assert!(response.to_actual().status.unwrap().is_success());
}

#[test]
fn test_load_should_decode_the_response_as_deflate_when_response_headers_have_content_encoding_deflate() {
    let handler = move |_: HyperRequest, mut response: HyperResponse| {
        response.headers_mut().set(ContentEncoding(vec![Encoding::Deflate]));
        let mut e = DeflateEncoder::new(Vec::new(), Compression::Default);
        e.write(b"Yay!").unwrap();
        let encoded_content = e.finish().unwrap();
        response.send(&encoded_content).unwrap();
    };
    let (mut server, url) = make_server(handler);

    let request = Request::from_init(RequestInit {
        url: url.clone(),
        method: Method::Get,
        body: None,
        destination: Destination::Document,
        origin: url.clone(),
        pipeline_id: Some(TEST_PIPELINE_ID),
        .. RequestInit::default()
    });
    let response = fetch(request, None);

    let _ = server.close();

    assert!(response.status.unwrap().is_success());
    assert_eq!(*response.body.lock().unwrap(),
               ResponseBody::Done(b"Yay!".to_vec()));
}

#[test]
fn test_load_should_decode_the_response_as_gzip_when_response_headers_have_content_encoding_gzip() {
    let handler = move |_: HyperRequest, mut response: HyperResponse| {
        response.headers_mut().set(ContentEncoding(vec![Encoding::Gzip]));
        let mut e = GzEncoder::new(Vec::new(), Compression::Default);
        e.write(b"Yay!").unwrap();
        let encoded_content = e.finish().unwrap();
        response.send(&encoded_content).unwrap();
    };
    let (mut server, url) = make_server(handler);

    let request = Request::from_init(RequestInit {
        url: url.clone(),
        method: Method::Get,
        body: None,
        destination: Destination::Document,
        origin: url.clone(),
        pipeline_id: Some(TEST_PIPELINE_ID),
        .. RequestInit::default()
    });
    let response = fetch(request, None);

    let _ = server.close();

    assert!(response.status.unwrap().is_success());
    assert_eq!(*response.body.lock().unwrap(),
               ResponseBody::Done(b"Yay!".to_vec()));
}

#[test]
fn test_load_doesnt_send_request_body_on_any_redirect() {
    let post_handler = move |mut request: HyperRequest, response: HyperResponse| {
        assert_eq!(request.method, Method::Get);
        let data = read_response(&mut request);
        assert_eq!(data, "");
        response.send(b"Yay!").unwrap();
    };
    let (mut post_server, post_url) = make_server(post_handler);

    let post_redirect_url = post_url.clone();
    let pre_handler = move |mut request: HyperRequest, mut response: HyperResponse| {
        let data = read_response(&mut request);
        assert_eq!(data, "Body on POST!");
        response.headers_mut().set(Location(post_redirect_url.to_string()));
        *response.status_mut() = StatusCode::MovedPermanently;
        response.send(b"").unwrap();
    };
    let (mut pre_server, pre_url) = make_server(pre_handler);

    let request = Request::from_init(RequestInit {
        url: pre_url.clone(),
        body: Some(b"Body on POST!".to_vec()),
        method: Method::Post,
        destination: Destination::Document,
        origin: pre_url.clone(),
        pipeline_id: Some(TEST_PIPELINE_ID),
        .. RequestInit::default()
    });
    let response = fetch(request, None);

    let _ = pre_server.close();
    let _ = post_server.close();

    assert!(response.to_actual().status.unwrap().is_success());
}

#[test]
fn test_load_doesnt_add_host_to_sts_list_when_url_is_http_even_if_sts_headers_are_present() {
    let handler = move |_: HyperRequest, mut response: HyperResponse| {
        response.headers_mut().set(StrictTransportSecurity::excluding_subdomains(31536000));
        response.send(b"Yay!").unwrap();
    };
    let (mut server, url) = make_server(handler);

    let request = Request::from_init(RequestInit {
        url: url.clone(),
        method: Method::Get,
        body: None,
        destination: Destination::Document,
        origin: url.clone(),
        pipeline_id: Some(TEST_PIPELINE_ID),
        .. RequestInit::default()
    });
    let context = new_fetch_context(None);
    let response = fetch_with_context(request, &context);

    let _ = server.close();

    assert!(response.status.unwrap().is_success());
    assert_eq!(context.state.hsts_list.read().unwrap().is_host_secure(url.host_str().unwrap()), false);
}

#[test]
fn test_load_sets_cookies_in_the_resource_manager_when_it_get_set_cookie_header_in_response() {
    let handler = move |_: HyperRequest, mut response: HyperResponse| {
        response.headers_mut().set(SetCookie(vec![CookiePair::new("mozillaIs".to_owned(), "theBest".to_owned())]));
        response.send(b"Yay!").unwrap();
    };
    let (mut server, url) = make_server(handler);

    let context = new_fetch_context(None);

    assert_cookie_for_domain(context.state.cookie_jar.clone(), url.as_str(), None);

    let request = Request::from_init(RequestInit {
        url: url.clone(),
        method: Method::Get,
        body: None,
        destination: Destination::Document,
        origin: url.clone(),
        pipeline_id: Some(TEST_PIPELINE_ID),
        credentials_mode: CredentialsMode::Include,
        .. RequestInit::default()
    });
    let response = fetch_with_context(request, &context);

    let _ = server.close();

    assert!(response.status.unwrap().is_success());

    assert_cookie_for_domain(context.state.cookie_jar.clone(), url.as_str(), Some("mozillaIs=theBest"));
}

#[test]
fn test_load_sets_requests_cookies_header_for_url_by_getting_cookies_from_the_resource_manager() {
    let handler = move |request: HyperRequest, response: HyperResponse| {
        assert_eq!(request.headers.get::<CookieHeader>(),
                   Some(&CookieHeader(vec![CookiePair::new("mozillaIs".to_owned(), "theBest".to_owned())])));
        response.send(b"Yay!").unwrap();
    };
    let (mut server, url) = make_server(handler);

    let context = new_fetch_context(None);

    {
        let mut cookie_jar = context.state.cookie_jar.write().unwrap();
        let cookie = Cookie::new_wrapped(
            CookiePair::new("mozillaIs".to_owned(), "theBest".to_owned()),
            &url,
            CookieSource::HTTP
        ).unwrap();
        cookie_jar.push(cookie, &url, CookieSource::HTTP);
    }

    let request = Request::from_init(RequestInit {
        url: url.clone(),
        method: Method::Get,
        body: None,
        destination: Destination::Document,
        origin: url.clone(),
        pipeline_id: Some(TEST_PIPELINE_ID),
        credentials_mode: CredentialsMode::Include,
        .. RequestInit::default()
    });
    let response = fetch_with_context(request, &context);

    let _ = server.close();

    assert!(response.status.unwrap().is_success());
}

#[test]
fn test_load_sends_cookie_if_nonhttp() {
    let handler = move |request: HyperRequest, response: HyperResponse| {
        assert_eq!(request.headers.get::<CookieHeader>(),
                   Some(&CookieHeader(vec![CookiePair::new("mozillaIs".to_owned(), "theBest".to_owned())])));
        response.send(b"Yay!").unwrap();
    };
    let (mut server, url) = make_server(handler);

    let context = new_fetch_context(None);

    {
        let mut cookie_jar = context.state.cookie_jar.write().unwrap();
        let cookie = Cookie::new_wrapped(
            CookiePair::new("mozillaIs".to_owned(), "theBest".to_owned()),
            &url,
            CookieSource::NonHTTP
        ).unwrap();
        cookie_jar.push(cookie, &url, CookieSource::HTTP);
    }

    let request = Request::from_init(RequestInit {
        url: url.clone(),
        method: Method::Get,
        body: None,
        destination: Destination::Document,
        origin: url.clone(),
        pipeline_id: Some(TEST_PIPELINE_ID),
        credentials_mode: CredentialsMode::Include,
        .. RequestInit::default()
    });
    let response = fetch_with_context(request, &context);

    let _ = server.close();

    assert!(response.status.unwrap().is_success());
}

#[test]
fn test_cookie_set_with_httponly_should_not_be_available_using_getcookiesforurl() {
    let handler = move |_: HyperRequest, mut response: HyperResponse| {
        let mut pair = CookiePair::new("mozillaIs".to_owned(), "theBest".to_owned());
        pair.httponly = true;
        response.headers_mut().set(SetCookie(vec![pair]));
        response.send(b"Yay!").unwrap();
    };
    let (mut server, url) = make_server(handler);

    let context = new_fetch_context(None);

    assert_cookie_for_domain(context.state.cookie_jar.clone(), url.as_str(), None);

    let request = Request::from_init(RequestInit {
        url: url.clone(),
        method: Method::Get,
        body: None,
        destination: Destination::Document,
        origin: url.clone(),
        pipeline_id: Some(TEST_PIPELINE_ID),
        credentials_mode: CredentialsMode::Include,
        .. RequestInit::default()
    });
    let response = fetch_with_context(request, &context);

    let _ = server.close();

    assert!(response.status.unwrap().is_success());

    assert_cookie_for_domain(context.state.cookie_jar.clone(), url.as_str(), Some("mozillaIs=theBest"));
    let mut cookie_jar = context.state.cookie_jar.write().unwrap();
    assert!(cookie_jar.cookies_for_url(&url, CookieSource::NonHTTP).is_none());
}

#[test]
fn test_when_cookie_received_marked_secure_is_ignored_for_http() {
    let handler = move |_: HyperRequest, mut response: HyperResponse| {
        let mut pair = CookiePair::new("mozillaIs".to_owned(), "theBest".to_owned());
        pair.secure = true;
        response.headers_mut().set(SetCookie(vec![pair]));
        response.send(b"Yay!").unwrap();
    };
    let (mut server, url) = make_server(handler);

    let context = new_fetch_context(None);

    assert_cookie_for_domain(context.state.cookie_jar.clone(), url.as_str(), None);

    let request = Request::from_init(RequestInit {
        url: url.clone(),
        method: Method::Get,
        body: None,
        destination: Destination::Document,
        origin: url.clone(),
        pipeline_id: Some(TEST_PIPELINE_ID),
        credentials_mode: CredentialsMode::Include,
        .. RequestInit::default()
    });
    let response = fetch_with_context(request, &context);

    let _ = server.close();

    assert!(response.status.unwrap().is_success());

    assert_cookie_for_domain(context.state.cookie_jar.clone(), url.as_str(), None);
}

#[test]
fn test_load_sets_content_length_to_length_of_request_body() {
    let content = b"This is a request body";
    let content_length = ContentLength(content.len() as u64);
    let handler = move |request: HyperRequest, response: HyperResponse| {
        assert_eq!(request.headers.get::<ContentLength>(), Some(&content_length));
        response.send(content).unwrap();
    };
    let (mut server, url) = make_server(handler);

    let request = Request::from_init(RequestInit {
        url: url.clone(),
        method: Method::Post,
        body: Some(content.to_vec()),
        destination: Destination::Document,
        origin: url.clone(),
        pipeline_id: Some(TEST_PIPELINE_ID),
        .. RequestInit::default()
    });
    let response = fetch(request, None);

    let _ = server.close();

    assert!(response.status.unwrap().is_success());
}

#[test]
fn test_load_uses_explicit_accept_from_headers_in_load_data() {
    let accept = Accept(vec![qitem(Mime(TopLevel::Text, SubLevel::Html, vec![]))]);
    let expected_accept = accept.clone();
    let handler = move |request: HyperRequest, response: HyperResponse| {
        assert_eq!(request.headers.get::<Accept>(), Some(&expected_accept));
        response.send(b"Yay!").unwrap();
    };
    let (mut server, url) = make_server(handler);

    let mut accept_headers = Headers::new();
    accept_headers.set(accept);
    let request = Request::from_init(RequestInit {
        url: url.clone(),
        method: Method::Get,
        headers: accept_headers,
        destination: Destination::Document,
        origin: url.clone(),
        pipeline_id: Some(TEST_PIPELINE_ID),
        .. RequestInit::default()
    });
    let response = fetch(request, None);

    let _ = server.close();

    assert!(response.status.unwrap().is_success());
}

#[test]
fn test_load_sets_default_accept_to_html_xhtml_xml_and_then_anything_else() {
    let handler = move |request: HyperRequest, response: HyperResponse| {
        assert_eq!(request.headers.get::<Accept>(), Some(&Accept(vec![
            qitem(Mime(TopLevel::Text, SubLevel::Html, vec![])),
            qitem(Mime(TopLevel::Application, SubLevel::Ext("xhtml+xml".to_owned()), vec![])),
            QualityItem::new(Mime(TopLevel::Application, SubLevel::Xml, vec![]), Quality(900)),
        ])));
        response.send(b"Yay!").unwrap();
    };
    let (mut server, url) = make_server(handler);

    let request = Request::from_init(RequestInit {
        url: url.clone(),
        method: Method::Get,
        destination: Destination::Document,
        origin: url.clone(),
        pipeline_id: Some(TEST_PIPELINE_ID),
        .. RequestInit::default()
    });
    let response = fetch(request, None);

    let _ = server.close();

    assert!(response.status.unwrap().is_success());
}

#[test]
fn test_load_uses_explicit_accept_encoding_from_load_data_headers() {
    let accept_encoding = AcceptEncoding(vec![qitem(Encoding::Chunked)]);
    let expected_accept_encoding = accept_encoding.clone();
    let handler = move |request: HyperRequest, response: HyperResponse| {
        assert_eq!(request.headers.get::<AcceptEncoding>(), Some(&expected_accept_encoding));
        response.send(b"Yay!").unwrap();
    };
    let (mut server, url) = make_server(handler);

    let mut accept_encoding_headers = Headers::new();
    accept_encoding_headers.set(accept_encoding);
    let request = Request::from_init(RequestInit {
        url: url.clone(),
        method: Method::Get,
        headers: accept_encoding_headers,
        destination: Destination::Document,
        origin: url.clone(),
        pipeline_id: Some(TEST_PIPELINE_ID),
        .. RequestInit::default()
    });
    let response = fetch(request, None);

    let _ = server.close();

    assert!(response.status.unwrap().is_success());
}

#[test]
fn test_load_sets_default_accept_encoding_to_gzip_and_deflate() {
    let handler = move |request: HyperRequest, response: HyperResponse| {
        assert_eq!(request.headers.get::<AcceptEncoding>(), Some(&AcceptEncoding(vec![
            qitem(Encoding::Gzip),
            qitem(Encoding::Deflate),
            qitem(Encoding::EncodingExt("br".to_owned()))
        ])));
        response.send(b"Yay!").unwrap();
    };
    let (mut server, url) = make_server(handler);

    let request = Request::from_init(RequestInit {
        url: url.clone(),
        method: Method::Get,
        destination: Destination::Document,
        origin: url.clone(),
        pipeline_id: Some(TEST_PIPELINE_ID),
        .. RequestInit::default()
    });
    let response = fetch(request, None);

    let _ = server.close();

    assert!(response.status.unwrap().is_success());
}

#[test]
fn test_load_errors_when_there_a_redirect_loop() {
    let url_b_for_a = Arc::new(Mutex::new(None::<ServoUrl>));
    let url_b_for_a_clone = url_b_for_a.clone();
    let handler_a = move |_: HyperRequest, mut response: HyperResponse| {
        response.headers_mut().set(Location(url_b_for_a_clone.lock().unwrap().as_ref().unwrap().to_string()));
        *response.status_mut() = StatusCode::MovedPermanently;
        response.send(b"").unwrap();
    };
    let (mut server_a, url_a) = make_server(handler_a);

    let url_a_for_b = url_a.clone();
    let handler_b = move |_: HyperRequest, mut response: HyperResponse| {
        response.headers_mut().set(Location(url_a_for_b.to_string()));
        *response.status_mut() = StatusCode::MovedPermanently;
        response.send(b"").unwrap();
    };
    let (mut server_b, url_b) = make_server(handler_b);

    *url_b_for_a.lock().unwrap() = Some(url_b.clone());

    let request = Request::from_init(RequestInit {
        url: url_a.clone(),
        method: Method::Get,
        destination: Destination::Document,
        origin: url_a.clone(),
        pipeline_id: Some(TEST_PIPELINE_ID),
        .. RequestInit::default()
    });
    let response = fetch(request, None);

    let _ = server_a.close();
    let _ = server_b.close();

    assert_eq!(response.get_network_error(),
               Some(&NetworkError::Internal("Too many redirects".to_owned())));
}

#[test]
fn test_load_succeeds_with_a_redirect_loop() {
    let url_b_for_a = Arc::new(Mutex::new(None::<ServoUrl>));
    let url_b_for_a_clone = url_b_for_a.clone();
    let handled_a = AtomicBool::new(false);
    let handler_a = move |_: HyperRequest, mut response: HyperResponse| {
        if !handled_a.swap(true, Ordering::SeqCst) {
            response.headers_mut().set(Location(url_b_for_a_clone.lock().unwrap().as_ref().unwrap().to_string()));
            *response.status_mut() = StatusCode::MovedPermanently;
            response.send(b"").unwrap();
        } else {
            response.send(b"Success").unwrap();
        }
    };
    let (mut server_a, url_a) = make_server(handler_a);

    let url_a_for_b = url_a.clone();
    let handler_b = move |_: HyperRequest, mut response: HyperResponse| {
        response.headers_mut().set(Location(url_a_for_b.to_string()));
        *response.status_mut() = StatusCode::MovedPermanently;
        response.send(b"").unwrap();
    };
    let (mut server_b, url_b) = make_server(handler_b);

    *url_b_for_a.lock().unwrap() = Some(url_b.clone());

    let request = Request::from_init(RequestInit {
        url: url_a.clone(),
        method: Method::Get,
        destination: Destination::Document,
        origin: url_a.clone(),
        pipeline_id: Some(TEST_PIPELINE_ID),
        .. RequestInit::default()
    });
    let response = fetch(request, None);

    let _ = server_a.close();
    let _ = server_b.close();

    let response = response.to_actual();
    assert_eq!(*response.url_list.borrow(),
               [url_a.clone(), url_b, url_a]);
    assert_eq!(*response.body.lock().unwrap(),
               ResponseBody::Done(b"Success".to_vec()));
}

#[test]
fn test_load_follows_a_redirect() {
    let post_handler = move |request: HyperRequest, response: HyperResponse| {
        assert_eq!(request.method, Method::Get);
        response.send(b"Yay!").unwrap();
    };
    let (mut post_server, post_url) = make_server(post_handler);

    let post_redirect_url = post_url.clone();
    let pre_handler = move |request: HyperRequest, mut response: HyperResponse| {
        assert_eq!(request.method, Method::Get);
        response.headers_mut().set(Location(post_redirect_url.to_string()));
        *response.status_mut() = StatusCode::MovedPermanently;
        response.send(b"").unwrap();
    };
    let (mut pre_server, pre_url) = make_server(pre_handler);

    let request = Request::from_init(RequestInit {
        url: pre_url.clone(),
        method: Method::Get,
        destination: Destination::Document,
        origin: pre_url.clone(),
        pipeline_id: Some(TEST_PIPELINE_ID),
        .. RequestInit::default()
    });
    let response = fetch(request, None);

    let _ = pre_server.close();
    let _ = post_server.close();

    let response = response.to_actual();
    assert!(response.status.unwrap().is_success());
    assert_eq!(*response.body.lock().unwrap(),
               ResponseBody::Done(b"Yay!".to_vec()));
}

#[test]
fn  test_redirect_from_x_to_y_provides_y_cookies_from_y() {
    let shared_url_y = Arc::new(Mutex::new(None::<ServoUrl>));
    let shared_url_y_clone = shared_url_y.clone();
    let handler = move |request: HyperRequest, mut response: HyperResponse| {
        let path = match request.uri {
            RequestUri::AbsolutePath(path) => path,
            uri => panic!("Unexpected uri: {:?}", uri),
        };
        if path == "/com/" {
            assert_eq!(request.headers.get(),
                       Some(&CookieHeader(vec![CookiePair::new("mozillaIsNot".to_owned(), "dotOrg".to_owned())])));
            let location = shared_url_y.lock().unwrap().as_ref().unwrap().to_string();
            response.headers_mut().set(Location(location));
            *response.status_mut() = StatusCode::MovedPermanently;
            response.send(b"").unwrap();
        } else if path == "/org/" {
            assert_eq!(request.headers.get(),
                       Some(&CookieHeader(vec![CookiePair::new("mozillaIs".to_owned(), "theBest".to_owned())])));
            response.send(b"Yay!").unwrap();
        } else {
            panic!("unexpected path {:?}", path)
        }
    };
    let (mut server, url) = make_server(handler);
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

    let context = new_fetch_context(None);
    {
        let mut cookie_jar = context.state.cookie_jar.write().unwrap();
        let cookie_x = Cookie::new_wrapped(
            CookiePair::new("mozillaIsNot".to_owned(), "dotOrg".to_owned()),
            &url_x,
            CookieSource::HTTP
        ).unwrap();

        cookie_jar.push(cookie_x, &url_x, CookieSource::HTTP);

        let cookie_y = Cookie::new_wrapped(
            CookiePair::new("mozillaIs".to_owned(), "theBest".to_owned()),
            &url_y,
            CookieSource::HTTP
        ).unwrap();
        cookie_jar.push(cookie_y, &url_y, CookieSource::HTTP);
    }

    let request = Request::from_init(RequestInit {
        url: url_x.clone(),
        method: Method::Get,
        destination: Destination::Document,
        origin: url_x.clone(),
        pipeline_id: Some(TEST_PIPELINE_ID),
        credentials_mode: CredentialsMode::Include,
        .. RequestInit::default()
    });
    let response = fetch_with_context(request, &context);

    let _ = server.close();

    let response = response.to_actual();
    assert!(response.status.unwrap().is_success());
    assert_eq!(*response.body.lock().unwrap(),
               ResponseBody::Done(b"Yay!".to_vec()));
}

#[test]
fn test_redirect_from_x_to_x_provides_x_with_cookie_from_first_response() {
    let handler = move |request: HyperRequest, mut response: HyperResponse| {
        let path = match request.uri {
            ::hyper::uri::RequestUri::AbsolutePath(path) => path,
            uri => panic!("Unexpected uri: {:?}", uri),
        };
        if path == "/initial/" {
            response.headers_mut().set_raw("set-cookie", vec![b"mozillaIs=theBest; path=/;".to_vec()]);
            let location = "/subsequent/".to_string();
            response.headers_mut().set(Location(location));
            *response.status_mut() = StatusCode::MovedPermanently;
            response.send(b"").unwrap();
        } else if path == "/subsequent/" {
            assert_eq!(request.headers.get(),
                       Some(&CookieHeader(vec![CookiePair::new("mozillaIs".to_owned(), "theBest".to_owned())])));
            response.send(b"Yay!").unwrap();
        } else {
            panic!("unexpected path {:?}", path)
        }
    };
    let (mut server, url) = make_server(handler);

    let url = url.join("/initial/").unwrap();

    let request = Request::from_init(RequestInit {
        url: url.clone(),
        method: Method::Get,
        destination: Destination::Document,
        origin: url.clone(),
        pipeline_id: Some(TEST_PIPELINE_ID),
        credentials_mode: CredentialsMode::Include,
        .. RequestInit::default()
    });
    let response = fetch(request, None);

    let _ = server.close();

    let response = response.to_actual();
    assert!(response.status.unwrap().is_success());
    assert_eq!(*response.body.lock().unwrap(),
               ResponseBody::Done(b"Yay!".to_vec()));
}

#[test]
fn test_if_auth_creds_not_in_url_but_in_cache_it_sets_it() {
    let handler = move |request: HyperRequest, response: HyperResponse| {
        let expected = Authorization(Basic {
            username: "username".to_owned(),
            password: Some("test".to_owned())
        });
        assert_eq!(request.headers.get(), Some(&expected));
        response.send(b"").unwrap();
    };
    let (mut server, url) = make_server(handler);

    let request = Request::from_init(RequestInit {
        url: url.clone(),
        method: Method::Get,
        body: None,
        destination: Destination::Document,
        origin: url.clone(),
        pipeline_id: Some(TEST_PIPELINE_ID),
        credentials_mode: CredentialsMode::Include,
        .. RequestInit::default()
    });
    let context = new_fetch_context(None);

    let auth_entry = AuthCacheEntry {
        user_name: "username".to_owned(),
        password: "test".to_owned(),
    };

    context.state.auth_cache.write().unwrap().entries.insert(url.origin().clone().ascii_serialization(), auth_entry);

    let response = fetch_with_context(request, &context);

    let _ = server.close();

    assert!(response.status.unwrap().is_success());
}

#[test]
fn test_auth_ui_needs_www_auth() {
    let handler = move |_: HyperRequest, mut response: HyperResponse| {
        *response.status_mut() = StatusCode::Unauthorized;
        response.send(b"").unwrap();
    };
    let (mut server, url) = make_server(handler);

    let request = Request::from_init(RequestInit {
        url: url.clone(),
        method: Method::Get,
        body: None,
        destination: Destination::Document,
        origin: url.clone(),
        pipeline_id: Some(TEST_PIPELINE_ID),
        credentials_mode: CredentialsMode::Include,
        .. RequestInit::default()
    });

    let response = fetch(request, None);

    let _ = server.close();

    assert_eq!(response.status.unwrap(), StatusCode::Unauthorized);
}

#[test]
fn test_content_blocked() {
    let handler = move |_: HyperRequest, response: HyperResponse| {
        response.send(b"Yay!").unwrap();
    };
    let (mut server, url) = make_server(handler);

    let url_filter = url.as_str().replace("http://", "https?://");
    let blocked_content_list = format!("[{{ \
        \"trigger\": {{ \"url-filter\": \"{}\" }}, \
        \"action\": {{ \"type\": \"block\" }} \
    }}]", url_filter);

    let mut context = new_fetch_context(None);
    context.state.blocked_content = Arc::new(Some(parse_list(&blocked_content_list).unwrap()));

    let request = Request::from_init(RequestInit {
        url: url.clone(),
        method: Method::Get,
        body: None,
        destination: Destination::Document,
        origin: url.clone(),
        .. RequestInit::default()
    });

    let response = fetch_with_context(request, &context);

    let _ = server.close();

    // TODO(#14307): this should fail.
    assert!(response.status.unwrap().is_success());
}

#[test]
fn test_cookies_blocked() {
    let handler = move |request: HyperRequest, response: HyperResponse| {
        assert_eq!(request.headers.get::<CookieHeader>(), None);
        response.send(b"hi").unwrap();
    };
    let (mut server, url) = make_server(handler);

    let url_filter = url.as_str().replace("http://", "https?://");
    let blocked_content_list = format!("[{{ \
        \"trigger\": {{ \"url-filter\": \"{}\" }}, \
        \"action\": {{ \"type\": \"block-cookies\" }} \
    }}]", url_filter);

    let mut context = new_fetch_context(None);
    context.state.blocked_content = Arc::new(Some(parse_list(&blocked_content_list).unwrap()));
    {
        let mut cookie_jar = context.state.cookie_jar.write().unwrap();
        let cookie = Cookie::new_wrapped(
            CookiePair::new("mozillaIs".to_owned(), "theBest".to_owned()),
            &url,
            CookieSource::HTTP
        ).unwrap();
        cookie_jar.push(cookie, &url, CookieSource::HTTP);
    }

    let request = Request::from_init(RequestInit {
        url: url.clone(),
        method: Method::Get,
        body: None,
        destination: Destination::Document,
        origin: url.clone(),
        .. RequestInit::default()
    });

    let response = fetch_with_context(request, &context);

    let _ = server.close();

    assert!(response.status.unwrap().is_success());
}
