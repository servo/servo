/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;

use crate::dom::bindings::codegen::Bindings::WebGPUBinding::GPUOutOfMemoryError_Binding::GPUOutOfMemoryErrorMethods;
use crate::dom::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::types::GPUError;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct GPUOutOfMemoryError {
    gpu_error: GPUError,
}

impl GPUOutOfMemoryError {
    fn new_inherited(message: DOMString) -> Self {
        Self {
            gpu_error: GPUError::new_inherited(message),
        }
    }

    pub(crate) fn new_with_proto(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        message: DOMString,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_proto(
            Box::new(Self::new_inherited(message)),
            global,
            proto,
            can_gc,
        )
    }
}

impl GPUOutOfMemoryErrorMethods<crate::DomTypeHolder> for GPUOutOfMemoryError {
    /// <https://gpuweb.github.io/gpuweb/#dom-GPUOutOfMemoryError-GPUOutOfMemoryError>
    fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        message: DOMString,
    ) -> DomRoot<Self> {
        Self::new_with_proto(global, proto, message, can_gc)
    }
}
