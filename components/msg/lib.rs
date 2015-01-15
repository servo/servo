/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(int_uint)]

#![deny(unused_imports)]
#![deny(unused_variables)]
#![allow(missing_copy_implementations)]

extern crate azure;
extern crate geom;
extern crate hyper;
extern crate layers;
extern crate serialize;
extern crate "util" as servo_util;
extern crate url;

#[cfg(target_os="macos")]
extern crate core_foundation;
#[cfg(target_os="macos")]
extern crate io_surface;

pub mod compositor_msg;
pub mod constellation_msg;
