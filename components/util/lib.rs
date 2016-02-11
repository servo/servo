/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(alloc)]
#![feature(box_syntax)]
#![feature(core_intrinsics)]
#![feature(custom_derive)]
#![cfg_attr(feature = "non-geckolib", feature(decode_utf16))]
#![feature(fnbox)]
#![feature(heap_api)]
#![feature(oom)]
#![feature(optin_builtin_traits)]
#![feature(plugin)]
#![feature(reflect_marker)]
#![feature(step_by)]
#![feature(step_trait)]
#![feature(zero_one)]

#![plugin(heapsize_plugin, plugins, serde_macros)]

extern crate alloc;
extern crate app_units;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate cssparser;
extern crate euclid;
extern crate getopts;
extern crate heapsize;
extern crate ipc_channel;
#[cfg(feature = "non-geckolib")]
extern crate js;
#[macro_use]
extern crate lazy_static;
extern crate libc;
#[macro_use]
extern crate log;
extern crate num as num_lib;
extern crate num_cpus;
extern crate rand;
extern crate rustc_serialize;
extern crate selectors;
extern crate serde;
extern crate smallvec;
extern crate string_cache;
extern crate url;
extern crate uuid;

use std::sync::Arc;

pub mod bezier;
pub mod cache;
pub mod cursor;
pub mod debug_utils;
pub mod deque;
pub mod geometry;
pub mod ipc;
pub mod linked_list;
pub mod logical_geometry;
#[cfg(feature = "non-geckolib")]
pub mod non_geckolib;
pub mod opts;
pub mod persistent_list;
pub mod prefs;
pub mod print_tree;
pub mod range;
pub mod resource_files;
pub mod str;
pub mod thread;
pub mod thread_state;
pub mod threadpool;
pub mod tid;
pub mod time;
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
