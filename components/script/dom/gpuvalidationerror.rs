/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;

use super::bindings::error::Fallible;
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::GPUValidationErrorMethods;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;

#[dom_struct]
pub struct GPUValidationError {
    reflector_: Reflector,
    message: DOMString,
}

impl GPUValidationError {
    fn new_inherited(message: DOMString) -> Self {
        Self {
            reflector_: Reflector::new(),
            message,
        }
    }

    pub fn new(global: &GlobalScope, message: DOMString) -> DomRoot<Self> {
        Self::new_with_proto(global, None, message)
    }

    fn new_with_proto(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        message: DOMString,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_proto(
            Box::new(GPUValidationError::new_inherited(message)),
            global,
            proto,
        )
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuvalidationerror-gpuvalidationerror
    #[allow(non_snake_case)]
    pub fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        message: DOMString,
    ) -> Fallible<DomRoot<Self>> {
        Ok(GPUValidationError::new_with_proto(global, proto, message))
    }
}

impl GPUValidationErrorMethods for GPUValidationError {
    /// https://gpuweb.github.io/gpuweb/#dom-gpuvalidationerror-message
    fn Message(&self) -> DOMString {
        self.message.clone()
    }
}
