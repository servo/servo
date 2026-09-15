/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! WebView context menu API unit tests.
mod common;

use std::rc::Rc;

use servo::{
    ContextMenu, ContextMenuAction, ContextMenuElementInformation,
    ContextMenuElementInformationFlags, ContextMenuItem, EmbedderControl, JSValue, MouseButton,
    WebView, WebViewBuilder,
};
use url::Url;
use webrender_api::units::DevicePoint;

use crate::common::{
    ServoTest, WebViewDelegateImpl, click_at_point, evaluate_javascript,
    show_webview_and_wait_for_rendering_to_be_ready,
};

struct ContextMenuTest {
    servo_test: ServoTest,
    webview: WebView,
    delegate: Rc<WebViewDelegateImpl>,
}

impl ContextMenuTest {
    fn new(body: &str) -> Self {
        let servo_test = ServoTest::new();
        let delegate = Rc::new(WebViewDelegateImpl::default());
        let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
            .delegate(delegate.clone())
            .url(Url::parse(format!("data:text/html,{body}").as_str()).unwrap())
            .build();
        show_webview_and_wait_for_rendering_to_be_ready(&servo_test, &webview, &delegate);
        Self {
            servo_test,
            webview,
            delegate,
        }
    }

    fn right_click(&self, point: DevicePoint) {
        click_at_point(&self.webview, point, MouseButton::Secondary);
    }

    fn find_open_context_menu(&self) -> ContextMenu {
        assert!(self.delegate.controls_shown.borrow().is_empty());

        // Wait for a context menu to appear.
        let captured_delegate = self.delegate.clone();
        self.servo_test.spin(move || {
            let controls = captured_delegate.controls_shown.borrow();
            !controls
                .iter()
                .any(|control| matches!(control, EmbedderControl::ContextMenu(_)))
        });
        assert!(self.delegate.number_of_controls_shown.get() > 0);

        let mut controls = self.delegate.controls_shown.borrow_mut();
        let Some(index) = controls
            .iter()
            .position(|control| matches!(control, EmbedderControl::ContextMenu(_)))
        else {
            unreachable!("Expected to find context menu in controls");
        };
        let EmbedderControl::ContextMenu(context_menu) = controls.remove(index) else {
            unreachable!("Expected embedder control to be a ContextMenu");
        };
        context_menu
    }

    fn open_context_menu(&self, point: DevicePoint) -> ContextMenu {
        self.right_click(point);
        let context_menu = self.find_open_context_menu();
        self.delegate.reset();
        context_menu
    }
}

fn assert_context_menu_items(
    context_menu: &ContextMenu,
    expected_actions: &[(ContextMenuAction, bool)],
    expected_info: ContextMenuElementInformation,
) {
    let menu_actions: Vec<_> = context_menu
        .items()
        .iter()
        .filter_map(|item| match item {
            ContextMenuItem::Item {
                action, enabled, ..
            } => Some((*action, *enabled)),
            ContextMenuItem::Separator => None,
        })
        .collect();
    assert_eq!(menu_actions, expected_actions);
    assert_eq!(context_menu.element_info(), &expected_info);
}

#[test]
fn test_simple_context_menu() {
    let test = ContextMenuTest::new("<!DOCTYPE html>");
    let context_menu = test.open_context_menu(DevicePoint::new(50.0, 50.0));
    assert_context_menu_items(
        &context_menu,
        &[
            (ContextMenuAction::Cut, false),
            (ContextMenuAction::Copy, false),
            (ContextMenuAction::Paste, false),
            (ContextMenuAction::SelectAll, true),
            (ContextMenuAction::GoBack, false),
            (ContextMenuAction::GoForward, false),
            (ContextMenuAction::Reload, true),
        ],
        ContextMenuElementInformation {
            flags: ContextMenuElementInformationFlags::empty(),
            link_url: None,
            image_url: None,
        },
    );

    test.delegate.load_status_changed.set(false);
    context_menu.select(ContextMenuAction::Reload);
    test.servo_test
        .spin(move || !test.delegate.load_status_changed.get());
}

#[test]
fn test_open_context_menu_closes_existing() {
    let test = ContextMenuTest::new("<!DOCTYPE html>");

    test.right_click(DevicePoint::new(50.0, 50.0));
    let _context_menu = test.find_open_context_menu();

    assert!(test.delegate.number_of_controls_shown.get() > 0);
    assert_eq!(test.delegate.number_of_controls_hidden.get(), 0);

    test.right_click(DevicePoint::new(50.0, 50.0));
    let _context_menu = test.find_open_context_menu();

    let captured_delegate = test.delegate.clone();
    test.servo_test
        .spin(move || captured_delegate.number_of_controls_hidden.get() != 1);
    let captured_delegate = test.delegate.clone();
    test.servo_test
        .spin(move || captured_delegate.number_of_controls_shown.get() != 2);
}

