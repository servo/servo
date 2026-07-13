/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::{JSContext, NoGC};
use js::cell::JSCell;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};
use webgpu_traits::{
    WebGPU, WebGPUCommandBuffer, WebGPUCommandEncoder, WebGPUComputePass, WebGPUDevice,
    WebGPURenderPass, WebGPURequest,
};
use wgpu_core::command as wgpu_com;

use crate::conversions::{Convert, TryConvert};
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::{
    GPUCommandBufferDescriptor, GPUCommandEncoderDescriptor, GPUCommandEncoderMethods,
    GPUComputePassDescriptor, GPUExtent3D, GPURenderPassDescriptor, GPUSize64,
    GPUTexelCopyBufferInfo, GPUTexelCopyTextureInfo,
};
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::USVString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::gpuconvert::{convert_load_op, convert_texture_for_wgpu_with_cx};
use crate::dom::types::GPUQuerySet;
use crate::dom::webgpu::gpubuffer::GPUBuffer;
use crate::dom::webgpu::gpucommandbuffer::GPUCommandBuffer;
use crate::dom::webgpu::gpucomputepassencoder::GPUComputePassEncoder;
use crate::dom::webgpu::gpudevice::GPUDevice;
use crate::dom::webgpu::gpurenderpassencoder::GPURenderPassEncoder;

#[derive(JSTraceable, MallocSizeOf)]
struct DroppableGPUCommandEncoder {
    #[no_trace]
    channel: WebGPU,
    #[no_trace]
    encoder: WebGPUCommandEncoder,
}

#[dom_struct]
pub(crate) struct GPUCommandEncoder {
    reflector_: Reflector,
    droppable: DroppableGPUCommandEncoder,
    #[ignore_malloc_size_of = "JSCell is hard to measure"]
    label: JSCell<USVString>,
    device: Dom<GPUDevice>,
}

impl Drop for DroppableGPUCommandEncoder {
    fn drop(&mut self) {
        if let Err(e) = self
            .channel
            .0
            .send(WebGPURequest::DropCommandEncoder(self.encoder.0))
        {
            warn!("Failed to send WebGPURequest::DropCommandEncoder with {e:?}");
        }
    }
}

impl GPUCommandEncoder {
    pub(crate) fn new_inherited(
        channel: WebGPU,
        device: &GPUDevice,
        encoder: WebGPUCommandEncoder,
        label: USVString,
    ) -> Self {
        Self {
            droppable: DroppableGPUCommandEncoder { channel, encoder },
            reflector_: Reflector::new(),
            label: JSCell::new(label),
            device: Dom::from_ref(device),
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        channel: WebGPU,
        device: &GPUDevice,
        encoder: WebGPUCommandEncoder,
        label: USVString,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_cx(
            Box::new(GPUCommandEncoder::new_inherited(
                channel, device, encoder, label,
            )),
            global,
            cx,
        )
    }
}

impl GPUCommandEncoder {
    pub(crate) fn id(&self) -> WebGPUCommandEncoder {
        self.droppable.encoder
    }

    pub(crate) fn device_id(&self) -> WebGPUDevice {
        self.device.id()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createcommandencoder>
    pub(crate) fn create(
        cx: &mut JSContext,
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
                desc: wgpu_types::CommandEncoderDescriptor {
                    label: (&descriptor.parent).convert(),
                },
            })
            .expect("Failed to create WebGPU command encoder");

        let encoder = WebGPUCommandEncoder(command_encoder_id);

        GPUCommandEncoder::new(
            cx,
            &device.global(),
            device.channel(),
            device,
            encoder,
            descriptor.parent.label.clone(),
        )
    }
}

