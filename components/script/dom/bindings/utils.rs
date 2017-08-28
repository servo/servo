/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Various utilities to glue JavaScript and the DOM implementation together.

use dom::bindings::codegen::InterfaceObjectMap;
use dom::bindings::codegen::PrototypeList;
use dom::bindings::codegen::PrototypeList::{MAX_PROTO_CHAIN_LENGTH, PROTO_OR_IFACE_LENGTH};
use dom::bindings::conversions::{jsstring_to_str, private_from_proto_check};
use dom::bindings::error::throw_invalid_this;
use dom::bindings::inheritance::TopTypeId;
use dom::bindings::str::DOMString;
use dom::bindings::trace::trace_object;
use dom::windowproxy;
use heapsize::HeapSizeOf;
use js;
use js::glue::{CallJitGetterOp, CallJitMethodOp, CallJitSetterOp, IsWrapper};
use js::glue::{GetCrossCompartmentWrapper, WrapperNew};
use js::glue::{RUST_FUNCTION_VALUE_TO_JITINFO, RUST_JSID_IS_INT, RUST_JSID_IS_STRING};
use js::glue::{RUST_JSID_TO_INT, RUST_JSID_TO_STRING, UnwrapObject};
use js::heap::Heap;
use js::jsapi;
use js::jsval::UndefinedValue;
use js::rust::{GCMethods, ToString, get_object_class, is_dom_class, is_global};
use libc;
use std::ffi::CString;
use std::os::raw::{c_char, c_void};
use std::ptr;
use std::slice;

/// Proxy handler for a WindowProxy.
pub struct WindowProxyHandler(pub *const libc::c_void);

impl HeapSizeOf for WindowProxyHandler {
    fn heap_size_of_children(&self) -> usize {
        // FIXME(#6907) this is a pointer to memory allocated by `new` in NewProxyHandler in rust-mozjs.
        0
    }
}

#[derive(HeapSizeOf, JSTraceable)]
/// Static data associated with a global object.
pub struct GlobalStaticData {
    /// The WindowProxy proxy handler for this global.
    pub windowproxy_handler: WindowProxyHandler,
}

impl GlobalStaticData {
    /// Creates a new GlobalStaticData.
    pub fn new() -> GlobalStaticData {
        GlobalStaticData {
            windowproxy_handler: windowproxy::new_window_proxy_handler(),
        }
    }
}

/// The index of the slot where the object holder of that interface's
/// unforgeable members are defined.
pub const DOM_PROTO_UNFORGEABLE_HOLDER_SLOT: u32 = 0;

/// The index of the slot that contains a reference to the ProtoOrIfaceArray.
// All DOM globals must have a slot at DOM_PROTOTYPE_SLOT.
pub const DOM_PROTOTYPE_SLOT: u32 = js::jsapi::JSCLASS_GLOBAL_SLOT_COUNT;

/// The flag set on the `JSClass`es for DOM global objects.
// NOTE: This is baked into the Ion JIT as 0 in codegen for LGetDOMProperty and
// LSetDOMProperty. Those constants need to be changed accordingly if this value
// changes.
pub const JSCLASS_DOM_GLOBAL: u32 = js::jsapi::JSCLASS_USERBIT1;


/// The struct that holds inheritance information for DOM object reflectors.
#[derive(Clone, Copy, Debug)]
pub struct DOMClass {
    /// A list of interfaces that this object implements, in order of decreasing
    /// derivedness.
    pub interface_chain: [PrototypeList::ID; MAX_PROTO_CHAIN_LENGTH],

    /// The type ID of that interface.
    pub type_id: TopTypeId,

    /// The HeapSizeOf function wrapper for that interface.
    pub heap_size_of: unsafe fn(*const c_void) -> usize,

    /// The `Globals` flag for this global interface, if any.
    pub global: InterfaceObjectMap::Globals,
}
unsafe impl Sync for DOMClass {}

