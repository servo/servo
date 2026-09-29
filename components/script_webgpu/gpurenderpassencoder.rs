/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::{JSContext, NoGC};
use log::warn;
use malloc_size_of_derive::MallocSizeOf;
use script_bindings::DomTypes;
use script_bindings::cell::DomRefCell;
use script_bindings::codegen::GenericBindings::WebGPUBinding::{
    GPUIndexFormat, GPURenderPassEncoderMethods, GPURenderPassEncoderWrap,
};
use script_bindings::codegen::GenericUnionTypes::DoubleSequenceOrGPUColorDict as GPUColor;
use script_bindings::error::Fallible;
use script_bindings::interfaces::PromiseHelpers;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_wrap};
use script_bindings::root::DomRoot;
use webgpu_traits::{
    BindingCommand, BufferSize, DebugCommand, IndexFormat, RenderCommand, RenderPassEncoderCommand,
    WebGPU, WebGPURenderPass, WebGPURequest,
};

use crate::JSTraceable;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::root::Dom;
use crate::dom::bindings::str::USVString;
use crate::gpubindgroup::GPUBindGroup;
use crate::gpubuffer::GPUBuffer;
use crate::gpucommandencoder::GPUCommandEncoder;
use crate::gpuconvert::WebGPUTryConvert;
use crate::gpurenderbundle::GPURenderBundle;
use crate::gpurenderpipeline::GPURenderPipeline;
use crate::traits::{Equivalence, WebGPUPromise};

#[derive(MallocSizeOf)]
struct DroppableGPURenderPassEncoder {
    channel: WebGPU,
    render_pass: WebGPURenderPass,
}

impl Drop for DroppableGPURenderPassEncoder {
    fn drop(&mut self) {
        if let Err(e) = self
            .channel
            .0
            .send(WebGPURequest::DropRenderPass(self.render_pass.0))
        {
            warn!("Failed to send WebGPURequest::DropRenderPass with {e:?}");
        }
    }
}
#[dom_struct]
pub struct GPURenderPassEncoder<D: DomTypes> {
    reflector_: Reflector,
    label: DomRefCell<USVString>,
    command_encoder: Dom<GPUCommandEncoder<D>>,
    #[no_trace]
    droppable: DroppableGPURenderPassEncoder,
}

impl<D> GPURenderPassEncoder<D>
where
    D: Equivalence,
    <D::Promise as PromiseHelpers<D>>::StackRoot: WebGPUPromise<D>,
{
    fn new_inherited(
        channel: WebGPU,
        render_pass: WebGPURenderPass,
        parent: &GPUCommandEncoder<D>,
        label: USVString,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            label: DomRefCell::new(label),
            command_encoder: Dom::from_ref(parent),
            droppable: DroppableGPURenderPassEncoder {
                channel,
                render_pass,
            },
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        global: &D::GlobalScope,
        channel: WebGPU,
        render_pass: WebGPURenderPass,
        parent: &GPUCommandEncoder<D>,
        label: USVString,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_wrap::<D, _, _>(
            cx,
            Box::new(GPURenderPassEncoder::new_inherited(
                channel,
                render_pass,
                parent,
                label,
            )),
            global,
            GPURenderPassEncoderWrap::<D>,
        )
    }

    fn send_render_command(&self, render_command: RenderPassEncoderCommand) {
        if let Err(e) = self
            .droppable
            .channel
            .0
            .send(WebGPURequest::RenderPassCommand {
                render_pass_id: self.id().0,
                render_command,
                device_id: self.command_encoder.device_id().0,
            })
        {
            warn!("Error sending WebGPURequest::RenderPassCommand: {e:?}")
        }
    }
}

impl<D: DomTypes> GPURenderPassEncoder<D> {
    pub(crate) fn id(&self) -> WebGPURenderPass {
        self.droppable.render_pass
    }
}

