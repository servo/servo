/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Contains routines for retrieving default config directories.
//! For linux based platforms, it uses the XDG base directory spec but provides
//! similar abstractions for non-linux platforms.

use std::path::PathBuf;

#[cfg(all(
    unix,
    not(target_os = "macos"),
    not(target_os = "ios"),
    not(target_os = "android")
))]
pub fn default_config_dir() -> Option<PathBuf> {
    let mut config_dir = ::dirs_next::config_dir().unwrap();
    config_dir.push("servo");
    config_dir.push("default");
    Some(config_dir)
}

#[cfg(target_os = "android")]
pub fn default_config_dir() -> Option<PathBuf> {
    None
}

#[cfg(target_os = "macos")]
pub fn default_config_dir() -> Option<PathBuf> {
    // FIXME: use `config_dir()` ($HOME/Library/Preferences)
    // instead of `data_dir()` ($HOME/Library/Application Support) ?
    let mut config_dir = ::dirs_next::data_dir().unwrap();
    config_dir.push("Servo");
    Some(config_dir)
}

#[cfg(target_os = "windows")]
pub fn default_config_dir() -> Option<PathBuf> {
    let mut config_dir = ::dirs_next::config_dir().unwrap();
    config_dir.push("Servo");
    Some(config_dir)
}
