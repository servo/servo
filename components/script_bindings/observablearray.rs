/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ffi::c_void;
use std::ptr;
use std::ptr::NonNull;

use js::context::{JSContext, RawJSContext};
use js::conversions::ToJSValConvertible;
use js::glue::{
    AppendToIdVector, CreateProxyHandler, GetProxyHandlerExtra, GetProxyPrivate,
    GetProxyReservedSlot, JS_GetReservedSlot, ProxyTraps, SetProxyReservedSlot,
};
use js::jsapi::{
    Handle as RawHandle, HandleId as RawHandleId, HandleObject as RawHandleObject,
    HandleValue as RawHandleValue, HandleValueArray, JS_SetReservedSlot, JSITER_HIDDEN,
    JSITER_OWNONLY, JSITER_SYMBOLS, JSObject, JSPROP_ENUMERATE, JSPROP_PERMANENT,
    MutableHandle as RawMutableHandle, MutableHandleIdVector as RawMutableHandleIdVector,
    MutableHandleValue as RawMutableHandleValue, ObjectOpResult, PropertyDescriptor, jsid,
};
use js::jsval::{ObjectValue, PrivateValue, UndefinedValue};
use js::rust::wrappers2::{
    GetArrayLength, GetPropertyKeys, JS_DefinePropertyById, JS_DeletePropertyById,
    JS_ForwardGetPropertyTo, JS_ForwardSetPropertyTo, JS_GetElement,
    JS_GetOwnPropertyDescriptorById, JS_HasPropertyById, JS_SetPrototype, NewArrayObject,
    SetArrayLength, int_to_jsid,
};
use js::rust::{
    Handle, HandleId, HandleObject, HandleValue, MutableHandle, MutableHandleObject,
    MutableHandleValue, ToNumber, ToUint32,
};

use crate::conversions::jsid_to_string;
use crate::proxyhandler::set_property_descriptor;
use crate::utils::get_array_index_from_id;

// Adapted from https://searchfox.org/firefox-main/rev/c681e91369f59d0efae43bdc465872b855e8b269/dom/bindings/ObservableArrayProxyHandler.cpp

pub const OBSERVABLE_ARRAY_OWNER_SLOT: u32 = 0;
pub const OBSERVABLE_ARRAY_BACKING_LIST_OBJECT_SLOT: u32 = 1;

pub struct ObservableArrayProxyHandlerConfig {
    pub on_delete_item: unsafe fn(&mut JSContext, HandleObject, HandleValue, u32) -> bool,
    pub set_indexed_value: unsafe fn(
        &mut JSContext,
        HandleObject,
        HandleObject,
        u32,
        HandleValue,
        *mut ObjectOpResult,
    ) -> bool,
}

static PROXY_TRAPS: ProxyTraps = ProxyTraps {
    enter: None,
    getOwnPropertyDescriptor: Some(get_own_property_descriptor),
    defineProperty: Some(define_property),
    ownPropertyKeys: Some(own_property_keys),
    delete_: Some(delete_),
    enumerate: None,
    getPrototypeIfOrdinary: None,
    getPrototype: None,
    setPrototype: None,
    setImmutablePrototype: None,
    preventExtensions: Some(prevent_extensions),
    isExtensible: None,
    has: Some(has),
    get: Some(get),
    set: Some(set),
    call: None,
    construct: None,
    hasOwn: None,
    getOwnEnumerablePropertyKeys: None,
    nativeCall: None,
    objectClassIs: None,
    className: None,
    fun_toString: None,
    boxedValue_unbox: None,
    defaultValue: None,
    trace: None,
    finalize: None,
    objectMoved: None,
    isCallable: None,
    isConstructor: None,
};

pub fn create_proxy_handler(config: *const ObservableArrayProxyHandlerConfig) -> *const c_void {
    unsafe { CreateProxyHandler(&PROXY_TRAPS, config as *const c_void) }
}

