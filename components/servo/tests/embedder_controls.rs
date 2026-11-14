/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! WebView embedder controls API unit tests.
mod common;

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex};

use dpi::PhysicalSize;
use embedder_traits::{RefreshDriver, UrlRequest};
use euclid::default::Size2D as UntypedSize2D;
use euclid::{Point2D, Size2D, Vector2D};
use http::{HeaderMap, HeaderName, HeaderValue, StatusCode};
use http_body_util::combinators::BoxBody;
use hyper::body::{Bytes, Incoming};
use hyper::{Request as HyperRequest, Response as HyperResponse};
use image::RgbaImage;
use itertools::Itertools;
use net::test_util::{make_body, make_server, replace_host_table};
use servo::JSValue::Boolean;
use servo::profile_traits::mem::MemoryReportResult;
use servo::{
    CreateNewWebViewRequest, Cursor, DeviceIntPoint, EmbedderControl, InputEvent, InputMethodType,
    JSValue, LoadStatus, MouseButton, MouseLeftViewportEvent, MouseMoveEvent, PrefValue,
    RenderingContext, Scroll, SimpleDialog, Theme, WebView, WebViewBuilder, WebViewDelegate,
    WebViewPoint, WebViewVector,
};
use servo_base::generic_channel::GenericCallback;
use servo_config::prefs::Preferences;
use servo_url::ServoUrl;
use surfman::{Error, Surface, SurfaceTexture};
use url::Url;
use webrender_api::units::{
    DeviceIntRect, DeviceIntSize, DevicePoint, DeviceRect, DeviceVector2D, RectExt,
};

use crate::common::{
    ServoTest, WebViewDelegateImpl, click_at_point, evaluate_javascript,
    show_webview_and_wait_for_rendering_to_be_ready, wait_for_webview_scene_to_be_up_to_date,
};

#[test]
fn test_control_show_and_hide() {
    let servo_test = ServoTest::new();
    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .delegate(delegate.clone())
        .url(
            Url::parse(
                "data:text/html,\
                    <!DOCTYPE html>\
                        <select id=select style=\"position: absolute; top: 20px; left: 10px; width: 100px; height: 100px\">\
                            <option>one</option>\
                        </select>
                        <script>\
                            select.addEventListener('click', () => {\
                                setTimeout(() => select.parentNode.removeChild(select), 100);\
                            });\
                        </script>",
            )
            .unwrap(),
        )
        .build();

    show_webview_and_wait_for_rendering_to_be_ready(&servo_test, &webview, &delegate);
    click_at_point(&webview, Point2D::new(50., 50.), MouseButton::Primary);

    // The form control should be shown and then immediately hidden.
    servo_test.spin(|| delegate.number_of_controls_shown.get() != 1);
    assert_eq!(
        delegate.last_embedder_control_rect().min,
        DeviceIntPoint::new(10, 20)
    );
    servo_test.spin(|| delegate.number_of_controls_hidden.get() != 1);
}

#[test]
fn test_control_rect_with_pinch_zoom() {
    let servo_test = ServoTest::new();
    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .delegate(delegate.clone())
        .url(
            Url::parse(
                "data:text/html,\
                    <!DOCTYPE html>\
                        <input id=input style=\"position: absolute; left: 100px; top: 50px; width: 100px; height: 50px; box-sizing: border-box;\">"
            )
            .unwrap(),
        )
        .build();
    show_webview_and_wait_for_rendering_to_be_ready(&servo_test, &webview, &delegate);

    // Pinch zoom at the origin.
    webview.adjust_pinch_zoom(2.0, Default::default());
    wait_for_webview_scene_to_be_up_to_date(&servo_test, &webview);

    let _ = evaluate_javascript(
        &servo_test,
        webview.clone(),
        "document.getElementById('input').focus()",
    );

    // The form control should be shown and then immediately hidden.
    servo_test.spin(|| delegate.number_of_controls_shown.get() != 1);
    assert_eq!(
        delegate.last_embedder_control_rect().min,
        DeviceIntPoint::new(200, 100)
    );
    assert_eq!(
        delegate.last_embedder_control_rect().size(),
        DeviceIntSize::new(200, 100)
    );
}

#[test]
fn test_control_rect_with_scrolling() {
    let servo_test = ServoTest::new();
    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .delegate(delegate.clone())
        .url(
            Url::parse(
                "data:text/html,\
                    <!DOCTYPE html>\
                        <input id=input style=\"position: absolute; left: 100px; top: 200px; width: 100px; height: 100px\">
                        <div style=\"width: 10000px; height: 1000px\"></div>"
            )
            .unwrap(),
        )
        .build();
    show_webview_and_wait_for_rendering_to_be_ready(&servo_test, &webview, &delegate);

    let point = DevicePoint::new(10., 10.);
    webview.notify_scroll_event(
        Scroll::Delta(WebViewVector::Device(point.to_vector())),
        WebViewPoint::Device(point),
    );
    wait_for_webview_scene_to_be_up_to_date(&servo_test, &webview);

    let _ = evaluate_javascript(
        &servo_test,
        webview.clone(),
        "document.getElementById('input').focus()",
    );

    // The form control should be shown and then immediately hidden.
    servo_test.spin(|| delegate.number_of_controls_shown.get() != 1);
    assert_eq!(
        delegate.last_embedder_control_rect().min,
        DeviceIntPoint::new(90, 190)
    );
}

#[test]
fn test_control_rect_in_iframe() {
    let servo_test = ServoTest::new();
    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .delegate(delegate.clone())
        .url(
            Url::parse(
                "data:text/html,\
                    <!DOCTYPE html>\
                        <iframe id=frame style=\"position: absolute; translate: 20px 20px; left: 30px; top: 30px; border: 0;\" \
                        srcdoc=\"<input id=input style='position: absolute; left: 10px; top: 10px; width: 100px; height: 100px'>\">"
            )
            .unwrap(),
        )
        .build();
    show_webview_and_wait_for_rendering_to_be_ready(&servo_test, &webview, &delegate);

    let _ = evaluate_javascript(
        &servo_test,
        webview.clone(),
        "document.getElementById('frame').contentDocument.getElementById('input').focus({ preventScroll: true })",
    );

    // The form control should be shown and then immediately hidden.
    servo_test.spin(|| delegate.number_of_controls_shown.get() != 1);
    assert_eq!(
        delegate.last_embedder_control_rect().min,
        DeviceIntPoint::new(60, 60)
    );
}
