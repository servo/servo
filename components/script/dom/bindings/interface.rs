/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Machinery to initialise interface prototype objects and interface objects.

use dom::bindings::codegen::InterfaceObjectMap::Globals;
use dom::bindings::codegen::PrototypeList;
use dom::bindings::conversions::get_dom_class;
use dom::bindings::guard::Guard;
use dom::bindings::utils::get_proto_or_iface_array;
use js::error::throw_type_error;
use js::glue::{RUST_SYMBOL_TO_JSID, UncheckedUnwrapObject};
use js::jsapi::{Class, ClassExtension, ClassSpec, GetGlobalForObjectCrossCompartment};
use js::jsapi::{GetWellKnownSymbol, HandleObject, HandleValue, JSClass, JSContext};
use js::jsapi::{JSFunctionSpec, JSNative, JSFUN_CONSTRUCTOR, JSPROP_ENUMERATE};
use js::jsapi::{JSPROP_PERMANENT, JSPROP_READONLY, JSPROP_RESOLVING, JSPropertySpec};
use js::jsapi::{JSString, JS_AtomizeAndPinString, JS_DefineProperty, JS_DefineProperty1};
use js::jsapi::{JS_DefineProperty2, JS_DefineProperty4, JS_DefinePropertyById3};
use js::jsapi::{JS_GetClass, JS_GetFunctionObject, JS_GetPrototype, JS_LinkConstructorAndPrototype};
use js::jsapi::{JS_NewFunction, JS_NewObject, JS_NewObjectWithUniqueType};
use js::jsapi::{JS_NewPlainObject, JS_NewStringCopyN, MutableHandleObject};
use js::jsapi::{MutableHandleValue, ObjectOps};
use js::jsapi::{SymbolCode, TrueHandleValue, Value};
use js::jsval::{BooleanValue, DoubleValue, Int32Value, JSVal, NullValue, UInt32Value};
use js::rust::{define_methods, define_properties};
use libc;
use std::ptr;

/// Representation of an IDL constant value.
#[derive(Clone)]
pub enum ConstantVal {
    /// `long` constant.
    IntVal(i32),
    /// `unsigned long` constant.
    UintVal(u32),
    /// `double` constant.
    DoubleVal(f64),
    /// `boolean` constant.
    BoolVal(bool),
    /// `null` constant.
    NullVal,
}

/// Representation of an IDL constant.
#[derive(Clone)]
pub struct ConstantSpec {
    /// name of the constant.
    pub name: &'static [u8],
    /// value of the constant.
    pub value: ConstantVal,
}

impl ConstantSpec {
    /// Returns a `JSVal` that represents the value of this `ConstantSpec`.
    pub fn get_value(&self) -> JSVal {
        match self.value {
            ConstantVal::NullVal => NullValue(),
            ConstantVal::IntVal(i) => Int32Value(i),
            ConstantVal::UintVal(u) => UInt32Value(u),
            ConstantVal::DoubleVal(d) => DoubleValue(d),
            ConstantVal::BoolVal(b) => BooleanValue(b),
        }
    }
}

/// A JSNative that cannot be null.
pub type NonNullJSNative =
    unsafe extern "C" fn (arg1: *mut JSContext, arg2: libc::c_uint, arg3: *mut JSVal) -> bool;

/// Defines constants on `obj`.
/// Fails on JSAPI failure.
fn define_constants(
        cx: *mut JSContext,
        obj: HandleObject,
        constants: &[ConstantSpec]) {
    for spec in constants {
        rooted!(in(cx) let value = spec.get_value());
        unsafe {
            assert!(JS_DefineProperty(cx,
                                      obj,
                                      spec.name.as_ptr() as *const libc::c_char,
                                      value.handle(),
                                      JSPROP_ENUMERATE | JSPROP_READONLY | JSPROP_PERMANENT,
                                      None,
                                      None));
        }
    }
}

