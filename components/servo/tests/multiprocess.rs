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

use common::{ServoTest, WebViewDelegateImpl, evaluate_javascript};
use servo::{JSValue, WebViewBuilder, run_content_process};
use servo_config::{opts, prefs};

fn test_multiprocess_preference_observer() {
    let servo_test = ServoTest::new_with_builder(|builder| {
        let mut opts = opts::Opts::default();
        opts.multiprocess = true;
        builder.opts(opts)
    });

    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .delegate(delegate.clone())
        .build();

    let result = evaluate_javascript(&servo_test, webview.clone(), "window.gc");
    assert_eq!(result, Ok(JSValue::Undefined));

    let mut prefs = prefs::get().clone();
    prefs.dom_servo_helpers_enabled = true;
    prefs::set(prefs);

    delegate.load_status_changed.set(false);
    webview.reload();
    servo_test.spin(move || !delegate.load_status_changed.get());

    let result = evaluate_javascript(&servo_test, webview.clone(), "window.gc");
    assert!(matches!(result, Ok(JSValue::Object(..))));
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

    // See nextest custom test harness requirements:
    // <https://nexte.st/docs/design/custom-test-harnesses/>
    let args = std::env::args().collect::<Vec<_>>();

    // If there are no ignored tests or if the test harness doesn't support ignored tests,
    // the output MUST be empty
    if args.contains(&String::from("--ignored")) {
        return;
    }

    // Expected output:
    // ```
    // my-test-1: test
    // my-test-2: test
    // ```
    if args.contains(&String::from("--list")) {
        println!("test_multiprocess_preference_observer: test");
        return;
    }

    test_multiprocess_preference_observer();
}
