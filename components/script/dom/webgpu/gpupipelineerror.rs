/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;

use crate::dom::bindings::codegen::Bindings::WebGPUBinding::{
    GPUPipelineErrorInit, GPUPipelineErrorMethods, GPUPipelineErrorReason,
};
use crate::dom::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::domexception::DOMException;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::CanGc;

/// <https://gpuweb.github.io/gpuweb/#gpupipelineerror>
#[dom_struct]
pub(crate) struct GPUPipelineError {
    exception: DOMException,
    reason: GPUPipelineErrorReason,
}

impl GPUPipelineError {
    fn new_inherited(message: DOMString, reason: GPUPipelineErrorReason) -> Self {
        Self {
            exception: DOMException::new_inherited(message, "GPUPipelineError".into()),
            reason,
        }
    }

    pub(crate) fn new_with_proto(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        message: DOMString,
        reason: GPUPipelineErrorReason,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_proto(
            Box::new(Self::new_inherited(message, reason)),
            global,
            proto,
            can_gc,
        )
    }

    pub(crate) fn new(
        global: &GlobalScope,
        message: DOMString,
        reason: GPUPipelineErrorReason,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        Self::new_with_proto(global, None, message, reason, can_gc)
    }
}

impl GPUPipelineErrorMethods<crate::DomTypeHolder> for GPUPipelineError {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpupipelineerror-constructor>
    fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        message: DOMString,
        options: &GPUPipelineErrorInit,
    ) -> DomRoot<Self> {
        Self::new_with_proto(global, proto, message, options.reason, can_gc)
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpupipelineerror-reason>
    fn Reason(&self) -> GPUPipelineErrorReason {
        self.reason
    }
}
