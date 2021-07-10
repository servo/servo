/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::GPUBufferBinding::GPUSize64;
use crate::dom::bindings::codegen::Bindings::GPUCommandEncoderBinding::{
    GPUBufferCopyView, GPUCommandBufferDescriptor, GPUCommandEncoderMethods,
    GPUComputePassDescriptor, GPUOrigin3D, GPURenderPassDescriptor, GPUStencilLoadValue,
    GPUStoreOp, GPUTextureCopyView, GPUTextureDataLayout,
};
use crate::dom::bindings::codegen::Bindings::GPUTextureBinding::GPUExtent3D;
use crate::dom::bindings::codegen::UnionTypes::{
    GPULoadOpOrDoubleSequenceOrGPUColorDict as GPUColorLoad, GPULoadOpOrFloat,
};
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::USVString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::gpubuffer::GPUBuffer;
use crate::dom::gpucommandbuffer::GPUCommandBuffer;
use crate::dom::gpucomputepassencoder::GPUComputePassEncoder;
use crate::dom::gpudevice::{convert_texture_size_to_dict, convert_texture_size_to_wgt, GPUDevice};
use crate::dom::gpurenderpassencoder::GPURenderPassEncoder;
use dom_struct::dom_struct;
use std::borrow::Cow;
use std::cell::Cell;
use std::collections::HashSet;
use webgpu::wgpu::command as wgpu_com;
use webgpu::{self, wgt, WebGPU, WebGPURequest};

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
    label: DomRefCell<Option<USVString>>,
    encoder: webgpu::WebGPUCommandEncoder,
    buffers: DomRefCell<HashSet<DomRoot<GPUBuffer>>>,
    state: DomRefCell<GPUCommandEncoderState>,
    device: Dom<GPUDevice>,
    valid: Cell<bool>,
}

impl GPUCommandEncoder {
    pub fn new_inherited(
        channel: WebGPU,
        device: &GPUDevice,
        encoder: webgpu::WebGPUCommandEncoder,
        label: Option<USVString>,
    ) -> Self {
        Self {
            channel,
            reflector_: Reflector::new(),
            label: DomRefCell::new(label),
            device: Dom::from_ref(device),
            encoder,
            buffers: DomRefCell::new(HashSet::new()),
            state: DomRefCell::new(GPUCommandEncoderState::Open),
            valid: Cell::new(true),
        }
    }

