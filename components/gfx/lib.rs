/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// For SIMD
#![cfg_attr(any(target_os = "linux", target_os = "android"), feature(alloc, allocator_api))]
#![feature(box_syntax)]
#![feature(cfg_target_feature)]
#![feature(range_contains)]
#![feature(unique)]

#![deny(unsafe_code)]

#[cfg(any(target_os = "linux", target_os = "android"))]
extern crate alloc;

extern crate app_units;
#[macro_use]
extern crate bitflags;

// Mac OS-specific library dependencies
#[cfg(target_os = "macos")] extern crate byteorder;
#[cfg(target_os = "macos")] extern crate core_foundation;
#[cfg(target_os = "macos")] extern crate core_graphics;
#[cfg(target_os = "macos")] extern crate core_text;

// Windows-specific library dependencies
#[cfg(target_os = "windows")] extern crate dwrote;
#[cfg(target_os = "windows")] extern crate truetype;

extern crate euclid;
extern crate fnv;

#[cfg(target_os = "linux")]
extern crate fontconfig;
extern crate fontsan;
#[cfg(any(target_os = "linux", target_os = "android"))]
extern crate freetype;
extern crate gfx_traits;

// Eventually we would like the shaper to be pluggable, as many operating systems have their own
// shapers. For now, however, this is a hard dependency.
extern crate harfbuzz_sys as harfbuzz;

extern crate heapsize;
#[macro_use] extern crate heapsize_derive;
extern crate ipc_channel;
#[macro_use]
extern crate lazy_static;
extern crate libc;
#[macro_use]
extern crate log;
extern crate msg;
extern crate net_traits;
extern crate ordered_float;
extern crate range;
#[macro_use] extern crate serde;
extern crate servo_geometry;
extern crate servo_url;
#[macro_use] extern crate servo_atoms;
#[cfg(any(target_feature = "sse2", target_feature = "neon"))]
extern crate simd;
extern crate smallvec;
extern crate style;
extern crate style_traits;
extern crate time;
extern crate unicode_bidi;
extern crate unicode_script;
extern crate webrender_traits;
extern crate xi_unicode;
#[cfg(target_os = "android")]
extern crate xml5ever;

#[deny(unsafe_code)]
pub mod display_list;

// Fonts
#[macro_use] pub mod font;
pub mod font_cache_thread;
pub mod font_context;
pub mod font_template;

// Platform-specific implementations.
#[allow(unsafe_code)]
mod platform;

// Text
pub mod text;
