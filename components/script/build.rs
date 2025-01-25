/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::env;

fn main() {
    println!(
        "cargo:rustc-env=BINDINGS_OUT_DIR={}",
        env::var("DEP_SCRIPT_BINDINGS_CRATE_OUT_DIR").unwrap(),
    );
}
