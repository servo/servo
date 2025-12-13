/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! WebView API unit tests.
mod common;

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use dpi::PhysicalSize;
use euclid::{Point2D, Size2D};
use http_body_util::combinators::BoxBody;
use hyper::body::{Bytes, Incoming};
use hyper::{Request as HyperRequest, Response as HyperResponse};
use net::test_util::{make_body, make_server};
use servo::{
    ContextMenuAction, ContextMenuElementInformation, ContextMenuElementInformationFlags,
    ContextMenuItem, Cursor, EmbedderControl, InputEvent, InputMethodType, JSValue,
    JavaScriptEvaluationError, LoadStatus, MouseButton, MouseButtonAction, MouseButtonEvent,
    MouseLeftViewportEvent, MouseMoveEvent, RenderingContext, Servo, SimpleDialog, Theme, WebView,
    WebViewBuilder, WebViewDelegate,
};
use servo_config::prefs::Preferences;
use url::Url;
use webrender_api::units::{DeviceIntSize, DevicePoint};

use crate::common::{ServoTest, WebViewDelegateImpl, evaluate_javascript};

/// Wait for the WebRender scene to reflect the current state of the WebView
/// by triggering a screenshot, waiting for it to be ready, and then throwing
/// away the results.
fn wait_for_webview_scene_to_be_up_to_date(servo_test: &ServoTest, webview: &WebView) {
    let waiting = Rc::new(Cell::new(true));
    let callback_waiting = waiting.clone();
    webview.take_screenshot(None, move |result| {
        assert!(result.is_ok());
        callback_waiting.set(false);
    });
    servo_test.spin(move || waiting.get());
}

fn show_webview_and_wait_for_rendering_to_be_ready(
    servo_test: &ServoTest,
    webview: &WebView,
    delegate: &Rc<WebViewDelegateImpl>,
) {
    let load_webview = webview.clone();
    servo_test.spin(move || load_webview.load_status() != LoadStatus::Complete);

    delegate.reset();

    // Trigger a change to the display of the document, so that we get at last one
    // new frame after load is complete.
    let _ = evaluate_javascript(
        &servo_test,
        webview.clone(),
        "requestAnimationFrame(() => { \
           document.body.style.background = 'red'; \
           document.body.style.background = 'green'; \
        });",
    );

    // Wait for at least one frame after the load completes.
    let captured_delegate = delegate.clone();
    servo_test.spin(move || !captured_delegate.new_frame_ready.get());
}

fn click_at_point(webview: &WebView, point: DevicePoint) {
    let point = point.into();
    webview.notify_input_event(InputEvent::MouseMove(MouseMoveEvent::new(point)));
    webview.notify_input_event(InputEvent::MouseButton(MouseButtonEvent::new(
        MouseButtonAction::Down,
        MouseButton::Left,
        point,
    )));
    webview.notify_input_event(InputEvent::MouseButton(MouseButtonEvent::new(
        MouseButtonAction::Up,
        MouseButton::Left,
        point,
    )));
}

fn open_context_menu_at_point(webview: &WebView, point: DevicePoint) {
    let point = point.into();
    webview.notify_input_event(InputEvent::MouseMove(MouseMoveEvent::new(point)));
    webview.notify_input_event(InputEvent::MouseButton(MouseButtonEvent::new(
        MouseButtonAction::Down,
        MouseButton::Right,
        point,
    )));
    webview.notify_input_event(InputEvent::MouseButton(MouseButtonEvent::new(
        MouseButtonAction::Up,
        MouseButton::Right,
        point,
    )));
}

#[test]
fn test_create_webview() {
    let servo_test = ServoTest::new();
    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .delegate(delegate.clone())
        .build();

    servo_test.spin(move || !delegate.url_changed.get());

    let url = webview.url();
    assert!(url.is_some());
    assert_eq!(url.unwrap().to_string(), "about:blank");
}

