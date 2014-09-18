/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Exports macros for use in other Servo crates.

#[macro_export]
macro_rules! bitfield(
    ($bitfieldname:ident, $getter:ident, $setter:ident, $value:expr) => (
        impl $bitfieldname {
            #[inline]
            pub fn $getter(self) -> bool {
                let $bitfieldname(this) = self;
                (this & $value) != 0
            }

            #[inline]
            pub fn $setter(&mut self, value: bool) {
                let $bitfieldname(this) = *self;
                *self = $bitfieldname((this & !$value) | (if value { $value } else { 0 }))
            }
        }
    )
)

// NB: if your crate uses these then you also need
// #[phase(plugin)] extern crate string_cache;

#[macro_export]
macro_rules! satom(
    ($str:tt) => (::servo_util::atom::Atom { atom: atom!($str) })
)

#[macro_export]
macro_rules! sns(
    ($str:tt) => (::servo_util::atom::Atom { atom: ns!($str) })
)
