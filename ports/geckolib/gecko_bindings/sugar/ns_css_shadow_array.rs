/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use bindings::Gecko_AddRefCSSShadowArrayArbitraryThread;
use bindings::Gecko_NewCSSShadowArray;
use bindings::Gecko_ReleaseCSSShadowArrayArbitraryThread;
use std::{ptr, slice};
use std::ops::{Deref, DerefMut};
use structs::{RefPtr, nsCSSShadowArray, nsCSSShadowItem};

impl RefPtr<nsCSSShadowArray> {
    pub fn replace_with_new(&mut self, len: u32) {
        unsafe {
            if !self.mRawPtr.is_null() {
                Gecko_ReleaseCSSShadowArrayArbitraryThread(self.mRawPtr);
            }

            self.mRawPtr = if len == 0 {
                ptr::null_mut()
            } else {
                Gecko_NewCSSShadowArray(len)
            }
        }
    }
    pub fn copy_from(&mut self, other: &Self) {
        unsafe {
            if !self.mRawPtr.is_null() {
                Gecko_ReleaseCSSShadowArrayArbitraryThread(self.mRawPtr);
            }
            if !other.mRawPtr.is_null() {
                Gecko_AddRefCSSShadowArrayArbitraryThread(other.mRawPtr);
            }

            self.mRawPtr = other.mRawPtr;
        }
    }
}

impl Deref for RefPtr<nsCSSShadowArray> {
    type Target = [nsCSSShadowItem];
    fn deref(&self) -> &[nsCSSShadowItem] {
        if self.mRawPtr.is_null() {
            &[]
        } else {
            unsafe {
                slice::from_raw_parts((*self.mRawPtr).mArray.as_ptr(),
                                      (*self.mRawPtr).mLength as usize)
            }
        }
    }
}

impl DerefMut for RefPtr<nsCSSShadowArray> {
    fn deref_mut(&mut self) -> &mut [nsCSSShadowItem] {
        if self.mRawPtr.is_null() {
            &mut []
        } else {
            unsafe {
                slice::from_raw_parts_mut((*self.mRawPtr).mArray.as_mut_ptr(),
                                          (*self.mRawPtr).mLength as usize)
            }
        }
    }
}
