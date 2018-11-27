/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

/**
```compile_fail
#[macro_use] extern crate deny_public_fields;

#[derive(DenyPublicFields)]
struct Foo {
    pub v1: i32,
    v2: i32
}

fn main() {}
```
*/
pub fn deny_public_fields_failing() {}

/**
```
#[macro_use] extern crate deny_public_fields;

#[derive(DenyPublicFields)]
struct Foo {
    v1: i32,
    v2: i32
}

fn main() {}
```
*/
pub fn deny_public_fields_ok() {}