unsafe extern "C" fn fun_to_string_hook(cx: *mut JSContext,
                                        obj: HandleObject,
                                        _indent: u32)
                                        -> *mut JSString {
    let js_class = JS_GetClass(obj.get());
    assert!(!js_class.is_null());
    let repr = (*(js_class as *const NonCallbackInterfaceObjectClass)).representation;
    assert!(!repr.is_empty());
    let ret = JS_NewStringCopyN(cx, repr.as_ptr() as *const libc::c_char, repr.len());
    assert!(!ret.is_null());
    ret
}

/// The class of a non-callback interface object.
#[derive(Copy, Clone)]
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
    pub const fn new(
            constructor_behavior: InterfaceConstructorBehavior,
            string_rep: &'static [u8],
            proto_id: PrototypeList::ID,
            proto_depth: u16)
            -> NonCallbackInterfaceObjectClass {
        NonCallbackInterfaceObjectClass {
            class: Class {
                name: b"Function\0" as *const _ as *const libc::c_char,
                flags: 0,
                addProperty: None,
                delProperty: None,
                getProperty: None,
                setProperty: None,
                enumerate: None,
                resolve: None,
                mayResolve: None,
                finalize: None,
                call: constructor_behavior.call,
                construct: constructor_behavior.construct,
                hasInstance: Some(has_instance_hook),
                trace: None,
                spec: ClassSpec {
                    createConstructor_: None,
                    createPrototype_: None,
                    constructorFunctions_: ptr::null(),
                    constructorProperties_: ptr::null(),
                    prototypeFunctions_: ptr::null(),
                    prototypeProperties_: ptr::null(),
                    finishInit_: None,
                    flags: 0,
                },
                ext: ClassExtension {
                    isWrappedNative: false,
                    weakmapKeyDelegateOp: None,
                    objectMovedOp: None,
                },
                ops: ObjectOps {
                    lookupProperty: None,
                    defineProperty: None,
                    hasProperty: None,
                    getProperty: None,
                    setProperty: None,
                    getOwnPropertyDescriptor: None,
                    deleteProperty: None,
                    watch: None,
                    unwatch: None,
                    getElements: None,
                    enumerate: None,
                    funToString: Some(fun_to_string_hook),
                }
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
pub struct InterfaceConstructorBehavior {
    call: JSNative,
    construct: JSNative,
}

impl InterfaceConstructorBehavior {
    /// An interface constructor that unconditionally throws a type error.
    pub const fn throw() -> InterfaceConstructorBehavior {
        InterfaceConstructorBehavior {
            call: Some(invalid_constructor),
            construct: Some(invalid_constructor),
        }
    }

    /// An interface constructor that calls a native Rust function.
    pub const fn call(hook: ConstructorClassHook) -> InterfaceConstructorBehavior {
        InterfaceConstructorBehavior {
            call: Some(non_new_constructor),
            construct: Some(hook),
        }
    }
}

/// Create and define the interface object of a callback interface.
pub unsafe fn create_callback_interface_object(
        cx: *mut JSContext,
        global: HandleObject,
        constants: &[Guard<&[ConstantSpec]>],
        name: &[u8],
        rval: MutableHandleObject) {
    assert!(!constants.is_empty());
    rval.set(JS_NewObject(cx, ptr::null()));
    assert!(!rval.ptr.is_null());
    for guard in constants {
        if let Some(specs) = guard.expose(cx, rval.handle()) {
            define_constants(cx, rval.handle(), specs);
        }
    }
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
        rooted!(in(cx) let mut unscopable_obj = ptr::null_mut());
        create_unscopable_object(cx, unscopable_names, unscopable_obj.handle_mut());

        let unscopable_symbol = GetWellKnownSymbol(cx, SymbolCode::unscopables);
        assert!(!unscopable_symbol.is_null());

        rooted!(in(cx) let unscopable_id = RUST_SYMBOL_TO_JSID(unscopable_symbol));
        assert!(JS_DefinePropertyById3(
            cx, rval.handle(), unscopable_id.handle(), unscopable_obj.handle(),
            JSPROP_READONLY, None, None))
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
        length: u32,
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
        named_constructors: &[(NonNullJSNative, &[u8], u32)],
        interface_prototype_object: HandleObject) {
    rooted!(in(cx) let mut constructor = ptr::null_mut());

    for &(native, name, arity) in named_constructors {
        assert!(*name.last().unwrap() == b'\0');

        let fun = JS_NewFunction(cx,
                                 Some(native),
                                 arity,
                                 JSFUN_CONSTRUCTOR,
                                 name.as_ptr() as *const libc::c_char);
        assert!(!fun.is_null());
        constructor.set(JS_GetFunctionObject(fun));
        assert!(!constructor.is_null());

        assert!(JS_DefineProperty1(cx,
                                   constructor.handle(),
                                   b"prototype\0".as_ptr() as *const libc::c_char,
                                   interface_prototype_object,
                                   JSPROP_PERMANENT | JSPROP_READONLY,
                                   None,
                                   None));

        define_on_global_object(cx, global, name, constructor.handle());
    }
}

/// Hook for instanceof on interface objects.
unsafe extern "C" fn has_instance_hook(cx: *mut JSContext,
        obj: HandleObject,
        value: MutableHandleValue,
        rval: *mut bool) -> bool {
    match has_instance(cx, obj, value.handle()) {
        Ok(result) => {
            *rval = result;
            true
        }
        Err(()) => false,
    }
}

/// Return whether a value is an instance of a given prototype.
/// http://heycam.github.io/webidl/#es-interface-hasinstance
unsafe fn has_instance(
        cx: *mut JSContext,
        interface_object: HandleObject,
        value: HandleValue)
        -> Result<bool, ()> {
    if !value.is_object() {
        // Step 1.
        return Ok(false);
    }
    rooted!(in(cx) let mut value = value.to_object());

    let js_class = JS_GetClass(interface_object.get());
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

    while JS_GetPrototype(cx, value.handle(), value.handle_mut()) {
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

unsafe fn create_object(
        cx: *mut JSContext,
        proto: HandleObject,
        class: &'static JSClass,
        methods: &[Guard<&'static [JSFunctionSpec]>],
        properties: &[Guard<&'static [JSPropertySpec]>],
        constants: &[Guard<&[ConstantSpec]>],
        rval: MutableHandleObject) {
    rval.set(JS_NewObjectWithUniqueType(cx, class, proto));
    assert!(!rval.ptr.is_null());
    define_guarded_methods(cx, rval.handle(), methods);
    define_guarded_properties(cx, rval.handle(), properties);
    for guard in constants {
        if let Some(specs) = guard.expose(cx, rval.handle()) {
            define_constants(cx, rval.handle(), specs);
        }
    }
}

unsafe fn create_unscopable_object(
        cx: *mut JSContext,
        names: &[&[u8]],
        rval: MutableHandleObject) {
    assert!(!names.is_empty());
    assert!(rval.is_null());
    rval.set(JS_NewPlainObject(cx));
    assert!(!rval.ptr.is_null());
    for &name in names {
        assert!(*name.last().unwrap() == b'\0');
        assert!(JS_DefineProperty(
            cx, rval.handle(), name.as_ptr() as *const libc::c_char, TrueHandleValue,
            JSPROP_READONLY, None, None));
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

unsafe fn define_name(cx: *mut JSContext, obj: HandleObject, name: &[u8]) {
    assert!(*name.last().unwrap() == b'\0');
    rooted!(in(cx) let name = JS_AtomizeAndPinString(cx, name.as_ptr() as *const libc::c_char));
    assert!(!name.is_null());
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

unsafe fn define_on_global_object(
        cx: *mut JSContext,
        global: HandleObject,
        name: &[u8],
        obj: HandleObject) {
    assert!(*name.last().unwrap() == b'\0');
    assert!(JS_DefineProperty1(cx,
                               global,
                               name.as_ptr() as *const libc::c_char,
                               obj,
                               JSPROP_RESOLVING,
                               None, None));
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

/// Returns whether an interface with exposure set given by `globals` should
/// be exposed in the global object `obj`.
pub unsafe fn is_exposed_in(object: HandleObject, globals: Globals) -> bool {
    let unwrapped = UncheckedUnwrapObject(object.get(), /* stopAtWindowProxy = */ 0);
    let dom_class = get_dom_class(unwrapped).unwrap();
    globals.contains(dom_class.global)
}
