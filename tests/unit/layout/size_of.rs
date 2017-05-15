/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use layout::Fragment;
use layout::SpecificFragmentInfo;

size_of_test!(test_size_of_fragment, Fragment, 160);
size_of_test!(test_size_of_specific_fragment_info, SpecificFragmentInfo, 24);
