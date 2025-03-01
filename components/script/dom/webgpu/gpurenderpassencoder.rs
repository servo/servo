/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use webgpu::{wgt, RenderCommand, WebGPU, WebGPURenderPass, WebGPURequest};

use crate::conversions::TryConvert;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::{
    GPUColor, GPUIndexFormat, GPURenderPassEncoderMethods,
};
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::USVString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::webgpu::gpubindgroup::GPUBindGroup;
use crate::dom::webgpu::gpubuffer::GPUBuffer;
use crate::dom::webgpu::gpucommandencoder::GPUCommandEncoder;
use crate::dom::webgpu::gpurenderbundle::GPURenderBundle;
use crate::dom::webgpu::gpurenderpipeline::GPURenderPipeline;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct GPURenderPassEncoder {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "defined in webgpu"]
    #[no_trace]
    channel: WebGPU,
    label: DomRefCell<USVString>,
    #[no_trace]
    render_pass: WebGPURenderPass,
    command_encoder: Dom<GPUCommandEncoder>,
}

impl GPURenderPassEncoder {
    fn new_inherited(
        channel: WebGPU,
        render_pass: WebGPURenderPass,
        parent: &GPUCommandEncoder,
        label: USVString,
    ) -> Self {
        Self {
            channel,
            reflector_: Reflector::new(),
            label: DomRefCell::new(label),
            render_pass,
            command_encoder: Dom::from_ref(parent),
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        channel: WebGPU,
        render_pass: WebGPURenderPass,
        parent: &GPUCommandEncoder,
        label: USVString,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(GPURenderPassEncoder::new_inherited(
                channel,
                render_pass,
                parent,
                label,
            )),
            global,
            can_gc,
        )
    }

    fn send_render_command(&self, render_command: RenderCommand) {
        if let Err(e) = self.channel.0.send(WebGPURequest::RenderPassCommand {
            render_pass_id: self.render_pass.0,
            render_command,
            device_id: self.command_encoder.device_id().0,
        }) {
            warn!("Error sending WebGPURequest::RenderPassCommand: {e:?}")
        }
    }
}

impl GPURenderPassEncoderMethods<crate::DomTypeHolder> for GPURenderPassEncoder {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn Label(&self) -> USVString {
        self.label.borrow().clone()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn SetLabel(&self, value: USVString) {
        *self.label.borrow_mut() = value;
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuprogrammablepassencoder-setbindgroup>
    fn SetBindGroup(&self, index: u32, bind_group: &GPUBindGroup, offsets: Vec<u32>) {
        self.send_render_command(RenderCommand::SetBindGroup {
            index,
            bind_group_id: bind_group.id().0,
            offsets,
        })
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
        self.send_render_command(RenderCommand::SetViewport {
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
        self.send_render_command(RenderCommand::SetScissorRect {
            x,
            y,
            width,
            height,
        })
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpurenderpassencoder-setblendcolor>
    fn SetBlendConstant(&self, color: GPUColor) -> Fallible<()> {
        self.send_render_command(RenderCommand::SetBlendConstant((&color).try_convert()?));
        Ok(())
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpurenderpassencoder-setstencilreference>
    fn SetStencilReference(&self, reference: u32) {
        self.send_render_command(RenderCommand::SetStencilReference(reference))
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpurenderpassencoder-end>
    fn End(&self) {
        if let Err(e) = self.channel.0.send(WebGPURequest::EndRenderPass {
            render_pass_id: self.render_pass.0,
            device_id: self.command_encoder.device_id().0,
            command_encoder_id: self.command_encoder.id().0,
        }) {
            warn!("Failed to send WebGPURequest::EndRenderPass: {e:?}");
        }
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpurenderencoderbase-setpipeline>
    fn SetPipeline(&self, pipeline: &GPURenderPipeline) {
        self.send_render_command(RenderCommand::SetPipeline(pipeline.id().0))
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpurendercommandsmixin-setindexbuffer>
    fn SetIndexBuffer(
        &self,
        buffer: &GPUBuffer,
        index_format: GPUIndexFormat,
        offset: u64,
        size: u64,
    ) {
        self.send_render_command(RenderCommand::SetIndexBuffer {
            buffer_id: buffer.id().0,
            index_format: match index_format {
                GPUIndexFormat::Uint16 => wgt::IndexFormat::Uint16,
                GPUIndexFormat::Uint32 => wgt::IndexFormat::Uint32,
            },
            offset,
            size: wgt::BufferSize::new(size),
        })
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpurenderencoderbase-setvertexbuffer>
    fn SetVertexBuffer(&self, slot: u32, buffer: &GPUBuffer, offset: u64, size: u64) {
        self.send_render_command(RenderCommand::SetVertexBuffer {
            slot,
            buffer_id: buffer.id().0,
            offset,
            size: wgt::BufferSize::new(size),
        })
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpurenderencoderbase-draw>
    fn Draw(&self, vertex_count: u32, instance_count: u32, first_vertex: u32, first_instance: u32) {
        self.send_render_command(RenderCommand::Draw {
            vertex_count,
            instance_count,
            first_vertex,
            first_instance,
        })
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
        self.send_render_command(RenderCommand::DrawIndexed {
            index_count,
            instance_count,
            first_index,
            base_vertex,
            first_instance,
        })
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpurenderencoderbase-drawindirect>
    fn DrawIndirect(&self, buffer: &GPUBuffer, offset: u64) {
        self.send_render_command(RenderCommand::DrawIndirect {
            buffer_id: buffer.id().0,
            offset,
        })
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpurenderencoderbase-drawindexedindirect>
    fn DrawIndexedIndirect(&self, buffer: &GPUBuffer, offset: u64) {
        self.send_render_command(RenderCommand::DrawIndexedIndirect {
            buffer_id: buffer.id().0,
            offset,
        })
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpurenderpassencoder-executebundles>
    #[allow(unsafe_code)]
    fn ExecuteBundles(&self, bundles: Vec<DomRoot<GPURenderBundle>>) {
        let bundle_ids: Vec<_> = bundles.iter().map(|b| b.id().0).collect();
        self.send_render_command(RenderCommand::ExecuteBundles(bundle_ids))
    }
}

impl Drop for GPURenderPassEncoder {
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
