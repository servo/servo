/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(plugin)]
#![plugin(script_plugins)]

extern crate script;

use script::test::Dom;
use script::test::DOMRefCell;
use script::test::Node;

struct Foo {
    bar: DOMRefCell<Dom<Node>>
    //~^ ERROR Banned type DOMRefCell<Dom<T>> detected. Use MutJS<T> instead
}

fn main() {}
