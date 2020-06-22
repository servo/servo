/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::GPUCommandEncoderBinding::GPUColor;
use crate::dom::bindings::codegen::Bindings::GPURenderPassEncoderBinding::GPURenderPassEncoderMethods;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::gpubindgroup::GPUBindGroup;
use crate::dom::gpubuffer::GPUBuffer;
use crate::dom::gpucommandencoder::{GPUCommandEncoder, GPUCommandEncoderState};
use crate::dom::gpurenderpipeline::GPURenderPipeline;
use dom_struct::dom_struct;
use webgpu::{
    wgpu::command::{render_ffi as wgpu_render, RawPass},
    wgpu::id,
    wgt, WebGPU, WebGPURequest,
};

#[dom_struct]
pub struct GPURenderPassEncoder {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "defined in webgpu"]
    channel: WebGPU,
    label: DomRefCell<Option<DOMString>>,
    #[ignore_malloc_size_of = "defined in wgpu-core"]
    raw_pass: DomRefCell<Option<RawPass<id::CommandEncoderId>>>,
    command_encoder: Dom<GPUCommandEncoder>,
}

impl GPURenderPassEncoder {
    fn new_inherited(
        channel: WebGPU,
        raw_pass: RawPass<id::CommandEncoderId>,
        parent: &GPUCommandEncoder,
    ) -> Self {
        Self {
            channel,
            reflector_: Reflector::new(),
            label: DomRefCell::new(None),
            raw_pass: DomRefCell::new(Some(raw_pass)),
            command_encoder: Dom::from_ref(parent),
        }
    }

    pub fn new(
        global: &GlobalScope,
        channel: WebGPU,
        raw_pass: RawPass<id::CommandEncoderId>,
        parent: &GPUCommandEncoder,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(GPURenderPassEncoder::new_inherited(
                channel, raw_pass, parent,
            )),
            global,
        )
    }
}

