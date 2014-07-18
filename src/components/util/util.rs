/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![crate_id = "github.com/mozilla/servo#util:0.1"]
#![crate_type = "lib"]
#![crate_type = "dylib"]
#![crate_type = "rlib"]

#![feature(macro_rules)]

#![feature(phase)]
#[phase(plugin, link)]
extern crate log;

extern crate debug;
extern crate alloc;
extern crate azure;
extern crate collections;
extern crate geom;
extern crate getopts;
extern crate libc;
extern crate native;
extern crate rand;
extern crate rustrt;
extern crate serialize;
extern crate sync;
#[cfg(target_os="macos")]
extern crate task_info;
extern crate std_time = "time";
extern crate std_url = "url";
extern crate string_cache;

pub mod atom;
pub mod cache;
pub mod debug_utils;
pub mod geometry;
pub mod logical_geometry;
pub mod memory;
pub mod namespace;
pub mod opts;
pub mod range;
pub mod smallvec;
pub mod sort;
pub mod str;
pub mod task;
pub mod time;
pub mod url;
pub mod vec;
pub mod workqueue;
