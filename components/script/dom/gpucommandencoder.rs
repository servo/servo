/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::GPUBufferBinding::GPUSize64;
use crate::dom::bindings::codegen::Bindings::GPUCommandEncoderBinding::{
    GPUCommandBufferDescriptor, GPUCommandEncoderMethods, GPUComputePassDescriptor,
    GPURenderPassDescriptor, GPUStencilLoadValue, GPUStoreOp,
};
use crate::dom::bindings::codegen::UnionTypes::{
    GPULoadOpOrDoubleSequenceOrGPUColorDict as GPUColorLoad, GPULoadOpOrFloat,
};
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::gpubuffer::GPUBuffer;
use crate::dom::gpucommandbuffer::GPUCommandBuffer;
use crate::dom::gpucomputepassencoder::GPUComputePassEncoder;
use crate::dom::gpurenderpassencoder::GPURenderPassEncoder;
use dom_struct::dom_struct;
use std::cell::Cell;
use std::collections::HashSet;
use webgpu::wgpu::command::{
    RawPass, RenderPassColorAttachmentDescriptor, RenderPassDepthStencilAttachmentDescriptor,
    RenderPassDescriptor,
};
use webgpu::{self, wgt, WebGPU, WebGPUDevice, WebGPURequest};

const BUFFER_COPY_ALIGN_MASK: u64 = 3;

// https://gpuweb.github.io/gpuweb/#enumdef-encoder-state
#[derive(MallocSizeOf, PartialEq)]
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
    encoder: webgpu::WebGPUCommandEncoder,
    buffers: DomRefCell<HashSet<DomRoot<GPUBuffer>>>,
    state: DomRefCell<GPUCommandEncoderState>,
    device: WebGPUDevice,
    valid: Cell<bool>,
}

impl GPUCommandEncoder {
    pub fn new_inherited(
        channel: WebGPU,
        device: WebGPUDevice,
        encoder: webgpu::WebGPUCommandEncoder,
        valid: bool,
    ) -> Self {
        Self {
            channel,
            reflector_: Reflector::new(),
            label: DomRefCell::new(None),
            device,
            encoder,
            buffers: DomRefCell::new(HashSet::new()),
            state: DomRefCell::new(GPUCommandEncoderState::Open),
            valid: Cell::new(valid),
        }
    }

    pub fn new(
        global: &GlobalScope,
        channel: WebGPU,
        device: WebGPUDevice,
        encoder: webgpu::WebGPUCommandEncoder,
        valid: bool,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(GPUCommandEncoder::new_inherited(
                channel, device, encoder, valid,
            )),
            global,
        )
    }
}

impl GPUCommandEncoder {
    pub fn id(&self) -> webgpu::WebGPUCommandEncoder {
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

    #[allow(unsafe_code)]
    /// https://gpuweb.github.io/gpuweb/#dom-gpucommandencoder-beginrenderpass
    fn BeginRenderPass(
        &self,
        descriptor: &GPURenderPassDescriptor,
    ) -> DomRoot<GPURenderPassEncoder> {
        self.set_state(
            GPUCommandEncoderState::EncodingRenderPass,
            GPUCommandEncoderState::Open,
        );

        let colors = descriptor
            .colorAttachments
            .iter()
            .map(|color| {
                let (load_op, clear_color) = match color.loadValue {
                    GPUColorLoad::GPULoadOp(_) => (wgt::LoadOp::Load, wgt::Color::TRANSPARENT),
                    GPUColorLoad::DoubleSequence(ref s) => (
                        wgt::LoadOp::Clear,
                        wgt::Color {
                            r: *s[0],
                            g: *s[1],
                            b: *s[2],
                            a: *s[3],
                        },
                    ),
                    GPUColorLoad::GPUColorDict(ref d) => (
                        wgt::LoadOp::Clear,
                        wgt::Color {
                            r: *d.r,
                            g: *d.g,
                            b: *d.b,
                            a: *d.a,
                        },
                    ),
                };
                RenderPassColorAttachmentDescriptor {
                    attachment: color.attachment.id().0,
                    resolve_target: color.resolveTarget.as_ref().map(|t| t.id().0),
                    load_op,
                    store_op: match color.storeOp {
                        GPUStoreOp::Store => wgt::StoreOp::Store,
                        GPUStoreOp::Clear => wgt::StoreOp::Clear,
                    },
                    clear_color,
                }
            })
            .collect::<Vec<_>>();

        let depth_stencil = descriptor.depthStencilAttachment.as_ref().map(|depth| {
            let (depth_load_op, clear_depth) = match depth.depthLoadValue {
                GPULoadOpOrFloat::GPULoadOp(_) => (wgt::LoadOp::Load, 0.0f32),
                GPULoadOpOrFloat::Float(f) => (wgt::LoadOp::Clear, *f),
            };
            let (stencil_load_op, clear_stencil) = match depth.stencilLoadValue {
                GPUStencilLoadValue::GPULoadOp(_) => (wgt::LoadOp::Load, 0u32),
                GPUStencilLoadValue::RangeEnforcedUnsignedLong(l) => (wgt::LoadOp::Clear, l),
            };
            RenderPassDepthStencilAttachmentDescriptor {
                attachment: depth.attachment.id().0,
                depth_load_op,
                depth_store_op: match depth.depthStoreOp {
                    GPUStoreOp::Store => wgt::StoreOp::Store,
                    GPUStoreOp::Clear => wgt::StoreOp::Clear,
                },
                clear_depth,
                depth_read_only: depth.depthReadOnly,
                stencil_load_op,
                stencil_store_op: match depth.stencilStoreOp {
                    GPUStoreOp::Store => wgt::StoreOp::Store,
                    GPUStoreOp::Clear => wgt::StoreOp::Clear,
                },
                clear_stencil,
                stencil_read_only: depth.stencilReadOnly,
            }
        });

        let desc = RenderPassDescriptor {
            color_attachments: colors.as_ptr(),
            color_attachments_length: colors.len(),
            depth_stencil_attachment: depth_stencil.as_ref(),
        };

        let raw_pass = unsafe { RawPass::new_render(self.id().0, &desc) };

        GPURenderPassEncoder::new(&self.global(), self.channel.clone(), raw_pass, &self)
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
        valid &= match wgt::BufferUsage::from_bits(source.usage()) {
            Some(usage) => usage.contains(wgt::BufferUsage::COPY_SRC),
            None => false,
        };
        valid &= match wgt::BufferUsage::from_bits(destination.usage()) {
            Some(usage) => usage.contains(wgt::BufferUsage::COPY_DST),
            None => false,
        };
        valid &= (*self.state.borrow() == GPUCommandEncoderState::Open) &&
            source.is_valid() &&
            destination.is_valid() &
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
        self.channel
            .0
            .send(WebGPURequest::CommandEncoderFinish {
                command_encoder_id: self.encoder.0,
                // TODO(zakorgy): We should use `_descriptor` here after it's not empty
                // and the underlying wgpu-core struct is serializable
            })
            .expect("Failed to send Finish");

        *self.state.borrow_mut() = GPUCommandEncoderState::Closed;
        let buffer = webgpu::WebGPUCommandBuffer(self.encoder.0);
        GPUCommandBuffer::new(
            &self.global(),
            self.channel.clone(),
            buffer,
            self.buffers.borrow_mut().drain().collect(),
        )
    }
}
