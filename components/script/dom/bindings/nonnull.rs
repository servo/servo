/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A wrapper type for `NonZero<*mut JSObject>`, to enable local trait impls

use js::jsapi::JSObject;
use nonzero::NonZero;

/// A wrapper type for `NonZero<*mut JSObject>`, to enable local trait impls
#[derive(Clone, Copy)]
pub struct NonNullJSObjectPtr(NonZero<*mut JSObject>);

impl NonNullJSObjectPtr {
    #[inline]
    pub unsafe fn new_unchecked(ptr: *mut JSObject) -> Self {
        NonNullJSObjectPtr(NonZero::new_unchecked(ptr))
    }

    #[inline]
    pub fn get(self) -> *mut JSObject {
        self.0.get()
    }
}
