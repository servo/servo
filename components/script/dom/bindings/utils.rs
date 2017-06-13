/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Various utilities to glue JavaScript and the DOM implementation together.

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
use html5ever::LocalName;
use js;
use js::JS_CALLEE;
use js::glue::{CallJitGetterOp, CallJitMethodOp, CallJitSetterOp, IsWrapper};
use js::glue::{GetCrossCompartmentWrapper, WrapperNew};
use js::glue::{RUST_FUNCTION_VALUE_TO_JITINFO, RUST_JSID_IS_INT, RUST_JSID_IS_STRING};
use js::glue::{RUST_JSID_TO_INT, RUST_JSID_TO_STRING, UnwrapObject};
use js::jsapi::{CallArgs, DOMCallbacks, GetGlobalForObjectCrossCompartment};
use js::jsapi::{HandleId, HandleObject, HandleValue, Heap, JSAutoCompartment, JSContext};
use js::jsapi::{JSJitInfo, JSObject, JSTracer, JSWrapObjectCallbacks};
use js::jsapi::{JS_DeletePropertyById, JS_EnumerateStandardClasses};
use js::jsapi::{JS_ForwardGetPropertyTo, JS_GetLatin1StringCharsAndLength};
use js::jsapi::{JS_GetProperty, JS_GetPrototype, JS_GetReservedSlot, JS_HasProperty};
use js::jsapi::{JS_HasPropertyById, JS_IsExceptionPending, JS_IsGlobalObject};
use js::jsapi::{JS_ResolveStandardClass, JS_SetProperty, ToWindowProxyIfWindow};
use js::jsapi::{JS_StringHasLatin1Chars, MutableHandleObject, MutableHandleValue, ObjectOpResult};
use js::jsval::{JSVal, UndefinedValue};
use js::rust::{GCMethods, ToString, get_object_class, is_dom_class};
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
#[derive(Copy, Clone)]
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
pub fn get_proto_or_iface_array(global: *mut JSObject) -> *mut ProtoOrIfaceArray {
    unsafe {
        assert!(((*get_object_class(global)).flags & JSCLASS_DOM_GLOBAL) != 0);
        JS_GetReservedSlot(global, DOM_PROTOTYPE_SLOT).to_private() as *mut ProtoOrIfaceArray
    }
}

/// An array of *mut JSObject of size PROTO_OR_IFACE_LENGTH.
pub type ProtoOrIfaceArray = [*mut JSObject; PROTO_OR_IFACE_LENGTH];

/// Gets the property `id` on  `proxy`'s prototype. If it exists, `*found` is
/// set to true and `*vp` to the value, otherwise `*found` is set to false.
///
/// Returns false on JSAPI failure.
pub unsafe fn get_property_on_prototype(cx: *mut JSContext,
                                        proxy: HandleObject,
                                        receiver: HandleValue,
                                        id: HandleId,
                                        found: *mut bool,
                                        vp: MutableHandleValue)
                                        -> bool {
    rooted!(in(cx) let mut proto = ptr::null_mut());
    if !JS_GetPrototype(cx, proxy, proto.handle_mut()) || proto.is_null() {
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

    JS_ForwardGetPropertyTo(cx, proto.handle(), id, receiver, vp)
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

/// Find the enum equivelent of a string given by `v` in `pairs`.
/// Returns `Err(())` on JSAPI failure (there is a pending exception), and
/// `Ok((None, value))` if there was no matching string.
pub unsafe fn find_enum_value<'a, T>(cx: *mut JSContext,
                                     v: HandleValue,
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
pub fn is_platform_object(obj: *mut JSObject) -> bool {
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
pub unsafe fn has_property_on_prototype(cx: *mut JSContext,
                                        proxy: HandleObject,
                                        id: HandleId,
                                        found: &mut bool)
                                        -> bool {
    rooted!(in(cx) let mut proto = ptr::null_mut());
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
            trace_object(tracer,
                         "prototype",
                         &*(proto as *const *mut JSObject as *const Heap<*mut JSObject>));
        }
    }
}

/// Enumerate lazy properties of a global object.
pub unsafe extern "C" fn enumerate_global(cx: *mut JSContext, obj: HandleObject) -> bool {
    assert!(JS_IsGlobalObject(obj.get()));
    if !JS_EnumerateStandardClasses(cx, obj) {
        return false;
    }
    for init_fun in InterfaceObjectMap::MAP.values() {
        init_fun(cx, obj);
    }
    true
}

/// Resolve a lazy global property, for interface objects and named constructors.
pub unsafe extern "C" fn resolve_global(
        cx: *mut JSContext,
        obj: HandleObject,
        id: HandleId,
        rval: *mut bool)
        -> bool {
    assert!(JS_IsGlobalObject(obj.get()));
    if !JS_ResolveStandardClass(cx, obj, id, rval) {
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
    if !JS_StringHasLatin1Chars(string) {
        *rval = false;
        return true;
    }
    let mut length = 0;
    let ptr = JS_GetLatin1StringCharsAndLength(cx, ptr::null(), string, &mut length);
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
    let obj = ToWindowProxyIfWindow(obj.get());
    assert!(!obj.is_null());
    obj
}

/// Callback table for use with JS_SetWrapObjectCallbacks
pub static WRAP_CALLBACKS: JSWrapObjectCallbacks = JSWrapObjectCallbacks {
    wrap: Some(wrap),
    preWrap: Some(pre_wrap),
};

/// Deletes the property `id` from `object`.
pub unsafe fn delete_property_by_id(cx: *mut JSContext,
                                    object: HandleObject,
                                    id: HandleId,
                                    bp: *mut ObjectOpResult)
                                    -> bool {
    JS_DeletePropertyById(cx, object, id, bp)
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
    rooted!(in(cx) let obj = obj);
    let info = RUST_FUNCTION_VALUE_TO_JITINFO(JS_CALLEE(cx, vp));
    let proto_id = (*info).protoID;
    let depth = (*info).depth;
    let proto_check = |class: &'static DOMClass| {
        class.interface_chain[depth as usize] as u16 == proto_id
    };
    let this = match private_from_proto_check(obj.get(), proto_check) {
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
