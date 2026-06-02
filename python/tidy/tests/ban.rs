/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

extern crate js;

use js::jsval::JSVal;
use std::cell::Cell;
use std::cell::UnsafeCell;

struct Foo {
    bar: Cell<JSVal>,
    //~^ ERROR Banned type Cell<JSVal> detected. Use MutDom<JSVal> instead
    foo: UnsafeCell<JSVal>
    //~^ NOT AN ERROR
}

fn main() {}
