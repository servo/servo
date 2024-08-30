/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use webgpu::wgc::command as wgpu_com;
use webgpu::{self, wgt, WebGPU, WebGPUComputePass, WebGPURenderPass, WebGPURequest};

use super::bindings::error::Fallible;
use super::gpuconvert::{convert_color, convert_label};
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::{
    GPUCommandBufferDescriptor, GPUCommandEncoderMethods, GPUComputePassDescriptor, GPUExtent3D,
    GPUImageCopyBuffer, GPUImageCopyTexture, GPURenderPassDescriptor, GPUSize64,
};
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::USVString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::gpubuffer::GPUBuffer;
use crate::dom::gpucommandbuffer::GPUCommandBuffer;
use crate::dom::gpucomputepassencoder::GPUComputePassEncoder;
use crate::dom::gpuconvert::{
    convert_ic_buffer, convert_ic_texture, convert_load_op, convert_store_op, convert_texture_size,
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
    device: Dom<GPUDevice>,
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
            descriptor.parent.label.clone(),
        )
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpucommandencoder-beginrenderpass>
    fn BeginRenderPass(
        &self,
        descriptor: &GPURenderPassDescriptor,
    ) -> Fallible<DomRoot<GPURenderPassEncoder>> {
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
            .map(|color| -> Fallible<_> {
                let channel = wgpu_com::PassChannel {
                    load_op: convert_load_op(Some(color.loadOp)),
                    store_op: convert_store_op(Some(color.storeOp)),
                    clear_value: color
                        .clearValue
                        .as_ref()
                        .map(|color| convert_color(color))
                        .transpose()?
                        .unwrap_or_default(),
                    read_only: false,
                };
                Ok(Some(wgpu_com::RenderPassColorAttachment {
                    resolve_target: color.resolveTarget.as_ref().map(|t| t.id().0),
                    channel,
                    view: color.view.id().0,
                }))
            })
            .collect::<Fallible<Vec<_>>>()?;
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

        Ok(GPURenderPassEncoder::new(
            &self.global(),
            self.channel.clone(),
            WebGPURenderPass(render_pass_id),
            self,
            descriptor.parent.label.clone(),
        ))
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
    ) -> Fallible<()> {
        self.channel
            .0
            .send(WebGPURequest::CopyBufferToTexture {
                command_encoder_id: self.encoder.0,
                source: convert_ic_buffer(source),
                destination: convert_ic_texture(destination)?,
                copy_size: convert_texture_size(&copy_size)?,
            })
            .expect("Failed to send CopyBufferToTexture");

        Ok(())
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpucommandencoder-copybuffertotexture>
    fn CopyTextureToBuffer(
        &self,
        source: &GPUImageCopyTexture,
        destination: &GPUImageCopyBuffer,
        copy_size: GPUExtent3D,
    ) -> Fallible<()> {
        self.channel
            .0
            .send(WebGPURequest::CopyTextureToBuffer {
                command_encoder_id: self.encoder.0,
                source: convert_ic_texture(source)?,
                destination: convert_ic_buffer(destination),
                copy_size: convert_texture_size(&copy_size)?,
            })
            .expect("Failed to send CopyTextureToBuffer");

        Ok(())
    }

    /// <https://gpuweb.github.io/gpuweb/#GPUCommandEncoder-copyTextureToTexture>
    fn CopyTextureToTexture(
        &self,
        source: &GPUImageCopyTexture,
        destination: &GPUImageCopyTexture,
        copy_size: GPUExtent3D,
    ) -> Fallible<()> {
        self.channel
            .0
            .send(WebGPURequest::CopyTextureToTexture {
                command_encoder_id: self.encoder.0,
                source: convert_ic_texture(source)?,
                destination: convert_ic_texture(destination)?,
                copy_size: convert_texture_size(&copy_size)?,
            })
            .expect("Failed to send CopyTextureToTexture");

        Ok(())
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpucommandencoder-finish>
    fn Finish(&self, descriptor: &GPUCommandBufferDescriptor) -> DomRoot<GPUCommandBuffer> {
        self.channel
            .0
            .send(WebGPURequest::CommandEncoderFinish {
                command_encoder_id: self.encoder.0,
                device_id: self.device.id().0,
                desc: wgt::CommandBufferDescriptor {
                    label: convert_label(&descriptor.parent),
                },
            })
            .expect("Failed to send Finish");

        let buffer = webgpu::WebGPUCommandBuffer(self.encoder.0.into_command_buffer_id());
        GPUCommandBuffer::new(
            &self.global(),
            self.channel.clone(),
            buffer,
            descriptor.parent.label.clone(),
        )
    }
}
