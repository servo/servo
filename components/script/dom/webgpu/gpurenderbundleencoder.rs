/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;

use dom_struct::dom_struct;
use js::context::{JSContext, NoGC};
use script_bindings::cell::DomRefCell;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};
use webgpu_traits::{RenderBundleCommand, WebGPU, WebGPURenderBundle, WebGPURenderBundleEncoder, WebGPURequest};
use wgpu_core::command::{
    RenderBundleEncoderDescriptor,
};

use crate::conversions::Convert;
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::{
    GPUIndexFormat, GPURenderBundleDescriptor, GPURenderBundleEncoderDescriptor,
    GPURenderBundleEncoderMethods,
};
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::USVString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::webgpu::gpubindgroup::GPUBindGroup;
use crate::dom::webgpu::gpubuffer::GPUBuffer;
use crate::dom::webgpu::gpudevice::GPUDevice;
use crate::dom::webgpu::gpurenderbundle::GPURenderBundle;
use crate::dom::webgpu::gpurenderpipeline::GPURenderPipeline;


#[derive(JSTraceable, MallocSizeOf)]
struct DroppableGPURenderBundleEncoder {
    #[no_trace]
    channel: WebGPU,
    #[no_trace]
    render_bundle_encoder: WebGPURenderBundleEncoder,
}

impl Drop for DroppableGPURenderBundleEncoder {
    fn drop(&mut self) {
        if let Err(error) = self
            .channel
            .0
            .send(WebGPURequest::DropRenderBundleEncoder(self.render_bundle_encoder.0))
        {
            warn!(
                "Failed to send WebGPURequest::DropRenderBundleEncoder({:?}) ({error})",
                self.render_bundle_encoder.0
            );
        }
    }
}

#[dom_struct]
pub(crate) struct GPURenderBundleEncoder {
    reflector_: Reflector,
    device: Dom<GPUDevice>,
    label: DomRefCell<USVString>,
    droppable: DroppableGPURenderBundleEncoder,
}

impl GPURenderBundleEncoder {
    fn new_inherited(
        device: &GPUDevice,
        channel: WebGPU,
        label: USVString,
        render_bundle_encoder: WebGPURenderBundleEncoder,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            device: Dom::from_ref(device),
            droppable: DroppableGPURenderBundleEncoder {
                channel,
                render_bundle_encoder,
            },
            label: DomRefCell::new(label),
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        render_bundle_encoder: WebGPURenderBundleEncoder,
        device: &GPUDevice,
        channel: WebGPU,
        label: USVString,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_cx(
            Box::new(GPURenderBundleEncoder::new_inherited(
                device,
                channel,
                label,
                render_bundle_encoder,
            )),
            global,
            cx,
        )
    }
}

impl GPURenderBundleEncoder {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createrenderbundleencoder>
    pub(crate) fn create(
        cx: &mut JSContext,
        device: &GPUDevice,
        descriptor: &GPURenderBundleEncoderDescriptor,
    ) -> Fallible<DomRoot<GPURenderBundleEncoder>> {
        let desc = RenderBundleEncoderDescriptor {
            label: (&descriptor.parent.parent).convert(),
            color_formats: Cow::Owned(
                descriptor
                    .parent
                    .colorFormats
                    .iter()
                    .map(|format| {
                        device
                            .validate_texture_format_required_features(format)
                            .map(Some)
                    })
                    .collect::<Fallible<Vec<_>>>()?,
            ),
            depth_stencil: descriptor
                .parent
                .depthStencilFormat
                .map(|dsf| {
                    device
                        .validate_texture_format_required_features(&dsf)
                        .map(|format| wgpu_types::RenderBundleDepthStencil {
                            format,
                            depth_read_only: descriptor.depthReadOnly,
                            stencil_read_only: descriptor.stencilReadOnly,
                        })
                })
                .transpose()?,
            sample_count: descriptor.parent.sampleCount,
            multiview: None,
        };

        let id = device.global().wgpu_id_hub().create_render_bundle_encoder_id();
        let render_bundle_encoder = WebGPURenderBundleEncoder(id);

        let channel = device.channel();

        channel
            .0
            .send(WebGPURequest::CreateRenderBundleEncoder {
                device_id: device.id().0,
                desc,
                render_bundle_encoder_id: render_bundle_encoder.0,
            })
            .expect("Failed to send WebGPURequest::CreateRenderBundleEncoder");


        Ok(GPURenderBundleEncoder::new(
            cx,
            &device.global(),
            render_bundle_encoder,
            device,
            device.channel(),
            descriptor.parent.parent.label.clone(),
        ))
    }

