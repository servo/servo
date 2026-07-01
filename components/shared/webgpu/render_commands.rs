/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Render pass commands

use serde::{Deserialize, Serialize};
use wgpu_core::command::{PassStateError};
use wgpu_core::global::Global;
use wgpu_core::id::{BindGroupId, BufferId, RenderBundleId, RenderPassEncoderId, RenderPipelineId};

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
        buffer_id: Option<BufferId>,
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
    PushDebugGroup(String),
    PopDebugGroup,
    InsertDebugMarker(String),
}

pub fn apply_render_command(
    global: &Global,
    render_pass_id: RenderPassEncoderId,
    command: RenderCommand,
) -> Result<(), PassStateError> {
    match command {
        RenderCommand::SetPipeline(pipeline_id) => {
            global.render_pass_set_pipeline_with_id(render_pass_id, pipeline_id)
        },
        RenderCommand::SetBindGroup {
            index,
            bind_group_id,
            offsets,
        } => global.render_pass_set_bind_group_with_id(render_pass_id, index, Some(bind_group_id), &offsets),
        RenderCommand::SetViewport {
            x,
            y,
            width,
            height,
            min_depth,
            max_depth,
        } => global.render_pass_set_viewport_with_id(render_pass_id, x, y, width, height, min_depth, max_depth),
        RenderCommand::SetScissorRect {
            x,
            y,
            width,
            height,
        } => global.render_pass_set_scissor_rect_with_id(render_pass_id, x, y, width, height),
        RenderCommand::SetBlendConstant(color) => {
            global.render_pass_set_blend_constant_with_id(render_pass_id, color)
        },
        RenderCommand::SetStencilReference(reference) => {
            global.render_pass_set_stencil_reference_with_id(render_pass_id, reference)
        },
        RenderCommand::SetIndexBuffer {
            buffer_id,
            index_format,
            offset,
            size,
        } => global.render_pass_set_index_buffer_with_id(render_pass_id, buffer_id, index_format, offset, size),
        RenderCommand::SetVertexBuffer {
            slot,
            buffer_id,
            offset,
            size,
        } => global.render_pass_set_vertex_buffer_with_id(render_pass_id, slot, buffer_id, offset, size),
        RenderCommand::Draw {
            vertex_count,
            instance_count,
            first_vertex,
            first_instance,
        } => global.render_pass_draw_with_id(render_pass_id, vertex_count, instance_count, first_vertex, first_instance),
        RenderCommand::DrawIndexed {
            index_count,
            instance_count,
            first_index,
            base_vertex,
            first_instance,
        } => global.render_pass_draw_indexed_with_id(
            render_pass_id,
            index_count,
            instance_count,
            first_index,
            base_vertex,
            first_instance,
        ),
        RenderCommand::DrawIndirect { buffer_id, offset } => {
            global.render_pass_draw_indirect_with_id(render_pass_id, buffer_id, offset)
        },
        RenderCommand::DrawIndexedIndirect { buffer_id, offset } => {
            global.render_pass_draw_indexed_indirect_with_id(render_pass_id, buffer_id, offset)
        },
        RenderCommand::ExecuteBundles(bundles) => {
            global.render_pass_execute_bundles_with_id(render_pass_id, &bundles)
        },
        RenderCommand::PushDebugGroup(label) => {
            global.render_pass_push_debug_group_with_id(render_pass_id, &label, 0)
        },
        RenderCommand::PopDebugGroup => global.render_pass_pop_debug_group_with_id(render_pass_id),
        RenderCommand::InsertDebugMarker(label) => {
            global.render_pass_insert_debug_marker_with_id(render_pass_id, &label, 0)
        },
    }
}
