/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Encoders commands

use serde::{Deserialize, Serialize};

use crate::id;

#[derive(Debug, Serialize, Deserialize)]
/// Corresponds to [`GPUCommandEncoder`](https://www.w3.org/TR/webgpu/#gpucommandencoder).
pub enum CommandEncoderCommand<'a> {
    BeginRenderPass {
        desc: crate::RenderPassDescriptor<'a>,
        render_pass_encoder_id: id::RenderPassEncoderId,
    },
    BeginComputePass {
        desc: crate::ComputePassDescriptor<'a>, // optional, defaults to {}
        compute_pass_encoder_id: id::ComputePassEncoderId,
    },
    CopyBufferToBuffer {
        source: id::BufferId,
        source_offset: u64,
        destination: id::BufferId,
        destination_offset: u64,
        size: Option<u64>,
    },
    CopyBufferToTexture {
        source: crate::TexelCopyBufferInfo,
        destination: crate::TexelCopyTextureInfo,
        copy_size: crate::Extent3d,
    },
    CopyTextureToBuffer {
        source: crate::TexelCopyTextureInfo,
        destination: crate::TexelCopyBufferInfo,
        copy_size: crate::Extent3d,
    },
    CopyTextureToTexture {
        source: crate::TexelCopyTextureInfo,
        destination: crate::TexelCopyTextureInfo,
        copy_size: crate::Extent3d,
    },
    ClearBuffer {
        buffer: id::BufferId,
        offset: u64, // optional, defaults to 0
        size: Option<u64>,
    },
    ResolveQuerySet {
        query_set: id::QuerySetId,
        first_query: u32,
        query_count: u32,
        destination: id::BufferId,
        destination_offset: u64,
    },
    DebugCommand(DebugCommand),
}

#[derive(Debug, Serialize, Deserialize)]
/// Corresponds to [`GPURenderPassEncoder`](https://www.w3.org/TR/webgpu/#gpurenderpassencoder).
pub enum RenderPassEncoderCommand {
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
    SetBlendConstant(crate::Color),
    SetStencilReference(u32),
    BeginOcclusionQuery(u32),
    EndOcclusionQuery,
    ExecuteBundles(Vec<id::RenderBundleId>),
    BindingCommand(BindingCommand),
    RenderCommand(RenderCommand),
    DebugCommand(DebugCommand),
}

#[derive(Debug, Serialize, Deserialize)]
/// Corresponds to [`GPURenderBundleEncoder`](https://www.w3.org/TR/webgpu/#gpurenderbundleencoder).
pub enum RenderBundleEncoderCommand {
    BindingCommand(BindingCommand),
    RenderCommand(RenderCommand),
    DebugCommand(DebugCommand),
}

#[derive(Debug, Serialize, Deserialize)]
/// Corresponds to [`GPUComputePassEncoder`](https://www.w3.org/TR/webgpu/#gpucomputepassencoder).
pub enum ComputePassEncoderCommand {
    BindingCommand(BindingCommand),
    SetPipeline(id::ComputePipelineId),
    DispatchWorkgroups {
        workgroup_count_x: u32,
        workgroup_count_y: u32, // optional, defaults to 1
        workgroup_count_z: u32, // optional, defaults to 1
    },
    DispatchWorkgroupsIndirect {
        indirect_buffer: id::BufferId,
        indirect_offset: u64,
    },
    DebugCommand(DebugCommand),
}

#[derive(Debug, Serialize, Deserialize)]
/// Corresponds to [`GPUDebugCommandsMixin`](https://www.w3.org/TR/webgpu/#gpudebugcommandsmixin).
pub enum DebugCommand {
    PushDebugGroup(String),
    PopDebugGroup,
    InsertDebugMarker(String),
}

#[derive(Debug, Serialize, Deserialize)]
/// Corresponds to [`GPUBindingCommandsMixin`](https://www.w3.org/TR/webgpu/#gpubindingcommandsmixin).
pub enum BindingCommand {
    SetBindGroup {
        index: u32,
        bind_group: Option<id::BindGroupId>,
        dynamic_offsets: Vec<u32>, // optional, defaults to []
    },
    SetImmediates {
        range_offset: u32,
        data: Vec<u8>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
/// Corresponds to [`GPURenderCommandsMixin`](https://www.w3.org/TR/webgpu/#gpurendercommandsmixin).
pub enum RenderCommand {
    SetPipeline(id::RenderPipelineId),
    SetIndexBuffer {
        buffer: id::BufferId,
        index_format: crate::IndexFormat,
        offset: u64, // optional, defaults to 0
        size: Option<crate::BufferSize>,
    },
    SetVertexBuffer {
        slot: u32,
        buffer: Option<id::BufferId>,
        offset: u64, // optional, defaults to 0
        size: Option<crate::BufferSize>,
    },
    Draw {
        vertex_count: u32,
        instance_count: u32, // optional, defaults to 1
        first_vertex: u32,   // optional, defaults to 0
        first_instance: u32, // optional, defaults to 0
    },
    DrawIndexed {
        index_count: u32,
        instance_count: u32, // optional, defaults to 1
        first_index: u32,    // optional, defaults to 0
        base_vertex: i32,    // optional, defaults to 0
        first_instance: u32, // optional, defaults to 0
    },
    DrawIndirect {
        indirect_buffer: id::BufferId,
        indirect_offset: u64,
    },
    DrawIndexedIndirect {
        indirect_buffer: id::BufferId,
        indirect_offset: u64,
    },
}
