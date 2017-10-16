/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! `NonZero*` types that are either `core::nonzero::NonZero<_>`
//! or some stable types with an equivalent API (but no memory layout optimization).

#![cfg_attr(feature = "unstable", feature(nonzero))]
#![cfg_attr(feature = "unstable", feature(const_fn))]
#![cfg_attr(feature = "unstable", feature(const_nonzero_new))]

#[cfg_attr(not(feature = "unstable"), macro_use)]
extern crate serde;

pub use imp::*;

#[cfg(feature = "unstable")]
mod imp {
    extern crate core;
    use self::core::nonzero::NonZero as CoreNonZero;
    use serde::{Serialize, Serializer, Deserialize, Deserializer};

    pub use self::core::nonzero::Zeroable;

    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub struct NonZero<T: Zeroable>(CoreNonZero<T>);

    impl<T: Zeroable> NonZero<T> {
        #[inline]
        pub const unsafe fn new_unchecked(x: T) -> Self {
            NonZero(CoreNonZero::new_unchecked(x))
        }

        #[inline]
        pub fn new(x: T) -> Option<Self> {
            CoreNonZero::new(x).map(NonZero)
        }

        #[inline]
        pub fn get(self) -> T {
            self.0.get()
        }
    }

    // Not using derive because of the additional Clone bound required by the inner impl

    impl<T> Serialize for NonZero<T>
    where
        T: Serialize + Zeroable + Clone,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            self.0.serialize(serializer)
        }
    }

    impl<'de, T> Deserialize<'de> for NonZero<T>
    where
        T: Deserialize<'de> + Zeroable,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            CoreNonZero::deserialize(deserializer).map(NonZero)
        }
    }
}

#[cfg(not(feature = "unstable"))]
mod imp {
    #[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
    pub struct NonZero<T: Zeroable>(T);

    impl<T: Zeroable> NonZero<T> {
        #[inline]
        pub unsafe fn new_unchecked(x: T) -> Self {
            NonZero(x)
        }

        #[inline]
        pub fn new(x: T) -> Option<Self> {
            if x.is_zero() {
                None
            } else {
                Some(NonZero(x))
            }
        }

        #[inline]
        pub fn get(self) -> T {
            self.0
        }
    }

    /// Unsafe trait to indicate what types are usable with the NonZero struct
    pub unsafe trait Zeroable {
        /// Whether this value is zero
        fn is_zero(&self) -> bool;
    }

    macro_rules! impl_zeroable_for_pointer_types {
        ( $( $Ptr: ty )+ ) => {
            $(
                /// For fat pointers to be considered "zero", only the "data" part needs to be null.
                unsafe impl<T: ?Sized> Zeroable for $Ptr {
                    #[inline]
                    fn is_zero(&self) -> bool {
                        // Cast because `is_null` is only available on thin pointers
                        (*self as *mut u8).is_null()
                    }
                }
            )+
        }
    }

    macro_rules! impl_zeroable_for_integer_types {
        ( $( $Int: ty )+ ) => {
            $(
                unsafe impl Zeroable for $Int {
                    #[inline]
                    fn is_zero(&self) -> bool {
                        *self == 0
                    }
                }
            )+
        }
    }

    impl_zeroable_for_pointer_types! {
        *const T
        *mut T
    }

    impl_zeroable_for_integer_types! {
        usize u8 u16 u32 u64
        isize i8 i16 i32 i64
    }
}
