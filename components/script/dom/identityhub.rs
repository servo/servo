/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use smallvec::SmallVec;
use webgpu::wgpu::id::markers::{
    Adapter, BindGroup, BindGroupLayout, Buffer, CommandEncoder, ComputePipeline, Device,
    PipelineLayout, RenderBundle, RenderPipeline, Sampler, ShaderModule, Texture, TextureView,
};
use webgpu::wgpu::id::{
    AdapterId, BindGroupId, BindGroupLayoutId, BufferId, CommandEncoderId, ComputePipelineId,
    DeviceId, PipelineLayoutId, RenderBundleId, RenderPipelineId, SamplerId, ShaderModuleId,
    TextureId, TextureViewId,
};
use webgpu::wgpu::identity::IdentityManager;
use webgpu::wgt::Backend;

#[derive(Debug)]
pub struct IdentityHub {
    adapters: IdentityManager<Adapter>,
    devices: IdentityManager<Device>,
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
}

impl IdentityHub {
    fn new() -> Self {
        IdentityHub {
            adapters: IdentityManager::new(),
            devices: IdentityManager::new(),
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
        }
    }
}

#[derive(Debug)]
pub struct Identities {
    #[cfg(any(target_os = "linux", target_os = "windows"))]
    vk_hub: IdentityHub,
    #[cfg(target_os = "windows")]
    dx12_hub: IdentityHub,
    #[cfg(any(target_os = "ios", target_os = "macos"))]
    metal_hub: IdentityHub,
    dummy_hub: IdentityHub,
}

impl Identities {
    pub fn new() -> Self {
        Identities {
            #[cfg(any(target_os = "linux", target_os = "windows"))]
            vk_hub: IdentityHub::new(),
            #[cfg(target_os = "windows")]
            dx12_hub: IdentityHub::new(),
            #[cfg(any(target_os = "ios", target_os = "macos"))]
            metal_hub: IdentityHub::new(),
            dummy_hub: IdentityHub::new(),
        }
    }

    fn select(&mut self, backend: Backend) -> &mut IdentityHub {
        match backend {
            #[cfg(any(target_os = "linux", target_os = "windows"))]
            Backend::Vulkan => &mut self.vk_hub,
            #[cfg(target_os = "windows")]
            Backend::Dx12 => &mut self.dx12_hub,
            #[cfg(any(target_os = "ios", target_os = "macos"))]
            Backend::Metal => &mut self.metal_hub,
            _ => &mut self.dummy_hub,
        }
    }

    fn hubs(&mut self) -> Vec<(&mut IdentityHub, Backend)> {
        vec![
            #[cfg(any(target_os = "linux", target_os = "windows"))]
            (&mut self.vk_hub, Backend::Vulkan),
            #[cfg(target_os = "windows")]
            (&mut self.dx12_hub, Backend::Dx12),
            #[cfg(any(target_os = "ios", target_os = "macos"))]
            (&mut self.metal_hub, Backend::Metal),
            (&mut self.dummy_hub, Backend::Empty),
        ]
    }

    pub fn create_device_id(&mut self, backend: Backend) -> DeviceId {
        self.select(backend).devices.process(backend)
    }

    pub fn kill_device_id(&mut self, id: DeviceId) {
        self.select(id.backend()).devices.free(id);
    }

    pub fn create_adapter_ids(&mut self) -> SmallVec<[AdapterId; 4]> {
        let mut ids = SmallVec::new();
        for hubs in self.hubs() {
            ids.push(hubs.0.adapters.process(hubs.1));
        }
        ids
    }

    pub fn kill_adapter_id(&mut self, id: AdapterId) {
        self.select(id.backend()).adapters.free(id);
    }

    pub fn create_buffer_id(&mut self, backend: Backend) -> BufferId {
        self.select(backend).buffers.process(backend)
    }

    pub fn kill_buffer_id(&mut self, id: BufferId) {
        self.select(id.backend()).buffers.free(id);
    }

    pub fn create_bind_group_id(&mut self, backend: Backend) -> BindGroupId {
        self.select(backend).bind_groups.process(backend)
    }

    pub fn kill_bind_group_id(&mut self, id: BindGroupId) {
        self.select(id.backend()).bind_groups.free(id);
    }

    pub fn create_bind_group_layout_id(&mut self, backend: Backend) -> BindGroupLayoutId {
        self.select(backend).bind_group_layouts.process(backend)
    }

    pub fn kill_bind_group_layout_id(&mut self, id: BindGroupLayoutId) {
        self.select(id.backend()).bind_group_layouts.free(id);
    }

    pub fn create_compute_pipeline_id(&mut self, backend: Backend) -> ComputePipelineId {
        self.select(backend).compute_pipelines.process(backend)
    }

    pub fn kill_compute_pipeline_id(&mut self, id: ComputePipelineId) {
        self.select(id.backend()).compute_pipelines.free(id);
    }

    pub fn create_pipeline_layout_id(&mut self, backend: Backend) -> PipelineLayoutId {
        self.select(backend).pipeline_layouts.process(backend)
    }

    pub fn kill_pipeline_layout_id(&mut self, id: PipelineLayoutId) {
        self.select(id.backend()).pipeline_layouts.free(id);
    }

    pub fn create_shader_module_id(&mut self, backend: Backend) -> ShaderModuleId {
        self.select(backend).shader_modules.process(backend)
    }

    pub fn kill_shader_module_id(&mut self, id: ShaderModuleId) {
        self.select(id.backend()).shader_modules.free(id);
    }

    pub fn create_command_encoder_id(&mut self, backend: Backend) -> CommandEncoderId {
        self.select(backend).command_encoders.process(backend)
    }

    pub fn kill_command_buffer_id(&mut self, id: CommandEncoderId) {
        self.select(id.backend()).command_encoders.free(id);
    }

    pub fn create_sampler_id(&mut self, backend: Backend) -> SamplerId {
        self.select(backend).samplers.process(backend)
    }

    pub fn kill_sampler_id(&mut self, id: SamplerId) {
        self.select(id.backend()).samplers.free(id);
    }

    pub fn create_render_pipeline_id(&mut self, backend: Backend) -> RenderPipelineId {
        self.select(backend).render_pipelines.process(backend)
    }

    pub fn kill_render_pipeline_id(&mut self, id: RenderPipelineId) {
        self.select(id.backend()).render_pipelines.free(id);
    }

    pub fn create_texture_id(&mut self, backend: Backend) -> TextureId {
        self.select(backend).textures.process(backend)
    }

    pub fn kill_texture_id(&mut self, id: TextureId) {
        self.select(id.backend()).textures.free(id);
    }

    pub fn create_texture_view_id(&mut self, backend: Backend) -> TextureViewId {
        self.select(backend).texture_views.process(backend)
    }

    pub fn kill_texture_view_id(&mut self, id: TextureViewId) {
        self.select(id.backend()).texture_views.free(id);
    }

    pub fn create_render_bundle_id(&mut self, backend: Backend) -> RenderBundleId {
        self.select(backend).render_bundles.process(backend)
    }

    pub fn kill_render_bundle_id(&mut self, id: RenderBundleId) {
        self.select(id.backend()).render_bundles.free(id);
    }
}

impl Default for Identities {
    fn default() -> Self {
        Self::new()
    }
}
