/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::GPUBufferBinding::GPUSize64;
use crate::dom::bindings::codegen::Bindings::GPUCommandEncoderBinding::{
    GPUCommandBufferDescriptor, GPUCommandEncoderMethods, GPUComputePassDescriptor,
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
use std::cell::Cell;
use std::collections::HashSet;
use webgpu::wgpu::resource::BufferUsage;
use webgpu::{WebGPU, WebGPUCommandEncoder, WebGPURequest};

const BUFFER_COPY_ALIGN_MASK: u64 = 3;

// https://gpuweb.github.io/gpuweb/#enumdef-encoder-state
#[derive(MallocSizeOf, PartialEq)]
#[allow(dead_code)]
pub enum GPUCommandEncoderState {
    Open,
    EncodingRenderPass,
    EncodingComputePass,
    Closed,
}

#[dom_struct]
pub struct GPUCommandEncoder {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "defined in webgpu"]
    channel: WebGPU,
    label: DomRefCell<Option<DOMString>>,
    encoder: WebGPUCommandEncoder,
    buffers: DomRefCell<HashSet<DomRoot<GPUBuffer>>>,
    state: DomRefCell<GPUCommandEncoderState>,
    valid: Cell<bool>,
}

impl GPUCommandEncoder {
    pub fn new_inherited(
        channel: WebGPU,
        encoder: WebGPUCommandEncoder,
        valid: bool,
    ) -> GPUCommandEncoder {
        GPUCommandEncoder {
            channel,
            reflector_: Reflector::new(),
            label: DomRefCell::new(None),
            encoder,
            buffers: DomRefCell::new(HashSet::new()),
            state: DomRefCell::new(GPUCommandEncoderState::Open),
            valid: Cell::new(valid),
        }
    }

    pub fn new(
        global: &GlobalScope,
        channel: WebGPU,
        encoder: WebGPUCommandEncoder,
        valid: bool,
    ) -> DomRoot<GPUCommandEncoder> {
        reflect_dom_object(
            Box::new(GPUCommandEncoder::new_inherited(channel, encoder, valid)),
            global,
        )
    }
}

impl GPUCommandEncoder {
    pub fn id(&self) -> WebGPUCommandEncoder {
        self.encoder
    }

    pub fn set_state(&self, set: GPUCommandEncoderState, expect: GPUCommandEncoderState) {
        if *self.state.borrow() == expect {
            *self.state.borrow_mut() = set;
        } else {
            self.valid.set(false);
            *self.state.borrow_mut() = GPUCommandEncoderState::Closed;
        }
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
        self.set_state(
            GPUCommandEncoderState::EncodingComputePass,
            GPUCommandEncoderState::Open,
        );
        GPUComputePassEncoder::new(&self.global(), self.channel.clone(), &self)
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpucommandencoder-copybuffertobuffer
    fn CopyBufferToBuffer(
        &self,
        source: &GPUBuffer,
        source_offset: GPUSize64,
        destination: &GPUBuffer,
        destination_offset: GPUSize64,
        size: GPUSize64,
    ) {
        let mut valid = match source_offset.checked_add(size) {
            Some(_) => true,
            None => false,
        };
        valid &= match destination_offset.checked_add(size) {
            Some(_) => true,
            None => false,
        };
        valid &= match BufferUsage::from_bits(source.usage()) {
            Some(usage) => usage.contains(BufferUsage::COPY_SRC),
            None => false,
        };
        valid &= match BufferUsage::from_bits(destination.usage()) {
            Some(usage) => usage.contains(BufferUsage::COPY_DST),
            None => false,
        };
        valid &= (*self.state.borrow() == GPUCommandEncoderState::Open) &&
            source.valid() &&
            destination.valid() &
                !(size & BUFFER_COPY_ALIGN_MASK == 0) &
                !(source_offset & BUFFER_COPY_ALIGN_MASK == 0) &
                !(destination_offset & BUFFER_COPY_ALIGN_MASK == 0) &
                (source.size() >= source_offset + size) &
                (destination.size() >= destination_offset + size);

        if source.id().0 == destination.id().0 {
            //TODO: maybe forbid this case based on https://github.com/gpuweb/gpuweb/issues/783
            valid &= source_offset > destination_offset + size ||
                source_offset + size < destination_offset;
        }

        if !valid {
            // TODO: Record an error in the current scope.
            self.valid.set(false);
            return;
        }

        self.buffers.borrow_mut().insert(DomRoot::from_ref(source));
        self.buffers
            .borrow_mut()
            .insert(DomRoot::from_ref(destination));
        self.channel
            .0
            .send(WebGPURequest::CopyBufferToBuffer {
                command_encoder_id: self.encoder.0,
                source_id: source.id().0,
                source_offset,
                destination_id: destination.id().0,
                destination_offset,
                size,
            })
            .expect("Failed to send CopyBufferToBuffer");
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpucommandencoder-finish
    fn Finish(&self, _descriptor: &GPUCommandBufferDescriptor) -> DomRoot<GPUCommandBuffer> {
        let (sender, receiver) = ipc::channel().unwrap();
        self.channel
            .0
            .send(WebGPURequest::CommandEncoderFinish {
                sender,
                command_encoder_id: self.encoder.0,
                // TODO(zakorgy): We should use `_descriptor` here after it's not empty
                // and the underlying wgpu-core struct is serializable
            })
            .expect("Failed to send Finish");

        *self.state.borrow_mut() = GPUCommandEncoderState::Closed;
        let buffer = receiver.recv().unwrap();
        GPUCommandBuffer::new(
            &self.global(),
            self.channel.clone(),
            buffer,
            self.buffers.borrow_mut().drain().collect(),
        )
    }
}
