/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(custom_derive, plugin)]
#![plugin(serde_macros, plugins)]

extern crate app_units;
#[macro_use]
extern crate bitflags;
extern crate azure;
extern crate canvas_traits;
#[cfg(target_os = "macos")]
extern crate core_foundation;
extern crate euclid;
extern crate hyper;
#[cfg(target_os = "macos")]
extern crate io_surface;
extern crate ipc_channel;
extern crate layers;
extern crate offscreen_gl_context;
extern crate png;
extern crate rustc_serialize;
extern crate serde;
extern crate style_traits;
extern crate url;
extern crate util;

pub mod compositor_msg;
pub mod constellation_msg;
pub mod webdriver_msg;
