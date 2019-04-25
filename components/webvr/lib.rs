/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

#[macro_use]
extern crate log;

mod webvr_thread;
pub use crate::webvr_thread::{WebVRCompositorHandler, WebVRThread};
pub use rust_webvr::api::VRExternalShmemPtr;
pub use rust_webvr::VRMainThreadHeartbeat;
pub use rust_webvr::VRServiceManager;
pub use rust_webvr_api::VRService;