/// The JSClass used for DOM object reflectors.
#[derive(Copy)]
pub struct DOMJSClass {
    /// The actual JSClass.
    pub base: js::jsapi::JSClass,
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
pub fn get_proto_or_iface_array(global: *mut jsapi::JSObject) -> *mut ProtoOrIfaceArray {
    unsafe {
        assert!(((*get_object_class(global)).flags & JSCLASS_DOM_GLOBAL) != 0);
        jsapi::JS_GetReservedSlot(global, DOM_PROTOTYPE_SLOT).to_private() as *mut ProtoOrIfaceArray
    }
}

/// An array of *mut jsapi::JSObject of size PROTO_OR_IFACE_LENGTH.
pub type ProtoOrIfaceArray = [*mut jsapi::JSObject; PROTO_OR_IFACE_LENGTH];

/// Gets the property `id` on  `proxy`'s prototype. If it exists, `*found` is
/// set to true and `*vp` to the value, otherwise `*found` is set to false.
///
/// Returns false on JSAPI failure.
pub unsafe fn get_property_on_prototype(cx: *mut jsapi::JSContext,
                                        proxy: jsapi::JS::HandleObject,
                                        receiver: jsapi::JS::HandleValue,
                                        id: jsapi::JS::HandleId,
                                        found: *mut bool,
                                        vp: jsapi::JS::MutableHandleValue)
                                        -> bool {
    rooted!(in(cx) let mut proto = ptr::null_mut());
    if !jsapi::JS_GetPrototype(cx, proxy, proto.handle_mut()) || proto.is_null() {
        *found = false;
        return true;
    }
    let mut has_property = false;
    if !jsapi::JS_HasPropertyById(cx, proto.handle(), id, &mut has_property) {
        return false;
    }
    *found = has_property;
    let no_output = vp.ptr.is_null();
    if !has_property || no_output {
        return true;
    }

    jsapi::JS_ForwardGetPropertyTo(cx, proto.handle(), id, receiver, vp)
}

/// Get an array index from the given `jsid`. Returns `None` if the given
/// `jsid` is not an integer.
pub fn get_array_index_from_id(_cx: *mut jsapi::JSContext, id: jsapi::JS::HandleId) -> Option<u32> {
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

/// Find the enum equivelent of a string given by `v` in `pairs`.
/// Returns `Err(())` on JSAPI failure (there is a pending exception), and
/// `Ok((None, value))` if there was no matching string.
pub unsafe fn find_enum_value<'a, T>(cx: *mut jsapi::JSContext,
                                     v: jsapi::JS::HandleValue,
                                     pairs: &'a [(&'static str, T)])
                                     -> Result<(Option<&'a T>, DOMString), ()> {
    let jsstr = ToString(cx, v);
    if jsstr.is_null() {
        return Err(());
    }

    let search = jsstring_to_str(cx, jsstr);
    Ok((pairs.iter().find(|&&(key, _)| search == *key).map(|&(_, ref ev)| ev), search))
}

/// Returns wether `obj` is a platform object
/// https://heycam.github.io/webidl/#dfn-platform-object
pub fn is_platform_object(obj: *mut jsapi::JSObject) -> bool {
    unsafe {
        // Fast-path the common case
        let mut clasp = get_object_class(obj);
        if is_dom_class(&*clasp) {
            return true;
        }
        // Now for simplicity check for security wrappers before anything else
        if IsWrapper(obj) {
            let unwrapped_obj = UnwrapObject(obj, /* stopAtWindowProxy = */ 0);
            if unwrapped_obj.is_null() {
                return false;
            }
            clasp = get_object_class(obj);
        }
        // TODO also check if jsapi::JS_IsArrayBufferObject
        is_dom_class(&*clasp)
    }
}

