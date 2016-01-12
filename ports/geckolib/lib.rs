/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(as_unsafe_cell)]
#![feature(box_syntax)]
#![feature(ptr_as_ref)]

extern crate app_units;
#[macro_use]
extern crate bitflags;
extern crate cssparser;
extern crate euclid;
extern crate libc;
extern crate num_cpus;
#[macro_use(state_pseudo_classes)]
extern crate selectors;
extern crate smallvec;
#[macro_use(atom, ns)]
extern crate string_cache;
extern crate style;
extern crate url;
extern crate util;

#[allow(dead_code, non_camel_case_types)]
mod bindings;
#[allow(non_snake_case)]
pub mod glue;
mod wrapper;
