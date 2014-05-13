/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![crate_id = "github.com/mozilla/servo#gfx:0.1"]
#![crate_type = "lib"]
#![crate_type = "dylib"]
#![crate_type = "rlib"]

#![feature(globs, macro_rules, phase)]

#![feature(phase)]
#[phase(syntax, link)]
extern crate log;

extern crate azure;
extern crate collections;
extern crate geom;
extern crate layers;
extern crate libc;
extern crate stb_image;
extern crate png;
#[phase(syntax)]
extern crate servo_macros = "macros";
extern crate servo_net = "net";
#[phase(syntax, link)]
extern crate servo_util = "util";
extern crate servo_msg = "msg";
extern crate style;
extern crate sync;

// Eventually we would like the shaper to be pluggable, as many operating systems have their own
// shapers. For now, however, this is a hard dependency.
extern crate harfbuzz;

// Linux and Android-specific library dependencies
#[cfg(target_os="linux")] #[cfg(target_os="android")] extern crate fontconfig;
#[cfg(target_os="linux")] #[cfg(target_os="android")] extern crate freetype;

// Mac OS-specific library dependencies
#[cfg(target_os="macos")] extern crate core_foundation;
#[cfg(target_os="macos")] extern crate core_graphics;
#[cfg(target_os="macos")] extern crate core_text;

pub use gfx_font = font;
pub use gfx_font_context = font_context;
pub use gfx_font_list = font_list;
pub use servo_gfx_font = font;
pub use servo_gfx_font_list = font_list;

// Private rendering modules
mod render_context;

// Rendering
pub mod color;
pub mod display_list;
pub mod render_task;

// Fonts
pub mod font;
pub mod font_context;
pub mod font_list;

// Misc.
mod buffer_map;

// Platform-specific implementations.
#[path="platform/mod.rs"]
pub mod platform;

// Text
#[path = "text/mod.rs"]
pub mod text;

