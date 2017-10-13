/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

#[macro_use]
extern crate bitflags;
#[macro_use] extern crate heapsize;
#[macro_use] extern crate heapsize_derive;
extern crate nonzero;
#[macro_use] extern crate serde;
extern crate webrender_api;

pub mod constellation_msg;
