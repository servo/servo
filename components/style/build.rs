/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#[macro_use]
extern crate lazy_static;
#[cfg(feature = "gecko")]
extern crate bindgen;
#[cfg(feature = "gecko")]
extern crate log;
#[cfg(feature = "gecko")]
extern crate regex;
#[cfg(feature = "gecko")]
extern crate toml;
extern crate walkdir;

use std::env;
use std::path::Path;
use std::process::{exit, Command};
use walkdir::WalkDir;

#[cfg(feature = "gecko")]
mod build_gecko;

#[cfg(not(feature = "gecko"))]
mod build_gecko {
    pub fn generate() {}
}

lazy_static! {
    pub static ref PYTHON: String = env::var("PYTHON3").ok().unwrap_or_else(|| {
        let candidates = if cfg!(windows) {
            ["python3.exe"]
        } else {
            ["python3"]
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
    });
}

fn generate_properties(engine: &str) {
    for entry in WalkDir::new("properties") {
        let entry = entry.unwrap();
        match entry.path().extension().and_then(|e| e.to_str()) {
            Some("mako") | Some("rs") | Some("py") | Some("zip") => {
                println!("cargo:rerun-if-changed={}", entry.path().display());
            },
            _ => {},
        }
    }

    let script = Path::new(&env::var_os("CARGO_MANIFEST_DIR").unwrap())
        .join("properties")
        .join("build.py");

    let status = Command::new(&*PYTHON)
        .arg(&script)
        .arg(engine)
        .arg("style-crate")
        .status()
        .unwrap();
    if !status.success() {
        exit(1)
    }
}

fn main() {
    let gecko = cfg!(feature = "gecko");
    let servo = cfg!(feature = "servo");
    let l2013 = cfg!(feature = "servo-layout-2013");
    let l2020 = cfg!(feature = "servo-layout-2020");
    let engine = match (gecko, servo, l2013, l2020) {
        (true, false, false, false) => "gecko",
        (false, true, true, false) => "servo-2013",
        (false, true, false, true) => "servo-2020",
        _ => panic!(
            "\n\n\
             The style crate requires enabling one of its 'servo' or 'gecko' feature flags \
             and, in the 'servo' case, one of 'servo-layout-2013' or 'servo-layout-2020'.\
             \n\n"
        ),
    };
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:out_dir={}", env::var("OUT_DIR").unwrap());
    generate_properties(engine);
    build_gecko::generate();
}
