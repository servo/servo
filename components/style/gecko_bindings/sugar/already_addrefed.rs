/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! little helpers for `already_AddRefed`.

use gecko_bindings::structs::already_AddRefed;
use std::marker::PhantomData;
use std::mem;

impl<T> already_AddRefed<T> {
    /// Create an already_AddRefed from an addrefed pointer.
    #[inline]
    pub unsafe fn new(ptr: *mut T) -> Option<Self> {
        if !ptr.is_null() {
            Some(Self::new_unchecked(ptr))
        } else {
            None
        }
    }

    /// Create an already_AddRefed from an non-nullable addrefed pointer.
    #[inline]
    pub unsafe fn new_unchecked(ptr: *mut T) -> Self {
        debug_assert!(!ptr.is_null());
        already_AddRefed {
            mRawPtr: ptr,
            _phantom_0: PhantomData,
        }
    }

    /// Take the addrefed pointer from this struct.
    #[inline]
    pub fn take(self) -> *mut T {
        let ptr = self.mRawPtr;
        mem::forget(self);
        ptr
    }
}

#[cfg(debug_assertions)]
impl<T> Drop for already_AddRefed<T> {
    fn drop(&mut self) {
        // We really should instead mark already_AddRefed must_use, but
        // we cannot currently, which is servo/rust-bindgen#710.
        unreachable!("Destructor shouldn't be called, otherwise we are leaking")
    }
}
