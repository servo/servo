/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate gl_generator;

use gl_generator::{Api, Fallbacks, Profile, Registry, StaticStructGenerator};
use std::env;
use std::fs::File;
use std::path::Path;
use std::process;
use std::process::{Command, Stdio};

fn main() {
    // Generate EGL bindings
    let dest = env::var("OUT_DIR").unwrap();
    let mut file = File::create(&Path::new(&dest).join("egl_bindings.rs")).unwrap();
    Registry::new(Api::Egl, (1, 5), Profile::Core, Fallbacks::All, [])
        .write_bindings(StaticStructGenerator, &mut file)
        .unwrap();

    let target = env::var("TARGET").unwrap();
    if target.contains("android") {
        android_main()
    }
}

fn android_main() {
    // Get the NDK path from NDK_HOME env.
    let ndk_path = env::var_os("ANDROID_NDK").expect("Please set the ANDROID_NDK environment variable");
    let ndk_path = Path::new(&ndk_path);

    // Build up the path to the NDK compilers
    // Options for host are:  "linux-x86_64" "linux-x86" "darwin-x86_64" "darwin-x86"
    // per: https://android.googlesource.com/platform/ndk/+/ics-mr0/docs/STANDALONE-TOOLCHAIN.html

    let host = env::var("HOST").unwrap();
    let google_host = match host.as_ref() {
        "i686-unknown-linux-gnu" => "linux-x86",
        "x86_64-apple-darwin" => "darwin-x86_64",
        "x86_64-unknown-linux-gnu" => "linux-x86_64",
        _ => panic!("Unknown support android cross-compile host: {}", host)
    };

    let target = env::var("TARGET").unwrap();
    let arch = if target.contains("arm") {
        "arch-arm"
    } else if target.contains("aarch64") {
        "arch-arm64"
    } else if target.contains("x86") || target.contains("i686") {
        "arch-x86"
    } else if target.contains("mips") {
        "arch-mips"
    } else {
        panic!("Invalid target architecture {}", target);
    };

    let platform = if target.contains("aarch64") {
        "android-21"
    } else {
        "android-18"
    };

    let (toolchain, prefix) = if target.contains("armv7") {
        let toolchain = "arm-linux-androideabi";
        (toolchain.into(), toolchain.into())
    } else if target.contains("i686") {
        ("x86".into(), target)
    } else {
        (target.clone(), target)
    };

    let toolchain_path = ndk_path.join("toolchains").join(format!("{}-4.9", toolchain)).join("prebuilt").
        join(google_host);
    println!("toolchain path is: {}", toolchain_path.to_str().unwrap());

    // Get the output directory.
    let out_dir = env::var("OUT_DIR").expect("Cargo should have set the OUT_DIR environment variable");
    let directory = Path::new(&out_dir);

    // compiling android_native_app_glue.c
    if Command::new(toolchain_path.join("bin").join(format!("{}-gcc", prefix)))
        .arg(ndk_path.join("sources").join("android").join("native_app_glue").join("android_native_app_glue.c"))
        .arg("-c")
        .arg("-o").arg(directory.join("android_native_app_glue.o"))
        .arg("--sysroot").arg(ndk_path.join("platforms").join(platform).join(arch))
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status().unwrap().code().unwrap() != 0
    {
        println!("Error while executing gcc");
        process::exit(1)
    }

    // compiling libandroid_native_app_glue.a
    if Command::new(toolchain_path.join("bin").join(format!("{}-ar", prefix)))
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
