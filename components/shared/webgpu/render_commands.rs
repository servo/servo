/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Render pass commands

use serde::{Deserialize, Serialize};
use wgpu_core::command::{RenderPass, RenderPassError};
use wgpu_core::global::Global;
use wgpu_core::id::{BindGroupId, BufferId, RenderBundleId, RenderPipelineId};

/// <https://github.com/gfx-rs/wgpu/blob/f25e07b984ab391628d9568296d5970981d79d8b/wgpu-core/src/command/render_command.rs#L17>
#[derive(Debug, Deserialize, Serialize)]
pub enum RenderCommand {
    SetPipeline(RenderPipelineId),
    SetBindGroup {
        index: u32,
        bind_group_id: BindGroupId,
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
    SetBlendConstant(wgpu_types::Color),
    SetStencilReference(u32),
    SetIndexBuffer {
        buffer_id: BufferId,
        index_format: wgpu_types::IndexFormat,
        offset: u64,
        size: Option<wgpu_types::BufferSize>,
    },
    SetVertexBuffer {
        slot: u32,
        buffer_id: BufferId,
        offset: u64,
        size: Option<wgpu_types::BufferSize>,
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
        buffer_id: BufferId,
        offset: u64,
    },
    DrawIndexedIndirect {
        buffer_id: BufferId,
        offset: u64,
    },
    ExecuteBundles(Vec<RenderBundleId>),
}

pub fn apply_render_command(
    global: &Global,
    pass: &mut RenderPass,
    command: RenderCommand,
) -> Result<(), RenderPassError> {
    match command {
        RenderCommand::SetPipeline(pipeline_id) => {
            global.render_pass_set_pipeline(pass, pipeline_id)
        },
        RenderCommand::SetBindGroup {
            index,
            bind_group_id,
            offsets,
        } => global.render_pass_set_bind_group(pass, index, Some(bind_group_id), &offsets),
        RenderCommand::SetViewport {
            x,
            y,
            width,
            height,
            min_depth,
            max_depth,
        } => global.render_pass_set_viewport(pass, x, y, width, height, min_depth, max_depth),
        RenderCommand::SetScissorRect {
            x,
            y,
            width,
            height,
        } => global.render_pass_set_scissor_rect(pass, x, y, width, height),
        RenderCommand::SetBlendConstant(color) => {
            global.render_pass_set_blend_constant(pass, color)
        },
        RenderCommand::SetStencilReference(reference) => {
            global.render_pass_set_stencil_reference(pass, reference)
        },
        RenderCommand::SetIndexBuffer {
            buffer_id,
            index_format,
            offset,
            size,
        } => global.render_pass_set_index_buffer(pass, buffer_id, index_format, offset, size),
        RenderCommand::SetVertexBuffer {
            slot,
            buffer_id,
            offset,
            size,
        } => global.render_pass_set_vertex_buffer(pass, slot, buffer_id, offset, size),
        RenderCommand::Draw {
            vertex_count,
            instance_count,
            first_vertex,
            first_instance,
        } => global.render_pass_draw(
            pass,
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
        } => global.render_pass_draw_indexed(
            pass,
            index_count,
            instance_count,
            first_index,
            base_vertex,
            first_instance,
        ),
        RenderCommand::DrawIndirect { buffer_id, offset } => {
            global.render_pass_draw_indirect(pass, buffer_id, offset)
        },
        RenderCommand::DrawIndexedIndirect { buffer_id, offset } => {
            global.render_pass_draw_indexed_indirect(pass, buffer_id, offset)
        },
        RenderCommand::ExecuteBundles(bundles) => {
            global.render_pass_execute_bundles(pass, &bundles)
        },
    }
}
