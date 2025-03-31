/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The `Finite<T>` struct.

use std::default::Default;
use std::ops::Deref;

use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use num_traits::Float;

/// Encapsulates the IDL restricted float type.
#[derive(Clone, Copy, Eq, JSTraceable, PartialEq)]
pub struct Finite<T: Float>(T);

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
        assert!(
            value.is_finite(),
            "Finite<T> doesn't encapsulate unrestricted value."
        );
        Finite(value)
    }
}

impl<T: Float> Deref for Finite<T> {
    type Target = T;

    fn deref(&self) -> &T {
        let Finite(value) = self;
        value
    }
}

impl<T: Float + MallocSizeOf> MallocSizeOf for Finite<T> {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        (**self).size_of(ops)
    }
}

impl<T: Float + Default> Default for Finite<T> {
    fn default() -> Finite<T> {
        Finite::wrap(T::default())
    }
}
