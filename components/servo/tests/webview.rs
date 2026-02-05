/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! WebView API unit tests.
mod common;

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use dpi::PhysicalSize;
use euclid::{Point2D, Size2D};
use http_body_util::combinators::BoxBody;
use hyper::body::{Bytes, Incoming};
use hyper::{Request as HyperRequest, Response as HyperResponse};
use net::test_util::{make_body, make_server, replace_host_table};
use servo::user_contents::UserStyleSheet;
use servo::{
    ContextMenuAction, ContextMenuElementInformation, ContextMenuElementInformationFlags,
    ContextMenuItem, CreateNewWebViewRequest, Cursor, EmbedderControl, InputEvent, InputMethodType,
    JSValue, JavaScriptEvaluationError, LoadStatus, MouseButton, MouseButtonAction,
    MouseButtonEvent, MouseLeftViewportEvent, MouseMoveEvent, RenderingContext, Servo,
    SimpleDialog, Theme, UserContentManager, UserScript, WebView, WebViewBuilder, WebViewDelegate,
};
use servo_config::prefs::Preferences;
use servo_url::ServoUrl;
use url::Url;
use webrender_api::units::{DeviceIntSize, DevicePoint};

use crate::common::{
    ServoTest, WebViewDelegateImpl, evaluate_javascript,
    show_webview_and_wait_for_rendering_to_be_ready,
};

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
fn test_create_webview_http_custom_host() {
    let servo_test = ServoTest::new();

    static MESSAGE: &'static [u8] = b"<!DOCTYPE html>\n<title>Hello</title>";
    let handler =
        move |_: HyperRequest<Incoming>,
              response: &mut HyperResponse<BoxBody<Bytes, hyper::Error>>| {
            *response.body_mut() = make_body(MESSAGE.to_vec());
        };
    let (server, url) = make_server(handler);
    let port = url.port().unwrap();

    let ip = "127.0.0.1".parse().unwrap();
    let mut host_table = HashMap::new();
    host_table.insert("www.example.com".to_owned(), ip);

    replace_host_table(host_table);

    let custom_url = ServoUrl::parse(&format!("http://www.example.com:{}", port)).unwrap();

    let delegate = Rc::new(WebViewDelegateImpl::default());

    let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .delegate(delegate.clone())
        .url(custom_url.clone().into_url())
        .build();

    servo_test.spin(move || !delegate.load_status_changed.get());

    let _ = server.close();

    let page_title = webview.page_title();
    assert!(page_title.is_some());
    assert_eq!(page_title.unwrap(), "Hello");

    let url = webview.url();
    assert!(url.is_some());
    assert_eq!(url.unwrap(), custom_url.into_url());
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
        rendering_context: Rc<dyn RenderingContext>,
        popup: RefCell<Option<WebView>>,
        resize_request: Cell<Option<DeviceIntSize>>,
    }

    impl WebViewDelegate for WebViewResizeTestDelegate {
        fn request_create_new(&self, parent_webview: WebView, request: CreateNewWebViewRequest) {
            let webview = request
                .builder(self.rendering_context.clone())
                .delegate(parent_webview.delegate())
                .build();
            self.popup.borrow_mut().replace(webview.clone());
        }

        fn request_resize_to(&self, _: WebView, requested_outer_size: DeviceIntSize) {
            self.resize_request.set(Some(requested_outer_size));
        }
    }

    let delegate = Rc::new(WebViewResizeTestDelegate {
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
    click_at_point(&webview, Point2D::new(100., 100.));

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
        assert_eq!(ime.insertion_point(), Some(5));
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

#[test]
fn test_user_content_manager_empty() {
    let servo_test = ServoTest::new();
    let user_content_manager = UserContentManager::new(servo_test.servo());
    let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .user_content_manager(Rc::new(user_content_manager))
        .url(Url::parse("data:text/html,Hello World").unwrap())
        .build();

    let load_webview = webview.clone();
    let _ = servo_test.spin(move || load_webview.load_status() != LoadStatus::Complete);
    let result = evaluate_javascript(&servo_test, webview.clone(), "window.fromUserContentScript");
    assert_eq!(result, Ok(JSValue::Undefined));
}

#[test]
fn test_user_content_manager_user_script() {
    let servo_test = ServoTest::new();

    // Use a http server instead of a data url to allow the `webview.reload()` call below to reuse
    // the exisitng script thread. This is necessary to test that mutations on a `UserContentManager`
    // take effect on script threads created before the mutation.
    let (_, url) = make_server(move |_, response| {
        *response.body_mut() = make_body(b"<!DOCTYPE html>\nHello".to_vec());
    });

    let user_content_manager = Rc::new(UserContentManager::new(servo_test.servo()));
    user_content_manager.add_script(Rc::new("window.fromUserContentScript = 42;".into()));

    let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .user_content_manager(user_content_manager.clone())
        .url(url.into_url())
        .build();

    let load_webview = webview.clone();
    let _ = servo_test.spin(move || load_webview.load_status() != LoadStatus::Complete);
    let result = evaluate_javascript(&servo_test, webview.clone(), "window.fromUserContentScript");
    assert_eq!(result, Ok(JSValue::Number(42.0)));

    // Add a second user script to the `UserContentManager`.
    let second_user_script = Rc::new(UserScript::from("window.fromSecondUserContentScript = 32;"));
    user_content_manager.add_script(second_user_script.clone());

    // The second user script must immediately take effect in any new WebViews.
    let new_webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .user_content_manager(user_content_manager.clone())
        .url(Url::parse("data:text/html,<!DOCTYPE html>").unwrap())
        .build();
    let load_webview = new_webview.clone();
    let _ = servo_test.spin(move || load_webview.load_status() != LoadStatus::Complete);
    let result = evaluate_javascript(
        &servo_test,
        new_webview,
        "window.fromSecondUserContentScript",
    );
    assert_eq!(result, Ok(JSValue::Number(32.0)));

    // The existing page in the first webview must not be affected since we haven't reloaded yet.
    let result = evaluate_javascript(
        &servo_test,
        webview.clone(),
        "window.fromSecondUserContentScript",
    );
    assert_eq!(result, Ok(JSValue::Undefined));

    // Now trigger a reload and ensure the second user script has effect on the page.
    webview.reload();

    let load_webview = webview.clone();
    let _ = servo_test.spin(move || load_webview.load_status() != LoadStatus::Complete);
    let result = evaluate_javascript(
        &servo_test,
        webview.clone(),
        "window.fromSecondUserContentScript",
    );

    assert_eq!(result, Ok(JSValue::Number(32.0)));

    // Test that removing the user script works. Trigger a reload and ensure the second user script
    // no longer has effect on the page.
    user_content_manager.remove_script(second_user_script);
    webview.reload();

    let load_webview = webview.clone();
    let _ = servo_test.spin(move || load_webview.load_status() != LoadStatus::Complete);
    let result = evaluate_javascript(&servo_test, webview, "window.fromSecondUserContentScript");

    assert_eq!(result, Ok(JSValue::Undefined));
}

#[test]
fn test_user_content_manager_for_auxiliary_webviews() {
    let servo_test = ServoTest::new();
    struct WebViewAuxiliaryTestDelegate {
        servo: Servo,
        rendering_context: Rc<dyn RenderingContext>,
        auxiliary_webview: RefCell<Option<WebView>>,
    }

    impl WebViewDelegate for WebViewAuxiliaryTestDelegate {
        fn request_create_new(&self, _parent_webview: WebView, request: CreateNewWebViewRequest) {
            let user_content_manager_for_auxiliary_webview = UserContentManager::new(&self.servo);
            // Add a different user script to the `UserContentManager` of auxiliary webview.
            user_content_manager_for_auxiliary_webview.add_script(Rc::new(
                "window.fromAuxiliaryUserContentScript = 32;".into(),
            ));
            let auxiliary_webview = request
                .builder(self.rendering_context.clone())
                .user_content_manager(Rc::new(user_content_manager_for_auxiliary_webview))
                .build();
            self.auxiliary_webview
                .borrow_mut()
                .replace(auxiliary_webview.clone());
        }
    }

    let delegate = Rc::new(WebViewAuxiliaryTestDelegate {
        servo: servo_test.servo.clone(),
        rendering_context: servo_test.rendering_context.clone(),
        auxiliary_webview: RefCell::new(None),
    });

    let user_content_manager = UserContentManager::new(servo_test.servo());
    user_content_manager.add_script(Rc::new("window.fromUserContentScript = 42;".into()));

    let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .delegate(delegate.clone())
        .user_content_manager(Rc::new(user_content_manager))
        .url(
            Url::parse(
                "data:text/html,<!DOCTYPE html>\
                <script>\
                    onload = () => window.open('data:text/html,<title>Auxiliary WebView</title>')\
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
                    auxiliary_webview.page_title() != Some("Auxiliary WebView".into())
                })
    });

    let result = evaluate_javascript(
        &servo_test,
        webview.clone(),
        "[ window.fromUserContentScript, window.fromAuxiliaryUserContentScript ]",
    );
    assert_eq!(
        result,
        Ok(JSValue::Array(vec![
            JSValue::Number(42.0),
            JSValue::Undefined
        ]))
    );

    let auxiliary_webview = delegate
        .auxiliary_webview
        .borrow_mut()
        .take()
        .expect("Gauranteed by spin");

    let result = evaluate_javascript(
        &servo_test,
        auxiliary_webview.clone(),
        "[ window.fromUserContentScript, window.fromAuxiliaryUserContentScript ]",
    );

    assert_eq!(
        result,
        Ok(JSValue::Array(vec![
            JSValue::Undefined,
            JSValue::Number(32.0),
        ]))
    );
}

