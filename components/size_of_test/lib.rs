/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub use static_assertions::const_assert_eq;

/// Asserts the size of a type at compile time.
#[macro_export]
macro_rules! size_of_test {
    ($t: ty, $expected_size: expr) => {
        #[cfg(target_pointer_width = "64")]
        $crate::const_assert_eq!(std::mem::size_of::<$t>(), $expected_size);
    };
}
