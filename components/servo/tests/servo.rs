/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Servo API unit tests.
//!
//! Since all Servo tests must run serially on the same thread, it is important
//! that tests never panic. In order to ensure this, use `anyhow::ensure!` instead
//! of `assert!` for test assertions. `ensure!` will produce a `Result::Err` in
//! place of panicking.

mod common;

use anyhow::ensure;
use common::{ServoTest, run_api_tests};

fn test_simple_servo_is_not_animating_by_default(
    servo_test: &ServoTest,
) -> Result<(), anyhow::Error> {
    ensure!(!servo_test.servo().animating());
    Ok(())
}

fn main() {
    run_api_tests!(test_simple_servo_is_not_animating_by_default);
}
