/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use webgpu_traits::{ComputePass, ComputePassId, RenderPass, RenderPassId};
use wgpu_core::id::markers::{
    Adapter, BindGroup, BindGroupLayout, Buffer, CommandEncoder, ComputePipeline, Device,
    PipelineLayout, Queue, RenderBundle, RenderPipeline, Sampler, ShaderModule, Texture,
    TextureView,
};
use wgpu_core::id::{
    AdapterId, BindGroupId, BindGroupLayoutId, BufferId, CommandEncoderId, ComputePipelineId,
    DeviceId, PipelineLayoutId, QueueId, RenderBundleId, RenderPipelineId, SamplerId,
    ShaderModuleId, TextureId, TextureViewId,
};
use wgpu_core::identity::IdentityManager;

#[derive(Debug)]
pub(crate) struct IdentityHub {
    adapters: IdentityManager<Adapter>,
    devices: IdentityManager<Device>,
    queues: IdentityManager<Queue>,
    buffers: IdentityManager<Buffer>,
    bind_groups: IdentityManager<BindGroup>,
    bind_group_layouts: IdentityManager<BindGroupLayout>,
    compute_pipelines: IdentityManager<ComputePipeline>,
    pipeline_layouts: IdentityManager<PipelineLayout>,
    shader_modules: IdentityManager<ShaderModule>,
    command_encoders: IdentityManager<CommandEncoder>,
    textures: IdentityManager<Texture>,
    texture_views: IdentityManager<TextureView>,
    samplers: IdentityManager<Sampler>,
    render_pipelines: IdentityManager<RenderPipeline>,
    render_bundles: IdentityManager<RenderBundle>,
    compute_passes: IdentityManager<ComputePass>,
    render_passes: IdentityManager<RenderPass>,
}

impl Default for IdentityHub {
    fn default() -> Self {
        IdentityHub {
            adapters: IdentityManager::new(),
            devices: IdentityManager::new(),
            queues: IdentityManager::new(),
            buffers: IdentityManager::new(),
            bind_groups: IdentityManager::new(),
            bind_group_layouts: IdentityManager::new(),
            compute_pipelines: IdentityManager::new(),
            pipeline_layouts: IdentityManager::new(),
            shader_modules: IdentityManager::new(),
            command_encoders: IdentityManager::new(),
            textures: IdentityManager::new(),
            texture_views: IdentityManager::new(),
            samplers: IdentityManager::new(),
            render_pipelines: IdentityManager::new(),
            render_bundles: IdentityManager::new(),
            compute_passes: IdentityManager::new(),
            render_passes: IdentityManager::new(),
        }
    }
}

impl IdentityHub {
    pub(crate) fn create_device_id(&self) -> DeviceId {
        self.devices.process()
    }

    pub(crate) fn free_device_id(&self, id: DeviceId) {
        self.devices.free(id);
    }

    pub(crate) fn create_queue_id(&self) -> QueueId {
        self.queues.process()
    }

    pub(crate) fn free_queue_id(&self, id: QueueId) {
        self.queues.free(id);
    }

    pub(crate) fn create_adapter_id(&self) -> AdapterId {
        self.adapters.process()
    }

    pub(crate) fn free_adapter_id(&self, id: AdapterId) {
        self.adapters.free(id);
    }

    pub(crate) fn create_buffer_id(&self) -> BufferId {
        self.buffers.process()
    }

    pub(crate) fn free_buffer_id(&self, id: BufferId) {
        self.buffers.free(id);
    }

    pub(crate) fn create_bind_group_id(&self) -> BindGroupId {
        self.bind_groups.process()
    }

    pub(crate) fn free_bind_group_id(&self, id: BindGroupId) {
        self.bind_groups.free(id);
    }

    pub(crate) fn create_bind_group_layout_id(&self) -> BindGroupLayoutId {
        self.bind_group_layouts.process()
    }

    pub(crate) fn free_bind_group_layout_id(&self, id: BindGroupLayoutId) {
        self.bind_group_layouts.free(id);
    }

    pub(crate) fn create_compute_pipeline_id(&self) -> ComputePipelineId {
        self.compute_pipelines.process()
    }

    pub(crate) fn free_compute_pipeline_id(&self, id: ComputePipelineId) {
        self.compute_pipelines.free(id);
    }

    pub(crate) fn create_pipeline_layout_id(&self) -> PipelineLayoutId {
        self.pipeline_layouts.process()
    }

    pub(crate) fn free_pipeline_layout_id(&self, id: PipelineLayoutId) {
        self.pipeline_layouts.free(id);
    }

    pub(crate) fn create_shader_module_id(&self) -> ShaderModuleId {
        self.shader_modules.process()
    }

    pub(crate) fn free_shader_module_id(&self, id: ShaderModuleId) {
        self.shader_modules.free(id);
    }

    pub(crate) fn create_command_encoder_id(&self) -> CommandEncoderId {
        self.command_encoders.process()
    }

    pub(crate) fn free_command_buffer_id(&self, id: CommandEncoderId) {
        self.command_encoders.free(id);
    }

    pub(crate) fn create_sampler_id(&self) -> SamplerId {
        self.samplers.process()
    }

    pub(crate) fn free_sampler_id(&self, id: SamplerId) {
        self.samplers.free(id);
    }

    pub(crate) fn create_render_pipeline_id(&self) -> RenderPipelineId {
        self.render_pipelines.process()
    }

    pub(crate) fn free_render_pipeline_id(&self, id: RenderPipelineId) {
        self.render_pipelines.free(id);
    }

    pub(crate) fn create_texture_id(&self) -> TextureId {
        self.textures.process()
    }

    pub(crate) fn free_texture_id(&self, id: TextureId) {
        self.textures.free(id);
    }

    pub(crate) fn create_texture_view_id(&self) -> TextureViewId {
        self.texture_views.process()
    }

    pub(crate) fn free_texture_view_id(&self, id: TextureViewId) {
        self.texture_views.free(id);
    }

    pub(crate) fn create_render_bundle_id(&self) -> RenderBundleId {
        self.render_bundles.process()
    }

    pub(crate) fn free_render_bundle_id(&self, id: RenderBundleId) {
        self.render_bundles.free(id);
    }

    pub(crate) fn create_compute_pass_id(&self) -> ComputePassId {
        self.compute_passes.process()
    }

    pub(crate) fn free_compute_pass_id(&self, id: ComputePassId) {
        self.compute_passes.free(id);
    }

    pub(crate) fn create_render_pass_id(&self) -> RenderPassId {
        self.render_passes.process()
    }

    pub(crate) fn free_render_pass_id(&self, id: RenderPassId) {
        self.render_passes.free(id);
    }
}
