/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

mod common;

use std::rc::Rc;

use http_body_util::combinators::BoxBody;
use hyper::body::{Bytes, Incoming};
use hyper::{Request as HyperRequest, Response as HyperResponse};
use net::test_util::{make_body, make_server};
use servo::{JSValue, ServoUrl, SiteData, StorageType, WebViewBuilder};

use crate::common::{ServoTest, WebViewDelegateImpl, evaluate_javascript};

fn sites_equal_unordered(actual: &[SiteData], expected: &[(&ServoUrl, StorageType)]) -> bool {
    let mut actual = actual.to_vec();

    let mut expected: Vec<SiteData> = expected
        .iter()
        .map(|(url, storage_types)| {
            SiteData::new(url.origin().ascii_serialization(), *storage_types)
        })
        .collect();

    actual.sort_by(|a, b| a.name().cmp(&b.name()));
    expected.sort_by(|a, b| a.name().cmp(&b.name()));

    actual == expected
}

#[test]
fn test_site_data() {
    let servo_test = ServoTest::new();
    let servo = servo_test.servo();
    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo, servo_test.rendering_context.clone())
        .delegate(delegate.clone())
        .build();
    let delegate_clone = delegate.clone();
    servo_test.spin(move || !delegate_clone.url_changed.get());

    let site_data_manager = servo.site_data_manager();

    let sites = site_data_manager.site_data(StorageType::Local);
    assert_eq!(sites.len(), 0);
    let sites = site_data_manager.site_data(StorageType::Session);
    assert_eq!(sites.len(), 0);
    let sites = site_data_manager.site_data(StorageType::all());
    assert_eq!(sites.len(), 0);

    static MESSAGE: &'static [u8] = b"<!DOCTYPE html>\nHello";
    let handler =
        move |_: HyperRequest<Incoming>,
              response: &mut HyperResponse<BoxBody<Bytes, hyper::Error>>| {
            *response.body_mut() = make_body(MESSAGE.to_vec());
        };

    let (server1, url1) = make_server(handler);

    delegate.reset();
    webview.load(url1.clone().into_url());
    let delegate_clone = delegate.clone();
    servo_test.spin(move || !delegate_clone.url_changed.get());

    let _ = evaluate_javascript(
        &servo_test,
        webview.clone(),
        "localStorage.setItem('foo', 'bar');",
    );

    let sites = site_data_manager.site_data(StorageType::Local);
    assert!(sites_equal_unordered(
        &sites,
        &[(&url1, StorageType::Local),]
    ));
    let sites = site_data_manager.site_data(StorageType::Session);
    assert_eq!(sites.len(), 0);
    let sites = site_data_manager.site_data(StorageType::all());
    assert!(sites_equal_unordered(
        &sites,
        &[(&url1, StorageType::Local),]
    ));

    let (server2, url2) = make_server(handler);

    delegate.reset();
    webview.load(url2.clone().into_url());
    servo_test.spin(move || !delegate.url_changed.get());

    let _ = evaluate_javascript(
        &servo_test,
        webview.clone(),
        "sessionStorage.setItem('foo', 'bar');",
    );

    let sites = site_data_manager.site_data(StorageType::Local);
    // TODO: File an issue for this, there should be only one site with
    // localStorage origin.
    assert!(sites_equal_unordered(
        &sites,
        &[(&url1, StorageType::Local), (&url2, StorageType::Local),]
    ));
    let sites = site_data_manager.site_data(StorageType::Session);
    assert!(sites_equal_unordered(
        &sites,
        &[(&url2, StorageType::Session),]
    ));
    let sites = site_data_manager.site_data(StorageType::all());
    assert!(sites_equal_unordered(
        &sites,
        &[
            (&url1, StorageType::Local),
            (&url2, StorageType::Local | StorageType::Session),
        ]
    ));

    let _ = server1.close();
    let _ = server2.close();
}

#[test]
fn test_clear_cookies() {
    let servo_test = ServoTest::new();

    static MESSAGE: &'static [u8] = b"<!DOCTYPE html>\nHello";

    let handler =
        move |_: HyperRequest<Incoming>,
              response: &mut HyperResponse<BoxBody<Bytes, hyper::Error>>| {
            *response.body_mut() = make_body(MESSAGE.to_vec());
        };
    let (server, url) = make_server(handler);

    let delegate = Rc::new(WebViewDelegateImpl::default());

    let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .delegate(delegate.clone())
        .url(url.into_url())
        .build();

    servo_test.spin(move || !delegate.url_changed.get());

    let _ = server.close();

    let result = evaluate_javascript(
        &servo_test,
        webview.clone(),
        "document.cookie = 'foo1=bar1';\
         document.cookie = 'foo2=bar2';\
         document.cookie = 'foo3=bar3';\
         document.cookie;",
    );
    assert_eq!(
        result,
        Ok(JSValue::String("foo1=bar1; foo2=bar2; foo3=bar3".into()))
    );

    servo_test.servo().site_data_manager().clear_cookies();

    let result = evaluate_javascript(&servo_test, webview.clone(), "document.cookie");
    assert_eq!(result, Ok(JSValue::String("".into())));
}

#[test]
fn test_clear_cache() {
    let servo_test = ServoTest::new();

    static MESSAGE: &'static [u8] = b"<!DOCTYPE html>\nHello";

    let handler =
        move |_: HyperRequest<Incoming>,
              response: &mut HyperResponse<BoxBody<Bytes, hyper::Error>>| {
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

    servo_test.servo().site_data_manager().clear_cache();

    // TODO: Check that the cache was actually cleared once there's a way to
    //       check it.
}
