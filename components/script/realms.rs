/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use js::jsapi::{GetCurrentRealmOrNull, JSAutoRealm};

use crate::dom::bindings::reflector::DomObject;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::JSContext;

pub struct AlreadyInRealm(());

impl AlreadyInRealm {
    #![allow(unsafe_code)]
    pub fn assert() -> AlreadyInRealm {
        unsafe {
            assert!(!GetCurrentRealmOrNull(*GlobalScope::get_cx()).is_null());
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

impl<'a> InRealm<'a> {
    pub fn already(token: &AlreadyInRealm) -> InRealm {
        InRealm::Already(token)
    }

    pub fn entered(token: &JSAutoRealm) -> InRealm {
        InRealm::Entered(token)
    }
}

pub fn enter_realm(object: &impl DomObject) -> JSAutoRealm {
    JSAutoRealm::new(
        *GlobalScope::get_cx(),
        object.reflector().get_jsobject().get(),
    )
}
