/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![cfg_attr(not(target_os = "windows"), feature(alloc_jemalloc))]
#![feature(box_syntax)]
#![feature(iter_arith)]
#![feature(plugin)]
#![plugin(plugins)]
#![feature(custom_derive)]
#![plugin(serde_macros)]

#![deny(unsafe_code)]

#[allow(unused_extern_crates)]
#[cfg(not(target_os = "windows"))]
extern crate alloc_jemalloc;
extern crate hbs_pow;
extern crate ipc_channel;
extern crate libc;
#[macro_use]
extern crate log;
#[macro_use]
extern crate profile_traits;
#[cfg(target_os = "linux")]
extern crate regex;
extern crate serde;
extern crate serde_json;
#[cfg(target_os = "macos")]
extern crate task_info;
extern crate time as std_time;
extern crate util;

#[allow(unsafe_code)]
mod heartbeats;
#[allow(unsafe_code)]
pub mod mem;
pub mod time;
pub mod trace_dump;