impl<D> GPURenderPassEncoderMethods<D> for GPURenderPassEncoder<D>
where
    D: Equivalence,
    <D::Promise as PromiseHelpers<D>>::StackRoot: WebGPUPromise<D>,
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
    fn SetBindGroup(&self, index: u32, bind_group: &GPUBindGroup<D>, offsets: Vec<u32>) {
        self.send_render_command(RenderPassEncoderCommand::BindingCommand(
            BindingCommand::SetBindGroup {
                index,
                bind_group: Some(bind_group.id().0),
                dynamic_offsets: offsets,
            },
        ))
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpurenderpassencoder-setviewport>
    fn SetViewport(
        &self,
        x: Finite<f32>,
        y: Finite<f32>,
        width: Finite<f32>,
        height: Finite<f32>,
        min_depth: Finite<f32>,
        max_depth: Finite<f32>,
    ) {
        self.send_render_command(RenderPassEncoderCommand::SetViewport {
            x: *x,
            y: *y,
            width: *width,
            height: *height,
            min_depth: *min_depth,
            max_depth: *max_depth,
        })
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpurenderpassencoder-setscissorrect>
    fn SetScissorRect(&self, x: u32, y: u32, width: u32, height: u32) {
        self.send_render_command(RenderPassEncoderCommand::SetScissorRect {
            x,
            y,
            width,
            height,
        })
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpurenderpassencoder-setblendcolor>
    fn SetBlendConstant(&self, color: GPUColor) -> Fallible<()> {
        self.send_render_command(RenderPassEncoderCommand::SetBlendConstant(
            (&color).try_convert()?,
        ));
        Ok(())
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpurenderpassencoder-setstencilreference>
    fn SetStencilReference(&self, reference: u32) {
        self.send_render_command(RenderPassEncoderCommand::SetStencilReference(reference))
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpurenderpassencoder-end>
    fn End(&self) {
        if let Err(e) = self.droppable.channel.0.send(WebGPURequest::EndRenderPass {
            render_pass_id: self.id().0,
            device_id: self.command_encoder.device_id().0,
        }) {
            warn!("Failed to send WebGPURequest::EndRenderPass: {e:?}");
        }
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpurenderencoderbase-setpipeline>
    fn SetPipeline(&self, pipeline: &GPURenderPipeline<D>) {
        self.send_render_command(RenderPassEncoderCommand::RenderCommand(
            RenderCommand::SetPipeline(pipeline.id().0),
        ))
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpurendercommandsmixin-setindexbuffer>
    fn SetIndexBuffer(
        &self,
        buffer: &GPUBuffer<D>,
        index_format: GPUIndexFormat,
        offset: u64,
        size: u64,
    ) {
        self.send_render_command(RenderPassEncoderCommand::RenderCommand(
            RenderCommand::SetIndexBuffer {
                buffer: buffer.id().0,
                index_format: match index_format {
                    GPUIndexFormat::Uint16 => IndexFormat::Uint16,
                    GPUIndexFormat::Uint32 => IndexFormat::Uint32,
                },
                offset,
                size: BufferSize::new(size),
            },
        ))
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpurenderencoderbase-setvertexbuffer>
    fn SetVertexBuffer(&self, slot: u32, buffer: Option<&GPUBuffer<D>>, offset: u64, size: u64) {
        self.send_render_command(RenderPassEncoderCommand::RenderCommand(
            RenderCommand::SetVertexBuffer {
                slot,
                buffer: buffer.map(|b| b.id().0),
                offset,
                size: BufferSize::new(size),
            },
        ))
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpurenderencoderbase-draw>
    fn Draw(&self, vertex_count: u32, instance_count: u32, first_vertex: u32, first_instance: u32) {
        self.send_render_command(RenderPassEncoderCommand::RenderCommand(
            RenderCommand::Draw {
                vertex_count,
                instance_count,
                first_vertex,
                first_instance,
            },
        ))
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
        self.send_render_command(RenderPassEncoderCommand::RenderCommand(
            RenderCommand::DrawIndexed {
                index_count,
                instance_count,
                first_index,
                base_vertex,
                first_instance,
            },
        ))
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpurenderencoderbase-drawindirect>
    fn DrawIndirect(&self, buffer: &GPUBuffer<D>, offset: u64) {
        self.send_render_command(RenderPassEncoderCommand::RenderCommand(
            RenderCommand::DrawIndirect {
                indirect_buffer: buffer.id().0,
                indirect_offset: offset,
            },
        ))
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpurenderencoderbase-drawindexedindirect>
    fn DrawIndexedIndirect(&self, buffer: &GPUBuffer<D>, offset: u64) {
        self.send_render_command(RenderPassEncoderCommand::RenderCommand(
            RenderCommand::DrawIndexedIndirect {
                indirect_buffer: buffer.id().0,
                indirect_offset: offset,
            },
        ))
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpurenderpassencoder-executebundles>
    fn ExecuteBundles(&self, bundles: Vec<DomRoot<GPURenderBundle<D>>>) {
        let bundle_ids: Vec<_> = bundles.iter().map(|b| b.id().0).collect();
        self.send_render_command(RenderPassEncoderCommand::ExecuteBundles(bundle_ids))
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudebugcommandsmixin-pushdebuggroup>
    fn PushDebugGroup(&self, group_label: USVString) {
        self.send_render_command(RenderPassEncoderCommand::DebugCommand(
            DebugCommand::PushDebugGroup(group_label.to_string()),
        ))
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudebugcommandsmixin-popdebuggroup>
    fn PopDebugGroup(&self) {
        self.send_render_command(RenderPassEncoderCommand::DebugCommand(
            DebugCommand::PopDebugGroup,
        ))
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudebugcommandsmixin-insertdebugmarker>
    fn InsertDebugMarker(&self, marker_label: USVString) {
        self.send_render_command(RenderPassEncoderCommand::DebugCommand(
            DebugCommand::InsertDebugMarker(marker_label.to_string()),
        ))
    }
}
