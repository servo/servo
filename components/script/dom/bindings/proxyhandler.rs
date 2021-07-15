/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Utilities for the implementation of JSAPI proxy handlers.

#![deny(missing_docs)]

use crate::dom::bindings::conversions::is_dom_proxy;
use crate::dom::bindings::error::{throw_dom_exception, Error};
use crate::dom::bindings::principals::ServoJSPrincipalsRef;
use crate::dom::bindings::utils::delete_property_by_id;
use crate::dom::globalscope::GlobalScope;
use crate::realms::{AlreadyInRealm, InRealm};
use crate::script_runtime::JSContext as SafeJSContext;
use js::conversions::ToJSValConvertible;
use js::glue::{
    GetProxyHandler, GetProxyHandlerFamily, InvokeGetOwnPropertyDescriptor, RUST_SYMBOL_TO_JSID,
};
use js::glue::{GetProxyPrivate, SetProxyPrivate};
use js::jsapi;
use js::jsapi::GetStaticPrototype;
use js::jsapi::Handle as RawHandle;
use js::jsapi::HandleId as RawHandleId;
use js::jsapi::HandleObject as RawHandleObject;
use js::jsapi::HandleValue as RawHandleValue;
use js::jsapi::JS_AtomizeAndPinString;
use js::jsapi::JS_DefinePropertyById;
use js::jsapi::JS_GetOwnPropertyDescriptorById;
use js::jsapi::JS_IsExceptionPending;
use js::jsapi::MutableHandle as RawMutableHandle;
use js::jsapi::MutableHandleIdVector as RawMutableHandleIdVector;
use js::jsapi::MutableHandleObject as RawMutableHandleObject;
use js::jsapi::MutableHandleValue as RawMutableHandleValue;
use js::jsapi::ObjectOpResult;
use js::jsapi::{jsid, GetObjectRealmOrNull, GetRealmPrincipals, JSFunctionSpec, JSPropertySpec};
use js::jsapi::{DOMProxyShadowsResult, JSContext, JSObject, PropertyDescriptor};
use js::jsapi::{GetWellKnownSymbol, SymbolCode};
use js::jsapi::{JSErrNum, SetDOMProxyInformation};
use js::jsval::ObjectValue;
use js::jsval::UndefinedValue;
use js::rust::wrappers::JS_AlreadyHasOwnPropertyById;
use js::rust::wrappers::JS_NewObjectWithGivenProto;
use js::rust::wrappers::{AppendToIdVector, RUST_INTERNED_STRING_TO_JSID};
use js::rust::{get_context_realm, Handle, HandleObject, MutableHandle, MutableHandleObject};
use std::{ffi::CStr, ptr};

/// Determine if this id shadows any existing properties for this proxy.
pub unsafe extern "C" fn shadow_check_callback(
    cx: *mut JSContext,
    object: RawHandleObject,
    id: RawHandleId,
) -> DOMProxyShadowsResult {
    // TODO: support OverrideBuiltins when #12978 is fixed.

    rooted!(in(cx) let mut expando = ptr::null_mut::<JSObject>());
    get_expando_object(object, expando.handle_mut());
    if !expando.get().is_null() {
        let mut has_own = false;
        let raw_id = Handle::from_raw(id);

        if !JS_AlreadyHasOwnPropertyById(cx, expando.handle(), raw_id, &mut has_own) {
            return DOMProxyShadowsResult::ShadowCheckFailed;
        }

        if has_own {
            return DOMProxyShadowsResult::ShadowsViaDirectExpando;
        }
    }

    // Our expando, if any, didn't shadow, so we're not shadowing at all.
    DOMProxyShadowsResult::DoesntShadow
}

/// Initialize the infrastructure for DOM proxy objects.
pub unsafe fn init() {
    SetDOMProxyInformation(
        GetProxyHandlerFamily(),
        Some(shadow_check_callback),
        ptr::null(),
    );
}

