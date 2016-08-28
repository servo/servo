/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

 use bindings::Gecko_Construct_CSSShadowArray;
 use bindings::Gecko_AddRefCSSShadowArrayArbitraryThread;
 use bindings::Gecko_ReleaseCSSShadowArrayArbitraryThread;
 use structs::{nsCSSShadowArray, nsCSSShadowItem, RefPtr};
 use std::ops::{Deref, DerefMut};
 use std::{ptr, slice};

 impl RefPtr<nsCSSShadowArray> {
    pub fn replace_with_new(&mut self, len: u32) {
        unsafe {
            if !self.mRawPtr.is_null() {
                Gecko_ReleaseCSSShadowArrayArbitraryThread(self.mRawPtr);
            }

            let self.mRawPtr = if len == 0 {
                ptr::null_mut()
            } else {
                Gecko_Construct_CSSShadowArray(len)
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
                slice::from_raw_parts(&(*self.mRawPtr).mArray[0] as *const _,
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
                slice::from_raw_parts_mut(&mut (*self.mRawPtr).mArray[0] as *mut _,
                                          (*self.mRawPtr).mLength as usize)
            }
        }
    }
 }
