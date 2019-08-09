/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::reflector::DomObject;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::JSContext;
use js::jsapi::{GetCurrentRealmOrNull, JSAutoRealm};

pub struct AlreadyInCompartment(());

impl AlreadyInCompartment {
    #![allow(unsafe_code)]
    pub fn assert(global: &GlobalScope) -> AlreadyInCompartment {
        unsafe {
            assert!(!GetCurrentRealmOrNull(*global.get_cx()).is_null());
        }
        AlreadyInCompartment(())
    }

    pub fn assert_for_cx(cx: JSContext) -> AlreadyInCompartment {
        unsafe {
            assert!(!GetCurrentRealmOrNull(*cx).is_null());
        }
        AlreadyInCompartment(())
    }
}

#[derive(Clone, Copy)]
pub enum InCompartment<'a> {
    Already(&'a AlreadyInCompartment),
    Entered(&'a JSAutoRealm),
}

impl<'a> InCompartment<'a> {
    pub fn in_compartment(token: &AlreadyInCompartment) -> InCompartment {
        InCompartment::Already(token)
    }

    pub fn entered(token: &JSAutoRealm) -> InCompartment {
        InCompartment::Entered(token)
    }
}

pub fn enter_realm(object: &impl DomObject) -> JSAutoRealm {
    JSAutoRealm::new(
        *object.global().get_cx(),
        object.reflector().get_jsobject().get(),
    )
}
