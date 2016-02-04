/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(plugin)]
#![plugin(plugins)]
#![allow(dead_code)]
#![deny(transmute_type_lint)]

use std::mem::{self, transmute};


fn main() {
    let _: &[u8] = unsafe { transmute("Rust") };
    //~^ ERROR Transmute to &[u8] from &'static str detected

    let _: &[u8] = unsafe { mem::transmute("Rust") };
    //~^ ERROR Transmute to &[u8] from &'static str detected
}
