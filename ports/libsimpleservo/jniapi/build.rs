/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::env;
use std::fs::File;
use std::io::Write;

fn main() {
    // Get the output directory.
    let out_dir =
        env::var("OUT_DIR").expect("Cargo should have set the OUT_DIR environment variable");

    // FIXME: we need this workaround since jemalloc-sys still links
    // to libgcc instead of libunwind, but Android NDK 23c and above
    // don't have libgcc. Alternatively, we could disable jemalloc
    // for android in servo_allocator
    let mut libgcc = File::create(out_dir.clone() + "/libgcc.a").unwrap();
    libgcc.write_all(b"INPUT(-lunwind)").unwrap();

    println!("cargo:rustc-link-search=native={}", out_dir);
}
