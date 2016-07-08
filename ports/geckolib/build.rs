/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::env;
use std::path::Path;
use std::process::{Command, exit};

#[cfg(windows)]
fn find_python() -> String {
    if Command::new("python2.7.exe").arg("--version").output().is_ok() {
        return "python2.7.exe".to_owned();
    }

    if Command::new("python27.exe").arg("--version").output().is_ok() {
        return "python27.exe".to_owned();
    }

    if Command::new("python.exe").arg("--version").output().is_ok() {
        return "python.exe".to_owned();
    }

    panic!(concat!("Can't find python (tried python2.7.exe, python27.exe, and python.exe)! ",
                   "Try fixing PATH or setting the PYTHON env var"));
}

#[cfg(not(windows))]
fn find_python() -> String {
    if Command::new("python2.7").arg("--version").output().unwrap().status.success() {
        "python2.7"
    } else {
        "python"
    }.to_owned()
}

fn main() {
    let python = env::var("PYTHON").ok().unwrap_or_else(find_python);

    // Mako refuses to load templates outside the scope of the current working directory,
    // so we need to run it from the top source directory.
    let geckolib_dir = Path::new(file!()).parent().unwrap();
    let top_dir = geckolib_dir.join("..").join("..");

    let properties_dir = Path::new("components").join("style").join("properties");
    println!("cargo:rerun-if-changed={}", top_dir.join(&properties_dir).to_str().unwrap());
    println!("cargo:rerun-if-changed={}", geckolib_dir.join("properties.mako.rs").to_str().unwrap());

    let status = Command::new(python)
        .current_dir(&top_dir)
        .arg(&properties_dir.join("build.py"))
        .arg("gecko")
        .arg("geckolib")
        .status()
        .unwrap();
    if !status.success() {
        exit(1)
    }
}
