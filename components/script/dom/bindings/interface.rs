/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Machinery to initialise interface prototype objects and interface objects.

use std::convert::TryFrom;
use std::ffi::CStr;
use std::ptr;

use js::error::throw_type_error;
use js::glue::UncheckedUnwrapObject;
use js::jsapi::JS::CompartmentIterResult;
use js::jsapi::{
    jsid, CallArgs, CheckedUnwrapStatic, Compartment, CompartmentSpecifier, CurrentGlobalOrNull,
    GetFunctionRealm, GetNonCCWObjectGlobal, GetRealmGlobalOrNull, GetWellKnownSymbol,
    HandleObject as RawHandleObject, IsSharableCompartment, IsSystemCompartment, JSAutoRealm,
    JSClass, JSClassOps, JSContext, JSFunctionSpec, JSObject, JSPropertySpec, JSString, JSTracer,
    JS_AtomizeAndPinString, JS_GetFunctionObject, JS_GetProperty, JS_IterateCompartments,
    JS_NewFunction, JS_NewGlobalObject, JS_NewObject, JS_NewPlainObject, JS_NewStringCopyN,
    JS_SetReservedSlot, JS_WrapObject, ObjectOps, OnNewGlobalHookOption, SymbolCode,
    TrueHandleValue, Value, JSFUN_CONSTRUCTOR, JSPROP_PERMANENT, JSPROP_READONLY, JSPROP_RESOLVING,
};
use js::jsval::{JSVal, NullValue, PrivateValue};
use js::rust::wrappers::{
    JS_DefineProperty, JS_DefineProperty3, JS_DefineProperty4, JS_DefineProperty5,
    JS_DefinePropertyById5, JS_FireOnNewGlobalObject, JS_LinkConstructorAndPrototype,
    JS_NewObjectWithGivenProto, RUST_SYMBOL_TO_JSID,
};
use js::rust::{
    define_methods, define_properties, get_object_class, is_dom_class, maybe_wrap_object,
    HandleObject, HandleValue, MutableHandleObject, RealmOptions,
};
use servo_url::MutableOrigin;

use crate::dom::bindings::codegen::InterfaceObjectMap::Globals;
use crate::dom::bindings::codegen::PrototypeList;
use crate::dom::bindings::constant::{define_constants, ConstantSpec};
use crate::dom::bindings::conversions::{get_dom_class, DOM_OBJECT_SLOT};
use crate::dom::bindings::guard::Guard;
use crate::dom::bindings::principals::ServoJSPrincipals;
use crate::dom::bindings::utils::{
    callargs_is_constructing, get_proto_or_iface_array, DOMJSClass, ProtoOrIfaceArray,
    DOM_PROTOTYPE_SLOT, JSCLASS_DOM_GLOBAL,
};
use crate::script_runtime::JSContext as SafeJSContext;

/// The class of a non-callback interface object.
#[derive(Clone, Copy)]
pub struct NonCallbackInterfaceObjectClass {
    /// The SpiderMonkey class structure.
    pub class: JSClass,
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
        constructor_behavior: &'static InterfaceConstructorBehavior,
        string_rep: &'static [u8],
        proto_id: PrototypeList::ID,
        proto_depth: u16,
    ) -> NonCallbackInterfaceObjectClass {
        NonCallbackInterfaceObjectClass {
            class: JSClass {
                name: c"Function".as_ptr(),
                flags: 0,
                cOps: &constructor_behavior.0,
                spec: 0 as *const _,
                ext: 0 as *const _,
                oOps: &OBJECT_OPS,
            },
            proto_id,
            proto_depth,
            representation: string_rep,
        }
    }

    /// cast own reference to `JSClass` reference
    pub fn as_jsclass(&self) -> &JSClass {
        unsafe { &*(self as *const _ as *const JSClass) }
    }
}

/// A constructor class hook.
pub type ConstructorClassHook =
    unsafe extern "C" fn(cx: *mut JSContext, argc: u32, vp: *mut Value) -> bool;

