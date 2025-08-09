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

use anyhow::{anyhow, ensure};
use common::{ServoTest, run_api_tests};
use servo::{
    JSValue, JavaScriptEvaluationError, LoadStatus, MessagePort, Theme, WebView, WebViewBuilder,
    WebViewDelegate,
};
use url::Url;

#[derive(Default)]
struct WebViewDelegateImpl {
    url_changed: Cell<bool>,
    onmessage: RefCell<Vec<JSValue>>,
}

impl WebViewDelegateImpl {
    pub(crate) fn reset(&self) {
        self.url_changed.set(false);
        self.onmessage.borrow_mut().clear();
    }
}

impl WebViewDelegate for WebViewDelegateImpl {
    fn notify_url_changed(&self, _webview: servo::WebView, _url: url::Url) {
        self.url_changed.set(true);
    }

    fn message_port_onmessage(
        &self,
        _webview: servo::WebView,
        _message_port: Rc<MessagePort>,
        data: JSValue,
    ) {
        self.onmessage.borrow_mut().push(data);
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

    let result = evaluate_javascript(servo_test, webview.clone(), "return undefined");
    ensure!(result == Ok(JSValue::Undefined));

    let result = evaluate_javascript(servo_test, webview.clone(), "return null");
    ensure!(result == Ok(JSValue::Null));

    let result = evaluate_javascript(servo_test, webview.clone(), "return 42");
    ensure!(result == Ok(JSValue::Number(42.0)));

    let result = evaluate_javascript(servo_test, webview.clone(), "return (3 + 4)");
    ensure!(result == Ok(JSValue::Number(7.0)));

    let result = evaluate_javascript(servo_test, webview.clone(), "return ('abc' + 'def')");
    ensure!(result == Ok(JSValue::String("abcdef".into())));

    let result = evaluate_javascript(
        servo_test,
        webview.clone(),
        "let foo = {blah: 123}; return foo",
    );
    ensure!(matches!(result, Ok(JSValue::Object(_))));
    if let Ok(JSValue::Object(values)) = result {
        ensure!(values.len() == 1);
        ensure!(values.get("blah") == Some(&JSValue::Number(123.0)));
    }

    let result = evaluate_javascript(servo_test, webview.clone(), "return [1, 2, 3, 4]");
    let expected = JSValue::Array(vec![
        JSValue::Number(1.0),
        JSValue::Number(2.0),
        JSValue::Number(3.0),
        JSValue::Number(4.0),
    ]);
    ensure!(result == Ok(expected));

    let result = evaluate_javascript(servo_test, webview.clone(), "return window");
    ensure!(matches!(result, Ok(JSValue::Window(..))));

    let result = evaluate_javascript(servo_test, webview.clone(), "return document.body");
    ensure!(matches!(result, Ok(JSValue::Element(..))));

    let result = evaluate_javascript(
        servo_test,
        webview.clone(),
        "document.body.innerHTML += '<iframe>'; return frames[0]",
    );
    ensure!(matches!(result, Ok(JSValue::Frame(..))));

    Ok(())
}

fn evaluate_javascript_with_messageport(
    servo_test: &ServoTest,
    webview: WebView,
    script: impl ToString,
) -> Result<(Rc<MessagePort>, JSValue), JavaScriptEvaluationError> {
    let load_webview = webview.clone();
    let _ = servo_test.spin(move || Ok(load_webview.load_status() != LoadStatus::Complete));

    let saved_result = Rc::new(RefCell::new(None));
    let callback_result = saved_result.clone();
    let port = webview.evaluate_javascript_with_message_port(script, move |result| {
        *callback_result.borrow_mut() = Some(result)
    });

    let spin_result = saved_result.clone();
    let _ = servo_test.spin(move || Ok(spin_result.borrow().is_none()));

    let js_value = (*saved_result.borrow())
        .clone()
        .expect("Should have waited until value available");
    js_value.map(|v| (port, v))
}

fn test_post_message_basic(servo_test: &ServoTest) -> Result<(), anyhow::Error> {
    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo_test.servo())
        .delegate(delegate.clone())
        .build();

    let Ok((port, value)) = evaluate_javascript_with_messageport(
        servo_test,
        webview.clone(),
        "port.onmessage = (msg) => port.postMessage(msg.data)",
    ) else {
        return Err(anyhow!("saw error when evaluating JS"));
    };
    ensure!(value == JSValue::Undefined);
    port.post_message(JSValue::Number(42.0));

    let delegate2 = delegate.clone();
    let _ = servo_test.spin(move || Ok(delegate2.onmessage.borrow().is_empty()));
    ensure!(delegate.onmessage.borrow().first() == Some(&JSValue::Number(42.0)));

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

    let is_dark_theme_script = "return window.matchMedia('(prefers-color-scheme: dark)').matches";

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
        test_post_message_basic,
        test_theme_change,
        // This test needs to be last, as it tests creating and dropping
        // a WebView right before shutdown.
        test_create_webview_and_immediately_drop_webview_before_shutdown
    );
}