/// Defines an expando on the given `proxy`.
pub unsafe extern "C" fn define_property(
    cx: *mut JSContext,
    proxy: RawHandleObject,
    id: RawHandleId,
    desc: RawHandle<PropertyDescriptor>,
    result: *mut ObjectOpResult,
) -> bool {
    rooted!(in(cx) let mut expando = ptr::null_mut::<JSObject>());
    ensure_expando_object(cx, proxy, expando.handle_mut());
    JS_DefinePropertyById(cx, expando.handle().into(), id, desc, result)
}

/// Deletes an expando off the given `proxy`.
pub unsafe extern "C" fn delete(
    cx: *mut JSContext,
    proxy: RawHandleObject,
    id: RawHandleId,
    bp: *mut ObjectOpResult,
) -> bool {
    rooted!(in(cx) let mut expando = ptr::null_mut::<JSObject>());
    get_expando_object(proxy, expando.handle_mut());
    if expando.is_null() {
        (*bp).code_ = 0 /* OkCode */;
        return true;
    }

    delete_property_by_id(cx, expando.handle(), Handle::from_raw(id), bp)
}

/// Controls whether the Extensible bit can be changed
pub unsafe extern "C" fn prevent_extensions(
    _cx: *mut JSContext,
    _proxy: RawHandleObject,
    result: *mut ObjectOpResult,
) -> bool {
    (*result).code_ = JSErrNum::JSMSG_CANT_PREVENT_EXTENSIONS as ::libc::uintptr_t;
    true
}

/// Reports whether the object is Extensible
pub unsafe extern "C" fn is_extensible(
    _cx: *mut JSContext,
    _proxy: RawHandleObject,
    succeeded: *mut bool,
) -> bool {
    *succeeded = true;
    true
}

/// If `proxy` (underneath any functionally-transparent wrapper proxies) has as
/// its `[[GetPrototypeOf]]` trap the ordinary `[[GetPrototypeOf]]` behavior
/// defined for ordinary objects, set `*is_ordinary` to true and store `obj`'s
/// prototype in `proto`.  Otherwise set `*isOrdinary` to false. In case of
/// error, both outparams have unspecified value.
///
/// This implementation always handles the case of the ordinary
/// `[[GetPrototypeOf]]` behavior. An alternative implementation will be
/// necessary for the Location object.
pub unsafe extern "C" fn get_prototype_if_ordinary(
    _: *mut JSContext,
    proxy: RawHandleObject,
    is_ordinary: *mut bool,
    proto: RawMutableHandleObject,
) -> bool {
    *is_ordinary = true;
    proto.set(GetStaticPrototype(proxy.get()));
    true
}

/// Get the expando object, or null if there is none.
pub unsafe fn get_expando_object(obj: RawHandleObject, mut expando: MutableHandleObject) {
    assert!(is_dom_proxy(obj.get()));
    let ref mut val = UndefinedValue();
    GetProxyPrivate(obj.get(), val);
    expando.set(if val.is_undefined() {
        ptr::null_mut()
    } else {
        val.to_object()
    });
}

/// Get the expando object, or create it if it doesn't exist yet.
/// Fails on JSAPI failure.
pub unsafe fn ensure_expando_object(
    cx: *mut JSContext,
    obj: RawHandleObject,
    mut expando: MutableHandleObject,
) {
    assert!(is_dom_proxy(obj.get()));
    get_expando_object(obj, expando);
    if expando.is_null() {
        expando.set(JS_NewObjectWithGivenProto(
            cx,
            ptr::null_mut(),
            HandleObject::null(),
        ));
        assert!(!expando.is_null());

        SetProxyPrivate(obj.get(), &ObjectValue(expando.get()));
    }
}

/// Set the property descriptor's object to `obj` and set it to enumerable,
/// and writable if `readonly` is true.
pub fn fill_property_descriptor(
    mut desc: MutableHandle<PropertyDescriptor>,
    obj: *mut JSObject,
    attrs: u32,
) {
    desc.obj = obj;
    desc.attrs = attrs;
    desc.getter = None;
    desc.setter = None;
}

