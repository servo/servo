/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! This module contains shared types and messages for use by devtools/script.
//! The traits are here instead of in script so that the devtools crate can be
//! modified independently of the rest of Servo.

#![crate_name = "style_traits"]
#![crate_type = "rlib"]

#![deny(unsafe_code)]

#![cfg_attr(feature = "servo", feature(custom_derive))]
#![cfg_attr(feature = "servo", feature(plugin))]
#![cfg_attr(feature = "servo", plugin(serde_macros))]
#![cfg_attr(feature = "servo", plugin(heapsize_plugin))]

#[macro_use]
extern crate cssparser;
extern crate euclid;
#[cfg(feature = "servo")] extern crate heapsize;
extern crate rustc_serialize;
#[cfg(feature = "servo")] extern crate serde;
extern crate util;

pub mod cursor;
#[macro_use]
pub mod values;
pub mod viewport;

