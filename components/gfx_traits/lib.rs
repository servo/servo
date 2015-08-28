/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! This module contains the shared types and messages used by other crates like
//! compositing, layout, etc. The enums and traits are here instead of inside gfx
//! because gfx crate can then be modified independently of the rest of Servo.

#![crate_name = "gfx_traits"]
#![crate_type = "rlib"]
#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate azure;
extern crate ipc_channel;
extern crate layers;
extern crate msg;
extern crate profile_traits;
extern crate serde;

pub mod color;
pub mod paint_task;
