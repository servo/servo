/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::env;
use std::fs::File;
use std::path::PathBuf;

use gl_generator::{Api, Fallbacks, Profile, Registry};
use serde_json::{self, Value};
use vergen::EmitBuilder;

fn main() {
    let target = env::var("TARGET").unwrap();
    let dest = PathBuf::from(&env::var("OUT_DIR").unwrap());

    if let Err(error) = EmitBuilder::builder()
        .fail_on_error()
        .git_sha(true /* short */)
        .emit()
    {
        println!(
            "cargo:warning=Could not generate git version information: {:?}",
            error
        );
        println!("cargo:rustc-env=VERGEN_GIT_SHA=nogit");
    }

    // On MacOS, all dylib dependencies are shipped along with the binary
    // in the "/lib" directory. Setting the rpath here, allows the dynamic
    // linker to locate them. See `man dyld` for more info.
    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path/lib/");

    // Generate GL bindings
    // For now, we only support EGL, and only on Windows and Android.
    if target.contains("android") || target.contains("windows") {
        let mut file = File::create(&dest.join("egl_bindings.rs")).unwrap();
        Registry::new(Api::Egl, (1, 5), Profile::Core, Fallbacks::All, [])
            .write_bindings(gl_generator::StaticStructGenerator, &mut file)
            .unwrap();

        // Historically, Android builds have succeeded with rustc-link-lib=EGL.
        // On Windows when relying on %LIBS% to contain libEGL.lib, however,
        // we must explicitly use rustc-link-lib=libEGL or rustc will attempt
        // to link EGL.lib instead.
        if target.contains("windows") {
            println!("cargo:rustc-link-lib=libEGL");
        } else {
            println!("cargo:rustc-link-lib=EGL");
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

    let mut default_prefs = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    default_prefs.push("../../../resources/prefs.json");
    let prefs: Value = serde_json::from_reader(File::open(&default_prefs).unwrap()).unwrap();
    let file = File::create(&dest.join("prefs.json")).unwrap();
    serde_json::to_writer(file, &prefs).unwrap();
}
