/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::is_dom_proxy;
use js::jsapi::{JSContext, jsid, JSPropertyDescriptor, JSObject, JSString, jschar};
use js::jsapi::{JS_GetPropertyDescriptorById, JS_NewUCString, JS_malloc, JS_free};
use js::jsapi::{JSBool, JS_DefinePropertyById, JS_NewObjectWithGivenProto};
use js::glue::{RUST_JSVAL_IS_VOID, RUST_JSVAL_TO_OBJECT, GetProxyExtra, RUST_OBJECT_TO_JSVAL};
use js::glue::{GetObjectProto, GetObjectParent, SetProxyExtra, GetProxyHandler};
use js::glue::InvokeGetOwnPropertyDescriptor;
use js::crust::{JS_StrictPropertyStub};
use js::{JSPROP_GETTER, JSPROP_ENUMERATE, JSPROP_READONLY, JSRESOLVE_QUALIFIED};

use std::cast;
use std::libc;
use std::ptr;
use std::str;
use std::sys::size_of;

type c_bool = libc::c_int;

static JSPROXYSLOT_EXPANDO: u32 = 0;

pub extern fn getPropertyDescriptor(cx: *JSContext, proxy: *JSObject, id: jsid,
                                set: c_bool, desc: *mut JSPropertyDescriptor) -> c_bool {
  unsafe {
    let handler = GetProxyHandler(proxy);
    if InvokeGetOwnPropertyDescriptor(handler, cx, proxy, id, set, desc) == 0 {
        return 0;
    }
    if (*desc).obj.is_not_null() {
        return 1;
    }

    //let proto = JS_GetPrototype(proxy);
    let proto = GetObjectProto(proxy);
    if proto.is_null() {
        (*desc).obj = ptr::null();
        return 1;
    }

    JS_GetPropertyDescriptorById(cx, proto, id, JSRESOLVE_QUALIFIED, cast::transmute(desc))
  }
}

pub extern fn defineProperty(cx: *JSContext, proxy: *JSObject, id: jsid,
                          desc: *JSPropertyDescriptor) -> JSBool {
    unsafe {
        if ((*desc).attrs & JSPROP_GETTER) != 0 && (*desc).setter == JS_StrictPropertyStub {
            /*return JS_ReportErrorFlagsAndNumber(cx,
            JSREPORT_WARNING | JSREPORT_STRICT |
            JSREPORT_STRICT_MODE_ERROR,
            js_GetErrorMessage, NULL,
            JSMSG_GETTER_ONLY);*/
            return 0;
        }

        let expando = EnsureExpandoObject(cx, proxy);
        if expando.is_null() {
            return 0;
        }

        return JS_DefinePropertyById(cx, expando, id, (*desc).value, (*desc).getter,
                                     (*desc).setter, (*desc).attrs);
    }
}

pub fn _obj_toString(cx: *JSContext, className: *libc::c_char) -> *JSString {
  unsafe {
    let name = str::raw::from_buf(className as *u8);
    let nchars = "[object ]".len() + name.len();
    let chars: *mut jschar = cast::transmute(JS_malloc(cx, (nchars + 1) as u64 * (size_of::<jschar>() as u64)));
    if chars.is_null() {
        return ptr::null();
    }

    let result = ~"[object " + name + "]";
    for result.iter().enumerate().advance |(i, c)| {
      *chars.offset(i) = c as jschar;
    }
    *chars.offset(nchars) = 0;
    let jsstr = JS_NewUCString(cx, cast::transmute(chars), nchars as u64);
    if jsstr.is_null() {
        JS_free(cx, cast::transmute(chars));
    }
    jsstr
  }
}

pub fn GetExpandoObject(obj: *JSObject) -> *JSObject {
    unsafe {
        assert!(is_dom_proxy(obj));
        let val = GetProxyExtra(obj, JSPROXYSLOT_EXPANDO);
        if RUST_JSVAL_IS_VOID(val) == 1 {
            ptr::null()
        } else {
            RUST_JSVAL_TO_OBJECT(val)
        }
    }
}

pub fn EnsureExpandoObject(cx: *JSContext, obj: *JSObject) -> *JSObject {
    unsafe {
        assert!(is_dom_proxy(obj));
        let mut expando = GetExpandoObject(obj);
        if expando.is_null() {
            expando = JS_NewObjectWithGivenProto(cx, ptr::null(), ptr::null(),
                                                 GetObjectParent(obj));
            if expando.is_null() {
                return ptr::null();
            }

            SetProxyExtra(obj, JSPROXYSLOT_EXPANDO, RUST_OBJECT_TO_JSVAL(expando));
        }
        return expando;
    }
}

pub fn FillPropertyDescriptor(desc: &mut JSPropertyDescriptor, obj: *JSObject, readonly: bool) {
    desc.obj = obj;
    desc.attrs = if readonly { JSPROP_READONLY } else { 0 } | JSPROP_ENUMERATE;
    desc.getter = ptr::null();
    desc.setter = ptr::null();
    desc.shortid = 0;
}
