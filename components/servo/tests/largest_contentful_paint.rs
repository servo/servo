/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Largest Contententful Paint JS API unit tests.
mod common;

use std::rc::Rc;

use euclid::Point2D;
use servo::{InputEvent, JSValue, MouseMoveEvent, WebViewBuilder};
use servo_config::prefs::Preferences;
use url::Url;
use webrender_api::units::DevicePoint;

use crate::common::{
    ServoTest, WebViewDelegateImpl, click_at_point, evaluate_javascript,
    show_webview_and_wait_for_rendering_to_be_ready,
};

// Page with a single 50x50 red square image using a data URL.
static DATA_URL_FOR_PAGE_WITH_SINGLE_RED_SQUARE: &str = "data:text/html,<!DOCTYPE html>\
<div><img src='data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAIAAAACCAYAAABytg0kAAAAEklEQVQIW2P8z8AARAwMjDAGACwBA/+8RVWvAAAAAElFTkSuQmCC'\
style='width: 50px; height: 50px;'></div>";

// Observer script that buffers all largest-contentful-paint entries
// into `window.lcpEntries`.
static OBSERVER_SCRIPT: &str = "
    window.lcpEntries = [];
    new PerformanceObserver(list => {
        window.lcpEntries.push(...list.getEntries());
    }).observe({type: 'largest-contentful-paint', buffered: true});
";

// Script that appends a 100x100 image and sets `window.image2Done` once it has
// been loaded and painted.
static APPEND_LARGER_IMAGE_SCRIPT: &str = r#"
    window.image2Done = false;
    (async () => {
        const img = document.createElement('img');
        img.id = 'image2';
        img.style.width = '100px';
        img.style.height = '100px';
        img.src = 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAoAAAAKCAYAAACNMs+9AAAAFElEQVR4nGP4z8DwnxjMMKqQvgoBksPHOas6/LEAAAAASUVORK5CYII=';
        document.body.appendChild(img);
        await new Promise(resolve => { img.addEventListener('load', resolve); });
        await new Promise(resolve => requestAnimationFrame(() => resolve()));
        window.image2Done = true;
    })();
"#;

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
        .url(Url::parse(DATA_URL_FOR_PAGE_WITH_SINGLE_RED_SQUARE).unwrap())
        .build();

    // Wait for the page to load and render before evaluating the LCP to ensure we don't miss LCP candidate.
    show_webview_and_wait_for_rendering_to_be_ready(&servo_test, &webview, &delegate);

    if let Err(err) = evaluate_javascript(&servo_test, webview.clone(), OBSERVER_SCRIPT) {
        panic!("Failed to evaluate LCP observer script: {:?}", err);
    }

    // The single image should produce exactly one LCP entry.
    let count = evaluate_javascript(&servo_test, webview.clone(), "window.lcpEntries.length;");
    assert_eq!(count, Ok(JSValue::Number(1.0)));

    // Check the entry's fields.
    let lcp = evaluate_javascript(
        &servo_test,
        webview.clone(),
        "window.lcpEntries[0].toJSON();",
    );
    if let Ok(JSValue::Object(obj)) = lcp {
        assert_eq!(
            obj.get("name"),
            Some(JSValue::String(String::new())).as_ref()
        );
        assert_eq!(obj.get("duration"), Some(JSValue::Number(0.0)).as_ref());
        assert_eq!(
            obj.get("entryType"),
            Some(JSValue::String("largest-contentful-paint".into())).as_ref()
        );
        assert_eq!(obj.get("size"), Some(JSValue::Number(4.0)).as_ref());
        assert!(obj.get("renderTime").is_some());
        assert!(obj.get("loadTime").is_some());
    } else {
        panic!("No entries for Largest Contentful Paint were recorded.");
    }
}

