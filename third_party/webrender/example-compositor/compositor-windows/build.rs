/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

fn main() {
    // HACK - This build script relies on Gecko having been built, so that the ANGLE libraries
    //        have already been compiled. It also assumes they are being built with an in-tree
    //        x86_64 object directory.

    cc::Build::new()
        .file("src/lib.cpp")
        .include("../../../angle/checkout/include")
        .compile("windows");

    // Set up linker paths for ANGLE that is built by Gecko
    println!("cargo:rustc-link-search=../../obj-x86_64-pc-mingw32/gfx/angle/targets/libEGL");
    println!("cargo:rustc-link-search=../../obj-x86_64-pc-mingw32/gfx/angle/targets/libGLESv2");

    // Link to libEGL and libGLESv2 (ANGLE) and D3D11 + DirectComposition
    println!("cargo:rustc-link-lib=libEGL");
    println!("cargo:rustc-link-lib=libGLESv2");
    println!("cargo:rustc-link-lib=dcomp");
    println!("cargo:rustc-link-lib=d3d11");
    println!("cargo:rustc-link-lib=dwmapi");
}
