/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Utilities for the implementation of JSAPI proxy handlers.

#![deny(missing_docs)]

use dom::bindings::conversions::is_dom_proxy;
use dom::bindings::utils::delete_property_by_id;
use js::glue::{GetProxyHandler, GetProxyHandlerFamily, SetProxyExtra};
use js::glue::GetProxyExtra;
use js::glue::InvokeGetOwnPropertyDescriptor;
use js::jsapi::{DOMProxyShadowsResult, JSContext, JSObject, JSPROP_GETTER, PropertyDescriptor};
use js::jsapi::{Handle, HandleId, HandleObject, MutableHandle, ObjectOpResult};
use js::jsapi::{JSErrNum, JS_AlreadyHasOwnPropertyById, JS_StrictPropertyStub};
use js::jsapi::{JS_DefinePropertyById, JS_NewObjectWithGivenProto, SetDOMProxyInformation};
use js::jsapi::GetObjectProto;
use js::jsapi::GetStaticPrototype;
use js::jsapi::JS_GetPropertyDescriptorById;
use js::jsapi::MutableHandleObject;
use js::jsval::ObjectValue;
use std::ptr;


static JSPROXYSLOT_EXPANDO: u32 = 0;

/// Determine if this id shadows any existing properties for this proxy.
pub unsafe extern "C" fn shadow_check_callback(cx: *mut JSContext,
                                               object: HandleObject,
                                               id: HandleId)
                                               -> DOMProxyShadowsResult {
    // TODO: support OverrideBuiltins when #12978 is fixed.

    rooted!(in(cx) let mut expando = ptr::null_mut());
    get_expando_object(object, expando.handle_mut());
    if !expando.get().is_null() {
        let mut has_own = false;
        if !JS_AlreadyHasOwnPropertyById(cx, expando.handle(), id, &mut has_own) {
            return DOMProxyShadowsResult::ShadowCheckFailed;
        }

        if has_own {
            return DOMProxyShadowsResult::ShadowsViaDirectExpando;
        }
    }

    // Our expando, if any, didn't shadow, so we're not shadowing at all.
    DOMProxyShadowsResult::DoesntShadow
}

/// Initialize the infrastructure for DOM proxy objects.
pub unsafe fn init() {
    SetDOMProxyInformation(GetProxyHandlerFamily(),
                           JSPROXYSLOT_EXPANDO,
                           Some(shadow_check_callback));
}

/// Invoke the [[GetOwnProperty]] trap (`getOwnPropertyDescriptor`) on `proxy`,
/// with argument `id` and return the result, if it is not `undefined`.
/// Otherwise, walk along the prototype chain to find a property with that
/// name.
pub unsafe extern "C" fn get_property_descriptor(cx: *mut JSContext,
                                                 proxy: HandleObject,
                                                 id: HandleId,
                                                 desc: MutableHandle<PropertyDescriptor>)
                                                 -> bool {
    let handler = GetProxyHandler(proxy.get());
    if !InvokeGetOwnPropertyDescriptor(handler, cx, proxy, id, desc) {
        return false;
    }
    if !desc.obj.is_null() {
        return true;
    }

    rooted!(in(cx) let mut proto = ptr::null_mut());
    if !GetObjectProto(cx, proxy, proto.handle_mut()) {
        // FIXME(#11868) Should assign to desc.obj, desc.get() is a copy.
        desc.get().obj = ptr::null_mut();
        return true;
    }

    JS_GetPropertyDescriptorById(cx, proto.handle(), id, desc)
}

