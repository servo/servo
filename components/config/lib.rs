/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This crate provides two mechanisms for configuring the behaviour of the Servo engine.
//!  - The [`opts`] module exposes a set of global flags that are initialized once
//!    and cannot be changed at runtime.
//!  - The [`prefs`] module provides a mechanism to get and set global preference
//!    values that can be changed at runtime.

#![deny(unsafe_code)]

pub mod opts;
pub mod pref_util;
pub mod prefs;