    pub(crate) fn id(&self) -> WebGPURenderBundleEncoder {
        self.droppable.render_bundle_encoder
    }
}

impl GPURenderBundleEncoderMethods<crate::DomTypeHolder> for GPURenderBundleEncoder {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn Label(&self) -> USVString {
        self.label.borrow().clone()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn SetLabel(&self, no_gc: &NoGC, value: USVString) {
        *self.label.safe_borrow_mut(no_gc) = value;
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuprogrammablepassencoder-setbindgroup>
    fn SetBindGroup(
        &self,
        index: u32,
        bind_group: &GPUBindGroup,
        dynamic_offsets: Vec<u32>,
    ) {
        if let Err(error) = self
            .droppable
            .channel
            .0
            .send(WebGPURequest::RenderBundleEncoderCommand { render_bundle_encoder_id: self.droppable.render_bundle_encoder.0, render_command: RenderBundleCommand::SetBindGroup { index, bind_group_id: bind_group.id().0, offsets: dynamic_offsets }, device_id: self.device.id().0 })
        {
            warn!(
                "Failed to send WebGPURequest::RenderBundleEncoderSetBindGroup({:?}) ({error})",
                self.droppable.render_bundle_encoder.0
            );
        }
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpurenderencoderbase-setpipeline>
    fn SetPipeline(&self, pipeline: &GPURenderPipeline) {
        if let Err(error) = self
            .droppable
            .channel
            .0
            .send(WebGPURequest::RenderBundleEncoderCommand { render_bundle_encoder_id: self.droppable.render_bundle_encoder.0, render_command: RenderBundleCommand::SetPipeline(pipeline.id().0), device_id: self.device.id().0 })
        {
            warn!(
                "Failed to send WebGPURequest::RenderBundleEncoderSetPipeline({:?}) ({error})",
                self.droppable.render_bundle_encoder.0
            );
        }
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpurenderencoderbase-setindexbuffer>
    fn SetIndexBuffer(
        &self,
        buffer: &GPUBuffer,
        index_format: GPUIndexFormat,
        offset: u64,
        size: u64,
    ) {
        if let Err(error) = self
            .droppable
            .channel
            .0
            .send(WebGPURequest::RenderBundleEncoderCommand { render_bundle_encoder_id: self.droppable.render_bundle_encoder.0, render_command: RenderBundleCommand::SetIndexBuffer { buffer_id: buffer.id().0, index_format: index_format.convert(), offset, size: wgpu_types::BufferSize::new(size) }, device_id: self.device.id().0 })
        {
            warn!(
                "Failed to send WebGPURequest::RenderBundleEncoderSetIndexBuffer({:?}) ({error})",
                self.droppable.render_bundle_encoder.0
            );
        }
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpurenderencoderbase-setvertexbuffer>
    fn SetVertexBuffer(&self, slot: u32, buffer: Option<&GPUBuffer>, offset: u64, size: u64) {
        if let Err(error) = self
            .droppable
            .channel
            .0
            .send(WebGPURequest::RenderBundleEncoderCommand { render_bundle_encoder_id: self.droppable.render_bundle_encoder.0, render_command: RenderBundleCommand::SetVertexBuffer { slot, buffer_id: buffer.map(|b| b.id().0), offset, size: wgpu_types::BufferSize::new(size) }, device_id: self.device.id().0 })
        {
            warn!(
                "Failed to send WebGPURequest::RenderBundleEncoderSetVertexBuffer({:?}) ({error})",
                self.droppable.render_bundle_encoder.0
            );
        }
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpurenderencoderbase-draw>
    fn Draw(
        &self,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    ) {
        if let Err(error) = self
            .droppable
            .channel
            .0
            .send(WebGPURequest::RenderBundleEncoderCommand { render_bundle_encoder_id: self.droppable.render_bundle_encoder.0, render_command: RenderBundleCommand::Draw { vertex_count, instance_count, first_vertex, first_instance }, device_id: self.device.id().0 })
        {
            warn!(
                "Failed to send WebGPURequest::RenderBundleEncoderDraw({:?}) ({error})",
                self.droppable.render_bundle_encoder.0
            );
        }
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpurenderencoderbase-drawindexed>
    fn DrawIndexed(
        &self,
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        base_vertex: i32,
        first_instance: u32,
    ) {
        if let Err(error) = self
            .droppable
            .channel
            .0
            .send(WebGPURequest::RenderBundleEncoderCommand { render_bundle_encoder_id: self.droppable.render_bundle_encoder.0, render_command: RenderBundleCommand::DrawIndexed { index_count, instance_count, first_index, base_vertex, first_instance }, device_id: self.device.id().0 })
        {
            warn!(
                "Failed to send WebGPURequest::RenderBundleEncoderDrawIndexed({:?}) ({error})",
                self.droppable.render_bundle_encoder.0
            );
        }
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpurenderencoderbase-drawindirect>
    fn DrawIndirect(&self, indirect_buffer: &GPUBuffer, indirect_offset: u64) {
        if let Err(error) = self
            .droppable
            .channel
            .0
            .send(WebGPURequest::RenderBundleEncoderCommand { render_bundle_encoder_id: self.droppable.render_bundle_encoder.0, render_command: RenderBundleCommand::DrawIndirect { buffer_id: indirect_buffer.id().0, offset: indirect_offset }, device_id: self.device.id().0 })
        {
            warn!(
                "Failed to send WebGPURequest::RenderBundleEncoderDrawIndirect({:?}) ({error})",
                self.droppable.render_bundle_encoder.0
            );
        }
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpurenderencoderbase-drawindexedindirect>
    fn DrawIndexedIndirect(&self, indirect_buffer: &GPUBuffer, indirect_offset: u64) {
        if let Err(error) = self
            .droppable
            .channel
            .0
            .send(WebGPURequest::RenderBundleEncoderCommand { render_bundle_encoder_id: self.droppable.render_bundle_encoder.0, render_command: RenderBundleCommand::DrawIndexedIndirect { buffer_id: indirect_buffer.id().0, offset: indirect_offset }, device_id: self.device.id().0 })
        {
            warn!(
                "Failed to send WebGPURequest::RenderBundleEncoderDrawIndexedIndirect({:?}) ({error})",
                self.droppable.render_bundle_encoder.0
            );
        }
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudebugcommandsmixin-pushdebuggroup>
    fn PushDebugGroup(&self, group_label: USVString) {
        if let Err(error) = self
            .droppable
            .channel
            .0
            .send(WebGPURequest::RenderBundleEncoderCommand { render_bundle_encoder_id: self.droppable.render_bundle_encoder.0, render_command: RenderBundleCommand::PushDebugGroup(group_label.to_string()), device_id: self.device.id().0 })
        {
            warn!(
                "Failed to send WebGPURequest::RenderBundleEncoderPushDebugGroup({:?}) ({error})",
                self.droppable.render_bundle_encoder.0
            );
        }
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudebugcommandsmixin-popdebuggroup>
    fn PopDebugGroup(&self) {
        if let Err(error) = self
            .droppable
            .channel
            .0
            .send(WebGPURequest::RenderBundleEncoderCommand { render_bundle_encoder_id: self.droppable.render_bundle_encoder.0, render_command: RenderBundleCommand::PopDebugGroup, device_id: self.device.id().0 })
        {
            warn!(
                "Failed to send WebGPURequest::RenderBundleEncoderPopDebugGroup({:?}) ({error})",
                self.id()
            );
        }
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudebugcommandsmixin-insertdebugmarker>
    fn InsertDebugMarker(&self, marker_label: USVString) {
        if let Err(error) = self
            .droppable
            .channel
            .0
            .send(WebGPURequest::RenderBundleEncoderCommand { render_bundle_encoder_id: self.droppable.render_bundle_encoder.0, render_command: RenderBundleCommand::InsertDebugMarker(marker_label.to_string()), device_id: self.device.id().0 })
        {
            warn!(
                "Failed to send WebGPURequest::RenderBundleEncoderInsertDebugMarker({:?}) ({error})",
                self.id()
            );
        }
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpurenderbundleencoder-finish>
    fn Finish(
        &self,
        cx: &mut JSContext,
        descriptor: &GPURenderBundleDescriptor,
    ) -> DomRoot<GPURenderBundle> {
        let desc = wgpu_types::RenderBundleDescriptor {
            label: (&descriptor.parent).convert(),
        };
        let render_bundle_id = self.global().wgpu_id_hub().create_render_bundle_id();

        self.droppable
            .channel
            .0
            .send(WebGPURequest::RenderBundleEncoderFinish {
                render_bundle_encoder_id: self.id().0,
                descriptor: desc,
                render_bundle_id,
                device_id: self.device.id().0,
            })
            .expect("Failed to send RenderBundleEncoderFinish");

        let render_bundle = WebGPURenderBundle(render_bundle_id);
        GPURenderBundle::new(
            cx,
            &self.global(),
            render_bundle,
            self.device.id(),
            self.droppable.channel.clone(),
            descriptor.parent.label.clone(),
        )
    }
}
