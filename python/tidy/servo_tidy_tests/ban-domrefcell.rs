/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(plugin)]
#![plugin(script_plugins)]

extern crate script;

use script::test::Dom;
use script::test::DomRefCell;
use script::test::Node;

struct Foo {
    bar: DomRefCell<Dom<Node>>
    //~^ ERROR Banned type DomRefCell<Dom<T>> detected. Use MutDom<T> instead
}

fn main() {}