/// <https://html.spec.whatwg.org/multipage/#isplatformobjectsameorigin-(-o-)>
pub unsafe fn is_platform_object_same_origin(cx: SafeJSContext, obj: RawHandleObject) -> bool {
    let subject_realm = get_context_realm(*cx);
    let obj_realm = GetObjectRealmOrNull(*obj);
    assert!(!obj_realm.is_null());

    let subject = ServoJSPrincipalsRef::from_raw_unchecked(GetRealmPrincipals(subject_realm));
    let obj = ServoJSPrincipalsRef::from_raw_unchecked(GetRealmPrincipals(obj_realm));

    let subject_origin = subject.origin();
    let obj_origin = obj.origin();

    subject_origin.same_origin_domain(&obj_origin)
}

/// Report a cross-origin denial for a property, Always returns `false`, so it
/// can be used as `return report_cross_origin_denial(...);`.
///
/// What this function does corresponds to the operations in
/// <https://html.spec.whatwg.org/multipage/#the-location-interface> denoted as
/// "Throw a `SecurityError` DOMException".
pub unsafe fn report_cross_origin_denial(
    cx: SafeJSContext,
    _id: RawHandleId,
    _access: &str,
) -> bool {
    let in_realm_proof = AlreadyInRealm::assert_for_cx(cx);
    if !JS_IsExceptionPending(*cx) {
        let global = GlobalScope::from_context(*cx, InRealm::Already(&in_realm_proof));
        // TODO: include `id` and `access` in the exception message
        throw_dom_exception(cx, &*global, Error::Security);
    }
    false
}

/// Property and method specs that correspond to the elements of
/// [`CrossOriginProperties(O)`].
///
/// [`CrossOriginProperties(O)`]: https://html.spec.whatwg.org/multipage/#crossoriginproperties-(-o-)
pub struct CrossOriginProperties {
    pub attributes: &'static [JSPropertySpec],
    pub methods: &'static [JSFunctionSpec],
}

impl CrossOriginProperties {
    /// Enumerate the property keys defined by `self`.
    fn keys(&self) -> impl Iterator<Item = *const std::os::raw::c_char> + '_ {
        // Safety: All cross-origin property keys are strings, not symbols
        self.attributes
            .iter()
            .map(|spec| unsafe { spec.name.string_ })
            .chain(self.methods.iter().map(|spec| unsafe { spec.name.string_ }))
            .filter(|ptr| !ptr.is_null())
    }
}

/// Implementation of [`CrossOriginOwnPropertyKeys`].
///
/// [`CrossOriginOwnPropertyKeys`]: https://html.spec.whatwg.org/multipage/#crossoriginownpropertykeys-(-o-)
pub unsafe fn cross_origin_own_property_keys(
    cx: SafeJSContext,
    _proxy: RawHandleObject,
    cross_origin_properties: &'static CrossOriginProperties,
    props: RawMutableHandleIdVector,
) -> bool {
    for key in cross_origin_properties.keys() {
        let jsstring = JS_AtomizeAndPinString(*cx, key);
        rooted!(in(*cx) let rooted = jsstring);
        rooted!(in(*cx) let mut rooted_jsid: jsid);
        RUST_INTERNED_STRING_TO_JSID(*cx, rooted.handle().get(), rooted_jsid.handle_mut());
        AppendToIdVector(props, rooted_jsid.handle());
    }
    true
}

