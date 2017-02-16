/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(dead_code)]
#![feature(plugin)]
#![plugin(script_plugins)]

#[must_root]
struct Foo {
    v: i32
}

struct Bar {
    f: Foo
    //~^ ERROR Type must be rooted, use #[must_root] on the struct definition to propagate
}

fn foo1(_: Foo) {} //~ ERROR Type must be rooted


fn foo2() -> Foo { //~ ERROR Type must be rooted
    Foo { v: 10 }
}


fn main() {}
