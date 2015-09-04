/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! This module contains shared types and messages for use by devtools/script.
//! The traits are here instead of in script so that the devtools crate can be
//! modified independently of the rest of Servo.

#![crate_name = "style_traits"]
#![crate_type = "rlib"]
#![feature(custom_derive)]
#![feature(plugin)]
#![plugin(serde_macros)]
#![deny(unsafe_code)]

extern crate euclid;
extern crate rustc_serialize;
extern crate serde;
extern crate util;

#[macro_use]
extern crate cssparser;

#[macro_use]
pub mod values;

pub mod viewport;
