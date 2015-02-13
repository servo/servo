/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(alloc)]
#![feature(box_syntax)]
#![feature(collections)]
#![feature(core)]
#![feature(hash)]
#![feature(int_uint)]
#![cfg_attr(any(target_os="linux", target_os = "android"), feature(io))]
#![feature(libc)]
#![cfg_attr(any(target_os="linux", target_os = "android"), feature(path))]
#![feature(plugin)]
#![feature(rustc_private)]
#![feature(std_misc)]
#![feature(unicode)]
#![feature(unsafe_destructor)]

#![allow(missing_copy_implementations)]

#[macro_use]
extern crate log;

extern crate azure;
#[macro_use] extern crate bitflags;
extern crate collections;
extern crate geom;
extern crate layers;
extern crate libc;
extern crate stb_image;
extern crate png;
extern crate script_traits;
extern crate "serialize" as rustc_serialize;
extern crate unicode;
#[no_link] #[plugin]
extern crate "plugins" as servo_plugins;
extern crate net;
#[macro_use]
extern crate util;
extern crate msg;
extern crate style;
extern crate skia;
extern crate time;
extern crate url;

// Eventually we would like the shaper to be pluggable, as many operating systems have their own
// shapers. For now, however, this is a hard dependency.
extern crate harfbuzz;

// Linux and Android-specific library dependencies
#[cfg(any(target_os="linux", target_os = "android"))]
extern crate fontconfig;

#[cfg(any(target_os="linux", target_os = "android"))]
extern crate freetype;

// Mac OS-specific library dependencies
#[cfg(target_os="macos")] extern crate core_foundation;
#[cfg(target_os="macos")] extern crate core_graphics;
#[cfg(target_os="macos")] extern crate core_text;

pub use paint_context::PaintContext;

// Private painting modules
mod paint_context;

// Painting
pub mod color;
#[path="display_list/mod.rs"]
pub mod display_list;
pub mod paint_task;

// Fonts
pub mod font;
pub mod font_context;
pub mod font_cache_task;
pub mod font_template;

// Misc.
mod buffer_map;
mod filters;

// Platform-specific implementations.
#[path="platform/mod.rs"]
pub mod platform;

// Text
#[path = "text/mod.rs"]
pub mod text;
