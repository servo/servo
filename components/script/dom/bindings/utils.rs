/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Various utilities to glue JavaScript and the DOM implementation together.

use dom::bindings::codegen::PrototypeList;
use dom::bindings::codegen::PrototypeList::MAX_PROTO_CHAIN_LENGTH;
use dom::bindings::conversions::{DOM_OBJECT_SLOT, is_dom_class};
use dom::bindings::conversions::{private_from_proto_check, root_from_handleobject};
use dom::bindings::error::throw_invalid_this;
use dom::bindings::inheritance::TopTypeId;
use dom::bindings::trace::trace_object;
use dom::browsercontext;
use dom::window;
use js;
use js::error::throw_type_error;
use js::glue::{CallJitGetterOp, CallJitMethodOp, CallJitSetterOp, IsWrapper};
use js::glue::{GetCrossCompartmentWrapper, WrapperNew};
use js::glue::{RUST_FUNCTION_VALUE_TO_JITINFO, RUST_JSID_IS_INT};
use js::glue::{RUST_JSID_TO_INT, UnwrapObject};
use js::jsapi::JSAutoCompartment;
use js::jsapi::JS_DeletePropertyById1;
use js::jsapi::JS_GetFunctionObject;
use js::jsapi::JS_IsExceptionPending;
use js::jsapi::JS_NewObjectWithUniqueType;
use js::jsapi::JS_ObjectToOuterObject;
use js::jsapi::{CallArgs, GetGlobalForObjectCrossCompartment, JSJitInfo};
use js::jsapi::{CompartmentOptions, OnNewGlobalHookOption};
use js::jsapi::{DOMCallbacks, JSWrapObjectCallbacks};
use js::jsapi::{HandleId, HandleObject, HandleValue, MutableHandleValue};
use js::jsapi::{Heap, MutableHandleObject, ObjectOpResult, RootedObject, RootedValue};
use js::jsapi::{JSClass, JSContext, JSObject, JSTracer};
use js::jsapi::{JSFunctionSpec, JSPropertySpec};
use js::jsapi::{JSTraceOp, JS_AlreadyHasOwnProperty, JS_NewFunction};
use js::jsapi::{JSVersion, JS_FireOnNewGlobalObject};
use js::jsapi::{JS_DefineProperty, JS_DefineProperty1, JS_ForwardGetPropertyTo};
use js::jsapi::{JS_GetClass, JS_LinkConstructorAndPrototype};
use js::jsapi::{JS_GetProperty, JS_HasProperty, JS_SetProperty};
use js::jsapi::{JS_GetPrototype, JS_HasPropertyById};
use js::jsapi::{JS_GetReservedSlot, JS_SetReservedSlot};
use js::jsapi::{JS_InitStandardClasses, JS_NewGlobalObject};
use js::jsval::{BooleanValue, DoubleValue, Int32Value, JSVal, NullValue};
use js::jsval::{PrivateValue, UInt32Value, UndefinedValue};
use js::rust::{GCMethods, ToString, define_methods, define_properties};
use js::{JSFUN_CONSTRUCTOR, JSPROP_ENUMERATE, JS_CALLEE};
use js::{JSPROP_PERMANENT, JSPROP_READONLY};
use libc::{self, c_uint};
use std::default::Default;
use std::ffi::CString;
use std::ptr;
use util::mem::HeapSizeOf;
use util::str::jsstring_to_str;

/// Proxy handler for a WindowProxy.
#[allow(raw_pointer_derive)]
pub struct WindowProxyHandler(pub *const libc::c_void);

impl HeapSizeOf for WindowProxyHandler {
    fn heap_size_of_children(&self) -> usize {
        // FIXME(#6907) this is a pointer to memory allocated by `new` in NewProxyHandler in rust-mozjs.
        0
    }
}

#[allow(raw_pointer_derive)]
#[derive(JSTraceable, HeapSizeOf)]
/// Static data associated with a global object.
pub struct GlobalStaticData {
    /// The WindowProxy proxy handler for this global.
    pub windowproxy_handler: WindowProxyHandler,
}

