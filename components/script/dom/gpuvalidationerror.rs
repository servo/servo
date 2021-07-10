/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::GPUValidationErrorBinding::GPUValidationErrorMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use dom_struct::dom_struct;

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
        reflect_dom_object(Box::new(GPUValidationError::new_inherited(message)), global)
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuvalidationerror-gpuvalidationerror
    #[allow(non_snake_case)]
    pub fn Constructor(global: &GlobalScope, message: DOMString) -> DomRoot<Self> {
        GPUValidationError::new(global, message)
    }
}

impl GPUValidationErrorMethods for GPUValidationError {
    /// https://gpuweb.github.io/gpuweb/#dom-gpuvalidationerror-message
    fn Message(&self) -> DOMString {
        self.message.clone()
    }
}
