/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;

use dom_struct::dom_struct;
use js::context::{JSContext, NoGC};
use log::warn;
use malloc_size_of_derive::MallocSizeOf;
use script_bindings::DomTypes;
use script_bindings::cell::DomRefCell;
use script_bindings::codegen::GenericBindings::WebGPUBinding::{
    GPUIndexFormat, GPURenderBundleDescriptor, GPURenderBundleEncoderDescriptor,
    GPURenderBundleEncoderMethods, GPURenderBundleEncoderWrap,
};
use script_bindings::interfaces::{GlobalScopeHelpers, PromiseHelpers};
use script_bindings::reflector::{DomGlobalGeneric, Reflector, reflect_dom_object_with_wrap};
use webgpu_traits::{
    RenderBundleCommand, WebGPU, WebGPURenderBundle, WebGPURenderBundleEncoder, WebGPURequest,
};
use wgpu_core::command::RenderBundleEncoderDescriptor;

use crate::JSTraceable;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::USVString;
use crate::gpubindgroup::GPUBindGroup;
use crate::gpubuffer::GPUBuffer;
use crate::gpuconvert::WebGPUConvert;
use crate::gpurenderbundle::GPURenderBundle;
use crate::gpurenderpipeline::GPURenderPipeline;
use crate::traits::{Equivalence, GPUDeviceTrait, GPUExternalTextureTrait, WebGPUGlobalTrait};

#[derive(JSTraceable, MallocSizeOf)]
struct DroppableGPURenderBundleEncoder {
    #[no_trace]
    channel: WebGPU,
    #[no_trace]
    render_bundle_encoder: WebGPURenderBundleEncoder,
}

impl Drop for DroppableGPURenderBundleEncoder {
    fn drop(&mut self) {
        if let Err(error) = self.channel.0.send(WebGPURequest::DropRenderBundleEncoder(
            self.render_bundle_encoder.0,
        )) {
            warn!(
                "Failed to send WebGPURequest::DropRenderBundleEncoder({:?}) ({error})",
                self.render_bundle_encoder.0
            );
        }
    }
}

#[dom_struct]
pub struct GPURenderBundleEncoder<D: DomTypes> {
    reflector_: Reflector,
    device: Dom<D::GPUDevice>,
    label: DomRefCell<USVString>,
    droppable: DroppableGPURenderBundleEncoder,
}

impl<D> GPURenderBundleEncoder<D>
where
    D: Equivalence,
{
    fn new_inherited(
        device: &D::GPUDevice,
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
        global: &D::GlobalScope,
        render_bundle_encoder: WebGPURenderBundleEncoder,
        device: &D::GPUDevice,
        channel: WebGPU,
        label: USVString,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_wrap::<D, _, _>(
            Box::new(GPURenderBundleEncoder::new_inherited(
                device,
                channel,
                label,
                render_bundle_encoder,
            )),
            global,
            cx,
            GPURenderBundleEncoderWrap::<D>,
        )
    }
}

