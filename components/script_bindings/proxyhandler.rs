/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Utilities for the implementation of JSAPI proxy handlers.

use std::ffi::CString;
use std::ops::{Deref, DerefMut};
use std::os::raw::c_char;
use std::ptr;
use std::ptr::NonNull;

use js::context::{JSContext, RawJSContext};
use js::conversions::{ToJSValConvertible, jsstr_to_string};
use js::glue::{
    GetProxyHandlerFamily, GetProxyPrivate, GetProxyReservedSlot, SetProxyPrivate,
    SetProxyReservedSlot, UncheckedUnwrapObject,
};
use js::jsapi::{
    DOMProxyShadowsResult, GetObjectRealmOrNull, GetRealmPrincipals, GetStaticPrototype,
    Handle as RawHandle, HandleId as RawHandleId, HandleObject as RawHandleObject,
    HandleValue as RawHandleValue, HandleValueArray, IsWeakMapObject, IsWindowProxy,
    JS_IsDeadWrapper, JSErrNum, JSFunctionSpec, JSITER_HIDDEN, JSITER_OWNONLY, JSITER_SYMBOLS,
    JSObject, JSPROP_READONLY, JSPropertySpec, JSString,
    MutableHandleIdVector as RawMutableHandleIdVector,
    MutableHandleObject as RawMutableHandleObject, ObjectOpResult, PropertyDescriptor,
    SetDOMProxyInformation, SymbolCode, jsid,
};
use js::jsid::SymbolId;
use js::jsval::{ObjectValue, UndefinedValue};
use js::realm::{AutoRealm, CurrentRealm};
use js::rust::wrappers2::{
    AppendToIdVector, Call, GetObjectProto, GetPropertyKeys, GetRealmKeyObject, GetWeakMapEntry,
    GetWellKnownSymbol, JS_AlreadyHasOwnPropertyById, JS_AtomizeAndPinString, JS_DefineFunctions,
    JS_DefineProperties, JS_DefinePropertyById, JS_DeletePropertyById,
    JS_GetOwnPropertyDescriptorById, JS_IdToValue, JS_IsExceptionPending,
    JS_NewObjectWithGivenProto, JS_ValueToSource, JS_WrapObject, JS_WrapValue, NewWeakMapObject,
    RUST_INTERNED_STRING_TO_JSID, RUST_JSID_IS_VOID, SetDataPropertyDescriptor,
    SetPropertyIgnoringNamedGetter, SetWeakMapEntry, int_to_jsid,
};
use js::rust::{
    Handle, HandleId, HandleObject, HandleValue, IntoHandle, MutableHandle, MutableHandleObject,
    MutableHandleValue,
};

use crate::DomTypes;
use crate::conversions::{is_dom_proxy, jsid_to_string, native_from_object};
use crate::error::Error;
use crate::interfaces::{DomHelpers, GlobalScopeHelpers};
use crate::principals::ServoJSPrincipalsRef;
use crate::reflector::DomObject;
use crate::str::DOMString;

/// In WindowProxy and ProxyObject, the second slot is reserved for the
/// cross-origin property holder weak map. This is a weak map between the realm
/// and a property holder which stores objects that are used in cross origin
/// realms.
pub const CROSS_ORIGIN_PROPERTY_HOLDER_WEAK_MAP_SLOT: u32 = 1;

