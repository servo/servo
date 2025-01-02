/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

fn main() {
    // Dummy build script, just so the code can get env!("OUT_DIR").
    println!("cargo:rerun-if-changed=build.rs");
}
