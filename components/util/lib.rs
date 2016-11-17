/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![cfg_attr(feature = "servo", feature(nonzero))]
#![cfg_attr(feature = "servo", feature(plugin))]
#![cfg_attr(feature = "servo", feature(proc_macro))]
#![cfg_attr(feature = "servo", plugin(plugins))]

#![deny(unsafe_code)]

extern crate app_units;
#[allow(unused_extern_crates)] #[macro_use] extern crate bitflags;
extern crate core;
#[macro_use] extern crate euclid;
extern crate getopts;
#[macro_use] extern crate heapsize;
#[allow(unused_extern_crates)] #[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;
extern crate num_cpus;
extern crate rustc_serialize;
#[cfg(feature = "servo")] extern crate serde;
#[cfg(feature = "servo")] #[macro_use] extern crate serde_derive;
extern crate servo_url;
extern crate url;
#[cfg(all(unix, not(target_os = "macos"), not(target_os = "ios"), not(target_os = "android")))]
extern crate xdg;

pub mod basedir;
pub mod geometry;
#[allow(unsafe_code)] pub mod opts;
pub mod prefs;
#[cfg(feature = "servo")] pub mod remutex;
pub mod resource_files;
pub mod thread;

pub fn servo_version() -> String {
    let cargo_version = env!("CARGO_PKG_VERSION");
    let git_info = option_env!("GIT_INFO");
    match git_info {
        Some(info) => format!("Servo {}{}", cargo_version, info),
        None => format!("Servo {}", cargo_version),
    }
}

pub fn clamp<T: Ord>(lo: T, mid: T, hi: T) -> T {
    if mid < lo {
        lo
    } else if mid > hi {
        hi
    } else {
        mid
    }
}
