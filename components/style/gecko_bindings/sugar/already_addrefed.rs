/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! little helpers for `already_AddRefed`.

use gecko_bindings::structs::already_AddRefed;
use std::marker::PhantomData;
use std::mem;

impl<T> already_AddRefed<T> {
    /// Create an already_AddRefed with null pointer.
    #[inline]
    pub fn null() -> Self {
        unsafe { mem::zeroed() }
    }

    /// Create an already_AddRefed from an addrefed pointer.
    #[inline]
    pub unsafe fn new(ptr: *mut T) -> Self {
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

    /// Return whether the pointer is null.
    #[inline]
    pub fn is_null(&self) -> bool {
        self.mRawPtr.is_null()
    }
}

#[cfg(debug_assertions)]
impl<T> Drop for already_AddRefed<T> {
    fn drop(&mut self) {
        debug_assert!(self.mRawPtr.is_null());
    }
}
