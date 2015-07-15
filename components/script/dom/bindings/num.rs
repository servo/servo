/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The `Finite<T>` struct.

use core::nonzero::Zeroable;
use num::Float;
use std::ops::Deref;

/// Encapsulates the IDL restricted float type.
#[derive(JSTraceable,Clone,Eq,PartialEq)]
pub struct Finite<T: Float>(T);

unsafe impl<T: Float> Zeroable for Finite<T> {}

impl<T: Float> Finite<T> {
    /// Create a new `Finite<T: Float>` safely.
    pub fn new(value: T) -> Option<Finite<T>> {
        if value.is_finite() {
            Some(Finite(value))
        } else {
            None
        }
    }

    /// Create a new `Finite<T: Float>`.
    #[inline]
    pub fn wrap(value: T) -> Finite<T> {
        assert!(value.is_finite(), "Finite<T> doesn't encapsulate unrestricted value.");
        Finite(value)
    }
}

impl<T: Float> Deref for Finite<T> {
    type Target = T;

    fn deref(&self) -> &T {
        let &Finite(ref value) = self;
        value
    }
}
