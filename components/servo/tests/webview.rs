/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! WebView API unit tests.
//!
//! Since all Servo tests must run serially on the same thread, it is important
//! that tests never panic. In order to ensure this, use `anyhow::ensure!` instead
//! of `assert!` for test assertions. `ensure!` will produce a `Result::Err` in
//! place of panicking.

mod common;

use std::cell::Cell;
use std::rc::Rc;

use anyhow::ensure;
use common::{ServoTest, run_api_tests};
use servo::{WebViewBuilder, WebViewDelegate};

#[derive(Default)]
struct WebViewDelegateImpl {
    url_changed: Cell<bool>,
}

impl WebViewDelegate for WebViewDelegateImpl {
    fn notify_url_changed(&self, _webview: servo::WebView, _url: url::Url) {
        self.url_changed.set(true);
    }
}

fn test_create_webview(servo_test: &ServoTest) -> Result<(), anyhow::Error> {
    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo_test.servo())
        .delegate(delegate.clone())
        .build();

    servo_test.spin(move || Ok(!delegate.url_changed.get()))?;

    let url = webview.url();
    ensure!(url.is_some());
    ensure!(url.unwrap().to_string() == "about:blank");

    Ok(())
}

fn test_create_webview_and_immediately_drop_webview_before_shutdown(
    servo_test: &ServoTest,
) -> Result<(), anyhow::Error> {
    WebViewBuilder::new(servo_test.servo()).build();
    Ok(())
}

fn main() {
    run_api_tests!(
        test_create_webview,
        // This test needs to be last, as it tests creating and dropping
        // a WebView right before shutdown.
        test_create_webview_and_immediately_drop_webview_before_shutdown
    );
}
