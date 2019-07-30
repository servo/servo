/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use serde_json::{self, Value};
use std::env;
use std::fs::File;
use std::path::Path;

#[test]
fn properties_list_json() {
    let top = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("..")
        .join("..")
        .join("..");
    let json = top
        .join("target")
        .join("doc")
        .join("servo")
        .join("css-properties.json");

    let properties: Value = serde_json::from_reader(File::open(json).unwrap()).unwrap();
    let longhands = properties["longhands"].as_object().unwrap();
    assert!(longhands.len() > 100);
    assert!(longhands.get("margin-top").is_some());
    assert!(properties["shorthands"].get("margin").is_some());
}
