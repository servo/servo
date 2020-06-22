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
    ColorAttachmentDescriptor, DepthStencilAttachmentDescriptor, RenderPass, RenderPassDescriptor,
};
use webgpu::{self, wgt, WebGPU, WebGPUDevice, WebGPURequest};

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
                let (load_op, clear_value) = match color.loadValue {
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
                let channel = wgt::PassChannel {
                    load_op,
                    store_op: match color.storeOp {
                        GPUStoreOp::Store => wgt::StoreOp::Store,
                        GPUStoreOp::Clear => wgt::StoreOp::Clear,
                    },
                    clear_value,
                    read_only: false,
                };
                ColorAttachmentDescriptor {
                    attachment: color.attachment.id().0,
                    resolve_target: color.resolveTarget.as_ref().map(|t| t.id().0),
                    channel,
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
            let depth_channel = wgt::PassChannel {
                load_op: depth_load_op,
                store_op: match depth.depthStoreOp {
                    GPUStoreOp::Store => wgt::StoreOp::Store,
                    GPUStoreOp::Clear => wgt::StoreOp::Clear,
                },
                clear_value: clear_depth,
                read_only: depth.depthReadOnly,
            };
            let stencil_channel = wgt::PassChannel {
                load_op: stencil_load_op,
                store_op: match depth.stencilStoreOp {
                    GPUStoreOp::Store => wgt::StoreOp::Store,
                    GPUStoreOp::Clear => wgt::StoreOp::Clear,
                },
                clear_value: clear_stencil,
                read_only: depth.stencilReadOnly,
            };
            DepthStencilAttachmentDescriptor {
                attachment: depth.attachment.id().0,
                depth: depth_channel,
                stencil: stencil_channel,
            }
        });

        let desc = RenderPassDescriptor {
            color_attachments: colors.as_slice(),
            depth_stencil_attachment: depth_stencil.as_ref(),
        };

        let render_pass = RenderPass::new(self.encoder.0, desc);

        GPURenderPassEncoder::new(&self.global(), self.channel.clone(), render_pass, &self)
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
        let valid = source.is_valid() &&
            destination.is_valid() &&
            *self.state.borrow() == GPUCommandEncoderState::Open;

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