/// # Safety
/// The caller must ensure that the returned pointer is not used after the `JSContext` is deallocated.
pub unsafe fn new_proxy_object(
    cx: &mut JSContext,
    handler: *const c_void,
    owner: *const c_void,
) -> *mut JSObject {
    rooted!(&in(cx) let target = unsafe { NewArrayObject(cx, &HandleValueArray::empty()) });
    if target.is_null() {
        return ptr::null_mut();
    }

    rooted!(&in(cx) let target_value = ObjectValue(target.get()));
    let proxy = unsafe {
        js::rust::wrappers2::NewProxyObject(
            cx,
            handler,
            target_value.handle(),
            ptr::null_mut(),
            ptr::null(),
            true,
        )
    };
    if proxy.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        SetProxyReservedSlot(proxy, OBSERVABLE_ARRAY_OWNER_SLOT, &PrivateValue(owner));
    }
    proxy
}

/// # Safety
/// The caller must ensure that the returned pointer is not used after the `JSContext` is deallocated.
pub unsafe fn get_or_create_proxy_object(
    cx: &mut JSContext,
    obj: HandleObject,
    is_proxy: bool,
    slot: u32,
    handler: *const c_void,
    owner: *const c_void,
    mut proxy: MutableHandleObject,
) -> bool {
    let mut slot_value = UndefinedValue();
    unsafe {
        if is_proxy {
            GetProxyReservedSlot(obj.get(), slot, &mut slot_value);
        } else {
            JS_GetReservedSlot(obj.get(), slot, &mut slot_value);
        }
    }

    if slot_value.is_undefined() {
        let new_proxy = unsafe { new_proxy_object(cx, handler, owner) };
        if new_proxy.is_null() {
            return false;
        }

        let value = ObjectValue(new_proxy);
        unsafe {
            if is_proxy {
                SetProxyReservedSlot(obj.get(), slot, &value);
            } else {
                JS_SetReservedSlot(obj.get(), slot, &value);
            }
        }
        proxy.set(new_proxy);
        return true;
    }

    proxy.set(slot_value.to_object());
    true
}

/// # Safety
/// The caller must ensure that the returned pointer is not used after the `JSContext` is deallocated.
pub unsafe fn clear_owner_slot(proxy: *mut JSObject) {
    unsafe {
        SetProxyReservedSlot(proxy, OBSERVABLE_ARRAY_OWNER_SLOT, &UndefinedValue());
    }
}

pub fn set_length(cx: &mut JSContext, proxy: HandleObject, length: u32) -> bool {
    let backing_list: *mut JSObject = ptr::null_mut();
    rooted!(&in(cx) let mut rooted_backing_list = backing_list);
    if !get_backing_list_object(cx, proxy, rooted_backing_list.handle_mut()) {
        return false;
    }

    set_length_internal(cx, proxy, rooted_backing_list.handle(), length)
}

fn config_for_proxy(proxy: HandleObject) -> &'static ObservableArrayProxyHandlerConfig {
    let extra = unsafe { GetProxyHandlerExtra(proxy.get()) };
    unsafe { &*(extra as *const ObservableArrayProxyHandlerConfig) }
}

fn is_length_id(cx: &JSContext, id: HandleId) -> bool {
    matches!(jsid_to_string(cx, id), Some(ref name) if name == "length")
}

fn get_proxy_target(proxy: HandleObject, mut target: MutableHandleObject) {
    let mut slot = UndefinedValue();
    unsafe { GetProxyPrivate(proxy.get(), &mut slot) };
    debug_assert!(slot.is_object());
    target.set(slot.to_object());
}

