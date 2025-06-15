/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::env;
use std::path::{Path, PathBuf};

fn main() {
    // copy include! files from script_bindings's OUT_DIR, to script's OUT_DIR
    // this is done to bypass limitation of Rust Analyzer: https://github.com/rust-lang/rust-analyzer/issues/17040
    let script_bindings_out_dir =
        PathBuf::from(env::var_os("DEP_SCRIPT_BINDINGS_CRATE_OUT_DIR").unwrap());
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    // copy concrete files
    [
        "InterfaceTypes.rs",
        "DomTypeHolder.rs",
        "InterfaceObjectMap.rs",
        "ConcreteInheritTypes.rs",
        "UnionTypes.rs",
        "InterfaceObjectMapPhf.rs",
        "StubbedInterfaces.rs",
    ]
    .iter()
    .map(Path::new)
    .for_each(|file| {
        println!(
            "cargo::rerun-if-changed={}",
            script_bindings_out_dir.join(file).display()
        );
        std::fs::copy(
            script_bindings_out_dir.join(file),
            out_dir.join(file.file_name().unwrap()),
        )
        .unwrap();
    });
    // copy ConcreteBindings folder
    let _ = std::fs::create_dir(out_dir.join("ConcreteBindings"));
    let script_concrete_bindings_out_dir = script_bindings_out_dir.join("ConcreteBindings");
    println!(
        "cargo::rerun-if-changed={}",
        script_concrete_bindings_out_dir.display()
    );
    std::fs::read_dir(script_concrete_bindings_out_dir)
        .unwrap()
        .filter_map(|res| res.map(|e| e.path()).ok())
        .filter(|path| path.is_file())
        .for_each(|file| {
            std::fs::copy(
                &file,
                out_dir
                    .join("ConcreteBindings")
                    .join(file.file_name().unwrap()),
            )
            .unwrap();
        });
}
