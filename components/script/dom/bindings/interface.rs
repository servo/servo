/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Machinery to initialise interface prototype objects and interface objects.

use dom::bindings::codegen::InterfaceObjectMap::Globals;
use dom::bindings::codegen::PrototypeList;
use dom::bindings::constant::{define_constants, ConstantSpec};
use dom::bindings::conversions::{get_dom_class, DOM_OBJECT_SLOT};
use dom::bindings::guard::Guard;
use dom::bindings::utils::{get_proto_or_iface_array, ProtoOrIfaceArray, DOM_PROTOTYPE_SLOT};
use js::error::throw_type_error;
use js::glue::{UncheckedUnwrapObject, RUST_SYMBOL_TO_JSID};
use js::jsapi::{Class, ClassOps, CompartmentOptions};
use js::jsapi::{GetGlobalForObjectCrossCompartment, GetWellKnownSymbol};
use js::jsapi::{JSAutoCompartment, JSClass, JSContext, JSFunctionSpec, JSObject, JSFUN_CONSTRUCTOR};
use js::jsapi::{JSPROP_PERMANENT, JSPROP_READONLY, JSPROP_RESOLVING};
use js::jsapi::{JSPropertySpec, JSString, JSTracer, JS_AtomizeAndPinString};
use js::jsapi::{JS_GetFunctionObject, JS_NewFunction, JS_NewGlobalObject};
use js::jsapi::{JS_NewObject, JS_NewPlainObject};
use js::jsapi::{JS_NewStringCopyN, JS_SetReservedSlot};
use js::jsapi::{ObjectOps, OnNewGlobalHookOption, SymbolCode};
use js::jsapi::{TrueHandleValue, Value};
use js::jsapi::HandleObject as RawHandleObject;
use js::jsapi::MutableHandleValue as RawMutableHandleValue;
use js::jsval::{JSVal, PrivateValue};
use js::rust::{HandleObject, HandleValue, MutableHandleObject};
use js::rust::{define_methods, define_properties, get_object_class};
use js::rust::wrappers::{JS_DefineProperty, JS_DefineProperty2};
use js::rust::wrappers::{JS_DefineProperty3, JS_DefineProperty4, JS_DefinePropertyById4};
use js::rust::wrappers::{JS_FireOnNewGlobalObject, JS_GetPrototype};
use js::rust::wrappers::{JS_LinkConstructorAndPrototype, JS_NewObjectWithUniqueType};
use libc;
use std::ptr;

/// The class of a non-callback interface object.
#[derive(Clone, Copy)]
pub struct NonCallbackInterfaceObjectClass {
    /// The SpiderMonkey Class structure.
    pub class: Class,
    /// The prototype id of that interface, used in the hasInstance hook.
    pub proto_id: PrototypeList::ID,
    /// The prototype depth of that interface, used in the hasInstance hook.
    pub proto_depth: u16,
    /// The string representation of the object.
    pub representation: &'static [u8],
}

unsafe impl Sync for NonCallbackInterfaceObjectClass {}

impl NonCallbackInterfaceObjectClass {
    /// Create a new `NonCallbackInterfaceObjectClass` structure.
    pub const fn new(constructor_behavior: &'static InterfaceConstructorBehavior,
                     string_rep: &'static [u8],
                     proto_id: PrototypeList::ID,
                     proto_depth: u16)
                     -> NonCallbackInterfaceObjectClass {
        NonCallbackInterfaceObjectClass {
            class: Class {
                name: b"Function\0" as *const _ as *const libc::c_char,
                flags: 0,
                cOps: &constructor_behavior.0,
                spec: 0 as *const _,
                ext: 0 as *const _,
                oOps: &OBJECT_OPS,
            },
            proto_id: proto_id,
            proto_depth: proto_depth,
            representation: string_rep,
        }
    }

    /// cast own reference to `JSClass` reference
    pub fn as_jsclass(&self) -> &JSClass {
        unsafe {
            &*(self as *const _ as *const JSClass)
        }
    }
}

/// A constructor class hook.
pub type ConstructorClassHook =
    unsafe extern "C" fn(cx: *mut JSContext, argc: u32, vp: *mut Value) -> bool;

/// The constructor behavior of a non-callback interface object.
pub struct InterfaceConstructorBehavior(ClassOps);

impl InterfaceConstructorBehavior {
    /// An interface constructor that unconditionally throws a type error.
    pub const fn throw() -> Self {
        InterfaceConstructorBehavior(ClassOps {
            addProperty: None,
            delProperty: None,
            enumerate: None,
            newEnumerate: None,
            resolve: None,
            mayResolve: None,
            finalize: None,
            call: Some(invalid_constructor),
            construct: Some(invalid_constructor),
            hasInstance: Some(has_instance_hook),
            trace: None,
        })
    }

