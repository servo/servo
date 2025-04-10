/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use js::jsapi::{GetCurrentRealmOrNull, JSAutoRealm};

use crate::DomTypes;
use crate::interfaces::GlobalScopeHelpers;
use crate::reflector::DomObject;
use crate::script_runtime::JSContext;

pub struct AlreadyInRealm(());

impl AlreadyInRealm {
    #![allow(unsafe_code)]
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
    pub fn already(token: &AlreadyInRealm) -> InRealm {
        InRealm::Already(token)
    }

    pub fn entered(token: &JSAutoRealm) -> InRealm {
        InRealm::Entered(token)
    }
}

pub fn enter_realm<D: DomTypes>(object: &impl DomObject) -> JSAutoRealm {
    JSAutoRealm::new(
        *D::GlobalScope::get_cx(),
        object.reflector().get_jsobject().get(),
    )
}
