/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use js::rust::HandleObject;
use script_bindings::reflector::reflect_dom_object_with_proto_and_cx;

use crate::dom::bindings::codegen::Bindings::WebGPUBinding::GPUValidationError_Binding::GPUValidationErrorMethods;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::types::GPUError;

#[dom_struct]
pub(crate) struct GPUValidationError {
    gpu_error: GPUError,
}

impl GPUValidationError {
    fn new_inherited(message: DOMString) -> Self {
        Self {
            gpu_error: GPUError::new_inherited(message),
        }
    }

    pub(crate) fn new_with_proto(
        cx: &mut JSContext,
        global: &GlobalScope,
        proto: Option<HandleObject>,
        message: DOMString,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_proto_and_cx(
            Box::new(Self::new_inherited(message)),
            global,
            proto,
            cx,
        )
    }
}

impl GPUValidationErrorMethods<crate::DomTypeHolder> for GPUValidationError {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpuvalidationerror-gpuvalidationerror>
    fn Constructor(
        cx: &mut js::context::JSContext,
        global: &GlobalScope,
        proto: Option<HandleObject>,
        message: DOMString,
    ) -> DomRoot<Self> {
        Self::new_with_proto(cx, global, proto, message)
    }
}
