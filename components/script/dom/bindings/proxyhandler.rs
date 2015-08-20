/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Utilities for the implementation of JSAPI proxy handlers.

#![deny(missing_docs)]

use dom::bindings::conversions::is_dom_proxy;
use dom::bindings::utils::delete_property_by_id;
use js::glue::GetProxyExtra;
use js::glue::InvokeGetOwnPropertyDescriptor;
use js::glue::{SetProxyExtra, GetProxyHandler};
use js::jsapi::GetObjectProto;
use js::jsapi::{Handle, HandleObject, HandleId, MutableHandle, RootedObject, ObjectOpResult};
use js::jsapi::{JSContext, JSPropertyDescriptor, JSObject};
use js::jsapi::{JS_DefinePropertyById6, JS_NewObjectWithGivenProto};
use js::jsapi::{JS_GetPropertyDescriptorById};
use js::jsapi::{JS_StrictPropertyStub, JSErrNum};
use js::jsval::ObjectValue;
use js::{JSPROP_GETTER, JSPROP_ENUMERATE, JSPROP_READONLY};
use js::{JSTrue, JSFalse};

use libc;
use std::mem;
use std::ptr;

static JSPROXYSLOT_EXPANDO: u32 = 0;

/// Invoke the [[GetOwnProperty]] trap (`getOwnPropertyDescriptor`) on `proxy`,
/// with argument `id` and return the result, if it is not `undefined`.
/// Otherwise, walk along the prototype chain to find a property with that
/// name.
pub unsafe extern fn get_property_descriptor(cx: *mut JSContext,
                                             proxy: HandleObject,
                                             id: HandleId,
                                             desc: MutableHandle<JSPropertyDescriptor>)
                                             -> u8 {
    let handler = GetProxyHandler(proxy.get());
    if InvokeGetOwnPropertyDescriptor(handler, cx, proxy, id, desc) == 0 {
        return JSFalse;
    }
    if !desc.get().obj.is_null() {
        return JSTrue;
    }

    let mut proto = RootedObject::new(cx, ptr::null_mut());
    if GetObjectProto(cx, proxy, proto.handle_mut()) == 0 {
        desc.get().obj = ptr::null_mut();
        return JSTrue;
    }

    JS_GetPropertyDescriptorById(cx, proto.handle(), id, desc)
}

/// Defines an expando on the given `proxy`.
pub unsafe extern fn define_property(cx: *mut JSContext, proxy: HandleObject,
                                     id: HandleId, desc: Handle<JSPropertyDescriptor>,
                                     result: *mut ObjectOpResult)
                                     -> u8 {
    //FIXME: Workaround for https://github.com/mozilla/rust/issues/13385
    let setter: *const libc::c_void = mem::transmute(desc.get().setter);
    let setter_stub: *const libc::c_void = mem::transmute(JS_StrictPropertyStub);
    if (desc.get().attrs & JSPROP_GETTER) != 0 && setter == setter_stub {
        (*result).code_ = JSErrNum::JSMSG_GETTER_ONLY as u32;
        return JSTrue;
    }

    let expando = RootedObject::new(cx, ensure_expando_object(cx, proxy));
    JS_DefinePropertyById6(cx, expando.handle(), id, desc, result)
}

/// Deletes an expando off the given `proxy`.
pub unsafe extern fn delete(cx: *mut JSContext, proxy: HandleObject, id: HandleId,
                            bp: *mut ObjectOpResult) -> u8 {
    let expando = RootedObject::new(cx, get_expando_object(proxy));
    if expando.ptr.is_null() {
        (*bp).code_ = 0 /* OkCode */;
        return JSTrue;
    }

    delete_property_by_id(cx, expando.handle(), id, bp)
}

/// Controls whether the Extensible bit can be changed
pub unsafe extern fn prevent_extensions(_cx: *mut JSContext,
                                        _proxy: HandleObject,
                                        result: *mut ObjectOpResult) -> u8 {
    (*result).code_ = JSErrNum::JSMSG_CANT_PREVENT_EXTENSIONS as u32;
    return JSTrue;
}

/// Reports whether the object is Extensible
pub unsafe extern fn is_extensible(_cx: *mut JSContext, _proxy: HandleObject,
                                   succeeded: *mut u8) -> u8 {
    *succeeded = JSTrue;
    return JSTrue;
}

/// Get the expando object, or null if there is none.
pub fn get_expando_object(obj: HandleObject) -> *mut JSObject {
    unsafe {
        assert!(is_dom_proxy(obj.get()));
        let val = GetProxyExtra(obj.get(), JSPROXYSLOT_EXPANDO);
        if val.is_undefined() {
            ptr::null_mut()
        } else {
            val.to_object()
        }
    }
}

/// Get the expando object, or create it if it doesn't exist yet.
/// Fails on JSAPI failure.
pub fn ensure_expando_object(cx: *mut JSContext, obj: HandleObject)
                             -> *mut JSObject {
    unsafe {
        assert!(is_dom_proxy(obj.get()));
        let mut expando = get_expando_object(obj);
        if expando.is_null() {
            expando = JS_NewObjectWithGivenProto(cx, ptr::null_mut(), HandleObject::null());
            assert!(!expando.is_null());

            SetProxyExtra(obj.get(), JSPROXYSLOT_EXPANDO, ObjectValue(&*expando));
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
}
