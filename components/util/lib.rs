/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(alloc)]
#![feature(box_syntax)]
#![feature(collections)]
#![feature(core)]
#![feature(exit_status)]
#![feature(optin_builtin_traits)]
#![cfg_attr(not(target_os = "android"), feature(path_ext))]
#![feature(plugin)]
#![feature(rustc_private)]
#![feature(step_by)]
#![feature(step_trait)]
#![feature(std_misc)]
#![feature(zero_one)]

#![plugin(string_cache_plugin)]

#[macro_use] extern crate log;

extern crate azure;
extern crate alloc;
#[macro_use] extern crate bitflags;
#[macro_use] extern crate cssparser;
extern crate fnv as fnv_;
extern crate geom;
extern crate getopts;
extern crate layers;
extern crate libc;
extern crate num as num_lib;
extern crate num_cpus;
extern crate rand;
extern crate rustc_serialize;
extern crate selectors;
extern crate smallvec as smallvec_;
extern crate string_cache;
extern crate url;

use std::sync::Arc;

pub use fnv_ as fnv;
pub use smallvec_ as smallvec;

pub mod bezier;
pub mod cache;
pub mod cursor;
pub mod debug_utils;
pub mod deque;
pub mod linked_list;
pub mod geometry;
pub mod logical_geometry;
pub mod mem;
pub mod namespace;
pub mod opts;
pub mod persistent_list;
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
