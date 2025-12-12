/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub mod error;
pub mod ids;
pub mod messages;
pub mod render_commands;

use std::ops::Range;

use base::generic_channel::GenericSharedMemory;
use ipc_channel::ipc::IpcSender;
use serde::{Deserialize, Serialize};
use webrender_api::euclid::default::Size2D;
use webrender_api::{ImageDescriptor, ImageDescriptorFlags, ImageFormat};
use wgpu_core::device::HostMap;
pub use wgpu_core::id::markers::{
    ComputePassEncoder as ComputePass, RenderPassEncoder as RenderPass,
};
pub use wgpu_core::id::{
    ComputePassEncoderId as ComputePassId, RenderPassEncoderId as RenderPassId,
};
use wgpu_core::id::{ComputePipelineId, DeviceId, QueueId, RenderPipelineId};
use wgpu_core::instance::FailedLimit;
use wgpu_core::pipeline::CreateShaderModuleError;
use wgpu_types::{AdapterInfo, COPY_BYTES_PER_ROW_ALIGNMENT, DeviceDescriptor, Features, Limits};

pub use crate::error::*;
pub use crate::ids::*;
pub use crate::messages::*;
pub use crate::render_commands::*;

pub const PRESENTATION_BUFFER_COUNT: usize = 10;

pub type WebGPUAdapterResponse = Option<Result<Adapter, String>>;
pub type WebGPUComputePipelineResponse = Result<Pipeline<ComputePipelineId>, Error>;
pub type WebGPUPoppedErrorScopeResponse = Result<Option<Error>, PopError>;
pub type WebGPURenderPipelineResponse = Result<Pipeline<RenderPipelineId>, Error>;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WebGPU(pub IpcSender<WebGPURequest>);

impl WebGPU {
    pub fn exit(&self, sender: IpcSender<()>) -> Result<(), &'static str> {
        self.0
            .send(WebGPURequest::Exit(sender))
            .map_err(|_| "Failed to send Exit message")
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Adapter {
    pub adapter_info: AdapterInfo,
    pub adapter_id: WebGPUAdapter,
    pub features: Features,
    pub limits: Limits,
    pub channel: WebGPU,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct ContextConfiguration {
    pub device_id: DeviceId,
    pub queue_id: QueueId,
    pub format: ImageFormat,
    pub is_opaque: bool,
    pub size: Size2D<u32>,
}

impl ContextConfiguration {
    pub fn stride(&self) -> u32 {
        (self.size.width * self.format.bytes_per_pixel() as u32)
            .next_multiple_of(COPY_BYTES_PER_ROW_ALIGNMENT)
    }

    pub fn buffer_size(&self) -> u64 {
        self.stride() as u64 * self.size.height as u64
    }
}

impl From<ContextConfiguration> for ImageDescriptor {
    fn from(config: ContextConfiguration) -> Self {
        ImageDescriptor {
            format: config.format,
            size: config.size.cast().cast_unit(),
            stride: Some(config.stride() as i32),
            offset: 0,
            flags: if config.is_opaque {
                ImageDescriptorFlags::IS_OPAQUE
            } else {
                ImageDescriptorFlags::empty()
            },
        }
    }
}

/// <https://gpuweb.github.io/gpuweb/#enumdef-gpudevicelostreason>
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum DeviceLostReason {
    Unknown,
    Destroyed,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ShaderCompilationInfo {
    pub line_number: u64,
    pub line_pos: u64,
    pub offset: u64,
    pub length: u64,
    pub message: String,
}

impl ShaderCompilationInfo {
    pub fn from(error: &CreateShaderModuleError, source: &str) -> Self {
        let location = match error {
            CreateShaderModuleError::Parsing(e) => e.inner.location(source),
            CreateShaderModuleError::Validation(e) => e.inner.location(source),
            _ => None,
        };

        if let Some(location) = location {
            // Naga reports locations in UTF-8 code units, but spec requires location in UTF-16 code units
            // Based on https://searchfox.org/mozilla-central/rev/5b037d9c6ecdb0729f39ad519f0b867d80a92aad/gfx/wgpu_bindings/src/server.rs#353
            fn len_utf16(s: &str) -> u64 {
                s.chars().map(|c| c.len_utf16() as u64).sum()
            }
            let start = location.offset as usize;
            let end = start + location.length as usize;
            let line_start = source[0..start].rfind('\n').map(|pos| pos + 1).unwrap_or(0);
            Self {
                line_number: location.line_number as u64,
                line_pos: len_utf16(&source[line_start..start]) + 1,
                offset: len_utf16(&source[0..start]),
                length: len_utf16(&source[start..end]),
                message: error.to_string(),
            }
        } else {
            Self {
                message: error.to_string(),
                ..Default::default()
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Pipeline<T: std::fmt::Debug + Serialize> {
    pub id: T,
    pub label: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Mapping {
    pub data: GenericSharedMemory,
    pub mode: HostMap,
    pub range: Range<u64>,
}

pub type WebGPUDeviceResponse = (
    WebGPUDevice,
    WebGPUQueue,
    Result<DeviceDescriptor<Option<String>>, RequestDeviceError>,
);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum RequestDeviceError {
    LimitsExceeded(FailedLimit),
    UnsupportedFeature(Features),
    Other(String),
}

impl From<wgpu_core::instance::RequestDeviceError> for RequestDeviceError {
    fn from(value: wgpu_core::instance::RequestDeviceError) -> Self {
        match value {
            wgpu_core::instance::RequestDeviceError::LimitsExceeded(failed_limit) => {
                RequestDeviceError::LimitsExceeded(failed_limit)
            },
            wgpu_core::instance::RequestDeviceError::UnsupportedFeature(features) => {
                RequestDeviceError::UnsupportedFeature(features)
            },
            e => RequestDeviceError::Other(e.to_string()),
        }
    }
}
