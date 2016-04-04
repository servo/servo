/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Contains routines for retrieving default config directories.
//! For linux, it uses the XDG base directory spec and provides
//! similar abstractions for non-linux platforms.

extern crate xdg;

use std::io::Error;
use std::path::PathBuf;

/// This function bootstraps all Servo specific directories
/// on linux platform following the XDG base directory spec.
/// during default command line args setup by opts.rs

#[cfg(all(unix, not(target_os = "macos"), not(target_os = "ios"), not(target_os = "windows")))]
pub fn bootstrap_default_dirs() -> Result<(), Error> {
    let xdg_dirs = xdg::BaseDirectories::with_profile("servo", "default").unwrap();
    let config_dir = default_config_dir().unwrap();
    let data_dir = default_data_dir().unwrap();
    let cache_dir = default_cache_dir().unwrap();
    xdg_dirs.create_config_directory(config_dir).unwrap();
    xdg_dirs.create_data_directory(data_dir).unwrap();
    xdg_dirs.create_cache_directory(cache_dir).unwrap();
    Ok(())
}

#[cfg(all(unix, not(target_os = "macos"), not(target_os = "ios"), not(target_os = "windows")))]
pub fn default_config_dir() -> Option<PathBuf> {
    let xdg_dirs = xdg::BaseDirectories::with_profile("servo", "default").unwrap();
    let config_dir = xdg_dirs.get_config_home();
    Some(config_dir)
}

#[cfg(all(unix, not(target_os = "macos"), not(target_os = "ios"), not(target_os = "windows")))]
pub fn default_data_dir() -> Option<PathBuf> {
    let xdg_dirs = xdg::BaseDirectories::with_profile("servo", "default").unwrap();
    let data_dir = xdg_dirs.get_data_home();
    Some(data_dir)
}

#[cfg(all(unix, not(target_os = "macos"), not(target_os = "ios"), not(target_os = "windows")))]
pub fn default_cache_dir() -> Option<PathBuf> {
    let xdg_dirs = xdg::BaseDirectories::with_profile("servo", "default").unwrap();
    let cache_dir = xdg_dirs.get_cache_home();
    Some(cache_dir)
}

#[cfg(target_os = "macos")]
pub fn bootstrap_default_dirs() -> Result<(), Error> {
    let config_dir = default_config_dir().unwrap();
    match fs::create_dir_all(config_dir) {
        Ok(_) => Ok(()),
        Err(reason) => Error(reason)
    }
}

#[cfg(target_os = "macos")]
pub fn default_config_dir() -> Option<PathBuf> {
    let mut config_dir = env::home_dir().unwrap();
    config_dir.push("Library");
    config_dir.push("Application Support");
    config_dir.push("Servo");
    config_dir
}

#[cfg(target_os = "windows")]
pub fn bootstrap_default_dirs() -> Result<(), Error> {
    let config_dir = default_config_dir().unwrap();
    match fs::create_dir_all(config_dir) {
        Ok(_) => Ok(()),
        Err(reason) => Error(reason)
    }
}

#[cfg(target_os = "windows")]
pub fn default_config_dir() -> Option<PathBuf> {
    let mut config_dir = match env::var("APPDATA") {
        Ok(appdata_path) => PathBuf::new(appdata_path),
        Err(_) => { let mut dir = env::home_dir().unwrap();
            dir.push("Appdata");
            dir.push("Roaming");
            dir
        }
    };
    config_dir.push("Servo");
    config_dir
}
