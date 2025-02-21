/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::ops::Deref;

use js::jsapi::JSContext as RawJSContext;

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct JSContext(*mut RawJSContext);

#[allow(unsafe_code)]
impl JSContext {
    /// Create a new [`JSContext`] object from the given raw pointer.
    ///
    /// # Safety
    ///
    /// The `RawJSContext` argument must point to a valid `RawJSContext` in memory.
    pub unsafe fn from_ptr(raw_js_context: *mut RawJSContext) -> Self {
        JSContext(raw_js_context)
    }
}

#[allow(unsafe_code)]
impl Deref for JSContext {
    type Target = *mut RawJSContext;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

thread_local!(
    static THREAD_ACTIVE: Cell<bool> = const { Cell::new(true) };
);

pub fn runtime_is_alive() -> bool {
    THREAD_ACTIVE.with(|t| t.get())
}

pub fn mark_runtime_dead() {
    THREAD_ACTIVE.with(|t| t.set(false));
}
