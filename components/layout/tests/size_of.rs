/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![cfg(target_pointer_width = "64")]

#[macro_use]
extern crate size_of_test;

use layout_2013::Fragment;
use layout_2013::SpecificFragmentInfo;

#[cfg(debug_assertions)]
size_of_test!(test_size_of_fragment, Fragment, 176);
#[cfg(not(debug_assertions))]
size_of_test!(test_size_of_fragment, Fragment, 152);
size_of_test!(
    test_size_of_specific_fragment_info,
    SpecificFragmentInfo,
    24
);
