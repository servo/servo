/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! IPC messages that are received in wgpu thread
//! (usually from script thread more specifically from dom objects)

use arrayvec::ArrayVec;
use base::id::PipelineId;
use ipc_channel::ipc::{IpcSender, IpcSharedMemory};
use serde::{Deserialize, Serialize};
use webrender_api::units::DeviceIntSize;
use webrender_api::ImageKey;
use wgc::binding_model::{
    BindGroupDescriptor, BindGroupLayoutDescriptor, PipelineLayoutDescriptor,
};
use wgc::command::{
    RenderBundleDescriptor, RenderBundleEncoder, TexelCopyBufferInfo, TexelCopyTextureInfo,
};
use wgc::device::HostMap;
use wgc::id;
use wgc::instance::RequestAdapterOptions;
use wgc::pipeline::{ComputePipelineDescriptor, RenderPipelineDescriptor};
use wgc::resource::{
    BufferDescriptor, SamplerDescriptor, TextureDescriptor, TextureViewDescriptor,
};
use wgpu_core::command::{RenderPassColorAttachment, RenderPassDepthStencilAttachment};
use wgpu_core::id::AdapterId;
use wgpu_core::Label;
pub use {wgpu_core as wgc, wgpu_types as wgt};

use crate::identity::*;
use crate::render_commands::RenderCommand;
use crate::swapchain::WebGPUContextId;
use crate::{Error, ErrorFilter, Mapping, WebGPUResponse, PRESENTATION_BUFFER_COUNT};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct ContextConfiguration {
    pub device_id: id::DeviceId,
    pub queue_id: id::QueueId,
    pub format: wgt::TextureFormat,
    pub is_opaque: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum WebGPURequest {
    BufferMapAsync {
        sender: IpcSender<WebGPUResponse>,
        buffer_id: id::BufferId,
        device_id: id::DeviceId,
        host_map: HostMap,
        offset: u64,
        size: Option<u64>,
    },
    CommandEncoderFinish {
        command_encoder_id: id::CommandEncoderId,
        device_id: id::DeviceId,
        desc: wgt::CommandBufferDescriptor<Label<'static>>,
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
        source: TexelCopyBufferInfo,
        destination: TexelCopyTextureInfo,
        copy_size: wgt::Extent3d,
    },
    CopyTextureToBuffer {
        command_encoder_id: id::CommandEncoderId,
        source: TexelCopyTextureInfo,
        destination: TexelCopyBufferInfo,
        copy_size: wgt::Extent3d,
    },
    CopyTextureToTexture {
        command_encoder_id: id::CommandEncoderId,
        source: TexelCopyTextureInfo,
        destination: TexelCopyTextureInfo,
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
        descriptor: BufferDescriptor<'static>,
    },
    CreateCommandEncoder {
        device_id: id::DeviceId,
        command_encoder_id: id::CommandEncoderId,
        desc: wgt::CommandEncoderDescriptor<Label<'static>>,
    },
    CreateComputePipeline {
        device_id: id::DeviceId,
        compute_pipeline_id: id::ComputePipelineId,
        descriptor: ComputePipelineDescriptor<'static>,
        implicit_ids: Option<(id::PipelineLayoutId, Vec<id::BindGroupLayoutId>)>,
        /// present only on ASYNC versions
        async_sender: Option<IpcSender<WebGPUResponse>>,
    },
    CreatePipelineLayout {
        device_id: id::DeviceId,
        pipeline_layout_id: id::PipelineLayoutId,
        descriptor: PipelineLayoutDescriptor<'static>,
    },
    CreateRenderPipeline {
        device_id: id::DeviceId,
        render_pipeline_id: id::RenderPipelineId,
        descriptor: RenderPipelineDescriptor<'static>,
        implicit_ids: Option<(id::PipelineLayoutId, Vec<id::BindGroupLayoutId>)>,
        /// present only on ASYNC versions
        async_sender: Option<IpcSender<WebGPUResponse>>,
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
        sender: IpcSender<WebGPUResponse>,
    },
    /// Creates context
    CreateContext {
        buffer_ids: ArrayVec<id::BufferId, PRESENTATION_BUFFER_COUNT>,
        size: DeviceIntSize,
        sender: IpcSender<(WebGPUContextId, ImageKey)>,
    },
    /// Recreates swapchain (if needed)
    UpdateContext {
        context_id: WebGPUContextId,
        size: DeviceIntSize,
        configuration: Option<ContextConfiguration>,
    },
    /// Reads texture to swapchains buffer and maps it
    SwapChainPresent {
        context_id: WebGPUContextId,
        texture_id: id::TextureId,
        encoder_id: id::CommandEncoderId,
    },
    /// Obtains image from latest presentation buffer (same as wr update)
    GetImage {
        context_id: WebGPUContextId,
        sender: IpcSender<IpcSharedMemory>,
    },
    ValidateTextureDescriptor {
        device_id: id::DeviceId,
        texture_id: id::TextureId,
        descriptor: TextureDescriptor<'static>,
    },
    DestroyContext {
        context_id: WebGPUContextId,
    },
    CreateTexture {
        device_id: id::DeviceId,
        texture_id: id::TextureId,
        descriptor: TextureDescriptor<'static>,
    },
    CreateTextureView {
        texture_id: id::TextureId,
        texture_view_id: id::TextureViewId,
        device_id: id::DeviceId,
        descriptor: Option<TextureViewDescriptor<'static>>,
    },
    DestroyBuffer(id::BufferId),
    DestroyDevice(id::DeviceId),
    DestroyTexture(id::TextureId),
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
    DropRenderPass(id::RenderPassEncoderId),
    Exit(IpcSender<()>),
    RenderBundleEncoderFinish {
        render_bundle_encoder: RenderBundleEncoder,
        descriptor: RenderBundleDescriptor<'static>,
        render_bundle_id: id::RenderBundleId,
        device_id: id::DeviceId,
    },
    RequestAdapter {
        sender: IpcSender<WebGPUResponse>,
        options: RequestAdapterOptions,
        adapter_id: AdapterId,
    },
    RequestDevice {
        sender: IpcSender<WebGPUResponse>,
        adapter_id: WebGPUAdapter,
        descriptor: wgt::DeviceDescriptor<Option<String>>,
        device_id: id::DeviceId,
        queue_id: id::QueueId,
        pipeline_id: PipelineId,
    },
    // Compute Pass
    BeginComputePass {
        command_encoder_id: id::CommandEncoderId,
        compute_pass_id: ComputePassId,
        label: Label<'static>,
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
    // Render Pass
    BeginRenderPass {
        command_encoder_id: id::CommandEncoderId,
        render_pass_id: RenderPassId,
        label: Label<'static>,
        color_attachments: Vec<Option<RenderPassColorAttachment>>,
        depth_stencil_attachment: Option<RenderPassDepthStencilAttachment>,
        device_id: id::DeviceId,
    },
    RenderPassCommand {
        render_pass_id: RenderPassId,
        render_command: RenderCommand,
        device_id: id::DeviceId,
    },
    EndRenderPass {
        render_pass_id: RenderPassId,
        device_id: id::DeviceId,
        command_encoder_id: id::CommandEncoderId,
    },
    Submit {
        device_id: id::DeviceId,
        queue_id: id::QueueId,
        command_buffers: Vec<id::CommandBufferId>,
    },
    UnmapBuffer {
        buffer_id: id::BufferId,
        /// Return back mapping for writeback
        mapping: Option<Mapping>,
    },
    WriteBuffer {
        device_id: id::DeviceId,
        queue_id: id::QueueId,
        buffer_id: id::BufferId,
        buffer_offset: u64,
        data: IpcSharedMemory,
    },
    WriteTexture {
        device_id: id::DeviceId,
        queue_id: id::QueueId,
        texture_cv: TexelCopyTextureInfo,
        data_layout: wgt::TexelCopyBufferLayout,
        size: wgt::Extent3d,
        data: IpcSharedMemory,
    },
    QueueOnSubmittedWorkDone {
        sender: IpcSender<WebGPUResponse>,
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
        sender: IpcSender<WebGPUResponse>,
    },
    ComputeGetBindGroupLayout {
        device_id: id::DeviceId,
        pipeline_id: id::ComputePipelineId,
        index: u32,
        id: id::BindGroupLayoutId,
    },
    RenderGetBindGroupLayout {
        device_id: id::DeviceId,
        pipeline_id: id::RenderPipelineId,
        index: u32,
        id: id::BindGroupLayoutId,
    },
}
