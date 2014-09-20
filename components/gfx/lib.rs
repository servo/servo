/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(globs, macro_rules, phase, unsafe_destructor)]

#![deny(unused_imports, unused_variable)]

#![feature(phase)]
#[phase(plugin, link)]
extern crate log;

extern crate debug;
extern crate azure;
extern crate collections;
extern crate geom;
extern crate layers;
extern crate libc;
extern crate native;
extern crate rustrt;
extern crate stb_image;
extern crate png;
extern crate serialize;
#[phase(plugin)]
extern crate "plugins" as servo_plugins;
extern crate "net" as servo_net;
#[phase(plugin, link)]
extern crate "util" as servo_util;
extern crate "msg" as servo_msg;
extern crate style;
extern crate sync;
extern crate url;

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

pub use render_context::RenderContext;

// Private rendering modules
mod render_context;

// Rendering
pub mod color;
#[path="display_list/mod.rs"]
pub mod display_list;
pub mod render_task;

// Fonts
pub mod font;
pub mod font_context;
pub mod font_cache_task;
pub mod font_template;

// Misc.
mod buffer_map;

// Platform-specific implementations.
#[path="platform/mod.rs"]
pub mod platform;

// Text
#[path = "text/mod.rs"]
pub mod text;

