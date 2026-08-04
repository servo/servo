/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
//@rustc-env:RUSTC_BOOTSTRAP=1

// Needed to mimic the path type `script_bindings::root::Root`
#![crate_name = "script_bindings"]
#![expect(dead_code)]
#![deny(crown::domroot_inside_dom_struct)]

use root::Root;

/// Mock `JSTraceable`
pub trait JSTraceable {}

/// Mock a `Root` type
pub mod root {
    pub struct Root<T> {
        /// The value to root.
        value: T,
    }
}

struct TraceableStruct {
    rooted_field: Root<u32>,
    //~^ Error: Storing a rooted type can lead to circular references
}

impl JSTraceable for TraceableStruct {}


fn main() {}
