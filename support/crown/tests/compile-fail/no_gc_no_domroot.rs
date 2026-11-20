/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
//@rustc-env:RUSTC_BOOTSTRAP=1
#![deny(crown::no_gc_no_domroot)]

// Needed to mimic the path type `script_bindings::root::Root`
#![crate_name = "script_bindings"]

use root::Root;
use js::context::NoGC;

pub mod root {
    pub struct Root<T> {
        /// The value to root.
        pub value: T,
    }
}

pub mod js {
    pub mod context {
        pub struct NoGC {}
    }
}

struct SomeImpl {}

impl SomeImpl {
    fn returns_domroot(&self) -> Root<u32> {
        Root {
            value: 0,
        }
    }
    fn calls_domroot(&self, _no_gc: &NoGC) -> u32 {
        self.returns_domroot().value
        //~^ ERROR: 35:14: 35:31: method call that returns DomRoot inside a method that has NoGC [crown::no_gc_no_domroot]
    }
}

fn main() {
    let some_impl = SomeImpl {};
    let _ = some_impl.calls_domroot(&NoGC {});
}
