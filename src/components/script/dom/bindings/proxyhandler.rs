/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::object_handle;
use js::jsapi::{JSContext, JSPropertyDescriptor, JSObject, JSString, jschar};
use js::jsapi::{JS_GetPropertyDescriptorById, JS_NewUCString, JS_malloc, JS_free};
use js::jsapi::{JSBool, JS_DefinePropertyById, JS_NewObjectWithGivenProto};
use js::jsapi::{JS_StrictPropertyStub, JSHandleObject, JSHandleId, JSMutableHandleObject};
use js::jsval::ObjectValue;
use js::glue::{GetProxyExtra, JSMutableHandle};
use js::glue::{GetObjectProto, GetObjectParent, SetProxyExtra, GetProxyHandler};
use js::glue::InvokeGetOwnPropertyDescriptor;
use js::{JSPROP_GETTER, JSPROP_ENUMERATE, JSPROP_READONLY, JSRESOLVE_QUALIFIED};

use libc;
use std::cast;
use std::ptr;
use std::str;
use std::mem::size_of;

static JSPROXYSLOT_EXPANDO: u32 = 0;

pub extern fn getPropertyDescriptor(cx: *mut JSContext, proxy: JSHandleObject, id: JSHandleId,
                                    desc: JSMutableHandle<JSPropertyDescriptor>, flags: u32) -> libc::c_int {
  unsafe {
    let handler = GetProxyHandler(proxy);
    if InvokeGetOwnPropertyDescriptor(handler, cx, proxy, id, desc, flags) == 0 {
        return 0;
    }
    if (*desc.unnamed_field1).is_not_null() {
        return 1;
    }

    //let proto = JS_GetPrototype(proxy);
    let proto = JSMutableHandleObject {
        unnamed_field1: &mut ptr::mut_null(),
    };
    assert!(GetObjectProto(cx, proxy, proto) != 0);
    if (*proto.unnamed_field1).is_null() {
        *desc.unnamed_field1 = ptr::mut_null();
        return 1;
    }

    JS_GetPropertyDescriptorById(cx, proto.immut(), id, JSRESOLVE_QUALIFIED, desc)
  }
}

pub fn defineProperty_(cx: *mut JSContext, proxy: JSHandleObject, id: JSHandleId,
                       desc: JSMutableHandle<JSPropertyDescriptor>) -> JSBool {
    unsafe {
        //FIXME: Workaround for https://github.com/mozilla/rust/issues/13385
        let setter: *libc::c_void = cast::transmute((**desc.unnamed_field1).setter);
        let setter_stub: *libc::c_void = cast::transmute(JS_StrictPropertyStub);
        if ((**desc.unnamed_field1).attrs & JSPROP_GETTER) != 0 && setter == setter_stub {
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

        return JS_DefinePropertyById(cx, expando, *id.unnamed_field1,
                                     (**desc.unnamed_field1).value,
                                     (**desc.unnamed_field1).getter,
                                     (**desc.unnamed_field1).setter,
                                     (**desc.unnamed_field1).attrs);
    }
}

pub extern fn defineProperty(cx: *mut JSContext, proxy: JSHandleObject, id: JSHandleId,
                             desc: JSMutableHandle<JSPropertyDescriptor>) -> JSBool {
    defineProperty_(cx, proxy, id, desc)
}

pub fn _obj_toString(cx: *mut JSContext, className: *libc::c_char) -> *mut JSString {
  unsafe {
    let name = str::raw::from_c_str(className);
    let nchars = "[object ]".len() + name.len();
    let chars = JS_malloc(cx, (nchars + 1) as libc::size_t * (size_of::<jschar>() as libc::size_t)) as *mut jschar;
    if chars.is_null() {
        return ptr::mut_null();
    }

    let result = "[object ".to_owned() + name + "]";
    for (i, c) in result.chars().enumerate() {
      *chars.offset(i as int) = c as jschar;
    }
    *chars.offset(nchars as int) = 0;
    let jsstr = JS_NewUCString(cx, chars, nchars as libc::size_t);
    if jsstr.is_null() {
        JS_free(cx, chars as *mut libc::c_void);
    }
    jsstr
  }
}

pub fn GetExpandoObject(obj: JSHandleObject) -> *mut JSObject {
    unsafe {
        //XXXjdm it would be nice to assert that obj's class is a proxy class
        let val = GetProxyExtra(obj, JSPROXYSLOT_EXPANDO);
        if val.is_undefined() {
            ptr::mut_null()
        } else {
            val.to_object()
        }
    }
}

pub fn EnsureExpandoObject(cx: *mut JSContext, obj: JSHandleObject) -> *mut JSObject {
    unsafe {
        //XXXjdm it would be nice to assert that obj's class is a proxy class
        let mut expando = GetExpandoObject(obj);
        if expando.is_null() {
            expando = JS_NewObjectWithGivenProto(cx, ptr::null(), object_handle(&ptr::mut_null()),
                                                 object_handle(&GetObjectParent(*obj.unnamed_field1)));
            if expando.is_null() {
                return ptr::mut_null();
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
}
