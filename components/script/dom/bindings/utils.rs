/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Various utilities to glue JavaScript and the DOM implementation together.

use std::ffi::CString;
use std::os::raw::{c_char, c_void};
use std::{ptr, slice, str};

use js::conversions::ToJSValConvertible;
use js::glue::{
    CallJitGetterOp, CallJitMethodOp, CallJitSetterOp, IsWrapper, JS_GetReservedSlot,
    UnwrapObjectDynamic, UnwrapObjectStatic, RUST_FUNCTION_VALUE_TO_JITINFO,
};
use js::jsapi::{
    AtomToLinearString, CallArgs, DOMCallbacks, GetLinearStringCharAt, GetLinearStringLength,
    GetNonCCWObjectGlobal, HandleId as RawHandleId, HandleObject as RawHandleObject, Heap, JSAtom,
    JSContext, JSJitInfo, JSObject, JSTracer, JS_DeprecatedStringHasLatin1Chars,
    JS_EnumerateStandardClasses, JS_FreezeObject, JS_GetLatin1StringCharsAndLength,
    JS_IsExceptionPending, JS_IsGlobalObject, JS_ResolveStandardClass,
    MutableHandleIdVector as RawMutableHandleIdVector, ObjectOpResult, StringIsArrayIndex,
};
use js::jsval::{JSVal, UndefinedValue};
use js::rust::wrappers::{
    JS_DeletePropertyById, JS_ForwardGetPropertyTo, JS_GetProperty, JS_GetPrototype,
    JS_HasProperty, JS_HasPropertyById, JS_SetProperty,
};
use js::rust::{
    get_object_class, is_dom_class, GCMethods, Handle, HandleId, HandleObject, HandleValue,
    MutableHandleValue, ToString,
};
use js::JS_CALLEE;
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};

use crate::dom::bindings::codegen::PrototypeList::{MAX_PROTO_CHAIN_LENGTH, PROTO_OR_IFACE_LENGTH};
use crate::dom::bindings::codegen::{InterfaceObjectMap, PrototypeList};
use crate::dom::bindings::conversions::{
    jsstring_to_str, private_from_proto_check, PrototypeCheck,
};
use crate::dom::bindings::error::throw_invalid_this;
use crate::dom::bindings::inheritance::TopTypeId;
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::trace::trace_object;
use crate::dom::windowproxy;
use crate::script_runtime::JSContext as SafeJSContext;

/// Proxy handler for a WindowProxy.
pub struct WindowProxyHandler(pub *const libc::c_void);

impl MallocSizeOf for WindowProxyHandler {
    fn size_of(&self, _ops: &mut MallocSizeOfOps) -> usize {
        // FIXME(#6907) this is a pointer to memory allocated by `new` in NewProxyHandler in rust-mozjs.
        0
    }
}

#[derive(JSTraceable, MallocSizeOf)]
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
pub const DOM_PROTOTYPE_SLOT: u32 = js::JSCLASS_GLOBAL_SLOT_COUNT;

/// The flag set on the `JSClass`es for DOM global objects.
// NOTE: This is baked into the Ion JIT as 0 in codegen for LGetDOMProperty and
// LSetDOMProperty. Those constants need to be changed accordingly if this value
// changes.
pub const JSCLASS_DOM_GLOBAL: u32 = js::JSCLASS_USERBIT1;

/// The struct that holds inheritance information for DOM object reflectors.
#[derive(Clone, Copy)]
pub struct DOMClass {
    /// A list of interfaces that this object implements, in order of decreasing
    /// derivedness.
    pub interface_chain: [PrototypeList::ID; MAX_PROTO_CHAIN_LENGTH],

    /// The last valid index of `interface_chain`.
    pub depth: u8,

    /// The type ID of that interface.
    pub type_id: TopTypeId,

    /// The MallocSizeOf function wrapper for that interface.
    pub malloc_size_of: unsafe fn(ops: &mut MallocSizeOfOps, *const c_void) -> usize,

    /// The `Globals` flag for this global interface, if any.
    pub global: InterfaceObjectMap::Globals,
}
unsafe impl Sync for DOMClass {}

/// The JSClass used for DOM object reflectors.
#[derive(Copy)]
#[repr(C)]
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

/// Returns a JSVal representing a frozen array of ports
pub fn to_frozen_array<T: ToJSValConvertible>(convertibles: &[T], cx: SafeJSContext) -> JSVal {
    rooted!(in(*cx) let mut ports = UndefinedValue());
    unsafe { convertibles.to_jsval(*cx, ports.handle_mut()) };

    rooted!(in(*cx) let obj = ports.to_object());
    unsafe { JS_FreezeObject(*cx, RawHandleObject::from(obj.handle())) };
    *ports
}

