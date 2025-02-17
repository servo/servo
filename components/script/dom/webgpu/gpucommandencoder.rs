/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use webgpu::wgc::command as wgpu_com;
use webgpu::{
    wgt, WebGPU, WebGPUCommandBuffer, WebGPUCommandEncoder, WebGPUComputePass, WebGPUDevice,
    WebGPURenderPass, WebGPURequest,
};

use crate::conversions::{Convert, TryConvert};
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::{
    GPUCommandBufferDescriptor, GPUCommandEncoderDescriptor, GPUCommandEncoderMethods,
    GPUComputePassDescriptor, GPUExtent3D, GPUImageCopyBuffer, GPUImageCopyTexture,
    GPURenderPassDescriptor, GPUSize64,
};
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::USVString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::gpuconvert::convert_load_op;
use crate::dom::webgpu::gpubuffer::GPUBuffer;
use crate::dom::webgpu::gpucommandbuffer::GPUCommandBuffer;
use crate::dom::webgpu::gpucomputepassencoder::GPUComputePassEncoder;
use crate::dom::webgpu::gpudevice::GPUDevice;
use crate::dom::webgpu::gpurenderpassencoder::GPURenderPassEncoder;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct GPUCommandEncoder {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "defined in webgpu"]
    #[no_trace]
    channel: WebGPU,
    label: DomRefCell<USVString>,
    #[no_trace]
    encoder: WebGPUCommandEncoder,
    device: Dom<GPUDevice>,
}

impl GPUCommandEncoder {
    pub(crate) fn new_inherited(
        channel: WebGPU,
        device: &GPUDevice,
        encoder: WebGPUCommandEncoder,
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

    pub(crate) fn new(
        global: &GlobalScope,
        channel: WebGPU,
        device: &GPUDevice,
        encoder: WebGPUCommandEncoder,
        label: USVString,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(GPUCommandEncoder::new_inherited(
                channel, device, encoder, label,
            )),
            global,
            can_gc,
        )
    }
}

impl GPUCommandEncoder {
    pub(crate) fn id(&self) -> WebGPUCommandEncoder {
        self.encoder
    }

    pub(crate) fn device_id(&self) -> WebGPUDevice {
        self.device.id()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createcommandencoder>
    pub(crate) fn create(
        device: &GPUDevice,
        descriptor: &GPUCommandEncoderDescriptor,
    ) -> DomRoot<GPUCommandEncoder> {
        let command_encoder_id = device.global().wgpu_id_hub().create_command_encoder_id();
        device
            .channel()
            .0
            .send(WebGPURequest::CreateCommandEncoder {
                device_id: device.id().0,
                command_encoder_id,
                desc: wgt::CommandEncoderDescriptor {
                    label: (&descriptor.parent).convert(),
                },
            })
            .expect("Failed to create WebGPU command encoder");

        let encoder = WebGPUCommandEncoder(command_encoder_id);

        GPUCommandEncoder::new(
            &device.global(),
            device.channel().clone(),
            device,
            encoder,
            descriptor.parent.label.clone(),
            CanGc::note(),
        )
    }
}

impl GPUCommandEncoderMethods<crate::DomTypeHolder> for GPUCommandEncoder {
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
        let compute_pass_id = self.global().wgpu_id_hub().create_compute_pass_id();

        if let Err(e) = self.channel.0.send(WebGPURequest::BeginComputePass {
            command_encoder_id: self.id().0,
            compute_pass_id,
            label: (&descriptor.parent).convert(),
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
            CanGc::note(),
        )
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpucommandencoder-beginrenderpass>
    fn BeginRenderPass(
        &self,
        descriptor: &GPURenderPassDescriptor,
    ) -> Fallible<DomRoot<GPURenderPassEncoder>> {
        let depth_stencil_attachment = descriptor.depthStencilAttachment.as_ref().map(|ds| {
            wgpu_com::RenderPassDepthStencilAttachment {
                depth: wgpu_com::PassChannel {
                    load_op: ds
                        .depthLoadOp
                        .as_ref()
                        .map(|l| convert_load_op(l, ds.depthClearValue.map(|v| *v))),
                    store_op: ds.depthStoreOp.as_ref().map(Convert::convert),
                    read_only: ds.depthReadOnly,
                },
                stencil: wgpu_com::PassChannel {
                    load_op: ds
                        .stencilLoadOp
                        .as_ref()
                        .map(|l| convert_load_op(l, Some(ds.stencilClearValue))),
                    store_op: ds.stencilStoreOp.as_ref().map(Convert::convert),
                    read_only: ds.stencilReadOnly,
                },
                view: ds.view.id().0,
            }
        });

        let color_attachments = descriptor
            .colorAttachments
            .iter()
            .map(|color| -> Fallible<_> {
                Ok(Some(wgpu_com::RenderPassColorAttachment {
                    resolve_target: color.resolveTarget.as_ref().map(|t| t.id().0),
                    load_op: convert_load_op(
                        &color.loadOp,
                        color
                            .clearValue
                            .as_ref()
                            .map(|color| (color).try_convert())
                            .transpose()?
                            .unwrap_or_default(),
                    ),
                    store_op: color.storeOp.convert(),
                    view: color.view.id().0,
                }))
            })
            .collect::<Fallible<Vec<_>>>()?;
        let render_pass_id = self.global().wgpu_id_hub().create_render_pass_id();

        if let Err(e) = self.channel.0.send(WebGPURequest::BeginRenderPass {
            command_encoder_id: self.id().0,
            render_pass_id,
            label: (&descriptor.parent).convert(),
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
            CanGc::note(),
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
                source: source.convert(),
                destination: destination.try_convert()?,
                copy_size: (&copy_size).try_convert()?,
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
                source: source.try_convert()?,
                destination: destination.convert(),
                copy_size: (&copy_size).try_convert()?,
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
                source: source.try_convert()?,
                destination: destination.try_convert()?,
                copy_size: (&copy_size).try_convert()?,
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
                    label: (&descriptor.parent).convert(),
                },
            })
            .expect("Failed to send Finish");

        let buffer = WebGPUCommandBuffer(self.encoder.0.into_command_buffer_id());
        GPUCommandBuffer::new(
            &self.global(),
            self.channel.clone(),
            buffer,
            descriptor.parent.label.clone(),
            CanGc::note(),
        )
    }
}
