/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(core)]
#![feature(collections)]
#![feature(rustc_private)]

extern crate azure;
extern crate cssparser;
extern crate geom;
extern crate gfx;
extern crate util;
extern crate gleam;
extern crate num;
extern crate offscreen_gl_context;

#[macro_use]
extern crate log;

pub mod canvas_paint_task;
pub mod webgl_paint_task;
pub mod canvas_msg;
