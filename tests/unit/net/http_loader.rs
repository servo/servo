/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use net::http_loader::{load, LoadError, HttpRequester, HttpResponse};
use url::Url;
use std::sync::{Arc, Mutex};
use ipc_channel::ipc;
use net_traits::LoadData;
use net::hsts::HSTSList;
use hyper::client::Response;
use hyper::method::Method;
use hyper::header::Headers;

struct MockHttpRequester;

impl HttpRequester for MockHttpRequester {
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

    match load(load_data, cookies_chan, None, hsts_list, &MockHttpRequester) {
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

    match load(load_data, cookies_chan, None, hsts_list, &MockHttpRequester) {
        Err(LoadError::UnsupportedScheme(_)) => {}
        _ => panic!("expected ftp scheme to be unsupported")
    }
}
