/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use js::jsapi::{GetCurrentRealmOrNull, JSAutoRealm};

//pub use script_bindings::realms::{AlreadyInRealm, InRealm};
use crate::dom::bindings::reflector::DomObject;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::JSContext;

pub struct AlreadyInRealm(());

impl<'a> From<&'a script_bindings::realms::AlreadyInRealm> for AlreadyInRealm {
    fn from(_realm: &'a script_bindings::realms::AlreadyInRealm) -> Self {
        Self(())
    }
}

impl AlreadyInRealm {
    #![allow(unsafe_code)]
    pub fn assert() -> script_bindings::realms::AlreadyInRealm {
        script_bindings::realms::AlreadyInRealm::assert::<crate::DomTypeHolder>()
    }

    pub fn assert_for_cx(cx: JSContext) -> script_bindings::realms::AlreadyInRealm {
        script_bindings::realms::AlreadyInRealm::assert_for_cx(cx)
    }
}

pub use script_bindings::realms::InRealm;

pub fn enter_realm(object: &impl DomObject) -> JSAutoRealm {
    script_bindings::realms::enter_realm::<crate::DomTypeHolder>(object)
}
