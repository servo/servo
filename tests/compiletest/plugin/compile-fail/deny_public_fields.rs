/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(dead_code)]

#[macro_use]
extern crate deny_public_fields;

#[derive(DenyPublicFields)]
//~^ ERROR proc-macro derive panicked
//~| HELP Field `v1` should not be public
struct Foo {
    pub v1: i32,
    v2: i32
}

fn main() {}
