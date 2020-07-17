/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use gl_generator::{Api, Fallbacks, Profile, Registry};
use serde_json::{self, Value};
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

    // Merge prefs.json and package-prefs.json
    let mut default_prefs = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    default_prefs.push("../../../resources/prefs.json");
    let mut prefs: Value = serde_json::from_reader(File::open(&default_prefs).unwrap()).unwrap();
    let mut pkg_prefs = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    pkg_prefs.push("../../../resources/package-prefs.json");
    let pkg_prefs: Value = serde_json::from_reader(File::open(&pkg_prefs).unwrap()).unwrap();
    if target.contains("uwp") {
        // Assuming Hololens build
        let to_merge = pkg_prefs
            .as_object()
            .unwrap()
            .get("hololens")
            .unwrap()
            .as_object()
            .unwrap();
        for (key, value) in to_merge.iter() {
            prefs
                .as_object_mut()
                .unwrap()
                .insert(key.clone(), value.clone());
        }
    }
    let file = File::create(&dest.join("prefs.json")).unwrap();
    serde_json::to_writer(file, &prefs).unwrap();
}
