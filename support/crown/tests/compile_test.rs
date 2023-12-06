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

fn run_mode(mode: &'static str, bless: bool) {
    let mut config = compiletest::Config {
        bless,
        edition: Some("2021".into()),
        ..Default::default()
    }
    .tempdir();

    let cfg_mode = mode.parse().expect("Invalid mode");
    config.mode = cfg_mode;
    config.src_base = PathBuf::from("tests").join(mode);
    config.rustc_path = PROFILE_PATH.join("crown");
    config.target_rustcflags = Some(format!(
        "-L {} -L {} -Zcrate-attr=feature(register_tool) -Zcrate-attr=register_tool(crown)",
        PROFILE_PATH.display(),
        PROFILE_PATH.join("deps").display()
    ));
    // Does not work reliably: https://github.com/servo/servo/pull/30508#issuecomment-1834542203
    //config.link_deps();
    config.strict_headers = true;

    compiletest::run_tests(&config);
}

#[test]
fn compile_test() {
    let bless = env::var("BLESS").map_or(false, |x| !x.trim().is_empty());
    run_mode("compile-fail", bless);
    run_mode("run-pass", bless);
    // UI test fails on windows
    if !cfg!(windows) {
        run_mode("ui", bless);
    }
}
