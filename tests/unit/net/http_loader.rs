/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use content_blocker::parse_list;
use cookie_rs::Cookie as CookiePair;
use devtools_traits::{ChromeToDevtoolsControlMsg, DevtoolsControlMsg, NetworkEvent};
use devtools_traits::HttpRequest as DevtoolsHttpRequest;
use devtools_traits::HttpResponse as DevtoolsHttpResponse;
use fetch_sync;
use flate2::Compression;
use flate2::write::{DeflateEncoder, GzEncoder};
use hyper::LanguageTag;
use hyper::header::{Accept, AcceptEncoding, ContentEncoding, ContentLength, Cookie as CookieHeader};
use hyper::header::{AcceptLanguage, Authorization, Basic, Date};
use hyper::header::{Encoding, Headers, Host, Location, Quality, QualityItem, Referer, SetCookie, qitem};
use hyper::header::{StrictTransportSecurity, UserAgent};
use hyper::http::RawStatus;
use hyper::method::Method;
use hyper::mime::{Mime, SubLevel, TopLevel};
use hyper::server::{Request as HyperRequest, Response as HyperResponse};
use hyper::status::StatusCode;
use make_server;
use msg::constellation_msg::{PipelineId, TEST_PIPELINE_ID};
use net::cookie::Cookie;
use net::cookie_storage::CookieStorage;
use net::fetch::methods::fetch;
use net::hsts::HstsEntry;
use net::resource_thread::{AuthCacheEntry, CancellationListener};
use net::test::{HttpRequest, HttpRequestFactory, HttpState, LoadError, UIProvider, load};
use net::test::{HttpResponse, LoadErrorType};
use net_traits::{CookieSource, IncludeSubdomains, LoadContext, LoadData};
use net_traits::{CustomResponse, LoadOrigin, Metadata, NetworkError, ReferrerPolicy};
use net_traits::request::{Request, RequestInit, CredentialsMode, Destination};
use net_traits::response::ResponseBody;
use new_fetch_context;
use std::borrow::Cow;
use std::io::{self, Cursor, Read, Write};
use std::rc::Rc;
use std::sync::{Arc, Mutex, RwLock, mpsc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use std::thread;
use url::Url;

const DEFAULT_USER_AGENT: &'static str = "Test-agent";

struct HttpTest;

impl LoadOrigin for HttpTest {
    fn referrer_url(&self) -> Option<Url> {
        None
    }
    fn referrer_policy(&self) -> Option<ReferrerPolicy> {
        None
    }
    fn pipeline_id(&self) -> Option<PipelineId> {
        Some(TEST_PIPELINE_ID)
    }
}

struct LoadOriginInfo<'a> {
    referrer_url: &'a str,
    referrer_policy: Option<ReferrerPolicy>,
}

impl<'a> LoadOrigin for LoadOriginInfo<'a> {
    fn referrer_url(&self) -> Option<Url> {
        Some(Url::parse(self.referrer_url).unwrap())
    }
    fn referrer_policy(&self) -> Option<ReferrerPolicy> {
        self.referrer_policy.clone()
    }
    fn pipeline_id(&self) -> Option<PipelineId> {
        None
    }
}

fn respond_with(body: Vec<u8>) -> MockResponse {
    let headers = Headers::new();
    respond_with_headers(body, headers)
}

fn respond_with_headers(body: Vec<u8>, mut headers: Headers) -> MockResponse {
    headers.set(ContentLength(body.len() as u64));

    MockResponse::new(
        headers,
        StatusCode::Ok,
        RawStatus(200, Cow::Borrowed("OK")),
        body
    )
}

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

struct MockResponse {
    h: Headers,
    sc: StatusCode,
    sr: RawStatus,
    msg: Cursor<Vec<u8>>
}

impl MockResponse {
    fn new(h: Headers, sc: StatusCode, sr: RawStatus, msg: Vec<u8>) -> MockResponse {
        MockResponse { h: h, sc: sc, sr: sr, msg: Cursor::new(msg) }
    }
}

impl Read for MockResponse {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.msg.read(buf)
    }
}

impl HttpResponse for MockResponse {
    fn headers(&self) -> &Headers { &self.h }
    fn status(&self) -> StatusCode { self.sc }
    fn status_raw(&self) -> &RawStatus { &self.sr }
}

fn redirect_to(host: String) -> MockResponse {
    let mut headers = Headers::new();
    headers.set(Location(host.to_owned()));

    MockResponse::new(
        headers,
        StatusCode::MovedPermanently,
        RawStatus(301, Cow::Borrowed("Moved Permanently")),
        b"".to_vec()
    )
}

struct TestProvider {
    username: String,
    password: String,
}

impl TestProvider {
    fn new() -> TestProvider {
        TestProvider { username: "default".to_owned(), password: "default".to_owned() }
    }
}
impl UIProvider for TestProvider {
    fn input_username_and_password(&self, _prompt: &str) -> (Option<String>, Option<String>) {
        (Some(self.username.to_owned()),
        Some(self.password.to_owned()))
    }
}

fn basic_auth(headers: Headers) -> MockResponse {
    MockResponse::new(
        headers,
        StatusCode::Unauthorized,
        RawStatus(401, Cow::Borrowed("Unauthorized")),
        b"".to_vec()
    )
}

fn redirect_with_headers(host: String, mut headers: Headers) -> MockResponse {
    headers.set(Location(host.to_string()));

    MockResponse::new(
        headers,
        StatusCode::MovedPermanently,
        RawStatus(301, Cow::Borrowed("Moved Permanently")),
        b"".to_vec()
    )
}

fn respond_404() -> MockResponse {
    MockResponse::new(
        Headers::new(),
        StatusCode::NotFound,
        RawStatus(404, Cow::Borrowed("Not Found")),
        b"".to_vec()
    )
}

enum ResponseType {
    Redirect(String),
    RedirectWithHeaders(String, Headers),
    Text(Vec<u8>),
    WithHeaders(Vec<u8>, Headers),
    NeedsAuth(Headers),
    Dummy404
}

struct MockRequest {
    t: ResponseType
}

