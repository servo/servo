/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! IPC messages that are sent to WebGPU DOM objects.

use wgpu_core::instance::RequestDeviceError;
use wgpu_types::DeviceDescriptor;

use crate::{WebGPUDevice, WebGPUQueue};

pub type WebGPUDeviceResponse = (
    WebGPUDevice,
    WebGPUQueue,
    Result<DeviceDescriptor<Option<String>>, RequestDeviceError>,
);
