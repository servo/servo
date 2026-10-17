/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Servo API unit tests.
//!
//! Since all Servo tests must run serially on the same thread, it is important
//! that tests never panic. In order to ensure this, use `anyhow::ensure!` instead
//! of `assert!` for test assertions. `ensure!` will produce a `Result::Err` in
//! place of panicking.

#[expect(dead_code)]
mod common;

use std::cell::RefCell;

use common::ServoTest;
use profile_traits::mem::MemoryReportResult;
use servo_base::generic_channel::{GenericCallback, TryReceiveError};

#[test]
fn test_simple_start_and_stop_servo() {
    // The drop implementation of `Servo` will trigger the shutdown processs and also spin the
    // event loop until shutdown is complete.
    ServoTest::new();
}

#[test]
fn test_create_memory_report() {
    let servo_test = ServoTest::new();

    let (callback, receiver) = GenericCallback::new_blocking().expect("failed to create callback");
    servo_test.servo.create_memory_report(callback);

    let callback_result: RefCell<Option<MemoryReportResult>> = RefCell::new(None);

    servo_test.spin(|| match receiver.try_recv() {
        Ok(result) => {
            *callback_result.borrow_mut() = Some(result);
            false
        },
        Err(TryReceiveError::Empty) => true,
        Err(TryReceiveError::ReceiveError(e)) => panic!("test failed: {e}"),
    });

    let result = callback_result.into_inner();

    let report = result
        .expect("must have memory report result")
        .results
        .into_iter()
        .find(|report| report.is_main_process);
    assert!(!report.expect("must have memory report").reports.is_empty());
}
