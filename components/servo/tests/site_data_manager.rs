/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

mod common;

use std::collections::HashMap;
use std::rc::Rc;

use http_body_util::combinators::BoxBody;
use hyper::body::{Bytes, Incoming};
use hyper::{Request as HyperRequest, Response as HyperResponse};
use net::test_util::{make_body, make_server, replace_host_table};
use servo::{JSValue, Servo, ServoUrl, SiteData, StorageType, WebView, WebViewBuilder};

use crate::common::{ServoTest, WebViewDelegateImpl, evaluate_javascript};

pub struct WebViewTest {
    servo_test: ServoTest,
    delegate: Rc<WebViewDelegateImpl>,
    webview: WebView,
}

impl WebViewTest {
    fn new() -> Self {
        let servo_test = ServoTest::new();
        let delegate = Rc::new(WebViewDelegateImpl::default());
        let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
            .delegate(delegate.clone())
            .build();
        let delegate_clone = delegate.clone();
        servo_test.spin(move || !delegate_clone.url_changed.get());

        Self {
            servo_test,
            delegate,
            webview,
        }
    }

    fn servo(&self) -> &Servo {
        &self.servo_test.servo()
    }

    fn load_and_wait(&self, url: ServoUrl) {
        self.delegate.reset();
        self.webview.load(url.clone().into_url());
        let delegate_clone = self.delegate.clone();
        self.servo_test
            .spin(move || !delegate_clone.url_changed.get());
    }

    fn evaluate_javascript(&self, script: impl ToString) {
        let _ = evaluate_javascript(&self.servo_test, self.webview.clone(), script);
    }
}

