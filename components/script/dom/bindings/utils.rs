/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Various utilities to glue JavaScript and the DOM implementation together.

use std::cell::Ref;
use std::ffi::CString;
use std::os::raw::{c_char, c_void};
use std::ptr::NonNull;
use std::{ptr, slice, str};

use js::conversions::ToJSValConvertible;
use js::glue::{
    CallJitGetterOp, CallJitMethodOp, CallJitSetterOp, IsWrapper, JS_GetReservedSlot,
    UnwrapObjectDynamic, UnwrapObjectStatic, RUST_FUNCTION_VALUE_TO_JITINFO,
};
use js::jsapi::{
    AtomToLinearString, CallArgs, DOMCallbacks, ExceptionStackBehavior, GetLinearStringCharAt,
    GetLinearStringLength, GetNonCCWObjectGlobal, HandleId as RawHandleId,
    HandleObject as RawHandleObject, Heap, JSAtom, JSContext, JSJitInfo, JSObject, JSTracer,
    JS_ClearPendingException, JS_DeprecatedStringHasLatin1Chars, JS_EnumerateStandardClasses,
    JS_FreezeObject, JS_GetLatin1StringCharsAndLength, JS_IsExceptionPending, JS_IsGlobalObject,
    JS_ResolveStandardClass, MutableHandleIdVector as RawMutableHandleIdVector,
    MutableHandleValue as RawMutableHandleValue, ObjectOpResult, StringIsArrayIndex,
};
use js::jsval::{JSVal, UndefinedValue};
use js::rust::wrappers::{
    CallOriginalPromiseReject, JS_DeletePropertyById, JS_ForwardGetPropertyTo,
    JS_GetPendingException, JS_GetProperty, JS_GetPrototype, JS_HasProperty, JS_HasPropertyById,
    JS_SetPendingException, JS_SetProperty,
};
use js::rust::{
    get_object_class, is_dom_class, GCMethods, Handle, HandleId, HandleObject, HandleValue,
    MutableHandleValue, ToString,
};
use js::JS_CALLEE;
use malloc_size_of::MallocSizeOfOps;
pub use script_bindings::utils::*;
use servo_media::audio::buffer_source_node::AudioBuffer as ServoMediaAudioBuffer;

use crate::dom::bindings::codegen::Bindings::WindowBinding::Window_Binding::WindowMethods;
use crate::dom::bindings::codegen::PrototypeList::{MAX_PROTO_CHAIN_LENGTH, PROTO_OR_IFACE_LENGTH};
use crate::dom::bindings::codegen::{InterfaceObjectMap, PrototypeList};
use crate::dom::bindings::conversions::{
    jsstring_to_str, private_from_proto_check, PrototypeCheck,
};
use crate::dom::bindings::error::{throw_dom_exception, throw_invalid_this};
use crate::dom::bindings::inheritance::TopTypeId;
use crate::dom::bindings::proxyhandler::report_cross_origin_denial;
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::trace::trace_object;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::readablestream::ReadableStream;
use crate::dom::testbinding::TestBinding;
use crate::dom::webgl2renderingcontext::WebGL2RenderingContext;
use crate::dom::window::Window;
use crate::dom::windowproxy::WindowProxyHandler;
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

#[derive(JSTraceable, MallocSizeOf)]
/// Static data associated with a global object.
pub struct GlobalStaticData {
    #[ignore_malloc_size_of = "WindowProxyHandler does not properly implement it anyway"]
    /// The WindowProxy proxy handler for this global.
    pub windowproxy_handler: &'static WindowProxyHandler,
}

impl GlobalStaticData {
    /// Creates a new GlobalStaticData.
    pub fn new() -> GlobalStaticData {
        GlobalStaticData {
            windowproxy_handler: WindowProxyHandler::proxy_handler(),
        }
    }
}

/// Returns a JSVal representing the frozen JavaScript array
pub fn to_frozen_array<T: ToJSValConvertible>(
    convertibles: &[T],
    cx: SafeJSContext,
    rval: MutableHandleValue,
) {
    unsafe { convertibles.to_jsval(*cx, rval) };

    rooted!(in(*cx) let obj = rval.to_object());
    unsafe { JS_FreezeObject(*cx, RawHandleObject::from(obj.handle())) };
}

