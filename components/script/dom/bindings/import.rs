/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#[allow(unused_imports)]
pub mod base {
    pub use std::ptr;
    pub use std::rc::Rc;

    pub use js::error::throw_type_error;
    pub use js::jsapi::{
        CurrentGlobalOrNull, HandleValue as RawHandleValue, HandleValueArray, Heap, IsCallable,
        JSContext, JSObject, JS_NewObject,
    };
    pub use js::jsval::{JSVal, NullValue, ObjectOrNullValue, ObjectValue, UndefinedValue};
    pub use js::panic::maybe_resume_unwind;
    pub use js::rust::wrappers::{JS_CallFunctionValue, JS_WrapValue};
    pub use js::rust::{HandleObject, HandleValue, MutableHandleObject, MutableHandleValue};

    pub use crate::dom::bindings::callback::{
        wrap_call_this_object, CallSetup, CallbackContainer, CallbackFunction, CallbackInterface,
        CallbackObject, ExceptionHandling,
    };
    pub use crate::dom::bindings::codegen::Bindings::AudioNodeBinding::{
        ChannelCountMode, ChannelCountModeValues, ChannelInterpretation,
        ChannelInterpretationValues,
    };
    pub use crate::dom::bindings::codegen::UnionTypes;
    pub use crate::dom::bindings::conversions::{
        root_from_handlevalue, ConversionBehavior, ConversionResult, FromJSValConvertible,
        StringificationBehavior, ToJSValConvertible,
    };
    pub use crate::dom::bindings::error::Error::JSFailed;
    pub use crate::dom::bindings::error::{throw_dom_exception, Fallible};
    pub use crate::dom::bindings::num::Finite;
    pub use crate::dom::bindings::reflector::DomObject;
    pub use crate::dom::bindings::root::DomRoot;
    pub use crate::dom::bindings::str::{ByteString, DOMString, USVString};
    pub use crate::dom::bindings::trace::RootedTraceableBox;
    pub use crate::dom::bindings::utils::{get_dictionary_property, set_dictionary_property};
    pub use crate::dom::globalscope::GlobalScope;
    pub use crate::script_runtime::JSContext as SafeJSContext;
}

#[allow(unused_imports)]
pub mod module {
    pub use std::cmp;
    pub use std::ffi::CString;
    pub use std::ptr::NonNull;

    pub use js::glue::{
        CreateProxyHandler, GetProxyReservedSlot, JS_GetReservedSlot, ProxyTraps,
        SetProxyReservedSlot,
    };
    pub use js::jsapi::{
        JSJitInfo_OpType, JSJitInfo__bindgen_ty_1, JSJitInfo__bindgen_ty_2,
        JSJitInfo__bindgen_ty_3, JSJitMethodCallArgs, JSJitSetterCallArgs, JSNativeWrapper,
        JSPropertySpec, JSPropertySpec_Accessor, JSPropertySpec_AccessorsOrValue,
        JSPropertySpec_AccessorsOrValue_Accessors, JSPropertySpec_Kind, JSPropertySpec_Name,
        JSPropertySpec_ValueWrapper, JSPropertySpec_ValueWrapper_Type,
        JSPropertySpec_ValueWrapper__bindgen_ty_1, JSTracer, JSTypedMethodJitInfo, JSValueType,
        JS_AtomizeAndPinString, JS_ForwardGetPropertyTo, JS_GetPropertyDescriptorById,
        JS_HasPropertyById, JS_NewPlainObject, JS_SetReservedSlot,
        MutableHandle as RawMutableHandle, MutableHandleIdVector as RawMutableHandleIdVector,
        MutableHandleObject as RawMutableHandleObject, MutableHandleValue as RawMutableHandleValue,
        ObjectOpResult, PropertyDescriptor, SymbolCode, UndefinedHandleValue,
        __BindgenBitfieldUnit, jsid, CallArgs, GCContext, GetRealmErrorPrototype,
        GetRealmFunctionPrototype, GetRealmIteratorPrototype, GetRealmObjectPrototype,
        GetWellKnownSymbol, Handle as RawHandle, HandleId as RawHandleId,
        HandleObject as RawHandleObject, JSAutoRealm, JSClass, JSClassOps, JSFunctionSpec,
        JSJitGetterCallArgs, JSJitInfo, JSJitInfo_AliasSet, JSJitInfo_ArgType,
        JSCLASS_FOREGROUND_FINALIZE, JSCLASS_RESERVED_SLOTS_SHIFT, JSITER_HIDDEN, JSITER_OWNONLY,
        JSITER_SYMBOLS, JSPROP_ENUMERATE, JSPROP_PERMANENT, JSPROP_READONLY,
    };
    pub use js::jsval::PrivateValue;
    pub use js::panic::wrap_panic;
    pub use js::rust::wrappers::{
        int_to_jsid, AppendToIdVector, Call, GetPropertyKeys, JS_CopyOwnPropertiesAndPrivateFields,
        JS_DefineProperty, JS_DefinePropertyById2, JS_GetProperty,
        JS_InitializePropertiesFromCompatibleNativeObject, JS_NewObjectWithGivenProto,
        JS_NewObjectWithoutMetadata, JS_SetImmutablePrototype, JS_SetProperty, JS_SetPrototype,
        JS_WrapObject, NewProxyObject, RUST_INTERNED_STRING_TO_JSID, RUST_SYMBOL_TO_JSID,
    };
    pub use js::rust::{
        get_context_realm, get_object_class, get_object_realm, CustomAutoRooterGuard, GCMethods,
        Handle, MutableHandle, RootedGuard,
    };
    pub use js::typedarray::{
        ArrayBuffer, ArrayBufferView, Float32Array, Float64Array, Uint8Array, Uint8ClampedArray,
    };
    pub use js::{
        jsapi, typedarray, JSCLASS_GLOBAL_SLOT_COUNT, JSCLASS_IS_DOMJSCLASS, JSCLASS_IS_GLOBAL,
        JSCLASS_RESERVED_SLOTS_MASK, JS_CALLEE,
    };
    pub use servo_config::pref;

