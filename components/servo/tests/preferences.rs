/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

mod common;

use std::cell::RefCell;
use std::rc::Rc;

use net::test_util::{make_body, make_server};
use servo::{
    CreateNewWebViewRequest, JSValue, LoadStatus, RenderingContext, Servo, WebView, WebViewBuilder,
    WebViewDelegate, WebViewPreferences,
};
use url::Url;

use crate::common::{ServoTest, WebViewDelegateImpl, evaluate_javascript};

/// Verify basic `WebViewPreferences` behaviour - creating a new `WebViewPreferences`
/// with different default font sizes should affect the computed style.
#[test]
fn test_webview_preferences_default_font_size() {
    let servo_test = ServoTest::new();

    // Three divs to validate different paths in layout/stylo:
    // #plain — stylo intializes computed values with default_font_size
    // #sans & #mono  — stylo uses FontMetricsProvider
    let html = "data:text/html,<!DOCTYPE html>\
                     <div id=plain>Plain</div>\
                     <div id=sans style='font-family: sans-serif'>Sans</div>\
                     <div id=mono style='font-family: monospace'>Mono</div>";
    // Create a new `WebViewPreferences` with custom font sizes.
    let preferences = Rc::new(WebViewPreferences::new(servo_test.servo()));
    preferences.set_default_font_size(32);
    preferences.set_default_monospace_font_size(18);

    // Verify the getters.
    assert_eq!(preferences.default_font_size(), 32);
    assert_eq!(preferences.default_monospace_font_size(), 18);

    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .delegate(delegate.clone())
        .preferences(preferences.clone())
        .url(Url::parse(html).unwrap())
        .build();

    let script = "[
        getComputedStyle(plain).fontSize,
        getComputedStyle(sans).fontSize,
        getComputedStyle(mono).fontSize
    ]";

    assert_eq!(
        evaluate_javascript(&servo_test, webview.clone(), script),
        Ok(JSValue::Array(vec![
            JSValue::String("32px".into()),
            JSValue::String("32px".into()),
            JSValue::String("18px".into())
        ]))
    );

    // Build a second WebView with the font sizes from the default preferences.
    let delegate2 = Rc::new(WebViewDelegateImpl::default());
    let webview2 = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .delegate(delegate2.clone())
        .url(Url::parse(html).unwrap())
        .build();

    assert_eq!(
        evaluate_javascript(&servo_test, webview2.clone(), script),
        Ok(JSValue::Array(vec![
            JSValue::String("16px".into()),
            JSValue::String("16px".into()),
            JSValue::String("13px".into())
        ]))
    );
}

/// Verify that changes to default font size preferences take effect after the
/// without a reload.
#[test]
fn test_webview_preferences_default_font_size_changes_without_reload() {
    let servo_test = ServoTest::new();

    // Use an HTTP server instead of a `data:` URL so the reload witl reuse the
    // existing script thread instead of spawning a new one.
    let (server, url) = make_server(move |_, response| {
        *response.body_mut() = make_body(b"<!DOCTYPE html>\n<div>Hello</div>".to_vec());
    });

    // Create a new `WebView` and ensure it has the default font size preference.
    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .delegate(delegate.clone())
        .url(url.as_url().clone())
        .build();

    let script = "getComputedStyle(document.querySelector('div')).fontSize";
    let result = evaluate_javascript(&servo_test, webview.clone(), script);
    assert_eq!(result, Ok(JSValue::String("16px".into())));

    // Change the font size and assert that it takes effect *without* a reload.
    let preferences = webview.preferences();
    preferences.set_default_font_size(32);

    // Reset the delegate before we wait so we don't see the stale `true` from the
    // initial load.
    delegate.reset();
    // Wait for a new frame — the preference change should trigger a restyle
    // and rendering update without needing a reload.
    servo_test.spin({
        let delegate = delegate.clone();
        move || !delegate.new_frame_ready.get()
    });

    let result = evaluate_javascript(&servo_test, webview.clone(), script);
    assert_eq!(result, Ok(JSValue::String("32px".into())));

    let _ = server.close();
}