impl GPUCommandEncoderMethods<crate::DomTypeHolder> for GPUCommandEncoder {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn Label(&self, no_gc: &NoGC) -> USVString {
        self.label.borrow(no_gc).clone()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn SetLabel(&self, no_gc_mut: &mut NoGC, value: USVString) {
        *self.label.borrow_mut(no_gc_mut) = value;
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpucommandencoder-begincomputepass>
    fn BeginComputePass(
        &self,
        cx: &mut JSContext,
        descriptor: &GPUComputePassDescriptor,
    ) -> DomRoot<GPUComputePassEncoder> {
        let compute_pass_id = self.global().wgpu_id_hub().create_compute_pass_id();

        if let Err(error) = self
            .droppable
            .channel
            .0
            .send(WebGPURequest::BeginComputePass {
                command_encoder_id: self.id().0,
                compute_pass_id,
                label: (&descriptor.parent).convert(),
                timestamp_writes: descriptor.timestampWrites.as_ref().map(Convert::convert),
                device_id: self.device.id().0,
            })
        {
            warn!("Failed to send WebGPURequest::BeginComputePass {error:?}");
        }

        GPUComputePassEncoder::new(
            cx,
            &self.global(),
            self.droppable.channel.clone(),
            self,
            WebGPUComputePass(compute_pass_id),
            descriptor.parent.label.clone(),
        )
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpucommandencoder-beginrenderpass>
    fn BeginRenderPass(
        &self,
        cx: &mut JSContext,
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
                view: convert_texture_for_wgpu_with_cx(cx, &ds.view).0,
            }
        });

        let color_attachments = descriptor
            .colorAttachments
            .iter()
            .map(|color| -> Fallible<_> {
                Ok(Some(wgpu_com::RenderPassColorAttachment {
                    resolve_target: color
                        .resolveTarget
                        .as_ref()
                        .map(|t| convert_texture_for_wgpu_with_cx(cx, t).0),
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
                    view: convert_texture_for_wgpu_with_cx(cx, &color.view).0,
                    depth_slice: None,
                }))
            })
            .collect::<Fallible<Vec<_>>>()?;
        let render_pass_id = self.global().wgpu_id_hub().create_render_pass_id();

        if let Err(error) = self
            .droppable
            .channel
            .0
            .send(WebGPURequest::BeginRenderPass {
                command_encoder_id: self.id().0,
                render_pass_id,
                label: (&descriptor.parent).convert(),
                depth_stencil_attachment,
                color_attachments,
                timestamp_writes: descriptor.timestampWrites.as_ref().map(Convert::convert),
                device_id: self.device.id().0,
            })
        {
            warn!("Failed to send WebGPURequest::BeginRenderPass {error:?}");
        }

