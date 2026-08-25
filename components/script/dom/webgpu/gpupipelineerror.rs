/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use js::rust::HandleObject;
use script_bindings::reflector::reflect_dom_object_with_proto;

use crate::dom::bindings::codegen::Bindings::WebGPUBinding::{
    GPUPipelineErrorInit, GPUPipelineErrorMethods, GPUPipelineErrorReason,
};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::domexception::DOMException;
use crate::dom::globalscope::GlobalScope;

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
        cx: &mut JSContext,
        global: &GlobalScope,
        proto: Option<HandleObject>,
        message: DOMString,
        reason: GPUPipelineErrorReason,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_proto(
            cx,
            Box::new(Self::new_inherited(message, reason)),
            global,
            proto,
        )
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        message: DOMString,
        reason: GPUPipelineErrorReason,
    ) -> DomRoot<Self> {
        Self::new_with_proto(cx, global, None, message, reason)
    }
}

impl GPUPipelineErrorMethods<crate::DomTypeHolder> for GPUPipelineError {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpupipelineerror-constructor>
    fn Constructor(
        cx: &mut JSContext,
        global: &GlobalScope,
        proto: Option<HandleObject>,
        message: DOMString,
        options: &GPUPipelineErrorInit,
    ) -> DomRoot<Self> {
        Self::new_with_proto(cx, global, proto, message, options.reason)
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpupipelineerror-reason>
    fn Reason(&self) -> GPUPipelineErrorReason {
        self.reason
    }
}
