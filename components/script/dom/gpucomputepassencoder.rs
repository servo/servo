/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use webgpu::wgpu::command::{compute_ffi as wgpu_comp, ComputePass};
use webgpu::{WebGPU, WebGPURequest};

use super::bindings::error::Fallible;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::GPUComputePassEncoderMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::USVString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::gpubindgroup::GPUBindGroup;
use crate::dom::gpubuffer::GPUBuffer;
use crate::dom::gpucommandencoder::{GPUCommandEncoder, GPUCommandEncoderState};
use crate::dom::gpucomputepipeline::GPUComputePipeline;

#[dom_struct]
pub struct GPUComputePassEncoder {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "defined in webgpu"]
    #[no_trace]
    channel: WebGPU,
    label: DomRefCell<USVString>,
    #[ignore_malloc_size_of = "defined in wgpu-core"]
    #[no_trace]
    compute_pass: DomRefCell<Option<ComputePass>>,
    command_encoder: Dom<GPUCommandEncoder>,
}

impl GPUComputePassEncoder {
    fn new_inherited(
        channel: WebGPU,
        parent: &GPUCommandEncoder,
        compute_pass: Option<ComputePass>,
        label: USVString,
    ) -> Self {
        Self {
            channel,
            reflector_: Reflector::new(),
            label: DomRefCell::new(label),
            compute_pass: DomRefCell::new(compute_pass),
            command_encoder: Dom::from_ref(parent),
        }
    }

    pub fn new(
        global: &GlobalScope,
        channel: WebGPU,
        parent: &GPUCommandEncoder,
        compute_pass: Option<ComputePass>,
        label: USVString,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(GPUComputePassEncoder::new_inherited(
                channel,
                parent,
                compute_pass,
                label,
            )),
            global,
        )
    }
}

impl GPUComputePassEncoderMethods for GPUComputePassEncoder {
    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn Label(&self) -> USVString {
        self.label.borrow().clone()
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn SetLabel(&self, value: USVString) {
        *self.label.borrow_mut() = value;
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpucomputepassencoder-dispatchworkgroups
    fn DispatchWorkgroups(&self, x: u32, y: u32, z: u32) {
        if let Some(compute_pass) = self.compute_pass.borrow_mut().as_mut() {
            wgpu_comp::wgpu_compute_pass_dispatch_workgroups(compute_pass, x, y, z);
        }
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpucomputepassencoder-dispatchworkgroupsindirect
    fn DispatchWorkgroupsIndirect(&self, indirect_buffer: &GPUBuffer, indirect_offset: u64) {
        if let Some(compute_pass) = self.compute_pass.borrow_mut().as_mut() {
            wgpu_comp::wgpu_compute_pass_dispatch_workgroups_indirect(
                compute_pass,
                indirect_buffer.id().0,
                indirect_offset,
            );
        }
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpurenderpassencoder-endpass
    fn End(&self) -> Fallible<()> {
        let compute_pass = self.compute_pass.borrow_mut().take();
        self.channel
            .0
            .send((
                None,
                WebGPURequest::RunComputePass {
                    command_encoder_id: self.command_encoder.id().0,
                    compute_pass,
                },
            ))
            .expect("Failed to send RunComputePass"); //TODO: handle error

        self.command_encoder.set_state(
            GPUCommandEncoderState::Open,
            GPUCommandEncoderState::EncodingComputePass,
        );
        Ok(())
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuprogrammablepassencoder-setbindgroup
    #[allow(unsafe_code)]
    fn SetBindGroup(&self, index: u32, bind_group: &GPUBindGroup, dynamic_offsets: Vec<u32>) {
        if let Some(compute_pass) = self.compute_pass.borrow_mut().as_mut() {
            unsafe {
                wgpu_comp::wgpu_compute_pass_set_bind_group(
                    compute_pass,
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
        if let Some(compute_pass) = self.compute_pass.borrow_mut().as_mut() {
            wgpu_comp::wgpu_compute_pass_set_pipeline(compute_pass, pipeline.id().0);
        }
    }
}
