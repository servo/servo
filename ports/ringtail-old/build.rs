/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::error::Error;
use std::process::Command;

fn git_sha() -> Result<String, String> {
    let output = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .map_err(|e| e.to_string())?;
    if output.status.success() {
        let hash = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
        Ok(hash.trim().to_owned())
    } else {
        let stderr = String::from_utf8(output.stderr).map_err(|e| e.to_string())?;
        Err(stderr)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo::rustc-check-cfg=cfg(servo_production)");
    println!("cargo::rustc-check-cfg=cfg(servo_do_not_use_in_production)");
    
    let out = std::env::var("OUT_DIR")?;
    let out = std::path::Path::new(&out);
    let krate = out.parent().unwrap();
    let build = krate.parent().unwrap();
    let profile = build
        .parent()
        .unwrap()
        .file_name()
        .unwrap()
        .to_string_lossy();
    if profile == "production" || profile.starts_with("production-") {
        println!("cargo:rustc-cfg=servo_production");
    } else {
        println!("cargo:rustc-cfg=servo_do_not_use_in_production");
    }

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();

    if target_os == "windows" {
        #[cfg(windows)]
        {
            let mut res = winresource::WindowsResource::new();
            res.set_icon("../../resources/servo.ico");
            res.compile().unwrap();
        }
        #[cfg(not(windows))]
        panic!("Cross-compiling to windows is currently not supported");
    } else if target_os == "macos" {
        println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path/lib/");
    }

    match git_sha() {
        Ok(hash) => println!("cargo:rustc-env=GIT_SHA={}", hash),
        Err(error) => {
            println!(
                "cargo:warning=Could not generate git version information: {:?}",
                error
            );
            println!("cargo:rustc-env=GIT_SHA=nogit");
        },
    }

    Ok(())
}
