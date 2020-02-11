/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use smallvec::SmallVec;
use webgpu::wgpu::{
    hub::IdentityManager,
    id::{
        AdapterId, BindGroupId, BindGroupLayoutId, BufferId, ComputePipelineId, DeviceId,
        PipelineLayoutId, ShaderModuleId,
    },
    Backend,
};

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
    backend: Backend,
}

impl IdentityHub {
    fn new(backend: Backend) -> Self {
        IdentityHub {
            adapters: IdentityManager::default(),
            devices: IdentityManager::default(),
            buffers: IdentityManager::default(),
            bind_groups: IdentityManager::default(),
            bind_group_layouts: IdentityManager::default(),
            compute_pipelines: IdentityManager::default(),
            pipeline_layouts: IdentityManager::default(),
            shader_modules: IdentityManager::default(),
            backend,
        }
    }

    fn create_adapter_id(&mut self) -> AdapterId {
        self.adapters.alloc(self.backend)
    }

    fn create_device_id(&mut self) -> DeviceId {
        self.devices.alloc(self.backend)
    }

    fn create_buffer_id(&mut self) -> BufferId {
        self.buffers.alloc(self.backend)
    }

    fn create_bind_group_id(&mut self) -> BindGroupId {
        self.bind_groups.alloc(self.backend)
    }

    fn create_bind_group_layout_id(&mut self) -> BindGroupLayoutId {
        self.bind_group_layouts.alloc(self.backend)
    }

    fn create_compute_pipeline_id(&mut self) -> ComputePipelineId {
        self.compute_pipelines.alloc(self.backend)
    }

    fn create_pipeline_layout_id(&mut self) -> PipelineLayoutId {
        self.pipeline_layouts.alloc(self.backend)
    }

    fn create_shader_module_id(&mut self) -> ShaderModuleId {
        self.shader_modules.alloc(self.backend)
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
            vk_hub: IdentityHub::new(Backend::Vulkan),
            #[cfg(target_os = "windows")]
            dx12_hub: IdentityHub::new(Backend::Dx12),
            #[cfg(target_os = "windows")]
            dx11_hub: IdentityHub::new(Backend::Dx11),
            #[cfg(any(target_os = "ios", target_os = "macos"))]
            metal_hub: IdentityHub::new(Backend::Metal),
            dummy_hub: IdentityHub::new(Backend::Empty),
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

    fn hubs(&mut self) -> Vec<&mut IdentityHub> {
        vec![
            #[cfg(any(target_os = "linux", target_os = "windows"))]
            &mut self.vk_hub,
            #[cfg(target_os = "windows")]
            &mut self.dx12_hub,
            #[cfg(target_os = "windows")]
            &mut self.dx11_hub,
            #[cfg(any(target_os = "ios", target_os = "macos"))]
            &mut self.metal_hub,
            &mut self.dummy_hub,
        ]
    }

    pub fn create_device_id(&mut self, backend: Backend) -> DeviceId {
        self.select(backend).create_device_id()
    }

    pub fn create_adapter_ids(&mut self) -> SmallVec<[AdapterId; 4]> {
        let mut ids = SmallVec::new();
        for hub in self.hubs() {
            ids.push(hub.create_adapter_id())
        }
        ids
    }

    pub fn create_buffer_id(&mut self, backend: Backend) -> BufferId {
        self.select(backend).create_buffer_id()
    }

    pub fn create_bind_group_id(&mut self, backend: Backend) -> BindGroupId {
        self.select(backend).create_bind_group_id()
    }

    pub fn create_bind_group_layout_id(&mut self, backend: Backend) -> BindGroupLayoutId {
        self.select(backend).create_bind_group_layout_id()
    }

    pub fn create_compute_pipeline_id(&mut self, backend: Backend) -> ComputePipelineId {
        self.select(backend).create_compute_pipeline_id()
    }

    pub fn create_pipeline_layout_id(&mut self, backend: Backend) -> PipelineLayoutId {
        self.select(backend).create_pipeline_layout_id()
    }

    pub fn create_shader_module_id(&mut self, backend: Backend) -> ShaderModuleId {
        self.select(backend).create_shader_module_id()
    }
}
