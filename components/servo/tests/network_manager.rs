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

use crate::common::{ServoTest, WebViewDelegateImpl, evaluate_javascript};

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
    webview.load(url.as_url().clone());
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
        .url(url.as_url().clone())
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

#[test]
fn test_set_network_online() {
    let servo_test = ServoTest::new();
    let delegate = Rc::new(WebViewDelegateImpl::default());

    let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .delegate(delegate.clone())
        .build();

    evaluate_javascript(
        &servo_test,
        webview.clone(),
        "addEventListener('online', () => console.log('online'))",
    )
    .unwrap();
    evaluate_javascript(
        &servo_test,
        webview.clone(),
        "addEventListener('offline', () => console.log('offline'))",
    )
    .unwrap();

    let network_manager = servo_test.servo().network_manager();
    network_manager.set_online_state(false);
    network_manager.set_online_state(true);

    servo_test.spin({
        let delegate = delegate.clone();
        move || delegate.console_messages.borrow().len() < 2
    });
    assert_eq!(
        delegate
            .console_messages
            .borrow()
            .iter()
            .map(|p| p.1.clone())
            .collect::<Vec<_>>(),
        vec!["offline".to_string(), "online".to_string(),]
    );
}
