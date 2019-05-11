/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::globalscope::GlobalScope;
use js::jsapi::{GetCurrentRealmOrNull, JSAutoRealm};

pub struct AlreadyInCompartment(());

impl AlreadyInCompartment {
    #![allow(unsafe_code)]
    pub fn assert(global: &GlobalScope) -> AlreadyInCompartment {
        unsafe {
            assert!(!GetCurrentRealmOrNull(global.get_cx()).is_null());
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
