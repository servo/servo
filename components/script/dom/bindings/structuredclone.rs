/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! This module implements structured cloning, as defined by [HTML]
//! (https://html.spec.whatwg.org/multipage/#safe-passing-of-structured-data).

use dom::bindings::error::{Error, Fallible};
use dom::globalscope::GlobalScope;
use js::jsapi::{HandleValue, MutableHandleValue};
use js::jsapi::{JSContext, JS_ReadStructuredClone, JS_STRUCTURED_CLONE_VERSION};
use js::jsapi::{JS_ClearPendingException, JS_WriteStructuredClone};
use libc::size_t;
use std::ptr;
use std::slice;

/// A buffer for a structured clone.
pub enum StructuredCloneData {
    /// A non-serializable (default) variant
    Struct(*mut u64, size_t),
    /// A variant that can be serialized
    Vector(Vec<u8>)
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
        Ok(StructuredCloneData::Struct(data, nbytes))
    }

    /// Converts a StructuredCloneData to Vec<u8> for inter-thread sharing
    pub fn move_to_arraybuffer(self) -> Vec<u8> {
        match self {
            StructuredCloneData::Struct(data, nbytes) => {
                unsafe {
                    slice::from_raw_parts(data as *mut u8, nbytes).to_vec()
                }
            }
            StructuredCloneData::Vector(msg) => msg
        }
    }

    /// Reads a structured clone.
    ///
    /// Panics if `JS_ReadStructuredClone` fails.
    fn read_clone(global: &GlobalScope, data: *mut u64, nbytes: size_t, rval: MutableHandleValue) {
        unsafe {
            assert!(JS_ReadStructuredClone(global.get_cx(),
                                           data,
                                           nbytes,
                                           JS_STRUCTURED_CLONE_VERSION,
                                           rval,
                                           ptr::null(),
                                           ptr::null_mut()));
        }
    }

    /// Thunk for the actual `read_clone` method. Resolves proper variant for read_clone.
    pub fn read(self, global: &GlobalScope, rval: MutableHandleValue) {
        match self {
            StructuredCloneData::Vector(mut vec_msg) => {
                let nbytes = vec_msg.len();
                let data = vec_msg.as_mut_ptr() as *mut u64;
                StructuredCloneData::read_clone(global, data, nbytes, rval);
            }
            StructuredCloneData::Struct(data, nbytes) => StructuredCloneData::read_clone(global, data, nbytes, rval)
        }
    }
}

unsafe impl Send for StructuredCloneData {}
