/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;

use super::bindings::codegen::Bindings::WebGPUBinding::{
    GPUPipelineErrorInit, GPUPipelineErrorMethods, GPUPipelineErrorReason,
};
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomObject, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::domexception::{DOMErrorName, DOMException};
use crate::dom::globalscope::GlobalScope;

#[dom_struct]
pub struct GPUPipelineError {
    exception: Dom<DOMException>,
    reason: GPUPipelineErrorReason,
}

impl GPUPipelineError {
    fn new_inherited(
        global: &GlobalScope,
        message: DOMString,
        init: &GPUPipelineErrorInit,
    ) -> GPUPipelineError {
        GPUPipelineError {
            exception: Dom::from_ref(&*DOMException::new(
                global,
                DOMErrorName::from(&message).unwrap(),
            )),
            reason: init.reason,
        }
    }

    pub fn new(
        global: &GlobalScope,
        message: DOMString,
        init: &GPUPipelineErrorInit,
    ) -> DomRoot<GPUPipelineError> {
        Self::new_with_proto(global, None, message, init)
    }

    fn new_with_proto(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        message: DOMString,
        init: &GPUPipelineErrorInit,
    ) -> DomRoot<GPUPipelineError> {
        reflect_dom_object_with_proto(
            Box::new(GPUPipelineError::new_inherited(global, message, init)),
            global,
            proto,
        )
    }

    #[allow(non_snake_case)]
    pub fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        message: DOMString,
        init: &GPUPipelineErrorInit,
    ) -> DomRoot<GPUPipelineError> {
        GPUPipelineError::new_with_proto(global, proto, message, init)
    }
}

impl GPUPipelineErrorMethods for GPUPipelineError {
    /// https://gpuweb.github.io/gpuweb/#dom-gpupipelineerror-reason
    fn Reason(&self) -> GPUPipelineErrorReason {
        self.reason
    }
}
