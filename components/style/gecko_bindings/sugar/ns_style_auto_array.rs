/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Rust helpers for Gecko's `nsStyleAutoArray`.

use gecko_bindings::bindings::Gecko_EnsureStyleAnimationArrayLength;
use gecko_bindings::structs::nsStyleAutoArray;
use std::iter::{once, Chain, Once, IntoIterator};
use std::ops::Index;
use std::slice::{Iter, IterMut};

impl<T> Index<usize> for nsStyleAutoArray<T> {
    type Output = T;
    fn index(&self, index: usize) -> &T {
        if index > self.len() {
            panic!("out of range")
        }
        match index {
            0 => &self.mFirstElement,
            _ => &self.mOtherElements[index - 1],
        }
    }
}

impl<T> nsStyleAutoArray<T> {
    /// Mutably iterate over the array elements.
    pub fn iter_mut(&mut self) -> Chain<Once<&mut T>, IterMut<T>> {
        once(&mut self.mFirstElement).chain(self.mOtherElements.iter_mut())
    }

    /// Iterate over the array elements.
    pub fn iter(&self) -> Chain<Once<&T>, Iter<T>> {
        once(&self.mFirstElement).chain(self.mOtherElements.iter())
    }

    /// Returns the length of the array.
    ///
    /// Note that often structs containing autoarrays will have additional
    /// member fields that contain the length, which must be kept in sync.
    pub fn len(&self) -> usize {
        1 + self.mOtherElements.len()
    }

    /// Ensures that the array has length at least the given length.
    pub fn ensure_len(&mut self, len: usize) {
        unsafe {
            Gecko_EnsureStyleAnimationArrayLength(self as *mut nsStyleAutoArray<T> as *mut _,
                                                  len);
        }
    }
}

impl<'a, T> IntoIterator for &'a mut nsStyleAutoArray<T> {
    type Item = &'a mut T;
    type IntoIter = Chain<Once<&'a mut T>, IterMut<'a, T>>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
