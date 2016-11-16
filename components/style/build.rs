/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[macro_use]
extern crate lazy_static;
#[cfg(feature = "gecko")]
extern crate libbindgen;
#[cfg(feature = "gecko")]
extern crate regex;
extern crate walkdir;

#[cfg(feature = "gecko")]
mod build_gecko;

#[cfg(not(feature = "gecko"))]
mod build_gecko {
    pub fn generate() {}
}

use std::env;
use std::path::Path;
use std::process::{Command, exit};
use walkdir::WalkDir;

lazy_static! {
    static ref PYTHON_ENV: Option<String> = env::var("PYTHON").ok();
    static ref PYTHON: &'static str = {
        if let Some(ref python) = *PYTHON_ENV {
            return &python;
        }
        if cfg!(windows) {
            for &exe in ["python2.7.exe", "python27.exe", "python.exe"].iter() {
                if Command::new(exe).arg("--version").output().is_ok() {
                    return exe;
                }
            }
            panic!(concat!("Can't find python (tried python2.7.exe, python27.exe, and python.exe)! ",
                           "Try fixing PATH or setting the PYTHON env var"));
        }
        if Command::new("python2.7").arg("--version").output().unwrap().status.success() {
            "python2.7"
        } else {
            "python"
        }
    };
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
    let status = Command::new(*PYTHON)
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
    generate_properties();
    if cfg!(feature = "gecko") {
        build_gecko::generate();
    }
}
