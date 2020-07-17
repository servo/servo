/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(dead_code)]

use crate::dom::bindings::codegen::Bindings::GPUDeviceLostInfoBinding::GPUDeviceLostInfoMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use dom_struct::dom_struct;

#[dom_struct]
pub struct GPUDeviceLostInfo {
    reflector_: Reflector,
    message: DOMString,
}

impl GPUDeviceLostInfo {
    fn new_inherited(message: DOMString) -> Self {
        Self {
            reflector_: Reflector::new(),
            message,
        }
    }

    pub fn new(global: &GlobalScope, message: DOMString) -> DomRoot<Self> {
        reflect_dom_object(Box::new(GPUDeviceLostInfo::new_inherited(message)), global)
    }
}

impl GPUDeviceLostInfoMethods for GPUDeviceLostInfo {
    /// https://gpuweb.github.io/gpuweb/#dom-gpudevicelostinfo-message
    fn Message(&self) -> DOMString {
        self.message.clone()
    }
}
