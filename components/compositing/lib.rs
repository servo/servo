/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![comment = "The Servo Parallel Browser Project"]
#![license = "MPL"]

#![feature(globs, phase, macro_rules)]

#![deny(unused_imports, unused_variable)]

#[phase(plugin, link)]
extern crate log;

extern crate debug;

extern crate alert;
extern crate azure;
extern crate devtools_traits;
extern crate geom;
extern crate gfx;
#[cfg(not(target_os="android"))]
extern crate glfw;
#[cfg(target_os="android")]
extern crate glut;
extern crate layers;
extern crate layout_traits;
extern crate opengles;
extern crate png;
extern crate script_traits;
extern crate "msg" as servo_msg;
extern crate "net" as servo_net;
#[phase(plugin, link)]
extern crate "util" as servo_util;

extern crate libc;
extern crate time;
extern crate url;

#[cfg(target_os="macos")]
extern crate core_graphics;
#[cfg(target_os="macos")]
extern crate core_text;

pub use compositor_task::{CompositorChan, CompositorTask};
pub use constellation::Constellation;

pub mod compositor_task;

mod compositor_data;
mod events;

mod compositor;
mod headless;

pub mod pipeline;
pub mod constellation;

mod windowing;

#[path="platform/mod.rs"]
pub mod platform;
