/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[macro_use]
extern crate lazy_static;
#[cfg(feature = "bindgen")]
extern crate bindgen;
#[cfg(feature = "bindgen")]
extern crate log;
#[cfg(feature = "bindgen")]
extern crate regex;
#[cfg(feature = "bindgen")]
extern crate toml;
extern crate walkdir;

use std::env;
use std::path::Path;
use std::process::{Command, exit};
use walkdir::WalkDir;

#[cfg(feature = "gecko")]
mod build_gecko;

#[cfg(not(feature = "gecko"))]
mod build_gecko {
    pub fn generate() {}
}

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

lazy_static! {
    pub static ref PYTHON: String = env::var("PYTHON").ok().unwrap_or_else(find_python);
}

fn generate_properties() {
    for entry in WalkDir::new("properties") {
        let entry = entry.unwrap();
        match entry.path().extension().and_then(|e| e.to_str()) {
            Some("mako") | Some("rs") | Some("py") | Some("zip") => {
                println!("cargo:rerun-if-changed={}", entry.path().display());
            }
            _ => {}
        }
    }

    let script = Path::new(file!()).parent().unwrap().join("properties").join("build.py");
    let product = if cfg!(feature = "gecko") { "gecko" } else { "servo" };
    let status = Command::new(&*PYTHON)
        .arg(&script)
        .arg(product)
        .arg("style-crate")
        .arg(if cfg!(feature = "testing") { "testing" } else { "regular" })
        .status()
        .unwrap();
    if !status.success() {
        exit(1)
    }
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:out_dir={}", env::var("OUT_DIR").unwrap());
    generate_properties();
    build_gecko::generate();
}
