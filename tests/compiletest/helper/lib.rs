/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate compiletest_rs as compiletest;

use std::env;
use std::path::PathBuf;

pub fn run_mode(mode: &'static str) {
    let mut config = compiletest::default_config();
    let cfg_mode = mode.parse().ok().expect("Invalid mode");

    config.mode = cfg_mode;
    config.src_base = PathBuf::from(format!("{}", mode));

    let mut base_path = env::current_dir().expect("Current directory is invalid");
    base_path.pop();
    base_path.pop();
    base_path.pop();

    let mode = env::var("BUILD_MODE").expect("BUILD_MODE environment variable must be set");
    let debug_path = base_path.join(PathBuf::from(format!("target/{}", mode)));
    let deps_path = base_path.join(PathBuf::from(format!("target/{}/deps", mode)));

    config.target_rustcflags = Some(format!("-L {} -L {}",  debug_path.display(), deps_path.display()));
    compiletest::run_tests(&config);
}
