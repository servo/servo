extern crate gl_generator;

use gl_generator::{Registry, Api, Profile, Fallbacks};
use std::env;
use std::fs::File;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(&env::var("OUT_DIR").unwrap());
    let mut file = File::create(&out_dir.join("egl_bindings.rs")).unwrap();
    Registry::new(Api::Egl, (1, 5), Profile::Core, Fallbacks::All, [
                      "EGL_KHR_create_context",
                      "EGL_KHR_platform_android",
                  ])
        .write_bindings(gl_generator::StaticGenerator, &mut file).unwrap();
}
