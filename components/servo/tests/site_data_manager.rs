/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

mod common;

use std::rc::Rc;

use http_body_util::combinators::BoxBody;
use hyper::body::{Bytes, Incoming};
use hyper::{Request as HyperRequest, Response as HyperResponse};
use net::test_util::{make_body, make_server};
use servo::{JSValue, SiteData, StorageTypes, WebViewBuilder};
use servo_url::ImmutableOrigin;

use crate::common::{ServoTest, WebViewDelegateImpl, evaluate_javascript};

fn sites_equal_unordered(sites: &[SiteData], expected_origins: &[ImmutableOrigin]) -> bool {
    let mut actual: Vec<String> = sites.iter().map(|s| s.name()).collect();

    let mut expected: Vec<String> = expected_origins
        .iter()
        .map(|o| o.ascii_serialization())
        .collect();

    actual.sort();
    expected.sort();

    actual == expected
}

#[test]
fn test_list_sites() {
    let servo_test = ServoTest::new();
    let servo = servo_test.servo();
    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo, servo_test.rendering_context.clone())
        .delegate(delegate.clone())
        .build();
    let delegate_clone = delegate.clone();
    servo_test.spin(move || !delegate_clone.url_changed.get());

    let site_data_manager = servo.site_data_manager();

    let sites = site_data_manager.list_sites(StorageTypes::LOCAL_STORAGE);
    assert_eq!(sites.len(), 0);
    let sites = site_data_manager.list_sites(StorageTypes::SESSION_STORAGE);
    assert_eq!(sites.len(), 0);
    let sites = site_data_manager.list_sites(StorageTypes::ALL);
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

    let sites = site_data_manager.list_sites(StorageTypes::LOCAL_STORAGE);
    assert!(sites_equal_unordered(&sites, &[url1.origin()]));
    let sites = site_data_manager.list_sites(StorageTypes::SESSION_STORAGE);
    assert_eq!(sites.len(), 0);
    let sites = site_data_manager.list_sites(StorageTypes::ALL);
    assert!(sites_equal_unordered(&sites, &[url1.origin()]));

    let (server2, url2) = make_server(handler);

    delegate.reset();
    webview.load(url2.clone().into_url());
    servo_test.spin(move || !delegate.url_changed.get());

    let _ = evaluate_javascript(
        &servo_test,
        webview.clone(),
        "sessionStorage.setItem('foo', 'bar');",
    );

    let sites = site_data_manager.list_sites(StorageTypes::LOCAL_STORAGE);
    // XXX File issue for this, there should be only one site with localStorage
    //     origin.
    assert!(sites_equal_unordered(
        &sites,
        &[url1.origin(), url2.origin()]
    ));
    let sites = site_data_manager.list_sites(StorageTypes::SESSION_STORAGE);
    assert!(sites_equal_unordered(&sites, &[url2.origin()]));
    let sites = site_data_manager.list_sites(StorageTypes::ALL);
    assert!(sites_equal_unordered(
        &sites,
        &[url1.origin(), url2.origin()]
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
