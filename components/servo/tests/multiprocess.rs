/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Unit tests exercising Servo API functionality with multiprocess mode enabled.
//!
//! Since all Servo tests must run serially on the same thread, it is important
//! that tests never panic. In order to ensure this, use `anyhow::ensure!` instead
//! of `assert!` for test assertions. `ensure!` will produce a `Result::Err` in
//! place of panicking.

#[allow(dead_code)]
mod common;

use std::rc::Rc;

use anyhow::ensure;
use common::{ServoTest, WebViewDelegateImpl, evaluate_javascript, run_api_tests};
use servo::{JSValue, WebViewBuilder, run_content_process};
use servo_config::{opts, prefs};

fn test_multiprocess_preference_observer(servo_test: &ServoTest) -> Result<(), anyhow::Error> {
    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo_test.servo())
        .delegate(delegate.clone())
        .build();

    let result = evaluate_javascript(servo_test, webview.clone(), "window.gc");
    ensure!(matches!(result, Ok(JSValue::Undefined)));

    let mut prefs = prefs::get().clone();
    prefs.dom_servo_helpers_enabled = true;
    prefs::set(prefs);

    webview.reload();

    let result = evaluate_javascript(servo_test, webview.clone(), "window.gc");
    ensure!(matches!(result, Ok(JSValue::Object(..))));

    Ok(())
}

fn main() {
    let mut token = None;
    let mut take_next = false;
    for arg in std::env::args() {
        if take_next {
            token = Some(arg);
            break;
        }
        if arg == "--content-process" {
            take_next = true;
        }
    }

    if let Some(token) = token {
        return run_content_process(token);
    }

    run_api_tests!(
        setup: |builder| {
            let mut opts = opts::Opts::default();
            opts.multiprocess = true;
            builder.opts(opts)
        },
        test_multiprocess_preference_observer
    );
}