#[test]
fn test_create_webview_http() {
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

    let url = webview.url();
    assert!(url.is_some());
    let url = url.unwrap();
    assert_eq!(url.scheme(), "http");
    let host = url.host_str();
    assert!(host.is_some());
    assert_eq!(host.unwrap(), "localhost");
}

#[test]
fn test_evaluate_javascript_basic() {
    let servo_test = ServoTest::new();
    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .delegate(delegate.clone())
        .build();

    let result = evaluate_javascript(&servo_test, webview.clone(), "undefined");
    assert_eq!(result, Ok(JSValue::Undefined));

    let result = evaluate_javascript(&servo_test, webview.clone(), "null");
    assert_eq!(result, Ok(JSValue::Null));

    let result = evaluate_javascript(&servo_test, webview.clone(), "42");
    assert_eq!(result, Ok(JSValue::Number(42.0)));

    let result = evaluate_javascript(&servo_test, webview.clone(), "3 + 4");
    assert_eq!(result, Ok(JSValue::Number(7.0)));

    let result = evaluate_javascript(&servo_test, webview.clone(), "'abc' + 'def'");
    assert_eq!(result, Ok(JSValue::String("abcdef".into())));

    let result = evaluate_javascript(&servo_test, webview.clone(), "let foo = {blah: 123}; foo");
    assert!(matches!(result, Ok(JSValue::Object(_))));
    if let Ok(JSValue::Object(values)) = result {
        assert_eq!(values.len(), 1);
        assert_eq!(values.get("blah"), Some(&JSValue::Number(123.0)));
    }

    let result = evaluate_javascript(&servo_test, webview.clone(), "[1, 2, 3, 4]");
    let expected = JSValue::Array(vec![
        JSValue::Number(1.0),
        JSValue::Number(2.0),
        JSValue::Number(3.0),
        JSValue::Number(4.0),
    ]);
    assert_eq!(result, Ok(expected));

    let result = evaluate_javascript(&servo_test, webview.clone(), "window");
    assert!(matches!(result, Ok(JSValue::Window(..))));

    let result = evaluate_javascript(&servo_test, webview.clone(), "document.body");
    assert!(matches!(result, Ok(JSValue::Element(..))));

    let result = evaluate_javascript(
        &servo_test,
        webview.clone(),
        "document.body.attachShadow({mode: 'open'})",
    );
    assert!(matches!(result, Ok(JSValue::ShadowRoot(..))));

    let result = evaluate_javascript(&servo_test, webview.clone(), "document.body.shadowRoot");
    assert!(matches!(result, Ok(JSValue::ShadowRoot(..))));

    let result = evaluate_javascript(
        &servo_test,
        webview.clone(),
        "document.body.innerHTML += '<iframe>'; frames[0]",
    );
    assert!(matches!(result, Ok(JSValue::Frame(..))));

    let result = evaluate_javascript(&servo_test, webview.clone(), "lettt badsyntax = 123");
    assert_eq!(result, Err(JavaScriptEvaluationError::CompilationFailure));

    let result = evaluate_javascript(&servo_test, webview.clone(), "throw new Error()");
    assert!(matches!(
        result,
        Err(JavaScriptEvaluationError::EvaluationFailure(_))
    ));
}

#[test]
fn test_evaluate_javascript_panic() {
    let servo_test = ServoTest::new();
    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .delegate(delegate.clone())
        .build();

    let input = "location";
    let result = evaluate_javascript(&servo_test, webview.clone(), input);
    assert!(matches!(result, Ok(JSValue::Object(..))));
}

#[test]
fn test_create_webview_and_immediately_drop_webview_before_shutdown() {
    let servo_test = ServoTest::new();
    WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone()).build();
}

