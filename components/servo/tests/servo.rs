/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

mod common;

use common::*;

#[test]
fn test_simple_servo_start_and_stop() {
    let shared_test = ServoTest::new();
    assert!(!shared_test.servo().animating());
}
