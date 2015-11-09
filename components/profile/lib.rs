/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(alloc_jemalloc)]
#![feature(box_syntax)]
#![feature(iter_arith)]
#![feature(slice_splits)]
#![feature(plugin)]
#![plugin(plugins)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate profile_traits;

extern crate alloc_jemalloc;
extern crate hbs_pow;
extern crate ipc_channel;
extern crate libc;
#[cfg(target_os = "linux")]
extern crate regex;
extern crate time as std_time;
#[cfg(target_os = "macos")]
extern crate task_info;
extern crate util;

mod heartbeats;
pub mod mem;
pub mod time;
