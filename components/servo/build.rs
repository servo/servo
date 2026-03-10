/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::path::Path;
use std::{env, fs};

fn main() {
    if cfg!(feature = "media-gstreamer") {
        println!("cargo::rerun-if-changed=gstreamer_plugin_lists");
        write_plugin_list();
    }
}

fn write_plugin_list() {
    // https://doc.rust-lang.org/reference/conditional-compilation.html#target_os
    let os = env::var("CARGO_CFG_TARGET_OS")
        .expect("CARGO_CFG_TARGET_OS not set. Third-party build system?");
    let plugins = if os == "macos" {
        macos_plugins()
    } else if os == "windows" {
        windows_plugins()
    } else {
        return;
    };

    let formatted_plugins = plugins
        .into_iter()
        .map(|plugin| format!("\"{plugin}\""))
        .collect::<Vec<_>>()
        .join(",\n");
    let output = format!(
        "/* This is a generated file. Do not modify. */\n\npub(crate) static GSTREAMER_PLUGINS: &[&str] = &[\n{formatted_plugins}\n];\n"
    );
    let path = Path::new(&env::var_os("OUT_DIR").unwrap()).join("gstreamer_plugins.rs");
    fs::write(path, output).expect("Failed to write gstreamer_plugins.rs to OUT_DIR");
}

fn load_plugin_libraries_from_file(file_name: &str) -> Vec<String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("gstreamer_plugin_lists")
        .join(file_name);
    let content = fs::read_to_string(path).unwrap_or_else(|e| {
        panic!("Failed to read GStreamer plugin library list from {file_name}: {e}")
    });
    content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(String::from)
        .collect()
}

fn windows_plugins() -> Vec<String> {
    let mut libraries = load_plugin_libraries_from_file("common.txt");
    libraries.extend(load_plugin_libraries_from_file("windows.txt"));
    libraries
        .into_iter()
        .map(|library| format!("{library}.dll"))
        .collect()
}

fn macos_plugins() -> Vec<String> {
    let mut libraries = load_plugin_libraries_from_file("common.txt");
    libraries.extend(load_plugin_libraries_from_file("macos.txt"));
    libraries
        .into_iter()
        .map(|library| format!("lib{library}.dylib"))
        .collect()
}
