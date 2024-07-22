/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::collections::HashSet;

use dom_struct::dom_struct;
use webgpu::wgc::command as wgpu_com;
use webgpu::{self, wgt, WebGPU, WebGPUComputePass, WebGPURenderPass, WebGPURequest};

use super::gpuconvert::convert_label;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::{
    GPUCommandBufferDescriptor, GPUCommandEncoderMethods, GPUComputePassDescriptor, GPUExtent3D,
    GPUImageCopyBuffer, GPUImageCopyTexture, GPURenderPassDescriptor, GPUSize64,
};
use crate::dom::bindings::codegen::UnionTypes::DoubleSequenceOrGPUColorDict;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::USVString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::gpubuffer::GPUBuffer;
use crate::dom::gpucommandbuffer::GPUCommandBuffer;
use crate::dom::gpucomputepassencoder::GPUComputePassEncoder;
use crate::dom::gpuconvert::{
    convert_ic_buffer, convert_ic_texture, convert_load_op, convert_store_op,
    convert_texture_size_to_dict, convert_texture_size_to_wgt,
};
use crate::dom::gpudevice::GPUDevice;
use crate::dom::gpurenderpassencoder::GPURenderPassEncoder;

#[dom_struct]
pub struct GPUCommandEncoder {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "defined in webgpu"]
    #[no_trace]
    channel: WebGPU,
    label: DomRefCell<USVString>,
    #[no_trace]
    encoder: webgpu::WebGPUCommandEncoder,
    buffers: DomRefCell<HashSet<DomRoot<GPUBuffer>>>,
    device: Dom<GPUDevice>,
    valid: Cell<bool>,
}

impl GPUCommandEncoder {
    pub fn new_inherited(
        channel: WebGPU,
        device: &GPUDevice,
        encoder: webgpu::WebGPUCommandEncoder,
        label: USVString,
    ) -> Self {
        Self {
            channel,
            reflector_: Reflector::new(),
            label: DomRefCell::new(label),
            device: Dom::from_ref(device),
            encoder,
            buffers: DomRefCell::new(HashSet::new()),
            valid: Cell::new(true),
        }
    }

