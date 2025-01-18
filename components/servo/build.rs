/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};

fn main() {
    if cfg!(feature = "media-gstreamer") {
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
}

/// Tries to find a suitable python
///
/// Algorithm
/// 1. Trying to find python3/python in $VIRTUAL_ENV (this should be from servos venv)
/// 2. Checking PYTHON3 (set by mach)
/// 3. Falling back to the system installation.
///
/// Note: This function should be kept in sync with the version in `components/script/build.rs`
fn find_python() -> PathBuf {
    let mut candidates = vec![];
    if let Some(venv) = env::var_os("VIRTUAL_ENV") {
        // See: https://docs.python.org/3/library/venv.html#how-venvs-work
        let bin_dir = if cfg!(windows) { "Scripts" } else { "bin" };
        let bin_directory = PathBuf::from(venv).join(bin_dir);
        candidates.push(bin_directory.join("python3"));
        candidates.push(bin_directory.join("python"));
    }
    if let Some(python3) = env::var_os("PYTHON3") {
        candidates.push(PathBuf::from(python3));
    }

    let system_python = ["python3", "python"].map(PathBuf::from);
    candidates.extend_from_slice(&system_python);

    for name in &candidates {
        // Command::new() allows us to omit the `.exe` suffix on windows
        if Command::new(name)
            .arg("--version")
            .output()
            .is_ok_and(|out| out.status.success())
        {
            return name.to_owned();
        }
    }
    let candidates = candidates
        .into_iter()
        .map(|c| c.into_os_string())
        .collect::<Vec<_>>();
    panic!(
        "Can't find python (tried {:?})! Try enabling Servo's Python venv, \
        setting the PYTHON3 env var or adding python3 to PATH.",
        candidates.join(", ".as_ref())
    )
}
