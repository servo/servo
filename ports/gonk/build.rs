/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::env;
use std::process::Command;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let result = Command::new("make")
        .args(&["-f", "makefile.cargo"])
        .status()
        .unwrap();
    assert!(result.success());
    println!("cargo:rustc-flags=-L native={}", out_dir);
}