/// Defines an expando on the given `proxy`.
pub unsafe extern "C" fn define_property(cx: *mut JSContext,
                                         proxy: HandleObject,
                                         id: HandleId,
                                         desc: Handle<PropertyDescriptor>,
                                         result: *mut ObjectOpResult)
                                         -> bool {
    if (desc.get().attrs & JSPROP_GETTER) != 0 && desc.get().setter == Some(JS_StrictPropertyStub) {
        (*result).code_ = JSErrNum::JSMSG_GETTER_ONLY as ::libc::uintptr_t;
        return true;
    }

    rooted!(in(cx) let mut expando = ptr::null_mut());
    ensure_expando_object(cx, proxy, expando.handle_mut());
    JS_DefinePropertyById(cx, expando.handle(), id, desc, result)
}

/// Deletes an expando off the given `proxy`.
pub unsafe extern "C" fn delete(cx: *mut JSContext,
                                proxy: HandleObject,
                                id: HandleId,
                                bp: *mut ObjectOpResult)
                                -> bool {
    rooted!(in(cx) let mut expando = ptr::null_mut());
    get_expando_object(proxy, expando.handle_mut());
    if expando.is_null() {
        (*bp).code_ = 0 /* OkCode */;
        return true;
    }

    delete_property_by_id(cx, expando.handle(), id, bp)
}

/// Controls whether the Extensible bit can be changed
pub unsafe extern "C" fn prevent_extensions(_cx: *mut JSContext,
                                            _proxy: HandleObject,
                                            result: *mut ObjectOpResult)
                                            -> bool {
    (*result).code_ = JSErrNum::JSMSG_CANT_PREVENT_EXTENSIONS as ::libc::uintptr_t;
    true
}

/// Reports whether the object is Extensible
pub unsafe extern "C" fn is_extensible(_cx: *mut JSContext,
                                       _proxy: HandleObject,
                                       succeeded: *mut bool)
                                       -> bool {
    *succeeded = true;
    true
}

/// If `proxy` (underneath any functionally-transparent wrapper proxies) has as
/// its `[[GetPrototypeOf]]` trap the ordinary `[[GetPrototypeOf]]` behavior
/// defined for ordinary objects, set `*is_ordinary` to true and store `obj`'s
/// prototype in `proto`.  Otherwise set `*isOrdinary` to false. In case of
/// error, both outparams have unspecified value.
///
/// This implementation always handles the case of the ordinary
/// `[[GetPrototypeOf]]` behavior. An alternative implementation will be
/// necessary for the Location object.
pub unsafe extern "C" fn get_prototype_if_ordinary(_: *mut JSContext,
                                                   proxy: HandleObject,
                                                   is_ordinary: *mut bool,
                                                   proto: MutableHandleObject)
                                                   -> bool {
    *is_ordinary = true;
    proto.set(GetStaticPrototype(proxy.get()));
    true
}

/// Get the expando object, or null if there is none.
pub unsafe fn get_expando_object(obj: HandleObject, expando: MutableHandleObject) {
    assert!(is_dom_proxy(obj.get()));
    let val = GetProxyExtra(obj.get(), JSPROXYSLOT_EXPANDO);
    expando.set(if val.is_undefined() {
        ptr::null_mut()
    } else {
        val.to_object()
    });
}

/// Get the expando object, or create it if it doesn't exist yet.
/// Fails on JSAPI failure.
pub unsafe fn ensure_expando_object(cx: *mut JSContext, obj: HandleObject, expando: MutableHandleObject) {
    assert!(is_dom_proxy(obj.get()));
    get_expando_object(obj, expando);
    if expando.is_null() {
        expando.set(JS_NewObjectWithGivenProto(cx, ptr::null_mut(), HandleObject::null()));
        assert!(!expando.is_null());

        SetProxyExtra(obj.get(), JSPROXYSLOT_EXPANDO, &ObjectValue(expando.get()));
    }
}

/// Set the property descriptor's object to `obj` and set it to enumerable,
/// and writable if `readonly` is true.
pub fn fill_property_descriptor(mut desc: MutableHandle<PropertyDescriptor>,
                                obj: *mut JSObject,
                                attrs: u32) {
    desc.obj = obj;
    desc.attrs = attrs;
    desc.getter = None;
    desc.setter = None;
}
