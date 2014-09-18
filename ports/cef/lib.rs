/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(globs, macro_rules, phase, thread_local, link_args)]

#![allow(experimental, non_camel_case_types)]

#![feature(phase)]
#[phase(plugin, link)]
extern crate log;

extern crate rustuv;

extern crate "macros" as servo_macros;
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

extern crate "net" as servo_net;
extern crate "msg" as servo_msg;
extern crate "util" as servo_util;
extern crate style;
extern crate stb_image;

extern crate green;
extern crate native;
extern crate libc;
extern crate "url" as std_url;

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

