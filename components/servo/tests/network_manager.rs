/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

mod common;

use std::rc::Rc;

use http_body_util::combinators::BoxBody;
use hyper::body::{Bytes, Incoming};
use hyper::header::{self, HeaderValue};
use hyper::{Request as HyperRequest, Response as HyperResponse};
use net::test_util::{make_body, make_server};
use servo::{CacheEntry, WebViewBuilder};

use crate::common::{ServoTest, WebViewDelegateImpl};

#[test]
fn test_cache_entries() {
    let servo_test = ServoTest::new();
    let servo = servo_test.servo();
    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo, servo_test.rendering_context.clone())
        .delegate(delegate.clone())
        .build();
    let delegate_clone = delegate.clone();
    servo_test.spin(move || !delegate_clone.url_changed.get());

    let network_manager = servo.network_manager();

    let cache_entries = network_manager.cache_entries();
    assert_eq!(cache_entries.len(), 0);

    static MESSAGE: &'static [u8] = b"<!DOCTYPE html>\nHello";
    let handler =
        move |_: HyperRequest<Incoming>,
              response: &mut HyperResponse<BoxBody<Bytes, hyper::Error>>| {
            response.headers_mut().insert(
                header::CACHE_CONTROL,
                HeaderValue::from_static("max-age=3600"),
            );
            *response.body_mut() = make_body(MESSAGE.to_vec());
        };

    let (server, url) = make_server(handler);
    let port = url.port().unwrap();

    delegate.reset();
    webview.load(url.clone().into_url());
    let delegate_clone = delegate.clone();
    servo_test.spin(move || !delegate_clone.url_changed.get());

    let _ = server.close();

    let cache_entries = network_manager.cache_entries();
    assert_eq!(
        &cache_entries,
        &[CacheEntry::new(format!("http://localhost:{port}/")),]
    );
}

#[test]
fn test_clear_cache() {
    let servo_test = ServoTest::new();

    static MESSAGE: &'static [u8] = b"<!DOCTYPE html>\nHello";

    let handler =
        move |_: HyperRequest<Incoming>,
              response: &mut HyperResponse<BoxBody<Bytes, hyper::Error>>| {
            response.headers_mut().insert(
                header::CACHE_CONTROL,
                HeaderValue::from_static("max-age=3600"),
            );
            *response.body_mut() = make_body(MESSAGE.to_vec());
        };
    let (server, url) = make_server(handler);

    let delegate = Rc::new(WebViewDelegateImpl::default());

    let _webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .delegate(delegate.clone())
        .url(url.into_url())
        .build();

    servo_test.spin(move || !delegate.url_changed.get());

    let _ = server.close();

    let network_manager = servo_test.servo().network_manager();

    let cache_entries = network_manager.cache_entries();
    assert_eq!(cache_entries.len(), 1);

    network_manager.clear_cache();

    let cache_entries = network_manager.cache_entries();
    assert_eq!(cache_entries.len(), 0);
}
