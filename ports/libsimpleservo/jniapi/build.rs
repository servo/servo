/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::env;
use std::path::Path;

fn main() {
    // Get the NDK path from NDK_HOME env.
    let ndk_path =
        env::var_os("ANDROID_NDK").expect("Please set the ANDROID_NDK environment variable");
    let ndk_path = Path::new(&ndk_path);

    // compiling android_native_app_glue.c
    let c_file = ndk_path
        .join("sources")
        .join("android")
        .join("native_app_glue")
        .join("android_native_app_glue.c");
    cc::Build::new()
        .file(c_file)
        .warnings(false)
        .compile("android_native_app_glue");

    // Get the output directory.
    let out_dir =
        env::var("OUT_DIR").expect("Cargo should have set the OUT_DIR environment variable");

    println!("cargo:rustc-link-lib=static=android_native_app_glue");
    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=log");
    println!("cargo:rustc-link-lib=android");
}
