/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Little helpers for `nsCOMPtr`.

use gecko_bindings::structs::nsCOMPtr;

impl<T> nsCOMPtr<T> {
    /// Get this pointer as a raw pointer.
    #[cfg(feature = "gecko_debug")]
    #[inline]
    pub fn raw(&self) -> *mut T {
        self.mRawPtr
    }

    /// Get this pointer as a raw pointer.
    #[cfg(not(feature = "gecko_debug"))]
    #[inline]
    pub fn raw(&self) -> *mut T {
        self._base.mRawPtr as *mut _
    }
}
