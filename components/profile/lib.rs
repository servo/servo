/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

#[allow(unused_extern_crates)]
extern crate heartbeats_simple;
extern crate influent;
extern crate ipc_channel;
#[cfg(all(feature = "unstable", not(target_os = "windows")))]
extern crate jemalloc_sys;
#[cfg(not(target_os = "windows"))]
extern crate libc;
#[macro_use]
extern crate log;
#[macro_use]
extern crate profile_traits;
#[cfg(target_os = "linux")]
extern crate regex;
#[macro_use]
extern crate serde;
extern crate serde_json;
extern crate servo_config;
#[cfg(target_os = "macos")]
extern crate task_info;
extern crate time as std_time;
extern crate tokio;

#[allow(unsafe_code)]
mod heartbeats;
#[allow(unsafe_code)]
pub mod mem;
pub mod time;
pub mod trace_dump;
