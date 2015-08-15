/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use net::http_loader::{load, LoadError, HttpRequestFactory, HttpRequest, HttpResponse};
use net::resource_task::new_resource_task;
use net_traits::{ResourceTask, ControlMsg, CookieSource};
use url::Url;
use ipc_channel::ipc;
use net_traits::LoadData;
use hyper::method::Method;
use hyper::http::RawStatus;
use hyper::status::StatusCode;
use hyper::header::{Headers, Location, ContentLength};
use std::io::{self, Read};
use std::cmp::{self};
use std::borrow::Cow;

fn respond_with(body: Vec<u8>) -> MockResponse {
    let mut headers = Headers::new();
    headers.set(ContentLength(body.len() as u64));

    respond_with_headers(body, headers)
}

fn respond_with_headers(body: Vec<u8>, headers: Headers) -> MockResponse {
    MockResponse::new(
        headers.clone(),
        StatusCode::Ok,
        RawStatus(200, Cow::Borrowed("Ok")),
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
        Ok(_) => "".to_string(),
        _ => panic!("problem reading response")
    }
}

struct MockResponse {
    h: Headers,
    sc: StatusCode,
    sr: RawStatus,
    msg: Vec<u8>
}

impl MockResponse {
    fn new(h: Headers, sc: StatusCode, sr: RawStatus, msg: Vec<u8>) -> MockResponse {
        MockResponse { h: h, sc: sc, sr: sr, msg: msg }
    }
}

impl Read for MockResponse {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let buf_len = buf.len();
        for (a, b) in buf.iter_mut().zip(&self.msg[0 .. cmp::min(buf_len, self.msg.len())]) {
            *a = *b
        }

        Ok(cmp::min(buf.len(), self.msg.len()))
    }
}

impl HttpResponse for MockResponse {
    fn headers(&self) -> &Headers { &self.h }
    fn status(&self) -> StatusCode { self.sc }
    fn status_raw(&self) -> &RawStatus { &self.sr }
}

fn redirect_to(host: String) -> MockResponse {
    let mut headers = Headers::new();
    headers.set(Location(host.to_string()));

    MockResponse::new(
        headers,
        StatusCode::MovedPermanently,
        RawStatus(301, Cow::Borrowed("Moved Permanently")),
        <[_]>::to_vec("".as_bytes())
    )
}


enum RequestType {
    Redirect(String),
    Text(Vec<u8>),
    WithHeaders(Vec<u8>, Headers)
}

struct MockRequest {
    headers: Headers,
    t: RequestType
}

impl MockRequest {
    fn new(t: RequestType) -> MockRequest {
        MockRequest { headers: Headers::new(), t: t }
    }
}

fn response_for_request_type(t: RequestType) -> Result<MockResponse, LoadError> {
    match t {
        RequestType::Redirect(location) => {
            Ok(redirect_to(location))
        },
        RequestType::Text(b) => {
            Ok(respond_with(b))
        },
        RequestType::WithHeaders(b, h) => {
            Ok(respond_with_headers(b, h))
        }
    }
}

impl HttpRequest for MockRequest {
    type R=MockResponse;

    fn headers_mut(&mut self) -> &mut Headers { &mut self.headers }

    fn send(self, _: &Option<Vec<u8>>) -> Result<MockResponse, LoadError> {
        response_for_request_type(self.t)
    }
}

struct AssertMustHaveHeadersRequest {
    expected_headers: Headers,
    request_headers: Headers,
    t: RequestType
}

impl AssertMustHaveHeadersRequest {
    fn new(t: RequestType, expected_headers: Headers) -> Self {
        AssertMustHaveHeadersRequest { expected_headers: expected_headers, request_headers: Headers::new(), t: t }
    }
}

impl HttpRequest for AssertMustHaveHeadersRequest {
    type R=MockResponse;

    fn headers_mut(&mut self) -> &mut Headers { &mut self.request_headers }

    fn send(self, _: &Option<Vec<u8>>) -> Result<MockResponse, LoadError> {
        for header in self.expected_headers.iter() {
            assert!(self.request_headers.get_raw(header.name()).is_some());
            assert_eq!(
                self.request_headers.get_raw(header.name()).unwrap(),
                self.expected_headers.get_raw(header.name()).unwrap()
            )
        }

        response_for_request_type(self.t)
    }
}

