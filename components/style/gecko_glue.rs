/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

use std::marker::PhantomData;
use std::mem::{forget, transmute};
use std::sync::Arc;

pub struct ArcHelpers<GeckoType, ServoType> {
    phantom1: PhantomData<GeckoType>,
    phantom2: PhantomData<ServoType>,
}


impl<GeckoType, ServoType> ArcHelpers<GeckoType, ServoType> {
    pub fn with<F, Output>(raw: *mut GeckoType, cb: F) -> Output
                           where F: FnOnce(&Arc<ServoType>) -> Output {
        debug_assert!(!raw.is_null());

        let owned = unsafe { Self::into(raw) };
        let result = cb(&owned);
        forget(owned);
        result
    }

    pub fn maybe_with<F, Output>(maybe_raw: *mut GeckoType, cb: F) -> Output
                                 where F: FnOnce(Option<&Arc<ServoType>>) -> Output {
        let owned = if maybe_raw.is_null() {
            None
        } else {
            Some(unsafe { Self::into(maybe_raw) })
        };

        let result = cb(owned.as_ref());
        forget(owned);

        result
    }

    pub unsafe fn into(ptr: *mut GeckoType) -> Arc<ServoType> {
        transmute(ptr)
    }

    pub fn from(owned: Arc<ServoType>) -> *mut GeckoType {
        unsafe { transmute(owned) }
    }

    pub unsafe fn addref(ptr: *mut GeckoType) {
        Self::with(ptr, |arc| forget(arc.clone()));
    }

    pub unsafe fn release(ptr: *mut GeckoType) {
        let _ = Self::into(ptr);
    }
}