fn get_backing_list_object(
    cx: &mut JSContext,
    proxy: HandleObject,
    mut backing_list: MutableHandleObject,
) -> bool {
    let mut slot_value = UndefinedValue();
    unsafe {
        GetProxyReservedSlot(
            proxy.get(),
            OBSERVABLE_ARRAY_BACKING_LIST_OBJECT_SLOT,
            &mut slot_value,
        );
    }

    if slot_value.is_undefined() {
        rooted!(&in(cx) let new_backing_list = unsafe { NewArrayObject(cx, &HandleValueArray::empty()) });
        if new_backing_list.is_null() {
            return false;
        }
        if unsafe { !JS_SetPrototype(cx, new_backing_list.handle(), HandleObject::null()) } {
            return false;
        }

        let value = ObjectValue(new_backing_list.get());
        unsafe {
            SetProxyReservedSlot(
                proxy.get(),
                OBSERVABLE_ARRAY_BACKING_LIST_OBJECT_SLOT,
                &value,
            );
        }
        backing_list.set(new_backing_list.get());
        return true;
    }

    backing_list.set(slot_value.to_object());
    true
}

fn get_backing_list_length(cx: &mut JSContext, proxy: HandleObject, length: &mut u32) -> bool {
    rooted!(&in(cx) let mut backing_list = ptr::null_mut::<JSObject>());
    if !get_backing_list_object(cx, proxy, backing_list.handle_mut()) {
        return false;
    }
    unsafe { GetArrayLength(cx, backing_list.handle(), length) }
}

fn set_length_internal(
    cx: &mut JSContext,
    proxy: HandleObject,
    backing_list: HandleObject,
    length: u32,
) -> bool {
    let mut old_len = 0;
    if unsafe { !GetArrayLength(cx, backing_list, &mut old_len) } {
        return false;
    }

    if length > old_len {
        return false;
    }

    let config = config_for_proxy(proxy);
    let mut current_len = old_len;
    while current_len > length {
        let index_to_delete = current_len - 1;
        rooted!(&in(cx) let mut value = UndefinedValue());
        if unsafe { !JS_GetElement(cx, backing_list, index_to_delete, value.handle_mut()) } {
            return false;
        }
        if unsafe { !(config.on_delete_item)(cx, proxy, value.handle(), index_to_delete) } {
            return false;
        }
        current_len -= 1;
    }

    unsafe { SetArrayLength(cx, backing_list, current_len) }
}

fn set_length_from_value(
    cx: &mut JSContext,
    proxy: HandleObject,
    backing_list: HandleObject,
    value: HandleValue,
    result: *mut ObjectOpResult,
) -> bool {
    let uint32_len = match unsafe { ToUint32(cx.raw_cx(), value) } {
        Ok(length) => length,
        Err(()) => return false,
    };
    let number_len = match unsafe { ToNumber(cx.raw_cx(), value) } {
        Ok(length) => length,
        Err(()) => return false,
    };

    if f64::from(uint32_len) != number_len {
        unsafe {
            (*result).fail_bad_index();
        }
        return true;
    }

    let mut old_len = 0;
    if unsafe { !GetArrayLength(cx, backing_list, &mut old_len) } {
        return false;
    }
    if uint32_len > old_len {
        unsafe {
            (*result).fail_bad_array_length();
        }
        return true;
    }

    if !set_length_internal(cx, proxy, backing_list, uint32_len) {
        return false;
    }

    unsafe { (*result).succeed() }
}

