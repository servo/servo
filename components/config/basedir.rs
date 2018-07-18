/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Contains routines for retrieving default config directories.
//! For linux based platforms, it uses the XDG base directory spec but provides
//! similar abstractions for non-linux platforms.

#[cfg(target_os = "android")]
use android_injected_glue;
#[cfg(target_os = "android")]
use std::ffi::CStr;
use std::path::PathBuf;

#[cfg(all(unix, not(target_os = "macos"), not(target_os = "ios"), not(target_os = "android")))]
pub fn default_config_dir() -> PathBuf {
    let mut config_dir = ::dirs::config_dir().unwrap();
    config_dir.push("servo");
    config_dir.push("default");
    config_dir
}

#[cfg(target_os = "android")]
#[allow(unsafe_code)]
pub fn default_config_dir() -> PathBuf {
    let dir = unsafe {
        CStr::from_ptr((*android_injected_glue::get_app().activity).externalDataPath)
    };
    PathBuf::from(dir.to_str().unwrap())
}

#[cfg(target_os = "macos")]
pub fn default_config_dir() -> PathBuf {
    // FIXME: use `config_dir()` ($HOME/Library/Preferences)
    // instead of `data_dir()` ($HOME/Library/Application Support) ?
    let mut config_dir = ::dirs::data_dir().unwrap();
    config_dir.push("Servo");
    config_dir
}

#[cfg(target_os = "windows")]
pub fn default_config_dir() -> PathBuf {
    let mut config_dir = ::dirs::config_dir().unwrap();
    config_dir.push("Servo");
    config_dir
}
