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
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    let status = Command::new(find_python())
        .arg("dom/bindings/codegen/run.py")
        .arg(style_out_dir.join("css-properties.json"))
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

fn find_python() -> String {
    env::var("PYTHON3").ok().unwrap_or_else(|| {
        let candidates = if cfg!(windows) {
            ["python.exe", "python"]
        } else {
            ["python3", "python"]
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