impl MockRequest {
    fn new(t: ResponseType) -> MockRequest {
        MockRequest { t: t }
    }
}

fn response_for_request_type(t: ResponseType) -> Result<MockResponse, LoadError> {
    match t {
        ResponseType::Redirect(location) => {
            Ok(redirect_to(location))
        },
        ResponseType::RedirectWithHeaders(location, headers) => {
            Ok(redirect_with_headers(location, headers))
        },
        ResponseType::Text(b) => {
            Ok(respond_with(b))
        },
        ResponseType::WithHeaders(b, h) => {
            Ok(respond_with_headers(b, h))
        },
        ResponseType::NeedsAuth(h) => {
            Ok(basic_auth(h))
        },
        ResponseType::Dummy404 => {
            Ok(respond_404())
        }
    }
}

impl HttpRequest for MockRequest {
    type R = MockResponse;

    fn send(self, _: &Option<Vec<u8>>) -> Result<MockResponse, LoadError> {
        response_for_request_type(self.t)
    }
}

struct AssertAuthHeaderRequestFactory {
    expected_headers: Headers,
    body: Vec<u8>
}

impl HttpRequestFactory for AssertAuthHeaderRequestFactory {
    type R = MockRequest;

    fn create(&self, _: Url, _: Method, headers: Headers) -> Result<MockRequest, LoadError> {
        let request = if headers.has::<Authorization<Basic>>() {
            assert_headers_included(&self.expected_headers, &headers);
            MockRequest::new(ResponseType::Text(self.body.clone()))
        } else {
            let mut headers = Headers::new();
            headers.set_raw("WWW-Authenticate", vec![b"Basic realm=\"Test realm\"".to_vec()]);
            MockRequest::new(ResponseType::NeedsAuth(headers))
        };

        Ok(request)
    }
}

fn assert_headers_included(expected: &Headers, request: &Headers) {
    assert!(expected.len() != 0);
    for header in expected.iter() {
        assert!(request.get_raw(header.name()).is_some());
        assert_eq!(request.get_raw(header.name()).unwrap(),
                   expected.get_raw(header.name()).unwrap())
    }
}

struct AssertMustIncludeHeadersRequestFactory {
    expected_headers: Headers,
    body: Vec<u8>
}

impl HttpRequestFactory for AssertMustIncludeHeadersRequestFactory {
    type R = MockRequest;

    fn create(&self, _: Url, _: Method, headers: Headers) -> Result<MockRequest, LoadError> {
        assert_headers_included(&self.expected_headers, &headers);
        Ok(MockRequest::new(ResponseType::Text(self.body.clone())))
    }
}

fn assert_cookie_for_domain(cookie_jar: Arc<RwLock<CookieStorage>>, domain: &str, cookie: Option<&str>) {
    let mut cookie_jar = cookie_jar.write().unwrap();
    let url = Url::parse(&*domain).unwrap();
    let cookies = cookie_jar.cookies_for_url(&url, CookieSource::HTTP);
    assert_eq!(cookies.as_ref().map(|c| &**c), cookie);
}

struct AssertMustNotIncludeHeadersRequestFactory {
    headers_not_expected: Vec<String>,
    body: Vec<u8>
}

impl HttpRequestFactory for AssertMustNotIncludeHeadersRequestFactory {
    type R = MockRequest;

    fn create(&self, _: Url, _: Method, headers: Headers) -> Result<MockRequest, LoadError> {
        assert!(self.headers_not_expected.len() != 0);
        for header in &self.headers_not_expected {
            assert!(headers.get_raw(header).is_none());
        }

        Ok(MockRequest::new(ResponseType::Text(self.body.clone())))
    }
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
    let response = fetch_sync(request, None);
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
    let response = fetch_sync(request, None);
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
    let response = fetch_sync(request, None);
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
    let response = fetch_sync(request, Some(devtools_chan));
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
    let response = fetch_sync(request, Some(devtools_chan));
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
    let response = fetch_sync(request, Some(devtools_chan));

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
    let response = fetch_sync(request, None);

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
    let response = fetch_sync(request, None);

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
    let response = fetch_sync(request, None);

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
    let response = fetch_sync(request, None);

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
    let response = fetch(Rc::new(request), &mut None, &context);

    let _ = server.close();

    assert!(response.status.unwrap().is_success());
    assert_eq!(context.state.hsts_list.read().unwrap().is_host_secure(url.host_str().unwrap()), false);
}

#[test]
fn test_load_adds_host_to_sts_list_when_url_is_https_and_sts_headers_are_present() {
    struct Factory;

    impl HttpRequestFactory for Factory {
        type R = MockRequest;

        fn create(&self, _: Url, _: Method, _: Headers) -> Result<MockRequest, LoadError> {
            let content = <[_]>::to_vec("Yay!".as_bytes());
            let mut headers = Headers::new();
            headers.set(StrictTransportSecurity::excluding_subdomains(31536000));
            Ok(MockRequest::new(ResponseType::WithHeaders(content, headers)))
        }
    }

    let url = Url::parse("https://mozilla.com").unwrap();

    let load_data = LoadData::new(LoadContext::Browsing, url.clone(), &HttpTest);

    let http_state = HttpState::new();
    let ui_provider = TestProvider::new();

    let _ = load(&load_data,
                 &ui_provider, &http_state,
                 None,
                 &Factory,
                 DEFAULT_USER_AGENT.into(),
                 &CancellationListener::new(None),
                 None);

    assert!(http_state.hsts_list.read().unwrap().is_host_secure("mozilla.com"));
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
    let response = fetch(Rc::new(request), &mut None, &context);

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
        cookie_jar.push(cookie, CookieSource::HTTP);
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
    let response = fetch(Rc::new(request), &mut None, &context);

    let _ = server.close();

    assert!(response.status.unwrap().is_success());
}

