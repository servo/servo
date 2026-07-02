/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ptr::NonNull;

use js::jsapi::{GetCurrentRealmOrNull, JSAutoRealm};
use js::realm::{AutoRealm, CurrentRealm};

use crate::DomTypes;
use crate::interfaces::GlobalScopeHelpers;
use crate::reflector::DomObject;
use crate::script_runtime::JSContext;

pub struct AlreadyInRealm(());

impl AlreadyInRealm {
    #![expect(unsafe_code)]
    pub fn assert<D: DomTypes>() -> AlreadyInRealm {
        unsafe {
            assert!(!GetCurrentRealmOrNull(*D::GlobalScope::get_cx()).is_null());
        }
        AlreadyInRealm(())
    }

    pub fn assert_for_cx(cx: JSContext) -> AlreadyInRealm {
        unsafe {
            assert!(!GetCurrentRealmOrNull(*cx).is_null());
        }
        AlreadyInRealm(())
    }
}

impl<'a, 'b> From<&'a mut CurrentRealm<'b>> for AlreadyInRealm {
    fn from(_: &'a mut CurrentRealm<'b>) -> AlreadyInRealm {
        AlreadyInRealm(())
    }
}

#[derive(Clone, Copy)]
pub enum InRealm<'a> {
    Already(&'a AlreadyInRealm),
    Entered(&'a JSAutoRealm),
}

impl<'a> From<&'a AlreadyInRealm> for InRealm<'a> {
    fn from(token: &'a AlreadyInRealm) -> InRealm<'a> {
        InRealm::already(token)
    }
}

impl<'a> From<&'a JSAutoRealm> for InRealm<'a> {
    fn from(token: &'a JSAutoRealm) -> InRealm<'a> {
        InRealm::entered(token)
    }
}

impl InRealm<'_> {
    pub fn already(token: &AlreadyInRealm) -> InRealm<'_> {
        InRealm::Already(token)
    }

    pub fn entered(token: &JSAutoRealm) -> InRealm<'_> {
        InRealm::Entered(token)
    }
}

pub fn enter_realm<D: DomTypes>(object: &impl DomObject) -> JSAutoRealm {
    JSAutoRealm::new(
        *D::GlobalScope::get_cx(),
        object.reflector().get_jsobject().get(),
    )
}

pub fn enter_auto_realm<'cx, D: DomTypes>(
    cx: &'cx mut js::context::JSContext,
    object: &impl DomObject,
) -> AutoRealm<'cx> {
    AutoRealm::new(
        cx,
        NonNull::new(object.reflector().get_jsobject().get()).unwrap(),
    )
}
