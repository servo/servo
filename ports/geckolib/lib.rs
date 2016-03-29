/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(as_unsafe_cell)]
#![feature(box_syntax)]
#![feature(ptr_as_ref)]
#![feature(custom_derive)]
#![feature(plugin)]

#![plugin(heapsize_plugin)]
#![plugin(plugins)]

extern crate app_units;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate cssparser;
extern crate euclid;
extern crate heapsize;
#[macro_use]
extern crate lazy_static;
extern crate libc;
#[macro_use]
extern crate log;
extern crate num_cpus;
extern crate selectors;
extern crate smallvec;
#[macro_use(atom, ns)]
extern crate string_cache;
extern crate style;
extern crate url;
extern crate util;

#[allow(dead_code, non_camel_case_types)]
mod bindings;
mod data;
#[allow(dead_code, non_camel_case_types, non_snake_case, non_upper_case_globals)]
mod gecko_style_structs;
#[allow(non_snake_case)]
pub mod glue;
mod selector_impl;
mod traversal;
mod wrapper;

// Generated from the properties.mako.rs template by build.rs
#[macro_use]
#[allow(unsafe_code)]
pub mod properties {
    include!(concat!(env!("OUT_DIR"), "/properties.rs"));
}
