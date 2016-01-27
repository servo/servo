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

    let debug_path = base_path.join(PathBuf::from("target/debug"));
    let deps_path = base_path.join(PathBuf::from("target/debug/deps"));

    config.target_rustcflags = Some(format!("-L {} -L {}",  debug_path.display(), deps_path.display()));
    compiletest::run_tests(&config);
}