    /// An interface constructor that calls a native Rust function.
    pub const fn call(hook: ConstructorClassHook) -> Self {
        InterfaceConstructorBehavior(ClassOps {
            addProperty: None,
            delProperty: None,
            enumerate: None,
            newEnumerate: None,
            resolve: None,
            mayResolve: None,
            finalize: None,
            call: Some(non_new_constructor),
            construct: Some(hook),
            hasInstance: Some(has_instance_hook),
            trace: None,
        })
    }
}

/// A trace hook.
pub type TraceHook =
    unsafe extern "C" fn(trc: *mut JSTracer, obj: *mut JSObject);

/// Create a global object with the given class.
pub unsafe fn create_global_object(
        cx: *mut JSContext,
        class: &'static JSClass,
        private: *const libc::c_void,
        trace: TraceHook,
        mut rval: MutableHandleObject) {
    assert!(rval.is_null());

    let mut options = CompartmentOptions::default();
    options.creationOptions_.traceGlobal_ = Some(trace);
    options.creationOptions_.sharedMemoryAndAtomics_ = true;

    rval.set(JS_NewGlobalObject(cx,
                                class,
                                ptr::null_mut(),
                                OnNewGlobalHookOption::DontFireOnNewGlobalHook,
                                &options));
    assert!(!rval.is_null());

    // Initialize the reserved slots before doing anything that can GC, to
    // avoid getting trace hooks called on a partially initialized object.
    let private_val = PrivateValue(private);
    JS_SetReservedSlot(rval.get(), DOM_OBJECT_SLOT, &private_val);
    let proto_array: Box<ProtoOrIfaceArray> =
        Box::new([0 as *mut JSObject; PrototypeList::PROTO_OR_IFACE_LENGTH]);
    let val = PrivateValue(Box::into_raw(proto_array) as *const libc::c_void);
    JS_SetReservedSlot(rval.get(), DOM_PROTOTYPE_SLOT, &val);

    let _ac = JSAutoCompartment::new(cx, rval.get());
    JS_FireOnNewGlobalObject(cx, rval.handle());
}

/// Create and define the interface object of a callback interface.
pub unsafe fn create_callback_interface_object(
        cx: *mut JSContext,
        global: HandleObject,
        constants: &[Guard<&[ConstantSpec]>],
        name: &[u8],
        mut rval: MutableHandleObject) {
    assert!(!constants.is_empty());
    rval.set(JS_NewObject(cx, ptr::null()));
    assert!(!rval.is_null());
    define_guarded_constants(cx, rval.handle(), constants);
    define_name(cx, rval.handle(), name);
    define_on_global_object(cx, global, name, rval.handle());
}

