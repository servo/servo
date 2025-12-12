/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub(crate) mod base {
    pub(crate) use std::ptr;
    pub(crate) use std::rc::Rc;

    #[allow(unused_imports)]
    pub(crate) use js::context::{JSContext, RawJSContext};
    pub(crate) use js::conversions::{
        ConversionBehavior, ConversionResult, FromJSValConvertible, ToJSValConvertible,
    };
    pub(crate) use js::error::throw_type_error;
    pub(crate) use js::jsapi::{
        HandleValue as RawHandleValue, HandleValueArray, Heap, IsCallable, JS_NewObject, JSObject,
    };
    pub(crate) use js::jsval::{JSVal, NullValue, ObjectOrNullValue, ObjectValue, UndefinedValue};
    pub(crate) use js::panic::maybe_resume_unwind;
    #[allow(unused_imports)]
    pub(crate) use js::realm::{AutoRealm, CurrentRealm};
    pub(crate) use js::rust::wrappers::Call;
    pub(crate) use js::rust::{HandleObject, HandleValue, MutableHandleObject, MutableHandleValue};
    pub(crate) use js::typedarray;
    pub(crate) use js::typedarray::{
        HeapArrayBuffer, HeapArrayBufferView, HeapFloat32Array, HeapFloat64Array, HeapUint8Array,
        HeapUint8ClampedArray,
    };

    pub(crate) use crate::callback::{
        CallSetup, CallbackContainer, CallbackFunction, CallbackInterface, CallbackObject,
        ExceptionHandling, ThisReflector, wrap_call_this_value,
    };
    pub(crate) use crate::codegen::DomTypes::DomTypes;
    pub(crate) use crate::codegen::GenericUnionTypes;
    pub(crate) use crate::conversions::{StringificationBehavior, root_from_handlevalue};
    pub(crate) use crate::error::Error::JSFailed;
    pub(crate) use crate::error::Fallible;
    pub(crate) use crate::interfaces::*;
    pub(crate) use crate::lock::ThreadUnsafeOnceLock;
    pub(crate) use crate::num::Finite;
    pub(crate) use crate::proxyhandler::CrossOriginProperties;
    pub(crate) use crate::reflector::{DomGlobalGeneric, DomObject};
    pub(crate) use crate::root::DomRoot;
    pub(crate) use crate::script_runtime::{CanGc, JSContext as SafeJSContext};
    pub(crate) use crate::str::{ByteString, DOMString, USVString};
    pub(crate) use crate::trace::RootedTraceableBox;
    pub(crate) use crate::utils::{get_dictionary_property, set_dictionary_property};
}

pub(crate) mod module {
    pub(crate) use std::cmp;
    pub(crate) use std::ffi::CString;
    pub(crate) use std::ptr::NonNull;

    pub(crate) use js::conversions::ToJSValConvertible;
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
    pub(crate) use js::{
        JS_CALLEE, JSCLASS_GLOBAL_SLOT_COUNT, JSCLASS_IS_DOMJSCLASS, JSCLASS_IS_GLOBAL,
        JSCLASS_RESERVED_SLOTS_MASK, typedarray,
    };
    pub(crate) use servo_config::pref;

    pub(crate) use super::base::*;
    pub(crate) use crate::codegen::Globals::Globals;
    pub(crate) use crate::codegen::{PrototypeList, RegisterBindings};
    pub(crate) use crate::constant::{ConstantSpec, ConstantVal};
    pub(crate) use crate::constructor::call_default_constructor;
    pub(crate) use crate::conversions::{
        DOM_OBJECT_SLOT, StringificationBehavior, is_array_like, jsid_to_string,
        native_from_handlevalue, native_from_object_static,
    };
    pub(crate) use crate::error::{Error, ErrorResult};
    pub(crate) use crate::finalize::{
        finalize_common, finalize_global, finalize_weak_referenceable,
    };
    pub(crate) use crate::guard::{Condition, Guard};
    pub(crate) use crate::inheritance::Castable;
    pub(crate) use crate::interface::{
        ConstructorClassHook, InterfaceConstructorBehavior, NonCallbackInterfaceObjectClass,
        ProtoOrIfaceIndex, create_callback_interface_object, create_global_object,
        create_interface_prototype_object, create_named_constructors,
        create_noncallback_interface_object, define_dom_interface, define_guarded_methods,
        define_guarded_properties, get_per_interface_object_handle, is_exposed_in,
    };
    pub(crate) use crate::iterable::{Iterable, IterableIterator, IteratorType};
    pub(crate) use crate::like::{Maplike, Setlike};
    pub(crate) use crate::mem::malloc_size_of_including_raw_self;
    pub(crate) use crate::namespace::{NamespaceObjectClass, create_namespace_object};
    pub(crate) use crate::proxyhandler::{
        ensure_expando_object, get_expando_object, set_property_descriptor,
    };
    pub(crate) use crate::realms::{AlreadyInRealm, InRealm};
    pub(crate) use crate::root::{Dom, DomSlice, MaybeUnreflectedDom, Root};
    pub(crate) use crate::script_runtime::CanGc;
    pub(crate) use crate::utils::{
        AsVoidPtr, DOM_PROTO_UNFORGEABLE_HOLDER_SLOT, DOMClass, DOMJSClass, JSCLASS_DOM_GLOBAL,
        ProtoOrIfaceArray, enumerate_global, enumerate_window, exception_to_promise,
        generic_getter, generic_lenient_getter, generic_lenient_setter, generic_method,
        generic_setter, generic_static_promise_method, get_array_index_from_id,
        get_property_on_prototype, has_property_on_prototype, may_resolve_global,
        may_resolve_window, resolve_global, resolve_window, trace_global,
    };
    pub(crate) use crate::weakref::DOM_WEAK_SLOT;
    pub(crate) use crate::{JSTraceable, proxyhandler};
}