#[test]
fn test_load_sends_secure_cookie_if_http_changed_to_https_due_to_entry_in_hsts_store() {
    let url = Url::parse("http://mozilla.com").unwrap();
    let secured_url = Url::parse("https://mozilla.com").unwrap();
    let ui_provider = TestProvider::new();
    let http_state = HttpState::new();
    {
        let mut hsts_list = http_state.hsts_list.write().unwrap();
        let entry = HstsEntry::new(
            "mozilla.com".to_owned(), IncludeSubdomains::Included, Some(1000000)
        ).unwrap();
        hsts_list.push(entry);
    }

    {
        let mut cookie_jar = http_state.cookie_jar.write().unwrap();
        let cookie_url = secured_url.clone();
        let mut cookie_pair = CookiePair::new("mozillaIs".to_owned(), "theBest".to_owned());
        cookie_pair.secure = true;

        let cookie = Cookie::new_wrapped(
            cookie_pair,
            &cookie_url,
            CookieSource::HTTP
        ).unwrap();
        cookie_jar.push(cookie, CookieSource::HTTP);
    }

    let mut load_data = LoadData::new(LoadContext::Browsing, url, &HttpTest);
    load_data.data = Some(<[_]>::to_vec("Yay!".as_bytes()));

    let mut headers = Headers::new();
    headers.set_raw("Cookie".to_owned(), vec![<[_]>::to_vec("mozillaIs=theBest".as_bytes())]);

    let _ = load(
        &load_data.clone(), &ui_provider, &http_state, None,
        &AssertMustIncludeHeadersRequestFactory {
            expected_headers: headers,
            body: <[_]>::to_vec(&*load_data.data.unwrap())
        }, DEFAULT_USER_AGENT.into(), &CancellationListener::new(None), None);
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
        cookie_jar.push(cookie, CookieSource::HTTP);
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
    let response = fetch(Rc::new(request), &mut None, &context);

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
    let response = fetch(Rc::new(request), &mut None, &context);

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
    let response = fetch(Rc::new(request), &mut None, &context);

    let _ = server.close();

    assert!(response.status.unwrap().is_success());

    assert_cookie_for_domain(context.state.cookie_jar.clone(), url.as_str(), None);
}

#[test]
fn test_when_cookie_set_marked_httpsonly_secure_isnt_sent_on_http_request() {
    let sec_url = Url::parse("https://mozilla.com").unwrap();
    let url = Url::parse("http://mozilla.com").unwrap();

    let http_state = HttpState::new();
    let ui_provider = TestProvider::new();

    {
        let mut cookie_jar = http_state.cookie_jar.write().unwrap();
        let cookie_url = sec_url.clone();
        let cookie = Cookie::new_wrapped(
            CookiePair::parse("mozillaIs=theBest; Secure;").unwrap(),
            &cookie_url,
            CookieSource::HTTP
        ).unwrap();
        cookie_jar.push(cookie, CookieSource::HTTP);
    }

    let mut load_data = LoadData::new(LoadContext::Browsing, url, &HttpTest);
    load_data.data = Some(<[_]>::to_vec("Yay!".as_bytes()));

    assert_cookie_for_domain(http_state.cookie_jar.clone(), "https://mozilla.com", Some("mozillaIs=theBest"));

    let _ = load(
        &load_data.clone(), &ui_provider, &http_state, None,
        &AssertMustNotIncludeHeadersRequestFactory {
            headers_not_expected: vec!["Cookie".to_owned()],
            body: <[_]>::to_vec(&*load_data.data.unwrap())
        }, DEFAULT_USER_AGENT.into(), &CancellationListener::new(None), None);
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
    let response = fetch_sync(request, None);

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
    let response = fetch_sync(request, None);

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
    let response = fetch_sync(request, None);

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
    let response = fetch_sync(request, None);

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
    let response = fetch_sync(request, None);

    let _ = server.close();

    assert!(response.status.unwrap().is_success());
}

#[test]
fn test_load_errors_when_there_a_redirect_loop() {
    let url_b_for_a = Arc::new(Mutex::new(None::<Url>));
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
    let response = fetch_sync(request, None);

    let _ = server_a.close();
    let _ = server_b.close();

    assert_eq!(response.get_network_error(),
               Some(&NetworkError::Internal("Too many redirects".to_owned())));
}

#[test]
fn test_load_succeeds_with_a_redirect_loop() {
    let url_b_for_a = Arc::new(Mutex::new(None::<Url>));
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
    let response = fetch_sync(request, None);

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
    struct Factory;

    impl HttpRequestFactory for Factory {
        type R = MockRequest;

        fn create(&self, url: Url, _: Method, _: Headers) -> Result<MockRequest, LoadError> {
            if url.domain().unwrap() == "mozilla.com" {
                Ok(MockRequest::new(ResponseType::Redirect("http://mozilla.org".to_owned())))
            } else if url.domain().unwrap() == "mozilla.org" {
                Ok(
                    MockRequest::new(
                        ResponseType::Text(
                            <[_]>::to_vec("Yay!".as_bytes())
                        )
                    )
                )
            } else {
                panic!("unexpected host {:?}", url)
            }
        }
    }

    let url = Url::parse("http://mozilla.com").unwrap();
    let load_data = LoadData::new(LoadContext::Browsing, url.clone(), &HttpTest);
    let http_state = HttpState::new();
    let ui_provider = TestProvider::new();

    match load(&load_data, &ui_provider, &http_state, None, &Factory,
               DEFAULT_USER_AGENT.into(), &CancellationListener::new(None), None) {
        Err(e) => panic!("expected to follow a redirect {:?}", e),
        Ok(mut lr) => {
            let response = read_response(&mut lr);
            assert_eq!(response, "Yay!".to_owned());
        }
    }
}

struct DontConnectFactory;

impl HttpRequestFactory for DontConnectFactory {
    type R = MockRequest;

    fn create(&self, url: Url, _: Method, _: Headers) -> Result<MockRequest, LoadError> {
        Err(LoadError::new(url, LoadErrorType::Connection { reason: "should not have connected".into() }))
    }
}

