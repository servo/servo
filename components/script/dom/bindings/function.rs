/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

use std::ffi::c_char;
use std::rc::Rc;

use js::jsapi::{JSContext, JSObject, JS_GetFunctionObject, JS_NewFunction, Value};

use crate::dom::bindings::codegen::Bindings::FunctionBinding::Function;
use crate::dom::types::GlobalScope;

type NativeFunction = unsafe extern "C" fn(*mut JSContext, u32, *mut Value) -> bool;

pub struct FunctionBinding {}

impl FunctionBinding {
    pub fn new_native(call: NativeFunction, name: &[u8], nargs: u32, flags: u32) -> Rc<Function> {
        let cx = GlobalScope::get_cx();
        let fun_obj = Self::new_raw_obj(cx, call, name, nargs, flags);
        unsafe { Function::new(cx, fun_obj) }
    }

    pub fn new_raw_obj(
        cx: crate::script_runtime::JSContext,
        call: NativeFunction,
        name: &[u8],
        nargs: u32,
        flags: u32,
    ) -> *mut JSObject {
        unsafe {
            let raw_fun = JS_NewFunction(
                *cx,
                Some(call),
                nargs,
                flags,
                name.as_ptr() as *const c_char,
            );
            assert!(!raw_fun.is_null());
            JS_GetFunctionObject(raw_fun)
        }
    }
}
