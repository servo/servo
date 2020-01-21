/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use smallvec::SmallVec;
use webgpu::wgpu::{
    hub::IdentityManager,
    id::{AdapterId, BindGroupLayoutId, BufferId, DeviceId, PipelineLayoutId},
    Backend,
};

#[derive(Debug)]
pub struct IdentityHub {
    adapters: IdentityManager,
    devices: IdentityManager,
    buffers: IdentityManager,
    bind_group_layouts: IdentityManager,
    pipeline_layouts: IdentityManager,
    backend: Backend,
}

impl IdentityHub {
    fn new(backend: Backend) -> Self {
        IdentityHub {
            adapters: IdentityManager::default(),
            devices: IdentityManager::default(),
            buffers: IdentityManager::default(),
            bind_group_layouts: IdentityManager::default(),
            pipeline_layouts: IdentityManager::default(),
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

    fn create_bind_group_layout_id(&mut self) -> BindGroupLayoutId {
        self.bind_group_layouts.alloc(self.backend)
    }

    fn create_pipeline_layout_id(&mut self) -> PipelineLayoutId {
        self.pipeline_layouts.alloc(self.backend)
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
        match backend {
            #[cfg(any(target_os = "linux", target_os = "windows"))]
            Backend::Vulkan => self.vk_hub.create_device_id(),
            #[cfg(target_os = "windows")]
            Backend::Dx12 => self.dx12_hub.create_device_id(),
            #[cfg(target_os = "windows")]
            Backend::Dx11 => self.dx11_hub.create_device_id(),
            #[cfg(any(target_os = "ios", target_os = "macos"))]
            Backend::Metal => self.metal_hub.create_device_id(),
            _ => self.dummy_hub.create_device_id(),
        }
    }

    pub fn create_adapter_ids(&mut self) -> SmallVec<[AdapterId; 4]> {
        let mut ids = SmallVec::new();
        for hub in self.hubs() {
            ids.push(hub.create_adapter_id())
        }
        ids
    }

    pub fn create_buffer_id(&mut self, backend: Backend) -> BufferId {
        match backend {
            #[cfg(any(target_os = "linux", target_os = "windows"))]
            Backend::Vulkan => self.vk_hub.create_buffer_id(),
            #[cfg(target_os = "windows")]
            Backend::Dx12 => self.dx12_hub.create_buffer_id(),
            #[cfg(target_os = "windows")]
            Backend::Dx11 => self.dx11_hub.create_buffer_id(),
            #[cfg(any(target_os = "ios", target_os = "macos"))]
            Backend::Metal => self.metal_hub.create_buffer_id(),
            _ => self.dummy_hub.create_buffer_id(),
        }
    }

    pub fn create_bind_group_layout_id(&mut self, backend: Backend) -> BindGroupLayoutId {
        match backend {
            #[cfg(any(target_os = "linux", target_os = "windows"))]
            Backend::Vulkan => self.vk_hub.create_bind_group_layout_id(),
            #[cfg(target_os = "windows")]
            Backend::Dx12 => self.dx12_hub.create_bind_group_layout_id(),
            #[cfg(target_os = "windows")]
            Backend::Dx11 => self.dx11_hub.create_bind_group_layout_id(),
            #[cfg(any(target_os = "ios", target_os = "macos"))]
            Backend::Metal => self.metal_hub.create_bind_group_layout_id(),
            _ => self.dummy_hub.create_bind_group_layout_id(),
        }
    }

    pub fn create_pipeline_layout_id(&mut self, backend: Backend) -> PipelineLayoutId {
        match backend {
            #[cfg(any(target_os = "linux", target_os = "windows"))]
            Backend::Vulkan => self.vk_hub.create_pipeline_layout_id(),
            #[cfg(target_os = "windows")]
            Backend::Dx12 => self.dx12_hub.create_pipeline_layout_id(),
            #[cfg(target_os = "windows")]
            Backend::Dx11 => self.dx11_hub.create_pipeline_layout_id(),
            #[cfg(any(target_os = "ios", target_os = "macos"))]
            Backend::Metal => self.metal_hub.create_pipeline_layout_id(),
            _ => self.dummy_hub.create_pipeline_layout_id(),
        }
    }
}