#[test]
fn test_load_errors_when_scheme_is_not_http_or_https() {
    let url = Url::parse("ftp://not-supported").unwrap();
    let load_data = LoadData::new(LoadContext::Browsing, url.clone(), &HttpTest);

    let http_state = HttpState::new();
    let ui_provider = TestProvider::new();

    match load(&load_data,
               &ui_provider, &http_state,
               None,
               &DontConnectFactory,
               DEFAULT_USER_AGENT.into(),
               &CancellationListener::new(None), None) {
        Err(ref load_err) if load_err.error == LoadErrorType::UnsupportedScheme { scheme: "ftp".into() } => (),
        _ => panic!("expected ftp scheme to be unsupported")
    }
}

#[test]
fn test_load_errors_when_viewing_source_and_inner_url_scheme_is_not_http_or_https() {
    let url = Url::parse("view-source:ftp://not-supported").unwrap();
    let load_data = LoadData::new(LoadContext::Browsing, url.clone(), &HttpTest);

    let http_state = HttpState::new();
    let ui_provider = TestProvider::new();

    match load(&load_data,
               &ui_provider, &http_state,
               None,
               &DontConnectFactory,
               DEFAULT_USER_AGENT.into(),
               &CancellationListener::new(None), None) {
        Err(ref load_err) if load_err.error == LoadErrorType::UnsupportedScheme { scheme: "ftp".into() } => (),
        _ => panic!("expected ftp scheme to be unsupported")
    }
}

#[test]
fn test_load_errors_when_cancelled() {
    use ipc_channel::ipc;
    use net::resource_thread::CancellableResource;
    use net_traits::ResourceId;

    struct Factory;

    impl HttpRequestFactory for Factory {
        type R = MockRequest;

        fn create(&self, _: Url, _: Method, _: Headers) -> Result<MockRequest, LoadError> {
            let mut headers = Headers::new();
            headers.set(Host { hostname: "Kaboom!".to_owned(), port: None });
            Ok(MockRequest::new(
                   ResponseType::WithHeaders(<[_]>::to_vec("BOOM!".as_bytes()), headers))
            )
        }
    }

    let (id_sender, _id_receiver) = ipc::channel().unwrap();
    let (cancel_sender, cancel_receiver) = mpsc::channel();
    let cancel_resource = CancellableResource::new(cancel_receiver, ResourceId(0), id_sender);
    let cancel_listener = CancellationListener::new(Some(cancel_resource));
    cancel_sender.send(()).unwrap();

    let url = Url::parse("https://mozilla.com").unwrap();
    let load_data = LoadData::new(LoadContext::Browsing, url.clone(), &HttpTest);
    let http_state = HttpState::new();
    let ui_provider = TestProvider::new();

    match load(&load_data,
               &ui_provider, &http_state,
               None,
               &Factory,
               DEFAULT_USER_AGENT.into(),
               &cancel_listener, None) {
        Err(ref load_err) if load_err.error == LoadErrorType::Cancelled => (),
        _ => panic!("expected load cancelled error!")
    }
}

#[test]
fn  test_redirect_from_x_to_y_provides_y_cookies_from_y() {
    let url_x = Url::parse("http://mozilla.com").unwrap();
    let url_y = Url::parse("http://mozilla.org").unwrap();

    struct Factory;

    impl HttpRequestFactory for Factory {
        type R = MockRequest;

        fn create(&self, url: Url, _: Method, headers: Headers) -> Result<MockRequest, LoadError> {
            if url.domain().unwrap() == "mozilla.com" {
                let mut expected_headers_x = Headers::new();
                expected_headers_x.set_raw("Cookie".to_owned(),
                    vec![<[_]>::to_vec("mozillaIsNot=dotCom".as_bytes())]);
                assert_headers_included(&expected_headers_x, &headers);

                Ok(MockRequest::new(
                    ResponseType::Redirect("http://mozilla.org".to_owned())))
            } else if url.domain().unwrap() == "mozilla.org" {
                let mut expected_headers_y = Headers::new();
                expected_headers_y.set_raw(
                    "Cookie".to_owned(),
                    vec![<[_]>::to_vec("mozillaIs=theBest".as_bytes())]);
                assert_headers_included(&expected_headers_y, &headers);

                Ok(MockRequest::new(
                    ResponseType::Text(<[_]>::to_vec("Yay!".as_bytes()))))
            } else {
                panic!("unexpected host {:?}", url)
            }
        }
    }

    let load_data = LoadData::new(LoadContext::Browsing, url_x.clone(), &HttpTest);

    let http_state = HttpState::new();
    let ui_provider = TestProvider::new();

    {
        let mut cookie_jar = http_state.cookie_jar.write().unwrap();
        let cookie_x_url = url_x.clone();
        let cookie_x = Cookie::new_wrapped(
            CookiePair::new("mozillaIsNot".to_owned(), "dotCom".to_owned()),
            &cookie_x_url,
            CookieSource::HTTP
        ).unwrap();

        cookie_jar.push(cookie_x, CookieSource::HTTP);

        let cookie_y_url = url_y.clone();
        let cookie_y = Cookie::new_wrapped(
            CookiePair::new("mozillaIs".to_owned(), "theBest".to_owned()),
            &cookie_y_url,
            CookieSource::HTTP
        ).unwrap();
        cookie_jar.push(cookie_y, CookieSource::HTTP);
    }

    match load(&load_data,
               &ui_provider, &http_state,
               None,
               &Factory,
               DEFAULT_USER_AGENT.into(),
               &CancellationListener::new(None), None) {
        Err(e) => panic!("expected to follow a redirect {:?}", e),
        Ok(mut lr) => {
            let response = read_response(&mut lr);
            assert_eq!(response, "Yay!".to_owned());
        }
    }
}

