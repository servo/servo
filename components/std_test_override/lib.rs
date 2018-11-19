/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![feature(test)]

extern crate embedder_traits;
extern crate test;

pub use test::*;

pub fn test_main_static(tests: &[&TestDescAndFn]) {
    embedder_traits::resources::set_for_tests();
    test::test_main_static(tests);
}
