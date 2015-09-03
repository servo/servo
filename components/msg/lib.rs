/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(custom_derive, plugin)]
#![plugin(serde_macros, plugins)]

extern crate azure;
#[macro_use] extern crate bitflags;
extern crate canvas_traits;
extern crate euclid;
extern crate hyper;
extern crate ipc_channel;
extern crate layers;
extern crate offscreen_gl_context;
extern crate png;
extern crate rustc_serialize;
extern crate serde;
extern crate util;
extern crate url;
extern crate style_traits;

#[cfg(target_os = "macos")]
extern crate core_foundation;
#[cfg(target_os = "macos")]
extern crate io_surface;

pub mod compositor_msg;
pub mod constellation_msg;
pub mod webdriver_msg;