#[test]
fn test_user_content_manager_for_user_stylesheets() {
    let servo_test = ServoTest::new();

    let user_content_manager = Rc::new(UserContentManager::new(servo_test.servo()));

    #[cfg(not(target_os = "windows"))]
    let url = Url::from_file_path("/test/test.css").unwrap();
    #[cfg(target_os = "windows")]
    let url = Url::from_file_path("C:\\test\\test.css").unwrap();

    let user_stylesheet = Rc::new(UserStyleSheet::new(
        "div { width: 100px; height: 50px }\
        p { width: 200px; height: 200px }"
            .into(),
        url,
    ));
    user_content_manager.add_stylesheet(user_stylesheet.clone());

    let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .user_content_manager(user_content_manager.clone())
        .url(
            Url::parse(
                "data:text/html,<!DOCTYPE html>\
                        <style>p { width: 300px; height: 300px }</style>\
                        <div id='div1'></div><p id='p1'>test paragraph</p>",
            )
            .unwrap(),
        )
        .build();

    let load_webview = webview.clone();
    let _ = servo_test.spin(move || load_webview.load_status() != LoadStatus::Complete);

    let result = evaluate_javascript(
        &servo_test,
        webview.clone(),
        "[ div1.offsetWidth, div1.offsetHeight, p1.offsetWidth, p1.offsetHeight ]",
    );
    assert_eq!(
        result,
        Ok(JSValue::Array(vec![
            // `div` elements uses the rules from the user stylesheet since the author stylesheet doesn't
            // have any rules that match `div`s.
            JSValue::Number(100.0),
            JSValue::Number(50.0),
            // `p` element uses the rules from author stylesheet as they have precendece over user
            // rules from user stylesheets.
            JSValue::Number(300.0),
            JSValue::Number(300.0),
        ]))
    );

    // Test that removing the stylesheet works.
    user_content_manager.remove_stylesheet(user_stylesheet);
    webview.reload();

    let load_webview = webview.clone();
    let _ = servo_test.spin(move || load_webview.load_status() != LoadStatus::Complete);

    let result = evaluate_javascript(&servo_test, webview.clone(), "div1.offsetHeight");

    assert_eq!(result, Ok(JSValue::Number(0.0)));
}

