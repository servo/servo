/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![crate_name = "gfx_traits"]
#![crate_type = "rlib"]

#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate azure;
extern crate euclid;
extern crate layers;
extern crate libc;
extern crate msg;
extern crate profile_traits;
extern crate serde;
extern crate string_cache;
extern crate style;
#[macro_use]
extern crate util;

pub mod color;

#[path="display_list/mod.rs"]
pub mod display_list;
pub mod paint_task;

// Platform-specific implementations.
#[path="platform/mod.rs"]
pub mod platform;

// Text
#[path = "text/mod.rs"]
pub mod text;

// Mac OS-specific library dependencies
#[cfg(target_os="macos")] extern crate core_text;