// https://searchfox.org/firefox-main/rev/c681e91369f59d0efae43bdc465872b855e8b269/dom/bindings/ObservableArrayProxyHandler.cpp#22
unsafe extern "C" fn define_property(
    cx: *mut RawJSContext,
    proxy: RawHandleObject,
    id: RawHandleId,
    desc: RawHandle<PropertyDescriptor>,
    result: *mut ObjectOpResult,
) -> bool {
    let mut cx = JSContext::from_ptr(NonNull::new(cx).unwrap());
    let proxy = HandleObject::from_raw(proxy);
    let id = Handle::from_raw(id);
    let desc = Handle::from_raw(desc);

    if is_length_id(&cx, id) {
        if desc.hasSetter_() || desc.hasGetter_() {
            return (*result).fail_not_data_descriptor();
        }
        if desc.hasConfigurable_() && desc.configurable_() {
            return (*result).fail_invalid_descriptor();
        }
        if desc.hasEnumerable_() && desc.enumerable_() {
            return (*result).fail_invalid_descriptor();
        }
        if desc.hasWritable_() && !desc.writable_() {
            return (*result).fail_invalid_descriptor();
        }
        if desc.hasValue_() {
            rooted!(&in(cx) let mut backing_list = ptr::null_mut::<JSObject>());
            if !get_backing_list_object(&mut cx, proxy, backing_list.handle_mut()) {
                return false;
            }
            rooted!(&in(cx) let desc_value = desc.value_);
            return set_length_from_value(
                &mut cx,
                proxy,
                backing_list.handle(),
                desc_value.handle(),
                result,
            );
        }
        return (*result).succeed();
    }

    if let Some(index) = get_array_index_from_id(id) {
        if desc.hasSetter_() || desc.hasGetter_() {
            return (*result).fail_not_data_descriptor();
        }
        if desc.hasConfigurable_() && !desc.configurable_() {
            return (*result).fail_invalid_descriptor();
        }
        if desc.hasEnumerable_() && !desc.enumerable_() {
            return (*result).fail_invalid_descriptor();
        }
        if desc.hasWritable_() && !desc.writable_() {
            return (*result).fail_invalid_descriptor();
        }
        if desc.hasValue_() {
            rooted!(&in(cx) let mut backing_list = ptr::null_mut::<JSObject>());
            if !get_backing_list_object(&mut cx, proxy, backing_list.handle_mut()) {
                return false;
            }
            rooted!(&in(cx) let desc_value = desc.value_);
            return (config_for_proxy(proxy).set_indexed_value)(
                &mut cx,
                proxy,
                backing_list.handle(),
                index,
                desc_value.handle(),
                result,
            );
        }
        return (*result).succeed();
    }

    rooted!(&in(cx) let mut target = ptr::null_mut::<JSObject>());
    get_proxy_target(proxy, target.handle_mut());
    JS_DefinePropertyById(&mut cx, target.handle(), id, desc, result)
}

// https://searchfox.org/firefox-main/rev/c681e91369f59d0efae43bdc465872b855e8b269/dom/bindings/ObservableArrayProxyHandler.cpp#79
unsafe extern "C" fn delete_(
    cx: *mut RawJSContext,
    proxy: RawHandleObject,
    id: RawHandleId,
    result: *mut ObjectOpResult,
) -> bool {
    let mut cx = JSContext::from_ptr(NonNull::new(cx).unwrap());
    let proxy = HandleObject::from_raw(proxy);
    let id = Handle::from_raw(id);

    if is_length_id(&cx, id) {
        return (*result).fail_cant_delete();
    }

    if let Some(index) = get_array_index_from_id(id) {
        rooted!(&in(cx) let mut backing_list = ptr::null_mut::<JSObject>());
        if !get_backing_list_object(&mut cx, proxy, backing_list.handle_mut()) {
            return false;
        }

        let mut old_len = 0;
        if !GetArrayLength(&mut cx, backing_list.handle(), &mut old_len) {
            return false;
        }

        if old_len != index + 1 {
            return (*result).fail_bad_index();
        }

        rooted!(&in(cx) let mut value = UndefinedValue());
        if !JS_GetElement(&mut cx, backing_list.handle(), index, value.handle_mut()) {
            return false;
        }

        if !(config_for_proxy(proxy).on_delete_item)(&mut cx, proxy, value.handle(), index) {
            return false;
        }

        if !SetArrayLength(&mut cx, backing_list.handle(), index) {
            return false;
        }

        return (*result).succeed();
    }

    rooted!(&in(cx) let mut target = ptr::null_mut::<JSObject>());
    get_proxy_target(proxy, target.handle_mut());
    JS_DeletePropertyById(&mut cx, target.handle(), id, result)
}

