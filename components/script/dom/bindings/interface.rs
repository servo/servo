/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Machinery to initialise interface prototype objects and interface objects.

use dom::bindings::codegen::PrototypeList;
use dom::bindings::conversions::get_dom_class;
use dom::bindings::utils::{ConstantSpec, NonNullJSNative, define_constants};
use js::jsapi::{HandleObject, HandleValue, JSClass, JSContext, JSFunctionSpec};
use js::jsapi::{JSPropertySpec, JS_DefineProperty1, JS_DefineProperty2};
use js::jsapi::{JS_DefineProperty4, JS_GetFunctionObject, JS_GetPrototype};
use js::jsapi::{JS_InternString, JS_LinkConstructorAndPrototype, JS_NewFunction};
use js::jsapi::{JS_NewObject, JS_NewObjectWithUniqueType, MutableHandleObject};
use js::jsapi::{RootedObject, RootedString};
use js::rust::{define_methods, define_properties};
use js::{JSFUN_CONSTRUCTOR, JSPROP_PERMANENT, JSPROP_READONLY};
use libc;
use std::ptr;

/// Create and define the interface object of a callback interface.
pub unsafe fn create_callback_interface_object(cx: *mut JSContext,
                                               receiver: HandleObject,
                                               constants: &'static [ConstantSpec],
                                               name: &'static [u8]) {
    assert!(!constants.is_empty());
    let obj = RootedObject::new(cx, JS_NewObject(cx, ptr::null()));
    assert!(!obj.ptr.is_null());
    define_constants(cx, obj.handle(), constants);
    define_name(cx, obj.handle(), name);
    define_global_object(cx, receiver, name, obj.handle());
}

/// Create the interface prototype object of a non-callback interface.
pub unsafe fn create_interface_prototype_object(
        cx: *mut JSContext,
        proto: HandleObject,
        class: &'static JSClass,
        regular_methods: Option<&'static [JSFunctionSpec]>,
        regular_properties: Option<&'static [JSPropertySpec]>,
        constants: &'static [ConstantSpec],
        rval: MutableHandleObject) {
    create_object(cx, proto, class, regular_methods, regular_properties, constants, rval);
}

/// Create and define the interface object of a non-callback interface.
pub unsafe fn create_noncallback_interface_object(
        cx: *mut JSContext,
        receiver: HandleObject,
        proto: HandleObject,
        class: &'static JSClass,
        static_methods: Option<&'static [JSFunctionSpec]>,
        static_properties: Option<&'static [JSPropertySpec]>,
        constants: &'static [ConstantSpec],
        interface_prototype_object: HandleObject,
        name: &'static [u8],
        length: u32,
        rval: MutableHandleObject) {
    create_object(cx, proto, class, static_methods, static_properties, constants, rval);
    assert!(JS_LinkConstructorAndPrototype(cx, rval.handle(), interface_prototype_object));
    define_name(cx, rval.handle(), name);
    define_length(cx, rval.handle(), length);
    define_global_object(cx, receiver, name, rval.handle());
}

/// Create and define the named constructors of a non-callback interface.
pub unsafe fn create_named_constructors(
        cx: *mut JSContext,
        receiver: HandleObject,
        named_constructors: &[(NonNullJSNative, &'static [u8], u32)],
        interface_prototype_object: HandleObject) {
    let mut constructor = RootedObject::new(cx, ptr::null_mut());

    for &(native, name, arity) in named_constructors {
        assert!(*name.last().unwrap() == b'\0');

        let fun = JS_NewFunction(cx,
                                 Some(native),
                                 arity,
                                 JSFUN_CONSTRUCTOR,
                                 name.as_ptr() as *const libc::c_char);
        assert!(!fun.is_null());
        constructor.ptr = JS_GetFunctionObject(fun);
        assert!(!constructor.ptr.is_null());

        assert!(JS_DefineProperty1(cx,
                                   constructor.handle(),
                                   b"prototype\0".as_ptr() as *const libc::c_char,
                                   interface_prototype_object,
                                   JSPROP_PERMANENT | JSPROP_READONLY,
                                   None,
                                   None));

        define_global_object(cx, receiver, name, constructor.handle());
    }
}

/// Return whether a value is an instance of a given prototype.
/// http://heycam.github.io/webidl/#es-interface-hasinstance
pub unsafe fn has_instance(
        cx: *mut JSContext,
        prototype: HandleObject,
        value: HandleValue,
        id: PrototypeList::ID,
        index: usize)
        -> bool {
    if !value.is_object() {
        // Step 1.
        return false;
    }
    let mut value = RootedObject::new(cx, value.to_object());

    // Steps 2-3 only concern callback interface objects.

    if let Ok(dom_class) = get_dom_class(value.ptr) {
        if dom_class.interface_chain[index] == id {
            // Step 4.
            return true;
        }
    }

    while JS_GetPrototype(cx, value.handle(), value.handle_mut()) {
        if value.ptr as *const _ == prototype.ptr {
            // Step 5.3.
            return true;
        }
    }
    // Step 5.2.
    false
}

unsafe fn create_object(
        cx: *mut JSContext,
        proto: HandleObject,
        class: &'static JSClass,
        methods: Option<&'static [JSFunctionSpec]>,
        properties: Option<&'static [JSPropertySpec]>,
        constants: &'static [ConstantSpec],
        rval: MutableHandleObject) {
    rval.set(JS_NewObjectWithUniqueType(cx, class, proto));
    assert!(!rval.ptr.is_null());
    if let Some(methods) = methods {
        define_methods(cx, rval.handle(), methods).unwrap();
    }
    if let Some(properties) = properties {
        define_properties(cx, rval.handle(), properties).unwrap();
    }
    define_constants(cx, rval.handle(), constants);
}

unsafe fn define_name(cx: *mut JSContext, obj: HandleObject, name: &'static [u8]) {
    assert!(*name.last().unwrap() == b'\0');
    let name =
        RootedString::new(cx, JS_InternString(cx, name.as_ptr() as *const libc::c_char));
    assert!(!name.ptr.is_null());
    assert!(JS_DefineProperty2(cx,
                               obj,
                               b"name\0".as_ptr() as *const libc::c_char,
                               name.handle(),
                               JSPROP_READONLY,
                               None, None));
}

unsafe fn define_length(cx: *mut JSContext, obj: HandleObject, length: u32) {
    assert!(JS_DefineProperty4(cx,
                               obj,
                               b"length\0".as_ptr() as *const libc::c_char,
                               length,
                               JSPROP_READONLY,
                               None, None));
}

unsafe fn define_global_object(
        cx: *mut JSContext,
        receiver: HandleObject,
        name: &'static [u8],
        obj: HandleObject) {
    assert!(JS_DefineProperty1(cx,
                               receiver,
                               name.as_ptr() as *const libc::c_char,
                               obj,
                               0,
                               None, None));
}