        Ok(GPURenderPassEncoder::new(
            cx,
            &self.global(),
            self.droppable.channel.clone(),
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
        self.droppable
            .channel
            .0
            .send(WebGPURequest::CopyBufferToBuffer {
                command_encoder_id: self.droppable.encoder.0,
                source_id: source.id().0,
                source_offset,
                destination_id: destination.id().0,
                destination_offset,
                size,
                device_id: self.device.id().0,
            })
            .expect("Failed to send CopyBufferToBuffer");
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpucommandencoder-copybuffertotexture>
    fn CopyBufferToTexture(
        &self,
        source: &GPUTexelCopyBufferInfo,
        destination: &GPUTexelCopyTextureInfo,
        copy_size: GPUExtent3D,
    ) -> Fallible<()> {
        self.droppable
            .channel
            .0
            .send(WebGPURequest::CopyBufferToTexture {
                command_encoder_id: self.droppable.encoder.0,
                source: source.convert(),
                destination: destination.try_convert()?,
                copy_size: (&copy_size).try_convert()?,
                device_id: self.device.id().0,
            })
            .expect("Failed to send CopyBufferToTexture");

        Ok(())
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpucommandencoder-copybuffertotexture>
    fn CopyTextureToBuffer(
        &self,
        source: &GPUTexelCopyTextureInfo,
        destination: &GPUTexelCopyBufferInfo,
        copy_size: GPUExtent3D,
    ) -> Fallible<()> {
        self.droppable
            .channel
            .0
            .send(WebGPURequest::CopyTextureToBuffer {
                command_encoder_id: self.droppable.encoder.0,
                source: source.try_convert()?,
                destination: destination.convert(),
                copy_size: (&copy_size).try_convert()?,
                device_id: self.device.id().0,
            })
            .expect("Failed to send CopyTextureToBuffer");

        Ok(())
    }

    /// <https://gpuweb.github.io/gpuweb/#GPUCommandEncoder-copyTextureToTexture>
    fn CopyTextureToTexture(
        &self,
        source: &GPUTexelCopyTextureInfo,
        destination: &GPUTexelCopyTextureInfo,
        copy_size: GPUExtent3D,
    ) -> Fallible<()> {
        self.droppable
            .channel
            .0
            .send(WebGPURequest::CopyTextureToTexture {
                command_encoder_id: self.droppable.encoder.0,
                source: source.try_convert()?,
                destination: destination.try_convert()?,
                copy_size: (&copy_size).try_convert()?,
                device_id: self.device.id().0,
            })
            .expect("Failed to send CopyTextureToTexture");

        Ok(())
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpucommandencoder-finish>
    fn Finish(
        &self,
        cx: &mut JSContext,
        descriptor: &GPUCommandBufferDescriptor,
    ) -> DomRoot<GPUCommandBuffer> {
        let command_buffer_id = self.global().wgpu_id_hub().create_command_buffer_id();
        self.droppable
            .channel
            .0
            .send(WebGPURequest::CommandEncoderFinish {
                command_encoder_id: self.droppable.encoder.0,
                device_id: self.device.id().0,
                desc: wgpu_types::CommandBufferDescriptor {
                    label: (&descriptor.parent).convert(),
                },
                command_buffer_id,
            })
            .expect("Failed to send Finish");

        let buffer = WebGPUCommandBuffer(command_buffer_id);
        GPUCommandBuffer::new(
            cx,
            &self.global(),
            self.droppable.channel.clone(),
            buffer,
            descriptor.parent.label.clone(),
        )
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudebugcommandsmixin-pushdebuggroup>
    fn PushDebugGroup(&self, group_label: USVString) {
        if let Err(e) = self
            .droppable
            .channel
            .0
            .send(WebGPURequest::CommandEncoderPushDebugGroup {
                command_encoder_id: self.droppable.encoder.0,
                label: group_label.to_string(),
                device_id: self.device.id().0,
            })
        {
            warn!("Error sending WebGPURequest::CommandEncoderPushDebugGroup: {e:?}")
        }
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudebugcommandsmixin-popdebuggroup>
    fn PopDebugGroup(&self) {
        if let Err(e) = self
            .droppable
            .channel
            .0
            .send(WebGPURequest::CommandEncoderPopDebugGroup {
                command_encoder_id: self.droppable.encoder.0,
                device_id: self.device.id().0,
            })
        {
            warn!("Error sending WebGPURequest::CommandEncoderPopDebugGroup: {e:?}")
        }
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudebugcommandsmixin-insertdebugmarker>
    fn InsertDebugMarker(&self, marker_label: USVString) {
        if let Err(e) =
            self.droppable
                .channel
                .0
                .send(WebGPURequest::CommandEncoderInsertDebugMarker {
                    command_encoder_id: self.droppable.encoder.0,
                    label: marker_label.to_string(),
                    device_id: self.device.id().0,
                })
        {
            warn!("Error sending WebGPURequest::CommandEncoderInsertDebugMarker: {e:?}")
        }
    }

    fn ResolveQuerySet(
        &self,
        query_set: &GPUQuerySet,
        first_query: u32,
        query_count: u32,
        destination: &GPUBuffer,
        destination_offset: u64,
    ) {
        if let Err(error) = self
            .droppable
            .channel
            .0
            .send(WebGPURequest::ResolveQuerySet {
                command_encoder_id: self.droppable.encoder.0,
                query_set_id: query_set.id().0,
                start_query: first_query,
                query_count,
                destination: destination.id().0,
                destination_offset,
                device_id: self.device.id().0,
            })
        {
            warn!("Error sending WebGPURequest::ResolveQuerySet: {error:?}")
        }
    }
}
