/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Various utilities to glue JavaScript and the DOM implementation together.

use dom::bindings::codegen::PrototypeList;
use dom::bindings::codegen::PrototypeList::MAX_PROTO_CHAIN_LENGTH;
use dom::bindings::conversions::native_from_handleobject;
use dom::bindings::conversions::private_from_proto_chain;
use dom::bindings::conversions::{is_dom_class, jsstring_to_str};
use dom::bindings::error::throw_type_error;
use dom::bindings::error::{Error, ErrorResult, Fallible, throw_invalid_this};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::trace::trace_object;
use dom::browsercontext;
use dom::window;
use util::mem::HeapSizeOf;
use util::str::DOMString;

use js;
use js::glue::{CallJitMethodOp, CallJitGetterOp, CallJitSetterOp, IsWrapper};
use js::glue::{RUST_FUNCTION_VALUE_TO_JITINFO, RUST_JSID_IS_INT};
use js::glue::{RUST_JSID_TO_INT, UnwrapObject};
use js::glue::{WrapperNew, GetCrossCompartmentWrapper};
use js::jsapi::JSAutoCompartment;
use js::jsapi::JS_DeletePropertyById1;
use js::jsapi::JS_GetFunctionObject;
use js::jsapi::JS_IsExceptionPending;
use js::jsapi::JS_NewObjectWithUniqueType;
use js::jsapi::JS_ObjectToOuterObject;
use js::jsapi::PropertyDefinitionBehavior;
use js::jsapi::{CallArgs, GetGlobalForObjectCrossCompartment, JSJitInfo};
use js::jsapi::{DOMCallbacks, JSWrapObjectCallbacks};
use js::jsapi::{HandleObject, HandleId, HandleValue, MutableHandleValue};
use js::jsapi::{JSContext, JSObject, JSClass, JSTracer};
use js::jsapi::{JSFunctionSpec, JSPropertySpec};
use js::jsapi::{JS_AlreadyHasOwnProperty, JS_NewFunction, JSTraceOp};
use js::jsapi::{JS_DefineFunctions, JS_DefineProperty, JS_DefineProperty1};
use js::jsapi::{JS_DefineProperties, JS_ForwardGetPropertyTo};
use js::jsapi::{JS_FireOnNewGlobalObject, JSVersion};
use js::jsapi::{JS_GetClass, JS_LinkConstructorAndPrototype};
use js::jsapi::{JS_GetProperty, JS_HasProperty, JS_SetProperty};
use js::jsapi::{JS_GetReservedSlot, JS_SetReservedSlot};
use js::jsapi::{JS_HasPropertyById, JS_GetPrototype};
use js::jsapi::{JS_NewGlobalObject, JS_InitStandardClasses};
use js::jsapi::{ObjectOpResult, RootedObject, RootedValue, Heap, MutableHandleObject};
use js::jsapi::{OnNewGlobalHookOption, CompartmentOptions};
use js::jsval::{BooleanValue, DoubleValue, Int32Value, JSVal, NullValue};
use js::jsval::{PrivateValue, UInt32Value, UndefinedValue};
use js::rust::{GCMethods, ToString};
use js::{JSPROP_PERMANENT, JSPROP_READONLY};
use js::{JS_CALLEE, JSFUN_CONSTRUCTOR, JSPROP_ENUMERATE};
use libc;
use libc::c_uint;
use std::cell::UnsafeCell;
use std::cmp::PartialEq;
use std::default::Default;
use std::ffi::CString;
use std::ptr;
use string_cache::{Atom, Namespace};

/// Proxy handler for a WindowProxy.
#[allow(raw_pointer_derive)]
pub struct WindowProxyHandler(pub *const libc::c_void);