impl GPURenderPassEncoderMethods for GPURenderPassEncoder {
    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn GetLabel(&self) -> Option<DOMString> {
        self.label.borrow().clone()
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn SetLabel(&self, value: Option<DOMString>) {
        *self.label.borrow_mut() = value;
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuprogrammablepassencoder-setbindgroup
    fn SetBindGroup(&self, index: u32, bind_group: &GPUBindGroup, dynamic_offsets: Vec<u32>) {
        if let Some(raw_pass) = self.raw_pass.borrow_mut().as_mut() {
            unsafe {
                wgpu_render::wgpu_render_pass_set_bind_group(
                    raw_pass,
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
        if let Some(raw_pass) = self.raw_pass.borrow_mut().as_mut() {
            unsafe {
                wgpu_render::wgpu_render_pass_set_viewport(
                    raw_pass, *x, *y, *width, *height, *min_depth, *max_depth,
                )
            };
        }
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpurenderpassencoder-setscissorrect
    fn SetScissorRect(&self, x: u32, y: u32, width: u32, height: u32) {
        if width <= 0 || height <= 0 {
            return warn!("Cannot set scissor rect- width and height must greater than 0");
        }
        if let Some(raw_pass) = self.raw_pass.borrow_mut().as_mut() {
            unsafe {
                wgpu_render::wgpu_render_pass_set_scissor_rect(raw_pass, x, y, width, height)
            };
        }
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpurenderpassencoder-setblendcolor
    fn SetBlendColor(&self, color: GPUColor) {
        let colors = match color {
            GPUColor::GPUColorDict(d) => wgt::Color {
                r: *d.r,
                g: *d.g,
                b: *d.b,
                a: *d.a,
            },
            GPUColor::DoubleSequence(s) => wgt::Color {
                r: *s[0],
                g: *s[1],
                b: *s[2],
                a: *s[3],
            },
        };
        if let Some(raw_pass) = self.raw_pass.borrow_mut().as_mut() {
            unsafe { wgpu_render::wgpu_render_pass_set_blend_color(raw_pass, &colors) };
        }
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpurenderpassencoder-setstencilreference
    fn SetStencilReference(&self, reference: u32) {
        if let Some(raw_pass) = self.raw_pass.borrow_mut().as_mut() {
            unsafe { wgpu_render::wgpu_render_pass_set_stencil_reference(raw_pass, reference) };
        }
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpurenderpassencoder-endpass
    fn EndPass(&self) {
        if let Some(raw_pass) = self.raw_pass.borrow_mut().take() {
            let (pass_data, command_encoder_id) = unsafe { raw_pass.finish_render() };

            self.channel
                .0
                .send(WebGPURequest::RunRenderPass {
                    command_encoder_id,
                    pass_data,
                })
                .unwrap();

            self.command_encoder.set_state(
                GPUCommandEncoderState::Open,
                GPUCommandEncoderState::EncodingRenderPass,
            );
        }
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpurenderencoderbase-setpipeline
    fn SetPipeline(&self, pipeline: &GPURenderPipeline) {
        if let Some(raw_pass) = self.raw_pass.borrow_mut().as_mut() {
            unsafe { wgpu_render::wgpu_render_pass_set_pipeline(raw_pass, pipeline.id().0) };
        }
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpurenderencoderbase-setindexbuffer
    fn SetIndexBuffer(&self, buffer: &GPUBuffer, offset: u64, size: u64) {
        let s = if size == 0 {
            wgt::BufferSize::WHOLE
        } else {
            wgt::BufferSize(size)
        };

        if let Some(raw_pass) = self.raw_pass.borrow_mut().as_mut() {
            unsafe {
                wgpu_render::wgpu_render_pass_set_index_buffer(raw_pass, buffer.id().0, offset, s)
            };
        }
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpurenderencoderbase-setvertexbuffer
    fn SetVertexBuffer(&self, slot: u32, buffer: &GPUBuffer, offset: u64, size: u64) {
        let s = if size == 0 {
            wgt::BufferSize::WHOLE
        } else {
            wgt::BufferSize(size)
        };

        if let Some(raw_pass) = self.raw_pass.borrow_mut().as_mut() {
            unsafe {
                wgpu_render::wgpu_render_pass_set_vertex_buffer(
                    raw_pass,
                    slot,
                    buffer.id().0,
                    offset,
                    s,
                )
            };
        }
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpurenderencoderbase-draw
    fn Draw(&self, vertex_count: u32, instance_count: u32, first_vertex: u32, first_instance: u32) {
        if let Some(raw_pass) = self.raw_pass.borrow_mut().as_mut() {
            unsafe {
                wgpu_render::wgpu_render_pass_draw(
                    raw_pass,
                    vertex_count,
                    instance_count,
                    first_vertex,
                    first_instance,
                )
            };
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
        if let Some(raw_pass) = self.raw_pass.borrow_mut().as_mut() {
            unsafe {
                wgpu_render::wgpu_render_pass_draw_indexed(
                    raw_pass,
                    index_count,
                    instance_count,
                    first_index,
                    base_vertex,
                    first_instance,
                )
            };
        }
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpurenderencoderbase-drawindirect
    fn DrawIndirect(&self, indirect_buffer: &GPUBuffer, indirect_offset: u64) {
        if let Some(raw_pass) = self.raw_pass.borrow_mut().as_mut() {
            unsafe {
                wgpu_render::wgpu_render_pass_draw_indirect(
                    raw_pass,
                    indirect_buffer.id().0,
                    indirect_offset,
                )
            };
        }
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpurenderencoderbase-drawindexedindirect
    fn DrawIndexedIndirect(&self, indirect_buffer: &GPUBuffer, indirect_offset: u64) {
        if let Some(raw_pass) = self.raw_pass.borrow_mut().as_mut() {
            unsafe {
                wgpu_render::wgpu_render_pass_draw_indexed_indirect(
                    raw_pass,
                    indirect_buffer.id().0,
                    indirect_offset,
                )
            };
        }
    }
}
