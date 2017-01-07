/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(plugin)]
#![plugin(plugins)]

#![deny(unsafe_code)]

#[allow(unused_extern_crates)]
#[macro_use]
extern crate bitflags;
extern crate heapsize;
#[macro_use] extern crate heapsize_derive;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate webrender_traits;

pub mod constellation_msg;