/// Verify that an auxiliary webview can use a `WebViewPreferences` that is
/// different from that of the opener webview.
#[test]
fn test_webview_preferences_for_auxiliary_webviews() {
    let servo_test = ServoTest::new();

    struct WebViewAuxiliaryPreferencesDelegate {
        servo: Servo,
        rendering_context: Rc<dyn RenderingContext>,
        auxiliary_webview: RefCell<Option<WebView>>,
    }

    impl WebViewDelegate for WebViewAuxiliaryPreferencesDelegate {
        fn request_create_new(&self, _parent_webview: WebView, request: CreateNewWebViewRequest) {
            // Create a new `WebViewPreferences`  for the auxiliary webview with
            // a larger font size than the parent (which uses the default).
            let auxiliary_preferences = WebViewPreferences::new(&self.servo);
            auxiliary_preferences.set_default_font_size(32);

            let auxiliary_webview = request
                .builder(self.rendering_context.clone())
                .preferences(Rc::new(auxiliary_preferences))
                .build();
            self.auxiliary_webview
                .borrow_mut()
                .replace(auxiliary_webview.clone());
        }
    }

    let delegate = Rc::new(WebViewAuxiliaryPreferencesDelegate {
        servo: servo_test.servo.clone(),
        rendering_context: servo_test.rendering_context.clone(),
        auxiliary_webview: RefCell::new(None),
    });

    // Create the parent `WebView` which uses default font size preference.
    let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .delegate(delegate.clone())
        .url(
            Url::parse(
                "data:text/html,<!DOCTYPE html>\
                <div>Parent</div>\
                <script>\
                    onload = () => window.open('data:text/html,\
                        <title>Auxiliary</title><div>Child</div>')\
                </script>",
            )
            .unwrap(),
        )
        .build();

    let load_webview = webview.clone();
    let delegate_clone = delegate.clone();
    let _ = servo_test.spin(move || {
        load_webview.load_status() != LoadStatus::Complete ||
            delegate_clone
                .auxiliary_webview
                .borrow()
                .as_ref()
                .is_none_or(|auxiliary_webview| {
                    auxiliary_webview.page_title() != Some("Auxiliary".into())
                })
    });

    // The parent `WebView` should use the default font size.
    let script = "getComputedStyle(document.querySelector('div')).fontSize";
    let result = evaluate_javascript(&servo_test, webview.clone(), script);
    assert_eq!(result, Ok(JSValue::String("16px".into())));

    let auxiliary_webview = delegate
        .auxiliary_webview
        .borrow_mut()
        .take()
        .expect("Guaranteed by spin");

    // The auxiliary webview should use the font size from its own `WebViewPreferences`.
    let result = evaluate_javascript(&servo_test, auxiliary_webview.clone(), script);
    assert_eq!(result, Ok(JSValue::String("32px".into())));
}

/// Verify that mutating the default `WebViewPreferences` handle affects
/// all `WebView`s associated with the default preferences.
#[test]
fn test_webview_preferences_default_shared_between_webviews() {
    let servo_test = ServoTest::new();
    let script = "getComputedStyle(document.querySelector('div')).fontSize";

    let (server, url) = make_server(move |_, response| {
        *response.body_mut() = make_body(b"<!DOCTYPE html>\n<div>Hello</div>".to_vec());
    });

    // First webview without explicit preferences.
    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .delegate(delegate.clone())
        .url(url.as_url().clone())
        .build();

    // Verify the first webview uses default font size.
    let result = evaluate_javascript(&servo_test, webview.clone(), script);
    assert_eq!(result, Ok(JSValue::String("16px".into())));

    // Mutate the default `WebViewPreferences` through the first webview's handle and
    // check that it uses the new font size.
    webview.preferences().set_default_font_size(32);
    webview.reload();
    let result = evaluate_javascript(&servo_test, webview.clone(), script);
    assert_eq!(result, Ok(JSValue::String("32px".into())));

    // Create a second webview, also using the default preferences.
    // Use a `data:` URL so it gets its own script thread.
    let html = "data:text/html,<!DOCTYPE html><div>Hello</div>";
    let delegate2 = Rc::new(WebViewDelegateImpl::default());
    let webview2 = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .delegate(delegate2.clone())
        .url(Url::parse(html).unwrap())
        .build();

    // The second webview should also use the new font size, since it shares
    // the default `WebViewPreferences`.
    let result = evaluate_javascript(&servo_test, webview2.clone(), script);
    assert_eq!(result, Ok(JSValue::String("32px".into())));

    let _ = server.close();
}

/// Verify that a page in an iframe inherits the parent's `WebViewPreferences`.
#[test]
fn test_webview_preferences_inherited_by_iframe() {
    let servo_test = ServoTest::new();

    let iframe_html = b"<!DOCTYPE html>\n<div>Iframe</div>\n".to_vec();
    let parent_html = b"<!DOCTYPE html>\
        <div>Parent</div>\
        <iframe id=child src=/iframe.html></iframe>"
        .to_vec();

    let (server, url) = make_server(move |request, response| {
        let body = match request.uri().path() {
            "/iframe.html" => make_body(iframe_html.clone()),
            _ => make_body(parent_html.clone()),
        };
        *response.body_mut() = body;
    });

    let preferences = Rc::new(WebViewPreferences::new(servo_test.servo()));
    preferences.set_default_font_size(32);

    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .delegate(delegate.clone())
        .preferences(preferences)
        .url(url.as_url().clone())
        .build();

    // The parent and the iframe should both use the custom font size.
    assert_eq!(
        evaluate_javascript(
            &servo_test,
            webview.clone(),
            "\
            [\
                getComputedStyle(document.querySelector('div')).fontSize,\
                getComputedStyle(child.contentDocument.querySelector('div')).fontSize\
            ]"
        ),
        Ok(JSValue::Array(vec![
            JSValue::String("32px".into()),
            JSValue::String("32px".into()),
        ]))
    );

    let _ = server.close();
}
