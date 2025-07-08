/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Various utilities to glue JavaScript and the DOM implementation together.

use std::cell::RefCell;
use std::thread::LocalKey;

use js::conversions::ToJSValConvertible;
use js::glue::{IsWrapper, JSPrincipalsCallbacks, UnwrapObjectDynamic, UnwrapObjectStatic};
use js::jsapi::{
    CallArgs, DOMCallbacks, HandleObject as RawHandleObject, JS_FreezeObject, JSContext, JSObject,
};
use js::rust::{HandleObject, MutableHandleValue, get_object_class, is_dom_class};
use script_bindings::interfaces::{DomHelpers, Interface};
use script_bindings::settings_stack::StackEntry;

use crate::DomTypes;
use crate::dom::bindings::codegen::{InterfaceObjectMap, PrototypeList};
use crate::dom::bindings::constructor::{
    call_html_constructor, pop_current_element_queue, push_new_element_queue,
};
use crate::dom::bindings::conversions::DerivedFrom;
use crate::dom::bindings::error::{Error, report_pending_exception, throw_dom_exception};
use crate::dom::bindings::principals::PRINCIPALS_CALLBACKS;
use crate::dom::bindings::proxyhandler::is_platform_object_same_origin;
use crate::dom::bindings::reflector::{DomObject, DomObjectWrap, reflect_dom_object};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::settings_stack;
use crate::dom::globalscope::GlobalScope;
use crate::dom::windowproxy::WindowProxyHandler;
use crate::realms::InRealm;
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

unsafe extern "C" fn instance_class_has_proto_at_depth(
    clasp: *const js::jsapi::JSClass,
    proto_id: u32,
    depth: u32,
) -> bool {
    let domclass: *const DOMJSClass = clasp as *const _;
    let domclass = &*domclass;
    domclass.dom_class.interface_chain[depth as usize] as u32 == proto_id
}

/// <https://searchfox.org/mozilla-central/rev/c18faaae88b30182e487fa3341bc7d923e22f23a/xpcom/base/CycleCollectedJSRuntime.cpp#792>
unsafe extern "C" fn instance_class_is_error(clasp: *const js::jsapi::JSClass) -> bool {
    if !is_dom_class(&*clasp) {
        return false;
    }
    let domclass: *const DOMJSClass = clasp as *const _;
    let domclass = &*domclass;
    let root_interface = domclass.dom_class.interface_chain[0] as u32;
    // TODO: support checking bare Exception prototype as well.
    root_interface == PrototypeList::ID::DOMException as u32
}

#[allow(missing_docs)] // FIXME
pub(crate) const DOM_CALLBACKS: DOMCallbacks = DOMCallbacks {
    instanceClassMatchesProto: Some(instance_class_has_proto_at_depth),
    instanceClassIsError: Some(instance_class_is_error),
};

/// Eagerly define all relevant WebIDL interface constructors on the
/// provided global object.
pub(crate) fn define_all_exposed_interfaces(
    global: &GlobalScope,
    _in_realm: InRealm,
    _can_gc: CanGc,
) {
    let cx = GlobalScope::get_cx();
    for (_, interface) in &InterfaceObjectMap::MAP {
        (interface.define)(cx, global.reflector().get_jsobject());
    }
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

    fn call_html_constructor<
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

    fn interface_map() -> &'static phf::Map<&'static [u8], Interface> {
        &InterfaceObjectMap::MAP
    }

    fn push_new_element_queue() {
        push_new_element_queue()
    }
    fn pop_current_element_queue(can_gc: CanGc) {
        pop_current_element_queue(can_gc)
    }

    fn reflect_dom_object<T, U>(obj: Box<T>, global: &U, can_gc: CanGc) -> DomRoot<T>
    where
        T: DomObject + DomObjectWrap<crate::DomTypeHolder>,
        U: DerivedFrom<GlobalScope>,
    {
        reflect_dom_object(obj, global, can_gc)
    }

    fn report_pending_exception(
        cx: SafeJSContext,
        dispatch_event: bool,
        realm: InRealm,
        can_gc: CanGc,
    ) {
        report_pending_exception(cx, dispatch_event, realm, can_gc)
    }
}
