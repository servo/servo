/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(core)]
#![feature(nonzero)]
#![feature(slice_bytes)]
#![feature(vec_push_all)]

extern crate core;
extern crate canvas_traits;
extern crate azure;
extern crate cssparser;
extern crate euclid;
extern crate gfx_traits;
extern crate util;
extern crate gleam;
extern crate num;
extern crate layers;
extern crate offscreen_gl_context;
extern crate ipc_channel;

#[macro_use]
extern crate log;

pub mod canvas_paint_task;
pub mod webgl_paint_task;
