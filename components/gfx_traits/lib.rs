/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![crate_name = "gfx_traits"]
#![crate_type = "rlib"]

extern crate azure;
extern crate layers;
extern crate msg;

pub mod color;
mod paint_listener;

pub use paint_listener::PaintListener;
