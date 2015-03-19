/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(alloc)]
#![feature(box_syntax)]
#![feature(collections)]
#![feature(core)]
#![feature(exit_status)]
#![feature(hash)]
#![feature(int_uint)]
#![feature(io)]
#![feature(optin_builtin_traits)]
#![feature(old_io)]
#![feature(path)]
#![feature(path_ext)]
#![feature(plugin)]
#![feature(rustc_private)]
#![feature(std_misc)]
#![feature(unicode)]
#![feature(unsafe_destructor)]

#![plugin(string_cache_plugin)]

#[macro_use] extern crate log;

extern crate alloc;
#[macro_use] extern crate bitflags;
extern crate collections;
extern crate cssparser;
extern crate geom;
extern crate getopts;
extern crate layers;
extern crate libc;
#[no_link] #[macro_use] extern crate cssparser;
extern crate rand;
#[cfg(target_os="linux")]
extern crate regex;
extern crate "rustc-serialize" as rustc_serialize;
#[cfg(target_os="macos")]
extern crate task_info;
extern crate "time" as std_time;
extern crate text_writer;
extern crate selectors;
extern crate string_cache;
extern crate unicode;
extern crate url;

extern crate lazy_static;

pub use selectors::smallvec;

use std::sync::Arc;

pub mod cache;
pub mod cursor;
pub mod debug_utils;
pub mod deque;
pub mod linked_list;
pub mod fnv;
pub mod geometry;
pub mod logical_geometry;
pub mod memory;
pub mod namespace;
pub mod opts;
pub mod persistent_list;
pub mod range;
pub mod resource_files;
pub mod str;
pub mod task;
pub mod tid;
pub mod time;
pub mod taskpool;
pub mod task_state;
pub mod vec;
pub mod workqueue;

pub fn breakpoint() {
    unsafe { ::std::intrinsics::breakpoint() };
}

// Workaround for lack of `ptr_eq` on Arcs...
#[inline]
pub fn arc_ptr_eq<T: 'static + Send + Sync>(a: &Arc<T>, b: &Arc<T>) -> bool {
    let a: &T = &**a;
    let b: &T = &**b;
    (a as *const T) == (b as *const T)
}
