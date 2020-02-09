/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use gl_generator::{Api, Fallbacks, Profile, Registry};
use std::env;
use std::fs::File;
use std::path::PathBuf;

fn main() {
    let target = env::var("TARGET").unwrap();
    let dest = PathBuf::from(&env::var("OUT_DIR").unwrap());

    // Generate GL bindings
    // For now, we only support EGL, and only on Windows and Android.
    if target.contains("android") || target.contains("windows") {
        let mut file = File::create(&dest.join("egl_bindings.rs")).unwrap();
        Registry::new(Api::Egl, (1, 5), Profile::Core, Fallbacks::All, [])
            .write_bindings(gl_generator::StaticStructGenerator, &mut file)
            .unwrap();

        // Historically, Android builds have succeeded with rust-link-lib=EGL.
        // On Windows when relying on %LIBS% to contain libEGL.lib, however,
        // we must explicitly use rustc-link-lib=libEGL or rustc will attempt
        // to link EGL.lib instead.
        if target.contains("windows") {
            println!("cargo:rustc-link-lib=libEGL");
        } else {
            println!("cargo:rust-link-lib=EGL");
        }
    }

    if target.contains("linux") ||
        target.contains("dragonfly") ||
        target.contains("freebsd") ||
        target.contains("openbsd")
    {
        let mut file = File::create(&dest.join("glx_bindings.rs")).unwrap();
        Registry::new(Api::Glx, (1, 4), Profile::Core, Fallbacks::All, [])
            .write_bindings(gl_generator::StructGenerator, &mut file)
            .unwrap();
    }
}
