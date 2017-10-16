/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Machinery to initialise interface prototype objects and interface objects.

use dom::bindings::codegen::Bindings::HTMLAnchorElementBinding;
use dom::bindings::codegen::Bindings::HTMLAreaElementBinding;
use dom::bindings::codegen::Bindings::HTMLAudioElementBinding;
use dom::bindings::codegen::Bindings::HTMLBRElementBinding;
use dom::bindings::codegen::Bindings::HTMLBaseElementBinding;
use dom::bindings::codegen::Bindings::HTMLBodyElementBinding;
use dom::bindings::codegen::Bindings::HTMLButtonElementBinding;
use dom::bindings::codegen::Bindings::HTMLCanvasElementBinding;
use dom::bindings::codegen::Bindings::HTMLDListElementBinding;
use dom::bindings::codegen::Bindings::HTMLDataElementBinding;
use dom::bindings::codegen::Bindings::HTMLDataListElementBinding;
use dom::bindings::codegen::Bindings::HTMLDetailsElementBinding;
use dom::bindings::codegen::Bindings::HTMLDialogElementBinding;
use dom::bindings::codegen::Bindings::HTMLDirectoryElementBinding;
use dom::bindings::codegen::Bindings::HTMLDivElementBinding;
use dom::bindings::codegen::Bindings::HTMLElementBinding;
use dom::bindings::codegen::Bindings::HTMLEmbedElementBinding;
use dom::bindings::codegen::Bindings::HTMLFieldSetElementBinding;
use dom::bindings::codegen::Bindings::HTMLFontElementBinding;
use dom::bindings::codegen::Bindings::HTMLFormElementBinding;
use dom::bindings::codegen::Bindings::HTMLFrameElementBinding;
use dom::bindings::codegen::Bindings::HTMLFrameSetElementBinding;
use dom::bindings::codegen::Bindings::HTMLHRElementBinding;
use dom::bindings::codegen::Bindings::HTMLHeadElementBinding;
use dom::bindings::codegen::Bindings::HTMLHeadingElementBinding;
use dom::bindings::codegen::Bindings::HTMLHtmlElementBinding;
use dom::bindings::codegen::Bindings::HTMLIFrameElementBinding;
use dom::bindings::codegen::Bindings::HTMLImageElementBinding;
use dom::bindings::codegen::Bindings::HTMLInputElementBinding;
use dom::bindings::codegen::Bindings::HTMLLIElementBinding;
use dom::bindings::codegen::Bindings::HTMLLabelElementBinding;
use dom::bindings::codegen::Bindings::HTMLLegendElementBinding;
use dom::bindings::codegen::Bindings::HTMLLinkElementBinding;
use dom::bindings::codegen::Bindings::HTMLMapElementBinding;
use dom::bindings::codegen::Bindings::HTMLMetaElementBinding;
use dom::bindings::codegen::Bindings::HTMLMeterElementBinding;
use dom::bindings::codegen::Bindings::HTMLModElementBinding;
use dom::bindings::codegen::Bindings::HTMLOListElementBinding;
use dom::bindings::codegen::Bindings::HTMLObjectElementBinding;
use dom::bindings::codegen::Bindings::HTMLOptGroupElementBinding;
use dom::bindings::codegen::Bindings::HTMLOptionElementBinding;
use dom::bindings::codegen::Bindings::HTMLOutputElementBinding;
use dom::bindings::codegen::Bindings::HTMLParagraphElementBinding;
use dom::bindings::codegen::Bindings::HTMLParamElementBinding;
use dom::bindings::codegen::Bindings::HTMLPreElementBinding;
use dom::bindings::codegen::Bindings::HTMLProgressElementBinding;
use dom::bindings::codegen::Bindings::HTMLQuoteElementBinding;
use dom::bindings::codegen::Bindings::HTMLScriptElementBinding;
use dom::bindings::codegen::Bindings::HTMLSelectElementBinding;
use dom::bindings::codegen::Bindings::HTMLSourceElementBinding;
use dom::bindings::codegen::Bindings::HTMLSpanElementBinding;
use dom::bindings::codegen::Bindings::HTMLStyleElementBinding;
use dom::bindings::codegen::Bindings::HTMLTableCaptionElementBinding;
use dom::bindings::codegen::Bindings::HTMLTableColElementBinding;
use dom::bindings::codegen::Bindings::HTMLTableDataCellElementBinding;
use dom::bindings::codegen::Bindings::HTMLTableElementBinding;
use dom::bindings::codegen::Bindings::HTMLTableHeaderCellElementBinding;
use dom::bindings::codegen::Bindings::HTMLTableRowElementBinding;
use dom::bindings::codegen::Bindings::HTMLTableSectionElementBinding;
use dom::bindings::codegen::Bindings::HTMLTemplateElementBinding;
use dom::bindings::codegen::Bindings::HTMLTextAreaElementBinding;
use dom::bindings::codegen::Bindings::HTMLTimeElementBinding;
use dom::bindings::codegen::Bindings::HTMLTitleElementBinding;
use dom::bindings::codegen::Bindings::HTMLTrackElementBinding;
use dom::bindings::codegen::Bindings::HTMLUListElementBinding;
use dom::bindings::codegen::Bindings::HTMLVideoElementBinding;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::codegen::InterfaceObjectMap::Globals;
use dom::bindings::codegen::PrototypeList;
use dom::bindings::constant::{ConstantSpec, define_constants};
use dom::bindings::conversions::{DOM_OBJECT_SLOT, DerivedFrom, get_dom_class};
use dom::bindings::error::{Error, Fallible};
use dom::bindings::guard::Guard;
use dom::bindings::root::DomRoot;
use dom::bindings::utils::{DOM_PROTOTYPE_SLOT, ProtoOrIfaceArray, get_proto_or_iface_array};
use dom::create::create_native_html_element;
use dom::customelementregistry::ConstructionStackEntry;
use dom::element::{CustomElementState, Element, ElementCreator};
use dom::htmlelement::HTMLElement;
use dom::window::Window;
use html5ever::LocalName;
use html5ever::interface::QualName;
use js::error::throw_type_error;
use js::glue::{RUST_SYMBOL_TO_JSID, UncheckedUnwrapObject, UnwrapObject};
use js::jsapi::{CallArgs, Class, ClassOps, CompartmentOptions, CurrentGlobalOrNull};
use js::jsapi::{GetGlobalForObjectCrossCompartment, GetWellKnownSymbol, HandleObject, HandleValue};
use js::jsapi::{JSAutoCompartment, JSClass, JSContext, JSFUN_CONSTRUCTOR, JSFunctionSpec, JSObject};
use js::jsapi::{JSPROP_PERMANENT, JSPROP_READONLY, JSPROP_RESOLVING};
use js::jsapi::{JSPropertySpec, JSString, JSTracer, JSVersion, JS_AtomizeAndPinString};
use js::jsapi::{JS_DefineProperty, JS_DefineProperty1, JS_DefineProperty2};
use js::jsapi::{JS_DefineProperty4, JS_DefinePropertyById3, JS_FireOnNewGlobalObject};
use js::jsapi::{JS_GetFunctionObject, JS_GetPrototype};
use js::jsapi::{JS_LinkConstructorAndPrototype, JS_NewFunction, JS_NewGlobalObject};
use js::jsapi::{JS_NewObject, JS_NewObjectWithUniqueType, JS_NewPlainObject};
use js::jsapi::{JS_NewStringCopyN, JS_SetReservedSlot, MutableHandleObject};
use js::jsapi::{MutableHandleValue, ObjectOps, OnNewGlobalHookOption, SymbolCode};
use js::jsapi::{TrueHandleValue, Value};
use js::jsval::{JSVal, PrivateValue};
use js::rust::{define_methods, define_properties, get_object_class};
use libc;
use script_thread::ScriptThread;
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
                spec: ptr::null(),
                ext: ptr::null(),
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
            getProperty: None,
            setProperty: None,
            enumerate: None,
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
            getProperty: None,
            setProperty: None,
            enumerate: None,
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
        rval: MutableHandleObject) {
    assert!(rval.is_null());

    let mut options = CompartmentOptions::default();
    options.behaviors_.version_ = JSVersion::JSVERSION_ECMA_5;
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
    JS_SetReservedSlot(rval.get(), DOM_OBJECT_SLOT, PrivateValue(private));
    let proto_array: Box<ProtoOrIfaceArray> =
        Box::new([0 as *mut JSObject; PrototypeList::PROTO_OR_IFACE_LENGTH]);
    JS_SetReservedSlot(rval.get(),
                       DOM_PROTOTYPE_SLOT,
                       PrivateValue(Box::into_raw(proto_array) as *const libc::c_void));

    let _ac = JSAutoCompartment::new(cx, rval.get());
    JS_FireOnNewGlobalObject(cx, rval.handle());
}

