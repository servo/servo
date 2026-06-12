use std::{env, path::PathBuf};

use webdriver_traits_codegen::io;

fn main() {
    println!("cargo::rerun-if-changed=cddls");

    let input_paths = vec![
        "cddls/remote.cddl".to_string(),
        "cddls/local.cddl".to_string(),
    ];
    let output_path = {
        let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
        Some(out.join("webdriver_bidi.rs").to_str().unwrap().to_string())
    };
    io(input_paths, output_path, false);
}
