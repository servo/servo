/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::process::Command;
use std::env;

fn main() {
    assert!(Command::new("make")
        .args(&["-f", "makefile.cargo"])
        .status()
        .unwrap()
        .success());
    println!("cargo:rustc-flags=-L native={}", env::var("OUT_DIR").unwrap());
}
