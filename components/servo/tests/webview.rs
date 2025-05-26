/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! WebView API unit tests.
//!
//! Since all Servo tests must run serially on the same thread, it is important
//! that tests never panic. In order to ensure this, use `anyhow::ensure!` instead
//! of `assert!` for test assertions. `ensure!` will produce a `Result::Err` in
//! place of panicking.

mod common;

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use anyhow::ensure;
use common::{ServoTest, run_api_tests};
use servo::{
    JSValue, JavaScriptEvaluationError, LoadStatus, Theme, WebView, WebViewBuilder, WebViewDelegate,
};
use url::Url;

#[derive(Default)]
struct WebViewDelegateImpl {
    url_changed: Cell<bool>,
}

impl WebViewDelegateImpl {
    pub(crate) fn reset(&self) {
        self.url_changed.set(false);
    }
}

impl WebViewDelegate for WebViewDelegateImpl {
    fn notify_url_changed(&self, _webview: servo::WebView, _url: url::Url) {
        self.url_changed.set(true);
    }
}

fn test_create_webview(servo_test: &ServoTest) -> Result<(), anyhow::Error> {
    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo_test.servo())
        .delegate(delegate.clone())
        .build();

    servo_test.spin(move || Ok(!delegate.url_changed.get()))?;

    let url = webview.url();
    ensure!(url.is_some());
    ensure!(url.unwrap().to_string() == "about:blank");

    Ok(())
}

fn evaluate_javascript(
    servo_test: &ServoTest,
    webview: WebView,
    script: impl ToString,
) -> Result<JSValue, JavaScriptEvaluationError> {
    let load_webview = webview.clone();
    let _ = servo_test.spin(move || Ok(load_webview.load_status() != LoadStatus::Complete));

    let saved_result = Rc::new(RefCell::new(None));
    let callback_result = saved_result.clone();
    webview.evaluate_javascript(script, move |result| {
        *callback_result.borrow_mut() = Some(result)
    });

    let spin_result = saved_result.clone();
    let _ = servo_test.spin(move || Ok(spin_result.borrow().is_none()));

    (*saved_result.borrow())
        .clone()
        .expect("Should have waited until value available")
}

fn test_evaluate_javascript_basic(servo_test: &ServoTest) -> Result<(), anyhow::Error> {
    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo_test.servo())
        .delegate(delegate.clone())
        .build();

    let result = evaluate_javascript(servo_test, webview.clone(), "undefined");
    ensure!(result == Ok(JSValue::Undefined));

    let result = evaluate_javascript(servo_test, webview.clone(), "null");
    ensure!(result == Ok(JSValue::Null));

    let result = evaluate_javascript(servo_test, webview.clone(), "42");
    ensure!(result == Ok(JSValue::Number(42.0)));

    let result = evaluate_javascript(servo_test, webview.clone(), "3 + 4");
    ensure!(result == Ok(JSValue::Number(7.0)));

    let result = evaluate_javascript(servo_test, webview.clone(), "'abc' + 'def'");
    ensure!(result == Ok(JSValue::String("abcdef".into())));

    let result = evaluate_javascript(servo_test, webview.clone(), "let foo = {blah: 123}; foo");
    ensure!(matches!(result, Ok(JSValue::Object(_))));
    if let Ok(JSValue::Object(values)) = result {
        ensure!(values.len() == 1);
        ensure!(values.get("blah") == Some(&JSValue::Number(123.0)));
    }

    let result = evaluate_javascript(servo_test, webview.clone(), "[1, 2, 3, 4]");
    let expected = JSValue::Array(vec![
        JSValue::Number(1.0),
        JSValue::Number(2.0),
        JSValue::Number(3.0),
        JSValue::Number(4.0),
    ]);
    ensure!(result == Ok(expected));

    let result = evaluate_javascript(servo_test, webview.clone(), "window");
    ensure!(matches!(result, Ok(JSValue::Window(..))));

    let result = evaluate_javascript(servo_test, webview.clone(), "document.body");
    ensure!(matches!(result, Ok(JSValue::Element(..))));

    let result = evaluate_javascript(
        servo_test,
        webview.clone(),
        "document.body.innerHTML += '<iframe>'; frames[0]",
    );
    ensure!(matches!(result, Ok(JSValue::Frame(..))));

    Ok(())
}

fn test_create_webview_and_immediately_drop_webview_before_shutdown(
    servo_test: &ServoTest,
) -> Result<(), anyhow::Error> {
    WebViewBuilder::new(servo_test.servo()).build();
    Ok(())
}

fn test_theme_change(servo_test: &ServoTest) -> Result<(), anyhow::Error> {
    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo_test.servo())
        .delegate(delegate.clone())
        .url(Url::parse("data:text/html,page one").unwrap())
        .build();

    let is_dark_theme_script = "window.matchMedia('(prefers-color-scheme: dark)').matches";

    // The default theme is "light".
    let result = evaluate_javascript(servo_test, webview.clone(), is_dark_theme_script);
    ensure!(result == Ok(JSValue::Boolean(false)));

    // Changing the theme updates the current page.
    webview.notify_theme_change(Theme::Dark);
    let result = evaluate_javascript(servo_test, webview.clone(), is_dark_theme_script);
    ensure!(result == Ok(JSValue::Boolean(true)));

    delegate.reset();
    webview.load(Url::parse("data:text/html,page two").unwrap());
    servo_test.spin(move || Ok(!delegate.url_changed.get()))?;

    // The theme persists after a navigation.
    let result = evaluate_javascript(servo_test, webview.clone(), is_dark_theme_script);
    ensure!(result == Ok(JSValue::Boolean(true)));

    Ok(())
}

fn main() {
    run_api_tests!(
        test_create_webview,
        test_evaluate_javascript_basic,
        test_theme_change,
        // This test needs to be last, as it tests creating and dropping
        // a WebView right before shutdown.
        test_create_webview_and_immediately_drop_webview_before_shutdown
    );
}
