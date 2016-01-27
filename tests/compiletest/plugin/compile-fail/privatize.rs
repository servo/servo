/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(plugin, custom_attribute)]
#![plugin(plugins)]
#![allow(dead_code)]

#[privatize]
struct Foo {
    pub v1: i32,
    //~^ ERROR Field v1 is public where only private fields are allowed
    v2: i32
}

fn main() {}
