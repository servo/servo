/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![crate_name = "process"]
#![crate_type = "rlib"]
#![deny(unsafe_code)]

//! This crate contains types related to process and thread handling like spawning and
//! communications.

// TODO move net/async_runtime.rs to here
// pub mod async_runtime;

pub mod generic_channel;
pub use generic_channel::channel;
