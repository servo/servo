/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(arc_weak)]
#![cfg_attr(any(target_os = "linux", target_os = "android"), feature(box_raw))]
#![feature(box_syntax)]
#![feature(custom_attribute)]
#![feature(custom_derive)]
#![feature(hashmap_hasher)]
#![cfg_attr(any(target_os = "linux", target_os = "android"), feature(heap_api))]
#![feature(mpsc_select)]
#![feature(plugin)]
#![feature(str_char)]
#![feature(unique)]
#![feature(vec_push_all)]

#![plugin(plugins)]
#![plugin(serde_macros)]

#[macro_use]
extern crate log;
extern crate serde;

extern crate azure;
#[macro_use] extern crate bitflags;
extern crate fnv;
extern crate euclid;
extern crate ipc_channel;
#[macro_use]
extern crate lazy_static;
extern crate layers;
extern crate libc;
#[macro_use]
extern crate profile_traits;
extern crate script_traits;
extern crate rustc_serialize;
extern crate net_traits;
#[macro_use]
extern crate util;
extern crate msg;
extern crate rand;
extern crate smallvec;
extern crate string_cache;
extern crate style;
extern crate skia;
extern crate time;
extern crate url;

extern crate gfx_traits;
extern crate canvas_traits;

// Eventually we would like the shaper to be pluggable, as many operating systems have their own
// shapers. For now, however, this is a hard dependency.
extern crate harfbuzz;

// Linux and Android-specific library dependencies
#[cfg(any(target_os = "linux", target_os = "android"))]
extern crate fontconfig;

#[cfg(any(target_os = "linux", target_os = "android"))]
extern crate freetype;

// Mac OS-specific library dependencies
#[cfg(target_os = "macos")] extern crate core_foundation;
#[cfg(target_os = "macos")] extern crate core_graphics;
#[cfg(target_os = "macos")] extern crate core_text;

pub use paint_context::PaintContext;

// Private painting modules
mod paint_context;

#[path = "display_list/mod.rs"]
pub mod display_list;
pub mod paint_task;

// Fonts
pub mod font;
pub mod font_context;
pub mod font_cache_task;
pub mod font_template;

// Misc.
mod filters;

// Platform-specific implementations.
#[path = "platform/mod.rs"]
pub mod platform;

// Text
#[path = "text/mod.rs"]
pub mod text;
