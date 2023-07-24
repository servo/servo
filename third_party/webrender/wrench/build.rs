/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let target = env::var("TARGET").unwrap();
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let out_dir = PathBuf::from(out_dir);

    println!("cargo:rerun-if-changed=res/wrench.exe.manifest");
    if target.contains("windows") {
        let src = PathBuf::from("res/wrench.exe.manifest");
        let mut dst = out_dir
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_owned();
        dst.push("wrench.exe.manifest");
        fs::copy(&src, &dst).unwrap();
    }
}