/// Returns the ProtoOrIfaceArray for the given global object.
/// Fails if `global` is not a DOM global object.
pub fn get_proto_or_iface_array(global: *mut JSObject) -> *mut ProtoOrIfaceArray {
    unsafe {
        assert_ne!(((*get_object_class(global)).flags & JSCLASS_DOM_GLOBAL), 0);
        let mut slot = UndefinedValue();
        JS_GetReservedSlot(global, DOM_PROTOTYPE_SLOT, &mut slot);
        slot.to_private() as *mut ProtoOrIfaceArray
    }
}

/// An array of *mut JSObject of size PROTO_OR_IFACE_LENGTH.
pub type ProtoOrIfaceArray = [*mut JSObject; PROTO_OR_IFACE_LENGTH];

/// Gets the property `id` on  `proxy`'s prototype. If it exists, `*found` is
/// set to true and `*vp` to the value, otherwise `*found` is set to false.
///
/// Returns false on JSAPI failure.
pub unsafe fn get_property_on_prototype(
    cx: *mut JSContext,
    proxy: HandleObject,
    receiver: HandleValue,
    id: HandleId,
    found: *mut bool,
    vp: MutableHandleValue,
) -> bool {
    rooted!(in(cx) let mut proto = ptr::null_mut::<JSObject>());
    if !JS_GetPrototype(cx, proxy, proto.handle_mut()) || proto.is_null() {
        *found = false;
        return true;
    }
    let mut has_property = false;
    if !JS_HasPropertyById(cx, proto.handle(), id, &mut has_property) {
        return false;
    }
    *found = has_property;
    if !has_property {
        return true;
    }

    JS_ForwardGetPropertyTo(cx, proto.handle(), id, receiver, vp)
}

/// Get an array index from the given `jsid`. Returns `None` if the given
/// `jsid` is not an integer.
pub unsafe fn get_array_index_from_id(_cx: *mut JSContext, id: HandleId) -> Option<u32> {
    let raw_id = *id;
    if raw_id.is_int() {
        return Some(raw_id.to_int() as u32);
    }

    if raw_id.is_void() || !raw_id.is_string() {
        return None;
    }

    let atom = raw_id.to_string() as *mut JSAtom;
    let s = AtomToLinearString(atom);
    if GetLinearStringLength(s) == 0 {
        return None;
    }

    let chars = [GetLinearStringCharAt(s, 0)];
    let first_char = char::decode_utf16(chars.iter().cloned())
        .next()
        .map_or('\0', |r| r.unwrap_or('\0'));
    if !first_char.is_ascii_lowercase() {
        return None;
    }

    let mut i = 0;
    if StringIsArrayIndex(s, &mut i) {
        Some(i)
    } else {
        None
    }

    /*let s = jsstr_to_string(cx, RUST_JSID_TO_STRING(raw_id));
    if s.len() == 0 {
        return None;
    }

    let first = s.chars().next().unwrap();
    if first.is_ascii_lowercase() {
        return None;
    }

    let mut i: u32 = 0;
    let is_array = if s.is_ascii() {
        let chars = s.as_bytes();
        StringIsArrayIndex1(chars.as_ptr() as *const _, chars.len() as u32, &mut i)
    } else {
        let chars = s.encode_utf16().collect::<Vec<u16>>();
        let slice = chars.as_slice();
        StringIsArrayIndex2(slice.as_ptr(), chars.len() as u32, &mut i)
    };

    if is_array {
        Some(i)
    } else {
        None
    }*/
}

/// Find the enum equivelent of a string given by `v` in `pairs`.
/// Returns `Err(())` on JSAPI failure (there is a pending exception), and
/// `Ok((None, value))` if there was no matching string.
pub unsafe fn find_enum_value<'a, T>(
    cx: *mut JSContext,
    v: HandleValue,
    pairs: &'a [(&'static str, T)],
) -> Result<(Option<&'a T>, DOMString), ()> {
    let jsstr = ToString(cx, v);
    if jsstr.is_null() {
        return Err(());
    }

    let search = jsstring_to_str(cx, jsstr);
    Ok((
        pairs
            .iter()
            .find(|&&(key, _)| search == *key)
            .map(|(_, ev)| ev),
        search,
    ))
}

/// Returns wether `obj` is a platform object using dynamic unwrap
/// <https://heycam.github.io/webidl/#dfn-platform-object>
#[allow(dead_code)]
pub fn is_platform_object_dynamic(obj: *mut JSObject, cx: *mut JSContext) -> bool {
    is_platform_object(obj, &|o| unsafe {
        UnwrapObjectDynamic(o, cx, /* stopAtWindowProxy = */ false)
    })
}

