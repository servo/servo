/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

extern crate backtrace;
extern crate ipc_channel;
#[cfg(any(target_os = "android", target_os = "linux"))]
#[macro_use] extern crate lazy_static;
extern crate libc;
#[macro_use]
extern crate log;
#[cfg(target_os = "macos")]
extern crate mach;
#[macro_use]
extern crate malloc_size_of;
#[macro_use]
extern crate malloc_size_of_derive;
#[macro_use]
extern crate servo_channel;
extern crate webrender_api;

mod background_hang_monitor;
pub mod constellation_msg;
