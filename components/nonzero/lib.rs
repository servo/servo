/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! `NonZero*` types that are either `core::nonzero::NonZero<_>`
//! or some stable types with an equivalent API (but no memory layout optimization).

#![cfg_attr(feature = "unstable", feature(nonzero))]

#[cfg(not(feature = "unstable"))]
#[macro_use]
extern crate serde;

pub use imp::*;

#[cfg(feature = "unstable")]
mod imp {
    extern crate core;
    use self::core::nonzero::NonZero;

    pub type NonZeroU32 = NonZero<u32>;
}

#[cfg(not(feature = "unstable"))]
mod imp {
    #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    pub struct NonZeroU32(u32);

    impl NonZeroU32 {
        pub fn new(x: u32) -> Option<Self> {
            if x != 0 {
                Some(NonZeroU32(x))
            } else {
                None
            }
        }

        pub fn get(self) -> u32 {
            self.0
        }
    }
}
