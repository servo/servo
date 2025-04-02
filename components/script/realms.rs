/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use js::jsapi::{GetCurrentRealmOrNull, JSAutoRealm};

use crate::DomTypes;
use crate::dom::bindings::reflector::DomObject;
use crate::dom::globalscope::GlobalScopeHelpers;
use crate::script_runtime::JSContext;

pub(crate) struct AlreadyInRealm(());

impl AlreadyInRealm {
    #![allow(unsafe_code)]
    pub(crate) fn assert<D: DomTypes>() -> AlreadyInRealm {
        unsafe {
            assert!(!GetCurrentRealmOrNull(*D::GlobalScope::get_cx()).is_null());
        }
        AlreadyInRealm(())
    }

    pub(crate) fn assert_for_cx(cx: JSContext) -> AlreadyInRealm {
        unsafe {
            assert!(!GetCurrentRealmOrNull(*cx).is_null());
        }
        AlreadyInRealm(())
    }
}

#[derive(Clone, Copy)]
pub(crate) enum InRealm<'a> {
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
    pub(crate) fn already(token: &AlreadyInRealm) -> InRealm {
        InRealm::Already(token)
    }

    pub(crate) fn entered(token: &JSAutoRealm) -> InRealm {
        InRealm::Entered(token)
    }
}

pub(crate) fn enter_realm_generic<D: DomTypes>(object: &impl DomObject) -> JSAutoRealm {
    JSAutoRealm::new(
        *D::GlobalScope::get_cx(),
        object.reflector().get_jsobject().get(),
    )
}

pub(crate) fn enter_realm(object: &impl DomObject) -> JSAutoRealm {
    enter_realm_generic::<crate::DomTypeHolder>(object)
}