#[test]
fn test_redirect_from_x_to_x_provides_x_with_cookie_from_first_response() {
    let url = Url::parse("http://mozilla.org/initial/").unwrap();

    struct Factory;

    impl HttpRequestFactory for Factory {
        type R = MockRequest;

        fn create(&self, url: Url, _: Method, headers: Headers) -> Result<MockRequest, LoadError> {
            if url.path_segments().unwrap().next().unwrap() == "initial" {
                let mut initial_answer_headers = Headers::new();
                initial_answer_headers.set_raw("set-cookie", vec![b"mozillaIs=theBest; path=/;".to_vec()]);
                Ok(MockRequest::new(
                    ResponseType::RedirectWithHeaders("http://mozilla.org/subsequent/".to_owned(),
                        initial_answer_headers)))
            } else if url.path_segments().unwrap().next().unwrap() == "subsequent" {
                let mut expected_subsequent_headers = Headers::new();
                expected_subsequent_headers.set_raw("Cookie", vec![b"mozillaIs=theBest".to_vec()]);
                assert_headers_included(&expected_subsequent_headers, &headers);
                Ok(MockRequest::new(ResponseType::Text(b"Yay!".to_vec())))
            } else {
                panic!("unexpected host {:?}", url)
            }
        }
    }

    let load_data = LoadData::new(LoadContext::Browsing, url.clone(), &HttpTest);

    let http_state = HttpState::new();
    let ui_provider = TestProvider::new();

    match load(&load_data,
               &ui_provider, &http_state,
               None,
               &Factory,
               DEFAULT_USER_AGENT.into(),
               &CancellationListener::new(None), None) {
        Err(e) => panic!("expected to follow a redirect {:?}", e),
        Ok(mut lr) => {
            let response = read_response(&mut lr);
            assert_eq!(response, "Yay!".to_owned());
        }
    }
}

#[test]
fn test_if_auth_creds_not_in_url_but_in_cache_it_sets_it() {
    let url = Url::parse("http://mozilla.com").unwrap();

    let http_state = HttpState::new();
    let ui_provider = TestProvider::new();

    let auth_entry = AuthCacheEntry {
                        user_name: "username".to_owned(),
                        password: "test".to_owned(),
                     };

    http_state.auth_cache.write().unwrap().entries.insert(url.origin().clone().ascii_serialization(), auth_entry);

    let mut load_data = LoadData::new(LoadContext::Browsing, url, &HttpTest);
    load_data.credentials_flag = true;

    let mut auth_header = Headers::new();

    auth_header.set(
       Authorization(
           Basic {
               username: "username".to_owned(),
               password: Some("test".to_owned())
           }
       )
    );

    let _ = load(
        &load_data, &ui_provider, &http_state,
        None, &AssertMustIncludeHeadersRequestFactory {
            expected_headers: auth_header,
            body: <[_]>::to_vec(&[])
        }, DEFAULT_USER_AGENT.into(), &CancellationListener::new(None), None);
}

#[test]
fn test_auth_ui_sets_header_on_401() {
    let url = Url::parse("http://mozilla.com").unwrap();
    let http_state = HttpState::new();
    let ui_provider = TestProvider { username: "test".to_owned(), password: "test".to_owned() };

    let mut auth_header = Headers::new();

    auth_header.set(
       Authorization(
           Basic {
               username: "test".to_owned(),
               password: Some("test".to_owned())
           }
       )
    );

    let load_data = LoadData::new(LoadContext::Browsing, url, &HttpTest);

    match load(
        &load_data, &ui_provider, &http_state,
        None, &AssertAuthHeaderRequestFactory {
            expected_headers: auth_header,
            body: <[_]>::to_vec(&[])
        }, DEFAULT_USER_AGENT.into(), &CancellationListener::new(None), None) {
        Err(e) => panic!("response contained error {:?}", e),
        Ok(response) => {
            assert_eq!(response.metadata.status,
                       Some((200, b"OK".to_vec())));
        }
    }
}

#[test]
fn test_auth_ui_needs_www_auth() {
    let url = Url::parse("http://mozilla.com").unwrap();
    let http_state = HttpState::new();
    struct AuthProvider;
    impl UIProvider for AuthProvider {
        fn input_username_and_password(&self, _prompt: &str) -> (Option<String>, Option<String>) {
            panic!("shouldn't be invoked")
        }
    }

    struct Factory;

    impl HttpRequestFactory for Factory {
        type R = MockRequest;

        fn create(&self, _: Url, _: Method, _: Headers) -> Result<MockRequest, LoadError> {
            Ok(MockRequest::new(ResponseType::NeedsAuth(Headers::new())))
        }
    }

    let load_data = LoadData::new(LoadContext::Browsing, url, &HttpTest);

    let response = load(&load_data, &AuthProvider, &http_state,
                        None, &Factory, DEFAULT_USER_AGENT.into(),
                        &CancellationListener::new(None), None);
    match response {
        Err(e) => panic!("response contained error {:?}", e),
        Ok(response) => {
            assert_eq!(response.metadata.status,
                       Some((401, "Unauthorized".as_bytes().to_vec())));
        }
    }
}

fn assert_referrer_header_matches(origin_info: &LoadOrigin,
                                  request_url: &str,
                                  expected_referrer: &str) {
    let url = Url::parse(request_url).unwrap();
    let ui_provider = TestProvider::new();

    let load_data = LoadData::new(LoadContext::Browsing,
                                  url.clone(),
                                  origin_info);

    let mut referrer_headers = Headers::new();
    referrer_headers.set(Referer(expected_referrer.to_owned()));

    let http_state = HttpState::new();

    let _ = load(&load_data.clone(), &ui_provider, &http_state, None,
                 &AssertMustIncludeHeadersRequestFactory {
                     expected_headers: referrer_headers,
                     body: <[_]>::to_vec(&[])
                 }, DEFAULT_USER_AGENT.into(),
                 &CancellationListener::new(None), None);
}

fn assert_referrer_header_not_included(origin_info: &LoadOrigin, request_url: &str) {
    let url = Url::parse(request_url).unwrap();
    let ui_provider = TestProvider::new();

    let load_data = LoadData::new(LoadContext::Browsing,
                                  url.clone(),
                                  origin_info);

    let http_state = HttpState::new();

    let _ = load(
        &load_data.clone(), &ui_provider, &http_state, None,
        &AssertMustNotIncludeHeadersRequestFactory {
            headers_not_expected: vec!["Referer".to_owned()],
            body: <[_]>::to_vec(&[])
        }, DEFAULT_USER_AGENT.into(), &CancellationListener::new(None), None);
}

