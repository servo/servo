/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! IPC massages that are send to script thread (global scope).

use base::id::PipelineId;
use serde::{Deserialize, Serialize};

use crate::gpu_error::Error;
use crate::identity::WebGPUDevice;
use crate::wgc::id::{
    AdapterId, BindGroupId, BindGroupLayoutId, BufferId, CommandBufferId, ComputePipelineId,
    DeviceId, PipelineLayoutId, QuerySetId, RenderBundleId, RenderPipelineId, SamplerId,
    ShaderModuleId, StagingBufferId, SurfaceId, TextureId, TextureViewId,
};
use crate::wgt;

// Workaround until https://github.com/gfx-rs/wgpu/pull/5732
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum DeviceLostReason {
    /// Triggered by driver
    Unknown = 0,
    /// After Device::destroy
    Destroyed = 1,
    /// After Device::drop
    ///
    /// WebGPU does not invoke the device lost callback when the device is
    /// dropped to prevent garbage collection from being observable. In wgpu,
    /// we invoke the callback on drop to help with managing memory owned by
    /// the callback.
    Dropped = 2,
    /// After replacing the device_lost_callback
    ///
    /// WebGPU does not have a concept of a device lost callback, but wgpu
    /// does. wgpu guarantees that any supplied callback will be invoked
    /// exactly once before it is dropped, which helps with managing the
    /// memory owned by the callback.
    ReplacedCallback = 3,
    /// When setting the callback, but the device is already invalid
    ///
    /// As above, when the callback is provided, wgpu guarantees that it
    /// will eventually be called. If the device is already invalid, wgpu
    /// will call the callback immediately, with this reason.
    DeviceInvalid = 4,
}

impl DeviceLostReason {
    pub fn new(reason: wgt::DeviceLostReason) -> Self {
        match reason {
            wgt::DeviceLostReason::Unknown => DeviceLostReason::Unknown,
            wgt::DeviceLostReason::Destroyed => DeviceLostReason::Destroyed,
            wgt::DeviceLostReason::Dropped => DeviceLostReason::Dropped,
            wgt::DeviceLostReason::ReplacedCallback => DeviceLostReason::ReplacedCallback,
            wgt::DeviceLostReason::DeviceInvalid => DeviceLostReason::DeviceInvalid,
        }
    }

    pub fn wgt(&self) -> wgt::DeviceLostReason {
        match self {
            DeviceLostReason::Unknown => wgt::DeviceLostReason::Unknown,
            DeviceLostReason::Destroyed => wgt::DeviceLostReason::Destroyed,
            DeviceLostReason::Dropped => wgt::DeviceLostReason::Dropped,
            DeviceLostReason::ReplacedCallback => wgt::DeviceLostReason::ReplacedCallback,
            DeviceLostReason::DeviceInvalid => wgt::DeviceLostReason::DeviceInvalid,
        }
    }
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
