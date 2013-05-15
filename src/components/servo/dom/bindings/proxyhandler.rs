/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use js::jsapi::{JSContext, jsid, JSPropertyDescriptor, JSObject, JSString, jschar};
use js::jsapi::bindgen::{JS_GetPropertyDescriptorById, JS_NewUCString, JS_malloc, JS_free};
use js::glue::bindgen::{RUST_JSVAL_IS_VOID, RUST_JSVAL_TO_OBJECT, GetProxyExtra};
use js::glue::bindgen::{GetObjectProto};

use core::sys::size_of;

type c_bool = libc::c_int;

pub extern fn getPropertyDescriptor(cx: *JSContext, proxy: *JSObject, id: jsid,
                                set: c_bool, desc: *mut JSPropertyDescriptor) -> c_bool {
  unsafe {
    if _getOwnPropertyDescriptor(cx, proxy, id, set, desc) == 0 {
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

    JS_GetPropertyDescriptorById(cx, proto, id, 0x01 /*JSRESOLVE_QUALIFIED*/,
                                 cast::transmute(desc))
  }
}

fn _getOwnPropertyDescriptor(cx: *JSContext, proxy: *JSObject, id: jsid,
                             _set: c_bool, desc: *mut JSPropertyDescriptor) -> c_bool {
  unsafe {
    let v = GetProxyExtra(proxy, 0 /*JSPROXYSLOT_EXPANDO*/);
    if RUST_JSVAL_IS_VOID(v) == 0 {
        let expando = RUST_JSVAL_TO_OBJECT(v);
        if JS_GetPropertyDescriptorById(cx, expando, id, 0x01 /*JSRESOLVE_QUALIFIED*/,
                                        cast::transmute(desc)) == 0 {
            return 0;
        }
        if (*desc).obj.is_not_null() {
            (*desc).obj = proxy;
            return 1;
        }
    }
    (*desc).obj = ptr::null();
    1
  }
}

pub extern fn getOwnPropertyDescriptor(cx: *JSContext, proxy: *JSObject, id: jsid,
                                   set: c_bool, desc: *mut JSPropertyDescriptor) -> c_bool {
    _getOwnPropertyDescriptor(cx, proxy, id, set, desc)
}

pub fn _obj_toString(cx: *JSContext, className: *libc::c_char) -> *JSString {
  unsafe {
    let name = str::raw::from_buf(className as *u8);
    let nchars = "[object ]".len() + name.len();
    let chars: *mut jschar = cast::transmute(JS_malloc(cx, (nchars + 1) as u64 * (size_of::<jschar>() as u64)));
    if chars.is_null() {
        return ptr::null();
    }

    let result = ~"[object " + name + ~"]";
    for result.each_chari |i, c| {
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

pub fn GetExpandoObject(_proxy: *JSObject) -> *JSObject {
    ptr::null()
}