/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(hash)]
#![feature(int_uint)]
#![feature(rustc_private)]

#![allow(missing_copy_implementations)]

extern crate azure;
#[macro_use] extern crate bitflags;
extern crate geom;
extern crate hyper;
extern crate layers;
extern crate serialize;
extern crate util;
extern crate url;

#[cfg(target_os="macos")]
extern crate core_foundation;
#[cfg(target_os="macos")]
extern crate io_surface;

pub mod compositor_msg;
pub mod constellation_msg;
