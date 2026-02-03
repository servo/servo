/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use script_bindings::codegen::GenericBindings::GeolocationPositionErrorBinding::GeolocationPositionErrorConstants::{PERMISSION_DENIED, POSITION_UNAVAILABLE, TIMEOUT};
use script_bindings::codegen::GenericBindings::GeolocationPositionErrorBinding::GeolocationPositionErrorMethods;
use script_bindings::reflector::Reflector;
use script_bindings::root::DomRoot;
use script_bindings::script_runtime::CanGc;
use script_bindings::str::DOMString;

use crate::dom::bindings::codegen::DomTypeHolder::DomTypeHolder;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::globalscope::GlobalScope;

#[dom_struct]
pub struct GeolocationPositionError {
    reflector_: Reflector,
    code: u16,
    message: DOMString,
}

impl GeolocationPositionError {
    fn new_inherited(code: u16, message: DOMString) -> Self {
        GeolocationPositionError {
            reflector_: Reflector::new(),
            code,
            message,
        }
    }

    fn new(global: &GlobalScope, code: u16, message: DOMString, can_gc: CanGc) -> DomRoot<Self> {
        reflect_dom_object(Box::new(Self::new_inherited(code, message)), global, can_gc)
    }

    pub(crate) fn permission_denied(
        global: &GlobalScope,
        message: DOMString,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        Self::new(global, PERMISSION_DENIED, message, can_gc)
    }

    pub(crate) fn position_unavailable(
        global: &GlobalScope,
        message: DOMString,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        Self::new(global, POSITION_UNAVAILABLE, message, can_gc)
    }

    #[expect(unused)]
    pub(crate) fn timeout(
        global: &GlobalScope,
        message: DOMString,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        Self::new(global, TIMEOUT, message, can_gc)
    }
}

impl GeolocationPositionErrorMethods<DomTypeHolder> for GeolocationPositionError {
    fn Code(&self) -> u16 {
        self.code
    }

    fn Message(&self) -> DOMString {
        self.message.clone()
    }
}
