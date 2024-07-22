/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use serde::{Deserialize, Serialize};

pub use crate::wgc::id::markers::{
    ComputePassEncoder as ComputePass, RenderPassEncoder as RenderPass,
};
use crate::wgc::id::{
    AdapterId, BindGroupId, BindGroupLayoutId, BufferId, CommandBufferId, CommandEncoderId,
    ComputePipelineId, DeviceId, PipelineLayoutId, QueueId, RenderBundleId, RenderPipelineId,
    SamplerId, ShaderModuleId, SurfaceId, TextureId, TextureViewId,
};
pub use crate::wgc::id::{
    ComputePassEncoderId as ComputePassId, RenderPassEncoderId as RenderPassId,
};

macro_rules! webgpu_resource {
    ($name:ident, $id:ty) => {
        #[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
        pub struct $name(pub $id);

        impl MallocSizeOf for $name {
            fn size_of(&self, _ops: &mut MallocSizeOfOps) -> usize {
                0
            }
        }

        impl Eq for $name {}
    };
}

webgpu_resource!(WebGPUAdapter, AdapterId);
webgpu_resource!(WebGPUBindGroup, BindGroupId);
webgpu_resource!(WebGPUBindGroupLayout, BindGroupLayoutId);
webgpu_resource!(WebGPUBuffer, BufferId);
webgpu_resource!(WebGPUCommandBuffer, CommandBufferId);
webgpu_resource!(WebGPUCommandEncoder, CommandEncoderId);
webgpu_resource!(WebGPUComputePipeline, ComputePipelineId);
webgpu_resource!(WebGPUDevice, DeviceId);
webgpu_resource!(WebGPUPipelineLayout, PipelineLayoutId);
webgpu_resource!(WebGPUQueue, QueueId);
webgpu_resource!(WebGPURenderBundle, RenderBundleId);
webgpu_resource!(WebGPURenderPipeline, RenderPipelineId);
webgpu_resource!(WebGPUSampler, SamplerId);
webgpu_resource!(WebGPUShaderModule, ShaderModuleId);
webgpu_resource!(WebGPUSurface, SurfaceId);
webgpu_resource!(WebGPUTexture, TextureId);
webgpu_resource!(WebGPUTextureView, TextureViewId);
webgpu_resource!(WebGPUComputePass, ComputePassId);
webgpu_resource!(WebGPURenderPass, RenderPassId);
