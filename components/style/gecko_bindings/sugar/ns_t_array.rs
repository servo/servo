/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Rust helpers for Gecko's nsTArray.

use gecko_bindings::bindings;
use gecko_bindings::structs::{nsTArray, nsTArrayHeader};
use std::mem;
use std::ops::{Deref, DerefMut};
use std::slice;

impl<T> Deref for nsTArray<T> {
    type Target = [T];

    fn deref<'a>(&'a self) -> &'a [T] {
        unsafe {
            slice::from_raw_parts(self.slice_begin(),
                                  self.header().mLength as usize)
        }
    }
}

impl<T> DerefMut for nsTArray<T> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut [T] {
        unsafe {
            slice::from_raw_parts_mut(self.slice_begin(),
                                      self.header().mLength as usize)
        }
    }
}

impl<T> nsTArray<T> {
    #[inline]
    fn header<'a>(&'a self) -> &'a nsTArrayHeader {
        debug_assert!(!self.mBuffer.is_null());
        unsafe { mem::transmute(self.mBuffer) }
    }
    // unsafe, since header may be in shared static or something
    unsafe fn header_mut<'a>(&'a mut self) -> &'a mut nsTArrayHeader {
        debug_assert!(!self.mBuffer.is_null());

        mem::transmute(self.mBuffer)
    }

    #[inline]
    unsafe fn slice_begin(&self) -> *mut T {
        debug_assert!(!self.mBuffer.is_null());
        (self.mBuffer as *const nsTArrayHeader).offset(1) as *mut _
    }

    /// Ensures the array has enough capacity at least to hold `cap` elements.
    ///
    /// NOTE: This doesn't call the constructor on the values!
    pub fn ensure_capacity(&mut self, cap: usize) {
        if cap >= self.len() {
            unsafe {
                bindings::Gecko_EnsureTArrayCapacity(self as *mut nsTArray<T> as *mut _,
                                                     cap, mem::size_of::<T>())
            }
        }
    }

    /// Clears the array storage without calling the destructor on the values.
    #[inline]
    pub unsafe fn clear(&mut self) {
        if self.len() != 0 {
            bindings::Gecko_ClearPODTArray(self as *mut nsTArray<T> as *mut _,
                                           mem::size_of::<T>(),
                                           mem::align_of::<T>());
        }
    }


    /// Clears a POD array. This is safe since copy types are memcopyable.
    #[inline]
    pub fn clear_pod(&mut self)
        where T: Copy
    {
        unsafe { self.clear() }
    }

    /// Resize and set the length of the array to `len`.
    ///
    /// unsafe because the array may contain uninitialized members.
    ///
    /// This will not call constructors, if you need that, either manually add
    /// bindings or run the typed `EnsureCapacity` call on the gecko side.
    pub unsafe fn set_len(&mut self, len: u32) {
        // this can leak
        debug_assert!(len >= self.len() as u32);
        self.ensure_capacity(len as usize);
        let mut header = self.header_mut();
        header.mLength = len;
    }
}
