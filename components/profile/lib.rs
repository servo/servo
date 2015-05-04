/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(box_syntax)]
#![feature(collections)]
#![feature(core)]
#![cfg_attr(target_os="linux", feature(page_size))]
#![feature(rustc_private)]

#[macro_use] extern crate log;

extern crate collections;
extern crate libc;
#[macro_use]
extern crate profile_traits;
#[cfg(target_os="linux")]
extern crate regex;
#[cfg(target_os="macos")]
extern crate task_info;
extern crate time as std_time;
extern crate util;

pub mod mem;
pub mod time;