/// Create the interface prototype object of a non-callback interface.
pub unsafe fn create_interface_prototype_object(
        cx: *mut JSContext,
        proto: HandleObject,
        class: &'static JSClass,
        regular_methods: &[Guard<&'static [JSFunctionSpec]>],
        regular_properties: &[Guard<&'static [JSPropertySpec]>],
        constants: &[Guard<&[ConstantSpec]>],
        unscopable_names: &[&[u8]],
        rval: MutableHandleObject) {
    create_object(cx, proto, class, regular_methods, regular_properties, constants, rval);

    if !unscopable_names.is_empty() {
        rooted!(in(cx) let mut unscopable_obj = ptr::null_mut::<JSObject>());
        create_unscopable_object(cx, unscopable_names, unscopable_obj.handle_mut());

        let unscopable_symbol = GetWellKnownSymbol(cx, SymbolCode::unscopables);
        assert!(!unscopable_symbol.is_null());

        rooted!(in(cx) let unscopable_id = RUST_SYMBOL_TO_JSID(unscopable_symbol));
        assert!(JS_DefinePropertyById4(
            cx, rval.handle(), unscopable_id.handle(), unscopable_obj.handle(),
            JSPROP_READONLY as u32))
    }
}

/// Create and define the interface object of a non-callback interface.
pub unsafe fn create_noncallback_interface_object(
        cx: *mut JSContext,
        global: HandleObject,
        proto: HandleObject,
        class: &'static NonCallbackInterfaceObjectClass,
        static_methods: &[Guard<&'static [JSFunctionSpec]>],
        static_properties: &[Guard<&'static [JSPropertySpec]>],
        constants: &[Guard<&[ConstantSpec]>],
        interface_prototype_object: HandleObject,
        name: &[u8],
        length: i32,
        rval: MutableHandleObject) {
    create_object(cx,
                  proto,
                  class.as_jsclass(),
                  static_methods,
                  static_properties,
                  constants,
                  rval);
    assert!(JS_LinkConstructorAndPrototype(cx, rval.handle(), interface_prototype_object));
    define_name(cx, rval.handle(), name);
    define_length(cx, rval.handle(), length);
    define_on_global_object(cx, global, name, rval.handle());
}

/// Create and define the named constructors of a non-callback interface.
pub unsafe fn create_named_constructors(
        cx: *mut JSContext,
        global: HandleObject,
        named_constructors: &[(ConstructorClassHook, &[u8], u32)],
        interface_prototype_object: HandleObject) {
    rooted!(in(cx) let mut constructor = ptr::null_mut::<JSObject>());

    for &(native, name, arity) in named_constructors {
        assert_eq!(*name.last().unwrap(), b'\0');

        let fun = JS_NewFunction(cx,
                                 Some(native),
                                 arity,
                                 JSFUN_CONSTRUCTOR,
                                 name.as_ptr() as *const libc::c_char);
        assert!(!fun.is_null());
        constructor.set(JS_GetFunctionObject(fun));
        assert!(!constructor.is_null());

        assert!(JS_DefineProperty2(cx,
                                   constructor.handle(),
                                   b"prototype\0".as_ptr() as *const libc::c_char,
                                   interface_prototype_object,
                                   (JSPROP_PERMANENT | JSPROP_READONLY) as u32));

        define_on_global_object(cx, global, name, constructor.handle());
    }
}

/// Create a new object with a unique type.
pub unsafe fn create_object(
        cx: *mut JSContext,
        proto: HandleObject,
        class: &'static JSClass,
        methods: &[Guard<&'static [JSFunctionSpec]>],
        properties: &[Guard<&'static [JSPropertySpec]>],
        constants: &[Guard<&[ConstantSpec]>],
        mut rval: MutableHandleObject) {
    rval.set(JS_NewObjectWithUniqueType(cx, class, proto));
    assert!(!rval.is_null());
    define_guarded_methods(cx, rval.handle(), methods);
    define_guarded_properties(cx, rval.handle(), properties);
    define_guarded_constants(cx, rval.handle(), constants);
}

/// Conditionally define constants on an object.
pub unsafe fn define_guarded_constants(
        cx: *mut JSContext,
        obj: HandleObject,
        constants: &[Guard<&[ConstantSpec]>]) {
    for guard in constants {
        if let Some(specs) = guard.expose(cx, obj) {
            define_constants(cx, obj, specs);
        }
    }
}

/// Conditionally define methods on an object.
pub unsafe fn define_guarded_methods(
        cx: *mut JSContext,
        obj: HandleObject,
        methods: &[Guard<&'static [JSFunctionSpec]>]) {
    for guard in methods {
        if let Some(specs) = guard.expose(cx, obj) {
            define_methods(cx, obj, specs).unwrap();
        }
    }
}

/// Conditionally define properties on an object.
pub unsafe fn define_guarded_properties(
        cx: *mut JSContext,
        obj: HandleObject,
        properties: &[Guard<&'static [JSPropertySpec]>]) {
    for guard in properties {
        if let Some(specs) = guard.expose(cx, obj) {
            define_properties(cx, obj, specs).unwrap();
        }
    }
}

/// Returns whether an interface with exposure set given by `globals` should
/// be exposed in the global object `obj`.
pub unsafe fn is_exposed_in(object: HandleObject, globals: Globals) -> bool {
    let unwrapped = UncheckedUnwrapObject(object.get(), /* stopAtWindowProxy = */ 0);
    let dom_class = get_dom_class(unwrapped).unwrap();
    globals.contains(dom_class.global)
}

/// Define a property with a given name on the global object. Should be called
/// through the resolve hook.
pub unsafe fn define_on_global_object(
        cx: *mut JSContext,
        global: HandleObject,
        name: &[u8],
        obj: HandleObject) {
    assert_eq!(*name.last().unwrap(), b'\0');
    assert!(JS_DefineProperty2(cx,
                               global,
                               name.as_ptr() as *const libc::c_char,
                               obj,
                               JSPROP_RESOLVING));
}

const OBJECT_OPS: ObjectOps = ObjectOps {
    lookupProperty: None,
    defineProperty: None,
    hasProperty: None,
    getProperty: None,
    setProperty: None,
    getOwnPropertyDescriptor: None,
    deleteProperty: None,
    getElements: None,
    funToString: Some(fun_to_string_hook),
};

unsafe extern "C" fn fun_to_string_hook(cx: *mut JSContext,
                                        obj: RawHandleObject,
                                        _is_to_source: bool)
                                        -> *mut JSString {
    let js_class = get_object_class(obj.get());
    assert!(!js_class.is_null());
    let repr = (*(js_class as *const NonCallbackInterfaceObjectClass)).representation;
    assert!(!repr.is_empty());
    let ret = JS_NewStringCopyN(cx, repr.as_ptr() as *const libc::c_char, repr.len());
    assert!(!ret.is_null());
    ret
}

/// Hook for instanceof on interface objects.
unsafe extern "C" fn has_instance_hook(cx: *mut JSContext,
        obj: RawHandleObject,
        value: RawMutableHandleValue,
        rval: *mut bool) -> bool {
    let obj_raw = HandleObject::from_raw(obj);
    let val_raw = HandleValue::from_raw(value.handle());
    match has_instance(cx, obj_raw, val_raw) {
        Ok(result) => {
            *rval = result;
            true
        }
        Err(()) => false,
    }
}

/// Return whether a value is an instance of a given prototype.
/// <http://heycam.github.io/webidl/#es-interface-hasinstance>
unsafe fn has_instance(
        cx: *mut JSContext,
        interface_object: HandleObject,
        value: HandleValue)
        -> Result<bool, ()> {
    if !value.is_object() {
        // Step 1.
        return Ok(false);
    }

    rooted!(in(cx) let mut value_out = value.to_object());
    rooted!(in(cx) let mut value = value.to_object());

    let js_class = get_object_class(interface_object.get());
    let object_class = &*(js_class as *const NonCallbackInterfaceObjectClass);

    if let Ok(dom_class) = get_dom_class(UncheckedUnwrapObject(value.get(),
                                                               /* stopAtWindowProxy = */ 0)) {
        if dom_class.interface_chain[object_class.proto_depth as usize] == object_class.proto_id {
            // Step 4.
            return Ok(true);
        }
    }

    // Step 2.
    let global = GetGlobalForObjectCrossCompartment(interface_object.get());
    assert!(!global.is_null());
    let proto_or_iface_array = get_proto_or_iface_array(global);
    rooted!(in(cx) let prototype = (*proto_or_iface_array)[object_class.proto_id as usize]);
    assert!(!prototype.is_null());
    // Step 3 only concern legacy callback interface objects (i.e. NodeFilter).

    while JS_GetPrototype(cx, value.handle(), value_out.handle_mut()) {
        *value = *value_out;
        if value.is_null() {
            // Step 5.2.
            return Ok(false);
        } else if value.get() as *const _ == prototype.get() {
            // Step 5.3.
            return Ok(true);
        }
    }
    // JS_GetPrototype threw an exception.
    Err(())
}

unsafe fn create_unscopable_object(
        cx: *mut JSContext,
        names: &[&[u8]],
        mut rval: MutableHandleObject) {
    assert!(!names.is_empty());
    assert!(rval.is_null());
    rval.set(JS_NewPlainObject(cx));
    assert!(!rval.is_null());
    for &name in names {
        assert_eq!(*name.last().unwrap(), b'\0');
        assert!(JS_DefineProperty(
            cx,
            rval.handle(),
            name.as_ptr() as *const libc::c_char,
            HandleValue::from_raw(TrueHandleValue),
            JSPROP_READONLY as u32,
        ));
    }
}

unsafe fn define_name(cx: *mut JSContext, obj: HandleObject, name: &[u8]) {
    assert_eq!(*name.last().unwrap(), b'\0');
    rooted!(in(cx) let name = JS_AtomizeAndPinString(cx, name.as_ptr() as *const libc::c_char));
    assert!(!name.is_null());
    assert!(JS_DefineProperty3(cx,
                               obj,
                               b"name\0".as_ptr() as *const libc::c_char,
                               name.handle().into(),
                               JSPROP_READONLY as u32));
}

unsafe fn define_length(cx: *mut JSContext, obj: HandleObject, length: i32) {
    assert!(JS_DefineProperty4(cx,
                               obj,
                               b"length\0".as_ptr() as *const libc::c_char,
                               length,
                               JSPROP_READONLY as u32));
}

unsafe extern "C" fn invalid_constructor(
        cx: *mut JSContext,
        _argc: libc::c_uint,
        _vp: *mut JSVal)
        -> bool {
    throw_type_error(cx, "Illegal constructor.");
    false
}

unsafe extern "C" fn non_new_constructor(
        cx: *mut JSContext,
        _argc: libc::c_uint,
        _vp: *mut JSVal)
        -> bool {
    throw_type_error(cx, "This constructor needs to be called with `new`.");
    false
}
