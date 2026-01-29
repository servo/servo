/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Largest Contententful Paint JS API unit tests.
mod common;

use std::rc::Rc;

use euclid::Point2D;
use servo::{JSValue, WebViewBuilder};
use servo_config::prefs::Preferences;
use url::Url;

use crate::common::{
    ServoTest, WebViewDelegateImpl, click_at_point, evaluate_javascript,
    show_webview_and_wait_for_rendering_to_be_ready,
};

#[test]
fn test_largest_contentful_paint_js_api() {
    let servo_test = ServoTest::new_with_builder(|builder| {
        let mut preferences = Preferences::default();
        preferences.largest_contentful_paint_enabled = true;
        builder.preferences(preferences)
    });

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

    let lcp_script = "(async () => {
        window.lcp = await new Promise(resolve => {
            (new PerformanceObserver(entryList => {
                resolve(entryList.getEntries()[0]);
            }))
            .observe({type: 'largest-contentful-paint', buffered: true});
        })
    })();";

    if let Err(err) = evaluate_javascript(&servo_test, webview.clone(), lcp_script) {
        panic!("Failed to evaluate LCP setup script: {:?}", err);
    }

    show_webview_and_wait_for_rendering_to_be_ready(&servo_test, &webview, &delegate);

    // Read from a global variable used to store the result since evaluate_javascript doesn't handle Promises
    let lcp = evaluate_javascript(&servo_test, webview.clone(), "window.lcp.toJSON();");

    if let Ok(JSValue::Object(obj)) = lcp {
        assert_eq!(obj.get("name"), Some(JSValue::String("".into())).as_ref());
        assert_eq!(obj.get("duration"), Some(JSValue::Number(0.0)).as_ref());
        assert_eq!(
            obj.get("entryType"),
            Some(JSValue::String("largest-contentful-paint".into())).as_ref()
        );
        assert_eq!(obj.get("size"), Some(JSValue::Number(2500.0)).as_ref());
        assert!(obj.get("renderTime").is_some());
        assert!(obj.get("loadTime").is_some());
    } else {
        panic!("No entries for Largest Contentful Paint were recorded.");
    }
}

#[test]
fn test_largest_contentful_paint_js_api_with_user_interaction() {
    let servo_test = ServoTest::new_with_builder(|builder| {
        let mut preferences = Preferences::default();
        preferences.largest_contentful_paint_enabled = true;
        builder.preferences(preferences)
    });

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

    // Simulate a user interaction to disable LCP calculation for the WebView.
    click_at_point(&webview, Point2D::new(1., 1.));

    let lcp_script = "(async () => {
        window.lcp = await new Promise(resolve => {
            (new PerformanceObserver(entryList => {
                resolve(entryList.getEntries()[0]);
            }))
            .observe({type: 'largest-contentful-paint', buffered: true});
        })
    })();";

    if let Err(err) = evaluate_javascript(&servo_test, webview.clone(), lcp_script) {
        panic!("Failed to evaluate LCP setup script: {:?}", err);
    }
    show_webview_and_wait_for_rendering_to_be_ready(&servo_test, &webview, &delegate);

    // Read from a global variable used to store the result since evaluate_javascript doesn't handle Promises
    let lcp = evaluate_javascript(&servo_test, webview.clone(), "window.lcp;");
    assert_eq!(lcp, Ok(JSValue::Undefined));

    // Reloading the WebView should re-enable LCP calculation.
    webview.reload();

    if let Err(err) = evaluate_javascript(&servo_test, webview.clone(), lcp_script) {
        panic!("Failed to evaluate LCP setup script: {:?}", err);
    }
    show_webview_and_wait_for_rendering_to_be_ready(&servo_test, &webview, &delegate);

    // Read from a global variable used to store the result since evaluate_javascript doesn't handle Promises
    let lcp = evaluate_javascript(&servo_test, webview.clone(), "window.lcp;");
    assert_eq!(lcp, Ok(JSValue::Object(std::collections::HashMap::new())));
}
