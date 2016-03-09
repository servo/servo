/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(plugin)]
#![plugin(plugins)]

extern crate script;

use script::dom::bindings::cell::DOMRefCell;
use script::dom::bindings::js::JS;
use script::dom::node::Node;

struct Foo {
    bar: DOMRefCell<JS<Node>>
    //~^ ERROR Banned type DOMRefCell<JS<T>> detected. Use MutHeap<JS<T>> instead,
}

fn main() {}