// https://html.spec.whatwg.org/multipage/#htmlconstructor
pub unsafe fn html_constructor<T>(window: &Window, call_args: &CallArgs) -> Fallible<DomRoot<T>>
                                  where T: DerivedFrom<Element> {
    let document = window.Document();

    // Step 1
    let registry = window.CustomElements();

    // Step 2 is checked in the generated caller code

    // Step 3
    rooted!(in(window.get_cx()) let new_target = call_args.new_target().to_object());
    let definition = match registry.lookup_definition_by_constructor(new_target.handle()) {
        Some(definition) => definition,
        None => return Err(Error::Type("No custom element definition found for new.target".to_owned())),
    };

    rooted!(in(window.get_cx()) let callee = UnwrapObject(call_args.callee(), 1));
    if callee.is_null() {
        return Err(Error::Security);
    }

    {
        let _ac = JSAutoCompartment::new(window.get_cx(), callee.get());
        rooted!(in(window.get_cx()) let mut constructor = ptr::null_mut());
        rooted!(in(window.get_cx()) let global_object = CurrentGlobalOrNull(window.get_cx()));

        if definition.is_autonomous() {
            // Step 4
            // Since this element is autonomous, its active function object must be the HTMLElement

            // Retrieve the constructor object for HTMLElement
            HTMLElementBinding::GetConstructorObject(window.get_cx(), global_object.handle(), constructor.handle_mut());

        } else {
            // Step 5
            get_constructor_object_from_local_name(definition.local_name.clone(),
                                                   window.get_cx(),
                                                   global_object.handle(),
                                                   constructor.handle_mut());
        }
        // Callee must be the same as the element interface's constructor object.
        if constructor.get() != callee.get() {
            return Err(Error::Type("Custom element does not extend the proper interface".to_owned()));
        }
    }

    let entry = definition.construction_stack.borrow().last().cloned();
    match entry {
        // Step 8
        None => {
            // Step 8.1
            let name = QualName::new(None, ns!(html), definition.local_name.clone());
            let element = if definition.is_autonomous() {
                DomRoot::upcast(HTMLElement::new(name.local, None, &*document))
            } else {
                create_native_html_element(name, None, &*document, ElementCreator::ScriptCreated)
            };

            // Step 8.2 is performed in the generated caller code.

            // Step 8.3
            element.set_custom_element_state(CustomElementState::Custom);

            // Step 8.4
            element.set_custom_element_definition(definition.clone());

            // Step 8.5
            DomRoot::downcast(element).ok_or(Error::InvalidState)
        },
        // Step 9
        Some(ConstructionStackEntry::Element(element)) => {
            // Step 11 is performed in the generated caller code.

            // Step 12
            let mut construction_stack = definition.construction_stack.borrow_mut();
            construction_stack.pop();
            construction_stack.push(ConstructionStackEntry::AlreadyConstructedMarker);

            // Step 13
            DomRoot::downcast(element).ok_or(Error::InvalidState)
        },
        // Step 10
        Some(ConstructionStackEntry::AlreadyConstructedMarker) => Err(Error::InvalidState),
    }
}