#[test]
fn test_theme_change() {
    let servo_test = ServoTest::new();
    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .delegate(delegate.clone())
        .url(Url::parse("data:text/html,page one").unwrap())
        .build();

    let is_dark_theme_script = "window.matchMedia('(prefers-color-scheme: dark)').matches";

    // The default theme is "light".
    let result = evaluate_javascript(&servo_test, webview.clone(), is_dark_theme_script);
    assert_eq!(result, Ok(JSValue::Boolean(false)));

    // Changing the theme updates the current page.
    webview.notify_theme_change(Theme::Dark);
    let result = evaluate_javascript(&servo_test, webview.clone(), is_dark_theme_script);
    assert_eq!(result, Ok(JSValue::Boolean(true)));

    delegate.reset();
    webview.load(Url::parse("data:text/html,page two").unwrap());
    servo_test.spin(move || !delegate.url_changed.get());

    // The theme persists after a navigation.
    let result = evaluate_javascript(&servo_test, webview.clone(), is_dark_theme_script);
    assert_eq!(result, Ok(JSValue::Boolean(true)));

    // Now test the same thing, but setting the theme immediately after creating the WebView.
    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .delegate(delegate.clone())
        .url(Url::parse("data:text/html,page one").unwrap())
        .build();
    webview.notify_theme_change(Theme::Dark);
    let result = evaluate_javascript(&servo_test, webview.clone(), is_dark_theme_script);
    assert_eq!(result, Ok(JSValue::Boolean(true)));
}

#[test]
fn test_cursor_change() {
    let servo_test = ServoTest::new();
    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .delegate(delegate.clone())
        .url(
            Url::parse(
                "data:text/html,<!DOCTYPE html><style> html { cursor: crosshair; margin: 0}</style><body>hello</body>",
            )
            .unwrap(),
        )
        .build();

    show_webview_and_wait_for_rendering_to_be_ready(&servo_test, &webview, &delegate);

    webview.notify_input_event(InputEvent::MouseMove(MouseMoveEvent::new(
        DevicePoint::new(10., 10.).into(),
    )));

    let captured_delegate = delegate.clone();
    servo_test.spin(move || !captured_delegate.cursor_changed.get());
    assert_eq!(webview.cursor(), Cursor::Crosshair);

    delegate.reset();
    webview.notify_input_event(InputEvent::MouseLeftViewport(
        MouseLeftViewportEvent::default(),
    ));

    let captured_delegate = delegate.clone();
    servo_test.spin(move || !captured_delegate.cursor_changed.get());
    assert_eq!(webview.cursor(), Cursor::Default);
}

// A test to ensure that the cursor doesn't change when hovering over a input with type color
#[test]
fn test_cursor_unchanged_input_color() {
    let servo_test = ServoTest::new();
    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .delegate(delegate.clone())
        .url(
            Url::parse(
                "data:text/html,<!DOCTYPE html><body><input type=\"color\"><p>Test text for Cursor change</p></body>",
            )
            .unwrap(),
        )
        .build();

    show_webview_and_wait_for_rendering_to_be_ready(&servo_test, &webview, &delegate);

    webview.notify_input_event(InputEvent::MouseMove(MouseMoveEvent::new(
        DevicePoint::new(20., 65.).into(),
    )));

    let captured_delegate = delegate.clone();
    servo_test.spin(move || !captured_delegate.cursor_changed.get());
    assert_eq!(webview.cursor(), Cursor::Text);

    delegate.reset();
    webview.notify_input_event(InputEvent::MouseMove(MouseMoveEvent::new(
        DevicePoint::new(20., 25.).into(),
    )));

    let captured_delegate = delegate.clone();
    servo_test.spin(move || !captured_delegate.cursor_changed.get());
    assert_eq!(webview.cursor(), Cursor::Default);
}