/// Implementation of [`CrossOriginGet`].
///
/// [`CrossOriginGet`]: https://html.spec.whatwg.org/multipage/#crossoriginget-(-o,-p,-receiver-)
pub unsafe fn cross_origin_get(
    cx: SafeJSContext,
    proxy: RawHandleObject,
    receiver: RawHandleValue,
    id: RawHandleId,
    vp: RawMutableHandleValue,
) -> bool {
    // > 1. Let `desc` be `? O.[[GetOwnProperty]](P)`.
    rooted!(in(*cx) let mut descriptor = PropertyDescriptor::default());
    if !InvokeGetOwnPropertyDescriptor(
        GetProxyHandler(*proxy),
        *cx,
        proxy,
        id,
        descriptor.handle_mut().into(),
    ) {
        return false;
    }
    //    let descriptor = descriptor.get();

    // > 2. Assert: `desc` is not undefined.
    assert!(
        !descriptor.obj.is_null(),
        "Callees should throw in all cases when they are not finding \
        a property decriptor"
    );

    // > 3. If `! IsDataDescriptor(desc)` is true, then return `desc.[[Value]]`.
    if is_data_descriptor(&descriptor) {
        vp.set(descriptor.value);
        return true;
    }

    // > 4. Assert: `IsAccessorDescriptor(desc)` is `true`.
    assert!(is_accessor_descriptor(&descriptor));

    // > 5. Let `getter` be `desc.[[Get]]`.
    // >
    // > 6. If `getter` is `undefined`, then throw a `SecurityError`
    // >    `DOMException`.
    rooted!(in(*cx) let mut getter = ptr::null_mut::<JSObject>());
    get_getter_object(&descriptor, getter.handle_mut().into());
    if getter.get().is_null() {
        return report_cross_origin_denial(cx, id, "get");
    }

    rooted!(in(*cx) let mut getter_jsval = UndefinedValue());
    getter.get().to_jsval(*cx, getter_jsval.handle_mut());

    // > 7. Return `? Call(getter, Receiver)`.
    jsapi::Call(
        *cx,
        receiver,
        getter_jsval.handle().into(),
        &jsapi::HandleValueArray::new(),
        vp,
    )
}

unsafe fn get_getter_object(d: &PropertyDescriptor, out: RawMutableHandleObject) {
    if (d.attrs & jsapi::JSPROP_GETTER as u32) != 0 {
        out.set(std::mem::transmute(d.getter));
    }
}

/// <https://tc39.es/ecma262/#sec-isaccessordescriptor>
fn is_accessor_descriptor(d: &PropertyDescriptor) -> bool {
    d.attrs & (jsapi::JSPROP_GETTER as u32 | jsapi::JSPROP_SETTER as u32) != 0
}

/// <https://tc39.es/ecma262/#sec-isdatadescriptor>
fn is_data_descriptor(d: &PropertyDescriptor) -> bool {
    let is_accessor = is_accessor_descriptor(d);
    let is_generic = d.attrs &
        (jsapi::JSPROP_GETTER as u32 |
            jsapi::JSPROP_SETTER as u32 |
            jsapi::JSPROP_IGNORE_READONLY |
            jsapi::JSPROP_IGNORE_VALUE) ==
        jsapi::JSPROP_IGNORE_READONLY | jsapi::JSPROP_IGNORE_VALUE;
    !is_accessor && !is_generic
}

/// Evaluate `CrossOriginGetOwnPropertyHelper(proxy, id) != null`.
pub unsafe fn cross_origin_has_own(
    cx: SafeJSContext,
    _proxy: RawHandleObject,
    cross_origin_properties: &'static CrossOriginProperties,
    id: RawHandleId,
    bp: *mut bool,
) -> bool {
    // TODO: Once we have the slot for the holder, it'd be more efficient to
    //       use `ensure_cross_origin_property_holder`.
    *bp = if let Some(key) =
        crate::dom::bindings::conversions::jsid_to_string(*cx, Handle::from_raw(id))
    {
        cross_origin_properties.keys().any(|defined_key| {
            let defined_key = CStr::from_ptr(defined_key);
            defined_key.to_bytes() == key.as_bytes()
        })
    } else {
        false
    };

    true
}

/// Implementation of [`CrossOriginGetOwnPropertyHelper`].
///
/// `cx` and `obj` are expected to be different-Realm here. `obj` can be a
/// `WindowProxy` or a `Location` or a `DissimilarOrigin*` proxy for one of
/// those.
///
/// [`CrossOriginGetOwnPropertyHelper`]: https://html.spec.whatwg.org/multipage/#crossorigingetownpropertyhelper-(-o,-p-)
pub unsafe fn cross_origin_get_own_property_helper(
    cx: SafeJSContext,
    proxy: RawHandleObject,
    cross_origin_properties: &'static CrossOriginProperties,
    id: RawHandleId,
    mut desc: RawMutableHandle<PropertyDescriptor>,
) -> bool {
    rooted!(in(*cx) let mut holder = ptr::null_mut::<JSObject>());

    ensure_cross_origin_property_holder(
        cx,
        proxy,
        cross_origin_properties,
        holder.handle_mut().into(),
    );

    if !JS_GetOwnPropertyDescriptorById(*cx, holder.handle().into(), id, desc) {
        return false;
    }

    if !desc.obj.is_null() {
        desc.obj = proxy.get();
    }

    true
}

