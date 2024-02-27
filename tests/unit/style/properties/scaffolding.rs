/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::env;
use std::fs::File;
use std::path::Path;

use serde_json::{self, Value};

#[test]
fn properties_list_json() {
    let top = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("..")
        .join("..")
        .join("..");
    let json = top
        .join("target")
        .join("doc")
        .join("stylo")
        .join("css-properties.json");

    #[cfg(windows)]
    {
        let mut paths = vec![];
        dump_tree(&mut paths, top.join("target"));
        paths.sort();
        panic!("{}", paths.join("\n"));
    }

    let properties: Value = serde_json::from_reader(File::open(json).unwrap()).unwrap();
    let longhands = properties["longhands"].as_object().unwrap();
    assert!(longhands.len() > 100);
    assert!(longhands.get("margin-top").is_some());
    assert!(properties["shorthands"].get("margin").is_some());
}

#[cfg(windows)]
fn dump_tree(result: &mut Vec<String>, path: impl AsRef<Path>) {
    let path = path.as_ref();
    if path.is_dir() {
        for entry in std::fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            dump_tree(result, path);
        }
        return;
    }
    if let Some(path) = path.to_str() {
        if "jemalloc mozjs incremental lsp fingerprint"
            .split(" ")
            .any(|word| path.contains(word))
        {
            return;
        }
        if ".o .h .d .rs .rmeta .rlib"
            .split(" ")
            .any(|suffix| path.ends_with(suffix))
        {
            return;
        }
        result.push(path.to_owned());
    }
}