    pub use super::base::*;
    pub use crate::dom::bindings::codegen::Bindings::AnalyserNodeBinding::AnalyserOptions;
    pub use crate::dom::bindings::codegen::Bindings::AudioNodeBinding::{
        AudioNode_Binding, ChannelCountMode, ChannelCountModeValues, ChannelInterpretation,
        ChannelInterpretationValues,
    };
    pub use crate::dom::bindings::codegen::Bindings::EventTargetBinding::EventTarget_Binding;
    pub use crate::dom::bindings::codegen::{InterfaceObjectMap, PrototypeList, RegisterBindings};
    pub use crate::dom::bindings::constant::{ConstantSpec, ConstantVal};
    pub use crate::dom::bindings::conversions::{
        is_array_like, jsid_to_string, native_from_handlevalue, native_from_object_static,
        IDLInterface, StringificationBehavior, ToJSValConvertible, DOM_OBJECT_SLOT,
    };
    pub use crate::dom::bindings::error::{throw_constructor_without_new, Error, ErrorResult};
    pub use crate::dom::bindings::finalize::{
        finalize_common, finalize_global, finalize_weak_referenceable,
    };
    pub use crate::dom::bindings::guard::{Condition, Guard};
    pub use crate::dom::bindings::htmlconstructor::{
        pop_current_element_queue, push_new_element_queue,
    };
    pub use crate::dom::bindings::inheritance::Castable;
    pub use crate::dom::bindings::interface::{
        create_callback_interface_object, create_global_object, create_interface_prototype_object,
        create_named_constructors, create_noncallback_interface_object, define_dom_interface,
        define_guarded_methods, define_guarded_properties, get_desired_proto,
        get_per_interface_object_handle, is_exposed_in, ConstructorClassHook,
        InterfaceConstructorBehavior, NonCallbackInterfaceObjectClass, ProtoOrIfaceIndex,
    };
    pub use crate::dom::bindings::iterable::{Iterable, IteratorType};
    pub use crate::dom::bindings::like::{Maplike, Setlike};
    pub use crate::dom::bindings::namespace::{create_namespace_object, NamespaceObjectClass};
    pub use crate::dom::bindings::proxyhandler;
    pub use crate::dom::bindings::proxyhandler::{
        ensure_expando_object, get_expando_object, set_property_descriptor,
    };
    pub use crate::dom::bindings::record::Record;
    pub use crate::dom::bindings::reflector::{DomObjectIteratorWrap, DomObjectWrap, Reflector};
    pub use crate::dom::bindings::root::{Dom, DomSlice, MaybeUnreflectedDom, Root};
    pub use crate::dom::bindings::trace::JSTraceable;
    pub use crate::dom::bindings::utils::{
        callargs_is_constructing, enumerate_global, generic_getter, generic_lenient_getter,
        generic_lenient_setter, generic_method, generic_setter, get_array_index_from_id,
        get_property_on_prototype, has_property_on_prototype, resolve_global, trace_global,
        AsVoidPtr, DOMClass, DOMJSClass, ProtoOrIfaceArray, DOM_PROTO_UNFORGEABLE_HOLDER_SLOT,
        JSCLASS_DOM_GLOBAL,
    };
    pub use crate::dom::bindings::weakref::{WeakReferenceable, DOM_WEAK_SLOT};
    pub use crate::dom::types::{AnalyserNode, AudioNode, BaseAudioContext, EventTarget};
    pub use crate::mem::malloc_size_of_including_raw_self;
    pub use crate::realms::{AlreadyInRealm, InRealm};
}
