/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// Disable this entire crate on Windows when Gecko symbols are not available
// as linking would fail:
// https://github.com/rust-lang/rust/pull/44603#issuecomment-338807312
//
// On Linux and OS X linking succeeds anyway.
// Presumably these symbol declarations don’t need to be resolved
// as they’re not used in any code called from this crate.
#![cfg(any(linking_with_gecko, not(windows)))]

extern crate atomic_refcell;
extern crate cssparser;
#[macro_use] extern crate cstr;
extern crate geckoservo;
#[macro_use] extern crate log;
extern crate malloc_size_of;
extern crate selectors;
extern crate smallvec;
#[macro_use] extern crate size_of_test;
#[macro_use] extern crate style;
extern crate style_traits;

#[cfg(target_pointer_width = "64")]
mod size_of;
mod specified_values;

mod servo_function_signatures;

use style::*;

#[allow(dead_code, improper_ctypes)]
mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}
