/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ptr::NonNull;

use js::glue::{DestroyRustJSPrincipals, GetRustJSPrincipalsPrivate, JSPrincipalsCallbacks};
use js::jsapi::{
    JS_ReadUint32Pair, JSContext, JSPrincipals, JSStructuredCloneReader, JSStructuredCloneWriter,
};
use script_bindings::principals::{ServoJSPrincipals, ServoJSPrincipalsRef};
use servo_url::MutableOrigin;

use super::structuredclone::StructuredCloneTags;
use crate::DomTypeHolder;

#[allow(unused)]
pub(crate) unsafe extern "C" fn destroy_servo_jsprincipal(principals: *mut JSPrincipals) {
    Box::from_raw(GetRustJSPrincipalsPrivate(principals) as *mut MutableOrigin);
    DestroyRustJSPrincipals(principals);
}

pub(crate) unsafe extern "C" fn write_jsprincipal(
    principal: *mut JSPrincipals,
    _cx: *mut JSContext,
    writer: *mut JSStructuredCloneWriter,
) -> bool {
    let Some(principal) = NonNull::new(principal) else {
        return false;
    };
    let obj = ServoJSPrincipalsRef::from_raw_nonnull(principal);
    let origin = obj.origin();
    let Ok(bytes_of_origin) = bincode::serialize(&origin) else {
        return false;
    };
    let Ok(len) = bytes_of_origin.len().try_into() else {
        return false;
    };
    if !js::jsapi::JS_WriteUint32Pair(writer, StructuredCloneTags::Principals as u32, len) {
        return false;
    }
    if !js::jsapi::JS_WriteBytes(writer, bytes_of_origin.as_ptr() as _, len as usize) {
        return false;
    }
    true
}

pub(crate) unsafe extern "C" fn read_jsprincipal(
    _cx: *mut JSContext,
    reader: *mut JSStructuredCloneReader,
    principals: *mut *mut JSPrincipals,
) -> bool {
    let mut tag: u32 = 0;
    let mut len: u32 = 0;
    if !JS_ReadUint32Pair(reader, &mut tag as *mut u32, &mut len as *mut u32) {
        return false;
    }
    if tag != StructuredCloneTags::Principals as u32 {
        return false;
    }
    let mut bytes = vec![0u8; len as usize];
    if !js::jsapi::JS_ReadBytes(reader, bytes.as_mut_ptr() as _, len as usize) {
        return false;
    }
    let Ok(origin) = bincode::deserialize(&bytes[..]) else {
        return false;
    };
    let principal = ServoJSPrincipals::new::<DomTypeHolder>(&origin);
    *principals = principal.as_raw();
    // we transferred ownership of principal to the caller
    std::mem::forget(principal);
    true
}

pub(crate) const PRINCIPALS_CALLBACKS: JSPrincipalsCallbacks = JSPrincipalsCallbacks {
    write: Some(write_jsprincipal),
    isSystemOrAddonPrincipal: Some(principals_is_system_or_addon_principal),
};

unsafe extern "C" fn principals_is_system_or_addon_principal(_: *mut JSPrincipals) -> bool {
    false
}

//TODO is same_origin_domain equivalent to subsumes for our purposes
pub(crate) unsafe extern "C" fn subsumes(obj: *mut JSPrincipals, other: *mut JSPrincipals) -> bool {
    match (NonNull::new(obj), NonNull::new(other)) {
        (Some(obj), Some(other)) => {
            let obj = ServoJSPrincipalsRef::from_raw_nonnull(obj);
            let other = ServoJSPrincipalsRef::from_raw_nonnull(other);
            let obj_origin = obj.origin();
            let other_origin = other.origin();
            obj_origin.same_origin_domain(&other_origin)
        },
        (None, Some(_)) => {
            // See https://github.com/servo/servo/issues/32999#issuecomment-2542522289 for why
            // it's safe to consider the null principal here subsumes all others.
            true
        },
        _ => {
            warn!("Received null JSPrincipal argument.");
            false
        },
    }
}
