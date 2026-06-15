/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Various utilities to glue JavaScript and the DOM implementation together.

use std::cell::RefCell;
use std::thread::LocalKey;

use js::context::JSContext;
use js::conversions::ToJSValConvertible;
use js::glue::{IsWrapper, JSPrincipalsCallbacks, UnwrapObjectStatic};
use js::jsapi::{CallArgs, DOMCallbacks, HandleObject as RawHandleObject, JSObject};
use js::realm::CurrentRealm;
use js::rust::wrappers2::JS_FreezeObject;
use js::rust::{HandleObject, MutableHandleValue, get_object_class, is_dom_class};
use script_bindings::interfaces::{DomHelpers, Interface};
use script_bindings::reflector::{DomObject, DomObjectWrap, reflect_dom_object};
use script_bindings::settings_stack::StackEntry;

use crate::DomTypes;
use crate::dom::bindings::codegen::{InterfaceObjectMap, PrototypeList};
use crate::dom::bindings::constructor::call_html_constructor;
use crate::dom::bindings::conversions::DerivedFrom;
use crate::dom::bindings::error::{Error, report_pending_exception, throw_dom_exception};
use crate::dom::bindings::principals::PRINCIPALS_CALLBACKS;
use crate::dom::bindings::proxyhandler::is_platform_object_same_origin;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::settings_stack;
use crate::dom::globalscope::GlobalScope;
use crate::dom::windowproxy::WindowProxyHandler;
use crate::script_runtime::CanGc;
use crate::script_thread::ScriptThread;

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
    cx: &mut JSContext,
    convertibles: &[T],
    mut rval: MutableHandleValue,
) {
    script_bindings::conversions::SafeToJSValConvertible::safe_to_jsval(
        convertibles,
        cx.into(),
        rval.reborrow(),
        CanGc::from_cx(cx),
    );

    rooted!(&in(cx) let obj = rval.to_object());
    unsafe { JS_FreezeObject(cx, obj.handle()) };
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
    let domclass = unsafe { &*domclass };
    domclass.dom_class.interface_chain[depth as usize] as u32 == proto_id
}

/// <https://searchfox.org/mozilla-central/rev/c18faaae88b30182e487fa3341bc7d923e22f23a/xpcom/base/CycleCollectedJSRuntime.cpp#792>
unsafe extern "C" fn instance_class_is_error(clasp: *const js::jsapi::JSClass) -> bool {
    if !is_dom_class(unsafe { &*clasp }) {
        return false;
    }
    let domclass: *const DOMJSClass = clasp as *const _;
    let domclass = unsafe { &*domclass };
    let root_interface = domclass.dom_class.interface_chain[0] as u32;
    // TODO: support checking bare Exception prototype as well.
    root_interface == PrototypeList::ID::DOMException as u32
}

pub(crate) const DOM_CALLBACKS: DOMCallbacks = DOMCallbacks {
    instanceClassMatchesProto: Some(instance_class_has_proto_at_depth),
    instanceClassIsError: Some(instance_class_is_error),
};

/// Eagerly define all relevant WebIDL interface constructors on the
/// provided global object.
pub(crate) fn define_all_exposed_interfaces(cx: &mut CurrentRealm, global: &GlobalScope) {
    for (_, interface) in &InterfaceObjectMap::MAP {
        (interface.define)(cx, global.reflector().get_jsobject());
    }
}

impl DomHelpers<crate::DomTypeHolder> for crate::DomTypeHolder {
    fn throw_dom_exception(
        cx: &mut JSContext,
        global: &<crate::DomTypeHolder as DomTypes>::GlobalScope,
        result: Error,
    ) {
        throw_dom_exception(cx, global, result)
    }

    fn call_html_constructor<
        T: DerivedFrom<<crate::DomTypeHolder as DomTypes>::Element> + DomObject,
    >(
        cx: &mut JSContext,
        args: &CallArgs,
        global: &<crate::DomTypeHolder as DomTypes>::GlobalScope,
        proto_id: PrototypeList::ID,
        creator: unsafe fn(&mut JSContext, HandleObject, *mut ProtoOrIfaceArray),
    ) -> bool {
        call_html_constructor::<T>(cx, args, global, proto_id, creator)
    }

    fn settings_stack() -> &'static LocalKey<RefCell<Vec<StackEntry<crate::DomTypeHolder>>>> {
        &settings_stack::STACK
    }

    fn principals_callbacks() -> &'static JSPrincipalsCallbacks {
        &PRINCIPALS_CALLBACKS
    }

    fn is_platform_object_same_origin(cx: &CurrentRealm, obj: RawHandleObject) -> bool {
        unsafe { is_platform_object_same_origin(cx, obj) }
    }

    fn interface_map() -> &'static phf::Map<&'static [u8], Interface> {
        &InterfaceObjectMap::MAP
    }

    fn push_new_element_queue() {
        ScriptThread::custom_element_reaction_stack().push_new_element_queue()
    }
    fn pop_current_element_queue(cx: &mut JSContext) {
        ScriptThread::custom_element_reaction_stack().pop_current_element_queue(cx)
    }

    fn reflect_dom_object<T, U>(obj: Box<T>, global: &U, can_gc: CanGc) -> DomRoot<T>
    where
        T: DomObject + DomObjectWrap<crate::DomTypeHolder>,
        U: DerivedFrom<GlobalScope>,
    {
        reflect_dom_object(obj, global, can_gc)
    }

    fn report_pending_exception(cx: &mut CurrentRealm) {
        report_pending_exception(cx)
    }
}
