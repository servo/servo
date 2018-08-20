/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Utilities for the implementation of JSAPI proxy handlers.

#![deny(missing_docs)]

use dom::bindings::conversions::is_dom_proxy;
use dom::bindings::utils::delete_property_by_id;
use js::glue::{GetProxyHandler, GetProxyHandlerFamily};
use js::glue::{GetProxyPrivate, SetProxyPrivate};
use js::glue::InvokeGetOwnPropertyDescriptor;
use js::jsapi::{DOMProxyShadowsResult, JSContext, JSObject, PropertyDescriptor};
use js::jsapi::{JSErrNum, SetDOMProxyInformation};
use js::jsapi::GetObjectProto;
use js::jsapi::GetStaticPrototype;
use js::jsapi::Handle as RawHandle;
use js::jsapi::HandleId as RawHandleId;
use js::jsapi::HandleObject as RawHandleObject;
use js::jsapi::JS_DefinePropertyById;
use js::jsapi::JS_GetPropertyDescriptorById;
use js::jsapi::MutableHandle as RawMutableHandle;
use js::jsapi::MutableHandleObject as RawMutableHandleObject;
use js::jsapi::ObjectOpResult;
use js::jsval::ObjectValue;
use js::jsval::UndefinedValue;
use js::rust::{Handle, HandleObject, MutableHandle, MutableHandleObject};
use js::rust::wrappers::JS_AlreadyHasOwnPropertyById;
use js::rust::wrappers::JS_NewObjectWithGivenProto;
use std::ptr;


/// Determine if this id shadows any existing properties for this proxy.
pub unsafe extern "C" fn shadow_check_callback(cx: *mut JSContext,
                                               object: RawHandleObject,
                                               id: RawHandleId)
                                               -> DOMProxyShadowsResult {
    // TODO: support OverrideBuiltins when #12978 is fixed.

    rooted!(in(cx) let mut expando = ptr::null_mut::<JSObject>());
    get_expando_object(object, expando.handle_mut());
    if !expando.get().is_null() {
        let mut has_own = false;
        let raw_id = Handle::from_raw(id);

        if !JS_AlreadyHasOwnPropertyById(cx, expando.handle(), raw_id, &mut has_own) {
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
                           Some(shadow_check_callback));
}

/// Invoke the [[GetOwnProperty]] trap (`getOwnPropertyDescriptor`) on `proxy`,
/// with argument `id` and return the result, if it is not `undefined`.
/// Otherwise, walk along the prototype chain to find a property with that
/// name.
pub unsafe extern "C" fn get_property_descriptor(cx: *mut JSContext,
                                                 proxy: RawHandleObject,
                                                 id: RawHandleId,
                                                 desc: RawMutableHandle<PropertyDescriptor>)
                                                 -> bool {
    let handler = GetProxyHandler(proxy.get());
    if !InvokeGetOwnPropertyDescriptor(handler, cx, proxy, id, desc) {
        return false;
    }
    if !desc.obj.is_null() {
        return true;
    }

    rooted!(in(cx) let mut proto = ptr::null_mut::<JSObject>());
    if !GetObjectProto(cx, proxy, proto.handle_mut().into()) {
        // FIXME(#11868) Should assign to desc.obj, desc.get() is a copy.
        desc.get().obj = ptr::null_mut();
        return true;
    }

    JS_GetPropertyDescriptorById(cx, proto.handle().into(), id, desc)
}

/// Defines an expando on the given `proxy`.
pub unsafe extern "C" fn define_property(cx: *mut JSContext,
                                         proxy: RawHandleObject,
                                         id: RawHandleId,
                                         desc: RawHandle<PropertyDescriptor>,
                                         result: *mut ObjectOpResult)
                                         -> bool {
    rooted!(in(cx) let mut expando = ptr::null_mut::<JSObject>());
    ensure_expando_object(cx, proxy, expando.handle_mut());
    JS_DefinePropertyById(cx, expando.handle().into(), id, desc, result)
}

/// Deletes an expando off the given `proxy`.
pub unsafe extern "C" fn delete(cx: *mut JSContext,
                                proxy: RawHandleObject,
                                id: RawHandleId,
                                bp: *mut ObjectOpResult)
                                -> bool {
    rooted!(in(cx) let mut expando = ptr::null_mut::<JSObject>());
    get_expando_object(proxy, expando.handle_mut());
    if expando.is_null() {
        (*bp).code_ = 0 /* OkCode */;
        return true;
    }

    delete_property_by_id(cx, expando.handle(), Handle::from_raw(id), bp)
}

/// Controls whether the Extensible bit can be changed
pub unsafe extern "C" fn prevent_extensions(_cx: *mut JSContext,
                                            _proxy: RawHandleObject,
                                            result: *mut ObjectOpResult)
                                            -> bool {
    (*result).code_ = JSErrNum::JSMSG_CANT_PREVENT_EXTENSIONS as ::libc::uintptr_t;
    true
}

/// Reports whether the object is Extensible
pub unsafe extern "C" fn is_extensible(_cx: *mut JSContext,
                                       _proxy: RawHandleObject,
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
                                                   proxy: RawHandleObject,
                                                   is_ordinary: *mut bool,
                                                   proto: RawMutableHandleObject)
                                                   -> bool {
    *is_ordinary = true;
    proto.set(GetStaticPrototype(proxy.get()));
    true
}

/// Get the expando object, or null if there is none.
pub unsafe fn get_expando_object(obj: RawHandleObject, mut expando: MutableHandleObject) {
    assert!(is_dom_proxy(obj.get()));
    let ref mut val = UndefinedValue();
    GetProxyPrivate(obj.get(), val);
    expando.set(if val.is_undefined() {
        ptr::null_mut()
    } else {
        val.to_object()
    });
}

/// Get the expando object, or create it if it doesn't exist yet.
/// Fails on JSAPI failure.
pub unsafe fn ensure_expando_object(cx: *mut JSContext, obj: RawHandleObject, mut expando: MutableHandleObject) {
    assert!(is_dom_proxy(obj.get()));
    get_expando_object(obj, expando);
    if expando.is_null() {
        expando.set(JS_NewObjectWithGivenProto(cx, ptr::null_mut(), HandleObject::null()));
        assert!(!expando.is_null());

        SetProxyPrivate(obj.get(), &ObjectValue(expando.get()));
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
