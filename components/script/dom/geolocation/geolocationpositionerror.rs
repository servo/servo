/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::codegen::GenericBindings::GeolocationPositionErrorBinding::GeolocationPositionErrorConstants::{PERMISSION_DENIED, POSITION_UNAVAILABLE, TIMEOUT};
use script_bindings::codegen::GenericBindings::GeolocationPositionErrorBinding::GeolocationPositionErrorMethods;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};
use script_bindings::root::DomRoot;
use script_bindings::str::DOMString;

use crate::dom::bindings::codegen::DomTypeHolder::DomTypeHolder;
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

    fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        code: u16,
        message: DOMString,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_cx(Box::new(Self::new_inherited(code, message)), global, cx)
    }

    pub(crate) fn permission_denied(
        cx: &mut JSContext,
        global: &GlobalScope,
        message: DOMString,
    ) -> DomRoot<Self> {
        Self::new(cx, global, PERMISSION_DENIED, message)
    }

    pub(crate) fn position_unavailable(
        cx: &mut JSContext,
        global: &GlobalScope,
        message: DOMString,
    ) -> DomRoot<Self> {
        Self::new(cx, global, POSITION_UNAVAILABLE, message)
    }

    #[expect(unused)]
    pub(crate) fn timeout(
        cx: &mut JSContext,
        global: &GlobalScope,
        message: DOMString,
    ) -> DomRoot<Self> {
        Self::new(cx, global, TIMEOUT, message)
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
