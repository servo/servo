/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(alloc)]
#![feature(box_syntax)]
#![feature(box_raw)]
#![feature(core_intrinsics)]
#![feature(custom_derive)]
#![feature(fnbox)]
#![feature(hashmap_hasher)]
#![feature(heap_api)]
#![feature(oom)]
#![feature(optin_builtin_traits)]
#![feature(path_ext)]
#![feature(plugin)]
#![feature(slice_splits)]
#![feature(step_by)]
#![feature(step_trait)]
#![feature(zero_one)]

#![plugin(serde_macros)]

#[macro_use] extern crate log;
#[macro_use] extern crate lazy_static;

extern crate azure;
extern crate alloc;
#[macro_use] extern crate bitflags;
#[macro_use] extern crate cssparser;
extern crate euclid;
extern crate getopts;
extern crate html5ever;
extern crate hyper;
extern crate ipc_channel;
extern crate js;
extern crate layers;
extern crate libc;
extern crate num as num_lib;
extern crate num_cpus;
extern crate rand;
extern crate rustc_serialize;
extern crate selectors;
extern crate serde;
extern crate smallvec;
extern crate string_cache;
extern crate url;

use std::sync::Arc;

pub mod bezier;
pub mod cache;
pub mod cursor;
pub mod debug_utils;
pub mod deque;
pub mod linked_list;
pub mod geometry;
pub mod ipc;
pub mod logical_geometry;
pub mod mem;
pub mod opts;
pub mod persistent_list;
pub mod prefs;
pub mod range;
pub mod resource_files;
pub mod str;
pub mod task;
pub mod tid;
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
