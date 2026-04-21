/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

mod common;

use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use http::StatusCode;
use http_body_util::combinators::BoxBody;
use hyper::body::{Bytes, Incoming};
use hyper::header::{self, HeaderValue};
use hyper::{Request as HyperRequest, Response as HyperResponse};
use net::test_util::{make_body, make_server};
use servo::{CacheEntry, WebViewBuilder, WebViewDelegate};

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
fn test_network_error() {
    let has_run = Arc::new(AtomicBool::new(false));
    let servo_test = ServoTest::new();
    let servo = servo_test.servo();

    struct NetworkErrorDelegate {
        has_run: Arc<AtomicBool>,
    }

    impl WebViewDelegate for NetworkErrorDelegate {
        fn notify_load_status_changed(&self, _webview: servo::WebView, status: servo::LoadStatus) {
            if let servo::LoadStatus::Failed(status_code) = status {
                assert_eq!(status_code, StatusCode::NOT_FOUND);
                self.has_run
                    .store(true, std::sync::atomic::Ordering::SeqCst);
            }
        }
    }

    let delegate = Rc::new(NetworkErrorDelegate {
        has_run: has_run.clone(),
    });
    let _webview = WebViewBuilder::new(servo, servo_test.rendering_context.clone())
        .delegate(delegate.clone())
        .build();
    servo_test.spin(move || false);

    let handler =
        move |_: HyperRequest<Incoming>,
              response: &mut HyperResponse<BoxBody<Bytes, hyper::Error>>| {
            *response.status_mut() = StatusCode::from_u16(404).unwrap();
        };
    let (server, url) = make_server(handler);

    let _webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .delegate(delegate.clone())
        .url(url.as_url().clone())
        .build();

    let has_run_clone = has_run.clone();
    servo_test.spin(move || !has_run_clone.load(std::sync::atomic::Ordering::SeqCst));

    let _ = server.close();
}

#[test]
fn test_no_network_error_in_iframe() {
    let has_run = Arc::new(AtomicBool::new(false));
    let servo_test = ServoTest::new();
    let servo = servo_test.servo();

    static MESSAGE: &'static [u8] = b"<iframe src=\"example.com\"></iframe>";
    struct NetworkErrorDelegate {
        has_run: Arc<AtomicBool>,
    }

    impl WebViewDelegate for NetworkErrorDelegate {
        fn notify_load_status_changed(&self, _webview: servo::WebView, status: servo::LoadStatus) {
            if let servo::LoadStatus::Complete = status {
                self.has_run
                    .store(true, std::sync::atomic::Ordering::SeqCst);
            } else if let servo::LoadStatus::Failed(_status_code) = status {
                assert!(false);
            }
        }
    }

    let delegate = Rc::new(NetworkErrorDelegate {
        has_run: has_run.clone(),
    });
    let _webview = WebViewBuilder::new(servo, servo_test.rendering_context.clone())
        .delegate(delegate.clone())
        .build();
    servo_test.spin(move || false);

    let handler =
        move |_: HyperRequest<Incoming>,
              response: &mut HyperResponse<BoxBody<Bytes, hyper::Error>>| {
            *response.body_mut() = make_body(MESSAGE.to_vec());
        };
    let (server, url) = make_server(handler);

    let _webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .delegate(delegate.clone())
        .url(url.as_url().clone())
        .build();

    let has_run_clone = has_run.clone();
    servo_test.spin(move || !has_run_clone.load(std::sync::atomic::Ordering::SeqCst));

    let _ = server.close();
}
