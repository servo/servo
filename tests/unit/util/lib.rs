/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![cfg_attr(test, feature(plugin, custom_derive))]
#![cfg_attr(test, plugin(plugins))]
#![feature(alloc)]

extern crate alloc;
extern crate app_units;
extern crate euclid;
extern crate libc;
extern crate util;

#[cfg(test)] mod cache;
#[cfg(test)] mod opts;
#[cfg(test)] mod str;
#[cfg(test)] mod thread;
#[cfg(test)] mod prefs;