// https://searchfox.org/firefox-main/rev/c681e91369f59d0efae43bdc465872b855e8b269/dom/bindings/ObservableArrayProxyHandler.cpp#125
unsafe extern "C" fn get(
    cx: *mut RawJSContext,
    proxy: RawHandleObject,
    receiver: RawHandleValue,
    id: RawHandleId,
    vp: RawMutableHandleValue,
) -> bool {
    let mut cx = JSContext::from_ptr(NonNull::new(cx).unwrap());
    let proxy = HandleObject::from_raw(proxy);
    let receiver = HandleValue::from_raw(receiver);
    let id = Handle::from_raw(id);
    let mut vp = MutableHandleValue::from_raw(vp);

    rooted!(&in(cx) let mut backing_list = ptr::null_mut::<JSObject>());
    if !get_backing_list_object(&mut cx, proxy, backing_list.handle_mut()) {
        return false;
    }

    let mut length = 0;
    if !GetArrayLength(&mut cx, backing_list.handle(), &mut length) {
        return false;
    }

    if is_length_id(&cx, id) {
        length.to_jsval(cx.raw_cx(), vp);
        return true;
    }

    if let Some(index) = get_array_index_from_id(id) {
        if index >= length {
            vp.set(UndefinedValue());
            return true;
        }
        return JS_GetElement(&mut cx, backing_list.handle(), index, vp);
    }

    rooted!(&in(cx) let mut target = ptr::null_mut::<JSObject>());
    get_proxy_target(proxy, target.handle_mut());
    JS_ForwardGetPropertyTo(&mut cx, target.handle(), id, receiver, vp)
}

// https://searchfox.org/firefox-main/rev/c681e91369f59d0efae43bdc465872b855e8b269/dom/bindings/ObservableArrayProxyHandler.cpp#154
unsafe extern "C" fn get_own_property_descriptor(
    cx: *mut RawJSContext,
    proxy: RawHandleObject,
    id: RawHandleId,
    desc: RawMutableHandle<PropertyDescriptor>,
    is_none: *mut bool,
) -> bool {
    let mut cx = JSContext::from_ptr(NonNull::new(cx).unwrap());
    let proxy = HandleObject::from_raw(proxy);
    let id = Handle::from_raw(id);
    let desc = MutableHandle::from_raw(desc);
    let is_none = &mut *is_none;

    rooted!(&in(cx) let mut backing_list = ptr::null_mut::<JSObject>());
    if !get_backing_list_object(&mut cx, proxy, backing_list.handle_mut()) {
        return false;
    }

    let mut length = 0;
    if !GetArrayLength(&mut cx, backing_list.handle(), &mut length) {
        return false;
    }

    if is_length_id(&cx, id) {
        rooted!(&in(cx) let mut value = UndefinedValue());
        length.to_jsval(cx.raw_cx(), value.handle_mut());
        set_property_descriptor(desc, value.handle(), JSPROP_PERMANENT.into(), is_none);
        return true;
    }

    if let Some(index) = get_array_index_from_id(id) {
        if index >= length {
            return true;
        }

        rooted!(&in(cx) let mut value = UndefinedValue());
        if !JS_GetElement(&mut cx, backing_list.handle(), index, value.handle_mut()) {
            return false;
        }
        set_property_descriptor(desc, value.handle(), JSPROP_ENUMERATE.into(), is_none);
        return true;
    }

    rooted!(&in(cx) let mut target = ptr::null_mut::<JSObject>());
    get_proxy_target(proxy, target.handle_mut());
    JS_GetOwnPropertyDescriptorById(&mut cx, target.handle(), id, desc, is_none)
}

