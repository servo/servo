/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

extern crate azure;
extern crate canvas_traits;
extern crate compositing;
extern crate cssparser;
extern crate euclid;
extern crate fnv;
extern crate gleam;
extern crate ipc_channel;
#[macro_use] extern crate log;
extern crate num_traits;
extern crate offscreen_gl_context;
extern crate servo_config;
extern crate webrender;
extern crate webrender_api;

pub mod canvas_paint_thread;
pub mod gl_context;
mod webgl_mode;
pub mod webgl_thread;