#[test]
fn test_site_data() {
    let webview_test = WebViewTest::new();

    let site_data_manager = webview_test.servo().site_data_manager();

    let sites = site_data_manager.site_data(StorageType::Cookies);
    assert_eq!(sites.len(), 0);
    let sites = site_data_manager.site_data(StorageType::Local);
    assert_eq!(sites.len(), 0);
    let sites = site_data_manager.site_data(StorageType::Session);
    assert_eq!(sites.len(), 0);
    let sites = site_data_manager.site_data(StorageType::all());
    assert_eq!(sites.len(), 0);

    let ip = "127.0.0.1".parse().unwrap();
    let mut host_table = HashMap::new();
    host_table.insert("www.site-data-0.test".to_owned(), ip);
    host_table.insert("www.site-data-1.test".to_owned(), ip);
    host_table.insert("www.site-data-2.test".to_owned(), ip);
    host_table.insert("www.site-data-3.test".to_owned(), ip);

    replace_host_table(host_table);

    static MESSAGE: &'static [u8] = b"<!DOCTYPE html>\nHello";
    let handler =
        move |_: HyperRequest<Incoming>,
              response: &mut HyperResponse<BoxBody<Bytes, hyper::Error>>| {
            *response.body_mut() = make_body(MESSAGE.to_vec());
        };

    let mut servers = Vec::new();
    for _ in 0..4 {
        let (server, url) = make_server(handler);
        servers.push((server, url));
    }

    servers.sort_by(|(_, a), (_, b)| a.cmp(b));

    let [
        (server0, url0),
        (server1, url1),
        (server2, url2),
        (server3, url3),
    ] = servers.try_into().unwrap();
    let port0 = url0.port().unwrap();
    let port1 = url1.port().unwrap();
    let port2 = url2.port().unwrap();
    let port3 = url3.port().unwrap();

    let custom_url0 = ServoUrl::parse(&format!("http://www.site-data-0.test:{}", port0)).unwrap();
    let custom_url1 = ServoUrl::parse(&format!("http://www.site-data-1.test:{}", port1)).unwrap();
    let custom_url2 = ServoUrl::parse(&format!("http://www.site-data-2.test:{}", port2)).unwrap();
    let custom_url3 = ServoUrl::parse(&format!("http://www.site-data-3.test:{}", port3)).unwrap();

    webview_test.load_and_wait(custom_url0);
    let _ = server0.close();
    webview_test.evaluate_javascript("document.cookie = 'foo=bar';");

    let sites = site_data_manager.site_data(StorageType::Cookies);
    assert_eq!(
        &sites,
        &[SiteData::new("site-data-0.test", StorageType::Cookies),]
    );
    let sites = site_data_manager.site_data(StorageType::Local);
    assert_eq!(sites.len(), 0);
    let sites = site_data_manager.site_data(StorageType::Session);
    assert_eq!(sites.len(), 0);
    let sites = site_data_manager.site_data(StorageType::all());
    assert_eq!(
        &sites,
        &[SiteData::new("site-data-0.test", StorageType::Cookies),]
    );

    webview_test.load_and_wait(custom_url1);
    let _ = server1.close();
    webview_test.evaluate_javascript("localStorage.setItem('foo', 'bar');");

    let sites = site_data_manager.site_data(StorageType::Cookies);
    assert_eq!(
        &sites,
        &[SiteData::new("site-data-0.test", StorageType::Cookies),]
    );
    let sites = site_data_manager.site_data(StorageType::Local);
    assert_eq!(
        &sites,
        &[SiteData::new("site-data-1.test", StorageType::Local),]
    );
    let sites = site_data_manager.site_data(StorageType::Session);
    assert_eq!(sites.len(), 0);
    let sites = site_data_manager.site_data(StorageType::all());
    assert_eq!(
        &sites,
        &[
            SiteData::new("site-data-0.test", StorageType::Cookies),
            SiteData::new("site-data-1.test", StorageType::Local),
        ]
    );

    webview_test.load_and_wait(custom_url2);
    let _ = server2.close();
    webview_test.evaluate_javascript("sessionStorage.setItem('foo', 'bar');");

    let sites = site_data_manager.site_data(StorageType::Cookies);
    assert_eq!(
        &sites,
        &[SiteData::new("site-data-0.test", StorageType::Cookies),]
    );
    let sites = site_data_manager.site_data(StorageType::Local);
    assert_eq!(
        &sites,
        &[SiteData::new("site-data-1.test", StorageType::Local),]
    );
    let sites = site_data_manager.site_data(StorageType::Session);
    assert_eq!(
        &sites,
        &[SiteData::new("site-data-2.test", StorageType::Session),]
    );
    let sites = site_data_manager.site_data(StorageType::all());
    assert_eq!(
        &sites,
        &[
            SiteData::new("site-data-0.test", StorageType::Cookies),
            SiteData::new("site-data-1.test", StorageType::Local),
            SiteData::new("site-data-2.test", StorageType::Session),
        ]
    );

    webview_test.load_and_wait(custom_url3);
    let _ = server3.close();
    webview_test.evaluate_javascript(
        "document.cookie = 'foo=bar';\
        localStorage.setItem('foo', 'bar');\
        sessionStorage.setItem('foo', 'bar');",
    );

    let sites = site_data_manager.site_data(StorageType::Cookies);
    assert_eq!(
        &sites,
        &[
            SiteData::new("site-data-0.test", StorageType::Cookies),
            SiteData::new("site-data-3.test", StorageType::Cookies),
        ]
    );
    let sites = site_data_manager.site_data(StorageType::Local);
    assert_eq!(
        &sites,
        &[
            SiteData::new("site-data-1.test", StorageType::Local),
            SiteData::new("site-data-3.test", StorageType::Local),
        ]
    );
    let sites = site_data_manager.site_data(StorageType::Session);
    assert_eq!(
        &sites,
        &[
            SiteData::new("site-data-2.test", StorageType::Session),
            SiteData::new("site-data-3.test", StorageType::Session),
        ]
    );
    let sites = site_data_manager.site_data(StorageType::all());
    assert_eq!(
        &sites,
        &[
            SiteData::new("site-data-0.test", StorageType::Cookies),
            SiteData::new("site-data-1.test", StorageType::Local),
            SiteData::new("site-data-2.test", StorageType::Session),
            SiteData::new(
                "site-data-3.test",
                StorageType::Cookies | StorageType::Local | StorageType::Session
            ),
        ]
    );
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
