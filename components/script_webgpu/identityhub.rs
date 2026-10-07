/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use parking_lot::Mutex;
use webgpu_traits::IdentityManager;
use webgpu_traits::id::*;
use webgpu_traits::markers::*;

#[derive(Debug)]
pub struct IdentityHub {
    adapters: Mutex<IdentityManager<Adapter>>,
    devices: Mutex<IdentityManager<Device>>,
    queues: Mutex<IdentityManager<Queue>>,
    buffers: Mutex<IdentityManager<Buffer>>,
    bind_groups: Mutex<IdentityManager<BindGroup>>,
    bind_group_layouts: Mutex<IdentityManager<BindGroupLayout>>,
    compute_pipelines: Mutex<IdentityManager<ComputePipeline>>,
    pipeline_layouts: Mutex<IdentityManager<PipelineLayout>>,
    shader_modules: Mutex<IdentityManager<ShaderModule>>,
    command_encoders: Mutex<IdentityManager<CommandEncoder>>,
    command_buffers: Mutex<IdentityManager<CommandBuffer>>,
    textures: Mutex<IdentityManager<Texture>>,
    texture_views: Mutex<IdentityManager<TextureView>>,
    samplers: Mutex<IdentityManager<Sampler>>,
    render_pipelines: Mutex<IdentityManager<RenderPipeline>>,
    render_bundles: Mutex<IdentityManager<RenderBundle>>,
    compute_passes: Mutex<IdentityManager<ComputePassEncoder>>,
    render_passes: Mutex<IdentityManager<RenderPassEncoder>>,
    query_sets: Mutex<IdentityManager<QuerySet>>,
    external_textures: Mutex<IdentityManager<ExternalTexture>>,
    render_bundle_encoders: Mutex<IdentityManager<RenderBundleEncoder>>,
}

impl Default for IdentityHub {
    fn default() -> Self {
        IdentityHub {
            adapters: Mutex::new(IdentityManager::new()),
            devices: Mutex::new(IdentityManager::new()),
            queues: Mutex::new(IdentityManager::new()),
            buffers: Mutex::new(IdentityManager::new()),
            bind_groups: Mutex::new(IdentityManager::new()),
            bind_group_layouts: Mutex::new(IdentityManager::new()),
            compute_pipelines: Mutex::new(IdentityManager::new()),
            pipeline_layouts: Mutex::new(IdentityManager::new()),
            shader_modules: Mutex::new(IdentityManager::new()),
            command_encoders: Mutex::new(IdentityManager::new()),
            command_buffers: Mutex::new(IdentityManager::new()),
            textures: Mutex::new(IdentityManager::new()),
            texture_views: Mutex::new(IdentityManager::new()),
            samplers: Mutex::new(IdentityManager::new()),
            render_pipelines: Mutex::new(IdentityManager::new()),
            render_bundles: Mutex::new(IdentityManager::new()),
            compute_passes: Mutex::new(IdentityManager::new()),
            render_passes: Mutex::new(IdentityManager::new()),
            query_sets: Mutex::new(IdentityManager::new()),
            external_textures: Mutex::new(IdentityManager::new()),
            render_bundle_encoders: Mutex::new(IdentityManager::new()),
        }
    }
}

impl IdentityHub {
    pub fn create_device_id(&self) -> DeviceId {
        self.devices.lock().process()
    }

    pub fn free_device_id(&self, id: DeviceId) {
        self.devices.lock().free(id);
    }

    pub fn create_queue_id(&self) -> QueueId {
        self.queues.lock().process()
    }

    #[expect(unused)]
    fn free_queue_id(&self, id: QueueId) {
        self.queues.lock().free(id);
    }

    pub fn create_adapter_id(&self) -> AdapterId {
        self.adapters.lock().process()
    }

    pub fn free_adapter_id(&self, id: AdapterId) {
        self.adapters.lock().free(id);
    }

    pub fn create_buffer_id(&self) -> BufferId {
        self.buffers.lock().process()
    }

    pub fn free_buffer_id(&self, id: BufferId) {
        self.buffers.lock().free(id);
    }

    pub fn create_bind_group_id(&self) -> BindGroupId {
        self.bind_groups.lock().process()
    }

