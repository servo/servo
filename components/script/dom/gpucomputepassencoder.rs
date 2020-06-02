/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::GPUComputePassEncoderBinding::GPUComputePassEncoderMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::gpubindgroup::GPUBindGroup;
use crate::dom::gpucommandencoder::{GPUCommandEncoder, GPUCommandEncoderState};
use crate::dom::gpucomputepipeline::GPUComputePipeline;
use dom_struct::dom_struct;
use webgpu::{
    wgpu::command::{
        compute_ffi::{
            wgpu_compute_pass_dispatch, wgpu_compute_pass_set_bind_group,
            wgpu_compute_pass_set_pipeline,
        },
        RawPass,
    },
    WebGPU, WebGPURequest,
};

#[dom_struct]
pub struct GPUComputePassEncoder {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "defined in webgpu"]
    channel: WebGPU,
    label: DomRefCell<Option<DOMString>>,
    #[ignore_malloc_size_of = "defined in wgpu-core"]
    raw_pass: DomRefCell<Option<RawPass>>,
    command_encoder: Dom<GPUCommandEncoder>,
}

impl GPUComputePassEncoder {
    fn new_inherited(channel: WebGPU, parent: &GPUCommandEncoder) -> Self {
        Self {
            channel,
            reflector_: Reflector::new(),
            label: DomRefCell::new(None),
            raw_pass: DomRefCell::new(Some(unsafe { RawPass::new_compute(parent.id().0) })),
            command_encoder: Dom::from_ref(parent),
        }
    }

    pub fn new(global: &GlobalScope, channel: WebGPU, parent: &GPUCommandEncoder) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(GPUComputePassEncoder::new_inherited(channel, parent)),
            global,
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

    /// https://gpuweb.github.io/gpuweb/#dom-gpucomputepassencoder-dispatch
    fn Dispatch(&self, x: u32, y: u32, z: u32) {
        if let Some(raw_pass) = self.raw_pass.borrow_mut().as_mut() {
            unsafe { wgpu_compute_pass_dispatch(raw_pass, x, y, z) };
        }
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpurenderpassencoder-endpass
    fn EndPass(&self) {
        if let Some(raw_pass) = self.raw_pass.borrow_mut().take() {
            let (pass_data, command_encoder_id) = unsafe { raw_pass.finish_compute() };

            self.channel
                .0
                .send(WebGPURequest::RunComputePass {
                    command_encoder_id,
                    pass_data,
                })
                .unwrap();

            self.command_encoder.set_state(
                GPUCommandEncoderState::Open,
                GPUCommandEncoderState::EncodingComputePass,
            );
        }
    }

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

    /// https://gpuweb.github.io/gpuweb/#dom-gpucomputepassencoder-setpipeline
    fn SetPipeline(&self, pipeline: &GPUComputePipeline) {
        if let Some(raw_pass) = self.raw_pass.borrow_mut().as_mut() {
            unsafe { wgpu_compute_pass_set_pipeline(raw_pass, pipeline.id().0) };
        }
    }
}
