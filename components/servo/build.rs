/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::env;
use std::path::Path;
use std::process;
use std::process::{Command, Stdio};

fn main() {
    // build.rs is not platform-specific, so we have to check the target here.
    let target = env::var("TARGET").unwrap();
    if target.contains("android") {
        android_main()
    }
}

fn android_main() {
    // Get the NDK path from NDK_HOME env.
    let ndk_path = env::var("NDK_HOME").ok().expect("Please set the NDK_HOME environment variable");
    let ndk_path = Path::new(&ndk_path);

    // Get the standalone NDK path from NDK_STANDALONE env.
    let standalone_path = env::var("NDK_STANDALONE").ok().expect("Please set the NDK_STANDALONE environment variable");
    let standalone_path = Path::new(&standalone_path);

    // Get the standalone NDK path from NDK_STANDALONE env.
    let out_dir = env::var("OUT_DIR").ok().expect("Cargo should have set the OUT_DIR environment variable");
    let directory = Path::new(&out_dir);

    // compiling android_native_app_glue.c
    if Command::new(standalone_path.join("bin").join("arm-linux-androideabi-gcc"))
        .arg(ndk_path.join("sources").join("android").join("native_app_glue").join("android_native_app_glue.c"))
        .arg("-c")
        .arg("-o").arg(directory.join("android_native_app_glue.o"))
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status().unwrap().code().unwrap() != 0
    {
        println!("Error while executing gcc");
        process::exit(1)
    }

    // compiling libandroid_native_app_glue.a
    if Command::new(standalone_path.join("bin").join("arm-linux-androideabi-ar"))
        .arg("rcs")
        .arg(directory.join("libandroid_native_app_glue.a"))
        .arg(directory.join("android_native_app_glue.o"))
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status().unwrap().code().unwrap() != 0
    {
        println!("Error while executing ar");
        process::exit(1)
    }

    println!("cargo:rustc-link-lib=static=android_native_app_glue");
    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=log");
    println!("cargo:rustc-link-lib=android");
}
