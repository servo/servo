/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! This module contains APIs for the `profile` crate used generically in the
//! rest of Servo. These APIs are here instead of in `profile` so that these
//! modules won't have to depend on `profile`.

#![feature(box_syntax)]
#![feature(plugin, proc_macro)]
#![plugin(plugins)]

#![deny(unsafe_code)]

extern crate ipc_channel;
#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate signpost;
extern crate util;

#[allow(unsafe_code)]
pub mod energy;
pub mod mem;
pub mod time;