#[test]
fn test_referrer_set_to_origin_with_origin_policy() {
    let request_url = "http://mozilla.com";
    let referrer_url = "http://username:password@someurl.com/some/path#fragment";
    let referrer_policy = Some(ReferrerPolicy::Origin);
    let expected_referrer = "http://someurl.com/";

    let origin_info = LoadOriginInfo {
        referrer_url: referrer_url,
        referrer_policy: referrer_policy
    };

    assert_referrer_header_matches(&origin_info, request_url, expected_referrer);
}

#[test]
fn test_referrer_set_to_ref_url_with_sameorigin_policy_same_orig() {
    let request_url = "http://mozilla.com";
    let referrer_url = "http://username:password@mozilla.com/some/path#fragment";
    let referrer_policy = Some(ReferrerPolicy::SameOrigin);
    let expected_referrer = "http://mozilla.com/some/path";

    let origin_info = LoadOriginInfo {
        referrer_url: referrer_url,
        referrer_policy: referrer_policy
    };

    assert_referrer_header_matches(&origin_info, request_url, expected_referrer);
}

#[test]
fn test_no_referrer_set_with_sameorigin_policy_cross_orig() {
    let request_url = "http://mozilla.com";
    let referrer_url = "http://username:password@someurl.com/some/path#fragment";
    let referrer_policy = Some(ReferrerPolicy::SameOrigin);

    let origin_info = LoadOriginInfo {
        referrer_url: referrer_url,
        referrer_policy: referrer_policy
    };

    assert_referrer_header_not_included(&origin_info, request_url);
}

#[test]
fn test_referrer_set_to_stripped_url_with_unsafeurl_policy() {
    let request_url = "http://mozilla.com";
    let referrer_url = "http://username:password@someurl.com/some/path#fragment";
    let referrer_policy = Some(ReferrerPolicy::UnsafeUrl);
    let expected_referrer = "http://someurl.com/some/path";
    let origin_info = LoadOriginInfo {
        referrer_url: referrer_url,
        referrer_policy: referrer_policy
    };

    assert_referrer_header_matches(&origin_info, request_url, expected_referrer);
}

#[test]
fn test_referrer_with_originwhencrossorigin_policy_cross_orig() {
    let request_url = "http://mozilla.com";
    let referrer_url = "http://username:password@someurl.com/some/path#fragment";
    let referrer_policy = Some(ReferrerPolicy::OriginWhenCrossOrigin);
    let expected_referrer = "http://someurl.com/";

    let origin_info = LoadOriginInfo {
        referrer_url: referrer_url,
        referrer_policy: referrer_policy
    };

    assert_referrer_header_matches(&origin_info, request_url, expected_referrer);
}

#[test]
fn test_referrer_with_originwhencrossorigin_policy_same_orig() {
    let request_url = "http://mozilla.com";
    let referrer_url = "http://username:password@mozilla.com/some/path#fragment";
    let referrer_policy = Some(ReferrerPolicy::OriginWhenCrossOrigin);
    let expected_referrer = "http://mozilla.com/some/path";

    let origin_info = LoadOriginInfo {
        referrer_url: referrer_url,
        referrer_policy: referrer_policy
    };

    assert_referrer_header_matches(&origin_info, request_url, expected_referrer);
}

#[test]
fn test_http_to_https_considered_cross_origin_for_referrer_header_logic() {
    let request_url = "https://mozilla.com";
    let referrer_url = "http://mozilla.com/some/path";
    let referrer_policy = Some(ReferrerPolicy::OriginWhenCrossOrigin);
    let expected_referrer = "http://mozilla.com/";

    let origin_info = LoadOriginInfo {
        referrer_url: referrer_url,
        referrer_policy: referrer_policy
    };

    assert_referrer_header_matches(&origin_info, request_url, expected_referrer);
}

#[test]
fn test_referrer_with_strictorigin_policy_http_to_https() {
    let request_url = "https://mozilla.com";
    let referrer_url = "http://mozilla.com";
    let referrer_policy = Some(ReferrerPolicy::StrictOrigin);
    let expected_referrer = "http://mozilla.com/";

    let origin_info = LoadOriginInfo {
        referrer_url: referrer_url,
        referrer_policy: referrer_policy
    };

    assert_referrer_header_matches(&origin_info, request_url, expected_referrer);
}

#[test]
fn test_no_referrer_with_strictorigin_policy_https_to_http() {
    let request_url = "http://mozilla.com";
    let referrer_url = "https://mozilla.com/some/path";
    let referrer_policy = Some(ReferrerPolicy::StrictOrigin);

    let origin_info = LoadOriginInfo {
        referrer_url: referrer_url,
        referrer_policy: referrer_policy
    };

    assert_referrer_header_not_included(&origin_info, request_url);
}

#[test]
fn test_referrer_with_strictorigin_policy_http_to_http() {
    let request_url = "http://mozilla.com/";
    let referrer_url = "http://mozilla.com/some/path";
    let referrer_policy = Some(ReferrerPolicy::StrictOrigin);
    let expected_referrer = "http://mozilla.com/";

    let origin_info = LoadOriginInfo {
        referrer_url: referrer_url,
        referrer_policy: referrer_policy
    };

    assert_referrer_header_matches(&origin_info, request_url, expected_referrer);
}

#[test]
fn test_referrer_with_strictorigin_policy_https_to_https() {
    let request_url = "https://mozilla.com/";
    let referrer_url = "https://mozilla.com/some/path";
    let referrer_policy = Some(ReferrerPolicy::StrictOrigin);
    let expected_referrer = "https://mozilla.com/";

    let origin_info = LoadOriginInfo {
        referrer_url: referrer_url,
        referrer_policy: referrer_policy
    };

    assert_referrer_header_matches(&origin_info, request_url, expected_referrer);
}

#[test]
fn test_referrer_with_strictoriginwhencrossorigin_policy_https_to_https_same_origin() {
    let request_url = "https://mozilla.com";
    let referrer_url = "https://mozilla.com/some/path";
    let referrer_policy = Some(ReferrerPolicy::StrictOriginWhenCrossOrigin);
    let expected_referrer = "https://mozilla.com/some/path";

    let origin_info = LoadOriginInfo {
        referrer_url: referrer_url,
        referrer_policy: referrer_policy
    };

    assert_referrer_header_matches(&origin_info, request_url, expected_referrer);
}

