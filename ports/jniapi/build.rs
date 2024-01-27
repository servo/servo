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

    // Generate GL bindings. For now, we only support EGL.
    if target.contains("android") {
        let mut file = File::create(&dest.join("egl_bindings.rs")).unwrap();
        Registry::new(Api::Egl, (1, 5), Profile::Core, Fallbacks::All, [])
            .write_bindings(gl_generator::StaticStructGenerator, &mut file)
            .unwrap();
        println!("cargo:rustc-link-lib=EGL");
    }

    let mut default_prefs = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    default_prefs.push("../../resources/prefs.json");
    let prefs: Value = serde_json::from_reader(File::open(&default_prefs).unwrap()).unwrap();
    let file = File::create(&dest.join("prefs.json")).unwrap();
    serde_json::to_writer(file, &prefs).unwrap();
}
