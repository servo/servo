/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::path;

// build.rs
fn main() {
    println!("cargo:rustc-check-cfg=cfg(sdk_api_21)");
    println!("cargo:rustc-check-cfg=cfg(sdk_api_22)");
    println!("cargo:rustc-check-cfg=cfg(sdk_api_23)");

    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap();
    if target_env != "ohos" {
        return;
    }

    let sdk_path_name = std::env::var("OHOS_SDK_NATIVE").expect("OHOS_SDK_NATIVE must be set");

    let sdk_path = path::PathBuf::from(sdk_path_name);
    let meta_file_path = sdk_path.join("oh-uni-package.json");
    let meta_info = serde_json::from_str::<serde_json::Value>(
        &std::fs::read_to_string(&meta_file_path).expect("Failed to read oh-uni-package.json"),
    )
    .expect("Failed to parse oh-uni-package.json");
    let api_version_str = meta_info
        .get("apiVersion")
        .expect("Unable to find apiVersion in oh-uni-package.json")
        .as_str()
        .expect("apiVersion should be a string");
    let api_version = api_version_str
        .parse::<u32>()
        .expect("apiVersion should be a valid integer");
    let low_api_version = 21;
    if let 21.. = api_version {
        for version in low_api_version..=api_version {
            println!("cargo:rustc-cfg=sdk_api_{}", version);
        }
    }
    println!("cargo:warning=Detected API version: {:?}", api_version);
    println!("cargo:rerun-if-env-changed=OHOS_SDK_NATIVE");
}
