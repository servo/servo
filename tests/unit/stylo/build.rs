/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::env;
use std::fs;
use std::io::Write;
use std::path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=check_bindings.py");
    println!("cargo:rerun-if-changed=../../../ports/geckolib/glue.rs");
    println!("cargo:rerun-if-changed=../../../components/style/gecko_bindings/bindings.rs");
    assert!(Command::new("python").arg("./check_bindings.py")
                                  .spawn().unwrap().wait().unwrap().success());

    // https://github.com/rust-lang/cargo/issues/3544
    let style_out_dir = env::var_os("DEP_FOR SOME REASON THE LINKS KEY IS REQUIRED \
                                     TO PASS DATA AROUND BETWEEN BUILD SCRIPTS_OUT_DIR").unwrap();
    fs::File::create(path::PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("bindings.rs"))
        .unwrap()
        .write_all(format!("include!(concat!({:?}, \"/gecko/structs_debug.rs\"));",
                           style_out_dir).as_bytes())
        .unwrap();
}
