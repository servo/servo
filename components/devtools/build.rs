/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::path::Path;
use std::{env, fs};

use chrono::Local;

fn main() {
    let path = Path::new(&env::var_os("OUT_DIR").unwrap()).join("build_id.rs");
    fs::write(
        path,
        format!(
            "const BUILD_ID: &str = \"{}\";",
            Local::now().format("%Y%m%d%H%M%S")
        ),
    )
    .unwrap();
}
