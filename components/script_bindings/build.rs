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

    println!("cargo:out_dir={}", out_dir.display());

    println!("cargo::rerun-if-changed=webidls");
    println!("cargo::rerun-if-changed=codegen");
    println!("cargo::rerun-if-changed={}", css_properties_json.display());
    println!("cargo::rerun-if-changed=../../third_party/WebIDL/WebIDL.py");
    // NB: We aren't handling changes in `third_party/ply` here.

    let status = find_python()
        .arg("codegen/run.py")
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
        let parts = value.as_array().unwrap();
        map.entry(
            Bytes(key),
            format!(
                "Interface {{ define: {}, enabled: {} }}",
                parts[0].as_str().unwrap(),
                parts[1].as_str().unwrap()
            ),
        );
    }
    let phf = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("InterfaceObjectMapPhf.rs");
    let mut phf = File::create(phf).unwrap();
    writeln!(
        &mut phf,
        "pub(crate) static MAP: phf::Map<&'static [u8], Interface> = {};",
        map.build(),
    )
    .unwrap();
}

#[derive(Eq, Hash, PartialEq)]
struct Bytes<'a>(&'a str);

impl FmtConst for Bytes<'_> {
    fn fmt_const(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "b\"{}\"", self.0)
    }
}

impl phf_shared::PhfHash for Bytes<'_> {
    fn phf_hash<H: std::hash::Hasher>(&self, hasher: &mut H) {
        self.0.as_bytes().phf_hash(hasher)
    }
}

fn try_python_command(mut command: Command) -> Result<Command, String> {
    let command_result = command.output();

    if let Ok(output) = command_result {
        return if output.status.success() {
            Ok(command)
        } else {
            Err(format!(
                "`{command:?}` failed with {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        };
    }
    Err(format!("`{command:?}` failed to run (is it installed?)"))
}

/// Tries to find a suitable python, which in Servo is always `uv run python`.
///
/// To be accommodating to different environments, which may manage python differently, we fallback
/// to `python3` and `python` in that order.
///
/// Note: This function should be kept in sync with the version in `components/servo/build.rs`
fn find_python() -> Command {
    let mut command = Command::new("uv");
    command.args(["run", "--frozen", "python"]);

    let command_result = try_python_command(command).inspect_err(|e| println!("cargo:warning={e}"));
    if let Ok(command) = command_result {
        return command;
    }

    println!(
        "cargo:warning=`uv` not found - Falling back to the default python! \
        If the build fails, please install uv and make sure it is in your PATH or make sure \
        to provision a python environment >= python 3.11."
    );

    let python3 = Command::new("python3");
    let python3_result = try_python_command(python3);
    if let Ok(command) = python3_result {
        return command;
    }

    let python = Command::new("python");
    let python_result = try_python_command(python);
    if let Ok(command) = python_result {
        return command;
    }

    // We first try `python` before printing an error for `python3`, since python3 is often missing
    // provided via python on Windows (but not necessarily on linux).
    println!("cargo:warning={}", python3_result.unwrap_err());
    println!("cargo:warning={}", python_result.unwrap_err());

    panic!("No suitable python found! Tried: `uv run python`, `python3`, `python`.");
}
