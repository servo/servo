/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// For simd (currently x86_64/aarch64)
#![cfg_attr(any(target_arch = "x86_64", target_arch = "aarch64"), feature(convert))]
#![cfg_attr(any(target_os = "linux", target_os = "android"), feature(heap_api))]

#![feature(alloc)]
#![feature(box_syntax)]
#![feature(custom_attribute)]
#![feature(custom_derive)]
#![feature(hashmap_hasher)]
#![feature(mpsc_select)]
#![feature(plugin)]
#![feature(str_char)]
#![feature(unique)]
#![feature(vec_push_all)]

#![plugin(plugins)]
#![plugin(serde_macros)]

extern crate app_units;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate profile_traits;
#[macro_use]
extern crate util;
extern crate alloc;
extern crate azure;
extern crate canvas_traits;

// Mac OS-specific library dependencies
#[cfg(target_os = "macos")] extern crate core_foundation;
#[cfg(target_os = "macos")] extern crate core_graphics;
#[cfg(target_os = "macos")] extern crate core_text;

extern crate euclid;
extern crate fnv;

// Linux and Android-specific library dependencies
#[cfg(any(target_os = "linux", target_os = "android"))]
extern crate fontconfig;
#[cfg(any(target_os = "linux", target_os = "android"))]
extern crate freetype;

extern crate gfx_traits;

// Eventually we would like the shaper to be pluggable, as many operating systems have their own
// shapers. For now, however, this is a hard dependency.
extern crate harfbuzz_sys as harfbuzz;

extern crate ipc_channel;
extern crate layers;
extern crate libc;
extern crate msg;
extern crate net_traits;
extern crate rand;
extern crate rustc_serialize;
extern crate script_traits;
extern crate serde;

#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
extern crate simd;

extern crate skia;
extern crate smallvec;
extern crate string_cache;
extern crate style;
extern crate time;
extern crate unicode_script;
extern crate url;


pub use paint_context::PaintContext;

// Misc.
mod filters;

// Private painting modules
mod paint_context;

#[deny(unsafe_code)]
#[path = "display_list/mod.rs"]
pub mod display_list;

// Fonts
pub mod font;
pub mod font_cache_task;
pub mod font_context;
pub mod font_template;

pub mod paint_task;

// Platform-specific implementations.
#[path = "platform/mod.rs"]
pub mod platform;

// Text
#[path = "text/mod.rs"]
pub mod text;
