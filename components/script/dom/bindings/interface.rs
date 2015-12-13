/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Machinery to initialise interface prototype objects and interface objects.

use dom::bindings::utils::{ConstantSpec, NonNullJSNative, define_constants};
use js::jsapi::{HandleObject, JSClass, JSContext, JSFunctionSpec, JSObject};
use js::jsapi::{JSPropertySpec, JS_DefineProperty1, JS_GetFunctionObject};
use js::jsapi::{JS_LinkConstructorAndPrototype, JS_NewFunction};
use js::jsapi::{JS_NewObjectWithUniqueType, MutableHandleObject, RootedObject};
use js::rust::{define_methods, define_properties};
use js::{JSFUN_CONSTRUCTOR, JSPROP_PERMANENT, JSPROP_READONLY};
use libc;
use std::ptr;

/// Create and define the interface object of a callback interface.
pub unsafe fn create_callback_interface_object(
        cx: *mut JSContext,
        receiver: HandleObject,
        constructor_native: NonNullJSNative,
        length: u32,
        constants: &'static [ConstantSpec],
        name: &'static [u8]) {
    assert!(!constants.is_empty());
    let interface_object =
        RootedObject::new(cx, create_constructor(cx, constructor_native, length, name));
    assert!(!interface_object.ptr.is_null());
    define_constants(cx, interface_object.handle(), constants);
    define_on_global_object(cx, receiver, name, interface_object.handle());
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
        constructor_native: NonNullJSNative,
        static_methods: Option<&'static [JSFunctionSpec]>,
        static_properties: Option<&'static [JSPropertySpec]>,
        constants: &'static [ConstantSpec],
        interface_prototype_object: HandleObject,
        name: &'static [u8],
        length: u32) {
    assert!(!interface_prototype_object.ptr.is_null());

    let interface_object =
        RootedObject::new(cx, create_constructor(cx, constructor_native, length, name));
    assert!(!interface_object.ptr.is_null());

    if let Some(static_methods) = static_methods {
        define_methods(cx, interface_object.handle(), static_methods).unwrap();
    }

    if let Some(static_properties) = static_properties {
        define_properties(cx, interface_object.handle(), static_properties).unwrap();
    }

    define_constants(cx, interface_object.handle(), constants);

    assert!(JS_LinkConstructorAndPrototype(cx, interface_object.handle(),
                                           interface_prototype_object));
    define_on_global_object(cx, receiver, name, interface_object.handle());
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

        constructor.ptr = create_constructor(cx, native, arity, name);
        assert!(!constructor.ptr.is_null());

        assert!(JS_DefineProperty1(cx,
                                   constructor.handle(),
                                   b"prototype\0".as_ptr() as *const libc::c_char,
                                   interface_prototype_object,
                                   JSPROP_PERMANENT | JSPROP_READONLY,
                                   None,
                                   None));

        define_on_global_object(cx, receiver, name, constructor.handle());
    }
}

unsafe fn create_constructor(
        cx: *mut JSContext,
        constructor_native: NonNullJSNative,
        ctor_nargs: u32,
        name: &'static [u8])
        -> *mut JSObject {
    assert!(*name.last().unwrap() == b'\0');

    let fun = JS_NewFunction(cx,
                             Some(constructor_native),
                             ctor_nargs,
                             JSFUN_CONSTRUCTOR,
                             name.as_ptr() as *const _);
    assert!(!fun.is_null());

    let constructor = JS_GetFunctionObject(fun);
    assert!(!constructor.is_null());

    constructor
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

unsafe fn define_on_global_object(
        cx: *mut JSContext,
        receiver: HandleObject,
        name: &'static [u8],
        obj: HandleObject) {
    assert!(*name.last().unwrap() == b'\0');
    assert!(JS_DefineProperty1(cx,
                               receiver,
                               name.as_ptr() as *const libc::c_char,
                               obj,
                               0,
                               None, None));
}
