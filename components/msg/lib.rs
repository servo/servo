/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate azure;
#[macro_use] extern crate bitflags;
extern crate geom;
extern crate hyper;
extern crate layers;
extern crate util;
extern crate url;
extern crate webdriver_traits;

#[cfg(target_os="macos")]
extern crate core_foundation;
#[cfg(target_os="macos")]
extern crate io_surface;

pub mod compositor_msg;
pub mod constellation_msg;
