/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::GPUCommandEncoderBinding::GPUColor;
use crate::dom::bindings::codegen::Bindings::GPURenderPassEncoderBinding::GPURenderPassEncoderMethods;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::USVString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::gpubindgroup::GPUBindGroup;
use crate::dom::gpubuffer::GPUBuffer;
use crate::dom::gpucommandencoder::{GPUCommandEncoder, GPUCommandEncoderState};
use crate::dom::gpurenderbundle::GPURenderBundle;
use crate::dom::gpurenderpipeline::GPURenderPipeline;
use dom_struct::dom_struct;
use webgpu::{
    wgpu::command::{render_ffi as wgpu_render, RenderPass},
    wgt, WebGPU, WebGPURequest,
};

#[dom_struct]
pub struct GPURenderPassEncoder {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "defined in webgpu"]
    channel: WebGPU,
    label: DomRefCell<Option<USVString>>,
    #[ignore_malloc_size_of = "defined in wgpu-core"]
    render_pass: DomRefCell<Option<RenderPass>>,
    command_encoder: Dom<GPUCommandEncoder>,
}

impl GPURenderPassEncoder {
    fn new_inherited(
        channel: WebGPU,
        render_pass: Option<RenderPass>,
        parent: &GPUCommandEncoder,
        label: Option<USVString>,
    ) -> Self {
        Self {
            channel,
            reflector_: Reflector::new(),
            label: DomRefCell::new(label),
            render_pass: DomRefCell::new(render_pass),
            command_encoder: Dom::from_ref(parent),
        }
    }

    pub fn new(
        global: &GlobalScope,
        channel: WebGPU,
        render_pass: Option<RenderPass>,
        parent: &GPUCommandEncoder,
        label: Option<USVString>,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(GPURenderPassEncoder::new_inherited(
                channel,
                render_pass,
                parent,
                label,
            )),
            global,
        )
    }
}

