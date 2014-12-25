/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

///! Utilities for the implementation of JSAPI proxy handlers.

use dom::bindings::conversions::is_dom_proxy;
use dom::bindings::utils::delete_property_by_id;
use js::jsapi::{JSContext, jsid, JSPropertyDescriptor, JSObject, JSString};
use js::jsapi::{JS_GetPropertyDescriptorById, JS_NewStringCopyN};
use js::jsapi::{JS_DefinePropertyById, JS_NewObjectWithGivenProto};
use js::jsapi::{JS_ReportErrorFlagsAndNumber, JS_StrictPropertyStub};
use js::jsapi::{JSREPORT_WARNING, JSREPORT_STRICT, JSREPORT_STRICT_MODE_ERROR};
use js::jsval::ObjectValue;
use js::glue::GetProxyExtra;
use js::glue::{GetObjectProto, GetObjectParent, SetProxyExtra, GetProxyHandler};
use js::glue::InvokeGetOwnPropertyDescriptor;
use js::glue::RUST_js_GetErrorMessage;
use js::glue::AutoIdVector;
use js::{JSPROP_GETTER, JSPROP_ENUMERATE, JSPROP_READONLY, JSRESOLVE_QUALIFIED};

use libc;
use std::mem;
use std::ptr;

static JSPROXYSLOT_EXPANDO: u32 = 0;

pub unsafe extern fn getPropertyDescriptor(cx: *mut JSContext, proxy: *mut JSObject,
                                           id: jsid, set: bool,
                                           desc: *mut JSPropertyDescriptor)
                                           -> bool {
    let handler = GetProxyHandler(proxy);
    if !InvokeGetOwnPropertyDescriptor(handler, cx, proxy, id, set, desc) {
        return false;
    }
    if (*desc).obj.is_not_null() {
        return true;
    }

    //let proto = JS_GetPrototype(proxy);
    let proto = GetObjectProto(proxy);
    if proto.is_null() {
        (*desc).obj = ptr::null_mut();
        return true;
    }

    JS_GetPropertyDescriptorById(cx, proto, id, JSRESOLVE_QUALIFIED, desc) != 0
}

pub unsafe extern fn defineProperty_(cx: *mut JSContext, proxy: *mut JSObject, id: jsid,
                              desc: *mut JSPropertyDescriptor) -> bool {
    static JSMSG_GETTER_ONLY: libc::c_uint = 160;

    //FIXME: Workaround for https://github.com/mozilla/rust/issues/13385
    let setter: *const libc::c_void = mem::transmute((*desc).setter);
    let setter_stub: *const libc::c_void = mem::transmute(JS_StrictPropertyStub);
    if ((*desc).attrs & JSPROP_GETTER) != 0 && setter == setter_stub {
        return JS_ReportErrorFlagsAndNumber(cx,
                                            JSREPORT_WARNING | JSREPORT_STRICT |
                                            JSREPORT_STRICT_MODE_ERROR,
                                            Some(RUST_js_GetErrorMessage), ptr::null_mut(),
                                            JSMSG_GETTER_ONLY) != 0;
    }

    let expando = EnsureExpandoObject(cx, proxy);
    if expando.is_null() {
        return false;
    }

    return JS_DefinePropertyById(cx, expando, id, (*desc).value, (*desc).getter,
                                 (*desc).setter, (*desc).attrs) != 0;
}

pub unsafe extern fn delete_(cx: *mut JSContext, proxy: *mut JSObject, id: jsid,
                             bp: *mut bool) -> bool {
    let expando = EnsureExpandoObject(cx, proxy);
    if expando.is_null() {
        return false;
    }

    return delete_property_by_id(cx, expando, id, &mut *bp);
}

pub fn _obj_toString(cx: *mut JSContext, name: &str) -> *mut JSString {
    unsafe {
        let result = format!("[object {}]", name);

        let chars = result.as_ptr() as *const libc::c_char;
        let length = result.len() as libc::size_t;

        let string = JS_NewStringCopyN(cx, chars, length);
        assert!(string.is_not_null());
        return string;
    }
}

pub fn GetExpandoObject(obj: *mut JSObject) -> *mut JSObject {
    unsafe {
        assert!(is_dom_proxy(obj));
        let val = GetProxyExtra(obj, JSPROXYSLOT_EXPANDO);
        if val.is_undefined() {
            ptr::null_mut()
        } else {
            val.to_object()
        }
    }
}

pub fn EnsureExpandoObject(cx: *mut JSContext, obj: *mut JSObject) -> *mut JSObject {
    unsafe {
        assert!(is_dom_proxy(obj));
        let mut expando = GetExpandoObject(obj);
        if expando.is_null() {
            expando = JS_NewObjectWithGivenProto(cx, ptr::null_mut(),
                                                 ptr::null_mut(),
                                                 GetObjectParent(obj));
            if expando.is_null() {
                return ptr::null_mut();
            }

            SetProxyExtra(obj, JSPROXYSLOT_EXPANDO, ObjectValue(&*expando));
        }
        return expando;
    }
}

pub fn FillPropertyDescriptor(desc: &mut JSPropertyDescriptor, obj: *mut JSObject, readonly: bool) {
    desc.obj = obj;
    desc.attrs = if readonly { JSPROP_READONLY } else { 0 } | JSPROP_ENUMERATE;
    desc.getter = None;
    desc.setter = None;
    desc.shortid = 0;
}

pub unsafe extern fn getOwnPropertyNames_(_cx: *mut JSContext,
                                          _obj: *mut JSObject,
                                          _v: *mut AutoIdVector) -> bool {
    true
}

pub unsafe extern fn enumerate_(_cx: *mut JSContext, _obj: *mut JSObject,
                                _v: *mut AutoIdVector) -> bool {
    true
}