impl GlobalStaticData {
    /// Creates a new GlobalStaticData.
    pub fn new() -> GlobalStaticData {
        GlobalStaticData {
            windowproxy_handler: browsercontext::new_window_proxy_handler(),
        }
    }
}

// NOTE: This is baked into the Ion JIT as 0 in codegen for LGetDOMProperty and
// LSetDOMProperty. Those constants need to be changed accordingly if this value
// changes.
const DOM_PROTO_INSTANCE_CLASS_SLOT: u32 = 0;

/// The index of the slot that contains a reference to the ProtoOrIfaceArray.
// All DOM globals must have a slot at DOM_PROTOTYPE_SLOT.
pub const DOM_PROTOTYPE_SLOT: u32 = js::JSCLASS_GLOBAL_SLOT_COUNT;

/// The flag set on the `JSClass`es for DOM global objects.
// NOTE: This is baked into the Ion JIT as 0 in codegen for LGetDOMProperty and
// LSetDOMProperty. Those constants need to be changed accordingly if this value
// changes.
pub const JSCLASS_DOM_GLOBAL: u32 = js::JSCLASS_USERBIT1;

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

/// Helper structure for cross-origin wrappers for DOM binding objects.
pub struct NativePropertyHooks {
    /// The property arrays for this interface.
    pub native_properties: &'static NativeProperties,

    /// The NativePropertyHooks instance for the parent interface, if any.
    pub proto_hooks: Option<&'static NativePropertyHooks>,
}

/// The struct that holds inheritance information for DOM object reflectors.
#[allow(raw_pointer_derive)]
#[derive(Copy, Clone)]
pub struct DOMClass {
    /// A list of interfaces that this object implements, in order of decreasing
    /// derivedness.
    pub interface_chain: [PrototypeList::ID; MAX_PROTO_CHAIN_LENGTH],

    /// The type ID of that interface.
    pub type_id: TopTypeId,

    /// The NativePropertyHooks for the interface associated with this class.
    pub native_hooks: &'static NativePropertyHooks,

    /// The HeapSizeOf function wrapper for that interface.
    pub heap_size_of: unsafe fn(*const libc::c_void) -> usize,
}
unsafe impl Sync for DOMClass {}

/// The JSClass used for DOM object reflectors.
#[derive(Copy)]
pub struct DOMJSClass {
    /// The actual JSClass.
    pub base: js::jsapi::Class,
    /// Associated data for DOM object reflectors.
    pub dom_class: DOMClass,
}
impl Clone for DOMJSClass {
    fn clone(&self) -> DOMJSClass {
        *self
    }
}
unsafe impl Sync for DOMJSClass {}

/// Returns the ProtoOrIfaceArray for the given global object.
/// Fails if `global` is not a DOM global object.
pub fn get_proto_or_iface_array(global: *mut JSObject) -> *mut ProtoOrIfaceArray {
    unsafe {
        assert!(((*JS_GetClass(global)).flags & JSCLASS_DOM_GLOBAL) != 0);
        JS_GetReservedSlot(global, DOM_PROTOTYPE_SLOT).to_private() as *mut ProtoOrIfaceArray
    }
}

/// Contains references to lists of methods, attributes, and constants for a
/// given interface.
pub struct NativeProperties {
    /// Instance methods for the interface.
    pub methods: Option<&'static [JSFunctionSpec]>,
    /// Instance attributes for the interface.
    pub attrs: Option<&'static [JSPropertySpec]>,
    /// Constants for the interface.
    pub consts: Option<&'static [ConstantSpec]>,
    /// Static methods for the interface.
    pub static_methods: Option<&'static [JSFunctionSpec]>,
    /// Static attributes for the interface.
    pub static_attrs: Option<&'static [JSPropertySpec]>,
}
unsafe impl Sync for NativeProperties {}

/// A JSNative that cannot be null.
pub type NonNullJSNative =
    unsafe extern "C" fn (arg1: *mut JSContext, arg2: c_uint, arg3: *mut JSVal) -> bool;

