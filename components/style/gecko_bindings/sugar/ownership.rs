/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Helpers for different FFI pointer kinds that Gecko's FFI layer uses.

use crate::gecko_bindings::structs::root::mozilla::detail::CopyablePtr;
use servo_arc::Arc;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::ptr;

/// Gecko-FFI-safe Arc (T is an ArcInner).
///
/// This can be null.
///
/// Leaks on drop. Please don't drop this.
#[repr(C)]
pub struct Strong<GeckoType> {
    ptr: *const GeckoType,
    _marker: PhantomData<GeckoType>,
}

impl<T> From<Arc<T>> for Strong<T> {
    fn from(arc: Arc<T>) -> Self {
        Self {
            ptr: Arc::into_raw(arc),
            _marker: PhantomData,
        }
    }
}

impl<GeckoType> Strong<GeckoType> {
    #[inline]
    /// Returns whether this reference is null.
    pub fn is_null(&self) -> bool {
        self.ptr.is_null()
    }

    #[inline]
    /// Returns a null pointer
    pub fn null() -> Self {
        Self {
            ptr: ptr::null(),
            _marker: PhantomData,
        }
    }
}

impl<T> Deref for CopyablePtr<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.mPtr
    }
}

impl<T> DerefMut for CopyablePtr<T> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut T {
        &mut self.mPtr
    }
}
