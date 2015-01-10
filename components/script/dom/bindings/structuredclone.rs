/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::error::Fallible;
use dom::bindings::error::Error::DataClone;

use js::jsapi::JSContext;
use js::jsapi::{JS_WriteStructuredClone, JS_ClearPendingException};
use js::jsval::JSVal;

use libc::size_t;
use std::ptr;

#[allow(raw_pointer_deriving)]
#[deriving(Copy)]
pub struct StructuredCloneData {
    pub data: *mut u64,
    pub nbytes: size_t,
}

impl StructuredCloneData {
    pub fn write(cx: *mut JSContext, message: JSVal)
                 -> Fallible<StructuredCloneData> {
        let mut data = ptr::null_mut();
        let mut nbytes = 0;
        let result = unsafe {
            JS_WriteStructuredClone(cx, message, &mut data, &mut nbytes,
                                    ptr::null(), ptr::null_mut())
        };
        if result == 0 {
            unsafe { JS_ClearPendingException(cx); }
            return Err(DataClone);
        }
        Ok(StructuredCloneData {
            data: data,
            nbytes: nbytes,
        })
    }
}
