/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let crate_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Re-run the build script whenever any crate in the workspace changes,
    // since `cargo cinstall` depends on `servo_capi` and transitive deps
    // like `servo`, `script`, etc. A better approach would be to use Cargo's
    // [artifact dependencies][1] once that feature is stabilized.
    // [1]: https://doc.rust-lang.org/nightly/cargo/reference/unstable.html#artifact-dependencies
    let root_dir = crate_dir.parent().unwrap().parent().unwrap();
    println!(
        "cargo:rerun-if-changed={}",
        root_dir.join("components").display()
    );
    println!("cargo:rerun-if-changed={}", root_dir.join("ffi").display());

    // Run cargo cinstall to build and install servo_capi library and headers in our
    // OUT_DIR.
    let cinstall_status = Command::new("cargo")
        .args(["cinstall", "-p", "servo-capi", "--destdir"])
        .arg(&out_dir)
        .args(["--prefix", "/"])
        .current_dir(
            crate_dir
                .parent()
                .and_then(|p| p.parent())
                .expect("servo_capi_tests is not under workspace root"),
        )
        .status()
        .expect("failed to run cargo cinstall");

    if !cinstall_status.success() {
        panic!("cargo cinstall failed");
    }

    let lib_dir = out_dir.join("lib");
    // cargo cinstall installs headers under "<prefix>/include/servo".
    // See [package.metadata.capi.header] in servo_capi/Cargo.toml.
    let include_dir = out_dir.join("include");

    // Link the test code with the C FFI cdylib.
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=dylib=servo_capi");

    cc::Build::new()
        .file(crate_dir.join("c").join("test_api.c"))
        .include(&include_dir)
        .compile("c_api_tests");

    cc::Build::new()
        .file(crate_dir.join("c").join("test_integration.c"))
        .include(&include_dir)
        .compile("c_integration_tests");
}