/// Get the property with name `property` from `object`.
/// Returns `Err(())` on JSAPI failure (there is a pending exception), and
/// `Ok(false)` if there was no property with the given name.
pub fn get_dictionary_property(cx: *mut jsapi::JSContext,
                               object: jsapi::JS::HandleObject,
                               property: &str,
                               rval: jsapi::JS::MutableHandleValue)
                               -> Result<bool, ()> {
    fn has_property(cx: *mut jsapi::JSContext,
                    object: jsapi::JS::HandleObject,
                    property: &CString,
                    found: &mut bool)
                    -> bool {
        unsafe { jsapi::JS_HasProperty(cx, object, property.as_ptr(), found) }
    }
    fn get_property(cx: *mut jsapi::JSContext,
                    object: jsapi::JS::HandleObject,
                    property: &CString,
                    value: jsapi::JS::MutableHandleValue)
                    -> bool {
        unsafe { jsapi::JS_GetProperty(cx, object, property.as_ptr(), value) }
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
pub fn set_dictionary_property(cx: *mut jsapi::JSContext,
                               object: jsapi::JS::HandleObject,
                               property: &str,
                               value: jsapi::JS::HandleValue)
                               -> Result<(), ()> {
    if object.get().is_null() {
        return Err(());
    }

    let property = CString::new(property).unwrap();
    unsafe {
        if !jsapi::JS_SetProperty(cx, object, property.as_ptr(), value) {
            return Err(());
        }
    }

    Ok(())
}

/// Returns whether `proxy` has a property `id` on its prototype.
pub unsafe fn has_property_on_prototype(cx: *mut jsapi::JSContext,
                                        proxy: jsapi::JS::HandleObject,
                                        id: jsapi::JS::HandleId,
                                        found: &mut bool)
                                        -> bool {
    rooted!(in(cx) let mut proto = ptr::null_mut());
    if !jsapi::JS_GetPrototype(cx, proxy, proto.handle_mut()) {
        return false;
    }
    assert!(!proto.is_null());
    jsapi::JS_HasPropertyById(cx, proto.handle(), id, found)
}

/// Drop the resources held by reserved slots of a global object
pub unsafe fn finalize_global(obj: *mut jsapi::JSObject) {
    let protolist = get_proto_or_iface_array(obj);
    let list = (*protolist).as_mut_ptr();
    for idx in 0..PROTO_OR_IFACE_LENGTH as isize {
        let entry = list.offset(idx);
        let value = *entry;
        <*mut jsapi::JSObject>::post_barrier(entry, value, ptr::null_mut());
    }
    let _: Box<ProtoOrIfaceArray> = Box::from_raw(protolist);
}

/// Trace the resources held by reserved slots of a global object
pub unsafe fn trace_global(tracer: *mut jsapi::JSTracer, obj: *mut jsapi::JSObject) {
    let array = get_proto_or_iface_array(obj);
    for proto in (*array).iter() {
        if !proto.is_null() {
            trace_object(tracer,
                         "prototype",
                         &*(proto as *const *mut jsapi::JSObject as *const Heap<*mut jsapi::JSObject>));
        }
    }
}

/// Enumerate lazy properties of a global object.
pub unsafe extern "C" fn enumerate_global(cx: *mut jsapi::JSContext, obj: jsapi::JS::HandleObject) -> bool {
    assert!(jsapi::JS_IsGlobalObject(obj.get()));
    if !jsapi::JS_EnumerateStandardClasses(cx, obj) {
        return false;
    }
    for init_fun in InterfaceObjectMap::MAP.values() {
        init_fun(cx, obj);
    }
    true
}

/// Resolve a lazy global property, for interface objects and named constructors.
pub unsafe extern "C" fn resolve_global(
        cx: *mut jsapi::JSContext,
        obj: jsapi::JS::HandleObject,
        id: jsapi::JS::HandleId,
        rval: *mut bool)
        -> bool {
    assert!(jsapi::JS_IsGlobalObject(obj.get()));
    if !jsapi::JS_ResolveStandardClass(cx, obj, id, rval) {
        return false;
    }
    if *rval {
        return true;
    }
    if !RUST_JSID_IS_STRING(id) {
        *rval = false;
        return true;
    }

    let string = RUST_JSID_TO_STRING(id);
    if !jsapi::JS_StringHasLatin1Chars(string) {
        *rval = false;
        return true;
    }
    let mut length = 0;
    let ptr = jsapi::JS_GetLatin1StringCharsAndLength(cx, ptr::null(), string, &mut length);
    assert!(!ptr.is_null());
    let bytes = slice::from_raw_parts(ptr, length as usize);

    if let Some(init_fun) = InterfaceObjectMap::MAP.get(bytes) {
        init_fun(cx, obj);
        *rval = true;
    } else {
        *rval = false;
    }
    true
}

unsafe extern "C" fn wrap(cx: *mut jsapi::JSContext,
                          _existing: jsapi::JS::HandleObject,
                          obj: jsapi::JS::HandleObject)
                          -> *mut jsapi::JSObject {
    // FIXME terrible idea. need security wrappers
    // https://github.com/servo/servo/issues/2382
    WrapperNew(cx, obj, GetCrossCompartmentWrapper(), ptr::null(), false)
}

unsafe extern "C" fn pre_wrap(cx: *mut jsapi::JSContext,
                              _existing: jsapi::JS::HandleObject,
                              obj: jsapi::JS::HandleObject,
                              _object_passed_to_wrap: jsapi::JS::HandleObject,
                              rval: jsapi::JS::MutableHandleObject) {
    let _ac = js::ac::AutoCompartment::with_obj(cx, obj.get());
    if is_global(obj.get()) {
        rval.set(jsapi::js::detail::ToWindowProxyIfWindowSlow(obj.get()));
    } else {
        rval.set(obj.get());
    }
}

/// Callback table for use with jsapi::JS_SetWrapObjectCallbacks
pub static WRAP_CALLBACKS: jsapi::JSWrapObjectCallbacks = jsapi::JSWrapObjectCallbacks {
    wrap: Some(wrap),
    preWrap: Some(pre_wrap),
};

/// Deletes the property `id` from `object`.
pub unsafe fn delete_property_by_id(cx: *mut jsapi::JSContext,
                                    object: jsapi::JS::HandleObject,
                                    id: jsapi::JS::HandleId,
                                    bp: *mut jsapi::JS::ObjectOpResult)
                                    -> bool {
    jsapi::JS_DeletePropertyById(cx, object, id, bp)
}

unsafe fn generic_call(cx: *mut jsapi::JSContext,
                       argc: libc::c_uint,
                       vp: *mut jsapi::JS::Value,
                       is_lenient: bool,
                       call: unsafe extern fn(*const jsapi::JSJitInfo, *mut jsapi::JSContext,
                                              jsapi::JS::HandleObject, *mut libc::c_void, u32,
                                              *mut jsapi::JS::Value)
                                              -> bool)
                       -> bool {
    let args = jsapi::JS::CallArgs::from_vp(vp, argc);
    let thisobj = args.thisv();
    if !thisobj.get().is_null_or_undefined() && !thisobj.get().is_object() {
        return false;
    }
    let obj = if thisobj.get().is_object() {
        thisobj.get().to_object()
    } else {
        jsapi::js::GetGlobalForObjectCrossCompartment(args.calleev().to_object_or_null())
    };
    rooted!(in(cx) let obj = obj);
    let info = RUST_FUNCTION_VALUE_TO_JITINFO(args.calleev().get());
    let proto_id = (*info).__bindgen_anon_2.protoID;
    let depth = (*info).__bindgen_anon_3.depth;
    let proto_check = |class: &'static DOMClass| {
        class.interface_chain[depth as usize] as u16 == proto_id
    };
    let this = match private_from_proto_check(obj.get(), proto_check) {
        Ok(val) => val,
        Err(()) => {
            if is_lenient {
                debug_assert!(!jsapi::JS_IsExceptionPending(cx));
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
pub unsafe extern "C" fn generic_method(cx: *mut jsapi::JSContext,
                                        argc: libc::c_uint,
                                        vp: *mut jsapi::JS::Value)
                                        -> bool {
    generic_call(cx, argc, vp, false, CallJitMethodOp)
}

/// Generic getter of IDL interface.
pub unsafe extern "C" fn generic_getter(cx: *mut jsapi::JSContext,
                                        argc: libc::c_uint,
                                        vp: *mut jsapi::JS::Value)
                                        -> bool {
    generic_call(cx, argc, vp, false, CallJitGetterOp)
}

/// Generic lenient getter of IDL interface.
pub unsafe extern "C" fn generic_lenient_getter(cx: *mut jsapi::JSContext,
                                                argc: libc::c_uint,
                                                vp: *mut jsapi::JS::Value)
                                                -> bool {
    generic_call(cx, argc, vp, true, CallJitGetterOp)
}

unsafe extern "C" fn call_setter(info: *const jsapi::JSJitInfo,
                                 cx: *mut jsapi::JSContext,
                                 handle: jsapi::JS::HandleObject,
                                 this: *mut libc::c_void,
                                 argc: u32,
                                 vp: *mut jsapi::JS::Value)
                                 -> bool {
    if !CallJitSetterOp(info, cx, handle, this, argc, vp) {
        return false;
    }
    *vp = UndefinedValue();
    true
}

/// Generic setter of IDL interface.
pub unsafe extern "C" fn generic_setter(cx: *mut jsapi::JSContext,
                                        argc: libc::c_uint,
                                        vp: *mut jsapi::JS::Value)
                                        -> bool {
    generic_call(cx, argc, vp, false, call_setter)
}

/// Generic lenient setter of IDL interface.
pub unsafe extern "C" fn generic_lenient_setter(cx: *mut jsapi::JSContext,
                                                argc: libc::c_uint,
                                                vp: *mut jsapi::JS::Value)
                                                -> bool {
    generic_call(cx, argc, vp, true, call_setter)
}

unsafe extern "C" fn instance_class_has_proto_at_depth(clasp: *const js::jsapi::js::Class,
                                                       proto_id: u32,
                                                       depth: u32)
                                                       -> bool {
    let domclass: *const DOMJSClass = clasp as *const _;
    let domclass = &*domclass;
    domclass.dom_class.interface_chain[depth as usize] as u32 == proto_id
}

#[allow(missing_docs)]  // FIXME
pub const DOM_CALLBACKS: jsapi::js::DOMCallbacks = jsapi::js::DOMCallbacks {
    instanceClassMatchesProto: Some(instance_class_has_proto_at_depth),
};

// Generic method for returning libc::c_void from caller
pub trait AsVoidPtr {
    fn as_void_ptr(&self) -> *const libc::c_void;
}
impl<T> AsVoidPtr for T {
    fn as_void_ptr(&self) -> *const libc::c_void {
        self as *const T as *const libc::c_void
    }
}

// Generic method for returning c_char from caller
pub trait AsCCharPtrPtr {
    fn as_c_char_ptr(&self) -> *const c_char;
}

impl AsCCharPtrPtr for [u8] {
    fn as_c_char_ptr(&self) -> *const c_char {
        self as *const [u8] as *const c_char
    }
}

pub fn leak_as_static<T>(t: T) -> &'static T {
    let boxed = Box::new(t);
    let ptr = Box::into_raw(boxed) as *const T;
    unsafe {
        ptr.as_ref().unwrap()
    }
}
