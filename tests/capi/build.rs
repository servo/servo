/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::{env, fs};

fn find_and_copy_dll(dll_name: &str, src_dir: &Path, dest_dir: &Path) -> bool {
    fn scan_dir(dll_name: &str, src_dir: &Path, dest_dir: &Path) -> bool {
        let Ok(entries) = fs::read_dir(src_dir) else {
            return false;
        };

        let paths = entries.filter_map(|entry| entry.ok().map(|entry| entry.path()));

        for path in paths {
            if path.is_dir() {
                if scan_dir(dll_name, &path, dest_dir) {
                    return true;
                }
            } else if path
                .file_name()
                .is_some_and(|file_name| file_name == dll_name)
            {
                let dest = dest_dir.join(dll_name);
                fs::copy(&path, &dest)
                    .unwrap_or_else(|e| panic!("error when copying {}: {}", dll_name, e));
                eprintln!("'copied '{}' to '{}'", path.display(), dest.display());
                return true;
            }
        }

        false
    }

    if !scan_dir(dll_name, src_dir, dest_dir) {
        eprintln!("DLL '{dll_name}' not found in {}", src_dir.display());
        return false;
    }

    true
}

fn main() {
    let crate_dir = PathBuf::from(
        env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR should be set by cargo"),
    );
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR should be set by cargo"));

    // Re-run the build script whenever any crate in the workspace changes
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

    let target = env::var("TARGET").expect("TARGET should be set by cargo");
    let profile = env::var("PROFILE").expect("PROFILE should be set by cargo");
    // `cargo cinstall` disallows the profile name 'debug' and requires us
    // to pass `dev` instead. The name of the directory inside `./target` is still `debug`.
    // So we need to map cargo's PROFILE to the output directory name and the
    // profile name `cargo cinstall` expects.
    let (profile_dir, profile_name) = match profile.as_str() {
        "release" => ("release", "release"),
        _ => ("debug", "dev"),
    };
    let root = crate_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("servo_capi_tests is not under workspace root");

    // This build script is about to execute a nested cargo instance. The outer cargo
    // instance that is running the current build script is already holding a lock
    // on the `target/debug` or `target/release` directory. So we specify our own
    // directory for `cargo cinstall` to use.
    //
    // NOTE: This is deliberately not inside `OUT_DIR` to avoid hitting the MAX_PATH
    // limit on Windows when building mozangle.
    let cinstall_target_dir = root.join("target").join("cinstall");
    eprintln!(
        "Running: cargo cinstall -p servo-capi --target {target} --profile {profile_name} --target-dir {} --destdir {} --prefix /",
        cinstall_target_dir.display(),
        out_dir.display()
    );

    // Run `cargo cinstall` to build and install servo_capi library and headers into OUT_DIR.
    let cinstall_status = Command::new("cargo")
        .current_dir(&root)
        .args(["cinstall", "-p", "servo-capi"])
        .args(["--target", &target])
        .args(["--profile", profile_name])
        .args(["--library-type", "cdylib"])
        .args(["--target-dir".as_ref(), cinstall_target_dir.as_os_str()])
        .args(["--destdir".as_ref(), out_dir.as_os_str()])
        .args(["--prefix", "/"])
        // `--meson-paths` is needed to ensure importlib is `servo_capi.lib`
        // instead of `servo_capi.dll.lib`
        .arg("--meson-paths")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect("failed to run cargo cinstall");

    if !cinstall_status.success() {
        panic!("cargo cinstall failed with status: {cinstall_status}");
    }
    eprintln!("cargo cinstall completed");

    if cfg!(target_os = "windows") {
        // On Windows, the DLL is copied to `OUT_DIR/bin/` but it needs to be
        // in the same directory as the test binary at runtime.
        // Assuming OUT_DIR is `<target>/<profile>/build/<crate>-<hash>/out`,
        // going up 3 levels should give us the final binary directory.
        let bin_dir = out_dir.join("bin");
        let test_binary_dir = out_dir
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
            .expect("failed to derive test binary directory");

        let paths = fs::read_dir(&bin_dir)
            .expect("unable to read OUT_DIR/bin")
            .filter_map(|entry| entry.ok().map(|entry| entry.path()));
        for path in paths {
            if path
                .extension()
                .is_some_and(|ext| ext.to_str() == Some("dll"))
            {
                let dest = test_binary_dir.join(path.file_name().unwrap());
                fs::copy(&path, &dest).expect("failed to copy DLL");
                eprintln!("'{}' copied to '{}'", path.display(), dest.display());
            }
        }

        // Windows also needs the ANGLE DLLs to be present in the test binary directory.
        let search_dir = cinstall_target_dir
            .join(&target)
            .join(profile_dir)
            .join("build");
        eprintln!("Searching for ANGLE DLLs in {}:", search_dir.display());
        let copied = if search_dir.exists() {
            ["libEGL.dll", "libGLESv2.dll"]
                .iter()
                .all(|dll_name| find_and_copy_dll(dll_name, &search_dir, &test_binary_dir))
        } else {
            panic!("cargo cinstall's build directory not found");
        };

        if !copied {
            panic!("Unable to copy required DLLs");
        }
    }

    // Link the test code with the C FFI cdylib.
    let lib_dir = out_dir.join("lib");
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=dylib=servo_capi");

    // cargo cinstall installs headers under "<prefix>/include/servo".
    // See [package.metadata.capi.header] in servo_capi/Cargo.toml.
    let include_dir = out_dir.join("include");
    cc::Build::new()
        .file(crate_dir.join("c").join("test_api.c"))
        .std("c11")
        .flag_if_supported("/experimental:c11atomics")
        .include(&include_dir)
        .compile("c_api_tests");

    cc::Build::new()
        .file(crate_dir.join("c").join("test_integration.c"))
        .std("c11")
        .flag_if_supported("/experimental:c11atomics")
        .include(&include_dir)
        .compile("c_integration_tests");
}
