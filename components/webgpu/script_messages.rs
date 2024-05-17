/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! IPC massages that are send to script thread (global scope).

use base::id::PipelineId;
use serde::{Deserialize, Serialize};

use crate::identity::WebGPUDevice;
use crate::wgc::id::{
    AdapterId, BindGroupId, BindGroupLayoutId, BufferId, CommandBufferId, ComputePipelineId,
    DeviceId, PipelineLayoutId, QuerySetId, RenderBundleId, RenderPipelineId, SamplerId,
    ShaderModuleId, StagingBufferId, SurfaceId, TextureId, TextureViewId,
};
use crate::ErrorScopeId;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum WebGPUOpResult {
    ValidationError(String),
    OutOfMemoryError,
    Success,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum WebGPUMsg {
    FreeAdapter(AdapterId),
    FreeDevice(DeviceId),
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
    WebGPUOpResult {
        device: WebGPUDevice,
        scope_id: Option<ErrorScopeId>,
        pipeline_id: PipelineId,
        result: WebGPUOpResult,
    },
    CleanDevice {
        device: WebGPUDevice,
        pipeline_id: PipelineId,
    },
    Exit,
}
