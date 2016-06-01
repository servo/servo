/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::mem;
use std::ops::{Index, IndexMut};
use structs::{nsTArray, nsTArrayHeader};

impl<T> Index<u32> for nsTArray<T> {
    type Output = T;

    fn index<'a>(&'a self, index: u32) -> &'a T {
        unsafe { mem::transmute(self.ptr_at(index)) }
    }
}

impl<T> IndexMut<u32> for nsTArray<T> {
    fn index_mut<'a>(&'a mut self, index: u32) -> &'a mut T {
        unsafe { mem::transmute(self.ptr_at_mut(index)) }
    }
}

impl<T> nsTArray<T> {
    #[inline]
    fn header<'a>(&'a self) -> &'a nsTArrayHeader {
        debug_assert!(!self.mBuffer.is_null());
        unsafe { mem::transmute(self.mBuffer) }
    }

    #[inline]
    pub fn len(&self) -> u32 {
        self.header().mLength
    }

    #[inline]
    unsafe fn slice_begin(&self) -> *mut T {
        debug_assert!(!self.mBuffer.is_null());
        (self.mBuffer as *const nsTArrayHeader).offset(1) as *mut _
    }

    #[inline]
    fn ptr_at_mut(&mut self, index: u32) -> *mut T {
        debug_assert!(index <= self.len());
        unsafe {
            self.slice_begin().offset(index as isize)
        }
    }

    #[inline]
    fn ptr_at(&self, index: u32) -> *const T {
        debug_assert!(index <= self.len());
        unsafe {
            self.slice_begin().offset(index as isize)
        }
    }
}
