/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::GPUCommandEncoderBinding::{
    self, GPUCommandBufferDescriptor, GPUCommandEncoderMethods, GPUComputePassDescriptor,
};
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::gpubuffer::GPUBuffer;
use crate::dom::gpucommandbuffer::GPUCommandBuffer;
use crate::dom::gpucomputepassencoder::GPUComputePassEncoder;
use dom_struct::dom_struct;
use ipc_channel::ipc;
use std::collections::HashSet;
use webgpu::{WebGPU, WebGPUCommandEncoder, WebGPURequest};

#[dom_struct]
pub struct GPUCommandEncoder {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "defined in webgpu"]
    channel: WebGPU,
    label: DomRefCell<Option<DOMString>>,
    encoder: WebGPUCommandEncoder,
    buffers: DomRefCell<HashSet<DomRoot<GPUBuffer>>>,
}

impl GPUCommandEncoder {
    pub fn new_inherited(channel: WebGPU, encoder: WebGPUCommandEncoder) -> GPUCommandEncoder {
        GPUCommandEncoder {
            channel,
            reflector_: Reflector::new(),
            label: DomRefCell::new(None),
            encoder,
            buffers: DomRefCell::new(HashSet::new()),
        }
    }

    pub fn new(
        global: &GlobalScope,
        channel: WebGPU,
        encoder: WebGPUCommandEncoder,
    ) -> DomRoot<GPUCommandEncoder> {
        reflect_dom_object(
            Box::new(GPUCommandEncoder::new_inherited(channel, encoder)),
            global,
            GPUCommandEncoderBinding::Wrap,
        )
    }
}

impl GPUCommandEncoderMethods for GPUCommandEncoder {
    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn GetLabel(&self) -> Option<DOMString> {
        self.label.borrow().clone()
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn SetLabel(&self, value: Option<DOMString>) {
        *self.label.borrow_mut() = value;
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpucommandencoder-begincomputepass
    fn BeginComputePass(
        &self,
        _descriptor: &GPUComputePassDescriptor,
    ) -> DomRoot<GPUComputePassEncoder> {
        GPUComputePassEncoder::new(&self.global(), self.channel.clone(), self.encoder)
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpucommandencoder-copybuffertobuffer
    fn CopyBufferToBuffer(
        &self,
        source: &GPUBuffer,
        source_offset: u64,
        destination: &GPUBuffer,
        destination_offset: u64,
        size: u64,
    ) {
        self.buffers.borrow_mut().insert(DomRoot::from_ref(source));
        self.buffers
            .borrow_mut()
            .insert(DomRoot::from_ref(destination));
        self.channel
            .0
            .send(WebGPURequest::CopyBufferToBuffer(
                self.encoder.0,
                source.id().0,
                source_offset,
                destination.id().0,
                destination_offset,
                size,
            ))
            .expect("Failed to send CopyBufferToBuffer");
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpucommandencoder-finish
    fn Finish(&self, _descriptor: &GPUCommandBufferDescriptor) -> DomRoot<GPUCommandBuffer> {
        let (sender, receiver) = ipc::channel().unwrap();
        self.channel
            .0
            .send(WebGPURequest::CommandEncoderFinish(
                sender,
                self.encoder.0,
                // TODO(zakorgy): We should use `_descriptor` here after it's not empty
                // and the underlying wgpu-core struct is serializable
            ))
            .expect("Failed to send Finish");

        let buffer = receiver.recv().unwrap();
        GPUCommandBuffer::new(
            &self.global(),
            self.channel.clone(),
            buffer,
            self.buffers.borrow_mut().drain().collect(),
        )
    }
}
