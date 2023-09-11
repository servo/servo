/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::process::Command;
use std::env;
use std::fs;

extern crate pkg_config;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    fs::create_dir_all(&format!("{}/include", out_dir)).unwrap();
    Command::new("wayland-scanner")
        .args(&["client-header", "/usr/share/wayland-protocols/stable/viewporter/viewporter.xml"])
        .arg(&format!("{}/include/viewporter-client-protocol.h", out_dir))
        .status().unwrap();

    Command::new("wayland-scanner")
        .args(&["public-code", "/usr/share/wayland-protocols/stable/viewporter/viewporter.xml"])
        .arg(&format!("{}/viewporter-protocol.c", out_dir))
        .status().unwrap();

    Command::new("wayland-scanner")
        .args(&["client-header", "/usr/share/wayland-protocols/stable/xdg-shell/xdg-shell.xml"])
        .arg(&format!("{}/include/xdg-shell-client-protocol.h", out_dir))
        .status().unwrap();

    Command::new("wayland-scanner")
        .args(&["public-code", "/usr/share/wayland-protocols/stable/xdg-shell/xdg-shell.xml"])
        .arg(&format!("{}/xdg-shell-protocol.c", out_dir))
        .status().unwrap();

    cc::Build::new()
        .include(&format!("{}/include", out_dir))
        .file("src/lib.cpp")
        .file(&format!("{}/viewporter-protocol.c", out_dir))
        .file(&format!("{}/xdg-shell-protocol.c", out_dir))
        .compile("wayland");

    println!("cargo:rustc-link-lib=dylib=stdc++");

    pkg_config::Config::new()
        .atleast_version("1")
        .probe("egl")
        .unwrap();
    pkg_config::Config::new()
        .atleast_version("1")
        .probe("gl")
        .unwrap();
    pkg_config::Config::new()
        .atleast_version("1")
        .probe("wayland-client")
        .unwrap();
    pkg_config::Config::new()
        .atleast_version("1")
        .probe("wayland-egl")
        .unwrap();

    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/lib.cpp");
}
