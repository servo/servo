/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Utilities for the implementation of JSAPI proxy handlers.

use std::ffi::CStr;
use std::os::raw::c_char;
use std::ptr;

use js::conversions::ToJSValConvertible;
use js::glue::{
    GetProxyHandler, GetProxyHandlerFamily, GetProxyPrivate, InvokeGetOwnPropertyDescriptor,
    SetProxyPrivate,
};
use js::jsapi::{
    DOMProxyShadowsResult, GetStaticPrototype, GetWellKnownSymbol, Handle as RawHandle,
    HandleId as RawHandleId, HandleObject as RawHandleObject, HandleValue as RawHandleValue,
    JS_AtomizeAndPinString, JS_DefinePropertyById, JS_GetOwnPropertyDescriptorById,
    JS_IsExceptionPending, JSAutoRealm, JSContext, JSErrNum, JSFunctionSpec, JSObject,
    JSPropertySpec, MutableHandle as RawMutableHandle,
    MutableHandleIdVector as RawMutableHandleIdVector,
    MutableHandleObject as RawMutableHandleObject, MutableHandleValue as RawMutableHandleValue,
    ObjectOpResult, PropertyDescriptor, SetDOMProxyInformation, SymbolCode, jsid,
};
use js::jsid::SymbolId;
use js::jsval::{ObjectValue, UndefinedValue};
use js::rust::wrappers::{
    AppendToIdVector, JS_AlreadyHasOwnPropertyById, JS_NewObjectWithGivenProto,
    RUST_INTERNED_STRING_TO_JSID, SetDataPropertyDescriptor,
};
use js::rust::{Handle, HandleObject, HandleValue, MutableHandle, MutableHandleObject};
use js::{jsapi, rooted};

use crate::DomTypes;
use crate::conversions::{is_dom_proxy, jsid_to_string, jsstring_to_str};
use crate::error::Error;
use crate::interfaces::{DomHelpers, GlobalScopeHelpers};
use crate::realms::{AlreadyInRealm, InRealm};
use crate::reflector::DomObject;
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};
use crate::str::DOMString;
use crate::utils::delete_property_by_id;