/// Determine if this id shadows any existing properties for this proxy.
///
/// # Safety
/// `cx` must point to a valid, non-null JSContext.
pub(crate) unsafe extern "C" fn shadow_check_callback(
    cx: *mut RawJSContext,
    object: RawHandleObject,
    id: RawHandleId,
) -> DOMProxyShadowsResult {
    // TODO: support OverrideBuiltins when #12978 is fixed.

    // SAFETY: it is safe to construct a JSContext from engine hook.
    let mut cx = unsafe { JSContext::from_ptr(NonNull::new(cx).unwrap()) };
    let cx = &mut cx;

    let object = unsafe { HandleObject::from_raw(object) };
    rooted!(&in(cx) let mut expando = ptr::null_mut::<JSObject>());
    get_expando_object(object, expando.handle_mut());
    if !expando.get().is_null() {
        let mut has_own = false;
        let raw_id = unsafe { Handle::from_raw(id) };

        if !unsafe { JS_AlreadyHasOwnPropertyById(cx, expando.handle(), raw_id, &mut has_own) } {
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
    cx: *mut RawJSContext,
    proxy: RawHandleObject,
    id: RawHandleId,
    desc: RawHandle<PropertyDescriptor>,
    result: *mut ObjectOpResult,
) -> bool {
    // SAFETY: it is safe to construct a JSContext from engine hook.
    let mut cx = unsafe { JSContext::from_ptr(NonNull::new(cx).unwrap()) };
    let cx = &mut cx;

    let proxy = unsafe { Handle::from_raw(proxy) };
    let id = unsafe { Handle::from_raw(id) };
    let desc = unsafe { Handle::from_raw(desc) };

    rooted!(&in(cx) let mut expando = ptr::null_mut::<JSObject>());
    ensure_expando_object(cx, proxy, expando.handle_mut());

    unsafe { JS_DefinePropertyById(cx, expando.handle(), id, desc, result) }
}

/// Deletes an expando off the given `proxy`.
///
/// # Safety
/// `cx` must point to a valid, non-null JSContext.
/// `bp` must point to a valid, non-null ObjectOpResult.
pub(crate) unsafe extern "C" fn delete(
    cx: *mut RawJSContext,
    proxy: RawHandleObject,
    id: RawHandleId,
    bp: *mut ObjectOpResult,
) -> bool {
    // SAFETY: it is safe to construct a JSContext from engine hook.
    let mut cx = unsafe { JSContext::from_ptr(NonNull::new(cx).unwrap()) };
    let cx = &mut cx;

    let proxy = unsafe { Handle::from_raw(proxy) };
    let id = unsafe { Handle::from_raw(id) };

    rooted!(&in(cx) let mut expando = ptr::null_mut::<JSObject>());
    get_expando_object(proxy, expando.handle_mut());

    if expando.is_null() {
        unsafe {
            (*bp).code_ = 0 /* OkCode */
        };
        return true;
    }

    unsafe { JS_DeletePropertyById(cx, expando.handle(), id, bp) }
}

/// Controls whether the Extensible bit can be changed
///
/// [`Location`]: https://html.spec.whatwg.org/multipage/#location-preventextensions
/// [`WindowProxy`]: <https://html.spec.whatwg.org/multipage/#windowproxy-preventextensions>
///
/// # Safety
/// `result` must point to a valid, non-null ObjectOpResult.
pub unsafe extern "C" fn prevent_extensions(
    _cx: *mut RawJSContext,
    _proxy: RawHandleObject,
    result: *mut ObjectOpResult,
) -> bool {
    unsafe { (*result).code_ = JSErrNum::JSMSG_CANT_PREVENT_EXTENSIONS as ::libc::uintptr_t };
    true
}

/// Reports whether the object is Extensible
///
/// [`Location`]: https://html.spec.whatwg.org/multipage/#location-isextensible
/// [`WindowProxy`]: <https://html.spec.whatwg.org/multipage/#windowproxy-isextensible>
///
/// # Safety
/// `succeeded` must point to a valid, non-null bool.
pub unsafe extern "C" fn is_extensible(
    _cx: *mut RawJSContext,
    _proxy: RawHandleObject,
    succeeded: *mut bool,
) -> bool {
    unsafe { *succeeded = true };
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
    _: *mut RawJSContext,
    proxy: RawHandleObject,
    is_ordinary: *mut bool,
    proto: RawMutableHandleObject,
) -> bool {
    unsafe { *is_ordinary = true };
    proto.set(unsafe { GetStaticPrototype(proxy.get()) });
    true
}

/// Get the expando object, or null if there is none.
pub(crate) fn get_expando_object(obj: HandleObject, mut expando: MutableHandleObject) {
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
pub(crate) fn ensure_expando_object(
    cx: &mut JSContext,
    obj: HandleObject,
    mut expando: MutableHandleObject,
) {
    unsafe {
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
}

/// Set the property descriptor's object to `obj` and set it to enumerable,
/// and writable if `readonly` is true.
pub fn set_property_descriptor(
    desc: MutableHandle<PropertyDescriptor>,
    value: HandleValue,
    attrs: u32,
    is_none: &mut bool,
) {
    unsafe { SetDataPropertyDescriptor(desc, value, attrs) };
    *is_none = false;
}

fn id_to_source(cx: &mut JSContext, id: HandleId) -> Option<DOMString> {
    unsafe {
        if RUST_JSID_IS_VOID(id) {
            return None;
        }
        rooted!(&in(cx) let mut value = UndefinedValue());
        rooted!(&in(cx) let mut jsstr = ptr::null_mut::<JSString>());
        JS_IdToValue(cx, id.get(), value.handle_mut())
            .then(|| {
                jsstr.set(JS_ValueToSource(cx, value.handle()));
                jsstr.get()
            })
            .and_then(NonNull::new)
            .map(|jsstr| jsstr_to_string(cx, jsstr).into())
    }
}

/// Property and method specs that correspond to the elements of
/// [`CrossOriginProperties(O)`].
///
/// [`CrossOriginProperties(O)`]: https://html.spec.whatwg.org/multipage/#crossoriginproperties-(-o-)
pub struct CrossOriginProperties {
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
pub fn cross_origin_own_property_keys(
    cx: &mut JSContext,
    _proxy: HandleObject,
    cross_origin_properties: &'static CrossOriginProperties,
    props: RawMutableHandleIdVector,
) -> bool {
    // > 2. For each `e` of `! CrossOriginProperties(O)`, append
    // >    `e.[[Property]]` to `keys`.
    for key in cross_origin_properties.keys() {
        unsafe {
            rooted!(&in(cx) let rooted = JS_AtomizeAndPinString(cx, key));
            rooted!(&in(cx) let mut rooted_jsid: jsid);
            RUST_INTERNED_STRING_TO_JSID(cx, rooted.handle().get(), rooted_jsid.handle_mut());
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
pub unsafe extern "C" fn maybe_cross_origin_get_prototype_if_ordinary_rawcx(
    _: *mut RawJSContext,
    _proxy: RawHandleObject,
    is_ordinary: *mut bool,
    _proto: RawMutableHandleObject,
) -> bool {
    // We have a custom `[[GetPrototypeOf]]`, so return `false`
    unsafe { *is_ordinary = false };
    true
}

/// Implementation of `[[SetPrototypeOf]]` for [`Location`] and [`WindowProxy`].
///
/// [`Location`]: https://html.spec.whatwg.org/multipage/#location-setprototypeof
/// [`WindowProxy`]: https://html.spec.whatwg.org/multipage/#windowproxy-setprototypeof
///
/// # Safety
/// `result` must point to a valid, non-null ObjectOpResult.
pub unsafe extern "C" fn maybe_cross_origin_set_prototype_rawcx(
    cx: *mut RawJSContext,
    proxy: RawHandleObject,
    proto: RawHandleObject,
    result: *mut ObjectOpResult,
) -> bool {
    // SAFETY: it is safe to construct a JSContext from engine hook.
    let mut cx = unsafe { JSContext::from_ptr(NonNull::new(cx).unwrap()) };
    let cx = &mut cx;
    // > 1. Return `! SetImmutablePrototype(this, V)`.
    //
    // <https://tc39.es/ecma262/#sec-set-immutable-prototype>:
    //
    // > 1. Assert: Either `Type(V)` is Object or `Type(V)` is Null.
    //
    // > 2. Let current be `? O.[[GetPrototypeOf]]()`.
    rooted!(&in(cx) let mut current = ptr::null_mut::<JSObject>());
    if !unsafe { GetObjectProto(cx, Handle::from_raw(proxy), current.handle_mut()) } {
        return false;
    }

    // > 3. If `SameValue(V, current)` is true, return true.
    if proto.get() == current.get() {
        unsafe {
            (*result).code_ = 0 /* OkCode */
        };
        return true;
    }

    // > 4. Return false.
    unsafe { (*result).code_ = JSErrNum::JSMSG_CANT_SET_PROTO as usize };
    true
}

fn get_getter_object(d: &PropertyDescriptor, out: RawMutableHandleObject) {
    if d.hasGetter_() {
        out.set(d.getter_);
    }
}

fn get_setter_object(d: &PropertyDescriptor, out: RawMutableHandleObject) {
    if d.hasSetter_() {
        out.set(d.setter_);
    }
}

/// <https://tc39.es/ecma262/#sec-isaccessordescriptor>
fn is_accessor_descriptor(d: &PropertyDescriptor) -> bool {
    d.hasSetter_() || d.hasGetter_()
}

/// <https://tc39.es/ecma262/#sec-isdatadescriptor>
fn is_data_descriptor(d: &PropertyDescriptor) -> bool {
    d.hasWritable_() || d.hasValue_()
}

/// Evaluate whether proxy has the own property 'id' in the cross-origin case.
///
/// `cx` and `proxy` are expected to be different-Realm here. `proxy` is a proxy
/// for a maybe-cross-origin object.
pub(crate) fn cross_origin_has_own<D: DomTypes>(
    cx: &mut CurrentRealm,
    proxy: HandleObject,
    cross_origin_properties: &'static CrossOriginProperties,
    id: HandleId,
    exists: &mut bool,
) -> bool {
    rooted!(&in(cx) let mut property_descriptor = PropertyDescriptor::default());
    let mut is_none = false;
    if !cross_origin_get_own_property_helper(
        cx,
        proxy,
        cross_origin_properties,
        id,
        property_descriptor.handle_mut(),
        &mut is_none,
    ) {
        return false;
    }

    if is_none &&
        !cross_origin_property_fallback::<D>(
            cx,
            proxy,
            id,
            property_descriptor.handle_mut(),
            &mut is_none,
        )
    {
        return false;
    }

    *exists = !is_none;
    true
}

/// Implementation of [`CrossOriginGetOwnPropertyHelper`].
///
/// `cx` and `proxy` are expected to be different-Realm here. `proxy` is a proxy
/// for a maybe-cross-origin object.
///
/// [`CrossOriginGetOwnPropertyHelper`]: https://html.spec.whatwg.org/multipage/#crossorigingetownpropertyhelper-(-o,-p-)
pub fn cross_origin_get_own_property_helper(
    cx: &mut CurrentRealm,
    proxy: HandleObject,
    cross_origin_properties: &'static CrossOriginProperties,
    id: HandleId,
    desc: MutableHandle<PropertyDescriptor>,
    is_none: &mut bool,
) -> bool {
    rooted!(&in(cx) let mut holder = ptr::null_mut::<JSObject>());
    if !ensure_cross_origin_property_holder(cx, proxy, cross_origin_properties, holder.handle_mut())
    {
        return false;
    }
    unsafe { JS_GetOwnPropertyDescriptorById(cx, holder.handle(), id, desc, is_none) }
}

const ALLOWLISTED_SYMBOL_CODES: &[SymbolCode] = &[
    SymbolCode::toStringTag,
    SymbolCode::hasInstance,
    SymbolCode::isConcatSpreadable,
];

fn is_cross_origin_allowlisted_prop(cx: &mut JSContext, id: HandleId) -> bool {
    unsafe {
        if jsid_to_string(cx, id).is_some_and(|st| st == "then") {
            return true;
        }

        rooted!(&in(cx) let mut allowed_id: jsid);
        ALLOWLISTED_SYMBOL_CODES.iter().any(|&allowed_code| {
            allowed_id.set(SymbolId(GetWellKnownSymbol(cx, allowed_code)));
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
fn append_cross_origin_allowlisted_prop_keys(cx: &mut JSContext, props: RawMutableHandleIdVector) {
    unsafe {
        rooted!(&in(cx) let mut id: jsid);

        let jsstring = JS_AtomizeAndPinString(cx, c"then".as_ptr());
        rooted!(&in(cx) let rooted = jsstring);
        RUST_INTERNED_STRING_TO_JSID(cx, rooted.handle().get(), id.handle_mut());
        AppendToIdVector(props, id.handle());

        for &allowed_code in ALLOWLISTED_SYMBOL_CODES.iter() {
            id.set(SymbolId(GetWellKnownSymbol(cx, allowed_code)));
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
    cx: &mut CurrentRealm,
    proxy: HandleObject,
    cross_origin_properties: &'static CrossOriginProperties,
    mut out_holder: MutableHandleObject,
) -> bool {
    rooted!(&in(cx) let mut weak_value_map = UndefinedValue());
    unsafe {
        GetProxyReservedSlot(
            proxy.get(),
            CROSS_ORIGIN_PROPERTY_HOLDER_WEAK_MAP_SLOT,
            weak_value_map.as_ptr(),
        )
    };
    if weak_value_map.is_undefined() {
        let mut realm = AutoRealm::new_from_handle(cx, proxy);
        let new_map = unsafe { NewWeakMapObject(&mut realm) };
        if new_map.is_null() {
            return false;
        }
        weak_value_map.set(ObjectValue(new_map));
        unsafe {
            SetProxyReservedSlot(
                proxy.get(),
                CROSS_ORIGIN_PROPERTY_HOLDER_WEAK_MAP_SLOT,
                weak_value_map.as_ptr(),
            )
        };
    }

    rooted!(&in(cx) let map = weak_value_map.to_object());
    debug_assert!(unsafe { IsWeakMapObject(map.get()) });

    // We need to be in "map"'s compartment to work with it.  Per spec, the key
    // for this map is supposed to be the pair (current settings, relevant
    // settings).  The current settings corresponds to the current Realm of cx.
    // The relevant settings corresponds to the Realm of "obj", but since all of
    // our objects are per-Realm singletons, we are basically using "obj" itself
    // as part of the key.
    //
    // To represent the current settings, we use a dedicated key object of the
    // current-Realm.
    //
    // We can't use the current global, because we can't get a useful
    // cross-compartment wrapper for it; such wrappers would always go
    // through a WindowProxy and would not be guarantee to keep pointing to a
    // single Realm when unwrapped.  We want to grab this key before we start
    // changing Realms.
    //
    // Also we can't use arbitrary object (e.g.: Object.prototype), because at
    // this point those compartments are not same-origin, and don't have access to
    // each other, and the object retrieved here will be wrapped by a security
    // wrapper below, and the wrapper will be stored into the cache
    // (see Compartment::wrap).  Those compartments can get access later by
    // modifying `document.domain`, and wrapping objects after that point
    // shouldn't result in a security wrapper.  Wrap operation looks up the
    // existing wrapper in the cache, that contains the security wrapper created
    // here.  We should use unique/private object here, so that this doesn't
    // affect later wrap operation.
    rooted!(&in(cx) let mut key = unsafe { GetRealmKeyObject(cx) });
    if key.is_null() {
        return false;
    }

    rooted!(&in(cx) let mut holder_value = UndefinedValue());
    {
        let mut realm = AutoRealm::new_from_handle(cx, map.handle());
        let cx = &mut realm;

        if !unsafe { JS_WrapObject(cx, key.handle_mut()) } {
            return false;
        }

        rooted!(&in(cx) let key_value = ObjectValue(key.get()));
        if !unsafe {
            GetWeakMapEntry(
                cx,
                map.handle(),
                key_value.handle(),
                holder_value.handle_mut(),
            )
        } {
            return false;
        }
    }

    if holder_value.is_object() {
        // We want to do an unchecked unwrap, because the holder (and the current
        // caller) may actually be more privileged than our map.
        out_holder.set(unsafe { UncheckedUnwrapObject(holder_value.to_object(), true) });

        // holder might be a dead object proxy if things got nuked.
        if !unsafe { JS_IsDeadWrapper(out_holder.get()) } {
            return true;
        }
    }

    // We didn't find a usable holder. Go ahead and allocate one. At this point
    // we have two options: we could allocate the holder in the current Realm
    // and store a cross-compartment wrapper for it in the map as needed, or we
    // could allocate the holder in the Realm of the map and have it hold
    // cross-compartment references to all the methods it holds, since those
    // methods need to be in our current Realm. It seems better to allocate the
    // holder in our current Realm.
    out_holder
        .set(unsafe { JS_NewObjectWithGivenProto(cx, ptr::null_mut(), HandleObject::null()) });

    if out_holder.get().is_null() ||
        !unsafe {
            JS_DefineProperties(
                cx,
                out_holder.handle(),
                cross_origin_properties.attributes.as_ptr(),
            )
        } ||
        !unsafe {
            JS_DefineFunctions(
                cx,
                out_holder.handle(),
                cross_origin_properties.methods.as_ptr(),
            )
        }
    {
        return false;
    }

    holder_value.set(ObjectValue(out_holder.get()));

    {
        let mut realm = AutoRealm::new_from_handle(cx, map.handle());
        let cx = &mut realm;

        // Key is already in the right Realm, but we need to wrap the value.
        if !unsafe { JS_WrapValue(cx, holder_value.handle_mut()) } {
            return false;
        }

        rooted!(&in(cx) let key_value = ObjectValue(key.get()));
        if !unsafe { SetWeakMapEntry(cx, map.handle(), key_value.handle(), holder_value.handle()) }
        {
            return false;
        }
    }

    true
}

/// Check if `obj` is a `Location` or `Window` object.
///
/// IDL operations on a cross-origin object involve [a security check][1].
///
/// [1]: https://html.spec.whatwg.org/multipage/#integration-with-idl
pub(crate) fn is_cross_origin_object<D: DomTypes>(cx: &mut JSContext, obj: HandleObject) -> bool {
    unsafe { IsWindowProxy(*obj) || native_from_object::<D::Location>(cx, *obj).is_ok() }
}

/// Report a cross-origin denial for a property, Always returns `false`, so it
/// can be used as `return report_cross_origin_denial(...);`.
///
/// What this function does corresponds to the operations in
/// <https://html.spec.whatwg.org/multipage/#the-location-interface> denoted as
/// "Throw a `SecurityError` DOMException".
pub fn report_cross_origin_denial<D: DomTypes>(
    cx: &mut CurrentRealm,
    id: HandleId,
    access: &str,
) -> bool {
    if let Some(id) = id_to_source(cx, id) {
        debug!(
            "permission denied to {} property {} on cross-origin object",
            access,
            &*id.str(),
        );
    } else {
        debug!("permission denied to {} on cross-origin object", access);
    }
    unsafe {
        if !JS_IsExceptionPending(cx) {
            let global = D::GlobalScope::from_current_realm(cx);
            // TODO: include `id` and `access` in the exception message
            <D as DomHelpers<D>>::throw_dom_exception(cx, &global, Error::Security(None));
        }
    }
    false
}

/// Implementation of `[[Set]]` for [`Location`].
///
/// [`Location`]: https://html.spec.whatwg.org/multipage/#location-set
pub(crate) unsafe extern "C" fn maybe_cross_origin_set_rawcx<D: DomTypes>(
    cx: *mut RawJSContext,
    proxy: RawHandleObject,
    id: RawHandleId,
    v: RawHandleValue,
    receiver: RawHandleValue,
    result: *mut ObjectOpResult,
) -> bool {
    unsafe {
        // SAFETY: it is safe to construct a JSContext from engine hook.
        let mut cx = JSContext::from_ptr(NonNull::new(cx).unwrap());
        let mut realm = CurrentRealm::assert(&mut cx);
        let proxy = HandleObject::from_raw(proxy);
        let id = Handle::from_raw(id);
        let v = Handle::from_raw(v);
        let receiver = Handle::from_raw(receiver);

        if !is_platform_object_same_origin(&realm, proxy) {
            return cross_origin_set::<D>(&mut realm, proxy, id, v.into_handle(), receiver, result);
        }

        // Safe to enter the Realm of proxy now.
        let mut realm = AutoRealm::new_from_handle(&mut realm, proxy);

        // OrdinarySet
        // <https://tc39.es/ecma262/#sec-ordinaryset>
        rooted!(&in(&mut realm) let mut own_desc = PropertyDescriptor::default());
        let mut is_none = false;
        if !JS_GetOwnPropertyDescriptorById(
            &mut realm,
            proxy,
            id,
            own_desc.handle_mut(),
            &mut is_none,
        ) {
            return false;
        }

        SetPropertyIgnoringNamedGetter(
            &mut realm,
            proxy,
            id,
            v,
            receiver,
            if is_none {
                None
            } else {
                Some(own_desc.handle())
            },
            result,
        )
    }
}

/// Implementation of `[[GetPrototypeOf]]` for [`Location`].
///
/// [`Location`]: https://html.spec.whatwg.org/multipage/#location-getprototypeof
/// [`WindowProxy`]: https://html.spec.whatwg.org/multipage/#windowproxy-getprototypeof
pub fn maybe_cross_origin_get_prototype<D: DomTypes>(
    cx: &mut CurrentRealm,
    proxy: HandleObject,
    get_proto_object: fn(cx: &mut JSContext, global: HandleObject, rval: MutableHandleObject),
    mut proto: MutableHandleObject,
) -> bool {
    // > 1. If ! IsPlatformObjectSameOrigin(this) is true, then return ! OrdinaryGetPrototypeOf(this).
    if is_platform_object_same_origin(cx, proxy) {
        let mut realm = AutoRealm::new_from_handle(cx, proxy);
        let mut realm = realm.current_realm();
        let global = D::GlobalScope::from_current_realm(&mut realm);
        get_proto_object(
            &mut realm,
            global.reflector().get_jsobject(),
            proto.reborrow(),
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
pub fn cross_origin_get<D: DomTypes>(
    cx: &mut CurrentRealm,
    proxy: HandleObject,
    receiver: HandleValue,
    id: HandleId,
    mut vp: MutableHandleValue,
) -> bool {
    // > 1. Let `desc` be `? O.[[GetOwnProperty]](P)`.
    rooted!(&in(cx) let mut descriptor = PropertyDescriptor::default());
    let mut is_none = false;
    if !unsafe {
        JS_GetOwnPropertyDescriptorById(cx, proxy, id, descriptor.handle_mut(), &mut is_none)
    } {
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
    rooted!(&in(cx) let mut getter = ptr::null_mut::<JSObject>());
    get_getter_object(&descriptor, getter.handle_mut().into());
    if getter.get().is_null() {
        return report_cross_origin_denial::<D>(cx, id, "get");
    }

    rooted!(&in(cx) let mut getter_jsval = UndefinedValue());
    getter.get().to_jsval(cx, getter_jsval.handle_mut());

    // > 7. Return `? Call(getter, Receiver)`.
    unsafe {
        Call(
            cx,
            receiver,
            getter_jsval.handle(),
            &HandleValueArray::empty(),
            vp,
        )
    }
}

/// Implementation of [`CrossOriginSet`].
///
/// `cx` and `proxy` are expected to be different-Realm here. `proxy` is a proxy
/// for a maybe-cross-origin object.
///
/// [`CrossOriginSet`]: https://html.spec.whatwg.org/multipage/#crossoriginset-(-o,-p,-v,-receiver-)
///
/// # Safety
/// `result` must point to a valid, non-null `ObjectObResult`.
pub unsafe fn cross_origin_set<D: DomTypes>(
    cx: &mut CurrentRealm,
    proxy: HandleObject,
    id: HandleId,
    v: RawHandleValue,
    receiver: HandleValue,
    result: *mut ObjectOpResult,
) -> bool {
    // > 1. Let desc be ? O.[[GetOwnProperty]](P).
    rooted!(&in(cx) let mut descriptor = PropertyDescriptor::default());
    let mut is_none = false;
    if !unsafe {
        JS_GetOwnPropertyDescriptorById(cx, proxy, id, descriptor.handle_mut(), &mut is_none)
    } {
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
    rooted!(&in(cx) let mut setter = ptr::null_mut::<JSObject>());
    get_setter_object(&descriptor, setter.handle_mut().into());
    if setter.get().is_null() {
        // > 4. Throw a "SecurityError" DOMException.
        return report_cross_origin_denial::<D>(cx, id, "set");
    }

    rooted!(&in(cx) let mut setter_jsval = UndefinedValue());
    setter.get().to_jsval(cx, setter_jsval.handle_mut());

    // > 3.1. Perform ? Call(setter, Receiver, «V»).
    // >
    // > 3.2. Return true.
    rooted!(&in(cx) let mut ignored = UndefinedValue());
    if !unsafe {
        Call(
            cx,
            receiver,
            setter_jsval.handle(),
            // FIXME: Our binding lacks `HandleValueArray(Handle<Value>)`
            // <https://searchfox.org/mozilla-central/rev/072710086ddfe25aa2962c8399fefb2304e8193b/js/public/ValueArray.h#54-55>
            &HandleValueArray {
                length_: 1,
                elements_: v.ptr,
            },
            ignored.handle_mut(),
        )
    } {
        return false;
    }

    unsafe {
        (*result).code_ = 0 /* OkCode */
    };
    true
}

/// Implementation of [`CrossOriginPropertyFallback`].
///
/// `cx` and `proxy` are expected to be different-Realm here. `proxy` is a proxy
/// for a maybe-cross-origin object.
///
/// [`CrossOriginPropertyFallback`]: https://html.spec.whatwg.org/multipage/#crossoriginpropertyfallback-(-p-)
pub fn cross_origin_property_fallback<D: DomTypes>(
    cx: &mut CurrentRealm,
    _proxy: HandleObject,
    id: HandleId,
    desc: MutableHandle<PropertyDescriptor>,
    is_none: &mut bool,
) -> bool {
    assert!(*is_none, "why are we being called?");

    // > 1. If P is `then`, `@@toStringTag`, `@@hasInstance`, or
    // >    `@@isConcatSpreadable`, then return `PropertyDescriptor{ [[Value]]:
    // >    undefined, [[Writable]]: false, [[Enumerable]]: false,
    // >    [[Configurable]]: true }`.
    if is_cross_origin_allowlisted_prop(cx, id) {
        set_property_descriptor(
            desc,
            HandleValue::undefined(),
            JSPROP_READONLY as u32,
            is_none,
        );
        return true;
    }

    // > 2. Throw a `SecurityError` `DOMException`.
    report_cross_origin_denial::<D>(cx, id, "access")
}

// The types will be rooted in the function using them
#[cfg_attr(crown, crown::unrooted_must_root_lint::allow_unrooted_interior)]
pub(crate) struct JSProxyHandlerOwnPropertyKeysConfig<T: DomObject> {
    pub(crate) indexed_getter_and_length: Option<fn(&T, &mut JSContext) -> u32>,
    pub(crate) cross_origin: Option<&'static CrossOriginProperties>,
    pub(crate) unwrapped_proxy: unsafe fn(RawHandleObject) -> *const T,
    pub(crate) supported_named_properties: Option<fn(*const T, &mut JSContext) -> Vec<DOMString>>,
}

/// Helper type to keep AutoRealm and &mut CurrentRealm alive with Deref to JSContext
enum Realm<'a> {
    AutoRealm(AutoRealm<'a>),
    CurrentRealm(&'a mut CurrentRealm<'a>),
}

impl<'cx> Deref for Realm<'cx> {
    type Target = JSContext;

    fn deref(&'_ self) -> &'_ Self::Target {
        match self {
            Realm::AutoRealm(auto_realm) => auto_realm,
            Realm::CurrentRealm(current_realm) => current_realm,
        }
    }
}

impl<'cx> DerefMut for Realm<'cx> {
    fn deref_mut(&'_ mut self) -> &'_ mut Self::Target {
        match self {
            Realm::AutoRealm(auto_realm) => auto_realm,
            Realm::CurrentRealm(current_realm) => current_realm,
        }
    }
}

#[expect(non_snake_case)]
/// SAFETY: cx must point to a valid, non-null JS context.
pub(crate) unsafe fn JSProxyHandlerOwnPropertyKeys<T>(
    config: JSProxyHandlerOwnPropertyKeysConfig<T>,
    cx: *mut RawJSContext,
    proxy: RawHandleObject,
    props: RawMutableHandleIdVector,
) -> bool
where
    T: DomObject,
{
    unsafe {
        // SAFETY: it is safe to construct a JSContext from engine hook.
        let mut cx = JSContext::from_ptr(ptr::NonNull::new(cx).unwrap());
        let mut cx = CurrentRealm::assert(&mut cx);
        let current_realm = &mut cx;
        let unwrapped_proxy = (config.unwrapped_proxy)(proxy);

        let proxy = Handle::from_raw(proxy);

        let mut cx = if let Some(cross_origin_properties) = config.cross_origin {
            if !is_platform_object_same_origin(current_realm, proxy) {
                return cross_origin_own_property_keys(
                    current_realm,
                    proxy,
                    cross_origin_properties,
                    props,
                );
            }

            // Safe to enter the Realm of proxy now.
            let cx = AutoRealm::new_from_handle(current_realm, proxy);
            Realm::AutoRealm(cx)
        } else {
            Realm::CurrentRealm(current_realm)
        };

        if let Some(length_fn) = config.indexed_getter_and_length {
            let length = (length_fn)(&*unwrapped_proxy, &mut cx);
            rooted!(&in(cx) let mut rooted_jsid: jsid);
            for i in 0..length {
                int_to_jsid(i as i32, rooted_jsid.handle_mut());
                AppendToIdVector(props, rooted_jsid.handle());
            }
        }

        if let Some(properties) = config.supported_named_properties {
            for name in properties(unwrapped_proxy, &mut cx) {
                let cstring = CString::new(name).unwrap();
                let jsstring = JS_AtomizeAndPinString(&cx, cstring.as_ptr());
                rooted!(&in(cx) let rooted = jsstring);
                rooted!(&in(cx) let mut rooted_jsid: jsid);
                RUST_INTERNED_STRING_TO_JSID(
                    &mut cx,
                    rooted.handle().get(),
                    rooted_jsid.handle_mut(),
                );
                AppendToIdVector(props, rooted_jsid.handle());
            }
        }

        rooted!(&in(cx) let mut expando = ptr::null_mut::<JSObject>());
        get_expando_object(proxy, expando.handle_mut());

        if !expando.is_null() &&
            !GetPropertyKeys(
                &mut cx,
                expando.handle(),
                JSITER_OWNONLY | JSITER_HIDDEN | JSITER_SYMBOLS,
                props,
            )
        {
            return false;
        }
    }
    true
}

// The types will be rooted in the function using them
#[cfg_attr(crown, crown::unrooted_must_root_lint::allow_unrooted_interior)]
pub(crate) struct JSProxyHandlerOwnEnumerablePropertyKeysConfig<T: DomObject> {
    pub(crate) unwrapped_proxy: unsafe fn(RawHandleObject) -> *const T,
    #[expect(clippy::type_complexity)]
    pub(crate) indexed_getter_and_length: Option<Box<dyn Fn(&T, &mut JSContext) -> u32>>,
    pub(crate) cross_origin: bool,
}

#[expect(non_snake_case)]
pub(crate) fn JSProxyHandlerGetOwnEnumerablePropertyKeys<T>(
    config: JSProxyHandlerOwnEnumerablePropertyKeysConfig<T>,
    cx: *mut RawJSContext,
    proxy: RawHandleObject,
    props: RawMutableHandleIdVector,
) -> bool
where
    T: DomObject,
{
    unsafe {
        // SAFETY: it is safe to construct a JSContext from engine hook.
        let mut cx = JSContext::from_ptr(ptr::NonNull::new(cx).unwrap());
        let unwrapped_proxy = (config.unwrapped_proxy)(proxy);
        let mut cx = CurrentRealm::assert(&mut cx);
        let current_realm = &mut cx;

        let proxy = Handle::from_raw(proxy);

        let mut cx = if config.cross_origin {
            if !is_platform_object_same_origin(current_realm, proxy) {
                // There are no enumerable cross-origin props, so we're done.
                return true;
            }

            // Safe to enter the Realm of proxy now.
            let cx = AutoRealm::new_from_handle(current_realm, proxy);
            Realm::AutoRealm(cx)
        } else {
            Realm::CurrentRealm(current_realm)
        };
        if let Some(length_fn) = config.indexed_getter_and_length {
            let length = (length_fn)(&*unwrapped_proxy, &mut cx);
            rooted!(&in(cx) let mut rooted_jsid: jsid);
            for i in 0..length {
                int_to_jsid(i as i32, rooted_jsid.handle_mut());
                AppendToIdVector(props, rooted_jsid.handle());
            }
        }

        rooted!(&in(cx) let mut expando = ptr::null_mut::<JSObject>());
        get_expando_object(proxy, expando.handle_mut());
        if !expando.is_null() &&
            !GetPropertyKeys(
                &mut cx,
                expando.handle(),
                JSITER_OWNONLY | JSITER_HIDDEN | JSITER_SYMBOLS,
                props,
            )
        {
            return false;
        }
    }

    true
}

/// <https://html.spec.whatwg.org/multipage/#isplatformobjectsameorigin-(-o-)>
pub fn is_platform_object_same_origin(realm: &CurrentRealm, obj: HandleObject) -> bool {
    let subject_realm = realm.realm().as_ptr();
    let object_realm = unsafe { GetObjectRealmOrNull(*obj) };
    assert!(!object_realm.is_null());

    if subject_realm == object_realm {
        return true;
    }

    let subject_principals =
        unsafe { ServoJSPrincipalsRef::from_raw_unchecked(GetRealmPrincipals(subject_realm)) };
    let object_principals =
        unsafe { ServoJSPrincipalsRef::from_raw_unchecked(GetRealmPrincipals(object_realm)) };

    let subject_origin = subject_principals.origin();
    let object_origin = object_principals.origin();

    let result = subject_origin.same_origin_domain(&object_origin);
    log::trace!(
        "object {:p} (realm = {:p}, principalls = {:p}, origin = {:?}) is {} \
        with reference to the current Realm (realm = {:p}, principals = {:p}, \
        origin = {:?})",
        obj.get(),
        object_realm,
        object_principals.as_raw(),
        object_origin.immutable(),
        ["NOT same domain-origin", "same domain-origin"][result as usize],
        subject_realm,
        subject_principals.as_raw(),
        subject_origin.immutable()
    );

    result
}