/// A test that ensure that negative resize requests do not get passed to the embedder.
#[test]
fn test_negative_resize_to_request() {
    let servo_test = ServoTest::new();
    struct WebViewResizeTestDelegate {
        servo: Servo,
        rendering_context: Rc<dyn RenderingContext>,
        popup: RefCell<Option<WebView>>,
        resize_request: Cell<Option<DeviceIntSize>>,
    }

    impl WebViewDelegate for WebViewResizeTestDelegate {
        fn request_open_auxiliary_webview(&self, parent_webview: WebView) -> Option<WebView> {
            let webview =
                WebViewBuilder::new_auxiliary(&self.servo, self.rendering_context.clone())
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
        rendering_context: servo_test.rendering_context.clone(),
        popup: None.into(),
        resize_request: None.into(),
    });

    let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
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
    let _ = servo_test.spin(move || load_webview.load_status() != LoadStatus::Complete);

    let popup = delegate
        .popup
        .borrow()
        .clone()
        .expect("Should have created popup");

    let load_webview = popup.clone();
    let _ = servo_test.spin(move || load_webview.load_status() != LoadStatus::Complete);

    // Resize requests should be floored to 1.
    assert_eq!(
        delegate.resize_request.get(),
        Some(DeviceIntSize::new(1, 1))
    );

    // Ensure that the popup WebView is released before the end of the test.
    *delegate.popup.borrow_mut() = None;
}

/// This test verifies that trying to set the WebView size to a negative value does
/// not crash Servo.
#[test]
fn test_resize_webview_zero() {
    let servo_test = ServoTest::new();
    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .delegate(delegate.clone())
        .url(Url::parse("data:text/html,<!DOCTYPE html><body>hello</body>").unwrap())
        .build();
    webview.resize(PhysicalSize::new(0, 0));

    let load_webview = webview.clone();
    let _ = servo_test.spin(move || load_webview.load_status() != LoadStatus::Complete);

    // Reset the WebView size for other tests.
    webview.resize(PhysicalSize::new(500, 500));
}

/// This test ensure's that when a `WebView` is resize, input event are handled properly in the newly
/// exposed region.
#[test]
fn test_webview_resize_interactivity() {
    let servo_test = ServoTest::new();
    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .delegate(delegate.clone())
        .url(
            Url::parse(
                "data:text/html,<!DOCTYPE html>\
                <style>\
                    html { margin: 0; }\
                    div { margin-top: 500px; width: 100px; height: 100px; cursor: crosshair; }\
                </style>\
                <body><div></div></body>",
            )
            .unwrap(),
        )
        .build();

    show_webview_and_wait_for_rendering_to_be_ready(&servo_test, &webview, &delegate);

    // Resize the WebView to expose the `<div>`.
    assert_eq!(webview.size(), Size2D::new(500., 500.));
    webview.resize(PhysicalSize::new(600, 600));

    wait_for_webview_scene_to_be_up_to_date(&servo_test, &webview);

    webview.notify_input_event(InputEvent::MouseMove(MouseMoveEvent::new(
        DevicePoint::new(20., 520.).into(),
    )));

    let captured_delegate = delegate.clone();
    servo_test.spin(move || !captured_delegate.cursor_changed.get());
    assert_eq!(webview.cursor(), Cursor::Crosshair);
}

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
                        <select id=select style=\"width: 500px; height: 500px\">\
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
    click_at_point(&webview, Point2D::new(50., 50.));

    // The form control should be shown and then immediately hidden.
    let captured_delegate = delegate.clone();
    servo_test.spin(move || captured_delegate.number_of_controls_shown.get() != 1);
    let captured_delegate = delegate.clone();
    servo_test.spin(move || captured_delegate.number_of_controls_hidden.get() != 1);
}

#[test]
fn test_page_zoom() {
    let servo_test = ServoTest::new();
    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .delegate(delegate.clone())
        .build();

    // Default zoom should be 1.0
    assert_eq!(webview.page_zoom(), 1.0);

    webview.set_page_zoom(1.5);
    assert_eq!(webview.page_zoom(), 1.5);

    webview.set_page_zoom(0.5);
    assert_eq!(webview.page_zoom(), 0.5);

    // Should clamp to minimum
    webview.set_page_zoom(-1.0);
    assert_eq!(webview.page_zoom(), 0.1);

    // Should clamp to maximum
    webview.set_page_zoom(100.0);
    assert_eq!(webview.page_zoom(), 10.0);
}

