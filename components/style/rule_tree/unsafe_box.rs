/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

use std::mem::ManuallyDrop;
use std::ops::Deref;
use std::ptr;

/// An unsafe box, derefs to `T`.
pub(super) struct UnsafeBox<T> {
    inner: ManuallyDrop<Box<T>>,
}

impl<T> UnsafeBox<T> {
    /// Creates a new unsafe box.
    pub(super) fn from_box(value: Box<T>) -> Self {
        Self {
            inner: ManuallyDrop::new(value),
        }
    }

    /// Creates a new box from a pointer.
    ///
    /// # Safety
    ///
    /// The input should point to a valid `T`.
    pub(super) unsafe fn from_raw(ptr: *mut T) -> Self {
        Self {
            inner: ManuallyDrop::new(Box::from_raw(ptr)),
        }
    }

    /// Creates a new unsafe box from an existing one.
    ///
    /// # Safety
    ///
    /// There is no refcounting or whatever else in an unsafe box, so this
    /// operation can lead to double frees.
    pub(super) unsafe fn clone(this: &Self) -> Self {
        Self {
            inner: ptr::read(&this.inner),
        }
    }

    /// Drops the inner value of this unsafe box.
    ///
    /// # Safety
    ///
    /// Given this doesn't consume the unsafe box itself, this has the same
    /// safety caveats as `ManuallyDrop::drop`.
    pub(super) unsafe fn drop(this: &mut Self) {
        ManuallyDrop::drop(&mut this.inner)
    }
}

impl<T> Deref for UnsafeBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
