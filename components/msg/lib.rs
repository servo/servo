/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate azure;
#[macro_use] extern crate bitflags;
extern crate euclid;
extern crate hyper;
extern crate layers;
extern crate png;
extern crate rustc_serialize;
extern crate util;
extern crate url;
extern crate style;

#[cfg(target_os="macos")]
extern crate core_foundation;
#[cfg(target_os="macos")]
extern crate io_surface;

pub mod compositor_msg;
pub mod constellation_msg;
pub mod webdriver_msg;
