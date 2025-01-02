/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;
use std::{env, fmt};

use phf_shared::{self, FmtConst};
use serde_json::{self, Value};

fn main() {
    let start = Instant::now();

    let style_out_dir = PathBuf::from(env::var_os("DEP_SERVO_STYLE_CRATE_OUT_DIR").unwrap());
    let css_properties_json = style_out_dir.join("css-properties.json");
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    println!("cargo::rerun-if-changed=dom/webidls");
    println!("cargo::rerun-if-changed=dom/bindings/codegen");
    println!("cargo::rerun-if-changed={}", css_properties_json.display());
    println!("cargo::rerun-if-changed=../../third_party/WebIDL/WebIDL.py");
    // NB: We aren't handling changes in `third_party/ply` here.

    let status = Command::new(find_python())
        .arg("dom/bindings/codegen/run.py")
        .arg(&css_properties_json)
        .arg(&out_dir)
        .status()
        .unwrap();
    if !status.success() {
        std::process::exit(1)
    }

    println!("Binding generation completed in {:?}", start.elapsed());

    let json = out_dir.join("InterfaceObjectMapData.json");
    let json: Value = serde_json::from_reader(File::open(json).unwrap()).unwrap();
    let mut map = phf_codegen::Map::new();
    for (key, value) in json.as_object().unwrap() {
        map.entry(Bytes(key), value.as_str().unwrap());
    }
    let phf = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("InterfaceObjectMapPhf.rs");
    let mut phf = File::create(phf).unwrap();
    writeln!(
        &mut phf,
        "pub static MAP: phf::Map<&'static [u8], fn(JSContext, HandleObject)> = {};",
        map.build(),
    )
    .unwrap();
}

#[derive(Eq, Hash, PartialEq)]
struct Bytes<'a>(&'a str);

impl<'a> FmtConst for Bytes<'a> {
    fn fmt_const(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        // https://github.com/rust-lang/rust/issues/55223
        // should technically be just `write!(formatter, "b\"{}\"", self.0)
        // but the referenced issue breaks promotion in the surrounding code
        write!(formatter, "{{ const FOO: &[u8] = b\"{}\"; FOO }}", self.0)
    }
}

impl<'a> phf_shared::PhfHash for Bytes<'a> {
    fn phf_hash<H: std::hash::Hasher>(&self, hasher: &mut H) {
        self.0.as_bytes().phf_hash(hasher)
    }
}

/// Tries to find a suitable python
///
/// Algorithm
/// 1. Trying to find python3/python in $VIRTUAL_ENV (this should be from Servo's venv)
/// 2. Checking PYTHON3 (set by mach)
/// 3. Falling back to the system installation.
///
/// Note: This function should be kept in sync with the version in `components/servo/build.rs`
fn find_python() -> PathBuf {
    let mut candidates = vec![];
    if let Some(venv) = env::var_os("VIRTUAL_ENV") {
        let bin_directory = PathBuf::from(venv).join("bin");

        let python3 = bin_directory.join("python3");
        if python3.exists() {
            candidates.push(python3);
        }
        let python = bin_directory.join("python");
        if python.exists() {
            candidates.push(python);
        }
    };
    if let Some(python3) = env::var_os("PYTHON3") {
        let python3 = PathBuf::from(python3);
        if python3.exists() {
            candidates.push(python3);
        }
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
