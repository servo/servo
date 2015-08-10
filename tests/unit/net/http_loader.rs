/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use net::http_loader::{load, LoadError, HttpRequester, HttpResponse};
use net::resource_task::new_resource_task;
use url::Url;
use std::sync::{Arc, Mutex};
use ipc_channel::ipc;
use net_traits::LoadData;
use net::hsts::HSTSList;
use hyper::client::Response;
use hyper::method::Method;
use hyper::http::RawStatus;
use hyper::status::StatusCode;
use hyper::header::{Headers, Location, ContentLength};
use std::io::{self, Read};
use std::cmp::{self};
use std::mem::{self};
use std::borrow::Cow;

fn redirect_to(host: &str) -> Box<HttpResponse> {
    let mut headers = Headers::new();
    headers.set(Location(host.to_string()));

    struct Redirect {
        h: Headers,
        sr: RawStatus
    }

    impl Read for Redirect {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            Ok(0)
        }
    }

    impl HttpResponse for Redirect {
        fn headers(&self) -> &Headers { &self.h }
        fn status(&self) -> StatusCode { StatusCode::MovedPermanently }
        fn status_raw(&self) -> &RawStatus { &self.sr }
    }

    Box::new(
        Redirect {
            h: headers,
            sr: RawStatus(301, Cow::Borrowed("Moved Permanently"))
        }
    )
}

fn respond_with(body: &str) -> Box<HttpResponse> {
    let mut headers = Headers::new();
    headers.set(ContentLength(body.len() as u64));

    struct TextResponse {
        h: Headers,
        body: Vec<u8>,
        sr: RawStatus
    }

    impl Read for TextResponse {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            let buf_len = buf.len();
            for (a, b) in buf.iter_mut().zip(&self.body[0 .. cmp::min(buf_len, self.body.len())]) {
                *a = *b
            }

            Ok(cmp::min(buf.len(), self.body.len()))
        }
    }

    impl HttpResponse for TextResponse {
        fn headers(&self) -> &Headers { &self.h }
        fn status(&self) -> StatusCode { StatusCode::Ok }
        fn status_raw(&self) -> &RawStatus { &self.sr }
    }

    Box::new(
        TextResponse {
            h: headers,
            body: <[_]>::to_vec(body.as_bytes()),
            sr: RawStatus(200, Cow::Borrowed("Ok"))
        }
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

#[test]
fn test_load_errors_when_there_a_redirect_loop() {
    struct Redirector;

    impl HttpRequester for Redirector {
        fn send(&self, url: &Url, _: &Method, _: &Headers, _: &Option<Vec<u8>>) -> Result<Box<HttpResponse>, LoadError> {
            if url.domain().unwrap() == "mozilla.com" {
                Ok(redirect_to("http://mozilla.org"))
            } else if url.domain().unwrap() == "mozilla.org" {
                Ok(redirect_to("http://mozilla.com"))
            } else {
                panic!("unexpected host {:?}", url)
            }
        }
    }

    let url = Url::parse("http://mozilla.com").unwrap();
    let resource_mgr = new_resource_task(None, None);
    let load_data = LoadData::new(url.clone(), None);
    let hsts_list = Arc::new(Mutex::new(HSTSList { entries: Vec::new() }));

    match load(load_data, resource_mgr, None, hsts_list, &Redirector) {
        Err(LoadError::InvalidRedirect(_, msg)) => {
            assert_eq!(msg, "redirect loop");
        },
        _ => panic!("expected max redirects to fail")
    }
}

#[test]
fn test_load_errors_when_there_is_too_many_redirects() {
    struct Redirector;

    impl HttpRequester for Redirector {
        fn send(&self, url: &Url, _: &Method, _: &Headers, _: &Option<Vec<u8>>) -> Result<Box<HttpResponse>, LoadError> {
            if url.domain().unwrap() == "mozilla.com" {
                Ok(redirect_to(&*format!("{}/1", url.serialize())))
            } else {
                panic!("unexpected host {:?}", url)
            }
        }
    }

    let url = Url::parse("http://mozilla.com").unwrap();
    let resource_mgr = new_resource_task(None, None);
    let load_data = LoadData::new(url.clone(), None);
    let hsts_list = Arc::new(Mutex::new(HSTSList { entries: Vec::new() }));

    match load(load_data, resource_mgr, None, hsts_list, &Redirector) {
        Err(LoadError::MaxRedirects(url)) => {
            assert_eq!(url.domain().unwrap(), "mozilla.com")
        },
        _ => panic!("expected max redirects to fail")
    }
}

#[test]
fn test_load_follows_a_redirect() {
    struct Redirector;

    impl HttpRequester for Redirector {
        fn send(&self, url: &Url, _: &Method, _: &Headers, _: &Option<Vec<u8>>) -> Result<Box<HttpResponse>, LoadError> {
            if url.domain().unwrap() == "mozilla.com" {
                Ok(redirect_to("http://mozilla.org"))
            } else if url.domain().unwrap() == "mozilla.org" {
                Ok(respond_with("Yay!"))
            } else {
                panic!("unexpected host")
            }
        }
    }

    let url = Url::parse("http://mozilla.com").unwrap();
    let resource_mgr = new_resource_task(None, None);
    let load_data = LoadData::new(url.clone(), None);
    let hsts_list = Arc::new(Mutex::new(HSTSList { entries: Vec::new() }));

    match load(load_data, resource_mgr, None, hsts_list, &Redirector) {
        Err(_) => panic!("expected to follow a redirect"),
        Ok((mut r, m)) => {
            let response = read_response(&mut *r);
            assert_eq!(response, "Yay!".to_string());
        }
    }
}

struct DontConnectHttpRequester;

impl HttpRequester for DontConnectHttpRequester {
    fn send(&self, _: &Url, _: &Method, _: &Headers, _: &Option<Vec<u8>>) -> Result<Box<HttpResponse>, LoadError> {
        Err(LoadError::Connection(Url::parse("http://example.com").unwrap(), "shouldn't connect".to_string()))
    }
}

#[test]
fn test_load_errors_when_scheme_is_not_http_or_https() {
    let url = Url::parse("ftp://not-supported").unwrap();
    let (cookies_chan, _) = ipc::channel().unwrap();
    let load_data = LoadData::new(url.clone(), None);
    let hsts_list = Arc::new(Mutex::new(HSTSList { entries: Vec::new() }));

    match load(load_data, cookies_chan, None, hsts_list, &DontConnectHttpRequester) {
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

    match load(load_data, cookies_chan, None, hsts_list, &DontConnectHttpRequester) {
        Err(LoadError::UnsupportedScheme(_)) => {}
        _ => panic!("expected ftp scheme to be unsupported")
    }
}