#[test]
fn test_viewport_meta_tag_initial_zoom() {
    let servo_test = ServoTest::new_with_builder(|builder| {
        let mut preferences = Preferences::default();
        preferences.viewport_meta_enabled = true;
        builder.preferences(preferences)
    });

    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .delegate(delegate.clone())
        .url(
            Url::parse(
                "data:text/html,\
                    <!DOCTYPE html>\
                    <meta name=viewport content=\"initial-scale=5\">",
            )
            .unwrap(),
        )
        .build();

    let load_webview = webview.clone();
    let _ = servo_test.spin(move || load_webview.load_status() != LoadStatus::Complete);

    // Wait for at least one frame after the load completes.
    delegate.reset();
    servo_test.spin(move || webview.page_zoom() != 5.0);
}

#[test]
fn test_show_and_hide_ime() {
    let servo_test = ServoTest::new();

    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .delegate(delegate.clone())
        .url(
            Url::parse(
                "data:text/html,<!DOCTYPE html> \
                <input type=\"text\" value=\"servo\" style=\"width: 200px; height: 200px;\">",
            )
            .unwrap(),
        )
        .build();

    show_webview_and_wait_for_rendering_to_be_ready(&servo_test, &webview, &delegate);
    click_at_point(&webview, Point2D::new(50., 50.));

    // The form control should be shown.
    let captured_delegate = delegate.clone();
    servo_test.spin(move || captured_delegate.number_of_controls_shown.get() != 1);

    {
        let controls = delegate.controls_shown.borrow();
        assert_eq!(controls.len(), 1);
        let EmbedderControl::InputMethod(ime) = &controls[0] else {
            unreachable!("Expected embedder control to be an IME");
        };

        assert_eq!(ime.input_method_type(), InputMethodType::Text);
        assert_eq!(ime.text(), "servo");
        assert_eq!(ime.insertion_point(), Some(0));
    }

    click_at_point(&webview, Point2D::new(300., 300.));

    // The form control should be hidden when the field no longer has focus.
    let captured_delegate = delegate.clone();
    servo_test.spin(move || captured_delegate.number_of_controls_hidden.get() != 1);
}

#[test]
fn test_alert_dialog() {
    test_simple_dialog("window.alert('Alert');", |dialog| {
        let SimpleDialog::Alert(..) = dialog else {
            unreachable!("Expected dialog to be a SimpleDialog::Alert");
        };
        assert_eq!(dialog.message(), "Alert");
    });
}

#[test]
fn test_prompt_dialog() {
    test_simple_dialog("window.prompt('Prompt');", |dialog| {
        let SimpleDialog::Prompt(..) = dialog else {
            unreachable!("Expected dialog to be a SimpleDialog::Prompt");
        };
        assert_eq!(dialog.message(), "Prompt");
    });
}

#[test]
fn test_confirm_dialog() {
    test_simple_dialog("window.confirm('Confirm');", |dialog| {
        let SimpleDialog::Confirm(..) = dialog else {
            unreachable!("Expected dialog to be a SimpleDialog::Confirm");
        };
        assert_eq!(dialog.message(), "Confirm");
    });
}

