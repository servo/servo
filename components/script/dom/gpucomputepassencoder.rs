/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::GPUComputePassEncoderBinding::{
    self, GPUComputePassEncoderMethods,
};
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use dom_struct::dom_struct;
use webgpu::{wgpu::command::RawPass, WebGPU};

#[dom_struct]
pub struct GPUComputePassEncoder {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "defined in webgpu"]
    channel: WebGPU,
    label: DomRefCell<Option<DOMString>>,
    #[ignore_malloc_size_of = "defined in wgpu-core"]
    pass: RawPass,
}

impl GPUComputePassEncoder {
    pub fn new_inherited(channel: WebGPU, pass: RawPass) -> GPUComputePassEncoder {
        GPUComputePassEncoder {
            channel,
            reflector_: Reflector::new(),
            label: DomRefCell::new(None),
            pass,
        }
    }

    pub fn new(
        global: &GlobalScope,
        channel: WebGPU,
        pass: RawPass,
    ) -> DomRoot<GPUComputePassEncoder> {
        reflect_dom_object(
            Box::new(GPUComputePassEncoder::new_inherited(channel, pass)),
            global,
            GPUComputePassEncoderBinding::Wrap,
        )
    }
}

impl GPUComputePassEncoderMethods for GPUComputePassEncoder {
    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn GetLabel(&self) -> Option<DOMString> {
        self.label.borrow().clone()
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn SetLabel(&self, value: Option<DOMString>) {
        *self.label.borrow_mut() = value;
    }
}
