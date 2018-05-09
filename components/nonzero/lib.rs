/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! `NonZero*` types that are either `core::nonzero::NonZero<_>`
//! or some stable types with an equivalent API (but no memory layout optimization).

#![cfg_attr(feature = "unstable", feature(nonzero))]
#![cfg_attr(feature = "unstable", feature(const_fn))]

extern crate serde;

use std::fmt;

macro_rules! impl_nonzero_fmt {
    ( ( $( $Trait: ident ),+ ) for $Ty: ident ) => {
        $(
            impl fmt::$Trait for $Ty {
                #[inline]
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    self.get().fmt(f)
                }
            }
        )+
    }
}

macro_rules! nonzero_integers {
    ( $( $Ty: ident($Int: ty); )+ ) => {
        $(
            #[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
            pub struct $Ty(
                #[cfg(feature = "unstable")] std::num::$Ty,
                #[cfg(not(feature = "unstable"))] $Int,
            );

            impl $Ty {
                #[cfg(feature = "unstable")]
                #[inline]
                pub const unsafe fn new_unchecked(n: $Int) -> Self {
                    $Ty(std::num::$Ty::new_unchecked(n))
                }

                #[cfg(not(feature = "unstable"))]
                #[inline]
                pub unsafe fn new_unchecked(n: $Int) -> Self {
                    $Ty(n)
                }

                #[cfg(feature = "unstable")]
                #[inline]
                pub fn new(n: $Int) -> Option<Self> {
                    std::num::$Ty::new(n).map($Ty)
                }

                #[cfg(not(feature = "unstable"))]
                #[inline]
                pub fn new(n: $Int) -> Option<Self> {
                    if n != 0 {
                        Some($Ty(n))
                    } else {
                        None
                    }
                }

                #[cfg(feature = "unstable")]
                #[inline]
                pub fn get(self) -> $Int {
                    self.0.get()
                }

                #[cfg(not(feature = "unstable"))]
                #[inline]
                pub fn get(self) -> $Int {
                    self.0
                }
            }

            impl_nonzero_fmt! {
                (Debug, Display, Binary, Octal, LowerHex, UpperHex) for $Ty
            }

            impl serde::Serialize for $Ty {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                    where S: serde::Serializer
                {
                    self.get().serialize(serializer)
                }
            }

            impl<'de> serde::Deserialize<'de> for $Ty {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                    where D: serde::Deserializer<'de>
                {
                    let value = <$Int>::deserialize(deserializer)?;
                    match <$Ty>::new(value) {
                        Some(nonzero) => Ok(nonzero),
                        None => Err(serde::de::Error::custom("expected a non-zero value")),
                    }
                }
            }
        )+
    }
}

nonzero_integers! {
    NonZeroU8(u8);
    NonZeroU16(u16);
    NonZeroU32(u32);
    NonZeroU64(u64);
    NonZeroUsize(usize);
}