impl HeapSizeOf for WindowProxyHandler {
    fn heap_size_of_children(&self) -> usize {
        //FIXME(#6907) this is a pointer to memory allocated by `new` in NewProxyHandler in rust-mozjs.
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
    pub value: ConstantVal
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
#[derive(Copy, Clone)]
pub struct DOMClass {
    /// A list of interfaces that this object implements, in order of decreasing
    /// derivedness.
    pub interface_chain: [PrototypeList::ID; MAX_PROTO_CHAIN_LENGTH],

    /// The NativePropertyHooks for the interface associated with this class.
    pub native_hooks: &'static NativePropertyHooks,
}
unsafe impl Sync for DOMClass {}

/// The JSClass used for DOM object reflectors.
#[derive(Copy)]
pub struct DOMJSClass {
    /// The actual JSClass.
    pub base: js::jsapi::Class,
    /// Associated data for DOM object reflectors.
    pub dom_class: DOMClass
}
impl Clone for DOMJSClass {
    fn clone(&self) -> DOMJSClass { *self }
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
    unsafe extern "C" fn (arg1: *mut JSContext, arg2: c_uint, arg3: *mut JSVal) -> u8;

/// Creates the *interface prototype object* (if a `proto_class` is given)
/// and the *interface object* (if a `constructor` is given).
/// Fails on JSAPI failure.
pub fn do_create_interface_objects(cx: *mut JSContext,
                                   receiver: HandleObject,
                                   proto_proto: HandleObject,
                                   proto_class: Option<&'static JSClass>,
                                   constructor: Option<(NonNullJSNative, &'static str, u32)>,
                                   named_constructors: &[(NonNullJSNative, &'static str, u32)],
                                   dom_class: *const DOMClass,
                                   members: &'static NativeProperties,
                                   rval: MutableHandleObject) {
    if let Some(proto_class) = proto_class {
        create_interface_prototype_object(cx, proto_proto,
                                          proto_class, members, rval);
    }

    unsafe {
        if !rval.get().is_null() {
            JS_SetReservedSlot(rval.get(), DOM_PROTO_INSTANCE_CLASS_SLOT,
                               PrivateValue(dom_class as *const libc::c_void));
        }
    }

    if let Some((native, name, nargs)) = constructor {
        let s = CString::new(name).unwrap();
        create_interface_object(cx, receiver,
                                native, nargs, rval.handle(),
                                members, s.as_ptr())
    }

    for ctor in named_constructors {
        let (cnative, cname, cnargs) = *ctor;

        let cs = CString::new(cname).unwrap();
        let constructor = RootedObject::new(cx, create_constructor(cx, cnative, cnargs, cs.as_ptr()));
        assert!(!constructor.ptr.is_null());
        unsafe {
            assert!(JS_DefineProperty1(cx, constructor.handle(), b"prototype\0".as_ptr() as *const libc::c_char,
                                       rval.handle(),
                                       JSPROP_PERMANENT | JSPROP_READONLY,
                                       None, None) != 0);
        }
        define_constructor(cx, receiver, cs.as_ptr(), constructor.handle());
    }

}

fn create_constructor(cx: *mut JSContext,
                      constructor_native: NonNullJSNative,
                      ctor_nargs: u32,
                      name: *const libc::c_char) -> *mut JSObject {
    unsafe {
        let fun = JS_NewFunction(cx, Some(constructor_native), ctor_nargs,
                                 JSFUN_CONSTRUCTOR, name);
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
        let mut already_defined = 0;
        assert!(JS_AlreadyHasOwnProperty(cx, receiver, name, &mut already_defined) != 0);

        if already_defined == 0 {
            assert!(JS_DefineProperty1(cx, receiver, name,
                                      constructor,
                                      0, None, None) != 0);
        }

    }
}

/// Creates the *interface object*.
/// Fails on JSAPI failure.
fn create_interface_object(cx: *mut JSContext,
                           receiver: HandleObject,
                           constructor_native: NonNullJSNative,
                           ctor_nargs: u32, proto: HandleObject,
                           members: &'static NativeProperties,
                           name: *const libc::c_char) {
    let constructor = RootedObject::new(cx, create_constructor(cx, constructor_native, ctor_nargs, name));
    assert!(!constructor.ptr.is_null());

    if let Some(static_methods) = members.static_methods {
        define_methods(cx, constructor.handle(), static_methods);
    }

    if let Some(static_properties) = members.static_attrs {
        define_properties(cx, constructor.handle(), static_properties);
    }

    if let Some(constants) = members.consts {
        define_constants(cx, constructor.handle(), constants);
    }

    unsafe {
        if !proto.get().is_null() {
            assert!(JS_LinkConstructorAndPrototype(cx, constructor.handle(), proto) != 0);
        }
    }

    define_constructor(cx, receiver, name, constructor.handle());
}

/// Defines constants on `obj`.
/// Fails on JSAPI failure.
fn define_constants(cx: *mut JSContext, obj: HandleObject,
                    constants: &'static [ConstantSpec]) {
    for spec in constants {
        let value = RootedValue::new(cx, spec.get_value());
        unsafe {
            assert!(JS_DefineProperty(cx, obj, spec.name.as_ptr() as *const libc::c_char,
                                      value.handle(),
                                      JSPROP_ENUMERATE | JSPROP_READONLY |
                                      JSPROP_PERMANENT, None, None) != 0);
        }
    }
}

/// Defines methods on `obj`. The last entry of `methods` must contain zeroed
/// memory.
/// Fails on JSAPI failure.
fn define_methods(cx: *mut JSContext, obj: HandleObject,
                  methods: &'static [JSFunctionSpec]) {
    unsafe {
        assert!(JS_DefineFunctions(cx, obj, methods.as_ptr(), PropertyDefinitionBehavior::DefineAllProperties) != 0);
    }
}

/// Defines attributes on `obj`. The last entry of `properties` must contain
/// zeroed memory.
/// Fails on JSAPI failure.
fn define_properties(cx: *mut JSContext, obj: HandleObject,
                     properties: &'static [JSPropertySpec]) {
    unsafe {
        assert!(JS_DefineProperties(cx, obj, properties.as_ptr()) != 0);
    }
}

/// Creates the *interface prototype object*.
/// Fails on JSAPI failure.
fn create_interface_prototype_object(cx: *mut JSContext, global: HandleObject,
                                     proto_class: &'static JSClass,
                                     members: &'static NativeProperties,
                                     rval: MutableHandleObject) {
    unsafe {
        rval.set(JS_NewObjectWithUniqueType(cx, proto_class, global));
        assert!(!rval.get().is_null());

        if let Some(methods) = members.methods {
            define_methods(cx, rval.handle(), methods);
        }

        if let Some(properties) = members.attrs {
            define_properties(cx, rval.handle(), properties);
        }

        if let Some(constants) = members.consts {
            define_constants(cx, rval.handle(), constants);
        }
    }
}

/// A throwing constructor, for those interfaces that have neither
/// `NoInterfaceObject` nor `Constructor`.
pub unsafe extern fn throwing_constructor(cx: *mut JSContext, _argc: c_uint,
                                          _vp: *mut JSVal) -> u8 {
    throw_type_error(cx, "Illegal constructor.");
    0
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

/// A trait to provide access to the `Reflector` for a DOM object.
pub trait Reflectable {
    /// Returns the receiver's reflector.
    fn reflector(&self) -> &Reflector;
    /// Initializes the Reflector
    fn init_reflector(&mut self, _obj: *mut JSObject) {
        panic!("Cannot call init on this Reflectable");
    }
}

/// Create the reflector for a new DOM object and yield ownership to the
/// reflector.
pub fn reflect_dom_object<T: Reflectable>
        (obj:     Box<T>,
         global:  GlobalRef,
         wrap_fn: extern "Rust" fn(*mut JSContext, GlobalRef, Box<T>) -> Root<T>)
         -> Root<T> {
    wrap_fn(global.get_cx(), global, obj)
}

/// A struct to store a reference to the reflector of a DOM object.
#[allow(raw_pointer_derive, unrooted_must_root)]
#[must_root]
#[servo_lang = "reflector"]
#[derive(HeapSizeOf)]
// If you're renaming or moving this field, update the path in plugins::reflector as well
pub struct Reflector {
    #[ignore_heap_size_of = "defined and measured in rust-mozjs"]
    object: UnsafeCell<*mut JSObject>,
}

#[allow(unrooted_must_root)]
impl PartialEq for Reflector {
    fn eq(&self, other: &Reflector) -> bool {
        unsafe { *self.object.get() == *other.object.get() }
    }
}

impl Reflector {
    /// Get the reflector.
    #[inline]
    pub fn get_jsobject(&self) -> HandleObject {
        HandleObject { ptr: self.object.get() }
    }

    /// Initialize the reflector. (May be called only once.)
    pub fn set_jsobject(&mut self, object: *mut JSObject) {
        unsafe {
            let obj = self.object.get();
            assert!((*obj).is_null());
            assert!(!object.is_null());
            *obj = object;
        }
    }

    /// Return a pointer to the memory location at which the JS reflector
    /// object is stored. Used to root the reflector, as
    /// required by the JSAPI rooting APIs.
    pub fn rootable(&self) -> *mut *mut JSObject {
        self.object.get()
    }

    /// Create an uninitialized `Reflector`.
    pub fn new() -> Reflector {
        Reflector {
            object: UnsafeCell::new(ptr::null_mut())
        }
    }
}

/// Gets the property `id` on  `proxy`'s prototype. If it exists, `*found` is
/// set to true and `*vp` to the value, otherwise `*found` is set to false.
///
/// Returns false on JSAPI failure.
pub fn get_property_on_prototype(cx: *mut JSContext, proxy: HandleObject,
                                 id: HandleId, found: *mut bool, vp: MutableHandleValue)
                                 -> bool {
    unsafe {
      //let proto = GetObjectProto(proxy);
      let mut proto = RootedObject::new(cx, ptr::null_mut());
      if JS_GetPrototype(cx, proxy, proto.handle_mut()) == 0 ||
         proto.ptr.is_null() {
          *found = false;
          return true;
      }
      let mut has_property = 0;
      if JS_HasPropertyById(cx, proto.handle(), id, &mut has_property) == 0 {
          return false;
      }
      *found = has_property != 0;
      let no_output = vp.ptr.is_null();
      if has_property == 0 || no_output {
          return true;
      }

      JS_ForwardGetPropertyTo(cx, proto.handle(), id, proxy, vp) != 0
  }
}

/// Get an array index from the given `jsid`. Returns `None` if the given
/// `jsid` is not an integer.
pub fn get_array_index_from_id(_cx: *mut JSContext, id: HandleId) -> Option<u32> {
    unsafe {
        if RUST_JSID_IS_INT(id) != 0 {
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
pub fn find_enum_string_index(cx: *mut JSContext,
                              v: HandleValue,
                              values: &[&'static str])
                              -> Result<Option<usize>, ()> {
    let jsstr = ToString(cx, v);
    if jsstr.is_null() {
        return Err(());
    }

    let search = jsstring_to_str(cx, jsstr);
    Ok(values.iter().position(|value| value == &search))
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
        if IsWrapper(obj) == 1 {
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
    fn has_property(cx: *mut JSContext, object: HandleObject, property: &CString,
                    found: &mut u8) -> bool {
        unsafe {
            JS_HasProperty(cx, object, property.as_ptr(), found) != 0
        }
    }
    fn get_property(cx: *mut JSContext, object: HandleObject, property: &CString,
                    value: MutableHandleValue) -> bool {
        unsafe {
            JS_GetProperty(cx, object, property.as_ptr(), value) != 0
        }
    }

    let property = CString::new(property).unwrap();
    if object.get().is_null() {
        return Ok(false);
    }

    let mut found: u8 = 0;
    if !has_property(cx, object, &property, &mut found) {
        return Err(());
    }

    if found == 0 {
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
                               value: HandleValue) -> Result<(), ()> {
    if object.get().is_null() {
        return Err(());
    }

    let property = CString::new(property).unwrap();
    unsafe {
        if JS_SetProperty(cx, object, property.as_ptr(), value) == 0 {
            return Err(());
        }
    }

    Ok(())
}

/// Returns whether `proxy` has a property `id` on its prototype.
pub fn has_property_on_prototype(cx: *mut JSContext, proxy: HandleObject,
                                 id: HandleId) -> bool {
    //  MOZ_ASSERT(js::IsProxy(proxy) && js::GetProxyHandler(proxy) == handler);
    let mut found = false;
    !get_property_on_prototype(cx, proxy, id, &mut found,
                               MutableHandleValue { ptr: ptr::null_mut() }) || found
}

/// Create a DOM global object with the given class.
pub fn create_dom_global(cx: *mut JSContext, class: *const JSClass,
                         trace: JSTraceOp)
                         -> *mut JSObject {
    unsafe {
        let mut options = CompartmentOptions::default();
        options.version_ = JSVersion::JSVERSION_LATEST;
        options.traceGlobal_ = trace;

        let obj =
            RootedObject::new(cx,
                              JS_NewGlobalObject(cx, class, ptr::null_mut(),
                                                 OnNewGlobalHookOption::DontFireOnNewGlobalHook, &options));
        if obj.ptr.is_null() {
            return ptr::null_mut();
        }
        let _ac = JSAutoCompartment::new(cx, obj.ptr);
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
    let _: Box<ProtoOrIfaceArray> =
        Box::from_raw(protolist);
}

/// Trace the resources held by reserved slots of a global object
pub unsafe fn trace_global(tracer: *mut JSTracer, obj: *mut JSObject) {
    let array = get_proto_or_iface_array(obj);
    for proto in (*array).iter() {
        if !proto.is_null() {
            trace_object(tracer, "prototype", &*(proto as *const *mut JSObject as *const Heap<*mut JSObject>));
        }
    }
}

unsafe extern fn wrap(cx: *mut JSContext,
                      _existing: HandleObject,
                      obj: HandleObject)
                      -> *mut JSObject {
    // FIXME terrible idea. need security wrappers
    // https://github.com/servo/servo/issues/2382
    WrapperNew(cx, obj, GetCrossCompartmentWrapper(), ptr::null(), false)
}

unsafe extern fn pre_wrap(cx: *mut JSContext, _existing: HandleObject,
                          obj: HandleObject, _object_passed_to_wrap: HandleObject)
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
pub unsafe extern fn outerize_global(_cx: *mut JSContext, obj: HandleObject) -> *mut JSObject {
    debug!("outerizing");
    let win: Root<window::Window> = native_from_handleobject(obj).unwrap();
    // FIXME(https://github.com/rust-lang/rust/issues/23338)
    let win = win.r();
    let context = win.browsing_context();
    context.as_ref().unwrap().window_proxy()
}

/// Deletes the property `id` from `object`.
pub unsafe fn delete_property_by_id(cx: *mut JSContext, object: HandleObject,
                                    id: HandleId, bp: *mut ObjectOpResult) -> u8 {
    JS_DeletePropertyById1(cx, object, id, bp)
}

unsafe fn generic_call(cx: *mut JSContext, argc: libc::c_uint, vp: *mut JSVal,
                       is_lenient: bool,
                       call: unsafe extern fn(*const JSJitInfo, *mut JSContext,
                                              HandleObject, *mut libc::c_void, u32,
                                              *mut JSVal)
                                              -> u8)
                       -> u8 {
    let args = CallArgs::from_vp(vp, argc);
    let thisobj = args.thisv();
    if !thisobj.get().is_null_or_undefined() && !thisobj.get().is_object() {
        return 0;
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
    let this = match private_from_proto_chain(obj.ptr, proto_id, depth) {
        Ok(val) => val,
        Err(()) => {
            if is_lenient {
                debug_assert!(JS_IsExceptionPending(cx) == 0);
                *vp = UndefinedValue();
                return 1;
            } else {
                throw_invalid_this(cx, proto_id);
                return 0;
            }
        }
    };
    call(info, cx, obj.handle(), this as *mut libc::c_void, argc, vp)
}

/// Generic method of IDL interface.
pub unsafe extern fn generic_method(cx: *mut JSContext,
                                    argc: libc::c_uint, vp: *mut JSVal)
                                    -> u8 {
    generic_call(cx, argc, vp, false, CallJitMethodOp)
}

/// Generic getter of IDL interface.
pub unsafe extern fn generic_getter(cx: *mut JSContext,
                                    argc: libc::c_uint, vp: *mut JSVal)
                                    -> u8 {
    generic_call(cx, argc, vp, false, CallJitGetterOp)
}

/// Generic lenient getter of IDL interface.
pub unsafe extern fn generic_lenient_getter(cx: *mut JSContext,
                                            argc: libc::c_uint,
                                            vp: *mut JSVal)
                                            -> u8 {
    generic_call(cx, argc, vp, true, CallJitGetterOp)
}

unsafe extern fn call_setter(info: *const JSJitInfo, cx: *mut JSContext,
                             handle: HandleObject, this: *mut libc::c_void,
                             argc: u32, vp: *mut JSVal)
                             -> u8 {
    if CallJitSetterOp(info, cx, handle, this, argc, vp) == 0 {
        return 0;
    }
    *vp = UndefinedValue();
    1
}

/// Generic setter of IDL interface.
pub unsafe extern fn generic_setter(cx: *mut JSContext,
                                    argc: libc::c_uint, vp: *mut JSVal)
                                    -> u8 {
    generic_call(cx, argc, vp, false, call_setter)
}

/// Generic lenient setter of IDL interface.
pub unsafe extern fn generic_lenient_setter(cx: *mut JSContext,
                                            argc: libc::c_uint,
                                            vp: *mut JSVal)
                                            -> u8 {
    generic_call(cx, argc, vp, true, call_setter)
}

/// Validate a qualified name. See https://dom.spec.whatwg.org/#validate for details.
pub fn validate_qualified_name(qualified_name: &str) -> ErrorResult {
    match xml_name_type(qualified_name) {
        XMLName::InvalidXMLName => {
            // Step 1.
            Err(Error::InvalidCharacter)
        },
        XMLName::Name => {
            // Step 2.
            Err(Error::Namespace)
        },
        XMLName::QName => Ok(())
    }
}

unsafe extern "C" fn instance_class_has_proto_at_depth(clasp: *const js::jsapi::Class,
                                                       proto_id: u32,
                                                       depth: u32) -> u8 {
    let domclass: *const DOMJSClass = clasp as *const _;
    let domclass = &*domclass;
    (domclass.dom_class.interface_chain[depth as usize] as u32 == proto_id) as u8
}

#[allow(missing_docs)]  // FIXME
pub const DOM_CALLBACKS: DOMCallbacks = DOMCallbacks {
    instanceClassMatchesProto: Some(instance_class_has_proto_at_depth),
};

/// Validate a namespace and qualified name and extract their parts.
/// See https://dom.spec.whatwg.org/#validate-and-extract for details.
pub fn validate_and_extract(namespace: Option<DOMString>, qualified_name: &str)
                            -> Fallible<(Namespace, Option<Atom>, Atom)> {
    // Step 1.
    let namespace = namespace_from_domstring(namespace);

    // Step 2.
    try!(validate_qualified_name(qualified_name));

    let colon = ':';

    // Step 5.
    let mut parts = qualified_name.splitn(2, colon);

    let (maybe_prefix, local_name) = {
        let maybe_prefix = parts.next();
        let maybe_local_name = parts.next();

        debug_assert!(parts.next().is_none());

        if let Some(local_name) = maybe_local_name {
            debug_assert!(!maybe_prefix.unwrap().is_empty());

            (maybe_prefix, local_name)
        } else {
            (None, maybe_prefix.unwrap())
        }
    };

    debug_assert!(!local_name.contains(colon));

    match (namespace, maybe_prefix) {
        (ns!(""), Some(_)) => {
            // Step 6.
            Err(Error::Namespace)
        },
        (ref ns, Some("xml")) if ns != &ns!(XML) => {
            // Step 7.
            Err(Error::Namespace)
        },
        (ref ns, p) if ns != &ns!(XMLNS) &&
                      (qualified_name == "xmlns" || p == Some("xmlns")) => {
            // Step 8.
            Err(Error::Namespace)
        },
        (ns!(XMLNS), p) if qualified_name != "xmlns" && p != Some("xmlns") => {
            // Step 9.
            Err(Error::Namespace)
        },
        (ns, p) => {
            // Step 10.
            Ok((ns, p.map(Atom::from_slice), Atom::from_slice(local_name)))
        }
    }
}

/// Results of `xml_name_type`.
#[derive(PartialEq)]
#[allow(missing_docs)]
pub enum XMLName {
    QName,
    Name,
    InvalidXMLName
}

/// Check if an element name is valid. See http://www.w3.org/TR/xml/#NT-Name
/// for details.
pub fn xml_name_type(name: &str) -> XMLName {
    fn is_valid_start(c: char) -> bool {
        match c {
            ':' |
            'A' ... 'Z' |
            '_' |
            'a' ... 'z' |
            '\u{C0}' ... '\u{D6}' |
            '\u{D8}' ... '\u{F6}' |
            '\u{F8}' ... '\u{2FF}' |
            '\u{370}' ... '\u{37D}' |
            '\u{37F}' ... '\u{1FFF}' |
            '\u{200C}' ... '\u{200D}' |
            '\u{2070}' ... '\u{218F}' |
            '\u{2C00}' ... '\u{2FEF}' |
            '\u{3001}' ... '\u{D7FF}' |
            '\u{F900}' ... '\u{FDCF}' |
            '\u{FDF0}' ... '\u{FFFD}' |
            '\u{10000}' ... '\u{EFFFF}' => true,
            _ => false,
        }
    }

    fn is_valid_continuation(c: char) -> bool {
        is_valid_start(c) || match c {
            '-' |
            '.' |
            '0' ... '9' |
            '\u{B7}' |
            '\u{300}' ... '\u{36F}' |
            '\u{203F}' ... '\u{2040}' => true,
            _ => false,
        }
    }

    let mut iter = name.chars();
    let mut non_qname_colons = false;
    let mut seen_colon = false;
    let mut last = match iter.next() {
        None => return XMLName::InvalidXMLName,
        Some(c) => {
            if !is_valid_start(c) {
                return XMLName::InvalidXMLName;
            }
            if c == ':' {
                non_qname_colons = true;
            }
            c
        }
    };

    for c in iter {
        if !is_valid_continuation(c) {
            return XMLName::InvalidXMLName;
        }
        if c == ':' {
            match seen_colon {
                true => non_qname_colons = true,
                false => seen_colon = true
            }
        }
        last = c
    }

    if last == ':' {
        non_qname_colons = true
    }

    match non_qname_colons {
        false => XMLName::QName,
        true => XMLName::Name
    }
}

/// Convert a possibly-null URL to a namespace.
///
/// If the URL is None, returns the empty namespace.
pub fn namespace_from_domstring(url: Option<DOMString>) -> Namespace {
    match url {
        None => ns!(""),
        Some(ref s) => Namespace(Atom::from_slice(s)),
    }
}
