/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::path::Path;
use std::process::Command;
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