impl script_bindings::DomHelpers<crate::DomTypeHolder> for crate::DomTypeHolder {
    fn throw_dom_exception(
        cx: SafeJSContext,
        global: &<crate::DomTypeHolder as script_bindings::DomTypes>::GlobalScope,
        result: script_bindings::error::Error,
    ) {
        throw_dom_exception(cx, global, result)
    }

    unsafe fn global_scope_from_object(
        obj: *mut js::jsapi::JSObject,
    ) -> crate::dom::bindings::root::DomRoot<
        <crate::DomTypeHolder as script_bindings::DomTypes>::GlobalScope,
    > {
        GlobalScope::from_object(obj)
    }

    fn global_scope_origin(
        global: &<crate::DomTypeHolder as script_bindings::DomTypes>::GlobalScope,
    ) -> &servo_url::MutableOrigin {
        global.origin()
    }

    fn Window_create_named_properties_object(
        cx: crate::script_runtime::JSContext,
        proto: js::rust::HandleObject,
        object: js::rust::MutableHandleObject,
    ) {
        Window::create_named_properties_object(cx, proto, object)
    }

    fn Promise_new_resolved(
        global: &<crate::DomTypeHolder as script_bindings::DomTypes>::GlobalScope,
        cx: crate::script_runtime::JSContext,
        value: js::rust::HandleValue,
    ) -> crate::dom::bindings::error::Fallible<
        std::rc::Rc<<crate::DomTypeHolder as script_bindings::DomTypes>::Promise>,
    > {
        Promise::new_resolved(global, cx, value)
    }

    unsafe fn GlobalScope_from_object_maybe_wrapped(
        obj: *mut js::jsapi::JSObject,
        cx: *mut js::jsapi::JSContext,
    ) -> crate::dom::bindings::root::DomRoot<
        <crate::DomTypeHolder as script_bindings::DomTypes>::GlobalScope,
    > {
        GlobalScope::from_object_maybe_wrapped(obj, cx)
    }

    fn GlobalScope_incumbent() -> Option<
        crate::dom::bindings::root::DomRoot<
            <crate::DomTypeHolder as script_bindings::DomTypes>::GlobalScope,
        >,
    > {
        GlobalScope::incumbent()
    }

    fn GlobalScope_get_cx() -> crate::script_runtime::JSContext {
        GlobalScope::get_cx()
    }

    unsafe fn GlobalScope_from_context(
        cx: *mut js::jsapi::JSContext,
        in_realm: crate::realms::InRealm,
    ) -> crate::dom::bindings::root::DomRoot<
        <crate::DomTypeHolder as script_bindings::DomTypes>::GlobalScope,
    > {
        GlobalScope::from_context(cx, in_realm)
    }

    fn GlobalScope_from_reflector(
        reflector: &impl script_bindings::reflector::DomObject,
        realm: &script_bindings::realms::AlreadyInRealm,
    ) -> crate::dom::bindings::root::DomRoot<
        <crate::DomTypeHolder as script_bindings::DomTypes>::GlobalScope,
    > {
        GlobalScope::from_reflector(reflector, &realm.into())
    }

    fn GlobalScope_report_an_error(
        global: &<crate::DomTypeHolder as script_bindings::DomTypes>::GlobalScope,
        info: crate::dom::bindings::error::ErrorInfo,
        value: js::rust::HandleValue,
        can_gc: CanGc,
    ) {
        global.report_an_error(info, value, can_gc)
    }

    fn TestBinding_condition_satisfied(
        cx: crate::script_runtime::JSContext,
        obj: js::rust::HandleObject,
    ) -> bool {
        TestBinding::condition_satisfied(cx, obj)
    }
    fn TestBinding_condition_unsatisfied(
        cx: crate::script_runtime::JSContext,
        obj: js::rust::HandleObject,
    ) -> bool {
        TestBinding::condition_unsatisfied(cx, obj)
    }
    fn WebGL2RenderingContext_is_webgl2_enabled(
        cx: crate::script_runtime::JSContext,
        obj: js::rust::HandleObject,
    ) -> bool {
        WebGL2RenderingContext::is_webgl2_enabled(cx, obj)
    }

