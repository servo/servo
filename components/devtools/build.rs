/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::path::Path;
use std::{env, fs};

use chrono::Local;

fn main() {
    println!("cargo:rerun-if-env-changed=SOURCE_DATE_EPOCH");
    let path = Path::new(&env::var_os("OUT_DIR").unwrap()).join("build_id.rs");
    let timestamp = env::var_os("SOURCE_DATE_EPOCH")
        .map(|s| {
            s.to_str()
                // if SOURCE_DATE_EPOCH is set, but not a valid str fail loudly
                // instead of falling back.
                .expect("SOURCE_DATE_EPOCH must be valid utf-8")
                .to_owned()
        })
        .unwrap_or_else(|| Local::now().format("%Y%m%d%H%M%S").to_string());
    fs::write(path, format!("const BUILD_ID: &str = \"{timestamp}\";",)).unwrap();
}