struct AssertMustHaveHeadersRequestFactory {
    expected_headers: Headers,
    body: Vec<u8>
}

impl HttpRequestFactory for AssertMustHaveHeadersRequestFactory {
    type R=AssertMustHaveHeadersRequest;

    fn create(&self, _: Url, _: Method) -> Result<AssertMustHaveHeadersRequest, LoadError> {
        Ok(
            AssertMustHaveHeadersRequest::new(
                RequestType::Text(self.body.clone()),
                self.expected_headers.clone()
            )
        )
    }
}

fn assert_cookie_for_domain(resource_mgr: &ResourceTask, domain: &str, cookie: &str) {
    let (tx, rx) = ipc::channel().unwrap();
    resource_mgr.send(ControlMsg::GetCookiesForUrl(Url::parse(&*domain).unwrap(),
                                                   tx,
                                                   CookieSource::HTTP)).unwrap();
    if let Some(cookie_list) = rx.recv().unwrap() {
        assert_eq!(cookie.to_string(), cookie_list);
    } else {
        assert_eq!(cookie.len(), 0);
    }
}

#[test]
fn test_load_doesnt_add_host_to_sts_list_when_url_is_http_even_if_sts_headers_are_present() {
    struct Factory;

    impl HttpRequestFactory for Factory {
        type R=MockRequest;

        fn create(&self, _: Url, _: Method) -> Result<MockRequest, LoadError> {
            let content = <[_]>::to_vec("Yay!".as_bytes());
            let mut headers = Headers::new();
            headers.set_raw("Strict-Transport-Security", vec![b"max-age=31536000".to_vec()]);
            Ok(MockRequest::new(RequestType::WithHeaders(content, headers)))
        }
    }

    let url = Url::parse("http://mozilla.com").unwrap();
    let resource_mgr = new_resource_task(None, None);

    let load_data = LoadData::new(url.clone(), None);

    let _ = load::<MockRequest>(load_data, resource_mgr.clone(), None, &Factory);

    let (tx, rx) = ipc::channel().unwrap();
    resource_mgr.send(ControlMsg::GetHostMustBeSecured("mozilla.com".to_string(), tx)).unwrap();

    assert!(!rx.recv().unwrap());
}

#[test]
fn test_load_adds_host_to_sts_list_when_url_is_https_and_sts_headers_are_present() {
    struct Factory;

    impl HttpRequestFactory for Factory {
        type R=MockRequest;

        fn create(&self, _: Url, _: Method) -> Result<MockRequest, LoadError> {
            let content = <[_]>::to_vec("Yay!".as_bytes());
            let mut headers = Headers::new();
            headers.set_raw("Strict-Transport-Security", vec![b"max-age=31536000".to_vec()]);
            Ok(MockRequest::new(RequestType::WithHeaders(content, headers)))
        }
    }

    let url = Url::parse("https://mozilla.com").unwrap();
    let resource_mgr = new_resource_task(None, None);

    let load_data = LoadData::new(url.clone(), None);

    let _ = load::<MockRequest>(load_data, resource_mgr.clone(), None, &Factory);

    let (tx, rx) = ipc::channel().unwrap();
    resource_mgr.send(ControlMsg::GetHostMustBeSecured("mozilla.com".to_string(), tx)).unwrap();

    assert!(rx.recv().unwrap());
}

#[test]
fn test_load_sets_cookies_in_the_resource_manager_when_it_get_set_cookie_header_in_response() {
    struct Factory;

    impl HttpRequestFactory for Factory {
        type R=MockRequest;

        fn create(&self, _: Url, _: Method) -> Result<MockRequest, LoadError> {
            let content = <[_]>::to_vec("Yay!".as_bytes());
            let mut headers = Headers::new();
            headers.set_raw("set-cookie", vec![b"mozillaIs=theBest".to_vec()]);
            Ok(MockRequest::new(RequestType::WithHeaders(content, headers)))
        }
    }

    let url = Url::parse("http://mozilla.com").unwrap();
    let resource_mgr = new_resource_task(None, None);
    assert_cookie_for_domain(&resource_mgr, "http://mozilla.com", "");

    let load_data = LoadData::new(url.clone(), None);

    let _ = load::<MockRequest>(load_data, resource_mgr.clone(), None, &Factory);

    assert_cookie_for_domain(&resource_mgr, "http://mozilla.com", "mozillaIs=theBest");
}