    pub fn new(
        global: &GlobalScope,
        channel: WebGPU,
        device: &GPUDevice,
        encoder: webgpu::WebGPUCommandEncoder,
        label: USVString,
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

    pub fn device_id(&self) -> webgpu::WebGPUDevice {
        self.device.id()
    }
}

impl GPUCommandEncoderMethods for GPUCommandEncoder {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn Label(&self) -> USVString {
        self.label.borrow().clone()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn SetLabel(&self, value: USVString) {
        *self.label.borrow_mut() = value;
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpucommandencoder-begincomputepass>
    fn BeginComputePass(
        &self,
        descriptor: &GPUComputePassDescriptor,
    ) -> DomRoot<GPUComputePassEncoder> {
        let compute_pass_id = self
            .global()
            .wgpu_id_hub()
            .create_compute_pass_id(self.device.id().0.backend());

        if let Err(e) = self.channel.0.send(WebGPURequest::BeginComputePass {
            command_encoder_id: self.id().0,
            compute_pass_id,
            label: convert_label(&descriptor.parent),
            device_id: self.device.id().0,
        }) {
            warn!("Failed to send WebGPURequest::BeginComputePass {e:?}");
        }

        GPUComputePassEncoder::new(
            &self.global(),
            self.channel.clone(),
            self,
            WebGPUComputePass(compute_pass_id),
            descriptor.parent.label.clone().unwrap_or_default(),
        )
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpucommandencoder-beginrenderpass>
    fn BeginRenderPass(
        &self,
        descriptor: &GPURenderPassDescriptor,
    ) -> DomRoot<GPURenderPassEncoder> {
        let depth_stencil_attachment = descriptor.depthStencilAttachment.as_ref().map(|depth| {
            wgpu_com::RenderPassDepthStencilAttachment {
                depth: wgpu_com::PassChannel {
                    load_op: convert_load_op(depth.depthLoadOp),
                    store_op: convert_store_op(depth.depthStoreOp),
                    clear_value: *depth.depthClearValue.unwrap_or_default(),
                    read_only: depth.depthReadOnly,
                },
                stencil: wgpu_com::PassChannel {
                    load_op: convert_load_op(depth.stencilLoadOp),
                    store_op: convert_store_op(depth.stencilStoreOp),
                    clear_value: depth.stencilClearValue,
                    read_only: depth.stencilReadOnly,
                },
                view: depth.view.id().0,
            }
        });

        let color_attachments = descriptor
            .colorAttachments
            .iter()
            .map(|color| {
                let channel = wgpu_com::PassChannel {
                    load_op: convert_load_op(Some(color.loadOp)),
                    store_op: convert_store_op(Some(color.storeOp)),
                    clear_value: if let Some(clear_val) = &color.clearValue {
                        match clear_val {
                            DoubleSequenceOrGPUColorDict::DoubleSequence(s) => {
                                let mut w = s.clone();
                                if w.len() < 3 {
                                    w.resize(3, Finite::wrap(0.0f64));
                                }
                                w.resize(4, Finite::wrap(1.0f64));
                                wgt::Color {
                                    r: *w[0],
                                    g: *w[1],
                                    b: *w[2],
                                    a: *w[3],
                                }
                            },
                            DoubleSequenceOrGPUColorDict::GPUColorDict(d) => wgt::Color {
                                r: *d.r,
                                g: *d.g,
                                b: *d.b,
                                a: *d.a,
                            },
                        }
                    } else {
                        wgt::Color::TRANSPARENT
                    },
                    read_only: false,
                };
                Some(wgpu_com::RenderPassColorAttachment {
                    resolve_target: color.resolveTarget.as_ref().map(|t| t.id().0),
                    channel,
                    view: color.view.id().0,
                })
            })
            .collect::<Vec<_>>();
        let render_pass_id = self
            .global()
            .wgpu_id_hub()
            .create_render_pass_id(self.device.id().0.backend());

        if let Err(e) = self.channel.0.send(WebGPURequest::BeginRenderPass {
            command_encoder_id: self.id().0,
            render_pass_id,
            label: convert_label(&descriptor.parent),
            depth_stencil_attachment,
            color_attachments,
            device_id: self.device.id().0,
        }) {
            warn!("Failed to send WebGPURequest::BeginRenderPass {e:?}");
        }

        GPURenderPassEncoder::new(
            &self.global(),
            self.channel.clone(),
            WebGPURenderPass(render_pass_id),
            self,
            descriptor.parent.label.clone().unwrap_or_default(),
        )
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpucommandencoder-copybuffertobuffer>
    fn CopyBufferToBuffer(
        &self,
        source: &GPUBuffer,
        source_offset: GPUSize64,
        destination: &GPUBuffer,
        destination_offset: GPUSize64,
        size: GPUSize64,
    ) {
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

    /// <https://gpuweb.github.io/gpuweb/#dom-gpucommandencoder-copybuffertotexture>
    fn CopyBufferToTexture(
        &self,
        source: &GPUImageCopyBuffer,
        destination: &GPUImageCopyTexture,
        copy_size: GPUExtent3D,
    ) {
        self.buffers
            .borrow_mut()
            .insert(DomRoot::from_ref(&*source.buffer));

        self.channel
            .0
            .send(WebGPURequest::CopyBufferToTexture {
                command_encoder_id: self.encoder.0,
                source: convert_ic_buffer(source),
                destination: convert_ic_texture(destination),
                copy_size: convert_texture_size_to_wgt(&convert_texture_size_to_dict(&copy_size)),
            })
            .expect("Failed to send CopyBufferToTexture");
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpucommandencoder-copybuffertotexture>
    fn CopyTextureToBuffer(
        &self,
        source: &GPUImageCopyTexture,
        destination: &GPUImageCopyBuffer,
        copy_size: GPUExtent3D,
    ) {
        self.buffers
            .borrow_mut()
            .insert(DomRoot::from_ref(&*destination.buffer));

        self.channel
            .0
            .send(WebGPURequest::CopyTextureToBuffer {
                command_encoder_id: self.encoder.0,
                source: convert_ic_texture(source),
                destination: convert_ic_buffer(destination),
                copy_size: convert_texture_size_to_wgt(&convert_texture_size_to_dict(&copy_size)),
            })
            .expect("Failed to send CopyTextureToBuffer");
    }

    /// <https://gpuweb.github.io/gpuweb/#GPUCommandEncoder-copyTextureToTexture>
    fn CopyTextureToTexture(
        &self,
        source: &GPUImageCopyTexture,
        destination: &GPUImageCopyTexture,
        copy_size: GPUExtent3D,
    ) {
        self.channel
            .0
            .send(WebGPURequest::CopyTextureToTexture {
                command_encoder_id: self.encoder.0,
                source: convert_ic_texture(source),
                destination: convert_ic_texture(destination),
                copy_size: convert_texture_size_to_wgt(&convert_texture_size_to_dict(&copy_size)),
            })
            .expect("Failed to send CopyTextureToTexture");
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpucommandencoder-finish>
    fn Finish(&self, descriptor: &GPUCommandBufferDescriptor) -> DomRoot<GPUCommandBuffer> {
        self.channel
            .0
            .send(WebGPURequest::CommandEncoderFinish {
                command_encoder_id: self.encoder.0,
                device_id: self.device.id().0,
                is_error: !self.valid.get(),
                // TODO(zakorgy): We should use `_descriptor` here after it's not empty
                // and the underlying wgpu-core struct is serializable
            })
            .expect("Failed to send Finish");

        let buffer = webgpu::WebGPUCommandBuffer(self.encoder.0.into_command_buffer_id());
        GPUCommandBuffer::new(
            &self.global(),
            self.channel.clone(),
            buffer,
            self.buffers.borrow_mut().drain().collect(),
            descriptor.parent.label.clone().unwrap_or_default(),
        )
    }
}