/// Determine if this id shadows any existing properties for this proxy.
///
/// # Safety
/// `cx` must point to a valid, non-null JSContext.
pub(crate) unsafe extern "C" fn shadow_check_callback(
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
pub fn init() {
    unsafe {
        SetDOMProxyInformation(
            GetProxyHandlerFamily(),
            Some(shadow_check_callback),
            ptr::null(),
        );
    }
}

/// Defines an expando on the given `proxy`.
///
/// # Safety
/// `cx` must point to a valid, non-null JSContext.
/// `result` must point to a valid, non-null ObjectOpResult.
pub(crate) unsafe extern "C" fn define_property(
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
///
/// # Safety
/// `cx` must point to a valid, non-null JSContext.
/// `bp` must point to a valid, non-null ObjectOpResult.
pub(crate) unsafe extern "C" fn delete(
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
///
/// # Safety
/// `result` must point to a valid, non-null ObjectOpResult.
pub(crate) unsafe extern "C" fn prevent_extensions(
    _cx: *mut JSContext,
    _proxy: RawHandleObject,
    result: *mut ObjectOpResult,
) -> bool {
    (*result).code_ = JSErrNum::JSMSG_CANT_PREVENT_EXTENSIONS as ::libc::uintptr_t;
    true
}

/// Reports whether the object is Extensible
///
/// # Safety
/// `succeeded` must point to a valid, non-null bool.
pub(crate) unsafe extern "C" fn is_extensible(
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
/// necessary for maybe-cross-origin objects.
///
/// # Safety
/// `is_ordinary` must point to a valid, non-null bool.
pub(crate) unsafe extern "C" fn get_prototype_if_ordinary(
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
pub(crate) fn get_expando_object(obj: RawHandleObject, mut expando: MutableHandleObject) {
    unsafe {
        assert!(is_dom_proxy(obj.get()));
        let val = &mut UndefinedValue();
        GetProxyPrivate(obj.get(), val);
        expando.set(if val.is_undefined() {
            ptr::null_mut()
        } else {
            val.to_object()
        });
    }
}

/// Get the expando object, or create it if it doesn't exist yet.
/// Fails on JSAPI failure.
///
/// # Safety
/// `cx` must point to a valid, non-null JSContext.
pub(crate) unsafe fn ensure_expando_object(
    cx: *mut JSContext,
    obj: RawHandleObject,
    mut expando: MutableHandleObject,
) {
    assert!(is_dom_proxy(obj.get()));
    get_expando_object(obj, expando.reborrow());
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
pub fn set_property_descriptor(
    desc: MutableHandle<PropertyDescriptor>,
    value: HandleValue,
    attrs: u32,
    is_none: &mut bool,
) {
    unsafe {
        SetDataPropertyDescriptor(desc, value, attrs);
    }
    *is_none = false;
}

pub(crate) fn id_to_source(cx: SafeJSContext, id: RawHandleId) -> Option<DOMString> {
    unsafe {
        rooted!(in(*cx) let mut value = UndefinedValue());
        rooted!(in(*cx) let mut jsstr = ptr::null_mut::<jsapi::JSString>());
        jsapi::JS_IdToValue(*cx, id.get(), value.handle_mut().into())
            .then(|| {
                jsstr.set(jsapi::JS_ValueToSource(*cx, value.handle().into()));
                jsstr.get()
            })
            .and_then(ptr::NonNull::new)
            .map(|jsstr| jsstring_to_str(*cx, jsstr))
    }
}

/// Property and method specs that correspond to the elements of
/// [`CrossOriginProperties(O)`].
///
/// [`CrossOriginProperties(O)`]: https://html.spec.whatwg.org/multipage/#crossoriginproperties-(-o-)
pub(crate) struct CrossOriginProperties {
    pub(crate) attributes: &'static [JSPropertySpec],
    pub(crate) methods: &'static [JSFunctionSpec],
}

impl CrossOriginProperties {
    /// Enumerate the property keys defined by `self`.
    fn keys(&self) -> impl Iterator<Item = *const c_char> + '_ {
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
pub(crate) fn cross_origin_own_property_keys(
    cx: SafeJSContext,
    _proxy: RawHandleObject,
    cross_origin_properties: &'static CrossOriginProperties,
    props: RawMutableHandleIdVector,
) -> bool {
    // > 2. For each `e` of `! CrossOriginProperties(O)`, append
    // >    `e.[[Property]]` to `keys`.
    for key in cross_origin_properties.keys() {
        unsafe {
            rooted!(in(*cx) let rooted = JS_AtomizeAndPinString(*cx, key));
            rooted!(in(*cx) let mut rooted_jsid: jsid);
            RUST_INTERNED_STRING_TO_JSID(*cx, rooted.handle().get(), rooted_jsid.handle_mut());
            AppendToIdVector(props, rooted_jsid.handle());
        }
    }

    // > 3. Return the concatenation of `keys` and `« "then", @@toStringTag,
    // > @@hasInstance, @@isConcatSpreadable »`.
    append_cross_origin_allowlisted_prop_keys(cx, props);

    true
}

/// # Safety
/// `is_ordinary` must point to a valid, non-null bool.
pub(crate) unsafe extern "C" fn maybe_cross_origin_get_prototype_if_ordinary_rawcx(
    _: *mut JSContext,
    _proxy: RawHandleObject,
    is_ordinary: *mut bool,
    _proto: RawMutableHandleObject,
) -> bool {
    // We have a custom `[[GetPrototypeOf]]`, so return `false`
    *is_ordinary = false;
    true
}

/// Implementation of `[[SetPrototypeOf]]` for [`Location`] and [`WindowProxy`].
///
/// [`Location`]: https://html.spec.whatwg.org/multipage/#location-setprototypeof
/// [`WindowProxy`]: https://html.spec.whatwg.org/multipage/#windowproxy-setprototypeof
///
/// # Safety
/// `result` must point to a valid, non-null ObjectOpResult.
pub(crate) unsafe extern "C" fn maybe_cross_origin_set_prototype_rawcx(
    cx: *mut JSContext,
    proxy: RawHandleObject,
    proto: RawHandleObject,
    result: *mut ObjectOpResult,
) -> bool {
    // > 1. Return `! SetImmutablePrototype(this, V)`.
    //
    // <https://tc39.es/ecma262/#sec-set-immutable-prototype>:
    //
    // > 1. Assert: Either `Type(V)` is Object or `Type(V)` is Null.
    //
    // > 2. Let current be `? O.[[GetPrototypeOf]]()`.
    rooted!(in(cx) let mut current = ptr::null_mut::<JSObject>());
    if !jsapi::GetObjectProto(cx, proxy, current.handle_mut().into()) {
        return false;
    }

    // > 3. If `SameValue(V, current)` is true, return true.
    if proto.get() == current.get() {
        (*result).code_ = 0 /* OkCode */;
        return true;
    }

    // > 4. Return false.
    (*result).code_ = JSErrNum::JSMSG_CANT_SET_PROTO as usize;
    true
}

pub(crate) fn get_getter_object(d: &PropertyDescriptor, out: RawMutableHandleObject) {
    if d.hasGetter_() {
        out.set(d.getter_);
    }
}

pub(crate) fn get_setter_object(d: &PropertyDescriptor, out: RawMutableHandleObject) {
    if d.hasSetter_() {
        out.set(d.setter_);
    }
}

/// <https://tc39.es/ecma262/#sec-isaccessordescriptor>
pub(crate) fn is_accessor_descriptor(d: &PropertyDescriptor) -> bool {
    d.hasSetter_() || d.hasGetter_()
}

/// <https://tc39.es/ecma262/#sec-isdatadescriptor>
pub(crate) fn is_data_descriptor(d: &PropertyDescriptor) -> bool {
    d.hasWritable_() || d.hasValue_()
}

/// Evaluate `CrossOriginGetOwnPropertyHelper(proxy, id) != null`.
/// SpiderMonkey-specific.
///
/// `cx` and `proxy` are expected to be different-Realm here. `proxy` is a proxy
/// for a maybe-cross-origin object.
///
/// # Safety
/// `bp` must point to a valid, non-null bool.
pub(crate) unsafe fn cross_origin_has_own(
    cx: SafeJSContext,
    _proxy: RawHandleObject,
    cross_origin_properties: &'static CrossOriginProperties,
    id: RawHandleId,
    bp: *mut bool,
) -> bool {
    // TODO: Once we have the slot for the holder, it'd be more efficient to
    //       use `ensure_cross_origin_property_holder`. We'll need `_proxy` to
    //       do that.
    *bp = jsid_to_string(*cx, Handle::from_raw(id)).is_some_and(|key| {
        cross_origin_properties.keys().any(|defined_key| {
            let defined_key = CStr::from_ptr(defined_key);
            defined_key.to_bytes() == key.as_bytes()
        })
    });

    true
}

/// Implementation of [`CrossOriginGetOwnPropertyHelper`].
///
/// `cx` and `proxy` are expected to be different-Realm here. `proxy` is a proxy
/// for a maybe-cross-origin object.
///
/// [`CrossOriginGetOwnPropertyHelper`]: https://html.spec.whatwg.org/multipage/#crossorigingetownpropertyhelper-(-o,-p-)
pub(crate) fn cross_origin_get_own_property_helper(
    cx: SafeJSContext,
    proxy: RawHandleObject,
    cross_origin_properties: &'static CrossOriginProperties,
    id: RawHandleId,
    desc: RawMutableHandle<PropertyDescriptor>,
    is_none: &mut bool,
) -> bool {
    rooted!(in(*cx) let mut holder = ptr::null_mut::<JSObject>());

    ensure_cross_origin_property_holder(
        cx,
        proxy,
        cross_origin_properties,
        holder.handle_mut().into(),
    );

    unsafe { JS_GetOwnPropertyDescriptorById(*cx, holder.handle().into(), id, desc, is_none) }
}

const ALLOWLISTED_SYMBOL_CODES: &[SymbolCode] = &[
    SymbolCode::toStringTag,
    SymbolCode::hasInstance,
    SymbolCode::isConcatSpreadable,
];

pub(crate) fn is_cross_origin_allowlisted_prop(cx: SafeJSContext, id: RawHandleId) -> bool {
    unsafe {
        if jsid_to_string(*cx, Handle::from_raw(id)).is_some_and(|st| st == "then") {
            return true;
        }

        rooted!(in(*cx) let mut allowed_id: jsid);
        ALLOWLISTED_SYMBOL_CODES.iter().any(|&allowed_code| {
            allowed_id.set(SymbolId(GetWellKnownSymbol(*cx, allowed_code)));
            // `jsid`s containing `JS::Symbol *` can be compared by
            // referential equality
            allowed_id.get().asBits_ == id.asBits_
        })
    }
}

/// Append `« "then", @@toStringTag, @@hasInstance, @@isConcatSpreadable »` to
/// `props`. This is used to implement [`CrossOriginOwnPropertyKeys`].
///
/// [`CrossOriginOwnPropertyKeys`]: https://html.spec.whatwg.org/multipage/#crossoriginownpropertykeys-(-o-)
fn append_cross_origin_allowlisted_prop_keys(cx: SafeJSContext, props: RawMutableHandleIdVector) {
    unsafe {
        rooted!(in(*cx) let mut id: jsid);

        let jsstring = JS_AtomizeAndPinString(*cx, c"then".as_ptr());
        rooted!(in(*cx) let rooted = jsstring);
        RUST_INTERNED_STRING_TO_JSID(*cx, rooted.handle().get(), id.handle_mut());
        AppendToIdVector(props, id.handle());

        for &allowed_code in ALLOWLISTED_SYMBOL_CODES.iter() {
            id.set(SymbolId(GetWellKnownSymbol(*cx, allowed_code)));
            AppendToIdVector(props, id.handle());
        }
    }
}

/// Get the holder for cross-origin properties for the current global of the
/// `JSContext`, creating one and storing it in a slot of the proxy object if it
/// doesn't exist yet.
///
/// This essentially creates a cache of [`CrossOriginGetOwnPropertyHelper`]'s
/// results for all property keys.
///
/// `cx` and `proxy` are expected to be different-Realm here. `proxy` is a proxy
/// for a maybe-cross-origin object. The `out_holder` return value will always
/// be in the Realm of `cx`.
///
/// [`CrossOriginGetOwnPropertyHelper`]: https://html.spec.whatwg.org/multipage/#crossorigingetownpropertyhelper-(-o,-p-)
fn ensure_cross_origin_property_holder(
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
    unsafe {
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
    }

    // TODO: Store the holder in the slot that we don't have yet.

    true
}

/// Report a cross-origin denial for a property, Always returns `false`, so it
/// can be used as `return report_cross_origin_denial(...);`.
///
/// What this function does corresponds to the operations in
/// <https://html.spec.whatwg.org/multipage/#the-location-interface> denoted as
/// "Throw a `SecurityError` DOMException".
pub(crate) unsafe fn report_cross_origin_denial<D: DomTypes>(
    cx: SafeJSContext,
    id: RawHandleId,
    access: &str,
) -> bool {
    debug!(
        "permission denied to {} property {} on cross-origin object",
        access,
        id_to_source(cx, id).as_deref().unwrap_or("< error >"),
    );
    let in_realm_proof = AlreadyInRealm::assert_for_cx(cx);
    if !JS_IsExceptionPending(*cx) {
        let global = D::GlobalScope::from_context(*cx, InRealm::Already(&in_realm_proof));
        // TODO: include `id` and `access` in the exception message
        <D as DomHelpers<D>>::throw_dom_exception(cx, &global, Error::Security, CanGc::note());
    }
    false
}

/// Implementation of `[[Set]]` for [`Location`].
///
/// [`Location`]: https://html.spec.whatwg.org/multipage/#location-set
pub(crate) unsafe extern "C" fn maybe_cross_origin_set_rawcx<D: DomTypes>(
    cx: *mut JSContext,
    proxy: RawHandleObject,
    id: RawHandleId,
    v: RawHandleValue,
    receiver: RawHandleValue,
    result: *mut ObjectOpResult,
) -> bool {
    let cx = SafeJSContext::from_ptr(cx);

    if !<D as DomHelpers<D>>::is_platform_object_same_origin(cx, proxy) {
        return cross_origin_set::<D>(cx, proxy, id, v, receiver, result);
    }

    // Safe to enter the Realm of proxy now.
    let _ac = JSAutoRealm::new(*cx, proxy.get());

    // OrdinarySet
    // <https://tc39.es/ecma262/#sec-ordinaryset>
    rooted!(in(*cx) let mut own_desc = PropertyDescriptor::default());
    let mut is_none = false;
    if !InvokeGetOwnPropertyDescriptor(
        GetProxyHandler(*proxy),
        *cx,
        proxy,
        id,
        own_desc.handle_mut().into(),
        &mut is_none,
    ) {
        return false;
    }

    js::jsapi::SetPropertyIgnoringNamedGetter(
        *cx,
        proxy,
        id,
        v,
        receiver,
        own_desc.handle().into(),
        result,
    )
}

/// Implementation of `[[GetPrototypeOf]]` for [`Location`].
///
/// [`Location`]: https://html.spec.whatwg.org/multipage/#location-getprototypeof
pub(crate) unsafe fn maybe_cross_origin_get_prototype<D: DomTypes>(
    cx: SafeJSContext,
    proxy: RawHandleObject,
    get_proto_object: unsafe fn(cx: SafeJSContext, global: HandleObject, rval: MutableHandleObject),
    proto: RawMutableHandleObject,
) -> bool {
    // > 1. If ! IsPlatformObjectSameOrigin(this) is true, then return ! OrdinaryGetPrototypeOf(this).
    if <D as DomHelpers<D>>::is_platform_object_same_origin(cx, proxy) {
        let ac = JSAutoRealm::new(*cx, proxy.get());
        let global = D::GlobalScope::from_context(*cx, InRealm::Entered(&ac));
        get_proto_object(
            cx,
            global.reflector().get_jsobject(),
            MutableHandleObject::from_raw(proto),
        );
        return !proto.is_null();
    }

    // > 2. Return null.
    proto.set(ptr::null_mut());
    true
}

/// Implementation of [`CrossOriginGet`].
///
/// `cx` and `proxy` are expected to be different-Realm here. `proxy` is a proxy
/// for a maybe-cross-origin object.
///
/// [`CrossOriginGet`]: https://html.spec.whatwg.org/multipage/#crossoriginget-(-o,-p,-receiver-)
pub(crate) unsafe fn cross_origin_get<D: DomTypes>(
    cx: SafeJSContext,
    proxy: RawHandleObject,
    receiver: RawHandleValue,
    id: RawHandleId,
    vp: RawMutableHandleValue,
) -> bool {
    // > 1. Let `desc` be `? O.[[GetOwnProperty]](P)`.
    rooted!(in(*cx) let mut descriptor = PropertyDescriptor::default());
    let mut is_none = false;
    if !InvokeGetOwnPropertyDescriptor(
        GetProxyHandler(*proxy),
        *cx,
        proxy,
        id,
        descriptor.handle_mut().into(),
        &mut is_none,
    ) {
        return false;
    }

    // > 2. Assert: `desc` is not undefined.
    assert!(
        !is_none,
        "Callees should throw in all cases when they are not finding \
        a property decriptor"
    );

    // > 3. If `! IsDataDescriptor(desc)` is true, then return `desc.[[Value]]`.
    if is_data_descriptor(&descriptor) {
        vp.set(descriptor.value_);
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
        return report_cross_origin_denial::<D>(cx, id, "get");
    }

    rooted!(in(*cx) let mut getter_jsval = UndefinedValue());
    getter.get().to_jsval(*cx, getter_jsval.handle_mut());

    // > 7. Return `? Call(getter, Receiver)`.
    jsapi::Call(
        *cx,
        receiver,
        getter_jsval.handle().into(),
        &jsapi::HandleValueArray::empty(),
        vp,
    )
}

/// Implementation of [`CrossOriginSet`].
///
/// `cx` and `proxy` are expected to be different-Realm here. `proxy` is a proxy
/// for a maybe-cross-origin object.
///
/// [`CrossOriginSet`]: https://html.spec.whatwg.org/multipage/#crossoriginset-(-o,-p,-v,-receiver-)
pub(crate) unsafe fn cross_origin_set<D: DomTypes>(
    cx: SafeJSContext,
    proxy: RawHandleObject,
    id: RawHandleId,
    v: RawHandleValue,
    receiver: RawHandleValue,
    result: *mut ObjectOpResult,
) -> bool {
    // > 1. Let desc be ? O.[[GetOwnProperty]](P).
    rooted!(in(*cx) let mut descriptor = PropertyDescriptor::default());
    let mut is_none = false;
    if !InvokeGetOwnPropertyDescriptor(
        GetProxyHandler(*proxy),
        *cx,
        proxy,
        id,
        descriptor.handle_mut().into(),
        &mut is_none,
    ) {
        return false;
    }

    // > 2. Assert: desc is not undefined.
    assert!(
        !is_none,
        "Callees should throw in all cases when they are not finding \
        a property decriptor"
    );

    // > 3. If desc.[[Set]] is present and its value is not undefined,
    // >    then: [...]
    rooted!(in(*cx) let mut setter = ptr::null_mut::<JSObject>());
    get_setter_object(&descriptor, setter.handle_mut().into());
    if setter.get().is_null() {
        // > 4. Throw a "SecurityError" DOMException.
        return report_cross_origin_denial::<D>(cx, id, "set");
    }

    rooted!(in(*cx) let mut setter_jsval = UndefinedValue());
    setter.get().to_jsval(*cx, setter_jsval.handle_mut());

    // > 3.1. Perform ? Call(setter, Receiver, «V»).
    // >
    // > 3.2. Return true.
    rooted!(in(*cx) let mut ignored = UndefinedValue());
    if !jsapi::Call(
        *cx,
        receiver,
        setter_jsval.handle().into(),
        // FIXME: Our binding lacks `HandleValueArray(Handle<Value>)`
        // <https://searchfox.org/mozilla-central/rev/072710086ddfe25aa2962c8399fefb2304e8193b/js/public/ValueArray.h#54-55>
        &jsapi::HandleValueArray {
            length_: 1,
            elements_: v.ptr,
        },
        ignored.handle_mut().into(),
    ) {
        return false;
    }

    (*result).code_ = 0 /* OkCode */;
    true
}

/// Implementation of [`CrossOriginPropertyFallback`].
///
/// `cx` and `proxy` are expected to be different-Realm here. `proxy` is a proxy
/// for a maybe-cross-origin object.
///
/// [`CrossOriginPropertyFallback`]: https://html.spec.whatwg.org/multipage/#crossoriginpropertyfallback-(-p-)
pub(crate) unsafe fn cross_origin_property_fallback<D: DomTypes>(
    cx: SafeJSContext,
    _proxy: RawHandleObject,
    id: RawHandleId,
    desc: RawMutableHandle<PropertyDescriptor>,
    is_none: &mut bool,
) -> bool {
    assert!(*is_none, "why are we being called?");

    // > 1. If P is `then`, `@@toStringTag`, `@@hasInstance`, or
    // >    `@@isConcatSpreadable`, then return `PropertyDescriptor{ [[Value]]:
    // >    undefined, [[Writable]]: false, [[Enumerable]]: false,
    // >    [[Configurable]]: true }`.
    if is_cross_origin_allowlisted_prop(cx, id) {
        set_property_descriptor(
            MutableHandle::from_raw(desc),
            HandleValue::undefined(),
            jsapi::JSPROP_READONLY as u32,
            is_none,
        );
        return true;
    }

    // > 2. Throw a `SecurityError` `DOMException`.
    report_cross_origin_denial::<D>(cx, id, "access")
}