impl GPURenderPassEncoderMethods for GPURenderPassEncoder {
    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn GetLabel(&self) -> Option<USVString> {
        self.label.borrow().clone()
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn SetLabel(&self, value: Option<USVString>) {
        *self.label.borrow_mut() = value;
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuprogrammablepassencoder-setbindgroup
    #[allow(unsafe_code)]
    fn SetBindGroup(&self, index: u32, bind_group: &GPUBindGroup, dynamic_offsets: Vec<u32>) {
        if let Some(render_pass) = self.render_pass.borrow_mut().as_mut() {
            unsafe {
                wgpu_render::wgpu_render_pass_set_bind_group(
                    render_pass,
                    index,
                    bind_group.id().0,
                    dynamic_offsets.as_ptr(),
                    dynamic_offsets.len(),
                )
            };
        }
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpurenderpassencoder-setviewport
    fn SetViewport(
        &self,
        x: Finite<f32>,
        y: Finite<f32>,
        width: Finite<f32>,
        height: Finite<f32>,
        min_depth: Finite<f32>,
        max_depth: Finite<f32>,
    ) {
        if let Some(render_pass) = self.render_pass.borrow_mut().as_mut() {
            wgpu_render::wgpu_render_pass_set_viewport(
                render_pass,
                *x,
                *y,
                *width,
                *height,
                *min_depth,
                *max_depth,
            );
        }
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpurenderpassencoder-setscissorrect
    fn SetScissorRect(&self, x: u32, y: u32, width: u32, height: u32) {
        if let Some(render_pass) = self.render_pass.borrow_mut().as_mut() {
            wgpu_render::wgpu_render_pass_set_scissor_rect(render_pass, x, y, width, height);
        }
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpurenderpassencoder-setblendcolor
    fn SetBlendColor(&self, color: GPUColor) {
        if let Some(render_pass) = self.render_pass.borrow_mut().as_mut() {
            let colors = match color {
                GPUColor::GPUColorDict(d) => wgt::Color {
                    r: *d.r,
                    g: *d.g,
                    b: *d.b,
                    a: *d.a,
                },
                GPUColor::DoubleSequence(mut s) => {
                    if s.len() < 3 {
                        s.resize(3, Finite::wrap(0.0f64));
                    }
                    s.resize(4, Finite::wrap(1.0f64));
                    wgt::Color {
                        r: *s[0],
                        g: *s[1],
                        b: *s[2],
                        a: *s[3],
                    }
                },
            };
            wgpu_render::wgpu_render_pass_set_blend_color(render_pass, &colors);
        }
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpurenderpassencoder-setstencilreference
    fn SetStencilReference(&self, reference: u32) {
        if let Some(render_pass) = self.render_pass.borrow_mut().as_mut() {
            wgpu_render::wgpu_render_pass_set_stencil_reference(render_pass, reference);
        }
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpurenderpassencoder-endpass
    fn EndPass(&self) {
        let render_pass = self.render_pass.borrow_mut().take();
        self.channel
            .0
            .send((
                None,
                WebGPURequest::RunRenderPass {
                    command_encoder_id: self.command_encoder.id().0,
                    render_pass,
                },
            ))
            .expect("Failed to send RunRenderPass");

        self.command_encoder.set_state(
            GPUCommandEncoderState::Open,
            GPUCommandEncoderState::EncodingRenderPass,
        );
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpurenderencoderbase-setpipeline
    fn SetPipeline(&self, pipeline: &GPURenderPipeline) {
        if let Some(render_pass) = self.render_pass.borrow_mut().as_mut() {
            wgpu_render::wgpu_render_pass_set_pipeline(render_pass, pipeline.id().0);
        }
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpurenderencoderbase-setindexbuffer
    fn SetIndexBuffer(&self, buffer: &GPUBuffer, offset: u64, size: u64) {
        if let Some(render_pass) = self.render_pass.borrow_mut().as_mut() {
            wgpu_render::wgpu_render_pass_set_index_buffer(
                render_pass,
                buffer.id().0,
                offset,
                wgt::BufferSize::new(size),
            );
        }
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpurenderencoderbase-setvertexbuffer
    fn SetVertexBuffer(&self, slot: u32, buffer: &GPUBuffer, offset: u64, size: u64) {
        if let Some(render_pass) = self.render_pass.borrow_mut().as_mut() {
            wgpu_render::wgpu_render_pass_set_vertex_buffer(
                render_pass,
                slot,
                buffer.id().0,
                offset,
                wgt::BufferSize::new(size),
            );
        }
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpurenderencoderbase-draw
    fn Draw(&self, vertex_count: u32, instance_count: u32, first_vertex: u32, first_instance: u32) {
        if let Some(render_pass) = self.render_pass.borrow_mut().as_mut() {
            wgpu_render::wgpu_render_pass_draw(
                render_pass,
                vertex_count,
                instance_count,
                first_vertex,
                first_instance,
            );
        }
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpurenderencoderbase-drawindexed
    fn DrawIndexed(
        &self,
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        base_vertex: i32,
        first_instance: u32,
    ) {
        if let Some(render_pass) = self.render_pass.borrow_mut().as_mut() {
            wgpu_render::wgpu_render_pass_draw_indexed(
                render_pass,
                index_count,
                instance_count,
                first_index,
                base_vertex,
                first_instance,
            );
        }
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpurenderencoderbase-drawindirect
    fn DrawIndirect(&self, indirect_buffer: &GPUBuffer, indirect_offset: u64) {
        if let Some(render_pass) = self.render_pass.borrow_mut().as_mut() {
            wgpu_render::wgpu_render_pass_draw_indirect(
                render_pass,
                indirect_buffer.id().0,
                indirect_offset,
            );
        }
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpurenderencoderbase-drawindexedindirect
    fn DrawIndexedIndirect(&self, indirect_buffer: &GPUBuffer, indirect_offset: u64) {
        if let Some(render_pass) = self.render_pass.borrow_mut().as_mut() {
            wgpu_render::wgpu_render_pass_draw_indexed_indirect(
                render_pass,
                indirect_buffer.id().0,
                indirect_offset,
            );
        }
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpurenderpassencoder-executebundles
    #[allow(unsafe_code)]
    fn ExecuteBundles(&self, bundles: Vec<DomRoot<GPURenderBundle>>) {
        let bundle_ids = bundles.iter().map(|b| b.id().0).collect::<Vec<_>>();
        if let Some(render_pass) = self.render_pass.borrow_mut().as_mut() {
            unsafe {
                wgpu_render::wgpu_render_pass_execute_bundles(
                    render_pass,
                    bundle_ids.as_ptr(),
                    bundle_ids.len(),
                )
            };
        }
    }
}