    fn perform_a_microtask_checkpoint(
        global: &<crate::DomTypeHolder as script_bindings::DomTypes>::GlobalScope,
        can_gc: CanGc,
    ) {
        global.perform_a_microtask_checkpoint(can_gc)
    }

    unsafe fn ReadableStream_from_js(
        cx: crate::script_runtime::JSContext,
        obj: *mut js::jsapi::JSObject,
        in_realm: crate::realms::InRealm,
    ) -> Result<
        crate::dom::bindings::root::DomRoot<
            <crate::DomTypeHolder as script_bindings::DomTypes>::ReadableStream,
        >,
        (),
    > {
        ReadableStream::from_js(cx, obj, in_realm)
    }

    fn DOMException_stringifier(
        exception: &<crate::DomTypeHolder as script_bindings::DomTypes>::DOMException,
    ) -> crate::dom::bindings::str::DOMString {
        exception.stringifier()
    }

    fn get_map() -> &'static phf::Map<
        &'static [u8],
        fn(crate::script_runtime::JSContext, js::rust::HandleObject),
    > {
        &InterfaceObjectMap::MAP
    }

    fn AudioBuffer_get_channels(
        buffer: &<crate::DomTypeHolder as script_bindings::DomTypes>::AudioBuffer,
    ) -> Ref<Option<ServoMediaAudioBuffer>> {
        buffer.get_channels()
    }

    fn push_new_element_queue() {
        crate::dom::bindings::constructor::push_new_element_queue()
    }

    fn pop_current_element_queue(can_gc: CanGc) {
        crate::dom::bindings::constructor::pop_current_element_queue(can_gc)
    }

    fn call_html_constructor<
        T: script_bindings::conversions::DerivedFrom<
                <crate::DomTypeHolder as script_bindings::DomTypes>::Element,
            > + script_bindings::reflector::DomObject,
    >(
        cx: SafeJSContext,
        args: &CallArgs,
        global: &<crate::DomTypeHolder as script_bindings::DomTypes>::GlobalScope,
        proto_id: crate::dom::bindings::codegen::PrototypeList::ID,
        creator: unsafe fn(SafeJSContext, HandleObject, *mut ProtoOrIfaceArray),
        can_gc: CanGc,
    ) -> bool {
        unsafe {
            crate::dom::bindings::constructor::call_html_constructor::<T>(
                cx, args, global, proto_id, creator, can_gc,
            )
        }
    }

    fn call_default_constructor(
        cx: SafeJSContext,
        args: &CallArgs,
        global: &<crate::DomTypeHolder as script_bindings::DomTypes>::GlobalScope,
        proto_id: crate::dom::bindings::codegen::PrototypeList::ID,
        ctor_name: &str,
        creator: unsafe fn(SafeJSContext, HandleObject, *mut ProtoOrIfaceArray),
        constructor: impl FnOnce(
            SafeJSContext,
            &CallArgs,
            &<crate::DomTypeHolder as script_bindings::DomTypes>::GlobalScope,
            HandleObject,
        ) -> bool,
    ) -> bool {
        unsafe {
            crate::dom::bindings::constructor::call_default_constructor(
                cx,
                args,
                global,
                proto_id,
                ctor_name,
                creator,
                constructor,
            )
        }
    }

    fn is_secure_context(cx: SafeJSContext) -> bool {
        unsafe {
            let in_realm_proof = crate::realms::AlreadyInRealm::assert_for_cx(cx);
            GlobalScope::from_context(*cx, crate::realms::InRealm::Already(&in_realm_proof))
                .is_secure_context()
        }
    }

    fn ensure_safe_to_run_script_or_layout(
        window: &<crate::DomTypeHolder as script_bindings::DomTypes>::Window,
    ) {
        window.Document().ensure_safe_to_run_script_or_layout();
    }
}
