/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(plugin)]
#![plugin(plugins)]

#![deny(unsafe_code)]

extern crate azure;
extern crate canvas_traits;
extern crate core;
extern crate euclid;
extern crate gfx_traits;
extern crate gleam;
extern crate ipc_channel;
extern crate layers;
#[macro_use]
extern crate log;
extern crate num;
extern crate offscreen_gl_context;
extern crate util;
extern crate webrender_traits;

pub mod canvas_paint_thread;
mod premultiplytable;
pub mod webgl_paint_thread;
