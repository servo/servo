/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;

use super::bindings::codegen::Bindings::WebGPUBinding::GPUErrorMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;

#[dom_struct]
pub struct GPUError {
    reflector_: Reflector,
    message: DOMString,
}

impl GPUError {
    pub fn new_inherited(message: DOMString) -> Self {
        Self {
            reflector_: Reflector::new(),
            message,
        }
    }

    pub fn new(global: &GlobalScope, message: DOMString) -> DomRoot<Self> {
        reflect_dom_object(Box::new(GPUError::new_inherited(message)), global)
    }

    pub fn msg(&self) -> DOMString {
        self.message
    }
}

impl GPUErrorMethods for GPUError {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpuerror-message>
    fn Message(&self) -> DOMString {
        self.message
    }
}