#[test]
fn test_load_sets_requests_cookies_header_for_url_by_getting_cookies_from_the_resource_manager() {
    let url = Url::parse("http://mozilla.com").unwrap();
    let resource_mgr = new_resource_task(None, None);
    resource_mgr.send(ControlMsg::SetCookiesForUrl(Url::parse("http://mozilla.com").unwrap(),
                                                   "mozillaIs=theBest".to_string(),
                                                   CookieSource::HTTP)).unwrap();

    let mut load_data = LoadData::new(url.clone(), None);
    load_data.data = Some(<[_]>::to_vec("Yay!".as_bytes()));

    let mut cookie = Headers::new();
    cookie.set_raw("Cookie".to_owned(), vec![<[_]>::to_vec("mozillaIs=theBest".as_bytes())]);

    let _ = load::<AssertMustHaveHeadersRequest>(
        load_data.clone(), resource_mgr, None,
        &AssertMustHaveHeadersRequestFactory {
            expected_headers: cookie,
            body: <[_]>::to_vec(&*load_data.data.unwrap())
        });
}

#[test]
fn test_load_sets_content_length_to_length_of_request_body() {
    let content = "This is a request body";

    let url = Url::parse("http://mozilla.com").unwrap();
    let resource_mgr = new_resource_task(None, None);
    let mut load_data = LoadData::new(url.clone(), None);
    load_data.data = Some(<[_]>::to_vec(content.as_bytes()));

    let mut content_len_headers= Headers::new();
    content_len_headers.set_raw(
        "Content-Length".to_owned(), vec![<[_]>::to_vec(&*format!("{}", content.len()).as_bytes())]
    );

    let _ = load::<AssertMustHaveHeadersRequest>(
        load_data.clone(), resource_mgr, None,
        &AssertMustHaveHeadersRequestFactory {
            expected_headers: content_len_headers,
            body: <[_]>::to_vec(&*load_data.data.unwrap())
        });
}

#[test]
fn test_load_sets_default_accept_to_html_xhtml_xml_and_then_anything_else() {
    let mut accept_headers = Headers::new();
    accept_headers.set_raw(
        "Accept".to_owned(), vec![b"text/html, application/xhtml+xml, application/xml; q=0.9, */*; q=0.8".to_vec()]
    );

    let url = Url::parse("http://mozilla.com").unwrap();
    let resource_mgr = new_resource_task(None, None);
    let mut load_data = LoadData::new(url.clone(), None);
    load_data.data = Some(<[_]>::to_vec("Yay!".as_bytes()));

    let _ = load::<AssertMustHaveHeadersRequest>(load_data, resource_mgr, None, &AssertMustHaveHeadersRequestFactory {
        expected_headers: accept_headers,
        body: <[_]>::to_vec("Yay!".as_bytes())
    });
}

#[test]
fn test_load_sets_default_accept_encoding_to_gzip_and_deflate() {
    let mut accept_encoding_headers = Headers::new();
    accept_encoding_headers.set_raw("Accept-Encoding".to_owned(), vec![b"gzip, deflate".to_vec()]);

    let url = Url::parse("http://mozilla.com").unwrap();
    let resource_mgr = new_resource_task(None, None);
    let mut load_data = LoadData::new(url.clone(), None);
    load_data.data = Some(<[_]>::to_vec("Yay!".as_bytes()));

    let _ = load::<AssertMustHaveHeadersRequest>(load_data, resource_mgr, None, &AssertMustHaveHeadersRequestFactory {
        expected_headers: accept_encoding_headers,
        body: <[_]>::to_vec("Yay!".as_bytes())
    });
}

