/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Paint Timing JS API unit tests.
mod common;

use std::rc::Rc;

use servo::{JSValue, WebViewBuilder};
use url::Url;

use crate::common::{
    ServoTest, WebViewDelegateImpl, evaluate_javascript,
    show_webview_and_wait_for_rendering_to_be_ready,
};

#[test]
fn test_paint_timing_js_api() {
    let servo_test = ServoTest::new();

    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .delegate(delegate.clone())
        .url(
            Url::parse(
                "data:text/html,<!DOCTYPE html>\
                <a href=\"https://servo.org\"><div style=\"width: 50px; height: 50px;\">Link</div></a> \
                <div><img src=\"data:image/svg+xml,<svg width='50' height='50'><circle cx='25' cy='25' r='20' fill='green'/></svg>\"\
                style=\"width: 50px; height: 50px;\"></div>"
            )
            .unwrap(),
        )
        .build();

    show_webview_and_wait_for_rendering_to_be_ready(&servo_test, &webview, &delegate);

    let paint_entries = evaluate_javascript(
        &servo_test,
        webview.clone(),
        "performance.getEntriesByType('paint').length;",
    );
    assert_eq!(paint_entries, Ok(JSValue::Number(2.0)));

    let first_paint = evaluate_javascript(
        &servo_test,
        webview.clone(),
        "performance.getEntriesByName('first-paint')[0].toJSON();",
    );

    if let Ok(JSValue::Object(obj)) = first_paint {
        assert_eq!(
            obj.get("name"),
            Some(JSValue::String("first-paint".to_string())).as_ref()
        );
        assert_eq!(
            obj.get("entryType"),
            Some(JSValue::String("paint".to_string())).as_ref()
        );
        assert!(obj.get("startTime").is_some());
        assert_eq!(obj.get("duration"), Some(JSValue::Number(0.0)).as_ref());
    } else {
        panic!("first-paint entry is not an object");
    }

    let first_contentful_paint = evaluate_javascript(
        &servo_test,
        webview.clone(),
        "performance.getEntriesByName('first-contentful-paint')[0].toJSON();",
    );

    if let Ok(JSValue::Object(obj)) = first_contentful_paint {
        assert_eq!(
            obj.get("name"),
            Some(JSValue::String("first-contentful-paint".to_string())).as_ref()
        );
        assert_eq!(
            obj.get("entryType"),
            Some(JSValue::String("paint".to_string())).as_ref()
        );
        assert!(obj.get("startTime").is_some());
        assert_eq!(obj.get("duration"), Some(JSValue::Number(0.0)).as_ref());
    } else {
        panic!("first-contentful-paint entry is not an object");
    }
}
