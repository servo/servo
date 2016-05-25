/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(box_syntax)]
#![feature(custom_derive)]
#![feature(plugin)]
#![feature(plugin)]
#![plugin(plugins)]

#![deny(unsafe_code)]
#![plugin(serde_macros)]

extern crate app_units;
extern crate azure;
extern crate compositing_traits;
extern crate euclid;
extern crate gfx;
extern crate gfx_traits;
extern crate gleam;
extern crate image;
extern crate ipc_channel;
extern crate layers;
extern crate layout_traits;
#[macro_use]
extern crate log;
extern crate msg;
extern crate net_traits;
#[macro_use]
extern crate profile_traits;
extern crate script_traits;
extern crate style_traits;
extern crate time;
extern crate url;
#[macro_use]
extern crate util;
extern crate webrender;
extern crate webrender_traits;

mod compositor;
mod compositor_layer;
pub mod compositor_thread;
mod delayed_composition;
mod surface_map;
mod touch;
pub mod windowing;
