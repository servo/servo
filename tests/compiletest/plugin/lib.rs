/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate compiletest_helper;
#[macro_use]
extern crate deny_public_fields;

#[test]
fn compile_test() {
    compiletest_helper::run_mode("compile-fail");
}
