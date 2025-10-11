/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;
use std::env;
use std::io::Write;

fn main() {
    let start = Instant::now();

    let style_out_dir = PathBuf::from(env::var_os("DEP_SERVO_STYLE_CRATE_OUT_DIR").unwrap());
    let css_properties_json = style_out_dir.join("css-properties.json");
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    println!("cargo:out_dir={}", out_dir.display());

    println!("cargo::rerun-if-changed=webidls");
    println!("cargo::rerun-if-changed=../../../script_bindings/codegen/");
    println!("cargo::rerun-if-changed={}", css_properties_json.display());
    println!("cargo::rerun-if-changed=../../../third_party/WebIDL/WebIDL.py");
    // NB: We aren't handling changes in `third_party/ply` here.

    let command = find_python()
        .arg("../../../script_bindings/codegen/run.py")
        .arg(&css_properties_json)
        .arg(&out_dir)
        .output()
        .unwrap();
    std::io::stdout().write_all(&command.stdout).unwrap();
    if !command.status.success() {
        std::process::exit(1)
    }

    println!("Binding generation completed in {:?}", start.elapsed());
}

/// Tries to find a suitable python, which in Servo is always `uv run python` unless we are running
/// as a descendant of `uv run python`. In that case, we can use either `uv run python` or `python`
/// (uv does not provide a `python3` on Windows).
///
/// More details: <https://book.servo.org/hacking/setting-up-your-environment.html#check-tools>
///
/// Note: This function should be kept in sync with the version in `components/script/build.rs`
fn find_python() -> Command {
    let mut command = Command::new("uv");
    command.args(["run", "--no-project", "python"]);

    if command.output().is_ok_and(|out| out.status.success()) {
        return command;
    }

    panic!("Can't find python (tried `{command:?}`)! Is uv installed and in PATH?")
}
