/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(plugin)]
#![plugin(plugins)]

extern crate js;

use js::jsval::JSVal;
use std::cell::Cell;

struct Foo {
    bar: Cell<JSVal>
    //~^ ERROR Banned type Cell<JSVal> detected. Use MutJS<JSVal> instead
}

fn main() {}
