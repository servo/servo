/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! IPC massages that are send/received from GPU DOM objects.

use std::borrow::Cow;

use arrayvec::ArrayVec;
use base::id::PipelineId;
use ipc_channel::ipc::{IpcSender, IpcSharedMemory};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use webrender_api::{ExternalImageId, ImageData, ImageDescriptor, ImageKey};
use wgc::binding_model::{
    BindGroupDescriptor, BindGroupLayoutDescriptor, PipelineLayoutDescriptor,
};
use wgc::command::{
    ImageCopyBuffer, ImageCopyTexture, RenderBundleDescriptor, RenderBundleEncoder, RenderPass,
};
use wgc::device::HostMap;
use wgc::id;
use wgc::instance::RequestAdapterOptions;
use wgc::pipeline::{ComputePipelineDescriptor, RenderPipelineDescriptor};
use wgc::resource::{
    BufferDescriptor, SamplerDescriptor, TextureDescriptor, TextureViewDescriptor,
};
use wgpu_core::pipeline::CreateShaderModuleError;
pub use {wgpu_core as wgc, wgpu_types as wgt};

use crate::identity::*;
use crate::{Error, ErrorFilter, PopError, WebGPU, PRESENTATION_BUFFER_COUNT};

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
#[allow(clippy::large_enum_variant)]
pub enum WebGPUResponse {
    RequestAdapter {
        adapter_info: wgt::AdapterInfo,
        adapter_id: WebGPUAdapter,
        features: wgt::Features,
        limits: wgt::Limits,
        channel: WebGPU,
    },
    RequestDevice {
        device_id: WebGPUDevice,
        queue_id: WebGPUQueue,
        descriptor: wgt::DeviceDescriptor<Option<String>>,
    },
    BufferMapAsync(IpcSharedMemory),
    SubmittedWorkDone,
    PoppedErrorScope(Result<Option<Error>, PopError>),
    CompilationInfo(Option<ShaderCompilationInfo>),
}

pub type WebGPUResponseResult = Result<WebGPUResponse, String>;