/// The constructor behavior of a non-callback interface object.
pub struct InterfaceConstructorBehavior(JSClassOps);

impl InterfaceConstructorBehavior {
    /// An interface constructor that unconditionally throws a type error.
    pub const fn throw() -> Self {
        InterfaceConstructorBehavior(JSClassOps {
            addProperty: None,
            delProperty: None,
            enumerate: None,
            newEnumerate: None,
            resolve: None,
            mayResolve: None,
            finalize: None,
            call: Some(invalid_constructor),
            construct: Some(invalid_constructor),
            trace: None,
        })
    }

    /// An interface constructor that calls a native Rust function.
    pub const fn call(hook: ConstructorClassHook) -> Self {
        InterfaceConstructorBehavior(JSClassOps {
            addProperty: None,
            delProperty: None,
            enumerate: None,
            newEnumerate: None,
            resolve: None,
            mayResolve: None,
            finalize: None,
            call: Some(non_new_constructor),
            construct: Some(hook),
            trace: None,
        })
    }
}

/// A trace hook.
pub type TraceHook = unsafe extern "C" fn(trc: *mut JSTracer, obj: *mut JSObject);

/// Create a global object with the given class.
pub unsafe fn create_global_object(
    cx: SafeJSContext,
    class: &'static JSClass,
    private: *const libc::c_void,
    trace: TraceHook,
    mut rval: MutableHandleObject,
    origin: &MutableOrigin,
) {
    assert!(rval.is_null());

    let mut options = RealmOptions::default();
    options.creationOptions_.traceGlobal_ = Some(trace);
    options.creationOptions_.sharedMemoryAndAtomics_ = false;
    options.creationOptions_.streams_ = true;
    select_compartment(cx, &mut options);

    let principal = ServoJSPrincipals::new(origin);

    rval.set(JS_NewGlobalObject(
        *cx,
        class,
        principal.as_raw(),
        OnNewGlobalHookOption::DontFireOnNewGlobalHook,
        &*options,
    ));
    assert!(!rval.is_null());

    // Initialize the reserved slots before doing anything that can GC, to
    // avoid getting trace hooks called on a partially initialized object.
    let private_val = PrivateValue(private);
    JS_SetReservedSlot(rval.get(), DOM_OBJECT_SLOT, &private_val);
    let proto_array: Box<ProtoOrIfaceArray> =
        Box::new([ptr::null_mut::<JSObject>(); PrototypeList::PROTO_OR_IFACE_LENGTH]);
    let val = PrivateValue(Box::into_raw(proto_array) as *const libc::c_void);
    JS_SetReservedSlot(rval.get(), DOM_PROTOTYPE_SLOT, &val);

    let _ac = JSAutoRealm::new(*cx, rval.get());
    JS_FireOnNewGlobalObject(*cx, rval.handle());
}

/// Choose the compartment to create a new global object in.
fn select_compartment(cx: SafeJSContext, options: &mut RealmOptions) {
    type Data = *mut Compartment;
    unsafe extern "C" fn callback(
        _cx: *mut JSContext,
        data: *mut libc::c_void,
        compartment: *mut Compartment,
    ) -> CompartmentIterResult {
        let data = data as *mut Data;

        if !IsSharableCompartment(compartment) || IsSystemCompartment(compartment) {
            return CompartmentIterResult::KeepGoing;
        }

        // Choose any sharable, non-system compartment in this context to allow
        // same-agent documents to share JS and DOM objects.
        *data = compartment;
        CompartmentIterResult::Stop
    }

    let mut compartment: Data = ptr::null_mut();
    unsafe {
        JS_IterateCompartments(
            *cx,
            (&mut compartment) as *mut Data as *mut libc::c_void,
            Some(callback),
        );
    }

    if compartment.is_null() {
        options.creationOptions_.compSpec_ = CompartmentSpecifier::NewCompartmentAndZone;
    } else {
        options.creationOptions_.compSpec_ = CompartmentSpecifier::ExistingCompartment;
        options.creationOptions_.__bindgen_anon_1.comp_ = compartment;
    }
}