/// Returns wether `obj` is a platform object using static unwrap
/// <https://heycam.github.io/webidl/#dfn-platform-object>
pub fn is_platform_object_static(obj: *mut JSObject) -> bool {
    is_platform_object(obj, &|o| unsafe { UnwrapObjectStatic(o) })
}

fn is_platform_object(
    obj: *mut JSObject,
    unwrap_obj: &dyn Fn(*mut JSObject) -> *mut JSObject,
) -> bool {
    unsafe {
        // Fast-path the common case
        let mut clasp = get_object_class(obj);
        if is_dom_class(&*clasp) {
            return true;
        }
        // Now for simplicity check for security wrappers before anything else
        if IsWrapper(obj) {
            let unwrapped_obj = unwrap_obj(obj);
            if unwrapped_obj.is_null() {
                return false;
            }
            clasp = get_object_class(obj);
        }
        // TODO also check if JS_IsArrayBufferObject
        is_dom_class(&*clasp)
    }
}

/// Get the property with name `property` from `object`.
/// Returns `Err(())` on JSAPI failure (there is a pending exception), and
/// `Ok(false)` if there was no property with the given name.
pub fn get_dictionary_property(
    cx: *mut JSContext,
    object: HandleObject,
    property: &str,
    rval: MutableHandleValue,
) -> Result<bool, ()> {
    fn has_property(
        cx: *mut JSContext,
        object: HandleObject,
        property: &CString,
        found: &mut bool,
    ) -> bool {
        unsafe { JS_HasProperty(cx, object, property.as_ptr(), found) }
    }
    fn get_property(
        cx: *mut JSContext,
        object: HandleObject,
        property: &CString,
        value: MutableHandleValue,
    ) -> bool {
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
pub fn set_dictionary_property(
    cx: *mut JSContext,
    object: HandleObject,
    property: &str,
    value: HandleValue,
) -> Result<(), ()> {
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
pub unsafe fn has_property_on_prototype(
    cx: *mut JSContext,
    proxy: HandleObject,
    id: HandleId,
    found: &mut bool,
) -> bool {
    rooted!(in(cx) let mut proto = ptr::null_mut::<JSObject>());
    if !JS_GetPrototype(cx, proxy, proto.handle_mut()) {
        return false;
    }
    assert!(!proto.is_null());
    JS_HasPropertyById(cx, proto.handle(), id, found)
}

/// Drop the resources held by reserved slots of a global object
pub unsafe fn finalize_global(obj: *mut JSObject) {
    let protolist = get_proto_or_iface_array(obj);
    let list = (*protolist).as_mut_ptr();
    for idx in 0..PROTO_OR_IFACE_LENGTH as isize {
        let entry = list.offset(idx);
        let value = *entry;
        <*mut JSObject>::post_barrier(entry, value, ptr::null_mut());
    }
    let _: Box<ProtoOrIfaceArray> = Box::from_raw(protolist);
}

/// Trace the resources held by reserved slots of a global object
pub unsafe fn trace_global(tracer: *mut JSTracer, obj: *mut JSObject) {
    let array = get_proto_or_iface_array(obj);
    for proto in (*array).iter() {
        if !proto.is_null() {
            trace_object(
                tracer,
                "prototype",
                &*(proto as *const *mut JSObject as *const Heap<*mut JSObject>),
            );
        }
    }
}

/// Enumerate lazy properties of a global object.
pub unsafe extern "C" fn enumerate_global(
    cx: *mut JSContext,
    obj: RawHandleObject,
    _props: RawMutableHandleIdVector,
    _enumerable_only: bool,
) -> bool {
    assert!(JS_IsGlobalObject(obj.get()));
    if !JS_EnumerateStandardClasses(cx, obj) {
        return false;
    }
    for init_fun in InterfaceObjectMap::MAP.values() {
        init_fun(SafeJSContext::from_ptr(cx), Handle::from_raw(obj));
    }
    true
}

/// Resolve a lazy global property, for interface objects and named constructors.
pub unsafe extern "C" fn resolve_global(
    cx: *mut JSContext,
    obj: RawHandleObject,
    id: RawHandleId,
    rval: *mut bool,
) -> bool {
    assert!(JS_IsGlobalObject(obj.get()));
    if !JS_ResolveStandardClass(cx, obj, id, rval) {
        return false;
    }
    if *rval {
        return true;
    }
    if !id.is_string() {
        *rval = false;
        return true;
    }

    let string = id.to_string();
    if !JS_DeprecatedStringHasLatin1Chars(string) {
        *rval = false;
        return true;
    }
    let mut length = 0;
    let ptr = JS_GetLatin1StringCharsAndLength(cx, ptr::null(), string, &mut length);
    assert!(!ptr.is_null());
    let bytes = slice::from_raw_parts(ptr, length);

    if let Some(init_fun) = InterfaceObjectMap::MAP.get(bytes) {
        init_fun(SafeJSContext::from_ptr(cx), Handle::from_raw(obj));
        *rval = true;
    } else {
        *rval = false;
    }
    true
}

/// Deletes the property `id` from `object`.
pub unsafe fn delete_property_by_id(
    cx: *mut JSContext,
    object: HandleObject,
    id: HandleId,
    bp: *mut ObjectOpResult,
) -> bool {
    JS_DeletePropertyById(cx, object, id, bp)
}

unsafe fn generic_call(
    cx: *mut JSContext,
    argc: libc::c_uint,
    vp: *mut JSVal,
    is_lenient: bool,
    call: unsafe extern "C" fn(
        *const JSJitInfo,
        *mut JSContext,
        RawHandleObject,
        *mut libc::c_void,
        u32,
        *mut JSVal,
    ) -> bool,
) -> bool {
    let args = CallArgs::from_vp(vp, argc);

    let info = RUST_FUNCTION_VALUE_TO_JITINFO(JS_CALLEE(cx, vp));
    let proto_id = (*info).__bindgen_anon_2.protoID;

    let thisobj = args.thisv();
    if !thisobj.get().is_null_or_undefined() && !thisobj.get().is_object() {
        throw_invalid_this(cx, proto_id);
        return false;
    }

    rooted!(in(cx) let obj = if thisobj.get().is_object() {
        thisobj.get().to_object()
    } else {
        GetNonCCWObjectGlobal(JS_CALLEE(cx, vp).to_object_or_null())
    });
    let depth = (*info).__bindgen_anon_3.depth as usize;
    let proto_check = PrototypeCheck::Depth { depth, proto_id };
    let this = match private_from_proto_check(obj.get(), cx, proto_check) {
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
        },
    };
    call(
        info,
        cx,
        obj.handle().into(),
        this as *mut libc::c_void,
        argc,
        vp,
    )
}

/// Generic method of IDL interface.
pub unsafe extern "C" fn generic_method(
    cx: *mut JSContext,
    argc: libc::c_uint,
    vp: *mut JSVal,
) -> bool {
    generic_call(cx, argc, vp, false, CallJitMethodOp)
}

/// Generic getter of IDL interface.
pub unsafe extern "C" fn generic_getter(
    cx: *mut JSContext,
    argc: libc::c_uint,
    vp: *mut JSVal,
) -> bool {
    generic_call(cx, argc, vp, false, CallJitGetterOp)
}

/// Generic lenient getter of IDL interface.
pub unsafe extern "C" fn generic_lenient_getter(
    cx: *mut JSContext,
    argc: libc::c_uint,
    vp: *mut JSVal,
) -> bool {
    generic_call(cx, argc, vp, true, CallJitGetterOp)
}

unsafe extern "C" fn call_setter(
    info: *const JSJitInfo,
    cx: *mut JSContext,
    handle: RawHandleObject,
    this: *mut libc::c_void,
    argc: u32,
    vp: *mut JSVal,
) -> bool {
    if !CallJitSetterOp(info, cx, handle, this, argc, vp) {
        return false;
    }
    *vp = UndefinedValue();
    true
}

/// Generic setter of IDL interface.
pub unsafe extern "C" fn generic_setter(
    cx: *mut JSContext,
    argc: libc::c_uint,
    vp: *mut JSVal,
) -> bool {
    generic_call(cx, argc, vp, false, call_setter)
}

/// Generic lenient setter of IDL interface.
pub unsafe extern "C" fn generic_lenient_setter(
    cx: *mut JSContext,
    argc: libc::c_uint,
    vp: *mut JSVal,
) -> bool {
    generic_call(cx, argc, vp, true, call_setter)
}

unsafe extern "C" fn instance_class_has_proto_at_depth(
    clasp: *const js::jsapi::JSClass,
    proto_id: u32,
    depth: u32,
) -> bool {
    let domclass: *const DOMJSClass = clasp as *const _;
    let domclass = &*domclass;
    domclass.dom_class.interface_chain[depth as usize] as u32 == proto_id
}

#[allow(missing_docs)] // FIXME
pub const DOM_CALLBACKS: DOMCallbacks = DOMCallbacks {
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

pub unsafe fn callargs_is_constructing(args: &CallArgs) -> bool {
    (*args.argv_.offset(-1)).is_magic()
}
