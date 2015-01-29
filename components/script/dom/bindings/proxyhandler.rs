/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Utilities for the implementation of JSAPI proxy handlers.

#![deny(missing_docs)]

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

/// Invoke the [[GetOwnProperty]] trap (`getOwnPropertyDescriptor`) on `proxy`,
/// with argument `id` and return the result, if it is not `undefined`.
/// Otherwise, walk along the prototype chain to find a property with that
/// name.
pub unsafe extern fn get_property_descriptor(cx: *mut JSContext,
                                             proxy: *mut JSObject,
                                             id: jsid, set: bool,
                                             desc: *mut JSPropertyDescriptor)
                                             -> bool {
    let handler = GetProxyHandler(proxy);
    if !InvokeGetOwnPropertyDescriptor(handler, cx, proxy, id, set, desc) {
        return false;
    }
    if !(*desc).obj.is_null() {
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

/// Defines an expando on the given `proxy`.
pub unsafe extern fn define_property(cx: *mut JSContext, proxy: *mut JSObject,
                                     id: jsid, desc: *mut JSPropertyDescriptor)
                                     -> bool {
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

    let expando = ensure_expando_object(cx, proxy);
    return JS_DefinePropertyById(cx, expando, id, (*desc).value, (*desc).getter,
                                 (*desc).setter, (*desc).attrs) != 0;
}

/// Deletes an expando off the given `proxy`.
pub unsafe extern fn delete(cx: *mut JSContext, proxy: *mut JSObject, id: jsid,
                            bp: *mut bool) -> bool {
    let expando = get_expando_object(proxy);
    if expando.is_null() {
        *bp = true;
        return true;
    }

    return delete_property_by_id(cx, expando, id, &mut *bp);
}

/// Returns the stringification of an object with class `name`.
pub fn object_to_string(cx: *mut JSContext, name: &str) -> *mut JSString {
    unsafe {
        let result = format!("[object {}]", name);

        let chars = result.as_ptr() as *const libc::c_char;
        let length = result.len() as libc::size_t;

        let string = JS_NewStringCopyN(cx, chars, length);
        assert!(!string.is_null());
        return string;
    }
}

/// Get the expando object, or null if there is none.
pub fn get_expando_object(obj: *mut JSObject) -> *mut JSObject {
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

/// Get the expando object, or create it if it doesn't exist yet.
/// Fails on JSAPI failure.
pub fn ensure_expando_object(cx: *mut JSContext, obj: *mut JSObject)
                             -> *mut JSObject {
    unsafe {
        assert!(is_dom_proxy(obj));
        let mut expando = get_expando_object(obj);
        if expando.is_null() {
            expando = JS_NewObjectWithGivenProto(cx, ptr::null_mut(),
                                                 ptr::null_mut(),
                                                 GetObjectParent(obj));
            assert!(!expando.is_null());

            SetProxyExtra(obj, JSPROXYSLOT_EXPANDO, ObjectValue(&*expando));
        }
        return expando;
    }
}

/// Set the property descriptor's object to `obj` and set it to enumerable,
/// and writable if `readonly` is true.
pub fn fill_property_descriptor(desc: &mut JSPropertyDescriptor,
                                obj: *mut JSObject, readonly: bool) {
    desc.obj = obj;
    desc.attrs = if readonly { JSPROP_READONLY } else { 0 } | JSPROP_ENUMERATE;
    desc.getter = None;
    desc.setter = None;
    desc.shortid = 0;
}

/// No-op required hook.
pub unsafe extern fn get_own_property_names(_cx: *mut JSContext,
                                            _obj: *mut JSObject,
                                            _v: *mut AutoIdVector) -> bool {
    true
}

/// No-op required hook.
pub unsafe extern fn enumerate(_cx: *mut JSContext, _obj: *mut JSObject,
                               _v: *mut AutoIdVector) -> bool {
    true
}
