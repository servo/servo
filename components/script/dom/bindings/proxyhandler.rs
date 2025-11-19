/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Utilities for the implementation of JSAPI proxy handlers.

#![deny(missing_docs)]

use js::jsapi::{GetObjectRealmOrNull, GetRealmPrincipals, HandleObject as RawHandleObject};
use js::realm::CurrentRealm;
use script_bindings::principals::ServoJSPrincipalsRef;
pub(crate) use script_bindings::proxyhandler::*;

/// <https://html.spec.whatwg.org/multipage/#isplatformobjectsameorigin-(-o-)>
pub(crate) unsafe fn is_platform_object_same_origin(
    realm: &CurrentRealm,
    obj: RawHandleObject,
) -> bool {
    let subject_realm = realm.realm().as_ptr();
    let obj_realm = unsafe { GetObjectRealmOrNull(*obj) };
    assert!(!obj_realm.is_null());

    let subject_principals =
        unsafe { ServoJSPrincipalsRef::from_raw_unchecked(GetRealmPrincipals(subject_realm)) };
    let obj_principals =
        unsafe { ServoJSPrincipalsRef::from_raw_unchecked(GetRealmPrincipals(obj_realm)) };

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
