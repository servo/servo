/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// For simd (currently x86_64/aarch64)
#![cfg_attr(any(target_os = "linux", target_os = "android", target_os = "windows"), feature(heap_api))]

#![feature(alloc)]
#![feature(box_syntax)]
#![feature(custom_attribute)]
#![feature(custom_derive)]
#![feature(mpsc_select)]
#![feature(plugin)]
#![feature(range_contains)]
#![feature(unique)]

#![plugin(heapsize_plugin)]
#![plugin(plugins)]
#![plugin(serde_macros)]

#![deny(unsafe_code)]

extern crate alloc;
extern crate app_units;
extern crate azure;
#[allow(unused_extern_crates)]
#[macro_use]
extern crate bitflags;

// Mac OS-specific library dependencies
#[cfg(target_os = "macos")] extern crate byteorder;
#[cfg(target_os = "macos")] extern crate core_foundation;
#[cfg(target_os = "macos")] extern crate core_graphics;
#[cfg(target_os = "macos")] extern crate core_text;

extern crate euclid;
extern crate fnv;

// Platforms that use Freetype/Fontconfig library dependencies
#[cfg(any(target_os = "linux", target_os = "android", all(target_os = "windows", target_env = "gnu")))]
extern crate fontconfig;
#[cfg(any(target_os = "linux", target_os = "android", all(target_os = "windows", target_env = "gnu")))]
extern crate freetype;

extern crate gfx_traits;

// Eventually we would like the shaper to be pluggable, as many operating systems have their own
// shapers. For now, however, this is a hard dependency.
extern crate harfbuzz_sys as harfbuzz;

extern crate heapsize;
extern crate ipc_channel;
extern crate layers;
#[allow(unused_extern_crates)]
#[macro_use]
extern crate lazy_static;
extern crate libc;
#[macro_use]
extern crate log;
extern crate mime;
extern crate msg;
extern crate net_traits;
#[macro_use]
extern crate profile_traits;
extern crate rand;
#[macro_use]
extern crate range;
extern crate rustc_serialize;
extern crate serde;

#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
extern crate simd;

extern crate smallvec;
#[macro_use]
extern crate string_cache;
extern crate style;
extern crate style_traits;
extern crate time;
extern crate unicode_script;
extern crate url;
extern crate util;
extern crate webrender_traits;
extern crate xi_unicode;

pub use paint_context::PaintContext;

// Misc.
mod filters;

// Private painting modules
mod paint_context;

#[deny(unsafe_code)]
pub mod display_list;

// Fonts
#[macro_use] pub mod font;
pub mod font_cache_thread;
pub mod font_context;
pub mod font_template;

pub mod paint_thread;

// Platform-specific implementations.
#[allow(unsafe_code)]
mod platform;

// Text
pub mod text;
