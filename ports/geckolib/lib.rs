/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate app_units;
extern crate env_logger;
extern crate euclid;
extern crate gecko_bindings;
#[macro_use] extern crate gecko_string_cache;
#[macro_use] extern crate lazy_static;
extern crate libc;
#[macro_use] extern crate log;
extern crate num_cpus;
extern crate selectors;
extern crate style;
extern crate style_traits;
extern crate url;

mod context;
mod data;
mod snapshot;
mod snapshot_helpers;
#[allow(non_snake_case)]
pub mod glue;
mod sanity_checks;
mod traversal;
mod wrapper;

// FIXME(bholley): This should probably go away once we harmonize the allocators.
#[no_mangle]
pub extern "C" fn je_malloc_usable_size(_: *const ::libc::c_void) -> ::libc::size_t { 0 }