/// Implementation of [`CrossOriginPropertyFallback`].
///

/// [`CrossOriginPropertyFallback`]: https://html.spec.whatwg.org/multipage/#crossoriginpropertyfallback-(-p-)
pub unsafe fn cross_origin_property_fallback(
    cx: SafeJSContext,
    proxy: RawHandleObject,
    id: RawHandleId,
    mut desc: RawMutableHandle<PropertyDescriptor>,
) -> bool {
    assert!(desc.obj.is_null(), "why are we being called?");

    // > 1. If P is `then`, `@@toStringTag`, `@@hasInstance`, or
    // >    `@@isConcatSpreadable`, then return `PropertyDescriptor{ [[Value]]:
    // >    undefined, [[Writable]]: false, [[Enumerable]]: false,
    // >    [[Configurable]]: true }`.
    if is_cross_origin_allowlisted_prop(cx, id) {
        *desc = PropertyDescriptor {
            getter: None,
            setter: None,
            value: UndefinedValue(),
            attrs: jsapi::JSPROP_READONLY as u32,
            obj: proxy.get(),
        };
        return true;
    }

    // > 2. Throw a `SecurityError` `DOMException`.
    report_cross_origin_denial(cx, id, "access")
}

unsafe fn is_cross_origin_allowlisted_prop(cx: SafeJSContext, id: RawHandleId) -> bool {
    const ALLOWLISTED_SYMBOL_CODES: &[SymbolCode] = &[
        SymbolCode::toStringTag,
        SymbolCode::hasInstance,
        SymbolCode::isConcatSpreadable,
    ];

    crate::dom::bindings::conversions::jsid_to_string(*cx, Handle::from_raw(id))
        .filter(|st| st == "then")
        .is_some() ||
        {
            rooted!(in(*cx) let mut allowed_id: jsid);
            ALLOWLISTED_SYMBOL_CODES.iter().any(|&allowed_code| {
                RUST_SYMBOL_TO_JSID(
                    GetWellKnownSymbol(*cx, allowed_code),
                    allowed_id.handle_mut().into(),
                );
                // `jsid`s containing `JS::Symbol *` can be compared by
                // referential equality
                allowed_id.get().asBits == id.asBits
            })
        }
}

/// Get the holder for cross-origin properties for the current global of the
/// `JSContext`, creating one and storing it in a slot of the proxy object if it
/// doesn't exist yet.
///
/// This essentially creates a cache of [`CrossOriginGetOwnPropertyHelper`]'s
/// results for all property keys.
///
/// [`CrossOriginGetOwnPropertyHelper`]: https://html.spec.whatwg.org/multipage/#crossorigingetownpropertyhelper-(-o,-p-)
unsafe fn ensure_cross_origin_property_holder(
    cx: SafeJSContext,
    _proxy: RawHandleObject,
    cross_origin_properties: &'static CrossOriginProperties,
    out_holder: RawMutableHandleObject,
) -> bool {
    // TODO: We don't have the slot to store the holder yet. For now,
    //       the holder is constructed every time this function is called,
    //       which is not only inefficient but also deviates from the
    //       specification in a subtle yet observable way.

    // Create a holder for the current Realm
    out_holder.set(jsapi::JS_NewObjectWithGivenProto(
        *cx,
        ptr::null_mut(),
        RawHandleObject::null(),
    ));

    if out_holder.get().is_null() ||
        !jsapi::JS_DefineProperties(
            *cx,
            out_holder.handle(),
            cross_origin_properties.attributes.as_ptr(),
        ) ||
        !jsapi::JS_DefineFunctions(
            *cx,
            out_holder.handle(),
            cross_origin_properties.methods.as_ptr(),
        )
    {
        return false;
    }

    // TODO: Store the holder in the slot that we don't have yet.

    true
}
