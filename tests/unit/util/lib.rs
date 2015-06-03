/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(plugin, custom_derive, custom_attributes)]
#![plugin(plugins)]
extern crate util;
extern crate geom;

#[cfg(test)] mod cache;
#[cfg(test)] mod logical_geometry;
#[cfg(test)] mod task;
#[cfg(test)] mod vec;
#[cfg(test)] mod mem;