/// Creates the *interface prototype object* (if a `proto_class` is given)
/// and the *interface object* (if a `constructor` is given).
/// Fails on JSAPI failure.
pub fn do_create_interface_objects(cx: *mut JSContext,
                                   receiver: HandleObject,
                                   proto_proto: HandleObject,
                                   proto_class: Option<&'static JSClass>,
                                   constructor: Option<(NonNullJSNative, &'static str, u32)>,
                                   named_constructors: &[(NonNullJSNative, &'static str, u32)],
                                   dom_class: Option<&'static DOMClass>,
                                   members: &'static NativeProperties,
                                   rval: MutableHandleObject) {
    assert!(rval.get().is_null());
    if let Some(proto_class) = proto_class {
        create_interface_prototype_object(cx, proto_proto, proto_class, members, rval);

        if !rval.get().is_null() {
            let dom_class_ptr = match dom_class {
                Some(dom_class) => dom_class as *const DOMClass as *const libc::c_void,
                None => ptr::null() as *const libc::c_void,
            };

            unsafe {
                JS_SetReservedSlot(rval.get(),
                                   DOM_PROTO_INSTANCE_CLASS_SLOT,
                                   PrivateValue(dom_class_ptr));
            }
        }
    }

    if let Some((native, name, nargs)) = constructor {
        let s = CString::new(name).unwrap();
        create_interface_object(cx,
                                receiver,
                                native,
                                nargs,
                                rval.handle(),
                                members,
                                s.as_ptr())
    }

    for ctor in named_constructors {
        let (cnative, cname, cnargs) = *ctor;

        let cs = CString::new(cname).unwrap();
        let constructor = RootedObject::new(cx,
                                            create_constructor(cx, cnative, cnargs, cs.as_ptr()));
        assert!(!constructor.ptr.is_null());
        unsafe {
            assert!(JS_DefineProperty1(cx,
                                       constructor.handle(),
                                       b"prototype\0".as_ptr() as *const libc::c_char,
                                       rval.handle(),
                                       JSPROP_PERMANENT | JSPROP_READONLY,
                                       None,
                                       None));
        }
        define_constructor(cx, receiver, cs.as_ptr(), constructor.handle());
    }

}

fn create_constructor(cx: *mut JSContext,
                      constructor_native: NonNullJSNative,
                      ctor_nargs: u32,
                      name: *const libc::c_char)
                      -> *mut JSObject {
    unsafe {
        let fun = JS_NewFunction(cx,
                                 Some(constructor_native),
                                 ctor_nargs,
                                 JSFUN_CONSTRUCTOR,
                                 name);
        assert!(!fun.is_null());

        let constructor = JS_GetFunctionObject(fun);
        assert!(!constructor.is_null());

        constructor
    }
}

fn define_constructor(cx: *mut JSContext,
                      receiver: HandleObject,
                      name: *const libc::c_char,
                      constructor: HandleObject) {
    unsafe {
        let mut already_defined = false;
        assert!(JS_AlreadyHasOwnProperty(cx, receiver, name, &mut already_defined));

        if !already_defined {
            assert!(JS_DefineProperty1(cx, receiver, name, constructor, 0, None, None));
        }

    }
}

/// Creates the *interface object*.
/// Fails on JSAPI failure.
fn create_interface_object(cx: *mut JSContext,
                           receiver: HandleObject,
                           constructor_native: NonNullJSNative,
                           ctor_nargs: u32,
                           proto: HandleObject,
                           members: &'static NativeProperties,
                           name: *const libc::c_char) {
    unsafe {
        let constructor = RootedObject::new(cx,
                                            create_constructor(cx,
                                                               constructor_native,
                                                               ctor_nargs,
                                                               name));
        assert!(!constructor.ptr.is_null());

        if let Some(static_methods) = members.static_methods {
            define_methods(cx, constructor.handle(), static_methods).unwrap();
        }

        if let Some(static_properties) = members.static_attrs {
            define_properties(cx, constructor.handle(), static_properties).unwrap();
        }

        if let Some(constants) = members.consts {
            define_constants(cx, constructor.handle(), constants);
        }

        if !proto.get().is_null() {
            assert!(JS_LinkConstructorAndPrototype(cx, constructor.handle(), proto));
        }

        define_constructor(cx, receiver, name, constructor.handle());
    }
}

/// Defines constants on `obj`.
/// Fails on JSAPI failure.
fn define_constants(cx: *mut JSContext, obj: HandleObject, constants: &'static [ConstantSpec]) {
    for spec in constants {
        let value = RootedValue::new(cx, spec.get_value());
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

/// Creates the *interface prototype object*.
/// Fails on JSAPI failure.
fn create_interface_prototype_object(cx: *mut JSContext,
                                     global: HandleObject,
                                     proto_class: &'static JSClass,
                                     members: &'static NativeProperties,
                                     rval: MutableHandleObject) {
    unsafe {
        rval.set(JS_NewObjectWithUniqueType(cx, proto_class, global));
        assert!(!rval.get().is_null());

        if let Some(methods) = members.methods {
            define_methods(cx, rval.handle(), methods).unwrap();
        }

        if let Some(properties) = members.attrs {
            define_properties(cx, rval.handle(), properties).unwrap();
        }

        if let Some(constants) = members.consts {
            define_constants(cx, rval.handle(), constants);
        }
    }
}

/// A throwing constructor, for those interfaces that have neither
/// `NoInterfaceObject` nor `Constructor`.
pub unsafe extern "C" fn throwing_constructor(cx: *mut JSContext,
                                              _argc: c_uint,
                                              _vp: *mut JSVal)
                                              -> bool {
    throw_type_error(cx, "Illegal constructor.");
    false
}

/// An array of *mut JSObject of size PrototypeList::ID::Count
pub type ProtoOrIfaceArray = [*mut JSObject; PrototypeList::ID::Count as usize];

/// Construct and cache the ProtoOrIfaceArray for the given global.
/// Fails if the argument is not a DOM global.
pub fn initialize_global(global: *mut JSObject) {
    let proto_array: Box<ProtoOrIfaceArray> =
        box [0 as *mut JSObject; PrototypeList::ID::Count as usize];
    unsafe {
        assert!(((*JS_GetClass(global)).flags & JSCLASS_DOM_GLOBAL) != 0);
        let box_ = Box::into_raw(proto_array);
        JS_SetReservedSlot(global,
                           DOM_PROTOTYPE_SLOT,
                           PrivateValue(box_ as *const libc::c_void));
    }
}

/// Gets the property `id` on  `proxy`'s prototype. If it exists, `*found` is
/// set to true and `*vp` to the value, otherwise `*found` is set to false.
///
/// Returns false on JSAPI failure.
pub fn get_property_on_prototype(cx: *mut JSContext,
                                 proxy: HandleObject,
                                 id: HandleId,
                                 found: *mut bool,
                                 vp: MutableHandleValue)
                                 -> bool {
    unsafe {
        // let proto = GetObjectProto(proxy);
        let mut proto = RootedObject::new(cx, ptr::null_mut());
        if !JS_GetPrototype(cx, proxy, proto.handle_mut()) || proto.ptr.is_null() {
            *found = false;
            return true;
        }
        let mut has_property = false;
        if !JS_HasPropertyById(cx, proto.handle(), id, &mut has_property) {
            return false;
        }
        *found = has_property;
        let no_output = vp.ptr.is_null();
        if !has_property || no_output {
            return true;
        }

        JS_ForwardGetPropertyTo(cx, proto.handle(), id, proxy, vp)
    }
}

/// Get an array index from the given `jsid`. Returns `None` if the given
/// `jsid` is not an integer.
pub fn get_array_index_from_id(_cx: *mut JSContext, id: HandleId) -> Option<u32> {
    unsafe {
        if RUST_JSID_IS_INT(id) {
            return Some(RUST_JSID_TO_INT(id) as u32);
        }
        None
    }
    // if id is length atom, -1, otherwise
    /*return if JSID_IS_ATOM(id) {
        let atom = JSID_TO_ATOM(id);
        //let s = *GetAtomChars(id);
        if s > 'a' && s < 'z' {
            return -1;
        }

        let i = 0;
        let str = AtomToLinearString(JSID_TO_ATOM(id));
        return if StringIsArray(str, &mut i) != 0 { i } else { -1 }
    } else {
        IdToInt32(cx, id);
    }*/
}

/// Find the index of a string given by `v` in `values`.
/// Returns `Err(())` on JSAPI failure (there is a pending exception), and
/// `Ok(None)` if there was no matching string.
pub unsafe fn find_enum_string_index(cx: *mut JSContext,
                                     v: HandleValue,
                                     values: &[&'static str])
                                     -> Result<Option<usize>, ()> {
    let jsstr = ToString(cx, v);
    if jsstr.is_null() {
        return Err(());
    }

    let search = jsstring_to_str(cx, jsstr);
    Ok(values.iter().position(|value| search == *value))
}

/// Returns wether `obj` is a platform object
/// https://heycam.github.io/webidl/#dfn-platform-object
pub fn is_platform_object(obj: *mut JSObject) -> bool {
    unsafe {
        // Fast-path the common case
        let mut clasp = JS_GetClass(obj);
        if is_dom_class(&*clasp) {
            return true;
        }
        // Now for simplicity check for security wrappers before anything else
        if IsWrapper(obj) {
            let unwrapped_obj = UnwrapObject(obj, /* stopAtOuter = */ 0);
            if unwrapped_obj.is_null() {
                return false;
            }
            clasp = js::jsapi::JS_GetClass(obj);
        }
        // TODO also check if JS_IsArrayBufferObject
        is_dom_class(&*clasp)
    }
}

/// Get the property with name `property` from `object`.
/// Returns `Err(())` on JSAPI failure (there is a pending exception), and
/// `Ok(false)` if there was no property with the given name.
pub fn get_dictionary_property(cx: *mut JSContext,
                               object: HandleObject,
                               property: &str,
                               rval: MutableHandleValue)
                               -> Result<bool, ()> {
    fn has_property(cx: *mut JSContext,
                    object: HandleObject,
                    property: &CString,
                    found: &mut bool)
                    -> bool {
        unsafe { JS_HasProperty(cx, object, property.as_ptr(), found) }
    }
    fn get_property(cx: *mut JSContext,
                    object: HandleObject,
                    property: &CString,
                    value: MutableHandleValue)
                    -> bool {
        unsafe { JS_GetProperty(cx, object, property.as_ptr(), value) }
    }

    let property = CString::new(property).unwrap();
    if object.get().is_null() {
        return Ok(false);
    }

    let mut found = false;
    if !has_property(cx, object, &property, &mut found) {
        return Err(());
    }

    if !found {
        return Ok(false);
    }

    if !get_property(cx, object, &property, rval) {
        return Err(());
    }

    Ok(true)
}

/// Set the property with name `property` from `object`.
/// Returns `Err(())` on JSAPI failure, or null object,
/// and Ok(()) otherwise
pub fn set_dictionary_property(cx: *mut JSContext,
                               object: HandleObject,
                               property: &str,
                               value: HandleValue)
                               -> Result<(), ()> {
    if object.get().is_null() {
        return Err(());
    }

    let property = CString::new(property).unwrap();
    unsafe {
        if !JS_SetProperty(cx, object, property.as_ptr(), value) {
            return Err(());
        }
    }

    Ok(())
}

/// Returns whether `proxy` has a property `id` on its prototype.
pub fn has_property_on_prototype(cx: *mut JSContext, proxy: HandleObject, id: HandleId) -> bool {
    // MOZ_ASSERT(js::IsProxy(proxy) && js::GetProxyHandler(proxy) == handler);
    let mut found = false;
    !get_property_on_prototype(cx, proxy, id, &mut found, unsafe {
        MutableHandleValue::from_marked_location(ptr::null_mut())
    }) || found
}

/// Create a DOM global object with the given class.
pub fn create_dom_global(cx: *mut JSContext,
                         class: *const JSClass,
                         private: *const libc::c_void,
                         trace: JSTraceOp)
                         -> *mut JSObject {
    unsafe {
        let mut options = CompartmentOptions::default();
        options.version_ = JSVersion::JSVERSION_ECMA_5;
        options.traceGlobal_ = trace;

        let obj =
            RootedObject::new(cx,
                              JS_NewGlobalObject(cx,
                                                 class,
                                                 ptr::null_mut(),
                                                 OnNewGlobalHookOption::DontFireOnNewGlobalHook,
                                                 &options));
        if obj.ptr.is_null() {
            return ptr::null_mut();
        }
        let _ac = JSAutoCompartment::new(cx, obj.ptr);
        JS_SetReservedSlot(obj.ptr, DOM_OBJECT_SLOT, PrivateValue(private));
        JS_InitStandardClasses(cx, obj.handle());
        initialize_global(obj.ptr);
        JS_FireOnNewGlobalObject(cx, obj.handle());
        obj.ptr
    }
}

/// Drop the resources held by reserved slots of a global object
pub unsafe fn finalize_global(obj: *mut JSObject) {
    let protolist = get_proto_or_iface_array(obj);
    let list = (*protolist).as_mut_ptr();
    for idx in 0..(PrototypeList::ID::Count as isize) {
        let entry = list.offset(idx);
        let value = *entry;
        if <*mut JSObject>::needs_post_barrier(value) {
            <*mut JSObject>::relocate(entry);
        }
    }
    let _: Box<ProtoOrIfaceArray> = Box::from_raw(protolist);
}

/// Trace the resources held by reserved slots of a global object
pub unsafe fn trace_global(tracer: *mut JSTracer, obj: *mut JSObject) {
    let array = get_proto_or_iface_array(obj);
    for proto in (*array).iter() {
        if !proto.is_null() {
            trace_object(tracer,
                         "prototype",
                         &*(proto as *const *mut JSObject as *const Heap<*mut JSObject>));
        }
    }
}

unsafe extern "C" fn wrap(cx: *mut JSContext,
                          _existing: HandleObject,
                          obj: HandleObject)
                          -> *mut JSObject {
    // FIXME terrible idea. need security wrappers
    // https://github.com/servo/servo/issues/2382
    WrapperNew(cx, obj, GetCrossCompartmentWrapper(), ptr::null(), false)
}

unsafe extern "C" fn pre_wrap(cx: *mut JSContext,
                              _existing: HandleObject,
                              obj: HandleObject,
                              _object_passed_to_wrap: HandleObject)
                              -> *mut JSObject {
    let _ac = JSAutoCompartment::new(cx, obj.get());
    JS_ObjectToOuterObject(cx, obj)
}

/// Callback table for use with JS_SetWrapObjectCallbacks
pub static WRAP_CALLBACKS: JSWrapObjectCallbacks = JSWrapObjectCallbacks {
    wrap: Some(wrap),
    preWrap: Some(pre_wrap),
};

/// Callback to outerize windows.
pub unsafe extern "C" fn outerize_global(_cx: *mut JSContext, obj: HandleObject) -> *mut JSObject {
    debug!("outerizing");
    let win = root_from_handleobject::<window::Window>(obj).unwrap();
    let context = win.browsing_context();
    context.as_ref().unwrap().window_proxy()
}

/// Deletes the property `id` from `object`.
pub unsafe fn delete_property_by_id(cx: *mut JSContext,
                                    object: HandleObject,
                                    id: HandleId,
                                    bp: *mut ObjectOpResult)
                                    -> bool {
    JS_DeletePropertyById1(cx, object, id, bp)
}

unsafe fn generic_call(cx: *mut JSContext,
                       argc: libc::c_uint,
                       vp: *mut JSVal,
                       is_lenient: bool,
                       call: unsafe extern fn(*const JSJitInfo, *mut JSContext,
                                              HandleObject, *mut libc::c_void, u32,
                                              *mut JSVal)
                                              -> bool)
                       -> bool {
    let args = CallArgs::from_vp(vp, argc);
    let thisobj = args.thisv();
    if !thisobj.get().is_null_or_undefined() && !thisobj.get().is_object() {
        return false;
    }
    let obj = if thisobj.get().is_object() {
        thisobj.get().to_object()
    } else {
        GetGlobalForObjectCrossCompartment(JS_CALLEE(cx, vp).to_object_or_null())
    };
    let obj = RootedObject::new(cx, obj);
    let info = RUST_FUNCTION_VALUE_TO_JITINFO(JS_CALLEE(cx, vp));
    let proto_id = (*info).protoID;
    let depth = (*info).depth;
    let proto_check = |class: &'static DOMClass| {
        class.interface_chain[depth as usize] as u16 == proto_id
    };
    let this = match private_from_proto_check(obj.ptr, proto_check) {
        Ok(val) => val,
        Err(()) => {
            if is_lenient {
                debug_assert!(!JS_IsExceptionPending(cx));
                *vp = UndefinedValue();
                return true;
            } else {
                throw_invalid_this(cx, proto_id);
                return false;
            }
        }
    };
    call(info, cx, obj.handle(), this as *mut libc::c_void, argc, vp)
}

/// Generic method of IDL interface.
pub unsafe extern "C" fn generic_method(cx: *mut JSContext,
                                        argc: libc::c_uint,
                                        vp: *mut JSVal)
                                        -> bool {
    generic_call(cx, argc, vp, false, CallJitMethodOp)
}

/// Generic getter of IDL interface.
pub unsafe extern "C" fn generic_getter(cx: *mut JSContext,
                                        argc: libc::c_uint,
                                        vp: *mut JSVal)
                                        -> bool {
    generic_call(cx, argc, vp, false, CallJitGetterOp)
}

/// Generic lenient getter of IDL interface.
pub unsafe extern "C" fn generic_lenient_getter(cx: *mut JSContext,
                                                argc: libc::c_uint,
                                                vp: *mut JSVal)
                                                -> bool {
    generic_call(cx, argc, vp, true, CallJitGetterOp)
}

unsafe extern "C" fn call_setter(info: *const JSJitInfo,
                                 cx: *mut JSContext,
                                 handle: HandleObject,
                                 this: *mut libc::c_void,
                                 argc: u32,
                                 vp: *mut JSVal)
                                 -> bool {
    if !CallJitSetterOp(info, cx, handle, this, argc, vp) {
        return false;
    }
    *vp = UndefinedValue();
    true
}

/// Generic setter of IDL interface.
pub unsafe extern "C" fn generic_setter(cx: *mut JSContext,
                                        argc: libc::c_uint,
                                        vp: *mut JSVal)
                                        -> bool {
    generic_call(cx, argc, vp, false, call_setter)
}

/// Generic lenient setter of IDL interface.
pub unsafe extern "C" fn generic_lenient_setter(cx: *mut JSContext,
                                                argc: libc::c_uint,
                                                vp: *mut JSVal)
                                                -> bool {
    generic_call(cx, argc, vp, true, call_setter)
}

unsafe extern "C" fn instance_class_has_proto_at_depth(clasp: *const js::jsapi::Class,
                                                       proto_id: u32,
                                                       depth: u32)
                                                       -> bool {
    let domclass: *const DOMJSClass = clasp as *const _;
    let domclass = &*domclass;
    domclass.dom_class.interface_chain[depth as usize] as u32 == proto_id
}

#[allow(missing_docs)]  // FIXME
pub const DOM_CALLBACKS: DOMCallbacks = DOMCallbacks {
    instanceClassMatchesProto: Some(instance_class_has_proto_at_depth),
};
