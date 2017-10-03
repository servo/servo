/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![deny(warnings)]

extern crate cssparser;
extern crate env_logger;
extern crate libc;
#[macro_use] extern crate log;
extern crate malloc_size_of;
extern crate selectors;
extern crate servo_arc;
extern crate smallvec;
#[macro_use] extern crate style;
extern crate style_traits;

mod error_reporter;
#[allow(non_snake_case)]
pub mod glue;
mod stylesheet_loader;

// FIXME(bholley): This should probably go away once we harmonize the allocators.
#[no_mangle]
pub extern "C" fn je_malloc_usable_size(_: *const ::libc::c_void) -> ::libc::size_t { 0 }
