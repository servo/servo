/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::env;
use std::path::PathBuf;

use compiletest_rs as compiletest;
use once_cell::sync::Lazy;

static PROFILE_PATH: Lazy<PathBuf> = Lazy::new(|| {
    let current_exe_path = env::current_exe().unwrap();
    let deps_path = current_exe_path.parent().unwrap();
    let profile_path = deps_path.parent().unwrap();
    profile_path.into()
});

#[test]
fn compile_test() {
    let bless = env::var("BLESS").map_or(false, |x| !x.trim().is_empty());
    let mut config = compiletest::Config {
        bless,
        edition: Some("2021".into()),
        mode: compiletest::common::Mode::Ui,
        ..Default::default()
    };

    config.target_rustcflags = Some(format!(
        "-Zcrate-attr=feature(register_tool) -Zcrate-attr=register_tool(crown)"
    ));

    config.src_base = "tests/ui".into();
    config.build_base = PROFILE_PATH.join("test/ui");
    config.rustc_path = PROFILE_PATH.join("crown");
    config.link_deps(); // Populate config.target_rustcflags with dependencies on the path

    compiletest::run_tests(&config);
}