pub fn push_new_element_queue() {
    ScriptThread::push_new_element_queue();
}

pub fn pop_current_element_queue() {
    ScriptThread::pop_current_element_queue();
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
        named_constructors: &[(ConstructorClassHook, &[u8], u32)],
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

/// Create a new object with a unique type.
pub unsafe fn create_object(
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
    assert!(*name.last().unwrap() == b'\0');
    assert!(JS_DefineProperty1(cx,
                               global,
                               name.as_ptr() as *const libc::c_char,
                               obj,
                               JSPROP_RESOLVING,
                               None, None));
}

const OBJECT_OPS: ObjectOps = ObjectOps {
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
};

unsafe extern "C" fn fun_to_string_hook(cx: *mut JSContext,
                                        obj: HandleObject,
                                        _indent: u32)
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

/// Returns the constructor object for the element associated with the given local name.
/// This list should only include elements marked with the [HTMLConstructor] extended attribute.
pub fn get_constructor_object_from_local_name(name: LocalName,
                                              cx: *mut JSContext,
                                              global: HandleObject,
                                              rval: MutableHandleObject)
                                              -> bool {
    macro_rules! get_constructor(
        ($binding:ident) => ({
            unsafe { $binding::GetConstructorObject(cx, global, rval); }
            true
        })
    );

    match name {
        local_name!("a")          => get_constructor!(HTMLAnchorElementBinding),
        local_name!("abbr")       => get_constructor!(HTMLElementBinding),
        local_name!("acronym")    => get_constructor!(HTMLElementBinding),
        local_name!("address")    => get_constructor!(HTMLElementBinding),
        local_name!("area")       => get_constructor!(HTMLAreaElementBinding),
        local_name!("article")    => get_constructor!(HTMLElementBinding),
        local_name!("aside")      => get_constructor!(HTMLElementBinding),
        local_name!("audio")      => get_constructor!(HTMLAudioElementBinding),
        local_name!("b")          => get_constructor!(HTMLElementBinding),
        local_name!("base")       => get_constructor!(HTMLBaseElementBinding),
        local_name!("bdi")        => get_constructor!(HTMLElementBinding),
        local_name!("bdo")        => get_constructor!(HTMLElementBinding),
        local_name!("big")        => get_constructor!(HTMLElementBinding),
        local_name!("blockquote") => get_constructor!(HTMLQuoteElementBinding),
        local_name!("body")       => get_constructor!(HTMLBodyElementBinding),
        local_name!("br")         => get_constructor!(HTMLBRElementBinding),
        local_name!("button")     => get_constructor!(HTMLButtonElementBinding),
        local_name!("canvas")     => get_constructor!(HTMLCanvasElementBinding),
        local_name!("caption")    => get_constructor!(HTMLTableCaptionElementBinding),
        local_name!("center")     => get_constructor!(HTMLElementBinding),
        local_name!("cite")       => get_constructor!(HTMLElementBinding),
        local_name!("code")       => get_constructor!(HTMLElementBinding),
        local_name!("col")        => get_constructor!(HTMLTableColElementBinding),
        local_name!("colgroup")   => get_constructor!(HTMLTableColElementBinding),
        local_name!("data")       => get_constructor!(HTMLDataElementBinding),
        local_name!("datalist")   => get_constructor!(HTMLDataListElementBinding),
        local_name!("dd")         => get_constructor!(HTMLElementBinding),
        local_name!("del")        => get_constructor!(HTMLModElementBinding),
        local_name!("details")    => get_constructor!(HTMLDetailsElementBinding),
        local_name!("dfn")        => get_constructor!(HTMLElementBinding),
        local_name!("dialog")     => get_constructor!(HTMLDialogElementBinding),
        local_name!("dir")        => get_constructor!(HTMLDirectoryElementBinding),
        local_name!("div")        => get_constructor!(HTMLDivElementBinding),
        local_name!("dl")         => get_constructor!(HTMLDListElementBinding),
        local_name!("dt")         => get_constructor!(HTMLElementBinding),
        local_name!("em")         => get_constructor!(HTMLElementBinding),
        local_name!("embed")      => get_constructor!(HTMLEmbedElementBinding),
        local_name!("fieldset")   => get_constructor!(HTMLFieldSetElementBinding),
        local_name!("figcaption") => get_constructor!(HTMLElementBinding),
        local_name!("figure")     => get_constructor!(HTMLElementBinding),
        local_name!("font")       => get_constructor!(HTMLFontElementBinding),
        local_name!("footer")     => get_constructor!(HTMLElementBinding),
        local_name!("form")       => get_constructor!(HTMLFormElementBinding),
        local_name!("frame")      => get_constructor!(HTMLFrameElementBinding),
        local_name!("frameset")   => get_constructor!(HTMLFrameSetElementBinding),
        local_name!("h1")         => get_constructor!(HTMLHeadingElementBinding),
        local_name!("h2")         => get_constructor!(HTMLHeadingElementBinding),
        local_name!("h3")         => get_constructor!(HTMLHeadingElementBinding),
        local_name!("h4")         => get_constructor!(HTMLHeadingElementBinding),
        local_name!("h5")         => get_constructor!(HTMLHeadingElementBinding),
        local_name!("h6")         => get_constructor!(HTMLHeadingElementBinding),
        local_name!("head")       => get_constructor!(HTMLHeadElementBinding),
        local_name!("header")     => get_constructor!(HTMLElementBinding),
        local_name!("hgroup")     => get_constructor!(HTMLElementBinding),
        local_name!("hr")         => get_constructor!(HTMLHRElementBinding),
        local_name!("html")       => get_constructor!(HTMLHtmlElementBinding),
        local_name!("i")          => get_constructor!(HTMLElementBinding),
        local_name!("iframe")     => get_constructor!(HTMLIFrameElementBinding),
        local_name!("img")        => get_constructor!(HTMLImageElementBinding),
        local_name!("input")      => get_constructor!(HTMLInputElementBinding),
        local_name!("ins")        => get_constructor!(HTMLModElementBinding),
        local_name!("kbd")        => get_constructor!(HTMLElementBinding),
        local_name!("label")      => get_constructor!(HTMLLabelElementBinding),
        local_name!("legend")     => get_constructor!(HTMLLegendElementBinding),
        local_name!("li")         => get_constructor!(HTMLLIElementBinding),
        local_name!("link")       => get_constructor!(HTMLLinkElementBinding),
        local_name!("listing")    => get_constructor!(HTMLPreElementBinding),
        local_name!("main")       => get_constructor!(HTMLElementBinding),
        local_name!("map")        => get_constructor!(HTMLMapElementBinding),
        local_name!("mark")       => get_constructor!(HTMLElementBinding),
        local_name!("marquee")    => get_constructor!(HTMLElementBinding),
        local_name!("meta")       => get_constructor!(HTMLMetaElementBinding),
        local_name!("meter")      => get_constructor!(HTMLMeterElementBinding),
        local_name!("nav")        => get_constructor!(HTMLElementBinding),
        local_name!("nobr")       => get_constructor!(HTMLElementBinding),
        local_name!("noframes")   => get_constructor!(HTMLElementBinding),
        local_name!("noscript")   => get_constructor!(HTMLElementBinding),
        local_name!("object")     => get_constructor!(HTMLObjectElementBinding),
        local_name!("ol")         => get_constructor!(HTMLOListElementBinding),
        local_name!("optgroup")   => get_constructor!(HTMLOptGroupElementBinding),
        local_name!("option")     => get_constructor!(HTMLOptionElementBinding),
        local_name!("output")     => get_constructor!(HTMLOutputElementBinding),
        local_name!("p")          => get_constructor!(HTMLParagraphElementBinding),
        local_name!("param")      => get_constructor!(HTMLParamElementBinding),
        local_name!("plaintext")  => get_constructor!(HTMLPreElementBinding),
        local_name!("pre")        => get_constructor!(HTMLPreElementBinding),
        local_name!("progress")   => get_constructor!(HTMLProgressElementBinding),
        local_name!("q")          => get_constructor!(HTMLQuoteElementBinding),
        local_name!("rp")         => get_constructor!(HTMLElementBinding),
        local_name!("rt")         => get_constructor!(HTMLElementBinding),
        local_name!("ruby")       => get_constructor!(HTMLElementBinding),
        local_name!("s")          => get_constructor!(HTMLElementBinding),
        local_name!("samp")       => get_constructor!(HTMLElementBinding),
        local_name!("script")     => get_constructor!(HTMLScriptElementBinding),
        local_name!("section")    => get_constructor!(HTMLElementBinding),
        local_name!("select")     => get_constructor!(HTMLSelectElementBinding),
        local_name!("small")      => get_constructor!(HTMLElementBinding),
        local_name!("source")     => get_constructor!(HTMLSourceElementBinding),
        local_name!("span")       => get_constructor!(HTMLSpanElementBinding),
        local_name!("strike")     => get_constructor!(HTMLElementBinding),
        local_name!("strong")     => get_constructor!(HTMLElementBinding),
        local_name!("style")      => get_constructor!(HTMLStyleElementBinding),
        local_name!("sub")        => get_constructor!(HTMLElementBinding),
        local_name!("summary")    => get_constructor!(HTMLElementBinding),
        local_name!("sup")        => get_constructor!(HTMLElementBinding),
        local_name!("table")      => get_constructor!(HTMLTableElementBinding),
        local_name!("tbody")      => get_constructor!(HTMLTableSectionElementBinding),
        local_name!("td")         => get_constructor!(HTMLTableDataCellElementBinding),
        local_name!("template")   => get_constructor!(HTMLTemplateElementBinding),
        local_name!("textarea")   => get_constructor!(HTMLTextAreaElementBinding),
        local_name!("tfoot")      => get_constructor!(HTMLTableSectionElementBinding),
        local_name!("th")         => get_constructor!(HTMLTableHeaderCellElementBinding),
        local_name!("thead")      => get_constructor!(HTMLTableSectionElementBinding),
        local_name!("time")       => get_constructor!(HTMLTimeElementBinding),
        local_name!("title")      => get_constructor!(HTMLTitleElementBinding),
        local_name!("tr")         => get_constructor!(HTMLTableRowElementBinding),
        local_name!("tt")         => get_constructor!(HTMLElementBinding),
        local_name!("track")      => get_constructor!(HTMLTrackElementBinding),
        local_name!("u")          => get_constructor!(HTMLElementBinding),
        local_name!("ul")         => get_constructor!(HTMLUListElementBinding),
        local_name!("var")        => get_constructor!(HTMLElementBinding),
        local_name!("video")      => get_constructor!(HTMLVideoElementBinding),
        local_name!("wbr")        => get_constructor!(HTMLElementBinding),
        local_name!("xmp")        => get_constructor!(HTMLPreElementBinding),
        _                         => false,
    }
}
