/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;
use webgpu_traits::{Error, ErrorFilter};

use crate::conversions::Convert;
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::{GPUErrorFilter, GPUErrorMethods};
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object_with_proto};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::types::{GPUInternalError, GPUOutOfMemoryError, GPUValidationError};
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct GPUError {
    reflector_: Reflector,
    message: DOMString,
}

impl GPUError {
    pub(crate) fn new_inherited(message: DOMString) -> Self {
        Self {
            reflector_: Reflector::new(),
            message,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn new(global: &GlobalScope, message: DOMString, can_gc: CanGc) -> DomRoot<Self> {
        Self::new_with_proto(global, None, message, can_gc)
    }

    #[allow(dead_code)]
    pub(crate) fn new_with_proto(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        message: DOMString,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_proto(
            Box::new(GPUError::new_inherited(message)),
            global,
            proto,
            can_gc,
        )
    }

    pub(crate) fn from_error(global: &GlobalScope, error: Error, can_gc: CanGc) -> DomRoot<Self> {
        match error {
            Error::Validation(msg) => DomRoot::upcast(GPUValidationError::new_with_proto(
                global,
                None,
                DOMString::from_string(msg),
                can_gc,
            )),
            Error::OutOfMemory(msg) => DomRoot::upcast(GPUOutOfMemoryError::new_with_proto(
                global,
                None,
                DOMString::from_string(msg),
                can_gc,
            )),
            Error::Internal(msg) => DomRoot::upcast(GPUInternalError::new_with_proto(
                global,
                None,
                DOMString::from_string(msg),
                can_gc,
            )),
        }
    }
}

impl GPUErrorMethods<crate::DomTypeHolder> for GPUError {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpuerror-message>
    fn Message(&self) -> DOMString {
        self.message.clone()
    }
}

impl Convert<GPUErrorFilter> for ErrorFilter {
    fn convert(self) -> GPUErrorFilter {
        match self {
            ErrorFilter::Validation => GPUErrorFilter::Validation,
            ErrorFilter::OutOfMemory => GPUErrorFilter::Out_of_memory,
            ErrorFilter::Internal => GPUErrorFilter::Internal,
        }
    }
}

pub(crate) trait AsWebGpu {
    fn as_webgpu(&self) -> ErrorFilter;
}

impl AsWebGpu for GPUErrorFilter {
    fn as_webgpu(&self) -> ErrorFilter {
        match self {
            GPUErrorFilter::Validation => ErrorFilter::Validation,
            GPUErrorFilter::Out_of_memory => ErrorFilter::OutOfMemory,
            GPUErrorFilter::Internal => ErrorFilter::Internal,
        }
    }
}