#[test]
fn test_referrer_with_strictoriginwhencrossorigin_policy_https_to_https_cross_origin() {
    let request_url = "https://servo.mozilla.com";
    let referrer_url = "https://mozilla.com/some/path";
    let referrer_policy = Some(ReferrerPolicy::StrictOriginWhenCrossOrigin);
    let expected_referrer = "https://mozilla.com/";

    let origin_info = LoadOriginInfo {
        referrer_url: referrer_url,
        referrer_policy: referrer_policy
    };

    assert_referrer_header_matches(&origin_info, request_url, expected_referrer);
}

#[test]
fn test_referrer_set_with_strictoriginwhencrossorigin_policy_http_to_http_cross_orig() {
    let request_url = "http://servo.mozilla.com";
    let referrer_url = "http://mozilla.com/some/path";
    let referrer_policy = Some(ReferrerPolicy::StrictOriginWhenCrossOrigin);
    let expected_referrer = "http://mozilla.com/";

    let origin_info = LoadOriginInfo {
        referrer_url: referrer_url,
        referrer_policy: referrer_policy
    };

    assert_referrer_header_matches(&origin_info, request_url, expected_referrer);
}

#[test]
fn test_referrer_set_with_strictoriginwhencrossorigin_policy_http_to_http_same_orig() {
    let request_url = "http://mozilla.com";
    let referrer_url = "http://mozilla.com/some/path";
    let referrer_policy = Some(ReferrerPolicy::StrictOriginWhenCrossOrigin);
    let expected_referrer = "http://mozilla.com/some/path";

    let origin_info = LoadOriginInfo {
        referrer_url: referrer_url,
        referrer_policy: referrer_policy
    };

    assert_referrer_header_matches(&origin_info, request_url, expected_referrer);
}

#[test]
fn test_referrer_set_with_strictoriginwhencrossorigin_policy_http_to_https_cross_orig() {
    let request_url = "https://servo.mozilla.com";
    let referrer_url = "http://mozilla.com/some/path";
    let referrer_policy = Some(ReferrerPolicy::StrictOriginWhenCrossOrigin);
    let expected_referrer = "http://mozilla.com/";

    let origin_info = LoadOriginInfo {
        referrer_url: referrer_url,
        referrer_policy: referrer_policy
    };

    assert_referrer_header_matches(&origin_info, request_url, expected_referrer);
}

#[test]
fn test_referrer_set_with_strictoriginwhencrossorigin_policy_http_to_https_same_orig() {
    let request_url = "https://mozilla.com";
    let referrer_url = "http://mozilla.com/some/path";
    let referrer_policy = Some(ReferrerPolicy::StrictOriginWhenCrossOrigin);
    let expected_referrer = "http://mozilla.com/";

    let origin_info = LoadOriginInfo {
        referrer_url: referrer_url,
        referrer_policy: referrer_policy
    };

    assert_referrer_header_matches(&origin_info, request_url, expected_referrer);
}

#[test]
fn test_referrer_set_to_ref_url_with_noreferrerwhendowngrade_policy_https_to_https() {
    let request_url = "https://mozilla.com";
    let referrer_url = "https://username:password@mozilla.com/some/path#fragment";
    let referrer_policy = Some(ReferrerPolicy::NoReferrerWhenDowngrade);
    let expected_referrer = "https://mozilla.com/some/path";

    let origin_info = LoadOriginInfo {
        referrer_url: referrer_url,
        referrer_policy: referrer_policy,
    };

    assert_referrer_header_matches(&origin_info, request_url, expected_referrer);
}

#[test]
fn test_no_referrer_set_with_noreferrerwhendowngrade_policy_https_to_http() {
    let request_url = "http://mozilla.com";
    let referrer_url = "https://username:password@mozilla.com/some/path#fragment";
    let referrer_policy = Some(ReferrerPolicy::NoReferrerWhenDowngrade);

    let origin_info = LoadOriginInfo {
        referrer_url: referrer_url,
        referrer_policy: referrer_policy
    };

    assert_referrer_header_not_included(&origin_info, request_url)
}

#[test]
fn test_referrer_set_to_ref_url_with_noreferrerwhendowngrade_policy_http_to_https() {
    let request_url = "https://mozilla.com";
    let referrer_url = "http://username:password@mozilla.com/some/path#fragment";
    let referrer_policy = Some(ReferrerPolicy::NoReferrerWhenDowngrade);
    let expected_referrer = "http://mozilla.com/some/path";

    let origin_info = LoadOriginInfo {
        referrer_url: referrer_url,
        referrer_policy: referrer_policy
    };

    assert_referrer_header_matches(&origin_info, request_url, expected_referrer);
}

#[test]
fn test_referrer_set_to_ref_url_with_noreferrerwhendowngrade_policy_http_to_http() {
    let request_url = "http://mozilla.com";
    let referrer_url = "http://username:password@mozilla.com/some/path#fragment";
    let referrer_policy = Some(ReferrerPolicy::NoReferrerWhenDowngrade);
    let expected_referrer = "http://mozilla.com/some/path";

    let origin_info = LoadOriginInfo {
        referrer_url: referrer_url,
        referrer_policy: referrer_policy
    };

    assert_referrer_header_matches(&origin_info, request_url, expected_referrer);
}

#[test]
fn test_no_referrer_policy_follows_noreferrerwhendowngrade_https_to_https() {
    let request_url = "https://mozilla.com";
    let referrer_url = "https://username:password@mozilla.com/some/path#fragment";
    let referrer_policy = None;
    let expected_referrer = "https://mozilla.com/some/path";

    let origin_info = LoadOriginInfo {
        referrer_url: referrer_url,
        referrer_policy: referrer_policy
    };

    assert_referrer_header_matches(&origin_info, request_url, expected_referrer);
}

