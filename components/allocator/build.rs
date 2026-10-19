/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

fn main() {
    let use_jemalloc = std::env::var("CARGO_FEATURE_USE_JEMALLOC").is_ok();
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap();
    let any_sanitizer_enabled = std::env::var("CARGO_CFG_SANITIZE").is_ok();

    println!("cargo::rustc-check-cfg=cfg(jemalloc_enabled)");
    if use_jemalloc && !any_sanitizer_enabled && target_os != "windows" && target_env != "ohos" {
        println!("cargo::rustc-cfg=jemalloc_enabled");
    }
}
