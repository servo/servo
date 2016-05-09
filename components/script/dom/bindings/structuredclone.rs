/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! This module implements structured cloning, as defined by [HTML]
//! (https://html.spec.whatwg.org/multipage/#safe-passing-of-structured-data).

use dom::bindings::error::{Error, Fallible};
use dom::bindings::global::GlobalRef;
use js::jsapi::{HandleValue, MutableHandleValue};
use js::jsapi::{JSContext, JS_ReadStructuredClone, JS_STRUCTURED_CLONE_VERSION};
use js::jsapi::{JS_ClearPendingException, JS_WriteStructuredClone};
use libc::size_t;
use std::ptr;

/// A buffer for a structured clone.
pub struct StructuredCloneData {
    data: *mut u64,
    nbytes: size_t,
}

impl StructuredCloneData {
    /// Writes a structured clone. Returns a `DataClone` error if that fails.
    pub fn write(cx: *mut JSContext, message: HandleValue) -> Fallible<StructuredCloneData> {
        let mut data = ptr::null_mut();
        let mut nbytes = 0;
        let result = unsafe {
            JS_WriteStructuredClone(cx,
                                    message,
                                    &mut data,
                                    &mut nbytes,
                                    ptr::null(),
                                    ptr::null_mut(),
                                    HandleValue::undefined())
        };
        if !result {
            unsafe {
                JS_ClearPendingException(cx);
            }
            return Err(Error::DataClone);
        }
        Ok(StructuredCloneData {
            data: data,
            nbytes: nbytes,
        })
    }

    /// Reads a structured clone.
    ///
    /// Panics if `JS_ReadStructuredClone` fails.
    pub fn read(self, global: GlobalRef, rval: MutableHandleValue) {
        unsafe {
            assert!(JS_ReadStructuredClone(global.get_cx(),
                                           self.data,
                                           self.nbytes,
                                           JS_STRUCTURED_CLONE_VERSION,
                                           rval,
                                           ptr::null(),
                                           ptr::null_mut()));
        }
    }
}

unsafe impl Send for StructuredCloneData {}
