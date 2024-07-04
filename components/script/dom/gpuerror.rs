/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;
use webgpu::{Error, ErrorFilter};

use super::types::{GPUInternalError, GPUOutOfMemoryError, GPUValidationError};
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::{GPUErrorFilter, GPUErrorMethods};
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, Reflector};
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

    #[allow(dead_code)]
    pub fn new(global: &GlobalScope, message: DOMString) -> DomRoot<Self> {
        Self::new_with_proto(global, None, message)
    }

    #[allow(dead_code)]
    pub fn new_with_proto(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        message: DOMString,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_proto(Box::new(GPUError::new_inherited(message)), global, proto)
    }

    pub fn from_error(global: &GlobalScope, error: Error) -> DomRoot<Self> {
        match error {
            Error::Validation(msg) => DomRoot::upcast(GPUValidationError::new_with_proto(
                global,
                None,
                DOMString::from_string(msg),
            )),
            Error::OutOfMemory(msg) => DomRoot::upcast(GPUOutOfMemoryError::new_with_proto(
                global,
                None,
                DOMString::from_string(msg),
            )),
            Error::Internal(msg) => DomRoot::upcast(GPUInternalError::new_with_proto(
                global,
                None,
                DOMString::from_string(msg),
            )),
        }
    }
}

impl GPUErrorMethods for GPUError {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpuerror-message>
    fn Message(&self) -> DOMString {
        self.message.clone()
    }
}

impl From<ErrorFilter> for GPUErrorFilter {
    fn from(filter: ErrorFilter) -> Self {
        match filter {
            ErrorFilter::Validation => GPUErrorFilter::Validation,
            ErrorFilter::OutOfMemory => GPUErrorFilter::Out_of_memory,
            ErrorFilter::Internal => GPUErrorFilter::Internal,
        }
    }
}

impl GPUErrorFilter {
    pub fn as_webgpu(&self) -> ErrorFilter {
        match self {
            GPUErrorFilter::Validation => ErrorFilter::Validation,
            GPUErrorFilter::Out_of_memory => ErrorFilter::OutOfMemory,
            GPUErrorFilter::Internal => ErrorFilter::Internal,
        }
    }
}
