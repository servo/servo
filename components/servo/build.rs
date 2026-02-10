/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::path::Path;
use std::process::{Command, Stdio};
use std::{env, fs};

fn main() {
    if cfg!(feature = "media-gstreamer") {
        println!("cargo:rerun-if-changed=../../python/servo/gstreamer.py");

        let output = find_python()
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

fn try_python_command(program: &str, args: &[&str]) -> Result<(), String> {
    let mut command = Command::new(program);
    command.args(args);
    command.arg("--version");
    command.stdin(Stdio::null());

    let command_result = command.output();

    if let Ok(output) = command_result {
        return if output.status.success() {
            Ok(())
        } else {
            Err(format!(
                "`{} {:?}` failed with {}",
                program,
                args,
                String::from_utf8_lossy(&output.stderr)
            ))
        };
    }
    Err(format!(
        "`{} {:?}` failed to run (is it installed?)",
        program, args
    ))
}

/// Tries to find a suitable python, which in Servo is always `uv run python`.
///
/// To be accommodating to different environments, which may manage python differently, we fallback
/// to `python3` and `python` in that order.
///
/// Note: This function should be kept in sync with the version in `components/script_bindings/build.rs`
fn find_python() -> Command {
    // Test uv first - if it works, create a FRESH command to return
    let uv_result =
        try_python_command("uv", &["run", "python"]).inspect_err(|e| println!("cargo:warning={e}"));
    if uv_result.is_ok() {
        let mut cmd = Command::new("uv");
        cmd.args(["run", "python"]);
        return cmd;
    }

    println!(
        "cargo:warning=`uv` not found - Falling back to the default python! \
        If the build fails, please install uv and make sure it is in your PATH or make sure \
        to provision a python environment >= python 3.11."
    );

    let python3_result = try_python_command("python3", &[]);
    if python3_result.is_ok() {
        return Command::new("python3");
    }

    let python_result = try_python_command("python", &[]);
    if python_result.is_ok() {
        return Command::new("python");
    }

    // We first try `python` before printing an error for `python3`, since python3 is often missing
    // provided via python on Windows (but not necessarily on linux).
    println!("cargo:warning={}", python3_result.unwrap_err());
    println!("cargo:warning={}", python_result.unwrap_err());

    panic!("No suitable python found! Tried: `uv run python`, `python3`, `python`.");
}