// A helper function to share code among the dialog tests.
//
// `prompt` must be a string that can be used in the `onclick` attribute and therefore must
// be escaped correctly. Use single quotes instead of double quotes for string literals.
fn test_simple_dialog(prompt: &str, validate: impl Fn(&SimpleDialog)) {
    let make_test_html = |prompt: &str| {
        let html = format!(
            "data:text/html,<!DOCTYPE html>\
            <input type=\"button\" value=\"click\" style=\"width: 200px; height: 200px;\"\
            onclick=\"{}\">",
            prompt
        );
        Url::parse(&html).unwrap()
    };

    let servo_test = ServoTest::new();
    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .delegate(delegate.clone())
        .url(make_test_html(prompt))
        .build();

    show_webview_and_wait_for_rendering_to_be_ready(&servo_test, &webview, &delegate);

    // The dialog should NOT be shown.
    assert!(delegate.active_dialog.borrow().is_none());

    click_at_point(&webview, Point2D::new(100., 100.));
    let captured_delegate = delegate.clone();
    servo_test.spin(move || captured_delegate.active_dialog.borrow().is_none());

    let active_dialog = delegate.active_dialog.borrow();
    validate(
        active_dialog
            .as_ref()
            .expect("the spin call above ensures this is not None"),
    );
}

#[test]
fn test_simple_context_menu() {
    let servo_test = ServoTest::new();

    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .delegate(delegate.clone())
        .url(Url::parse("data:text/html,<!DOCTYPE html>").unwrap())
        .build();

    show_webview_and_wait_for_rendering_to_be_ready(&servo_test, &webview, &delegate);
    open_context_menu_at_point(&webview, DevicePoint::new(50.0, 50.0));

    // The form control should be shown.
    let captured_delegate = delegate.clone();
    servo_test.spin(move || captured_delegate.number_of_controls_shown.get() == 0);

    let context_menu = {
        let mut controls = delegate.controls_shown.borrow_mut();

        let Some(index) = controls
            .iter()
            .position(|control| matches!(control, EmbedderControl::ContextMenu(_)))
        else {
            unreachable!("Exepcted to find context menu in controls");
        };
        let EmbedderControl::ContextMenu(context_menu) = controls.remove(index) else {
            unreachable!("Expected embedder control to be a ContextMenu");
        };

        let items = context_menu.items();
        assert!(matches!(
            items[0],
            ContextMenuItem::Item {
                action: ContextMenuAction::GoBack,
                ..
            }
        ));
        assert!(matches!(
            items[1],
            ContextMenuItem::Item {
                action: ContextMenuAction::GoForward,
                ..
            }
        ));
        assert!(matches!(
            items[2],
            ContextMenuItem::Item {
                action: ContextMenuAction::Reload,
                ..
            }
        ));

        context_menu
    };

    delegate.reset();
    context_menu.select(ContextMenuAction::Reload);

    servo_test.spin(move || !delegate.load_status_changed.get());
}

