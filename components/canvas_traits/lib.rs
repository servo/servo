/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![crate_name = "canvas_traits"]
#![crate_type = "rlib"]
#![feature(nonzero)]

#![deny(unsafe_code)]

extern crate core;
extern crate cssparser;
extern crate euclid;
extern crate heapsize;
#[macro_use] extern crate heapsize_derive;
extern crate ipc_channel;
#[macro_use] extern crate lazy_static;
extern crate offscreen_gl_context;
#[macro_use] extern crate serde;
extern crate servo_config;
extern crate webrender_api;

pub mod canvas;
pub mod webgl;
mod webgl_channel;
