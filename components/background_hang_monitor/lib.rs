/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

extern crate backtrace;
extern crate ipc_channel;
#[cfg(any(target_os = "android", target_os = "linux"))]
#[macro_use]
extern crate lazy_static;
#[cfg(any(target_os = "android", target_os = "linux"))]
extern crate libc;
#[macro_use]
extern crate log;
#[cfg(target_os = "macos")]
extern crate mach;
extern crate msg;
#[macro_use]
extern crate servo_channel;

pub mod background_hang_monitor;
mod sampler;

pub use background_hang_monitor::*;
