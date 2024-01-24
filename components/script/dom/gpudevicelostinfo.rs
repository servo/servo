/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(dead_code)]

use dom_struct::dom_struct;

use super::bindings::codegen::Bindings::WebGPUBinding::GPUDeviceLostReason;
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::GPUDeviceLostInfoMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;

#[dom_struct]
pub struct GPUDeviceLostInfo {
    reflector_: Reflector,
    message: DOMString,
    reason: GPUDeviceLostReason,
}

impl GPUDeviceLostInfo {
    fn new_inherited(message: DOMString, reason: GPUDeviceLostReason) -> Self {
        Self {
            reflector_: Reflector::new(),
            message,
            reason,
        }
    }

    pub fn new(
        global: &GlobalScope,
        message: DOMString,
        reason: GPUDeviceLostReason,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(GPUDeviceLostInfo::new_inherited(message, reason)),
            global,
        )
    }
}

impl GPUDeviceLostInfoMethods for GPUDeviceLostInfo {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevicelostinfo-message>
    fn Message(&self) -> DOMString {
        self.message.clone()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevicelostinfo-reason>
    fn Reason(&self) -> GPUDeviceLostReason {
        self.reason
    }
}
