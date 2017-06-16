/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! This module contains APIs for the `profile` crate used generically in the
//! rest of Servo. These APIs are here instead of in `profile` so that these
//! modules won't have to depend on `profile`.

#![deny(unsafe_code)]
#![feature(box_syntax)]

extern crate ipc_channel;
#[macro_use]
extern crate log;
#[macro_use] extern crate serde;
extern crate servo_config;
extern crate signpost;

#[allow(unsafe_code)]
pub mod energy;
pub mod mem;
pub mod time;
