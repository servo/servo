/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! WebView tooltip API unit tests.
mod common;

use std::rc::Rc;

use servo::{
    EmbedderControl, InputEvent, MouseLeftViewportEvent, MouseMoveEvent, TooltipControl, WebView,
    WebViewBuilder, WebViewPoint,
};
use url::Url;
use webrender_api::units::DevicePoint;

use crate::common::{
    ServoTest, WebViewDelegateImpl, show_webview_and_wait_for_rendering_to_be_ready,
};

fn image(top: u32, extra_attrs: &str) -> String {
    let img_url = r"data:image/gif;base64,R0lGODlhAQABAAAAACH5BAEKAAEALAAAAAABAAEAAAICTAEAOw==\;";
    format!(
        "<img src=\"{img_url}\" {extra_attrs} \
          style=\"position:absolute;left:0;top:{top}px;width:100px;height:100px\">"
    )
}

struct TooltipTest {
    servo_test: ServoTest,
    webview: WebView,
    delegate: Rc<WebViewDelegateImpl>,
}

impl TooltipTest {
    fn new(page_body: &str) -> Self {
        let servo_test = ServoTest::new();
        let delegate = Rc::new(WebViewDelegateImpl::default());
        let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
            .delegate(delegate.clone())
            .url(Url::parse(format!("data:text/html,{page_body}").as_str()).unwrap())
            .build();

        show_webview_and_wait_for_rendering_to_be_ready(&servo_test, &webview, &delegate);

        Self {
            servo_test,
            webview,
            delegate,
        }
    }

    fn move_mouse(&self, x: f32, y: f32) {
        self.webview
            .notify_input_event(InputEvent::MouseMove(MouseMoveEvent::new(
                WebViewPoint::Device(DevicePoint::new(x, y)),
            )));
    }

    /// Wait for a tooltip to be shown, then take it out of the delegate's list.
    fn find_tooltip(&self) -> TooltipControl {
        let delegate = self.delegate.clone();
        self.servo_test.spin(move || {
            !delegate
                .controls_shown
                .borrow()
                .iter()
                .any(|c| matches!(c, EmbedderControl::Tooltip(_)))
        });

        let mut controls = self.delegate.controls_shown.borrow_mut();
        let index = controls
            .iter()
            .position(|c| matches!(c, EmbedderControl::Tooltip(_)))
            .expect("Expected a tooltip in controls");
        let EmbedderControl::Tooltip(tooltip) = controls.remove(index) else {
            unreachable!("Expected embedder control to be a Tooltip");
        };

        tooltip
    }

    fn hover_and_get_tooltip(&self, x: f32, y: f32) -> TooltipControl {
        self.move_mouse(x, y);
        self.find_tooltip()
    }
}

#[test]
fn test_alt_text_tooltip() {
    let test = TooltipTest::new(&format!("<!DOCTYPE html>{}", image(0, "alt=\"hello alt\"")));

    test.move_mouse(50., 50.);

    let tooltip = test.find_tooltip();

    assert_eq!(tooltip.text, "hello alt");
}

#[test]
fn test_alt_text_tooltip_hidden_outsize_img() {
    let test = TooltipTest::new(&format!("<!DOCTYPE html>{}", image(0, "alt=\"hello alt\"")));

    let tooltip = test.hover_and_get_tooltip(50., 50.);
    assert_eq!(tooltip.text, "hello alt");
    assert_eq!(test.delegate.number_of_controls_hidden.get(), 0);

    test.move_mouse(300., 300.); // blank area
    let delegate = test.delegate.clone();
    test.servo_test
        .spin(move || delegate.number_of_controls_hidden.get() != 1);
}

#[test]
fn test_no_tooltip_for_img_without_alt() {
    // Image without `alt`, image with empty `alt`, and a sentinel image that does have one.
    let test = TooltipTest::new(&format!(
        "<!DOCTYPE html>{}{}{}",
        image(0, ""),
        image(100, "alt=\"\""),
        image(200, "alt=\"sentinel\""),
    ));

    test.move_mouse(50., 50.); // no alt
    test.move_mouse(50., 150.); // empty alt

    // Absence is checked deterministically: the first tooltip delivered must be the sentinel's.
    let tooltip = test.hover_and_get_tooltip(50., 250.);
    assert_eq!(tooltip.text, "sentinel");
    assert_eq!(test.delegate.number_of_controls_shown.get(), 1);
}

#[test]
fn test_alt_test_changed_between_two_images() {
    // Mirrors `test_open_context_menu_closes_existing`.
    let test = TooltipTest::new(&format!(
        "<!DOCTYPE html>{}{}",
        image(0, "alt=\"first\""),
        image(200, "alt=\"second\""),
    ));

    assert_eq!(test.hover_and_get_tooltip(50., 50.).text, "first");
    assert_eq!(test.delegate.number_of_controls_hidden.get(), 0);

    assert_eq!(test.hover_and_get_tooltip(50., 250.).text, "second");
    let delegate = test.delegate.clone();
    test.servo_test
        .spin(move || delegate.number_of_controls_hidden.get() != 1);
    assert_eq!(test.delegate.number_of_controls_shown.get(), 2);
}

#[test]
fn test_tooltip_not_redisplayed_for_the_same_alt() {
    // `update_image_alt_text` returns early when the alt text is unchanged.
    let test = TooltipTest::new(&format!(
        "<!DOCTYPE html>{}{}{}",
        image(0, "alt=\"same\""),
        image(200, "alt=\"same\""),
        image(400, "alt=\"sentinel\""),
    ));

    assert_eq!(test.hover_and_get_tooltip(50., 50.).text, "same");
    test.move_mouse(50., 250.); // different element, same alt: nothing should happen
    assert_eq!(test.hover_and_get_tooltip(50., 450.).text, "sentinel");
    assert_eq!(test.delegate.number_of_controls_shown.get(), 2);
}

#[test]
fn test_tooltip_hidden_outside_viewport() {
    let test = TooltipTest::new(&format!("<!DOCTYPE html>{}", image(0, "alt=\"hello alt\"")));
    test.hover_and_get_tooltip(50., 50.);

    test.webview
        .notify_input_event(InputEvent::MouseLeftViewport(MouseLeftViewportEvent {
            focus_moving_to_another_iframe: false,
        }));
    let delegate = test.delegate.clone();
    test.servo_test
        .spin(move || delegate.number_of_controls_hidden.get() != 1);
}
