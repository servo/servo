/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use smallvec::SmallVec;
use webgpu::wgpu::{
    hub::IdentityManager,
    id::{
        AdapterId, BindGroupId, BindGroupLayoutId, BufferId, CommandEncoderId, ComputePipelineId,
        DeviceId, PipelineLayoutId, RenderBundleId, RenderPipelineId, SamplerId, ShaderModuleId,
        TextureId, TextureViewId,
    },
};
use webgpu::wgt::Backend;

#[derive(Debug)]
pub struct IdentityHub {
    adapters: IdentityManager,
    devices: IdentityManager,
    buffers: IdentityManager,
    bind_groups: IdentityManager,
    bind_group_layouts: IdentityManager,
    compute_pipelines: IdentityManager,
    pipeline_layouts: IdentityManager,
    shader_modules: IdentityManager,
    command_encoders: IdentityManager,
    textures: IdentityManager,
    texture_views: IdentityManager,
    samplers: IdentityManager,
    render_pipelines: IdentityManager,
    render_bundles: IdentityManager,
}

impl IdentityHub {
    fn new() -> Self {
        IdentityHub {
            adapters: IdentityManager::default(),
            devices: IdentityManager::default(),
            buffers: IdentityManager::default(),
            bind_groups: IdentityManager::default(),
            bind_group_layouts: IdentityManager::default(),
            compute_pipelines: IdentityManager::default(),
            pipeline_layouts: IdentityManager::default(),
            shader_modules: IdentityManager::default(),
            command_encoders: IdentityManager::default(),
            textures: IdentityManager::default(),
            texture_views: IdentityManager::default(),
            samplers: IdentityManager::default(),
            render_pipelines: IdentityManager::default(),
            render_bundles: IdentityManager::default(),
        }
    }
}

#[derive(Debug)]
pub struct Identities {
    surface: IdentityManager,
    #[cfg(any(target_os = "linux", target_os = "windows"))]
    vk_hub: IdentityHub,
    #[cfg(target_os = "windows")]
    dx12_hub: IdentityHub,
    #[cfg(target_os = "windows")]
    dx11_hub: IdentityHub,
    #[cfg(any(target_os = "ios", target_os = "macos"))]
    metal_hub: IdentityHub,
    dummy_hub: IdentityHub,
}

impl Identities {
    pub fn new() -> Self {
        Identities {
            surface: IdentityManager::default(),
            #[cfg(any(target_os = "linux", target_os = "windows"))]
            vk_hub: IdentityHub::new(),
            #[cfg(target_os = "windows")]
            dx12_hub: IdentityHub::new(),
            #[cfg(target_os = "windows")]
            dx11_hub: IdentityHub::new(),
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
            #[cfg(target_os = "windows")]
            Backend::Dx11 => &mut self.dx11_hub,
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
            #[cfg(target_os = "windows")]
            (&mut self.dx11_hub, Backend::Dx11),
            #[cfg(any(target_os = "ios", target_os = "macos"))]
            (&mut self.metal_hub, Backend::Metal),
            (&mut self.dummy_hub, Backend::Empty),
        ]
    }

    pub fn create_device_id(&mut self, backend: Backend) -> DeviceId {
        self.select(backend).devices.alloc(backend)
    }

    pub fn kill_device_id(&mut self, id: DeviceId) {
        self.select(id.backend()).devices.free(id);
    }

    pub fn create_adapter_ids(&mut self) -> SmallVec<[AdapterId; 4]> {
        let mut ids = SmallVec::new();
        for hubs in self.hubs() {
            ids.push(hubs.0.adapters.alloc(hubs.1));
        }
        ids
    }

    pub fn kill_adapter_id(&mut self, id: AdapterId) {
        self.select(id.backend()).adapters.free(id);
    }

    pub fn create_buffer_id(&mut self, backend: Backend) -> BufferId {
        self.select(backend).buffers.alloc(backend)
    }

    pub fn kill_buffer_id(&mut self, id: BufferId) {
        self.select(id.backend()).buffers.free(id);
    }

    pub fn create_bind_group_id(&mut self, backend: Backend) -> BindGroupId {
        self.select(backend).bind_groups.alloc(backend)
    }

    pub fn kill_bind_group_id(&mut self, id: BindGroupId) {
        self.select(id.backend()).bind_groups.free(id);
    }

    pub fn create_bind_group_layout_id(&mut self, backend: Backend) -> BindGroupLayoutId {
        self.select(backend).bind_group_layouts.alloc(backend)
    }

    pub fn kill_bind_group_layout_id(&mut self, id: BindGroupLayoutId) {
        self.select(id.backend()).bind_group_layouts.free(id);
    }

    pub fn create_compute_pipeline_id(&mut self, backend: Backend) -> ComputePipelineId {
        self.select(backend).compute_pipelines.alloc(backend)
    }

    pub fn kill_compute_pipeline_id(&mut self, id: ComputePipelineId) {
        self.select(id.backend()).compute_pipelines.free(id);
    }

    pub fn create_pipeline_layout_id(&mut self, backend: Backend) -> PipelineLayoutId {
        self.select(backend).pipeline_layouts.alloc(backend)
    }

    pub fn kill_pipeline_layout_id(&mut self, id: PipelineLayoutId) {
        self.select(id.backend()).pipeline_layouts.free(id);
    }

    pub fn create_shader_module_id(&mut self, backend: Backend) -> ShaderModuleId {
        self.select(backend).shader_modules.alloc(backend)
    }

    pub fn kill_shader_module_id(&mut self, id: ShaderModuleId) {
        self.select(id.backend()).shader_modules.free(id);
    }

    pub fn create_command_encoder_id(&mut self, backend: Backend) -> CommandEncoderId {
        self.select(backend).command_encoders.alloc(backend)
    }

    pub fn kill_command_buffer_id(&mut self, id: CommandEncoderId) {
        self.select(id.backend()).command_encoders.free(id);
    }

    pub fn create_sampler_id(&mut self, backend: Backend) -> SamplerId {
        self.select(backend).samplers.alloc(backend)
    }

    pub fn kill_sampler_id(&mut self, id: SamplerId) {
        self.select(id.backend()).samplers.free(id);
    }

    pub fn create_render_pipeline_id(&mut self, backend: Backend) -> RenderPipelineId {
        self.select(backend).render_pipelines.alloc(backend)
    }

    pub fn kill_render_pipeline_id(&mut self, id: RenderPipelineId) {
        self.select(id.backend()).render_pipelines.free(id);
    }

    pub fn create_texture_id(&mut self, backend: Backend) -> TextureId {
        self.select(backend).textures.alloc(backend)
    }

    pub fn kill_texture_id(&mut self, id: TextureId) {
        self.select(id.backend()).textures.free(id);
    }

    pub fn create_texture_view_id(&mut self, backend: Backend) -> TextureViewId {
        self.select(backend).texture_views.alloc(backend)
    }

    pub fn kill_texture_view_id(&mut self, id: TextureViewId) {
        self.select(id.backend()).texture_views.free(id);
    }

    pub fn create_render_bundle_id(&mut self, backend: Backend) -> RenderBundleId {
        self.select(backend).render_bundles.alloc(backend)
    }

    pub fn kill_render_bundle_id(&mut self, id: RenderBundleId) {
        self.select(id.backend()).render_bundles.free(id);
    }
}
