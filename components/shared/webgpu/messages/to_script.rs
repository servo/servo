/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! IPC messages that are sent to the ScriptThread.

use base::id::PipelineId;
use serde::{Deserialize, Serialize};
use wgpu_core::id::{
    AdapterId, BindGroupId, BindGroupLayoutId, BufferId, CommandBufferId, ComputePassEncoderId,
    ComputePipelineId, DeviceId, PipelineLayoutId, QuerySetId, RenderBundleId, RenderPassEncoderId,
    RenderPipelineId, SamplerId, ShaderModuleId, StagingBufferId, SurfaceId, TextureId,
    TextureViewId,
};

use crate::{DeviceLostReason, Error, WebGPUDevice};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum WebGPUMsg {
    FreeAdapter(AdapterId),
    FreeDevice {
        device_id: DeviceId,
        pipeline_id: PipelineId,
    },
    FreeBuffer(BufferId),
    FreePipelineLayout(PipelineLayoutId),
    FreeComputePipeline(ComputePipelineId),
    FreeRenderPipeline(RenderPipelineId),
    FreeBindGroup(BindGroupId),
    FreeBindGroupLayout(BindGroupLayoutId),
    FreeCommandBuffer(CommandBufferId),
    FreeTexture(TextureId),
    FreeTextureView(TextureViewId),
    FreeSampler(SamplerId),
    FreeSurface(SurfaceId),
    FreeShaderModule(ShaderModuleId),
    FreeRenderBundle(RenderBundleId),
    FreeStagingBuffer(StagingBufferId),
    FreeQuerySet(QuerySetId),
    FreeComputePass(ComputePassEncoderId),
    FreeRenderPass(RenderPassEncoderId),
    UncapturedError {
        device: WebGPUDevice,
        pipeline_id: PipelineId,
        error: Error,
    },
    DeviceLost {
        device: WebGPUDevice,
        pipeline_id: PipelineId,
        reason: DeviceLostReason,
        msg: String,
    },
    Exit,
}
