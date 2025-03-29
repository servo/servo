/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Utilities for the implementation of JSAPI proxy handlers.

#![deny(missing_docs)]

use std::ptr;

use js::conversions::ToJSValConvertible;
use js::glue::{GetProxyHandler, InvokeGetOwnPropertyDescriptor};
use js::jsapi;
use js::jsapi::{
    GetObjectRealmOrNull, GetRealmPrincipals, HandleId as RawHandleId,
    HandleObject as RawHandleObject, HandleValue as RawHandleValue,
    JS_IsExceptionPending, JSAutoRealm, JSContext, JSObject,
    MutableHandle as RawMutableHandle,
    MutableHandleObject as RawMutableHandleObject, MutableHandleValue as RawMutableHandleValue,
    ObjectOpResult, PropertyDescriptor,
};
use js::jsval::UndefinedValue;
use js::rust::{
    HandleObject, HandleValue, MutableHandle, MutableHandleObject, get_context_realm,
};

use crate::DomTypes;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::principals::ServoJSPrincipalsRef;
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::utils::DomHelpers;
use crate::dom::globalscope::GlobalScopeHelpers;
use crate::realms::{AlreadyInRealm, InRealm};
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

pub(crate) use script_bindings::proxyhandler::*;

/// <https://html.spec.whatwg.org/multipage/#isplatformobjectsameorigin-(-o-)>
pub(crate) unsafe fn is_platform_object_same_origin(
    cx: SafeJSContext,
    obj: RawHandleObject,
) -> bool {
    let subject_realm = get_context_realm(*cx);
    let obj_realm = GetObjectRealmOrNull(*obj);
    assert!(!obj_realm.is_null());

    let subject_principals =
        ServoJSPrincipalsRef::from_raw_unchecked(GetRealmPrincipals(subject_realm));
    let obj_principals = ServoJSPrincipalsRef::from_raw_unchecked(GetRealmPrincipals(obj_realm));

    let subject_origin = subject_principals.origin();
    let obj_origin = obj_principals.origin();

    let result = subject_origin.same_origin_domain(&obj_origin);
    log::trace!(
        "object {:p} (realm = {:p}, principalls = {:p}, origin = {:?}) is {} \
        with reference to the current Realm (realm = {:p}, principals = {:p}, \
        origin = {:?})",
        obj.get(),
        obj_realm,
        obj_principals.as_raw(),
        obj_origin.immutable(),
        ["NOT same domain-origin", "same domain-origin"][result as usize],
        subject_realm,
        subject_principals.as_raw(),
        subject_origin.immutable()
    );

    result
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
    if is_platform_object_same_origin(cx, proxy) {
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
