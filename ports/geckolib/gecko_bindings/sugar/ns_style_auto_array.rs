/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::iter::{once, Chain, Once, IntoIterator};
use std::slice::{Iter, IterMut};
use structs::nsStyleAutoArray;

impl<T> nsStyleAutoArray<T> {
    pub fn iter_mut(&mut self) -> Chain<Once<&mut T>, IterMut<T>> {
        once(&mut self.mFirstElement).chain(self.mOtherElements.iter_mut())
    }
    pub fn iter(&self) -> Chain<Once<&T>, Iter<T>> {
        once(&self.mFirstElement).chain(self.mOtherElements.iter())
    }

    // Note that often structs containing autoarrays will have
    // additional member fields that contain the length, which must be kept
    // in sync
    pub fn len(&self) -> usize {
        1 + self.mOtherElements.len()
    }
}

impl<'a, T> IntoIterator for &'a mut nsStyleAutoArray<T> {
    type Item = &'a mut T;
    type IntoIter = Chain<Once<&'a mut T>, IterMut<'a, T>>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
