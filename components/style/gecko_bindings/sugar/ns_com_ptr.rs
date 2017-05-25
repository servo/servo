/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Little helpers for `nsCOMPtr`.

use gecko_bindings::structs::nsCOMPtr;

#[cfg(feature = "gecko_debug")]
impl<T> nsCOMPtr<T> {
    /// Get this pointer as a raw pointer.
    #[inline]
    pub fn raw<U>(&self) -> *mut T {
        self.mRawPtr
    }

    /// Set this pointer from an addrefed raw pointer.
    /// It leaks the old pointer.
    #[inline]
    pub unsafe fn set_raw_from_addrefed<U>(&mut self, ptr: *mut T) {
        self.mRawPtr = ptr;
    }
}

#[cfg(not(feature = "gecko_debug"))]
impl nsCOMPtr {
    /// Get this pointer as a raw pointer.
    #[inline]
    pub fn raw<T>(&self) -> *mut T {
        self._base.mRawPtr as *mut _
    }

    /// Set this pointer from an addrefed raw pointer.
    /// It leaks the old pointer.
    #[inline]
    pub unsafe fn set_raw_from_addrefed<T>(&mut self, ptr: *mut T) {
        self._base.mRawPtr = ptr as *mut _;
    }
}