#[derive(Debug, Deserialize, Serialize)]
pub enum WebGPURequest {
    BufferMapAsync {
        sender: IpcSender<Option<WebGPUResponseResult>>,
        buffer_id: id::BufferId,
        device_id: id::DeviceId,
        host_map: HostMap,
        offset: u64,
        size: Option<u64>,
    },
    CommandEncoderFinish {
        command_encoder_id: id::CommandEncoderId,
        device_id: id::DeviceId,
        is_error: bool,
        // TODO(zakorgy): Serialize CommandBufferDescriptor in wgpu-core
        // wgc::command::CommandBufferDescriptor,
    },
    CopyBufferToBuffer {
        command_encoder_id: id::CommandEncoderId,
        source_id: id::BufferId,
        source_offset: wgt::BufferAddress,
        destination_id: id::BufferId,
        destination_offset: wgt::BufferAddress,
        size: wgt::BufferAddress,
    },
    CopyBufferToTexture {
        command_encoder_id: id::CommandEncoderId,
        source: ImageCopyBuffer,
        destination: ImageCopyTexture,
        copy_size: wgt::Extent3d,
    },
    CopyTextureToBuffer {
        command_encoder_id: id::CommandEncoderId,
        source: ImageCopyTexture,
        destination: ImageCopyBuffer,
        copy_size: wgt::Extent3d,
    },
    CopyTextureToTexture {
        command_encoder_id: id::CommandEncoderId,
        source: ImageCopyTexture,
        destination: ImageCopyTexture,
        copy_size: wgt::Extent3d,
    },
    CreateBindGroup {
        device_id: id::DeviceId,
        bind_group_id: id::BindGroupId,
        descriptor: BindGroupDescriptor<'static>,
    },
    CreateBindGroupLayout {
        device_id: id::DeviceId,
        bind_group_layout_id: id::BindGroupLayoutId,
        descriptor: Option<BindGroupLayoutDescriptor<'static>>,
    },
    CreateBuffer {
        device_id: id::DeviceId,
        buffer_id: id::BufferId,
        descriptor: Option<BufferDescriptor<'static>>,
    },
    CreateCommandEncoder {
        device_id: id::DeviceId,
        // TODO(zakorgy): Serialize CommandEncoderDescriptor in wgpu-core
        // wgc::command::CommandEncoderDescriptor,
        command_encoder_id: id::CommandEncoderId,
        label: Option<Cow<'static, str>>,
    },
    CreateComputePipeline {
        device_id: id::DeviceId,
        compute_pipeline_id: id::ComputePipelineId,
        descriptor: ComputePipelineDescriptor<'static>,
        implicit_ids: Option<(id::PipelineLayoutId, Vec<id::BindGroupLayoutId>)>,
    },
    CreateContext(IpcSender<ExternalImageId>),
    CreatePipelineLayout {
        device_id: id::DeviceId,
        pipeline_layout_id: id::PipelineLayoutId,
        descriptor: PipelineLayoutDescriptor<'static>,
    },
    CreateRenderPipeline {
        device_id: id::DeviceId,
        render_pipeline_id: id::RenderPipelineId,
        descriptor: Option<RenderPipelineDescriptor<'static>>,
        implicit_ids: Option<(id::PipelineLayoutId, Vec<id::BindGroupLayoutId>)>,
    },
    CreateSampler {
        device_id: id::DeviceId,
        sampler_id: id::SamplerId,
        descriptor: SamplerDescriptor<'static>,
    },
    CreateShaderModule {
        device_id: id::DeviceId,
        program_id: id::ShaderModuleId,
        program: String,
        label: Option<String>,
        sender: IpcSender<Option<WebGPUResponseResult>>,
    },
    CreateSwapChain {
        device_id: id::DeviceId,
        buffer_ids: ArrayVec<id::BufferId, PRESENTATION_BUFFER_COUNT>,
        external_id: u64,
        sender: IpcSender<ImageKey>,
        image_desc: ImageDescriptor,
        image_data: ImageData,
    },
    CreateTexture {
        device_id: id::DeviceId,
        texture_id: id::TextureId,
        descriptor: Option<TextureDescriptor<'static>>,
    },
    CreateTextureView {
        texture_id: id::TextureId,
        texture_view_id: id::TextureViewId,
        device_id: id::DeviceId,
        descriptor: Option<TextureViewDescriptor<'static>>,
    },
    DestroyBuffer(id::BufferId),
    DestroyDevice(id::DeviceId),
    DestroyTexture {
        device_id: id::DeviceId,
        texture_id: id::TextureId,
    },
    DestroySwapChain {
        external_id: u64,
        image_key: ImageKey,
    },
    DropTexture(id::TextureId),
    DropAdapter(id::AdapterId),
    DropDevice(id::DeviceId),
    DropBuffer(id::BufferId),
    DropPipelineLayout(id::PipelineLayoutId),
    DropComputePipeline(id::ComputePipelineId),
    DropRenderPipeline(id::RenderPipelineId),
    DropBindGroup(id::BindGroupId),
    DropBindGroupLayout(id::BindGroupLayoutId),
    DropCommandBuffer(id::CommandBufferId),
    DropTextureView(id::TextureViewId),
    DropSampler(id::SamplerId),
    DropShaderModule(id::ShaderModuleId),
    DropRenderBundle(id::RenderBundleId),
    DropQuerySet(id::QuerySetId),
    DropComputePass(id::ComputePassEncoderId),
    Exit(IpcSender<()>),
    RenderBundleEncoderFinish {
        render_bundle_encoder: RenderBundleEncoder,
        descriptor: RenderBundleDescriptor<'static>,
        render_bundle_id: id::RenderBundleId,
        device_id: id::DeviceId,
    },
    RequestAdapter {
        sender: IpcSender<Option<WebGPUResponseResult>>,
        options: RequestAdapterOptions,
        ids: SmallVec<[id::AdapterId; 4]>,
    },
    RequestDevice {
        sender: IpcSender<Option<WebGPUResponseResult>>,
        adapter_id: WebGPUAdapter,
        descriptor: wgt::DeviceDescriptor<Option<String>>,
        device_id: id::DeviceId,
        pipeline_id: PipelineId,
    },
    BeginComputePass {
        command_encoder_id: id::CommandEncoderId,
        compute_pass_id: ComputePassId,
        label: Option<Cow<'static, str>>,
        device_id: id::DeviceId,
    },
    ComputePassSetPipeline {
        compute_pass_id: ComputePassId,
        pipeline_id: id::ComputePipelineId,
        device_id: id::DeviceId,
    },
    ComputePassSetBindGroup {
        compute_pass_id: ComputePassId,
        index: u32,
        bind_group_id: id::BindGroupId,
        offsets: Vec<u32>,
        device_id: id::DeviceId,
    },
    ComputePassDispatchWorkgroups {
        compute_pass_id: ComputePassId,
        x: u32,
        y: u32,
        z: u32,
        device_id: id::DeviceId,
    },
    ComputePassDispatchWorkgroupsIndirect {
        compute_pass_id: ComputePassId,
        buffer_id: id::BufferId,
        offset: u64,
        device_id: id::DeviceId,
    },
    EndComputePass {
        compute_pass_id: ComputePassId,
        device_id: id::DeviceId,
        command_encoder_id: id::CommandEncoderId,
    },
    EndRenderPass {
        render_pass: Option<RenderPass>,
        device_id: id::DeviceId,
    },
    Submit {
        queue_id: id::QueueId,
        command_buffers: Vec<id::CommandBufferId>,
    },
    SwapChainPresent {
        external_id: u64,
        texture_id: id::TextureId,
        encoder_id: id::CommandEncoderId,
    },
    UnmapBuffer {
        buffer_id: id::BufferId,
        device_id: id::DeviceId,
        array_buffer: IpcSharedMemory,
        is_map_read: bool,
        offset: u64,
        size: u64,
    },
    WriteBuffer {
        queue_id: id::QueueId,
        buffer_id: id::BufferId,
        buffer_offset: u64,
        data: IpcSharedMemory,
    },
    WriteTexture {
        queue_id: id::QueueId,
        texture_cv: ImageCopyTexture,
        data_layout: wgt::ImageDataLayout,
        size: wgt::Extent3d,
        data: IpcSharedMemory,
    },
    QueueOnSubmittedWorkDone {
        sender: IpcSender<Option<WebGPUResponseResult>>,
        queue_id: id::QueueId,
    },
    PushErrorScope {
        device_id: id::DeviceId,
        filter: ErrorFilter,
    },
    DispatchError {
        device_id: id::DeviceId,
        error: Error,
    },
    PopErrorScope {
        device_id: id::DeviceId,
        sender: IpcSender<Option<WebGPUResponseResult>>,
    },
}
