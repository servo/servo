/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Different static asserts that ensure the build does what it's expected to.
//!
//! TODO: maybe cfg(test) this?

#![allow(unused_imports)]

use std::mem;

macro_rules! check_enum_value {
    ($a:expr, $b:expr) => {
        unsafe {
            mem::transmute::<[u32; $a as usize],
                             [u32; $b as usize]>([0; $a as usize]);
        }
    }
}

// NB: It's a shame we can't do this statically with bitflags, but no
// const-fn and no other way to access the numerical value :-(
macro_rules! check_enum_value_non_static {
    ($a:expr, $b:expr) => {
        assert_eq!($a as usize, $b as usize);
    }
}

#[test]
fn assert_restyle_hints_match() {
    use style::restyle_hints::*; // For flags
    use gecko_bindings::structs::nsRestyleHint;

    check_enum_value_non_static!(nsRestyleHint::eRestyle_Self, RESTYLE_SELF.bits());
    // XXX This for servo actually means something like an hipotetical
    // Restyle_AllDescendants (but without running selector matching on the
    // element). ServoRestyleManager interprets it like that, but in practice we
    // should align the behavior with Gecko adding a new restyle hint, maybe?
    check_enum_value_non_static!(nsRestyleHint::eRestyle_SomeDescendants, RESTYLE_DESCENDANTS.bits());
    check_enum_value_non_static!(nsRestyleHint::eRestyle_LaterSiblings, RESTYLE_LATER_SIBLINGS.bits());
}