#[test]
fn test_pinch_zoom_update_dom_visual_viewport() {
    let servo_test = ServoTest::new_with_builder(|builder| {
        let mut preferences = Preferences::default();
        preferences.dom_visual_viewport_enabled = true;
        builder.preferences(preferences)
    });

    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .delegate(delegate.clone())
        .url(Url::parse("data:text/html,<!DOCTYPE html><body>Hello world!</body>").unwrap())
        .build();

    show_webview_and_wait_for_rendering_to_be_ready(&servo_test, &webview, &delegate);
    let eval_visual_viewport = |attr: &str| {
        evaluate_javascript(
            &servo_test,
            webview.clone(),
            format!("window.visualViewport.{}", attr),
        )
    };

    // Default value of the DOM visual viewport is initialized correctly.
    assert_eq!(eval_visual_viewport("scale"), Ok(JSValue::Number(1.)));
    assert_eq!(eval_visual_viewport("width"), Ok(JSValue::Number(500.)));
    assert_eq!(eval_visual_viewport("height"), Ok(JSValue::Number(500.)));
    assert_eq!(eval_visual_viewport("offsetLeft"), Ok(JSValue::Number(0.)));
    assert_eq!(eval_visual_viewport("offsetTop"), Ok(JSValue::Number(0.)));

    webview.pinch_zoom(5., DevicePoint::new(100., 100.));
    wait_for_webview_scene_to_be_up_to_date(&servo_test, &webview);

    // The visual viewport dimension is correct after a pinch zoom.
    assert_eq!(eval_visual_viewport("scale"), Ok(JSValue::Number(5.)));
    assert_eq!(eval_visual_viewport("width"), Ok(JSValue::Number(100.)));
    assert_eq!(eval_visual_viewport("height"), Ok(JSValue::Number(100.)));
    assert_eq!(eval_visual_viewport("offsetLeft"), Ok(JSValue::Number(80.)));
    assert_eq!(eval_visual_viewport("offsetTop"), Ok(JSValue::Number(80.)));
}
