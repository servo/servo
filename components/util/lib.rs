/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(box_syntax)]
#![feature(core_intrinsics)]
#![feature(custom_derive)]
#![cfg_attr(feature = "non-geckolib", feature(decode_utf16))]
#![feature(optin_builtin_traits)]
#![feature(plugin)]
#![feature(panic_handler)]
#![feature(reflect_marker)]
#![feature(step_by)]

#![plugin(heapsize_plugin, plugins, serde_macros)]

#![deny(unsafe_code)]

extern crate app_units;
#[macro_use]
extern crate bitflags;
extern crate deque;
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
extern crate num_cpus;
extern crate rand;
extern crate rustc_serialize;
extern crate serde;
extern crate smallvec;
extern crate string_cache;
extern crate url;

use std::sync::Arc;

pub mod cache;
#[allow(unsafe_code)]
pub mod debug_utils;
pub mod geometry;
#[allow(unsafe_code)]
pub mod ipc;
pub mod linked_list;
#[cfg(feature = "non-geckolib")]
#[allow(unsafe_code)]
pub mod non_geckolib;
#[allow(unsafe_code)]
pub mod opts;
#[allow(unsafe_code)]
pub mod prefs;
pub mod print_tree;
#[allow(unsafe_code)]
pub mod resource_files;
#[allow(unsafe_code)]
pub mod str;
pub mod thread;
pub mod thread_state;
pub mod tid;
pub mod time;
pub mod vec;
#[allow(unsafe_code)]
pub mod workqueue;

#[allow(unsafe_code)]
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
