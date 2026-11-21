/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This crate defines the Rust implementation of WebXR for various devices.
//!
//! Supported native OpenXR backends (behind the `openxr-api` feature):
//! - **Windows**: D3D11 (`XR_KHR_d3d11_enable`) bridged to Surfman/ANGLE.
//! - **Linux (non-OHOS)**: Vulkan (`XR_KHR_vulkan_enable2`) using the bundled
//!   static Khronos OpenXR loader and DMA-BUF `EGLImage` interop with Surfman's
//!   EGL/OpenGL contexts (compatible with SteamVR, Monado, and WiVRn).

#[cfg(feature = "glwindow")]
pub mod glwindow;

#[cfg(feature = "headless")]
pub mod headless;

#[cfg(feature = "openxr-api")]
pub mod openxr;

pub mod inline;

pub mod surfman_layer_manager;
pub use surfman_layer_manager::{SurfmanGL, SurfmanLayerManager};
pub type MainThreadRegistry = webxr_api::MainThreadRegistry<surfman_layer_manager::SurfmanGL>;
pub type Discovery = Box<dyn webxr_api::DiscoveryAPI<SurfmanGL>>;

pub(crate) mod gl_utils;

pub trait WebXrRegistry {
    /// Register services with a WebXR Registry.
    fn register(&self, _registry: &mut MainThreadRegistry) {}
}
