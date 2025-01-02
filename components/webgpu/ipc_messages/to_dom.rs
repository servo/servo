/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! IPC messages that are send to WebGPU DOM objects.

use std::ops::Range;

use ipc_channel::ipc::IpcSharedMemory;
use serde::{Deserialize, Serialize};
use wgc::id;
use wgc::pipeline::CreateShaderModuleError;
use wgpu_core::device::HostMap;
use wgpu_core::instance::{RequestAdapterError, RequestDeviceError};
use wgpu_core::resource::BufferAccessError;
pub use {wgpu_core as wgc, wgpu_types as wgt};

use crate::identity::*;
use crate::{Error, PopError, WebGPU};

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
pub struct Adapter {
    pub adapter_info: wgt::AdapterInfo,
    pub adapter_id: WebGPUAdapter,
    pub features: wgt::Features,
    pub limits: wgt::Limits,
    pub channel: WebGPU,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Pipeline<T: std::fmt::Debug + Serialize> {
    pub id: T,
    pub label: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Mapping {
    pub data: IpcSharedMemory,
    pub mode: HostMap,
    pub range: Range<u64>,
}

#[derive(Debug, Deserialize, Serialize)]
#[allow(clippy::large_enum_variant)]
pub enum WebGPUResponse {
    /// WebGPU is disabled
    None,
    Adapter(Result<Adapter, RequestAdapterError>),
    Device(
        (
            WebGPUDevice,
            WebGPUQueue,
            Result<wgt::DeviceDescriptor<Option<String>>, RequestDeviceError>,
        ),
    ),
    BufferMapAsync(Result<Mapping, BufferAccessError>),
    SubmittedWorkDone,
    PoppedErrorScope(Result<Option<Error>, PopError>),
    CompilationInfo(Option<ShaderCompilationInfo>),
    RenderPipeline(Result<Pipeline<id::RenderPipelineId>, Error>),
    ComputePipeline(Result<Pipeline<id::ComputePipelineId>, Error>),
}
