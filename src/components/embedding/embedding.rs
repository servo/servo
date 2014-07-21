/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![crate_name = "embedding"]
#![crate_type = "lib"]
#![crate_type = "rlib"]
#![crate_type = "dylib"]

#![feature(globs, macro_rules, phase, thread_local)]

#![feature(phase)]
#[phase(plugin, link)]
extern crate log;

extern crate rustuv;

extern crate servo_macros = "macros";
extern crate servo;

extern crate azure;
extern crate geom;
extern crate gfx;
#[cfg(not(target_os="android"))]
extern crate glfw;
#[cfg(target_os="android")]
extern crate glut;
extern crate js;
extern crate layers;
extern crate opengles;
extern crate png;
extern crate script;

extern crate servo_net = "net";
extern crate servo_msg = "msg";
extern crate servo_util = "util";
extern crate style;
extern crate sharegl;
extern crate stb_image;

extern crate green;
extern crate native;
extern crate libc;
extern crate std_url = "url";

#[cfg(target_os="macos")]
extern crate core_graphics;
#[cfg(target_os="macos")]
extern crate core_text;

pub mod browser;
pub mod command_line;
pub mod core;
pub mod eutil;
#[cfg(target_os="linux")] #[cfg(target_os="macos")]
pub mod mem;
pub mod request;
pub mod string;
pub mod task;
pub mod types;
pub mod urlrequest;