    pub fn free_bind_group_id(&self, id: BindGroupId) {
        self.bind_groups.lock().free(id);
    }

    pub fn create_bind_group_layout_id(&self) -> BindGroupLayoutId {
        self.bind_group_layouts.lock().process()
    }

    pub fn free_bind_group_layout_id(&self, id: BindGroupLayoutId) {
        self.bind_group_layouts.lock().free(id);
    }

    pub fn create_compute_pipeline_id(&self) -> ComputePipelineId {
        self.compute_pipelines.lock().process()
    }

    pub fn free_compute_pipeline_id(&self, id: ComputePipelineId) {
        self.compute_pipelines.lock().free(id);
    }

    pub fn create_pipeline_layout_id(&self) -> PipelineLayoutId {
        self.pipeline_layouts.lock().process()
    }

    pub fn free_pipeline_layout_id(&self, id: PipelineLayoutId) {
        self.pipeline_layouts.lock().free(id);
    }

    pub fn create_shader_module_id(&self) -> ShaderModuleId {
        self.shader_modules.lock().process()
    }

    pub fn free_shader_module_id(&self, id: ShaderModuleId) {
        self.shader_modules.lock().free(id);
    }

    pub fn create_command_encoder_id(&self) -> CommandEncoderId {
        self.command_encoders.lock().process()
    }

    #[expect(unused)]
    fn free_command_encoder_id(&self, id: CommandEncoderId) {
        self.command_encoders.lock().free(id);
    }

    pub fn create_command_buffer_id(&self) -> CommandBufferId {
        self.command_buffers.lock().process()
    }

    pub fn free_command_buffer_id(&self, id: CommandBufferId) {
        self.command_buffers.lock().free(id);
    }

    pub fn create_sampler_id(&self) -> SamplerId {
        self.samplers.lock().process()
    }

    pub fn free_sampler_id(&self, id: SamplerId) {
        self.samplers.lock().free(id);
    }

    pub fn create_render_pipeline_id(&self) -> RenderPipelineId {
        self.render_pipelines.lock().process()
    }

    pub fn free_render_pipeline_id(&self, id: RenderPipelineId) {
        self.render_pipelines.lock().free(id);
    }

    pub fn create_texture_id(&self) -> TextureId {
        self.textures.lock().process()
    }

    pub fn free_texture_id(&self, id: TextureId) {
        self.textures.lock().free(id);
    }

    pub fn create_texture_view_id(&self) -> TextureViewId {
        self.texture_views.lock().process()
    }

    pub fn free_texture_view_id(&self, id: TextureViewId) {
        self.texture_views.lock().free(id);
    }

    pub fn create_render_bundle_id(&self) -> RenderBundleId {
        self.render_bundles.lock().process()
    }

    pub fn free_render_bundle_id(&self, id: RenderBundleId) {
        self.render_bundles.lock().free(id);
    }

    pub fn create_compute_pass_id(&self) -> ComputePassEncoderId {
        self.compute_passes.lock().process()
    }

    pub fn free_compute_pass_id(&self, id: ComputePassEncoderId) {
        self.compute_passes.lock().free(id);
    }

    pub fn create_render_pass_id(&self) -> RenderPassEncoderId {
        self.render_passes.lock().process()
    }

    pub fn free_render_pass_id(&self, id: RenderPassEncoderId) {
        self.render_passes.lock().free(id);
    }

    pub fn create_query_set_id(&self) -> QuerySetId {
        self.query_sets.lock().process()
    }

    #[expect(unused)]
    fn free_query_set_id(&self, id: QuerySetId) {
        self.query_sets.lock().free(id);
    }

    pub fn create_external_texture_id(&self) -> ExternalTextureId {
        self.external_textures.lock().process()
    }

    #[expect(unused)]
    fn free_external_texture_id(&self, id: ExternalTextureId) {
        self.external_textures.lock().free(id);
    }

    pub fn create_render_bundle_encoder_id(&self) -> RenderBundleEncoderId {
        self.render_bundle_encoders.lock().process()
    }

    #[expect(unused)]
    fn free_render_bundle_encoder_id(&self, id: RenderBundleEncoderId) {
        self.render_bundle_encoders.lock().free(id);
    }
}
