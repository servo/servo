/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Various utilities to glue JavaScript and the DOM implementation together.

use std::cell::RefCell;
use std::os::raw::c_char;
use std::thread::LocalKey;
use std::{ptr, slice};

use js::conversions::ToJSValConvertible;
use js::glue::{IsWrapper, JSPrincipalsCallbacks, UnwrapObjectDynamic, UnwrapObjectStatic};
use js::jsapi::{
    CallArgs, DOMCallbacks, HandleId as RawHandleId, HandleObject as RawHandleObject,
    JS_DeprecatedStringHasLatin1Chars, JS_EnumerateStandardClasses, JS_FreezeObject,
    JS_GetLatin1StringCharsAndLength, JS_IsGlobalObject, JS_ResolveStandardClass, JSContext,
    JSObject, MutableHandleIdVector as RawMutableHandleIdVector,
};
use js::rust::{Handle, HandleObject, MutableHandleValue, get_object_class, is_dom_class};

use crate::DomTypes;
use crate::dom::bindings::codegen::{InterfaceObjectMap, PrototypeList};
use crate::dom::bindings::constructor::call_html_constructor;
use crate::dom::bindings::conversions::DerivedFrom;
use crate::dom::bindings::error::{Error, throw_dom_exception};
use crate::dom::bindings::principals::PRINCIPALS_CALLBACKS;
use crate::dom::bindings::proxyhandler::is_platform_object_same_origin;
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::settings_stack::{self, StackEntry};
use crate::dom::windowproxy::WindowProxyHandler;
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

#[derive(JSTraceable, MallocSizeOf)]
/// Static data associated with a global object.
pub(crate) struct GlobalStaticData {
    #[ignore_malloc_size_of = "WindowProxyHandler does not properly implement it anyway"]
    /// The WindowProxy proxy handler for this global.
    pub(crate) windowproxy_handler: &'static WindowProxyHandler,
}

impl GlobalStaticData {
    /// Creates a new GlobalStaticData.
    pub(crate) fn new() -> GlobalStaticData {
        GlobalStaticData {
            windowproxy_handler: WindowProxyHandler::proxy_handler(),
        }
    }
}

pub(crate) use script_bindings::utils::*;

/// Returns a JSVal representing the frozen JavaScript array
pub(crate) fn to_frozen_array<T: ToJSValConvertible>(
    convertibles: &[T],
    cx: SafeJSContext,
    mut rval: MutableHandleValue,
    _can_gc: CanGc,
) {
    unsafe { convertibles.to_jsval(*cx, rval.reborrow()) };

    rooted!(in(*cx) let obj = rval.to_object());
    unsafe { JS_FreezeObject(*cx, RawHandleObject::from(obj.handle())) };
}

/// Returns wether `obj` is a platform object using dynamic unwrap
/// <https://heycam.github.io/webidl/#dfn-platform-object>
#[allow(dead_code)]
pub(crate) fn is_platform_object_dynamic(obj: *mut JSObject, cx: *mut JSContext) -> bool {
    is_platform_object(obj, &|o| unsafe {
        UnwrapObjectDynamic(o, cx, /* stopAtWindowProxy = */ false)
    })
}

/// Returns wether `obj` is a platform object using static unwrap
/// <https://heycam.github.io/webidl/#dfn-platform-object>
pub(crate) fn is_platform_object_static(obj: *mut JSObject) -> bool {
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

/// Enumerate lazy properties of a global object.
pub(crate) unsafe extern "C" fn enumerate_global<D: DomTypes>(
    cx: *mut JSContext,
    obj: RawHandleObject,
    _props: RawMutableHandleIdVector,
    _enumerable_only: bool,
) -> bool {
    assert!(JS_IsGlobalObject(obj.get()));
    if !JS_EnumerateStandardClasses(cx, obj) {
        return false;
    }
    for init_fun in  <D as DomHelpers<D>>::interface_map().values() {
        init_fun(SafeJSContext::from_ptr(cx), Handle::from_raw(obj));
    }
    true
}

/// Resolve a lazy global property, for interface objects and named constructors.
pub(crate) unsafe extern "C" fn resolve_global<D: DomTypes>(
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

    if let Some(init_fun) = <D as DomHelpers<D>>::interface_map().get(bytes) {
        init_fun(SafeJSContext::from_ptr(cx), Handle::from_raw(obj));
        *rval = true;
    } else {
        *rval = false;
    }
    true
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
pub(crate) const DOM_CALLBACKS: DOMCallbacks = DOMCallbacks {
    instanceClassMatchesProto: Some(instance_class_has_proto_at_depth),
};

// Generic method for returning c_char from caller
pub(crate) trait AsCCharPtrPtr {
    fn as_c_char_ptr(&self) -> *const c_char;
}

impl AsCCharPtrPtr for [u8] {
    fn as_c_char_ptr(&self) -> *const c_char {
        self as *const [u8] as *const c_char
    }
}

/// Operations that must be invoked from the generated bindings.
pub(crate) trait DomHelpers<D: DomTypes> {
    fn throw_dom_exception(cx: SafeJSContext, global: &D::GlobalScope, result: Error, can_gc: CanGc);

    unsafe fn call_html_constructor<T: DerivedFrom<D::Element> + DomObject>(
        cx: SafeJSContext,
        args: &CallArgs,
        global: &D::GlobalScope,
        proto_id: crate::dom::bindings::codegen::PrototypeList::ID,
        creator: unsafe fn(SafeJSContext, HandleObject, *mut ProtoOrIfaceArray),
        can_gc: CanGc,
    ) -> bool;

    fn settings_stack() -> &'static LocalKey<RefCell<Vec<StackEntry<D>>>>;

    fn principals_callbacks() -> &'static JSPrincipalsCallbacks;

    fn is_platform_object_same_origin(cx: SafeJSContext, obj: RawHandleObject) -> bool;

    fn interface_map() -> &'static phf::Map<&'static [u8], for<'a> fn(SafeJSContext, HandleObject)>;
}

impl DomHelpers<crate::DomTypeHolder> for crate::DomTypeHolder {
    fn throw_dom_exception(
        cx: SafeJSContext,
        global: &<crate::DomTypeHolder as DomTypes>::GlobalScope,
        result: Error,
        can_gc: CanGc,
    ) {
        throw_dom_exception(cx, global, result, can_gc)
    }

    unsafe fn call_html_constructor<
        T: DerivedFrom<<crate::DomTypeHolder as DomTypes>::Element> + DomObject,
    >(
        cx: SafeJSContext,
        args: &CallArgs,
        global: &<crate::DomTypeHolder as DomTypes>::GlobalScope,
        proto_id: PrototypeList::ID,
        creator: unsafe fn(SafeJSContext, HandleObject, *mut ProtoOrIfaceArray),
        can_gc: CanGc,
    ) -> bool {
        call_html_constructor::<T>(cx, args, global, proto_id, creator, can_gc)
    }

    fn settings_stack() -> &'static LocalKey<RefCell<Vec<StackEntry<crate::DomTypeHolder>>>> {
        &settings_stack::STACK
    }

    fn principals_callbacks() -> &'static JSPrincipalsCallbacks {
        &PRINCIPALS_CALLBACKS
    }

    fn is_platform_object_same_origin(cx: SafeJSContext, obj: RawHandleObject) -> bool {
        unsafe { is_platform_object_same_origin(cx, obj) }
    }

    fn interface_map() -> &'static phf::Map<&'static [u8], for<'a> fn(SafeJSContext, HandleObject)> {
        &InterfaceObjectMap::MAP
    }
}
