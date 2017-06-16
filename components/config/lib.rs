/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

#[cfg(target_os = "android")]
extern crate android_injected_glue;
extern crate euclid;
extern crate getopts;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;
extern crate num_cpus;
extern crate rustc_serialize;
#[macro_use] extern crate serde;
extern crate servo_geometry;
extern crate servo_url;
extern crate url;
#[cfg(all(unix, not(target_os = "macos"), not(target_os = "ios"), not(target_os = "android")))]
extern crate xdg;

pub mod basedir;
#[allow(unsafe_code)] pub mod opts;
pub mod prefs;
pub mod resource_files;

pub fn servo_version() -> String {
    let cargo_version = env!("CARGO_PKG_VERSION");
    let git_info = option_env!("GIT_INFO");
    match git_info {
        Some(info) => format!("Servo {}{}", cargo_version, info),
        None => format!("Servo {}", cargo_version),
    }
}
