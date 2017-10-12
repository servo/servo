/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! `NonZero*` types that are either `core::nonzero::NonZero<_>`
//! or some stable types with an equivalent API (but no memory layout optimization).

#![cfg_attr(feature = "unstable", feature(nonzero))]
#![cfg_attr(feature = "unstable", feature(const_fn))]
#![cfg_attr(feature = "unstable", feature(const_nonzero_new))]

#[macro_use]
extern crate serde;

pub use imp::*;

#[cfg(feature = "unstable")]
mod imp {
    extern crate core;
    use self::core::nonzero::NonZero;

    #[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
    pub struct NonZeroU32(NonZero<u32>);

    #[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
    pub struct NonZeroUsize(NonZero<usize>);

    impl NonZeroU32 {
        #[inline] pub const unsafe fn new_unchecked(x: u32) -> Self { NonZeroU32(NonZero::new_unchecked(x)) }
        #[inline] pub fn new(x: u32) -> Option<Self> { NonZero::new(x).map(NonZeroU32) }
        #[inline] pub fn get(self) -> u32 { self.0.get() }
    }

    impl NonZeroUsize {
        #[inline] pub const unsafe fn new_unchecked(x: usize) -> Self { NonZeroUsize(NonZero::new_unchecked(x)) }
        #[inline] pub fn new(x: usize) -> Option<Self> { NonZero::new(x).map(NonZeroUsize) }
        #[inline] pub fn get(self) -> usize { self.0.get() }
    }
}

#[cfg(not(feature = "unstable"))]
mod imp {
    use std::cmp;
    use std::hash;

    #[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
    pub struct NonZeroU32(u32);

    impl NonZeroU32 {
        #[inline]
        pub fn new(x: u32) -> Option<Self> {
            if x != 0 {
                Some(NonZeroU32(x))
            } else {
                None
            }
        }

        #[inline]
        pub unsafe fn new_unchecked(x: u32) -> Self {
            NonZeroU32(x)
        }

        #[inline]
        pub fn get(self) -> u32 {
            self.0
        }
    }

    #[derive(Clone, Copy, Debug, Eq)]
    pub struct NonZeroUsize(&'static ());

    impl NonZeroUsize {
        #[inline]
        pub fn new(x: usize) -> Option<Self> {
            if x != 0 {
                Some(unsafe { Self::new_unchecked(x) })
            } else {
                None
            }
        }

        #[inline]
        pub unsafe fn new_unchecked(x: usize) -> Self {
            NonZeroUsize(&*(x as *const ()))
        }

        #[inline]
        pub fn get(self) -> usize {
            self.0 as *const () as usize
        }
    }

    impl PartialEq for NonZeroUsize {
        #[inline]
        fn eq(&self, other: &Self) -> bool {
            self.get() == other.get()
        }
    }

    impl PartialOrd for NonZeroUsize {
        #[inline]
        fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
            self.get().partial_cmp(&other.get())
        }
    }

    impl Ord for NonZeroUsize {
        #[inline]
        fn cmp(&self, other: &Self) -> cmp::Ordering {
            self.get().cmp(&other.get())
        }
    }

    impl hash::Hash for NonZeroUsize {
        #[inline]
        fn hash<H: hash::Hasher>(&self, hasher: &mut H) {
            self.get().hash(hasher)
        }
    }
}