#[test]
fn test_no_referrer_policy_follows_noreferrerwhendowngrade_https_to_http() {
    let request_url = "http://mozilla.com";
    let referrer_url = "https://username:password@mozilla.com/some/path#fragment";
    let referrer_policy = None;

    let origin_info = LoadOriginInfo {
        referrer_url: referrer_url,
        referrer_policy: referrer_policy
    };

    assert_referrer_header_not_included(&origin_info, request_url);
}

#[test]
fn test_no_referrer_policy_follows_noreferrerwhendowngrade_http_to_https() {
    let request_url = "https://mozilla.com";
    let referrer_url = "http://username:password@mozilla.com/some/path#fragment";
    let referrer_policy = None;
    let expected_referrer = "http://mozilla.com/some/path";

    let origin_info = LoadOriginInfo {
        referrer_url: referrer_url,
        referrer_policy: referrer_policy
    };

    assert_referrer_header_matches(&origin_info, request_url, expected_referrer);
}

#[test]
fn test_no_referrer_policy_follows_noreferrerwhendowngrade_http_to_http() {
    let request_url = "http://mozilla.com";
    let referrer_url = "http://username:password@mozilla.com/some/path#fragment";
    let referrer_policy = None;
    let expected_referrer = "http://mozilla.com/some/path";

    let origin_info = LoadOriginInfo {
        referrer_url: referrer_url,
        referrer_policy: referrer_policy
    };

    assert_referrer_header_matches(&origin_info, request_url, expected_referrer);
}

#[test]
fn test_no_referrer_set_with_noreferrer_policy() {
    let request_url = "http://mozilla.com";
    let referrer_url = "http://someurl.com";
    let referrer_policy = Some(ReferrerPolicy::NoReferrer);

    let origin_info = LoadOriginInfo {
        referrer_url: referrer_url,
        referrer_policy: referrer_policy,
    };

    assert_referrer_header_not_included(&origin_info, request_url)
}

fn load_request_for_custom_response(expected_body: Vec<u8>) -> (Metadata, String) {
    use ipc_channel::ipc;
    let (sender, receiver) = ipc::channel().unwrap();

    struct Factory;
    impl HttpRequestFactory for Factory {
        type R = MockRequest;
        fn create(&self, _: Url, _: Method, _: Headers) -> Result<MockRequest, LoadError> {
            Ok(MockRequest::new(ResponseType::Dummy404))
        }
    }

    let mock_response = CustomResponse::new(
        Headers::new(),
        RawStatus(200, Cow::Borrowed("OK")),
        expected_body
    );
    let url = Url::parse("http://mozilla.com").unwrap();
    let http_state = HttpState::new();
    let ui_provider = TestProvider::new();
    let load_data = LoadData::new(LoadContext::Browsing, url.clone(), &HttpTest);

    let join_handle = thread::spawn(move || {
        let response = load(&load_data.clone(), &ui_provider, &http_state,
        None, &Factory, DEFAULT_USER_AGENT.into(), &CancellationListener::new(None), Some(sender));
        match response {
            Ok(mut response) => {
                let metadata = response.metadata.clone();
                let body = read_response(&mut response);
                (metadata, body)
            }
            Err(e) => panic!("Error Getting Response: {:?}", e)
        }
    });

    let mediator = receiver.recv().unwrap();
    mediator.response_chan.send(Some(mock_response)).unwrap();
    let (metadata, body) = join_handle.join().unwrap();
    (metadata, body)
}

#[test]
fn test_custom_response() {
    let expected_body = b"Yay!".to_vec();
    let (metadata, body) = load_request_for_custom_response(expected_body.clone());
    assert_eq!(metadata.status, Some((200, b"OK".to_vec())));
    assert_eq!(body, String::from_utf8(expected_body).unwrap());
}

#[test]
fn test_content_blocked() {
    struct Factory;
    impl HttpRequestFactory for Factory {
        type R = MockRequest;

        fn create(&self, _url: Url, _method: Method, _: Headers) -> Result<MockRequest, LoadError> {
            Ok(MockRequest::new(ResponseType::Text(<[_]>::to_vec("Yay!".as_bytes()))))
        }
    }

    let blocked_url = Url::parse("http://mozilla.com").unwrap();
    let url_without_cookies = Url::parse("http://mozilla2.com").unwrap();
    let mut http_state = HttpState::new();

    let blocked_content_list = "[{ \"trigger\": { \"url-filter\": \"https?://mozilla.com\" }, \
                                   \"action\": { \"type\": \"block\" } },\
                                 { \"trigger\": { \"url-filter\": \"https?://mozilla2.com\" }, \
                                   \"action\": { \"type\": \"block-cookies\" } }]";
    http_state.blocked_content = Arc::new(parse_list(blocked_content_list).ok());
    assert!(http_state.blocked_content.is_some());

    {
        let mut cookie_jar = http_state.cookie_jar.write().unwrap();
        let cookie = Cookie::new_wrapped(
            CookiePair::parse("mozillaIs=theBest;").unwrap(),
            &url_without_cookies,
            CookieSource::HTTP
        ).unwrap();
        cookie_jar.push(cookie, CookieSource::HTTP);
    }

    let ui_provider = TestProvider::new();

    let load_data = LoadData::new(LoadContext::Browsing, url_without_cookies, &HttpTest);

    let response = load(
        &load_data, &ui_provider, &http_state,
        None, &AssertMustNotIncludeHeadersRequestFactory {
            headers_not_expected: vec!["Cookie".to_owned()],
            body: b"hi".to_vec(),
        }, DEFAULT_USER_AGENT.into(), &CancellationListener::new(None), None);
    match response {
        Ok(_) => {},
        _ => panic!("request should have succeeded without cookies"),
    }

    let load_data = LoadData::new(LoadContext::Browsing, blocked_url, &HttpTest);

    let response = load(
        &load_data, &ui_provider, &http_state,
        None, &Factory,
        DEFAULT_USER_AGENT.into(), &CancellationListener::new(None), None);
    match response {
        Err(LoadError { error: LoadErrorType::ContentBlocked, .. }) => {},
        _ => panic!("request should have been blocked"),
    }
}
