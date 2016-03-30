/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use layout::Fragment;
use std::mem::size_of;

#[test]
fn test_size_of_fragment() {
    let expected = 168;
    let actual = size_of::<Fragment>();

    if actual < expected {
        panic!("Your changes have decreased the stack size of layout::fragment::Fragment \
                from {} to {}. Good work! Please update the size in tests/layout/size_of.rs",
                expected, actual);
    }

    if actual > expected {
        panic!("Your changes have increased the stack size of layout::fragment::Fragment \
                from {} to {}.  Please consider choosing a design which avoids this increase. \
                If you feel that the increase is necessary, update the size in \
                tests/layout/size_of.rs.",
                expected, actual);
    }
}
