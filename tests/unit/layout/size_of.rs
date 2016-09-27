/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use layout::Fragment;
use layout::SpecificFragmentInfo;
use std::mem::size_of;

fn check_size_for(name: &'static str, expected: usize, actual: usize) {
    if actual < expected {
        panic!("Your changes have decreased the stack size of {} \
                from {} to {}. Good work! Please update the size in tests/unit/layout/size_of.rs",
                name, expected, actual);
    }

    if actual > expected {
        panic!("Your changes have increased the stack size of {} \
                from {} to {}.  Please consider choosing a design which avoids this increase. \
                If you feel that the increase is necessary, update the size in \
                tests/unit/layout/size_of.rs.",
                name, expected, actual);
    }
}

#[test]
fn test_size_of_fragment() {
    let expected = 160;
    let actual = size_of::<Fragment>();
    check_size_for("layout::fragment::Fragment", expected, actual);
}

#[test]
fn test_size_of_specific_fragment_info() {
    let expected = 24;
    let actual = size_of::<SpecificFragmentInfo>();
    check_size_for("layout::fragment::SpecificFragmentInfo", expected, actual);
}