/// Create and define the interface object of a callback interface.
pub fn create_callback_interface_object(
    cx: SafeJSContext,
    global: HandleObject,
    constants: &[Guard<&[ConstantSpec]>],
    name: &CStr,
    mut rval: MutableHandleObject,
) {
    assert!(!constants.is_empty());
    unsafe {
        rval.set(JS_NewObject(*cx, ptr::null()));
    }
    assert!(!rval.is_null());
    define_guarded_constants(cx, rval.handle(), constants, global);
    define_name(cx, rval.handle(), name);
    define_on_global_object(cx, global, name, rval.handle());
}

/// Create the interface prototype object of a non-callback interface.
#[allow(clippy::too_many_arguments)]
pub fn create_interface_prototype_object(
    cx: SafeJSContext,
    global: HandleObject,
    proto: HandleObject,
    class: &'static JSClass,
    regular_methods: &[Guard<&'static [JSFunctionSpec]>],
    regular_properties: &[Guard<&'static [JSPropertySpec]>],
    constants: &[Guard<&[ConstantSpec]>],
    unscopable_names: &[&CStr],
    rval: MutableHandleObject,
) {
    create_object(
        cx,
        global,
        proto,
        class,
        regular_methods,
        regular_properties,
        constants,
        rval,
    );

    if !unscopable_names.is_empty() {
        rooted!(in(*cx) let mut unscopable_obj = ptr::null_mut::<JSObject>());
        create_unscopable_object(cx, unscopable_names, unscopable_obj.handle_mut());
        unsafe {
            let unscopable_symbol = GetWellKnownSymbol(*cx, SymbolCode::unscopables);
            assert!(!unscopable_symbol.is_null());

            rooted!(in(*cx) let mut unscopable_id: jsid);
            RUST_SYMBOL_TO_JSID(unscopable_symbol, unscopable_id.handle_mut());

            assert!(JS_DefinePropertyById5(
                *cx,
                rval.handle(),
                unscopable_id.handle(),
                unscopable_obj.handle(),
                JSPROP_READONLY as u32
            ))
        }
    }
}

/// Create and define the interface object of a non-callback interface.
#[allow(clippy::too_many_arguments)]
pub fn create_noncallback_interface_object(
    cx: SafeJSContext,
    global: HandleObject,
    proto: HandleObject,
    class: &'static NonCallbackInterfaceObjectClass,
    static_methods: &[Guard<&'static [JSFunctionSpec]>],
    static_properties: &[Guard<&'static [JSPropertySpec]>],
    constants: &[Guard<&[ConstantSpec]>],
    interface_prototype_object: HandleObject,
    name: &CStr,
    length: u32,
    legacy_window_alias_names: &[&CStr],
    rval: MutableHandleObject,
) {
    create_object(
        cx,
        global,
        proto,
        class.as_jsclass(),
        static_methods,
        static_properties,
        constants,
        rval,
    );
    unsafe {
        assert!(JS_LinkConstructorAndPrototype(
            *cx,
            rval.handle(),
            interface_prototype_object
        ));
    }
    define_name(cx, rval.handle(), name);
    define_length(cx, rval.handle(), i32::try_from(length).expect("overflow"));
    define_on_global_object(cx, global, name, rval.handle());

    if is_exposed_in(global, Globals::WINDOW) {
        for legacy_window_alias in legacy_window_alias_names {
            define_on_global_object(cx, global, legacy_window_alias, rval.handle());
        }
    }
}

/// Create and define the named constructors of a non-callback interface.
pub fn create_named_constructors(
    cx: SafeJSContext,
    global: HandleObject,
    named_constructors: &[(ConstructorClassHook, &CStr, u32)],
    interface_prototype_object: HandleObject,
) {
    rooted!(in(*cx) let mut constructor = ptr::null_mut::<JSObject>());

    for &(native, name, arity) in named_constructors {
        unsafe {
            let fun = JS_NewFunction(*cx, Some(native), arity, JSFUN_CONSTRUCTOR, name.as_ptr());
            assert!(!fun.is_null());
            constructor.set(JS_GetFunctionObject(fun));
            assert!(!constructor.is_null());

            assert!(JS_DefineProperty3(
                *cx,
                constructor.handle(),
                c"prototype".as_ptr(),
                interface_prototype_object,
                (JSPROP_PERMANENT | JSPROP_READONLY) as u32
            ));
        }

        define_on_global_object(cx, global, name, constructor.handle());
    }
}

/// Create a new object with a unique type.
#[allow(clippy::too_many_arguments)]
pub fn create_object(
    cx: SafeJSContext,
    global: HandleObject,
    proto: HandleObject,
    class: &'static JSClass,
    methods: &[Guard<&'static [JSFunctionSpec]>],
    properties: &[Guard<&'static [JSPropertySpec]>],
    constants: &[Guard<&[ConstantSpec]>],
    mut rval: MutableHandleObject,
) {
    unsafe {
        rval.set(JS_NewObjectWithGivenProto(*cx, class, proto));
    }
    assert!(!rval.is_null());
    define_guarded_methods(cx, rval.handle(), methods, global);
    define_guarded_properties(cx, rval.handle(), properties, global);
    define_guarded_constants(cx, rval.handle(), constants, global);
}

/// Conditionally define constants on an object.
pub fn define_guarded_constants(
    cx: SafeJSContext,
    obj: HandleObject,
    constants: &[Guard<&[ConstantSpec]>],
    global: HandleObject,
) {
    for guard in constants {
        if let Some(specs) = guard.expose(cx, obj, global) {
            define_constants(cx, obj, specs);
        }
    }
}

/// Conditionally define methods on an object.
pub fn define_guarded_methods(
    cx: SafeJSContext,
    obj: HandleObject,
    methods: &[Guard<&'static [JSFunctionSpec]>],
    global: HandleObject,
) {
    for guard in methods {
        if let Some(specs) = guard.expose(cx, obj, global) {
            unsafe {
                define_methods(*cx, obj, specs).unwrap();
            }
        }
    }
}

/// Conditionally define properties on an object.
pub fn define_guarded_properties(
    cx: SafeJSContext,
    obj: HandleObject,
    properties: &[Guard<&'static [JSPropertySpec]>],
    global: HandleObject,
) {
    for guard in properties {
        if let Some(specs) = guard.expose(cx, obj, global) {
            unsafe {
                define_properties(*cx, obj, specs).unwrap();
            }
        }
    }
}

/// Returns whether an interface with exposure set given by `globals` should
/// be exposed in the global object `obj`.
pub fn is_exposed_in(object: HandleObject, globals: Globals) -> bool {
    unsafe {
        let unwrapped = UncheckedUnwrapObject(object.get(), /* stopAtWindowProxy = */ false);
        let dom_class = get_dom_class(unwrapped).unwrap();
        globals.contains(dom_class.global)
    }
}

/// Define a property with a given name on the global object. Should be called
/// through the resolve hook.
pub fn define_on_global_object(
    cx: SafeJSContext,
    global: HandleObject,
    name: &CStr,
    obj: HandleObject,
) {
    unsafe {
        assert!(JS_DefineProperty3(
            *cx,
            global,
            name.as_ptr(),
            obj,
            JSPROP_RESOLVING
        ));
    }
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

unsafe extern "C" fn fun_to_string_hook(
    cx: *mut JSContext,
    obj: RawHandleObject,
    _is_to_source: bool,
) -> *mut JSString {
    let js_class = get_object_class(obj.get());
    assert!(!js_class.is_null());
    let repr = (*(js_class as *const NonCallbackInterfaceObjectClass)).representation;
    assert!(!repr.is_empty());
    let ret = JS_NewStringCopyN(cx, repr.as_ptr() as *const libc::c_char, repr.len());
    assert!(!ret.is_null());
    ret
}

fn create_unscopable_object(cx: SafeJSContext, names: &[&CStr], mut rval: MutableHandleObject) {
    assert!(!names.is_empty());
    assert!(rval.is_null());
    unsafe {
        rval.set(JS_NewPlainObject(*cx));
        assert!(!rval.is_null());
        for &name in names {
            assert!(JS_DefineProperty(
                *cx,
                rval.handle(),
                name.as_ptr(),
                HandleValue::from_raw(TrueHandleValue),
                JSPROP_READONLY as u32,
            ));
        }
    }
}

fn define_name(cx: SafeJSContext, obj: HandleObject, name: &CStr) {
    unsafe {
        rooted!(in(*cx) let name = JS_AtomizeAndPinString(*cx, name.as_ptr()));
        assert!(!name.is_null());
        assert!(JS_DefineProperty4(
            *cx,
            obj,
            c"name".as_ptr(),
            name.handle(),
            JSPROP_READONLY as u32
        ));
    }
}

fn define_length(cx: SafeJSContext, obj: HandleObject, length: i32) {
    unsafe {
        assert!(JS_DefineProperty5(
            *cx,
            obj,
            c"length".as_ptr(),
            length,
            JSPROP_READONLY as u32
        ));
    }
}

unsafe extern "C" fn invalid_constructor(
    cx: *mut JSContext,
    _argc: libc::c_uint,
    _vp: *mut JSVal,
) -> bool {
    throw_type_error(cx, "Illegal constructor.");
    false
}

unsafe extern "C" fn non_new_constructor(
    cx: *mut JSContext,
    _argc: libc::c_uint,
    _vp: *mut JSVal,
) -> bool {
    throw_type_error(cx, "This constructor needs to be called with `new`.");
    false
}

pub enum ProtoOrIfaceIndex {
    ID(PrototypeList::ID),
    Constructor(PrototypeList::Constructor),
}

impl From<ProtoOrIfaceIndex> for usize {
    fn from(index: ProtoOrIfaceIndex) -> usize {
        match index {
            ProtoOrIfaceIndex::ID(id) => id as usize,
            ProtoOrIfaceIndex::Constructor(constructor) => constructor as usize,
        }
    }
}

pub fn get_per_interface_object_handle(
    cx: SafeJSContext,
    global: HandleObject,
    id: ProtoOrIfaceIndex,
    creator: unsafe fn(SafeJSContext, HandleObject, *mut ProtoOrIfaceArray),
    mut rval: MutableHandleObject,
) {
    unsafe {
        assert!(((*get_object_class(global.get())).flags & JSCLASS_DOM_GLOBAL) != 0);

        /* Check to see whether the interface objects are already installed */
        let proto_or_iface_array = get_proto_or_iface_array(global.get());
        let index: usize = id.into();
        rval.set((*proto_or_iface_array)[index]);
        if !rval.get().is_null() {
            return;
        }

        creator(cx, global, proto_or_iface_array);
        rval.set((*proto_or_iface_array)[index]);
        assert!(!rval.get().is_null());
    }
}

pub fn define_dom_interface(
    cx: SafeJSContext,
    global: HandleObject,
    id: ProtoOrIfaceIndex,
    creator: unsafe fn(SafeJSContext, HandleObject, *mut ProtoOrIfaceArray),
    enabled: fn(SafeJSContext, HandleObject) -> bool,
) {
    assert!(!global.get().is_null());

    if !enabled(cx, global) {
        return;
    }

    rooted!(in(*cx) let mut proto = ptr::null_mut::<JSObject>());
    get_per_interface_object_handle(cx, global, id, creator, proto.handle_mut());
    assert!(!proto.is_null());
}

fn get_proto_id_for_new_target(new_target: HandleObject) -> Option<PrototypeList::ID> {
    unsafe {
        let new_target_class = get_object_class(*new_target);
        if is_dom_class(&*new_target_class) {
            let domjsclass: *const DOMJSClass = new_target_class as *const DOMJSClass;
            let dom_class = &(*domjsclass).dom_class;
            return Some(dom_class.interface_chain[dom_class.depth as usize]);
        }
        None
    }
}

pub fn get_desired_proto(
    cx: SafeJSContext,
    args: &CallArgs,
    proto_id: PrototypeList::ID,
    creator: unsafe fn(SafeJSContext, HandleObject, *mut ProtoOrIfaceArray),
    mut desired_proto: MutableHandleObject,
) -> Result<(), ()> {
    unsafe {
        // This basically implements
        // https://heycam.github.io/webidl/#internally-create-a-new-object-implementing-the-interface
        // step 3.

        assert!(callargs_is_constructing(args));

        // The desired prototype depends on the actual constructor that was invoked,
        // which is passed to us as the newTarget in the callargs.  We want to do
        // something akin to the ES6 specification's GetProtototypeFromConstructor (so
        // get .prototype on the newTarget, with a fallback to some sort of default).

        // First, a fast path for the case when the the constructor is in fact one of
        // our DOM constructors.  This is safe because on those the "constructor"
        // property is non-configurable and non-writable, so we don't have to do the
        // slow JS_GetProperty call.
        rooted!(in(*cx) let mut new_target = args.new_target().to_object());
        rooted!(in(*cx) let original_new_target = *new_target);
        // See whether we have a known DOM constructor here, such that we can take a
        // fast path.
        let target_proto_id = get_proto_id_for_new_target(new_target.handle()).or_else(|| {
            // We might still have a cross-compartment wrapper for a known DOM
            // constructor.  CheckedUnwrapStatic is fine here, because we're looking for
            // DOM constructors and those can't be cross-origin objects.
            *new_target = CheckedUnwrapStatic(*new_target);
            if !new_target.is_null() && *new_target != *original_new_target {
                get_proto_id_for_new_target(new_target.handle())
            } else {
                None
            }
        });

        if let Some(proto_id) = target_proto_id {
            let global = GetNonCCWObjectGlobal(*new_target);
            let proto_or_iface_cache = get_proto_or_iface_array(global);
            desired_proto.set((*proto_or_iface_cache)[proto_id as usize]);
            if *new_target != *original_new_target && !JS_WrapObject(*cx, desired_proto.into()) {
                return Err(());
            }
            return Ok(());
        }

        // Slow path.  This basically duplicates the ES6 spec's
        // GetPrototypeFromConstructor except that instead of taking a string naming
        // the fallback prototype we determine the fallback based on the proto id we
        // were handed.
        rooted!(in(*cx) let mut proto_val = NullValue());
        if !JS_GetProperty(
            *cx,
            original_new_target.handle().into(),
            c"prototype".as_ptr(),
            proto_val.handle_mut().into(),
        ) {
            return Err(());
        }

        if proto_val.is_object() {
            desired_proto.set(proto_val.to_object());
            return Ok(());
        }

        // Fall back to getting the proto for our given proto id in the realm that
        // GetFunctionRealm(newTarget) returns.
        let realm = GetFunctionRealm(*cx, new_target.handle().into());

        if realm.is_null() {
            return Err(());
        }

        {
            let _realm = JSAutoRealm::new(*cx, GetRealmGlobalOrNull(realm));
            rooted!(in(*cx) let global = CurrentGlobalOrNull(*cx));
            get_per_interface_object_handle(
                cx,
                global.handle(),
                ProtoOrIfaceIndex::ID(proto_id),
                creator,
                desired_proto,
            );
            if desired_proto.is_null() {
                return Err(());
            }
        }

        maybe_wrap_object(*cx, desired_proto);
        Ok(())
    }
}
