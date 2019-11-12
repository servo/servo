/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use webgpu::wgpu::{AdapterId, Backend, DeviceId, IdentityManager, SurfaceId};

#[derive(Debug)]
pub struct IdentityHub {
    adapters: IdentityManager<AdapterId>,
    devices: IdentityManager<DeviceId>,
}

impl IdentityHub {
    fn new(backend: Backend) -> Self {
        IdentityHub {
            adapters: IdentityManager::new(backend),
            devices: IdentityManager::new(backend),
        }
    }
}

#[derive(Debug)]
pub struct Identities {
    surface: IdentityManager<SurfaceId>,
    hub: IdentityHub,
}

impl Identities {
    pub fn new() -> Self {
        let hub = if cfg!(any(target_os = "linux", target_os = "windows")) {
            IdentityHub::new(Backend::Vulkan)
        } else if cfg!(any(target_os = "ios", target_os = "macos")) {
            IdentityHub::new(Backend::Metal)
        } else {
            IdentityHub::new(Backend::Empty)
        };

        Identities {
            surface: IdentityManager::new(Backend::Empty),
            hub,
        }
    }

    pub fn create_adapter_id(&mut self) -> AdapterId {
        self.hub.adapters.alloc()
    }

    pub fn create_device_id(&mut self) -> DeviceId {
        self.hub.devices.alloc()
    }
}
