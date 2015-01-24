/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(box_syntax, plugin)]
#![feature(int_uint)]

#![deny(unused_imports)]
#![deny(unused_variables)]
#![allow(missing_copy_implementations)]
#![allow(unstable)]

#[macro_use]
extern crate log;

extern crate azure;
extern crate devtools_traits;
extern crate geom;
extern crate gfx;
extern crate layers;
extern crate layout_traits;
extern crate png;
extern crate script_traits;
extern crate "msg" as servo_msg;
extern crate "net" as servo_net;
#[macro_use]
#[plugin]
extern crate "util" as servo_util;
extern crate gleam;

extern crate libc;
extern crate time;
extern crate url;

#[cfg(target_os="macos")]
extern crate core_graphics;
#[cfg(target_os="macos")]
extern crate core_text;

pub use compositor_task::{CompositorEventListener, CompositorProxy, CompositorTask};
pub use constellation::Constellation;

pub mod compositor_task;

mod compositor_layer;
mod scrolling;

mod compositor;
mod headless;

pub mod pipeline;
pub mod constellation;

pub mod windowing;