    pub fn new(
        global: &GlobalScope,
        channel: WebGPU,
        device: &GPUDevice,
        encoder: webgpu::WebGPUCommandEncoder,
        label: Option<USVString>,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(GPUCommandEncoder::new_inherited(
                channel, device, encoder, label,
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
    fn GetLabel(&self) -> Option<USVString> {
        self.label.borrow().clone()
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn SetLabel(&self, value: Option<USVString>) {
        *self.label.borrow_mut() = value;
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpucommandencoder-begincomputepass
    fn BeginComputePass(
        &self,
        descriptor: &GPUComputePassDescriptor,
    ) -> DomRoot<GPUComputePassEncoder> {
        self.set_state(
            GPUCommandEncoderState::EncodingComputePass,
            GPUCommandEncoderState::Open,
        );

        let compute_pass = if !self.valid.get() {
            None
        } else {
            Some(wgpu_com::ComputePass::new(self.encoder.0))
        };

        GPUComputePassEncoder::new(
            &self.global(),
            self.channel.clone(),
            &self,
            compute_pass,
            descriptor.parent.label.as_ref().cloned(),
        )
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

        let render_pass = if !self.valid.get() {
            None
        } else {
            let depth_stencil = descriptor.depthStencilAttachment.as_ref().map(|depth| {
                let (depth_load_op, clear_depth) = match depth.depthLoadValue {
                    GPULoadOpOrFloat::GPULoadOp(_) => (wgpu_com::LoadOp::Load, 0.0f32),
                    GPULoadOpOrFloat::Float(f) => (wgpu_com::LoadOp::Clear, *f),
                };
                let (stencil_load_op, clear_stencil) = match depth.stencilLoadValue {
                    GPUStencilLoadValue::GPULoadOp(_) => (wgpu_com::LoadOp::Load, 0u32),
                    GPUStencilLoadValue::RangeEnforcedUnsignedLong(l) => {
                        (wgpu_com::LoadOp::Clear, l)
                    },
                };
                let depth_channel = wgpu_com::PassChannel {
                    load_op: depth_load_op,
                    store_op: match depth.depthStoreOp {
                        GPUStoreOp::Store => wgpu_com::StoreOp::Store,
                        GPUStoreOp::Clear => wgpu_com::StoreOp::Clear,
                    },
                    clear_value: clear_depth,
                    read_only: depth.depthReadOnly,
                };
                let stencil_channel = wgpu_com::PassChannel {
                    load_op: stencil_load_op,
                    store_op: match depth.stencilStoreOp {
                        GPUStoreOp::Store => wgpu_com::StoreOp::Store,
                        GPUStoreOp::Clear => wgpu_com::StoreOp::Clear,
                    },
                    clear_value: clear_stencil,
                    read_only: depth.stencilReadOnly,
                };
                wgpu_com::DepthStencilAttachmentDescriptor {
                    attachment: depth.attachment.id().0,
                    depth: depth_channel,
                    stencil: stencil_channel,
                }
            });

            let desc = wgpu_com::RenderPassDescriptor {
                color_attachments: Cow::Owned(
                    descriptor
                        .colorAttachments
                        .iter()
                        .map(|color| {
                            let (load_op, clear_value) = match color.loadValue {
                                GPUColorLoad::GPULoadOp(_) => {
                                    (wgpu_com::LoadOp::Load, wgt::Color::TRANSPARENT)
                                },
                                GPUColorLoad::DoubleSequence(ref s) => {
                                    let mut w = s.clone();
                                    if w.len() < 3 {
                                        w.resize(3, Finite::wrap(0.0f64));
                                    }
                                    w.resize(4, Finite::wrap(1.0f64));
                                    (
                                        wgpu_com::LoadOp::Clear,
                                        wgt::Color {
                                            r: *w[0],
                                            g: *w[1],
                                            b: *w[2],
                                            a: *w[3],
                                        },
                                    )
                                },
                                GPUColorLoad::GPUColorDict(ref d) => (
                                    wgpu_com::LoadOp::Clear,
                                    wgt::Color {
                                        r: *d.r,
                                        g: *d.g,
                                        b: *d.b,
                                        a: *d.a,
                                    },
                                ),
                            };
                            let channel = wgpu_com::PassChannel {
                                load_op,
                                store_op: match color.storeOp {
                                    GPUStoreOp::Store => wgpu_com::StoreOp::Store,
                                    GPUStoreOp::Clear => wgpu_com::StoreOp::Clear,
                                },
                                clear_value,
                                read_only: false,
                            };
                            wgpu_com::ColorAttachmentDescriptor {
                                attachment: color.attachment.id().0,
                                resolve_target: color.resolveTarget.as_ref().map(|t| t.id().0),
                                channel,
                            }
                        })
                        .collect::<Vec<_>>(),
                ),
                depth_stencil_attachment: depth_stencil.as_ref(),
            };
            Some(wgpu_com::RenderPass::new(self.encoder.0, desc))
        };

        GPURenderPassEncoder::new(
            &self.global(),
            self.channel.clone(),
            render_pass,
            &self,
            descriptor.parent.label.as_ref().cloned(),
        )
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
        if !(*self.state.borrow() == GPUCommandEncoderState::Open) {
            self.valid.set(false);
            return;
        }

        self.buffers.borrow_mut().insert(DomRoot::from_ref(source));
        self.buffers
            .borrow_mut()
            .insert(DomRoot::from_ref(destination));
        self.channel
            .0
            .send((
                None,
                WebGPURequest::CopyBufferToBuffer {
                    command_encoder_id: self.encoder.0,
                    source_id: source.id().0,
                    source_offset,
                    destination_id: destination.id().0,
                    destination_offset,
                    size,
                },
            ))
            .expect("Failed to send CopyBufferToBuffer");
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpucommandencoder-copybuffertotexture
    fn CopyBufferToTexture(
        &self,
        source: &GPUBufferCopyView,
        destination: &GPUTextureCopyView,
        copy_size: GPUExtent3D,
    ) {
        if !(*self.state.borrow() == GPUCommandEncoderState::Open) {
            self.valid.set(false);
            return;
        }

        self.buffers
            .borrow_mut()
            .insert(DomRoot::from_ref(&*source.buffer));

        self.channel
            .0
            .send((
                None,
                WebGPURequest::CopyBufferToTexture {
                    command_encoder_id: self.encoder.0,
                    source: convert_buffer_cv(source),
                    destination: convert_texture_cv(destination),
                    copy_size: convert_texture_size_to_wgt(&convert_texture_size_to_dict(
                        &copy_size,
                    )),
                },
            ))
            .expect("Failed to send CopyBufferToTexture");
    }

    /// https://gpuweb.github.io/gpuweb/#GPUCommandEncoder-copyTextureToBuffer
    fn CopyTextureToBuffer(
        &self,
        source: &GPUTextureCopyView,
        destination: &GPUBufferCopyView,
        copy_size: GPUExtent3D,
    ) {
        if !(*self.state.borrow() == GPUCommandEncoderState::Open) {
            self.valid.set(false);
            return;
        }

        self.buffers
            .borrow_mut()
            .insert(DomRoot::from_ref(&*destination.buffer));

        self.channel
            .0
            .send((
                None,
                WebGPURequest::CopyTextureToBuffer {
                    command_encoder_id: self.encoder.0,
                    source: convert_texture_cv(source),
                    destination: convert_buffer_cv(destination),
                    copy_size: convert_texture_size_to_wgt(&convert_texture_size_to_dict(
                        &copy_size,
                    )),
                },
            ))
            .expect("Failed to send CopyTextureToBuffer");
    }

    /// https://gpuweb.github.io/gpuweb/#GPUCommandEncoder-copyTextureToTexture
    fn CopyTextureToTexture(
        &self,
        source: &GPUTextureCopyView,
        destination: &GPUTextureCopyView,
        copy_size: GPUExtent3D,
    ) {
        if !(*self.state.borrow() == GPUCommandEncoderState::Open) {
            self.valid.set(false);
            return;
        }

        self.channel
            .0
            .send((
                None,
                WebGPURequest::CopyTextureToTexture {
                    command_encoder_id: self.encoder.0,
                    source: convert_texture_cv(source),
                    destination: convert_texture_cv(destination),
                    copy_size: convert_texture_size_to_wgt(&convert_texture_size_to_dict(
                        &copy_size,
                    )),
                },
            ))
            .expect("Failed to send CopyTextureToTexture");
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpucommandencoder-finish
    fn Finish(&self, descriptor: &GPUCommandBufferDescriptor) -> DomRoot<GPUCommandBuffer> {
        self.channel
            .0
            .send((
                self.device.use_current_scope(),
                WebGPURequest::CommandEncoderFinish {
                    command_encoder_id: self.encoder.0,
                    device_id: self.device.id().0,
                    is_error: !self.valid.get(),
                    // TODO(zakorgy): We should use `_descriptor` here after it's not empty
                    // and the underlying wgpu-core struct is serializable
                },
            ))
            .expect("Failed to send Finish");

        *self.state.borrow_mut() = GPUCommandEncoderState::Closed;
        let buffer = webgpu::WebGPUCommandBuffer(self.encoder.0);
        GPUCommandBuffer::new(
            &self.global(),
            self.channel.clone(),
            buffer,
            self.buffers.borrow_mut().drain().collect(),
            descriptor.parent.label.as_ref().cloned(),
        )
    }
}

fn convert_buffer_cv(buffer_cv: &GPUBufferCopyView) -> wgpu_com::BufferCopyView {
    wgpu_com::BufferCopyView {
        buffer: buffer_cv.buffer.id().0,
        layout: convert_texture_data_layout(&buffer_cv.parent),
    }
}

pub fn convert_texture_cv(texture_cv: &GPUTextureCopyView) -> wgpu_com::TextureCopyView {
    wgpu_com::TextureCopyView {
        texture: texture_cv.texture.id().0,
        mip_level: texture_cv.mipLevel,
        origin: match texture_cv.origin {
            GPUOrigin3D::RangeEnforcedUnsignedLongSequence(ref v) => {
                let mut w = v.clone();
                w.resize(3, 0);
                wgt::Origin3d {
                    x: w[0],
                    y: w[1],
                    z: w[2],
                }
            },
            GPUOrigin3D::GPUOrigin3DDict(ref d) => wgt::Origin3d {
                x: d.x,
                y: d.y,
                z: d.z,
            },
        },
    }
}

pub fn convert_texture_data_layout(data_layout: &GPUTextureDataLayout) -> wgt::TextureDataLayout {
    wgt::TextureDataLayout {
        offset: data_layout.offset as wgt::BufferAddress,
        bytes_per_row: data_layout.bytesPerRow,
        rows_per_image: data_layout.rowsPerImage,
    }
}