// https://searchfox.org/firefox-main/rev/c681e91369f59d0efae43bdc465872b855e8b269/dom/bindings/ObservableArrayProxyHandler.cpp#195
unsafe extern "C" fn has(
    cx: *mut RawJSContext,
    proxy: RawHandleObject,
    id: RawHandleId,
    bp: *mut bool,
) -> bool {
    let mut cx = JSContext::from_ptr(NonNull::new(cx).unwrap());
    let proxy = HandleObject::from_raw(proxy);
    let id = Handle::from_raw(id);

    if is_length_id(&cx, id) {
        *bp = true;
        return true;
    }

    if let Some(index) = get_array_index_from_id(id) {
        let mut length = 0;
        if !get_backing_list_length(&mut cx, proxy, &mut length) {
            return false;
        }
        *bp = index < length;
        return true;
    }

    rooted!(&in(cx) let mut target = ptr::null_mut::<JSObject>());
    get_proxy_target(proxy, target.handle_mut());
    JS_HasPropertyById(&mut cx, target.handle(), id, bp)
}

// https://searchfox.org/firefox-main/rev/c681e91369f59d0efae43bdc465872b855e8b269/dom/bindings/ObservableArrayProxyHandler.cpp#216
unsafe extern "C" fn own_property_keys(
    cx: *mut RawJSContext,
    proxy: RawHandleObject,
    props: RawMutableHandleIdVector,
) -> bool {
    let mut cx = JSContext::from_ptr(NonNull::new(cx).unwrap());
    let proxy = HandleObject::from_raw(proxy);

    let mut length = 0;
    if !get_backing_list_length(&mut cx, proxy, &mut length) {
        return false;
    }

    rooted!(&in(cx) let mut rooted_jsid: jsid);
    for i in 0..length {
        int_to_jsid(i as i32, rooted_jsid.handle_mut());
        AppendToIdVector(props, rooted_jsid.handle().into());
    }

    rooted!(&in(cx) let mut target = ptr::null_mut::<JSObject>());
    get_proxy_target(proxy, target.handle_mut());
    GetPropertyKeys(
        &mut cx,
        target.handle(),
        JSITER_OWNONLY | JSITER_HIDDEN | JSITER_SYMBOLS,
        props,
    )
}

// https://searchfox.org/firefox-main/rev/c681e91369f59d0efae43bdc465872b855e8b269/dom/bindings/ObservableArrayProxyHandler.cpp#232
unsafe extern "C" fn prevent_extensions(
    _cx: *mut RawJSContext,
    _proxy: RawHandleObject,
    result: *mut ObjectOpResult,
) -> bool {
    (*result).fail_cant_prevent_extensions()
}

// https://searchfox.org/firefox-main/rev/c681e91369f59d0efae43bdc465872b855e8b269/dom/bindings/ObservableArrayProxyHandler.cpp#238
unsafe extern "C" fn set(
    cx: *mut RawJSContext,
    proxy: RawHandleObject,
    id: RawHandleId,
    value: RawHandleValue,
    receiver: RawHandleValue,
    result: *mut ObjectOpResult,
) -> bool {
    let mut cx = JSContext::from_ptr(NonNull::new(cx).unwrap());
    let proxy = HandleObject::from_raw(proxy);
    let id = Handle::from_raw(id);
    let value = HandleValue::from_raw(value);
    let receiver = HandleValue::from_raw(receiver);

    if is_length_id(&cx, id) {
        rooted!(&in(cx) let mut backing_list = ptr::null_mut::<JSObject>());
        if !get_backing_list_object(&mut cx, proxy, backing_list.handle_mut()) {
            return false;
        }
        return set_length_from_value(&mut cx, proxy, backing_list.handle(), value, result);
    }

    if let Some(index) = get_array_index_from_id(id) {
        rooted!(&in(cx) let mut backing_list = ptr::null_mut::<JSObject>());
        if !get_backing_list_object(&mut cx, proxy, backing_list.handle_mut()) {
            return false;
        }
        return (config_for_proxy(proxy).set_indexed_value)(
            &mut cx,
            proxy,
            backing_list.handle(),
            index,
            value,
            result,
        );
    }

    rooted!(&in(cx) let mut target = ptr::null_mut::<JSObject>());
    get_proxy_target(proxy, target.handle_mut());
    JS_ForwardSetPropertyTo(&mut cx, target.handle(), id, value, receiver, result)
}
