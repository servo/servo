/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(plugin)]
#![plugin(plugins)]
use script::dom::htmlimageelement::parse_a_sizes_attribute;
#[test]
fn some_parse_sizes_test() {
    let result = parse_a_sizes_attribute(...);
    assert!("/* something about result that is boolean */");
}

extern crate msg;
extern crate script;
extern crate url;
extern crate util;

#[cfg(test)] mod origin;
#[cfg(all(test, target_pointer_width = "64"))] mod size_of;
#[cfg(test)] mod textinput;
#[cfg(test)] mod dom {
    mod bindings;
    mod blob;
    mod xmlhttprequest;
}