#[test]
fn test_contextual_context_menu_items() {
    let servo_test = ServoTest::new();

    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .delegate(delegate.clone())
        .url(
            Url::parse(
                "data:text/html,<!DOCTYPE html>\
                <a href=\"https://servo.org\"><div style=\"width: 50px; height: 50px;\">Link</div></a> \
                <div><img src=\"https://servo.org/img.png\" style=\"width: 50px; height: 50px;\"></div> \
                <div><input type=\"text\" style=\"width: 50px; height: 50px;\"></div> \
                <a href=\"https://nested.org\"><img src=\"https://servo.org/nested.png\" style=\"width: 50px; height: 50px;\"></a>"
            )
            .unwrap(),
        )
        .build();

    show_webview_and_wait_for_rendering_to_be_ready(&servo_test, &webview, &delegate);

    let assert_context_menu =
        |delegate: Rc<WebViewDelegateImpl>,
         expected_actions: &[ContextMenuAction],
         expected_info: ContextMenuElementInformation| {
            assert!(delegate.controls_shown.borrow().is_empty());

            // The form control should be shown.
            let captured_delegate = delegate.clone();
            servo_test.spin(move || captured_delegate.number_of_controls_shown.get() == 0);

            {
                let mut controls = delegate.controls_shown.borrow_mut();

                let Some(index) = controls
                    .iter()
                    .position(|control| matches!(control, EmbedderControl::ContextMenu(_)))
                else {
                    unreachable!("Exepcted to find context menu in controls");
                };
                let EmbedderControl::ContextMenu(context_menu) = controls.remove(index) else {
                    unreachable!("Expected embedder control to be a ContextMenu");
                };

                assert_eq!(context_menu.element_info(), &expected_info);

                let items = context_menu.items();
                for expected_action in expected_actions {
                    assert!(items.iter().any(|item| {
                        let ContextMenuItem::Item { action, .. } = item else {
                            return false;
                        };
                        action == expected_action
                    }));
                }
                context_menu.dismiss();
            }

            delegate.reset();
        };

    open_context_menu_at_point(&webview, DevicePoint::new(25.0, 25.0));
    assert_context_menu(
        delegate.clone(),
        &[
            ContextMenuAction::CopyLink,
            ContextMenuAction::OpenLinkInNewWebView,
        ],
        ContextMenuElementInformation {
            flags: ContextMenuElementInformationFlags::Link,
            link_url: Url::parse("https://servo.org").ok(),
            image_url: None,
        },
    );

    open_context_menu_at_point(&webview, DevicePoint::new(25.0, 75.0));
    assert_context_menu(
        delegate.clone(),
        &[
            ContextMenuAction::CopyImageLink,
            ContextMenuAction::OpenImageInNewView,
        ],
        ContextMenuElementInformation {
            flags: ContextMenuElementInformationFlags::Image,
            link_url: None,
            image_url: Url::parse("https://servo.org/img.png").ok(),
        },
    );

    open_context_menu_at_point(&webview, DevicePoint::new(25.0, 125.0));
    assert_context_menu(
        delegate.clone(),
        &[
            ContextMenuAction::SelectAll,
            ContextMenuAction::Cut,
            ContextMenuAction::Copy,
            ContextMenuAction::Paste,
        ],
        ContextMenuElementInformation {
            flags: ContextMenuElementInformationFlags::EditableText,
            link_url: None,
            image_url: None,
        },
    );

    open_context_menu_at_point(&webview, DevicePoint::new(25.0, 175.0));
    assert_context_menu(
        delegate.clone(),
        &[
            ContextMenuAction::CopyLink,
            ContextMenuAction::OpenLinkInNewWebView,
            ContextMenuAction::CopyImageLink,
            ContextMenuAction::OpenImageInNewView,
        ],
        ContextMenuElementInformation {
            flags: ContextMenuElementInformationFlags::Link |
                ContextMenuElementInformationFlags::Image,
            link_url: Url::parse("https://nested.org").ok(),
            image_url: Url::parse("https://servo.org/nested.png").ok(),
        },
    );

    servo_test.spin(move || !delegate.load_status_changed.get());
}

#[test]
fn test_can_go_forward_and_can_go_back() {
    let servo_test = ServoTest::new();

    let page_1_url = Url::parse("data:text/html,<!DOCTYPE html> page 1").unwrap();
    let page_2_url = Url::parse("data:text/html,<!DOCTYPE html> page 2").unwrap();

    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .delegate(delegate.clone())
        .url(page_1_url.clone())
        .build();

    show_webview_and_wait_for_rendering_to_be_ready(&servo_test, &webview, &delegate);

    assert!(!webview.can_go_forward());
    assert!(!webview.can_go_back());

    let load_webview = webview.clone();
    webview.load(page_2_url.clone());
    servo_test.spin(move || load_webview.url() != Some(page_2_url.clone()));

    assert!(!webview.can_go_forward());
    assert!(webview.can_go_back());

    webview.go_back(1);

    let load_webview = webview.clone();
    servo_test.spin(move || load_webview.url() != Some(page_1_url.clone()));

    assert!(webview.can_go_forward());
    assert!(!webview.can_go_back());
}
