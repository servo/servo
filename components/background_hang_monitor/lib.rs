/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

#[macro_use]
extern crate crossbeam_channel;
#[macro_use]
extern crate log;

pub mod background_hang_monitor;
mod sampler;
#[cfg(any(target_os = "android", target_os = "linux"))]
mod sampler_linux;
#[cfg(target_os = "macos")]
mod sampler_mac;
#[cfg(target_os = "windows")]
mod sampler_windows;

pub use self::background_hang_monitor::*;
