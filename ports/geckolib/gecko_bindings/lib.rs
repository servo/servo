/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(const_fn)]
#![feature(concat_idents)]
#![feature(type_macros)]

extern crate heapsize;

#[allow(dead_code, non_camel_case_types)]
pub mod bindings;
pub mod ptr;
pub mod sugar;
#[allow(dead_code, non_camel_case_types, non_snake_case, non_upper_case_globals)]
pub mod structs;
