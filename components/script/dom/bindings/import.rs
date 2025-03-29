/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#[allow(unused_imports)]
pub(crate) mod base {
    pub(crate) use std::ptr;
    pub(crate) use std::rc::Rc;

    pub(crate) use js::error::throw_type_error;
    pub(crate) use js::jsapi::{
        CurrentGlobalOrNull, HandleValue as RawHandleValue, HandleValueArray, Heap, IsCallable,
        JS_NewObject, JSContext, JSObject,
    };
    pub(crate) use js::jsval::{JSVal, NullValue, ObjectOrNullValue, ObjectValue, UndefinedValue};
    pub(crate) use js::panic::maybe_resume_unwind;
    pub(crate) use js::rust::wrappers::{Call, JS_WrapValue};
    pub(crate) use js::rust::{HandleObject, HandleValue, MutableHandleObject, MutableHandleValue};

    pub(crate) use crate::dom::bindings::callback::{
        CallSetup, CallbackContainer, CallbackFunction, CallbackInterface, CallbackObject,
        ExceptionHandling, ThisReflector, wrap_call_this_value,
    };
    pub(crate) use crate::dom::bindings::codegen::Bindings::AudioNodeBinding::{
        ChannelCountMode, ChannelCountModeValues, ChannelInterpretation,
        ChannelInterpretationValues,
    };
    pub(crate) use crate::dom::bindings::codegen::DomTypes::DomTypes;
    pub(crate) use crate::dom::bindings::codegen::{GenericUnionTypes, UnionTypes};
    pub(crate) use crate::dom::bindings::conversions::{
        ConversionBehavior, ConversionResult, FromJSValConvertible, StringificationBehavior,
        ToJSValConvertible, root_from_handlevalue,
    };
    pub(crate) use crate::dom::bindings::error::Error::JSFailed;
    pub(crate) use crate::dom::bindings::error::{Fallible, throw_dom_exception};
    pub(crate) use crate::dom::bindings::num::Finite;
    pub(crate) use crate::dom::bindings::proxyhandler::CrossOriginProperties;
    pub(crate) use crate::dom::bindings::reflector::{DomGlobalGeneric, DomObject};
    pub(crate) use crate::dom::bindings::root::DomRoot;
    pub(crate) use crate::dom::bindings::str::{ByteString, DOMString, USVString};
    pub(crate) use crate::dom::bindings::trace::RootedTraceableBox;
    pub(crate) use script_bindings::lock::ThreadUnsafeOnceLock;
    pub(crate) use crate::dom::bindings::utils::{
        DomHelpers, get_dictionary_property, set_dictionary_property,
    };
    pub(crate) use crate::dom::globalscope::{GlobalScope, GlobalScopeHelpers};
    pub(crate) use crate::dom::promise::PromiseHelpers;
    pub(crate) use crate::realms::{AlreadyInRealm, InRealm};
    pub(crate) use crate::script_runtime::{CanGc, JSContext as SafeJSContext};
}

#[allow(unused_imports)]
pub(crate) mod module {
    pub(crate) use std::cmp;
    pub(crate) use std::ffi::CString;
    pub(crate) use std::ptr::NonNull;

    pub(crate) use js::glue::{
        CreateProxyHandler, GetProxyReservedSlot, JS_GetReservedSlot, ProxyTraps,
        SetProxyReservedSlot,
    };
    pub(crate) use js::jsapi::{
        __BindgenBitfieldUnit, CallArgs, GCContext, GetRealmErrorPrototype,
        GetRealmFunctionPrototype, GetRealmIteratorPrototype, GetRealmObjectPrototype,
        GetWellKnownSymbol, Handle as RawHandle, HandleId as RawHandleId,
        HandleObject as RawHandleObject, JS_AtomizeAndPinString, JS_ForwardGetPropertyTo,
        JS_GetPropertyDescriptorById, JS_HasPropertyById, JS_NewPlainObject, JS_SetReservedSlot,
        JSAutoRealm, JSCLASS_FOREGROUND_FINALIZE, JSCLASS_RESERVED_SLOTS_SHIFT, JSClass,
        JSClassOps, JSFunctionSpec, JSITER_HIDDEN, JSITER_OWNONLY, JSITER_SYMBOLS,
        JSJitGetterCallArgs, JSJitInfo, JSJitInfo__bindgen_ty_1, JSJitInfo__bindgen_ty_2,
        JSJitInfo__bindgen_ty_3, JSJitInfo_AliasSet, JSJitInfo_ArgType, JSJitInfo_OpType,
        JSJitMethodCallArgs, JSJitSetterCallArgs, JSNativeWrapper, JSPROP_ENUMERATE,
        JSPROP_PERMANENT, JSPROP_READONLY, JSPropertySpec, JSPropertySpec_Accessor,
        JSPropertySpec_AccessorsOrValue, JSPropertySpec_AccessorsOrValue_Accessors,
        JSPropertySpec_Kind, JSPropertySpec_Name, JSPropertySpec_ValueWrapper,
        JSPropertySpec_ValueWrapper__bindgen_ty_1, JSPropertySpec_ValueWrapper_Type, JSTracer,
        JSTypedMethodJitInfo, JSValueType, MutableHandle as RawMutableHandle,
        MutableHandleIdVector as RawMutableHandleIdVector,
        MutableHandleObject as RawMutableHandleObject, MutableHandleValue as RawMutableHandleValue,
        ObjectOpResult, PropertyDescriptor, SymbolCode, UndefinedHandleValue, jsid,
    };
    pub(crate) use js::jsval::PrivateValue;
    pub(crate) use js::panic::wrap_panic;
    pub(crate) use js::rust::wrappers::{
        AppendToIdVector, Call, GetPropertyKeys, JS_CopyOwnPropertiesAndPrivateFields,
        JS_DefineProperty, JS_DefinePropertyById2, JS_GetProperty,
        JS_InitializePropertiesFromCompatibleNativeObject, JS_NewObjectWithGivenProto,
        JS_NewObjectWithoutMetadata, JS_SetImmutablePrototype, JS_SetProperty, JS_SetPrototype,
        JS_WrapObject, NewProxyObject, RUST_INTERNED_STRING_TO_JSID, RUST_SYMBOL_TO_JSID,
        int_to_jsid,
    };
    pub(crate) use js::rust::{
        CustomAutoRooterGuard, GCMethods, Handle, MutableHandle, get_context_realm,
        get_object_class, get_object_realm,
    };
    pub(crate) use js::typedarray::{
        ArrayBuffer, ArrayBufferView, Float32Array, Float64Array, Uint8Array, Uint8ClampedArray,
    };
    pub(crate) use js::{
        JS_CALLEE, JSCLASS_GLOBAL_SLOT_COUNT, JSCLASS_IS_DOMJSCLASS, JSCLASS_IS_GLOBAL,
        JSCLASS_RESERVED_SLOTS_MASK, jsapi, typedarray,
    };
    pub(crate) use script_bindings::constant::{ConstantSpec, ConstantVal};
    pub(crate) use script_bindings::record::Record;
    pub(crate) use servo_config::pref;

