/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(custom_derive)]
#![feature(plugin)]
#![plugin(heapsize_plugin, serde_macros)]

#[macro_use]
extern crate heapsize;
extern crate ipc_channel;
#[macro_use]
extern crate serde;
extern crate url;

pub mod storage_thread;
