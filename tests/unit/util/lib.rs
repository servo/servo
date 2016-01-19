/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![cfg_attr(test, feature(plugin, custom_derive, heap_api))]
#![cfg_attr(test, plugin(plugins))]
#![feature(alloc)]
#![feature(plugin)]
#![plugin(serde_macros)]

extern crate alloc;
extern crate app_units;
extern crate euclid;
extern crate ipc_channel;
extern crate libc;
extern crate serde;
extern crate util;

#[cfg(test)] mod cache;
#[cfg(test)] mod logical_geometry;
#[cfg(test)] mod thread;
#[cfg(test)] mod vec;
#[cfg(test)] mod mem;
#[cfg(test)] mod str;
#[cfg(test)] mod opts;
#[cfg(test)] mod ipc;
