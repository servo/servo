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
use crate::dom::gpubindgroup::GPUBindGroup;
use crate::dom::gpucomputepipeline::GPUComputePipeline;
use dom_struct::dom_struct;
use std::cell::RefCell;
use webgpu::{
    wgpu::command::{
        compute_ffi::{
            wgpu_compute_pass_dispatch, wgpu_compute_pass_set_bind_group,
            wgpu_compute_pass_set_pipeline,
        },
        RawPass,
    },
    WebGPU, WebGPUCommandEncoder, WebGPURequest,
};

#[dom_struct]
pub struct GPUComputePassEncoder {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "defined in webgpu"]
    channel: WebGPU,
    label: DomRefCell<Option<DOMString>>,
    #[ignore_malloc_size_of = "defined in wgpu-core"]
    raw_pass: RefCell<Option<RawPass>>,
}

impl GPUComputePassEncoder {
    fn new_inherited(channel: WebGPU, parent: WebGPUCommandEncoder) -> GPUComputePassEncoder {
        GPUComputePassEncoder {
            channel,
            reflector_: Reflector::new(),
            label: DomRefCell::new(None),
            raw_pass: RefCell::new(Some(RawPass::new_compute(parent.0))),
        }
    }

    pub fn new(
        global: &GlobalScope,
        channel: WebGPU,
        parent: WebGPUCommandEncoder,
    ) -> DomRoot<GPUComputePassEncoder> {
        reflect_dom_object(
            Box::new(GPUComputePassEncoder::new_inherited(channel, parent)),
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

    #[allow(unsafe_code)]
    /// https://gpuweb.github.io/gpuweb/#dom-gpucomputepassencoder-dispatch
    fn Dispatch(&self, x: u32, y: u32, z: u32) {
        if let Some(raw_pass) = self.raw_pass.borrow_mut().as_mut() {
            unsafe { wgpu_compute_pass_dispatch(raw_pass, x, y, z) };
        }
    }

    #[allow(unsafe_code)]
    /// https://gpuweb.github.io/gpuweb/#dom-gpurenderpassencoder-endpass
    fn EndPass(&self) {
        if let Some(raw_pass) = self.raw_pass.borrow_mut().take() {
            let (pass_data, id) = unsafe { raw_pass.finish_compute() };

            self.channel
                .0
                .send(WebGPURequest::RunComputePass(id, pass_data))
                .unwrap();
        }
    }

    #[allow(unsafe_code)]
    /// https://gpuweb.github.io/gpuweb/#dom-gpuprogrammablepassencoder-setbindgroup
    fn SetBindGroup(&self, index: u32, bind_group: &GPUBindGroup, dynamic_offsets: Vec<u32>) {
        if let Some(raw_pass) = self.raw_pass.borrow_mut().as_mut() {
            unsafe {
                wgpu_compute_pass_set_bind_group(
                    raw_pass,
                    index,
                    bind_group.id().0,
                    dynamic_offsets.as_ptr(),
                    dynamic_offsets.len(),
                )
            };
        }
    }

    #[allow(unsafe_code)]
    /// https://gpuweb.github.io/gpuweb/#dom-gpucomputepassencoder-setpipeline
    fn SetPipeline(&self, pipeline: &GPUComputePipeline) {
        if let Some(raw_pass) = self.raw_pass.borrow_mut().as_mut() {
            unsafe { wgpu_compute_pass_set_pipeline(raw_pass, pipeline.id().0) };
        }
    }
}
