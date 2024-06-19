/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use gl_generator::{Api, Fallbacks, Profile, Registry};
use serde_json::Value;
use vergen::EmitBuilder;

fn main() -> Result<(), Box<dyn Error>> {
    // Cargo does not expose the profile name to crates or their build scripts,
    // but we can extract it from OUT_DIR and set a custom cfg() ourselves.
    let out = std::env::var("OUT_DIR")?;
    let out = Path::new(&out);
    let krate = out.parent().unwrap();
    let build = krate.parent().unwrap();
    let profile = build.parent().unwrap();
    if profile.file_name().unwrap() == "production" {
        println!("cargo:rustc-cfg=servo_production");
    } else {
        println!("cargo:rustc-cfg=servo_do_not_use_in_production");
    }

    // Note: We can't use `#[cfg(windows)]`, since that would check the host platform
    // and not the target platform
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();

    if target_os == "windows" {
        #[cfg(windows)]
        {
            let mut res = winres::WindowsResource::new();
            res.set_icon("../../resources/servo.ico");
            res.set_manifest_file("platform/windows/servo.exe.manifest");
            res.compile().unwrap();
        }
        #[cfg(not(windows))]
        panic!("Cross-compiling to windows is currently not supported");
    } else if target_os == "macos" {
        cc::Build::new()
            .file("platform/macos/count_threads.c")
            .compile("count_threads");
    } else if target_os == "android" {
        // Generate GL bindings. For now, we only support EGL.
        let mut file = File::create(out.join("egl_bindings.rs")).unwrap();
        Registry::new(Api::Egl, (1, 5), Profile::Core, Fallbacks::All, [])
            .write_bindings(gl_generator::StaticStructGenerator, &mut file)
            .unwrap();
        println!("cargo:rustc-link-lib=EGL");

        // FIXME: We need this workaround since jemalloc-sys still links
        // to libgcc instead of libunwind, but Android NDK 23c and above
        // don't have libgcc. We can't disable jemalloc for Android as
        // in 64-bit aarch builds, the system allocator uses tagged
        // pointers by default which causes the assertions in SM & mozjs
        // to fail. See https://github.com/servo/servo/issues/32175.
        let mut libgcc = File::create(out.join("libgcc.a")).unwrap();
        libgcc.write_all(b"INPUT(-lunwind)").unwrap();
        println!("cargo:rustc-link-search=native={}", out.display());

        let mut default_prefs = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        default_prefs.push("../../resources/prefs.json");
        let prefs: Value = serde_json::from_reader(File::open(&default_prefs).unwrap()).unwrap();
        let file = File::create(out.join("prefs.json")).unwrap();
        serde_json::to_writer(file, &prefs).unwrap();
    }

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
    if target_os == "macos" {
        println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path/lib/");
    }
    Ok(())
}