#[test]
fn test_load_errors_when_there_a_redirect_loop() {
    struct Factory;

    impl HttpRequestFactory for Factory {
        type R=MockRequest;

        fn create(&self, url: Url, _: Method) -> Result<MockRequest, LoadError> {
            if url.domain().unwrap() == "mozilla.com" {
                Ok(MockRequest::new(RequestType::Redirect("http://mozilla.org".to_string())))
            } else if url.domain().unwrap() == "mozilla.org" {
                Ok(MockRequest::new(RequestType::Redirect("http://mozilla.com".to_string())))
            } else {
                panic!("unexpected host {:?}", url)
            }
        }
    }

    let url = Url::parse("http://mozilla.com").unwrap();
    let resource_mgr = new_resource_task(None, None);
    let load_data = LoadData::new(url.clone(), None);

    match load::<MockRequest>(load_data, resource_mgr, None, &Factory) {
        Err(LoadError::InvalidRedirect(_, msg)) => {
            assert_eq!(msg, "redirect loop");
        },
        _ => panic!("expected max redirects to fail")
    }
}

#[test]
fn test_load_errors_when_there_is_too_many_redirects() {
    struct Factory;

    impl HttpRequestFactory for Factory {
        type R=MockRequest;

        fn create(&self, url: Url, _: Method) -> Result<MockRequest, LoadError> {
            if url.domain().unwrap() == "mozilla.com" {
                Ok(MockRequest::new(RequestType::Redirect(format!("{}/1", url.serialize()))))
            } else {
                panic!("unexpected host {:?}", url)
            }
        }
    }

    let url = Url::parse("http://mozilla.com").unwrap();
    let resource_mgr = new_resource_task(None, None);
    let load_data = LoadData::new(url.clone(), None);

    match load::<MockRequest>(load_data, resource_mgr, None, &Factory) {
        Err(LoadError::MaxRedirects(url)) => {
            assert_eq!(url.domain().unwrap(), "mozilla.com")
        },
        _ => panic!("expected max redirects to fail")
    }
}

#[test]
fn test_load_follows_a_redirect() {
    struct Factory;

    impl HttpRequestFactory for Factory {
        type R=MockRequest;

        fn create(&self, url: Url, _: Method) -> Result<MockRequest, LoadError> {
            if url.domain().unwrap() == "mozilla.com" {
                Ok(MockRequest::new(RequestType::Redirect("http://mozilla.org".to_string())))
            } else if url.domain().unwrap() == "mozilla.org" {
                Ok(
                    MockRequest::new(
                        RequestType::Text(
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
    let resource_mgr = new_resource_task(None, None);
    let load_data = LoadData::new(url.clone(), None);

    match load::<MockRequest>(load_data, resource_mgr, None, &Factory) {
        Err(_) => panic!("expected to follow a redirect"),
        Ok((mut r, _)) => {
            let response = read_response(&mut *r);
            assert_eq!(response, "Yay!".to_string());
        }
    }
}

struct DontConnectFactory;

impl HttpRequestFactory for DontConnectFactory {
    type R=MockRequest;

    fn create(&self, url: Url, _: Method) -> Result<MockRequest, LoadError> {
        Err(LoadError::Connection(url, "should not have connected".to_string()))
    }
}

#[test]
fn test_load_errors_when_scheme_is_not_http_or_https() {
    let url = Url::parse("ftp://not-supported").unwrap();
    let resource_mgr = new_resource_task(None, None);
    let load_data = LoadData::new(url.clone(), None);

    match load::<MockRequest>(load_data, resource_mgr, None, &DontConnectFactory) {
        Err(LoadError::UnsupportedScheme(_)) => {}
        _ => panic!("expected ftp scheme to be unsupported")
    }
}

#[test]
fn test_load_errors_when_viewing_source_and_inner_url_scheme_is_not_http_or_https() {
    let url = Url::parse("view-source:ftp://not-supported").unwrap();
    let resource_mgr = new_resource_task(None, None);
    let load_data = LoadData::new(url.clone(), None);

    match load::<MockRequest>(load_data, resource_mgr, None, &DontConnectFactory) {
        Err(LoadError::UnsupportedScheme(_)) => {}
        _ => panic!("expected ftp scheme to be unsupported")
    }
}
