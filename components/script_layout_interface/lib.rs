/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! This module contains traits in script used generically in the rest of Servo.
//! The traits are here instead of in script so that these modules won't have
//! to depend on script.

#![deny(unsafe_code)]
#![feature(plugin)]
#![plugin(plugins)]

#[allow(unused_extern_crates)]
#[macro_use]
extern crate bitflags;
extern crate style;

pub mod restyle_damage;