#[test]
fn test_largest_contentful_paint_js_api_with_mouse_move() {
    let servo_test = ServoTest::new_with_builder(|builder| {
        let mut preferences = Preferences::default();
        preferences.largest_contentful_paint_enabled = true;
        builder.preferences(preferences)
    });

    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .delegate(delegate.clone())
        .url(Url::parse(DATA_URL_FOR_PAGE_WITH_SINGLE_RED_SQUARE).unwrap())
        .build();

    show_webview_and_wait_for_rendering_to_be_ready(&servo_test, &webview, &delegate);

    if let Err(err) = evaluate_javascript(&servo_test, webview.clone(), OBSERVER_SCRIPT) {
        panic!("Failed to evaluate LCP observer script: {:?}", err);
    }

    // The initial image should produce exactly one LCP entry.
    let count = evaluate_javascript(&servo_test, webview.clone(), "window.lcpEntries.length;");
    assert_eq!(count, Ok(JSValue::Number(1.0)));

    // A mouse move is not an activation-triggering input event, so it should not
    // halt LCP calculation.
    webview.notify_input_event(InputEvent::MouseMove(MouseMoveEvent::new(
        DevicePoint::new(10., 10.).into(),
    )));

    // Append a larger image
    if let Err(err) = evaluate_javascript(&servo_test, webview.clone(), APPEND_LARGER_IMAGE_SCRIPT)
    {
        panic!("Failed to evaluate append image script: {:?}", err);
    }

    // Wait for the larger image to load and a rendering update to happen.
    loop {
        if evaluate_javascript(&servo_test, webview.clone(), "window.image2Done === true;") ==
            Ok(JSValue::Boolean(true))
        {
            break;
        }
    }

    // Wait for the larger image's LCP entry to be reported.
    loop {
        let count = evaluate_javascript(&servo_test, webview.clone(), "window.lcpEntries.length;");
        if count == Ok(JSValue::Number(2.0)) {
            break;
        }
    }
}

#[test]
fn test_largest_contentful_paint_js_api_with_mouse_click_and_reload() {
    let servo_test = ServoTest::new_with_builder(|builder| {
        let mut preferences = Preferences::default();
        preferences.largest_contentful_paint_enabled = true;
        builder.preferences(preferences)
    });

    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .delegate(delegate.clone())
        .url(Url::parse(DATA_URL_FOR_PAGE_WITH_SINGLE_RED_SQUARE).unwrap())
        .build();

    show_webview_and_wait_for_rendering_to_be_ready(&servo_test, &webview, &delegate);

    // Observe all largest-contentful-paint entries (buffered).
    if let Err(err) = evaluate_javascript(&servo_test, webview.clone(), OBSERVER_SCRIPT) {
        panic!("Failed to evaluate LCP observer script: {:?}", err);
    }

    // The initial image should produce exactly one LCP entry.
    let count = evaluate_javascript(&servo_test, webview.clone(), "window.lcpEntries.length;");
    assert_eq!(count, Ok(JSValue::Number(1.0)));

    // Simulate a click, which should halt LCP calculation.
    click_at_point(&webview, Point2D::new(1., 1.));

    // Append a larger image; it should not be reported because LCP is halted.
    if let Err(err) = evaluate_javascript(&servo_test, webview.clone(), APPEND_LARGER_IMAGE_SCRIPT)
    {
        panic!("Failed to evaluate append image script: {:?}", err);
    }

    // Wait for the larger image to load and a rendering update to happen.
    loop {
        if evaluate_javascript(&servo_test, webview.clone(), "window.image2Done === true;") ==
            Ok(JSValue::Boolean(true))
        {
            break;
        }
    }

    // The LCP entry count should still be 1: the larger image was not reported.
    let count = evaluate_javascript(&servo_test, webview.clone(), "window.lcpEntries.length;");
    assert_eq!(count, Ok(JSValue::Number(1.0)));

    // Reloading the WebView should re-enable LCP reporting.
    webview.reload();
    show_webview_and_wait_for_rendering_to_be_ready(&servo_test, &webview, &delegate);

    if let Err(err) = evaluate_javascript(&servo_test, webview.clone(), OBSERVER_SCRIPT) {
        panic!("Failed to evaluate LCP observer script: {:?}", err);
    }

    // After reload, it should produce exactly one LCP entry.
    let count = evaluate_javascript(&servo_test, webview.clone(), "window.lcpEntries.length;");
    assert_eq!(count, Ok(JSValue::Number(1.0)));
}
