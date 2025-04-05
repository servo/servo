/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::marker::PhantomData;
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

#[derive(Clone, Copy, Debug)]
/// A compile-time marker that there are operations that could trigger a JS garbage collection
/// operation within the current stack frame. It is trivially copyable, so it should be passed
/// as a function argument and reused when calling other functions whenever possible. Since it
/// is only meaningful within the current stack frame, it is impossible to move it to a different
/// thread or into a task that will execute asynchronously.
pub struct CanGc(PhantomData<*mut ()>);

impl CanGc {
    /// Create a new CanGc value, representing that a GC operation is possible within the
    /// current stack frame.
    pub fn note() -> CanGc {
        CanGc(PhantomData)
    }
}
