/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::path::Path;
use std::process::Command;
use std::{env, fs};

fn main() {
    println!("cargo:rerun-if-changed=../../python/servo/gstreamer.py");

    let output = Command::new(find_python())
        .arg("../../python/servo/gstreamer.py")
        .arg(std::env::var_os("TARGET").unwrap())
        .output()
        .unwrap();
    if !output.status.success() {
        eprintln!("{}", String::from_utf8_lossy(&output.stdout));
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        std::process::exit(1)
    }
    let path = Path::new(&env::var_os("OUT_DIR").unwrap()).join("gstreamer_plugins.rs");
    fs::write(path, output.stdout).unwrap();
}

fn find_python() -> String {
    env::var("PYTHON3").ok().unwrap_or_else(|| {
        let candidates = if cfg!(windows) {
            ["python3.8.exe", "python38.exe", "python.exe"]
        } else {
            ["python3.8", "python3", "python"]
        };
        for &name in &candidates {
            if Command::new(name)
                .arg("--version")
                .output()
                .ok()
                .map_or(false, |out| out.status.success())
            {
                return name.to_owned();
            }
        }
        panic!(
            "Can't find python (tried {})! Try fixing PATH or setting the PYTHON3 env var",
            candidates.join(", ")
        )
    })
}
