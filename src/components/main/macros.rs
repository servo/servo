/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
#[macro_escape];
macro_rules! special_stream(
    ($Chan:ident) => (
        {
            let (port, chan) = stream::();
            (port, $Chan::new(chan))
        }
    );
)

// Spawn a task, capturing the listed variables in a way that avoids the
// move-from-closure error.  This is sugar around the function spawn_with,
// taking care of building a tuple and a lambda.
//
// FIXME: Once cross-crate macros work, there are a few places outside of
// the main crate which could benefit from this macro.
macro_rules! spawn_with(
    ($task:expr, [ $($var:ident),+ ], $body:block) => (
        do ($task).spawn_with(( $($var),+ , () )) |( $($var),+ , () )| $body
    )
)

macro_rules! bitfield(
    ($bitfieldname:ident, $getter:ident, $setter:ident, $value:expr) => (
        impl $bitfieldname {
            #[inline]
            pub fn $getter(self) -> bool {
                (*self & $value) != 0
            }

            #[inline]
            pub fn $setter(&mut self, value: bool) {
                *self = $bitfieldname((**self & !$value) | (if value { $value } else { 0 }))
            }
        }
    )
)

