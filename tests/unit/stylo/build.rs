/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=check_bindings.py");
    println!("cargo:rerun-if-changed=../../../ports/geckolib/glue.rs");
    println!("cargo:rerun-if-changed=../../../components/style/gecko_bindings/bindings.rs");
    assert!(Command::new("python").arg("./check_bindings.py")
                                  .spawn().unwrap().wait().unwrap().success());
}
