/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(plugin)]
#![plugin(plugins)]

extern crate script;

use script::test::DOMRefCell;
use script::test::JS;
use script::test::Node;

struct Foo {
    bar: DOMRefCell<JS<Node>>
    //~^ ERROR Banned type DOMRefCell<JS<T>> detected. Use MutJS<JS<T>> instead,
}

fn main() {}
