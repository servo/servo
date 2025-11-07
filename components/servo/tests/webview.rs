/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! WebView API unit tests.
mod common;

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use dpi::PhysicalSize;
use euclid::{Point2D, Size2D};
use servo::{
    ContextMenuAction, ContextMenuItem, Cursor, EmbedderControl, InputEvent, InputMethodType,
    JSValue, JavaScriptEvaluationError, LoadStatus, MouseButton, MouseButtonAction,
    MouseButtonEvent, MouseLeftViewportEvent, MouseMoveEvent, Servo, SimpleDialog, Theme, WebView,
    WebViewBuilder, WebViewDelegate,
};
use servo_config::prefs::Preferences;
use url::Url;
use webrender_api::units::{DeviceIntSize, DevicePoint};

use crate::common::{ServoTest, WebViewDelegateImpl, evaluate_javascript};

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

#[test]
fn test_create_webview() {
    let servo_test = ServoTest::new();
    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo_test.servo())
        .delegate(delegate.clone())
        .build();

    servo_test.spin(move || !delegate.url_changed.get());

    let url = webview.url();
    assert!(url.is_some());
    assert_eq!(url.unwrap().to_string(), "about:blank");
}

#[test]
fn test_evaluate_javascript_basic() {
    let servo_test = ServoTest::new();
    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo_test.servo())
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
    let webview = WebViewBuilder::new(servo_test.servo())
        .delegate(delegate.clone())
        .build();

    let input = "location";
    let result = evaluate_javascript(&servo_test, webview.clone(), input);
    assert!(matches!(result, Ok(JSValue::Object(..))));
}

#[test]
fn test_create_webview_and_immediately_drop_webview_before_shutdown() {
    let servo_test = ServoTest::new();
    WebViewBuilder::new(servo_test.servo()).build();
}

#[test]
fn test_theme_change() {
    let servo_test = ServoTest::new();
    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo_test.servo())
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
    let webview = WebViewBuilder::new(servo_test.servo())
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
    let _ = servo_test.spin(move || load_webview.load_status() != LoadStatus::Complete);

    // Wait for at least one frame after the load completes.
    delegate.reset();
    let captured_delegate = delegate.clone();
    servo_test.spin(move || !captured_delegate.new_frame_ready.get());

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

/// A test that ensure that negative resize requests do not get passed to the embedder.
#[test]
fn test_negative_resize_to_request() {
    let servo_test = ServoTest::new();
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
    let _ = servo_test.spin(move || load_webview.load_status() != LoadStatus::Complete);

    // Reset the WebView size for other tests.
    webview.resize(PhysicalSize::new(500, 500));
}

#[test]
fn test_control_show_and_hide() {
    let servo_test = ServoTest::new();
    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo_test.servo())
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

    webview.focus();
    webview.show(true);
    webview.move_resize(servo_test.rendering_context.size2d().to_f32().into());

    let load_webview = webview.clone();
    let _ = servo_test.spin(move || load_webview.load_status() != LoadStatus::Complete);

    // Wait for at least one frame after the load completes.
    delegate.reset();
    let captured_delegate = delegate.clone();
    servo_test.spin(move || !captured_delegate.new_frame_ready.get());

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
    let webview = WebViewBuilder::new(servo_test.servo())
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
    let webview = WebViewBuilder::new(servo_test.servo())
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

    webview.focus();
    webview.show(true);
    webview.move_resize(servo_test.rendering_context.size2d().to_f32().into());

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
    let webview = WebViewBuilder::new(servo_test.servo())
        .delegate(delegate.clone())
        .url(
            Url::parse(
                "data:text/html,<!DOCTYPE html> \
                <input type=\"text\" value=\"servo\" style=\"width: 200px; height: 200px;\">",
            )
            .unwrap(),
        )
        .build();

    webview.focus();
    webview.show(true);
    webview.move_resize(servo_test.rendering_context.size2d().to_f32().into());

    let load_webview = webview.clone();
    servo_test.spin(move || load_webview.load_status() != LoadStatus::Complete);

    // Wait for at least one frame after the load completes.
    delegate.reset();
    let captured_delegate = delegate.clone();
    servo_test.spin(move || !captured_delegate.new_frame_ready.get());

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
        let SimpleDialog::Alert { .. } = dialog else {
            unreachable!("Expected dialog to be a SimpleDialog::Alert");
        };
        assert_eq!(dialog.message(), "Alert");
    });
}

#[test]
fn test_prompt_dialog() {
    test_simple_dialog("window.prompt('Prompt');", |dialog| {
        let SimpleDialog::Prompt { .. } = dialog else {
            unreachable!("Expected dialog to be a SimpleDialog::Prompt");
        };
        assert_eq!(dialog.message(), "Prompt");
    });
}

#[test]
fn test_confirm_dialog() {
    test_simple_dialog("window.confirm('Confirm');", |dialog| {
        let SimpleDialog::Confirm { .. } = dialog else {
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
    let webview = WebViewBuilder::new(servo_test.servo())
        .delegate(delegate.clone())
        .url(make_test_html(prompt))
        .build();

    webview.focus();
    webview.show(true);
    webview.move_resize(servo_test.rendering_context.size2d().to_f32().into());

    let load_webview = webview.clone();
    servo_test.spin(move || load_webview.load_status() != LoadStatus::Complete);

    // Wait for at least one frame after the load completes.
    delegate.reset();
    let captured_delegate = delegate.clone();
    servo_test.spin(move || !captured_delegate.new_frame_ready.get());

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
    let webview = WebViewBuilder::new(servo_test.servo())
        .delegate(delegate.clone())
        .url(Url::parse("data:text/html,<!DOCTYPE html>").unwrap())
        .build();

    webview.focus();
    webview.show(true);
    webview.move_resize(servo_test.rendering_context.size2d().to_f32().into());

    let load_webview = webview.clone();
    servo_test.spin(move || load_webview.load_status() != LoadStatus::Complete);

    // Wait for at least one frame after the load completes.
    delegate.reset();
    let captured_delegate = delegate.clone();
    servo_test.spin(move || !captured_delegate.new_frame_ready.get());

    let point = DevicePoint::new(50.0, 50.0).into();
    webview.notify_input_event(InputEvent::MouseMove(MouseMoveEvent::new(point)));
    webview.notify_input_event(InputEvent::MouseButton(MouseButtonEvent::new(
        MouseButtonAction::Down,
        MouseButton::Right,
        point,
    )));

    // The form control should be shown.
    let captured_delegate = delegate.clone();
    servo_test.spin(move || captured_delegate.number_of_controls_shown.get() != 1);

    let context_menu = {
        let mut controls = delegate.controls_shown.borrow_mut();
        assert_eq!(controls.len(), 1);
        let Some(EmbedderControl::ContextMenu(context_menu)) = controls.pop() else {
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
