/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This crate defines the Rust implementation of WebXR for various devices.

#[cfg(feature = "glwindow")]
pub mod glwindow;

#[cfg(feature = "headless")]
pub mod headless;

#[cfg(feature = "openxr-api")]
pub mod openxr;

pub mod surfman_layer_manager;
pub use surfman_layer_manager::SurfmanGL;
pub use surfman_layer_manager::SurfmanLayerManager;
pub type MainThreadRegistry = webxr_api::MainThreadRegistry<surfman_layer_manager::SurfmanGL>;
pub type Discovery = Box<dyn webxr_api::DiscoveryAPI<SurfmanGL>>;

pub(crate) mod gl_utils;
