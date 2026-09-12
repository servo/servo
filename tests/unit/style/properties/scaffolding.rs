/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::env;
use std::fs::File;
use std::path::Path;

use serde_json::{self, Value};

// Skip this test when cross-running tests, since we won't have the file on the target device
// and the test itself should be target independant.
#[test]
#[cfg_attr(any(target_os = "android", target_env = "ohos"), ignore)]
fn properties_list_json() {
    // Four dotdots: /path/to/target(4)/debug(3)/build(2)/style_tests-*(1)/out
    // Do not ascend above the target dir, because it may not be called target
    // or even have a parent (see CARGO_TARGET_DIR).
    let target_dir = Path::new(env!("OUT_DIR"))
        .join("..")
        .join("..")
        .join("..")
        .join("..");
    let json = target_dir
        .join("doc")
        .join("stylo")
        .join("css-properties.json");

    let properties: Value = serde_json::from_reader(File::open(json).unwrap()).unwrap();
    let longhands = properties["longhands"].as_object().unwrap();
    assert!(longhands.len() > 100);
    assert!(longhands.get("margin-top").is_some());
    assert!(properties["shorthands"].get("margin").is_some());
}
