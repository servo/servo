/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::env;
use std::path::PathBuf;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let target_dir = env::var("CARGO_TARGET_DIR").unwrap();
    let mut path: PathBuf = [crate_dir.clone(), target_dir].iter().collect();
    let target = env::var("TARGET").unwrap();
    let host = env::var("HOST").unwrap();
    if target != host {
        path.push(target);
    }
    let profile_dir = env::var("PROFILE").unwrap();
    path.push(profile_dir);
    path.push("simpleservo.h");
    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_language(cbindgen::Language::C)
        .exclude_item("OutputDebugStringA")
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(path);
}
