/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use bindings::ServoFFIAddRefed;
use std::mem;
use std::ops;

impl<T> ops::Deref for ServoFFIAddRefed<T> {
    type Target = T;
    fn deref<'a>(&'a self) -> &'a T {
        debug_assert!(!self.mRawPtr.is_null());
        unsafe { mem::transmute(self.mRawPtr) }
    }
}

impl<T> ops::DerefMut for ServoFFIAddRefed<T> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut T {
        debug_assert!(!self.mRawPtr.is_null());
        unsafe { mem::transmute(self.mRawPtr) }
    }
}

impl<T> ServoFFIAddRefed<T> {
    pub fn new(ptr: *mut T) -> ServoFFIAddRefed<T> {
        ServoFFIAddRefed {
            mRawPtr: ptr,
        }
    }

    pub fn checked<'a>(&'a self) -> Option<&'a T> {
        // TODO: This can probably just be written as:
        //
        // mem::transmute(self.mRawPtr);
        //
        // since rust optimises the Option<&T> case with None as the null
        // pointer.
        //
        // Nontheless this is probably optimised away, so we just don't rely on
        // it.
        if self.mRawPtr.is_null() {
            None
        } else {
            Some(unsafe { mem::transmute(self.mRawPtr) })
        }
    }

    pub fn checked_mut<'a>(&'a mut self) -> Option<&'a mut T> {
        if self.mRawPtr.is_null() {
            None
        } else {
            Some(unsafe { mem::transmute(self.mRawPtr) })
        }
    }
}
