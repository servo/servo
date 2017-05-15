/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[macro_export]
macro_rules! size_of_test {
    ($testname: ident, $t: ty, $expected_size: expr) => {
        #[test]
        fn $testname() {
            let new = ::std::mem::size_of::<$t>();
            let old = $expected_size;
            if new < old {
                panic!(
                    "Your changes have decreased the stack size of {} from {} to {}. \
                     Good work! Please update the expected size in {}.",
                    stringify!($t), old, new, file!()
                )
            } else if new > old {
                panic!(
                    "Your changes have increased the stack size of {} from {} to {}. \
                     Please consider choosing a design which avoids this increase. \
                     If you feel that the increase is necessary, update the size in {}.",
                    stringify!($t), old, new, file!()
                )
            }
        }
    }
}
