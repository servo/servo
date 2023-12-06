/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Little helpers for `nsCOMPtr`.

use crate::gecko_bindings::structs::nsCOMPtr;

impl<T> nsCOMPtr<T> {
    /// Get this pointer as a raw pointer.
    #[inline]
    pub fn raw(&self) -> *mut T {
        self.mRawPtr
    }
}