impl<D> GPURenderBundleEncoder<D>
where
    D: Equivalence,
    D::GPUDevice: GPUDeviceTrait<D>,
    D::GlobalScope: WebGPUGlobalTrait,
    Self: DomGlobalGeneric<D>,
{
    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createrenderbundleencoder>
    pub fn create(
        cx: &mut JSContext,
        device: &D::GPUDevice,
        descriptor: &GPURenderBundleEncoderDescriptor,
    ) -> Fallible<DomRoot<GPURenderBundleEncoder<D>>> {
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

        let id = device
            .global_from_reflector()
            .global_wgpu_id_hub()
            .create_render_bundle_encoder_id();
        let render_bundle_encoder = WebGPURenderBundleEncoder(id);

        let channel = device.channel();

        if let Err(error) = channel.0.send(WebGPURequest::CreateRenderBundleEncoder {
            device_id: device.id().0,
            desc,
            render_bundle_encoder_id: render_bundle_encoder.0,
        }) {
            warn!(
                "Failed to send WebGPURequest::CreateRenderBundleEncoder({:?}) ({error})",
                render_bundle_encoder.0
            );
        }

        Ok(GPURenderBundleEncoder::new(
            cx,
            &*device.global_from_reflector(),
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

impl<D> GPURenderBundleEncoderMethods<D> for GPURenderBundleEncoder<D>
where
    D: Equivalence,
    D::GPUDevice: DomGlobalGeneric<D> + GPUDeviceTrait<D>,
    D::GPUExternalTexture: GPUExternalTextureTrait<D>,
    D::GlobalScope: WebGPUGlobalTrait + GlobalScopeHelpers<D>,
    D::Promise: PromiseHelpers<D>,
    Self: DomGlobalGeneric<D>,
{
    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn Label(&self) -> USVString {
        self.label.borrow().clone()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn SetLabel(&self, no_gc: &NoGC, value: USVString) {
        *self.label.safe_borrow_mut(no_gc) = value;
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuprogrammablepassencoder-setbindgroup>
    fn SetBindGroup(&self, index: u32, bind_group: &GPUBindGroup<D>, dynamic_offsets: Vec<u32>) {
        if let Err(error) =
            self.droppable
                .channel
                .0
                .send(WebGPURequest::RenderBundleEncoderCommand {
                    render_bundle_encoder_id: self.droppable.render_bundle_encoder.0,
                    render_command: RenderBundleCommand::SetBindGroup {
                        index,
                        bind_group_id: bind_group.id().0,
                        offsets: dynamic_offsets,
                    },
                    device_id: self.device.id().0,
                })
        {
            warn!(
                "Failed to send WebGPURequest::RenderBundleEncoderSetBindGroup({:?}) ({error})",
                self.droppable.render_bundle_encoder.0
            );
        }
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpurenderencoderbase-setpipeline>
    fn SetPipeline(&self, pipeline: &GPURenderPipeline<D>) {
        if let Err(error) =
            self.droppable
                .channel
                .0
                .send(WebGPURequest::RenderBundleEncoderCommand {
                    render_bundle_encoder_id: self.droppable.render_bundle_encoder.0,
                    render_command: RenderBundleCommand::SetPipeline(pipeline.id().0),
                    device_id: self.device.id().0,
                })
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
        buffer: &D::GPUBuffer,
        index_format: GPUIndexFormat,
        offset: u64,
        size: u64,
    ) {
        if let Err(error) =
            self.droppable
                .channel
                .0
                .send(WebGPURequest::RenderBundleEncoderCommand {
                    render_bundle_encoder_id: self.droppable.render_bundle_encoder.0,
                    render_command: RenderBundleCommand::SetIndexBuffer {
                        buffer_id: buffer.id().0,
                        index_format: index_format.convert(),
                        offset,
                        size: wgpu_types::BufferSize::new(size),
                    },
                    device_id: self.device.id().0,
                })
        {
            warn!(
                "Failed to send WebGPURequest::RenderBundleEncoderSetIndexBuffer({:?}) ({error})",
                self.droppable.render_bundle_encoder.0
            );
        }
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpurenderencoderbase-setvertexbuffer>
    fn SetVertexBuffer(&self, slot: u32, buffer: Option<&GPUBuffer<D>>, offset: u64, size: u64) {
        if let Err(error) =
            self.droppable
                .channel
                .0
                .send(WebGPURequest::RenderBundleEncoderCommand {
                    render_bundle_encoder_id: self.droppable.render_bundle_encoder.0,
                    render_command: RenderBundleCommand::SetVertexBuffer {
                        slot,
                        buffer_id: buffer.map(|b| b.id().0),
                        offset,
                        size: wgpu_types::BufferSize::new(size),
                    },
                    device_id: self.device.id().0,
                })
        {
            warn!(
                "Failed to send WebGPURequest::RenderBundleEncoderSetVertexBuffer({:?}) ({error})",
                self.droppable.render_bundle_encoder.0
            );
        }
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpurenderencoderbase-draw>
    fn Draw(&self, vertex_count: u32, instance_count: u32, first_vertex: u32, first_instance: u32) {
        if let Err(error) =
            self.droppable
                .channel
                .0
                .send(WebGPURequest::RenderBundleEncoderCommand {
                    render_bundle_encoder_id: self.droppable.render_bundle_encoder.0,
                    render_command: RenderBundleCommand::Draw {
                        vertex_count,
                        instance_count,
                        first_vertex,
                        first_instance,
                    },
                    device_id: self.device.id().0,
                })
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
        if let Err(error) =
            self.droppable
                .channel
                .0
                .send(WebGPURequest::RenderBundleEncoderCommand {
                    render_bundle_encoder_id: self.droppable.render_bundle_encoder.0,
                    render_command: RenderBundleCommand::DrawIndexed {
                        index_count,
                        instance_count,
                        first_index,
                        base_vertex,
                        first_instance,
                    },
                    device_id: self.device.id().0,
                })
        {
            warn!(
                "Failed to send WebGPURequest::RenderBundleEncoderDrawIndexed({:?}) ({error})",
                self.droppable.render_bundle_encoder.0
            );
        }
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpurenderencoderbase-drawindirect>
    fn DrawIndirect(&self, indirect_buffer: &GPUBuffer<D>, indirect_offset: u64) {
        if let Err(error) =
            self.droppable
                .channel
                .0
                .send(WebGPURequest::RenderBundleEncoderCommand {
                    render_bundle_encoder_id: self.droppable.render_bundle_encoder.0,
                    render_command: RenderBundleCommand::DrawIndirect {
                        buffer_id: indirect_buffer.id().0,
                        offset: indirect_offset,
                    },
                    device_id: self.device.id().0,
                })
        {
            warn!(
                "Failed to send WebGPURequest::RenderBundleEncoderDrawIndirect({:?}) ({error})",
                self.droppable.render_bundle_encoder.0
            );
        }
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpurenderencoderbase-drawindexedindirect>
    fn DrawIndexedIndirect(&self, indirect_buffer: &GPUBuffer<D>, indirect_offset: u64) {
        if let Err(error) =
            self.droppable
                .channel
                .0
                .send(WebGPURequest::RenderBundleEncoderCommand {
                    render_bundle_encoder_id: self.droppable.render_bundle_encoder.0,
                    render_command: RenderBundleCommand::DrawIndexedIndirect {
                        buffer_id: indirect_buffer.id().0,
                        offset: indirect_offset,
                    },
                    device_id: self.device.id().0,
                })
        {
            warn!(
                "Failed to send WebGPURequest::RenderBundleEncoderDrawIndexedIndirect({:?}) ({error})",
                self.droppable.render_bundle_encoder.0
            );
        }
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudebugcommandsmixin-pushdebuggroup>
    fn PushDebugGroup(&self, group_label: USVString) {
        if let Err(error) =
            self.droppable
                .channel
                .0
                .send(WebGPURequest::RenderBundleEncoderCommand {
                    render_bundle_encoder_id: self.droppable.render_bundle_encoder.0,
                    render_command: RenderBundleCommand::PushDebugGroup(group_label.to_string()),
                    device_id: self.device.id().0,
                })
        {
            warn!(
                "Failed to send WebGPURequest::RenderBundleEncoderPushDebugGroup({:?}) ({error})",
                self.droppable.render_bundle_encoder.0
            );
        }
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudebugcommandsmixin-popdebuggroup>
    fn PopDebugGroup(&self) {
        if let Err(error) =
            self.droppable
                .channel
                .0
                .send(WebGPURequest::RenderBundleEncoderCommand {
                    render_bundle_encoder_id: self.droppable.render_bundle_encoder.0,
                    render_command: RenderBundleCommand::PopDebugGroup,
                    device_id: self.device.id().0,
                })
        {
            warn!(
                "Failed to send WebGPURequest::RenderBundleEncoderPopDebugGroup({:?}) ({error})",
                self.id()
            );
        }
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudebugcommandsmixin-insertdebugmarker>
    fn InsertDebugMarker(&self, marker_label: USVString) {
        if let Err(error) =
            self.droppable
                .channel
                .0
                .send(WebGPURequest::RenderBundleEncoderCommand {
                    render_bundle_encoder_id: self.droppable.render_bundle_encoder.0,
                    render_command: RenderBundleCommand::InsertDebugMarker(
                        marker_label.to_string(),
                    ),
                    device_id: self.device.id().0,
                })
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
    ) -> DomRoot<D::GPURenderBundle> {
        let desc = wgpu_types::RenderBundleDescriptor {
            label: (&descriptor.parent).convert(),
        };
        let render_bundle_id = self
            .global_from_reflector()
            .global_wgpu_id_hub()
            .create_render_bundle_id();

        if let Err(error) =
            self.droppable
                .channel
                .0
                .send(WebGPURequest::RenderBundleEncoderFinish {
                    render_bundle_encoder_id: self.id().0,
                    descriptor: desc,
                    render_bundle_id,
                    device_id: self.device.id().0,
                })
        {
            warn!(
                "Failed to send WebGPURequest::RenderBundleEncoderFinish({:?}) ({error})",
                self.id()
            );
        }

        let render_bundle = WebGPURenderBundle(render_bundle_id);
        GPURenderBundle::new(
            cx,
            &*self.global_from_reflector(),
            render_bundle,
            self.device.id(),
            self.droppable.channel.clone(),
            descriptor.parent.label.clone(),
        )
    }
}
