/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate app_units;
#[macro_use]
extern crate cssparser;
extern crate env_logger;
extern crate euclid;
extern crate gecko_bindings;
#[cfg(feature = "servo_features")] #[macro_use] extern crate heapsize;
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
extern crate style_traits;
extern crate url;
extern crate util;

mod data;
#[allow(non_snake_case)]
pub mod glue;
mod selector_impl;
mod traversal;
mod values;
mod wrapper;

// Generated from the properties.mako.rs template by build.rs
#[macro_use]
#[allow(unsafe_code)]
pub mod properties {
    include!(concat!(env!("OUT_DIR"), "/properties.rs"));
}
