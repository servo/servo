/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::error::Fallible;
use dom::bindings::error::Error::DataClone;
use dom::bindings::global::GlobalRef;

use js::glue::JS_STRUCTURED_CLONE_VERSION;
use js::jsapi::JSContext;
use js::jsapi::{JS_WriteStructuredClone, JS_ClearPendingException};
use js::jsapi::JS_ReadStructuredClone;
use js::jsval::{JSVal, UndefinedValue};

use libc::size_t;
use std::ptr;

pub struct StructuredCloneData {
    data: *mut u64,
    nbytes: size_t,
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

    pub fn read(self, global: GlobalRef) -> JSVal {
        let mut message = UndefinedValue();
        unsafe {
            assert!(JS_ReadStructuredClone(
                global.get_cx(), self.data as *const u64, self.nbytes,
                JS_STRUCTURED_CLONE_VERSION, &mut message,
                ptr::null(), ptr::null_mut()) != 0);
        }
        message
    }
}
