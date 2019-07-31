/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde;

pub mod pref_util;
#[macro_use]
pub mod prefs;

pub mod basedir;
#[allow(unsafe_code)]
pub mod opts;

pub fn servo_version() -> String {
    let cargo_version = env!("CARGO_PKG_VERSION");
    let git_info = option_env!("GIT_INFO");
    match git_info {
        Some(info) => format!("Servo {}{}", cargo_version, info),
        None => format!("Servo {}", cargo_version),
    }
}