#[test]
fn test_contextual_context_menu_items() {
    let test = ContextMenuTest::new(
        "<!DOCTYPE html>\
        <a href=\"https://servo.org\"><div style=\"width: 50px; height: 50px;\">Link</div></a> \
        <div><img src=\"https://servo.org/img.png\" style=\"width: 50px; height: 50px;\"></div> \
        <div style=\"width: 50px; height: 50px;\"><input type=\"text\"></div> \
        <div style=\"width: 50px; height: 50px;\"><input type=\"text\" readonly value=\"text\"></div> \
        <a href=\"https://nested.org\"><img src=\"https://servo.org/nested.png\" style=\"width: 50px; height: 50px;\"></a>",
    );

    let context_menu_on_link = test.open_context_menu(DevicePoint::new(25.0, 25.0));
    assert_context_menu_items(
        &context_menu_on_link,
        &[
            (ContextMenuAction::OpenLinkInNewWebView, true),
            (ContextMenuAction::CopyLink, true),
            (ContextMenuAction::Cut, false),
            (ContextMenuAction::Copy, false),
            (ContextMenuAction::Paste, false),
            (ContextMenuAction::SelectAll, true),
            (ContextMenuAction::GoBack, false),
            (ContextMenuAction::GoForward, false),
            (ContextMenuAction::Reload, true),
        ],
        ContextMenuElementInformation {
            flags: ContextMenuElementInformationFlags::Link,
            link_url: Url::parse("https://servo.org").ok(),
            image_url: None,
        },
    );
    context_menu_on_link.dismiss();

    let context_menu_on_image = test.open_context_menu(DevicePoint::new(25.0, 75.0));
    assert_context_menu_items(
        &context_menu_on_image,
        &[
            (ContextMenuAction::OpenImageInNewView, true),
            (ContextMenuAction::CopyImageLink, true),
            (ContextMenuAction::Cut, false),
            (ContextMenuAction::Copy, false),
            (ContextMenuAction::Paste, false),
            (ContextMenuAction::SelectAll, true),
            (ContextMenuAction::GoBack, false),
            (ContextMenuAction::GoForward, false),
            (ContextMenuAction::Reload, true),
        ],
        ContextMenuElementInformation {
            flags: ContextMenuElementInformationFlags::Image,
            link_url: None,
            image_url: Url::parse("https://servo.org/img.png").ok(),
        },
    );
    context_menu_on_image.dismiss();

    let context_menu_on_input = test.open_context_menu(DevicePoint::new(25.0, 125.0));
    assert_context_menu_items(
        &context_menu_on_input,
        &[
            (ContextMenuAction::Cut, false),
            (ContextMenuAction::Copy, false),
            (ContextMenuAction::Paste, true),
            (ContextMenuAction::SelectAll, false),
            (ContextMenuAction::GoBack, false),
            (ContextMenuAction::GoForward, false),
            (ContextMenuAction::Reload, true),
        ],
        ContextMenuElementInformation {
            flags: ContextMenuElementInformationFlags::EditableText,
            link_url: None,
            image_url: None,
        },
    );
    context_menu_on_input.dismiss();

    let context_menu_on_readonly_input = test.open_context_menu(DevicePoint::new(25.0, 175.0));
    assert_context_menu_items(
        &context_menu_on_readonly_input,
        &[
            (ContextMenuAction::Cut, false),
            (ContextMenuAction::Copy, false),
            (ContextMenuAction::Paste, false),
            (ContextMenuAction::SelectAll, true),
            (ContextMenuAction::GoBack, false),
            (ContextMenuAction::GoForward, false),
            (ContextMenuAction::Reload, true),
        ],
        ContextMenuElementInformation {
            flags: ContextMenuElementInformationFlags::empty(),
            link_url: None,
            image_url: None,
        },
    );
    context_menu_on_readonly_input.dismiss();

    let context_menu_on_image_link = test.open_context_menu(DevicePoint::new(25.0, 230.0));
    assert_context_menu_items(
        &context_menu_on_image_link,
        &[
            (ContextMenuAction::OpenLinkInNewWebView, true),
            (ContextMenuAction::CopyLink, true),
            (ContextMenuAction::OpenImageInNewView, true),
            (ContextMenuAction::CopyImageLink, true),
            (ContextMenuAction::Cut, false),
            (ContextMenuAction::Copy, false),
            (ContextMenuAction::Paste, false),
            (ContextMenuAction::SelectAll, true),
            (ContextMenuAction::GoBack, false),
            (ContextMenuAction::GoForward, false),
            (ContextMenuAction::Reload, true),
        ],
        ContextMenuElementInformation {
            flags: ContextMenuElementInformationFlags::Link |
                ContextMenuElementInformationFlags::Image,
            link_url: Url::parse("https://nested.org").ok(),
            image_url: Url::parse("https://servo.org/nested.png").ok(),
        },
    );
    context_menu_on_image_link.dismiss();
}

#[test]
fn test_contextual_context_menu_select_all() {
    let test = ContextMenuTest::new("<!DOCTYPE html>abcdef");

    let context_menu = test.open_context_menu(DevicePoint::new(25.0, 25.0));
    assert_context_menu_items(
        &context_menu,
        &[
            (ContextMenuAction::Cut, false),
            (ContextMenuAction::Copy, false),
            (ContextMenuAction::Paste, false),
            (ContextMenuAction::SelectAll, true),
            (ContextMenuAction::GoBack, false),
            (ContextMenuAction::GoForward, false),
            (ContextMenuAction::Reload, true),
        ],
        ContextMenuElementInformation {
            flags: ContextMenuElementInformationFlags::empty(),
            link_url: None,
            image_url: None,
        },
    );
    context_menu.select(ContextMenuAction::SelectAll);

    let result = evaluate_javascript(
        &test.servo_test,
        test.webview.clone(),
        "let selection = document.getSelection(); \
         selection.anchorOffset == 0 && \
         selection.anchorNode == document.body && \
         selection.focusOffset == 1 && \
         selection.focusNode == document.body",
    );
    assert_eq!(result, Ok(JSValue::Boolean(true)));
}
