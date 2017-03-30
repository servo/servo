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
        assert_eq!($a.0 as usize, $b as usize);
    }
}

// Note that we can't call each_pseudo_element, parse_pseudo_element, or
// similar, because we'd need the foreign atom symbols to link.
#[test]
fn assert_basic_pseudo_elements() {
    let saw_before;
    let saw_after;

    macro_rules! pseudo_element {
        (":before", $atom:expr, false) => {
            saw_before = true;
        };
        (":after", $atom:expr, false) => {
            saw_after = true;
        };
        ($pseudo_str_with_colon:expr, $atom:expr, $is_anon_box:expr) => {
            // Do nothing
        };
    }

    include!("../../../components/style/gecko/generated/gecko_pseudo_element_helper.rs");

    assert!(saw_before);
    assert!(saw_after);
}
