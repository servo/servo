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
use common::{ServoTest, WebViewDelegateImpl, evaluate_javascript, run_api_tests};
use dpi::PhysicalSize;
use euclid::{Point2D, Size2D};
use servo::{
    Cursor, InputEvent, JSValue, JavaScriptEvaluationError, LoadStatus, MouseLeftViewportEvent,
    MouseMoveEvent, Servo, Theme, WebView, WebViewBuilder, WebViewDelegate,
};
use url::Url;
use webrender_api::units::DeviceIntSize;

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
        "document.body.attachShadow({mode: 'open'})",
    );
    ensure!(matches!(result, Ok(JSValue::ShadowRoot(..))));

    let result = evaluate_javascript(servo_test, webview.clone(), "document.body.shadowRoot");
    ensure!(matches!(result, Ok(JSValue::ShadowRoot(..))));

    let result = evaluate_javascript(
        servo_test,
        webview.clone(),
        "document.body.innerHTML += '<iframe>'; frames[0]",
    );
    ensure!(matches!(result, Ok(JSValue::Frame(..))));

    let result = evaluate_javascript(servo_test, webview.clone(), "lettt badsyntax = 123");
    ensure!(result == Err(JavaScriptEvaluationError::CompilationFailure));

    let result = evaluate_javascript(servo_test, webview.clone(), "throw new Error()");
    ensure!(result == Err(JavaScriptEvaluationError::EvaluationFailure));

    Ok(())
}

fn test_evaluate_javascript_panic(servo_test: &ServoTest) -> Result<(), anyhow::Error> {
    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo_test.servo())
        .delegate(delegate.clone())
        .build();

    let input = "location";
    let result = evaluate_javascript(servo_test, webview.clone(), input);
    ensure!(matches!(result, Ok(JSValue::Object(..))));

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

fn test_cursor_change(servo_test: &ServoTest) -> Result<(), anyhow::Error> {
    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo_test.servo())
        .delegate(delegate.clone())
        .url(
            Url::parse(
                "data:text/html,<!DOCTYPE html><style> html { cursor: crosshair; margin: 0}</style><body>hello</body>",
            )
            .unwrap(),
        )
        .build();

    webview.focus();
    webview.show(true);
    webview.move_resize(servo_test.rendering_context.size2d().to_f32().into());

    let load_webview = webview.clone();
    let _ = servo_test.spin(move || Ok(load_webview.load_status() != LoadStatus::Complete));

    // Wait for at least one frame after the load completes.
    delegate.reset();
    let captured_delegate = delegate.clone();
    servo_test.spin(move || Ok(!captured_delegate.new_frame_ready.get()))?;

    webview.notify_input_event(InputEvent::MouseMove(MouseMoveEvent::new(Point2D::new(
        10., 10.,
    ))));

    let captured_delegate = delegate.clone();
    servo_test.spin(move || Ok(!captured_delegate.cursor_changed.get()))?;
    ensure!(webview.cursor() == Cursor::Crosshair);

    delegate.reset();
    webview.notify_input_event(InputEvent::MouseLeftViewport(
        MouseLeftViewportEvent::default(),
    ));

    let captured_delegate = delegate.clone();
    servo_test.spin(move || Ok(!captured_delegate.cursor_changed.get()))?;
    ensure!(webview.cursor() == Cursor::Default);

    Ok(())
}

/// A test that ensure that negative resize requests do not get passed to the embedder.
fn test_negative_resize_to_request(servo_test: &ServoTest) -> Result<(), anyhow::Error> {
    struct WebViewResizeTestDelegate {
        servo: Rc<Servo>,
        popup: RefCell<Option<WebView>>,
        resize_request: Cell<Option<DeviceIntSize>>,
    }

    impl WebViewDelegate for WebViewResizeTestDelegate {
        fn request_open_auxiliary_webview(&self, parent_webview: WebView) -> Option<WebView> {
            let webview = WebViewBuilder::new_auxiliary(&self.servo)
                .delegate(parent_webview.delegate())
                .build();
            self.popup.borrow_mut().replace(webview.clone());
            Some(webview)
        }

        fn request_resize_to(&self, _: WebView, requested_outer_size: DeviceIntSize) {
            self.resize_request.set(Some(requested_outer_size));
        }
    }

    let delegate = Rc::new(WebViewResizeTestDelegate {
        servo: servo_test.servo.clone(),
        popup: None.into(),
        resize_request: None.into(),
    });

    let webview = WebViewBuilder::new(servo_test.servo())
        .delegate(delegate.clone())
        .url(
            Url::parse(
                "data:text/html,<!DOCTYPE html><script>\
                    let popup = window.open('about:blank');\
                    popup.resizeTo(-100, -100);\
                </script></body>",
            )
            .unwrap(),
        )
        .build();

    let load_webview = webview.clone();
    let _ = servo_test.spin(move || Ok(load_webview.load_status() != LoadStatus::Complete));

    let popup = delegate
        .popup
        .borrow()
        .clone()
        .expect("Should have created popup");

    let load_webview = popup.clone();
    let _ = servo_test.spin(move || Ok(load_webview.load_status() != LoadStatus::Complete));

    // Resize requests should be floored to 1.
    ensure!(delegate.resize_request.get() == Some(DeviceIntSize::new(1, 1)));

    // Ensure that the popup WebView is released before the end of the test.
    *delegate.popup.borrow_mut() = None;

    Ok(())
}

/// This test verifies that trying to set the WebView size to a negative value does
/// not crash Servo.
fn test_resize_webview_zero(servo_test: &ServoTest) -> Result<(), anyhow::Error> {
    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo_test.servo())
        .delegate(delegate.clone())
        .url(
            Url::parse(
                "data:text/html,<!DOCTYPE html><style> html { cursor: crosshair; margin: 0}</style><body>hello</body>",
            )
            .unwrap(),
        )
        .build();

    webview.focus();
    webview.show(true);

    webview.move_resize(Size2D::new(-100.0, -100.0).into());
    webview.resize(PhysicalSize::new(0, 0));

    let load_webview = webview.clone();
    let _ = servo_test.spin(move || Ok(load_webview.load_status() != LoadStatus::Complete));

    Ok(())
}

fn test_page_zoom(servo_test: &ServoTest) -> Result<(), anyhow::Error> {
    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo_test.servo())
        .delegate(delegate.clone())
        .build();

    // Default zoom should be 1.0
    ensure!(webview.page_zoom() == 1.0);

    webview.set_page_zoom(1.5);
    ensure!(webview.page_zoom() == 1.5);

    webview.set_page_zoom(0.5);
    ensure!(webview.page_zoom() == 0.5);

    // Should clamp to minimum
    webview.set_page_zoom(-1.0);
    ensure!(webview.page_zoom() == 0.1);

    // Should clamp to maximum
    webview.set_page_zoom(100.0);
    ensure!(webview.page_zoom() == 10.0);

    Ok(())
}

fn main() {
    run_api_tests!(
        test_create_webview,
        test_cursor_change,
        test_evaluate_javascript_basic,
        test_evaluate_javascript_panic,
        test_theme_change,
        test_negative_resize_to_request,
        test_resize_webview_zero,
        test_page_zoom,
        // This test needs to be last, as it tests creating and dropping
        // a WebView right before shutdown.
        test_create_webview_and_immediately_drop_webview_before_shutdown
    );
}
