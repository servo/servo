/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Crash handling API unit tests.
mod common;

use std::panic::PanicHookInfo;
use std::rc::Rc;

use dpi::PhysicalSize;
use euclid::default::Point2D;
use log::error;
use servo::{DeviceIntRect, DevicePoint, Opts, WebViewBuilder};
use servo_config::prefs::Preferences;
use url::Url;

use crate::common::{
    ServoTest, WebViewDelegateImpl, click_at_point,
    show_webview_and_wait_for_rendering_to_be_ready, wait_for_webview_scene_to_be_up_to_date,
};

#[test]
fn test_crash_in_webview() {
    let servo_test = ServoTest::new_with_builder(|builder| {
        let mut preferences = Preferences::default();
        preferences.dom_servo_helpers_enabled = true;
        let mut opts = Opts::default();
        opts.hard_fail = false;
        builder.preferences(preferences).opts(opts)
    });

    servo_test.servo().setup_logging();

    // This is necessary for the panic to be communicated to different parts of Servo.
    std::panic::set_hook(Box::new(|info: &PanicHookInfo| {
        let panic_message = match info.payload().downcast_ref::<&'static str>() {
            Some(string) => *string,
            None => match info.payload().downcast_ref::<String>() {
                Some(string) => &**string,
                None => "Box<Any>",
            },
        };
        error!("{panic_message}");
    }));

    let url = Url::parse(
        "data:text/html,<button style='width: 500px; height: 500px' \
         onclick='ServoTestUtils.panic()'>crash</button>",
    )
    .unwrap();

    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .url(url)
        .delegate(delegate.clone())
        .build();

    show_webview_and_wait_for_rendering_to_be_ready(&servo_test, &webview, &delegate);
    click_at_point(&webview, DevicePoint::new(100.0, 100.0));

    let delegate_clone = delegate.clone();
    servo_test.spin(move || delegate_clone.crashes.get() == 0);

    assert_eq!(
        delegate.crashes.get(),
        1,
        "A forced crash triggers only a single crash notification"
    );

    webview.resize(PhysicalSize::new(300, 300));
    wait_for_webview_scene_to_be_up_to_date(&servo_test, &webview);

    let image = webview
        .rendering_context()
        .read_to_image(DeviceIntRect::new(
            Point2D::origin().cast_unit(),
            Point2D::new(300, 300).cast_unit(),
        ))
        .expect("Should be able to read image");

    let first_pixel = image.pixels().next().expect("Should always have pixels");
    assert!(
        image.pixels().any(|pixel| pixel != first_pixel),
        "The image should not be a solid color"
    )
}
