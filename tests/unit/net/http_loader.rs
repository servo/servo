/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use net::http_loader::{load, LoadError, HttpRequestFactory, HttpRequest, HttpResponse};
use net::resource_task::new_resource_task;
use url::Url;
use std::sync::{Arc, Mutex};
use ipc_channel::ipc;
use net_traits::LoadData;
use net::hsts::HSTSList;
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

    MockResponse::new(
        headers,
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
    Text(Vec<u8>)
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

impl HttpRequest for MockRequest {
    type R=MockResponse;

    fn headers_mut(&mut self) -> &mut Headers { &mut self.headers }

    fn send(self, _: &Option<Vec<u8>>) -> Result<MockResponse, LoadError> {
        match self.t {
            RequestType::Redirect(location) => {
                Ok(redirect_to(location))
            },
            RequestType::Text(b) => {
                Ok(respond_with(b))
            }
        }
    }
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
    let hsts_list = Arc::new(Mutex::new(HSTSList { entries: Vec::new() }));

    match load::<MockRequest>(load_data, resource_mgr, None, hsts_list, &Factory) {
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
    let hsts_list = Arc::new(Mutex::new(HSTSList { entries: Vec::new() }));

    match load::<MockRequest>(load_data, resource_mgr, None, hsts_list, &Factory) {
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
    let hsts_list = Arc::new(Mutex::new(HSTSList { entries: Vec::new() }));

    match load::<MockRequest>(load_data, resource_mgr, None, hsts_list, &Factory) {
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
    let (cookies_chan, _) = ipc::channel().unwrap();
    let load_data = LoadData::new(url.clone(), None);
    let hsts_list = Arc::new(Mutex::new(HSTSList { entries: Vec::new() }));

    match load::<MockRequest>(load_data, cookies_chan, None, hsts_list, &DontConnectFactory) {
        Err(LoadError::UnsupportedScheme(_)) => {}
        _ => panic!("expected ftp scheme to be unsupported")
    }
}

#[test]
fn test_load_errors_when_viewing_source_and_inner_url_scheme_is_not_http_or_https() {
    let url = Url::parse("view-source:ftp://not-supported").unwrap();
    let (cookies_chan, _) = ipc::channel().unwrap();
    let load_data = LoadData::new(url.clone(), None);
    let hsts_list = Arc::new(Mutex::new(HSTSList { entries: Vec::new() }));

    match load::<MockRequest>(load_data, cookies_chan, None, hsts_list, &DontConnectFactory) {
        Err(LoadError::UnsupportedScheme(_)) => {}
        _ => panic!("expected ftp scheme to be unsupported")
    }
}
