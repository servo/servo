/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Largest Contententful Paint Calculator API unit tests.
mod common;

use std::rc::Rc;

use euclid::Point2D;
use servo::WebViewBuilder;
use servo_config::prefs::Preferences;
use url::Url;

use crate::common::{
    ServoTest, WebViewDelegateImpl, click_at_point, show_webview_and_wait_for_rendering_to_be_ready,
};

#[test]
fn test_lcp_calculation_enabled_for_webview() {
    let servo_test = ServoTest::new_with_builder(|builder| {
        let mut preferences = Preferences::default();
        preferences.largest_contentful_paint_enabled = true;
        builder.preferences(preferences)
    });

    let page_1_url = Url::parse("data:text/html,<!DOCTYPE html> page 1").unwrap();
    let page_2_url = Url::parse("data:text/html,<!DOCTYPE html> page 2").unwrap();

    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .delegate(delegate.clone())
        .url(page_1_url)
        .build();

    show_webview_and_wait_for_rendering_to_be_ready(&servo_test, &webview, &delegate);

    // Initially, LCP calculation should be enabled for the WebView.
    assert_eq!(webview.lcp_calculation_enabled(), true);

    // Simulate a user interaction to disable LCP calculation for the WebView.
    click_at_point(&webview, Point2D::new(1., 1.));
    assert_eq!(webview.lcp_calculation_enabled(), false);

    // Reloading the WebView should re-enable LCP calculation.
    webview.reload();
    show_webview_and_wait_for_rendering_to_be_ready(&servo_test, &webview, &delegate);
    assert_eq!(webview.lcp_calculation_enabled(), true);

    // Simulate another user interaction to disable LCP calculation again.
    click_at_point(&webview, Point2D::new(1., 1.));
    assert_eq!(webview.lcp_calculation_enabled(), false);

    // Load a different page in the WebView. This should re-enable LCP calculation.
    let load_webview = webview.clone();
    webview.load(page_2_url.clone());
    servo_test.spin(move || load_webview.url() != Some(page_2_url.clone()));
    assert_eq!(webview.lcp_calculation_enabled(), true);
}