    pub(crate) use super::base::*;
    pub(crate) use crate::dom::bindings::codegen::Bindings::AnalyserNodeBinding::AnalyserOptions;
    pub(crate) use crate::dom::bindings::codegen::Bindings::AudioNodeBinding::{
        AudioNode_Binding, ChannelCountMode, ChannelCountModeValues, ChannelInterpretation,
        ChannelInterpretationValues,
    };
    pub(crate) use crate::dom::bindings::codegen::Bindings::EventTargetBinding::EventTarget_Binding;
    pub(crate) use script_bindings::codegen::Globals::Globals;
    pub(crate) use script_bindings::interfaces::*;
    pub(crate) use crate::dom::bindings::codegen::{
        InterfaceObjectMap, PrototypeList, RegisterBindings,
    };
    pub(crate) use crate::dom::bindings::constructor::{
        call_default_constructor, call_html_constructor, pop_current_element_queue,
        push_new_element_queue,
    };
    pub(crate) use crate::dom::bindings::conversions::{
        DOM_OBJECT_SLOT, IDLInterface, StringificationBehavior, ToJSValConvertible, is_array_like,
        jsid_to_string, native_from_handlevalue, native_from_object_static,
    };
    pub(crate) use crate::dom::bindings::error::{
        Error, ErrorResult, throw_constructor_without_new,
    };
    pub(crate) use script_bindings::finalize::{
        finalize_common, finalize_global, finalize_weak_referenceable,
    };
    pub(crate) use crate::dom::bindings::guard::{Condition, Guard};
    pub(crate) use crate::dom::bindings::inheritance::Castable;
    pub(crate) use crate::dom::bindings::interface::{
        ConstructorClassHook, InterfaceConstructorBehavior, NonCallbackInterfaceObjectClass,
        ProtoOrIfaceIndex, create_callback_interface_object, create_global_object,
        create_interface_prototype_object, create_named_constructors,
        create_noncallback_interface_object, define_dom_interface, define_guarded_methods,
        define_guarded_properties, get_desired_proto, get_per_interface_object_handle,
        is_exposed_in,
    };
    pub(crate) use crate::dom::bindings::iterable::{Iterable, IteratorType};
    pub(crate) use crate::dom::bindings::like::{Maplike, Setlike};
    pub(crate) use crate::dom::bindings::namespace::{
        NamespaceObjectClass, create_namespace_object,
    };
    pub(crate) use crate::dom::bindings::proxyhandler;
    pub(crate) use crate::dom::bindings::proxyhandler::{
        ensure_expando_object, get_expando_object, set_property_descriptor,
    };
    pub(crate) use crate::dom::bindings::reflector::{
        DomObjectIteratorWrap, DomObjectWrap, Reflector,
    };
    pub(crate) use crate::dom::bindings::root::{Dom, DomSlice, MaybeUnreflectedDom, Root};
    pub(crate) use crate::dom::bindings::trace::JSTraceable;
    pub(crate) use crate::dom::bindings::utils::{
        AsVoidPtr, DOM_PROTO_UNFORGEABLE_HOLDER_SLOT, DOMClass, DOMJSClass, JSCLASS_DOM_GLOBAL,
        ProtoOrIfaceArray, enumerate_global, exception_to_promise, generic_getter,
        generic_lenient_getter, generic_lenient_setter, generic_method, generic_setter,
        generic_static_promise_method, get_array_index_from_id, get_property_on_prototype,
        has_property_on_prototype, resolve_global, trace_global,
    };
    pub(crate) use crate::dom::bindings::weakref::{DOM_WEAK_SLOT, WeakReferenceable};
    pub(crate) use crate::dom::types::{AnalyserNode, AudioNode, BaseAudioContext, EventTarget};
    pub(crate) use crate::mem::malloc_size_of_including_raw_self;
    pub(crate) use crate::realms::{AlreadyInRealm, InRealm};
    pub(crate) use crate::script_runtime::CanGc;
}
