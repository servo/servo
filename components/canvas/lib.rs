/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate log;

mod raqote_backend;

pub use webgl_mode::WebGLComm;
pub use webgl_thread::SurfaceProvider;
pub use webgl_thread::SurfaceProviders;
pub use webgl_thread::WebGlExecutor;

pub mod canvas_data;
pub mod canvas_paint_thread;
mod webgl_limits;
mod webgl_mode;
pub mod webgl_thread;
