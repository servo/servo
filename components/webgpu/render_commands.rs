/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Render pass commands

use serde::{Deserialize, Serialize};
use wgpu_core::command::{DynRenderPass, RenderPassError};
use wgpu_core::global::Global;

use crate::wgc::id;
use crate::wgt;

/// <https://github.com/gfx-rs/wgpu/blob/f25e07b984ab391628d9568296d5970981d79d8b/wgpu-core/src/command/render_command.rs#L17>
#[derive(Debug, Deserialize, Serialize)]
pub enum RenderCommand {
    SetPipeline(id::RenderPipelineId),
    SetBindGroup {
        index: u32,
        bind_group_id: id::BindGroupId,
        offsets: Vec<u32>,
    },
    SetViewport {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        min_depth: f32,
        max_depth: f32,
    },
    SetScissorRect {
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    },
    SetBlendConstant(wgt::Color),
    SetStencilReference(u32),
    SetIndexBuffer {
        buffer_id: id::BufferId,
        index_format: wgt::IndexFormat,
        offset: u64,
        size: Option<wgt::BufferSize>,
    },
    SetVertexBuffer {
        slot: u32,
        buffer_id: id::BufferId,
        offset: u64,
        size: Option<wgt::BufferSize>,
    },
    Draw {
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    },
    DrawIndexed {
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        base_vertex: i32,
        first_instance: u32,
    },
    DrawIndirect {
        buffer_id: id::BufferId,
        offset: u64,
    },
    DrawIndexedIndirect {
        buffer_id: id::BufferId,
        offset: u64,
    },
    ExecuteBundles(Vec<id::RenderBundleId>),
}

pub fn apply_render_command(
    context: &Global,
    pass: &mut Box<dyn DynRenderPass>,
    command: RenderCommand,
) -> Result<(), RenderPassError> {
    match command {
        RenderCommand::SetPipeline(pipeline_id) => pass.set_pipeline(context, pipeline_id),
        RenderCommand::SetBindGroup {
            index,
            bind_group_id,
            offsets,
        } => pass.set_bind_group(context, index, bind_group_id, &offsets),
        RenderCommand::SetViewport {
            x,
            y,
            width,
            height,
            min_depth,
            max_depth,
        } => pass.set_viewport(context, x, y, width, height, min_depth, max_depth),
        RenderCommand::SetScissorRect {
            x,
            y,
            width,
            height,
        } => pass.set_scissor_rect(context, x, y, width, height),
        RenderCommand::SetBlendConstant(color) => pass.set_blend_constant(context, color),
        RenderCommand::SetStencilReference(reference) => {
            pass.set_stencil_reference(context, reference)
        },
        RenderCommand::SetIndexBuffer {
            buffer_id,
            index_format,
            offset,
            size,
        } => pass.set_index_buffer(context, buffer_id, index_format, offset, size),
        RenderCommand::SetVertexBuffer {
            slot,
            buffer_id,
            offset,
            size,
        } => pass.set_vertex_buffer(context, slot, buffer_id, offset, size),
        RenderCommand::Draw {
            vertex_count,
            instance_count,
            first_vertex,
            first_instance,
        } => pass.draw(
            context,
            vertex_count,
            instance_count,
            first_vertex,
            first_instance,
        ),
        RenderCommand::DrawIndexed {
            index_count,
            instance_count,
            first_index,
            base_vertex,
            first_instance,
        } => pass.draw_indexed(
            context,
            index_count,
            instance_count,
            first_index,
            base_vertex,
            first_instance,
        ),
        RenderCommand::DrawIndirect { buffer_id, offset } => {
            pass.draw_indirect(context, buffer_id, offset)
        },
        RenderCommand::DrawIndexedIndirect { buffer_id, offset } => {
            pass.draw_indexed_indirect(context, buffer_id, offset)
        },
        RenderCommand::ExecuteBundles(bundles) => pass.execute_bundles(context, &bundles),
    }
}
