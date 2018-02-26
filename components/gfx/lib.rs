/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

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
#[cfg(any(target_os = "linux", target_os = "android"))] extern crate freetype;
#[cfg(any(target_os = "linux", target_os = "android"))] extern crate servo_allocator;
extern crate gfx_traits;

// Eventually we would like the shaper to be pluggable, as many operating systems have their own
// shapers. For now, however, this is a hard dependency.
extern crate harfbuzz_sys as harfbuzz;

extern crate ipc_channel;
#[macro_use]
extern crate lazy_static;
#[cfg(any(target_os = "linux", target_os = "android"))] extern crate libc;
#[macro_use]
extern crate log;
#[cfg_attr(target_os = "windows", macro_use)]
extern crate malloc_size_of;
extern crate net_traits;
extern crate ordered_float;
extern crate range;
#[macro_use] extern crate serde;
extern crate servo_arc;
#[macro_use] extern crate servo_atoms;
extern crate servo_url;
#[cfg(feature = "unstable")]
#[cfg(any(target_feature = "sse2", target_feature = "neon"))]
extern crate simd;
extern crate smallvec;
extern crate style;
extern crate time;
extern crate ucd;
extern crate unicode_bidi;
extern crate unicode_script;
extern crate webrender_api;
extern crate xi_unicode;
#[cfg(target_os = "android")]
extern crate xml5ever;

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
