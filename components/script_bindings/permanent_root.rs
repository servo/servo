/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ffi::CStr;

use js::context::JSContext;
use js::jsapi::{Heap, JSObject, RemoveRawValueRoot};
use js::jsval::{JSVal, ObjectValue};
use js::rust::Runtime;
use js::rust::wrappers2::AddRawValueRoot;

/// A manual GC root that will exist until this PermanentRoot is dropped.
#[derive(JSTraceable)] // TODO: remove this once this is no longer part of Promise and callback objects.
#[derive(Default, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::allow_unrooted_interior)]
/// Maintains a GC root for the contained value until this object is dropped.
///
/// # Safety
/// The root (and the contained value) is only valid as long as this value
/// is never moved after it is initialized. It should only be used inside
/// of a container like Box or Rc and never extracted from it.
pub struct PermanentRoot(#[ignore_malloc_size_of = "mozjs value"] Heap<JSVal>);

impl PermanentRoot {
    /// Add a GC root for the provided JS object.
    ///
    /// # Safety
    /// - This method must only be called on a `PermanentRoot` that will not
    ///   move for the remainder of its lifetime (e.g. inside of Box, Rc, etc.)
    /// - This must only be called once per instance of `PermanentRoot`
    #[expect(unsafe_code)]
    pub unsafe fn init(&self, cx: &JSContext, object: *mut JSObject, name: &'static CStr) {
        self.0.set(ObjectValue(object));
        unsafe {
            assert!(AddRawValueRoot(cx, self.0.get_unsafe(), name.as_ptr(),));
        }
    }
}

impl Drop for PermanentRoot {
    #[expect(unsafe_code)]
    fn drop(&mut self) {
        let js_root = self.0.get();
        if js_root.is_undefined() {
            return;
        }
        let object = js_root.to_object();
        assert!(!object.is_null());
        if let Some(cx) = Runtime::get() {
            unsafe {
                RemoveRawValueRoot(cx.as_ptr(), self.0.get_unsafe());
            }
        }
    }
}
