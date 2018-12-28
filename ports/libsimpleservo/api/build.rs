/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use gl_generator::{Api, Fallbacks, Profile, Registry};
use std::env;
use std::fs::File;
use std::path::Path;

fn main() {
    let target = env::var("TARGET").unwrap();
    // Generate GL bindings
    // For now, we only support EGL, and only on Windows and Android.
    if target.contains("android") || target.contains("windows") {
        let dest = env::var("OUT_DIR").unwrap();
        let mut file = File::create(&Path::new(&dest).join("egl_bindings.rs")).unwrap();
        if target.contains("android") {
            Registry::new(Api::Egl, (1, 5), Profile::Core, Fallbacks::All, [])
                .write_bindings(gl_generator::StaticStructGenerator, &mut file)
                .unwrap();
        }
        if target.contains("windows") {
            Registry::new(Api::Egl, (1, 5), Profile::Core, Fallbacks::All, [])
                .write_bindings(gl_generator::StructGenerator, &mut file)
                .unwrap();
        };
    }
}
