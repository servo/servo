use std::{env, path::PathBuf};

#[path = "codegen/main.rs"]
mod codegen;

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
    codegen::io(input_paths, output_path, false);
}
