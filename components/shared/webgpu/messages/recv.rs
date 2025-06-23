/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! IPC messages that are received in the WebGPU thread
//! (usually from the ScriptThread, and more specifically from DOM objects)

use arrayvec::ArrayVec;
use base::id::PipelineId;
use ipc_channel::ipc::{IpcSender, IpcSharedMemory};
use pixels::IpcSnapshot;
use serde::{Deserialize, Serialize};
use webrender_api::ImageKey;
use webrender_api::units::DeviceIntSize;
use wgpu_core::Label;
use wgpu_core::binding_model::{
    BindGroupDescriptor, BindGroupLayoutDescriptor, PipelineLayoutDescriptor,
};
use wgpu_core::command::{
    RenderBundleDescriptor, RenderBundleEncoder, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, TexelCopyBufferInfo, TexelCopyTextureInfo,
};
use wgpu_core::device::HostMap;
pub use wgpu_core::id::markers::{
    ComputePassEncoder as ComputePass, RenderPassEncoder as RenderPass,
};
use wgpu_core::id::{
    AdapterId, BindGroupId, BindGroupLayoutId, BufferId, CommandBufferId, CommandEncoderId,
    ComputePassEncoderId, ComputePipelineId, DeviceId, PipelineLayoutId, QuerySetId, QueueId,
    RenderBundleId, RenderPassEncoderId, RenderPipelineId, SamplerId, ShaderModuleId, TextureId,
    TextureViewId,
};
pub use wgpu_core::id::{
    ComputePassEncoderId as ComputePassId, RenderPassEncoderId as RenderPassId,
};
use wgpu_core::instance::RequestAdapterOptions;
use wgpu_core::pipeline::{ComputePipelineDescriptor, RenderPipelineDescriptor};
use wgpu_core::resource::{
    BufferAccessError, BufferDescriptor, SamplerDescriptor, TextureDescriptor,
    TextureViewDescriptor,
};
use wgpu_types::{
    BufferAddress, CommandBufferDescriptor, CommandEncoderDescriptor, DeviceDescriptor, Extent3d,
    TexelCopyBufferLayout,
};

use crate::{
    ContextConfiguration, Error, ErrorFilter, Mapping, PRESENTATION_BUFFER_COUNT, RenderCommand,
    ShaderCompilationInfo, WebGPUAdapter, WebGPUAdapterResponse, WebGPUComputePipelineResponse,
    WebGPUContextId, WebGPUDeviceResponse, WebGPUPoppedErrorScopeResponse,
    WebGPURenderPipelineResponse,
};

#[derive(Debug, Deserialize, Serialize)]
pub enum WebGPURequest {
    BufferMapAsync {
        sender: IpcSender<Result<Mapping, BufferAccessError>>,
        buffer_id: BufferId,
        device_id: DeviceId,
        host_map: HostMap,
        offset: u64,
        size: Option<u64>,
    },
    CommandEncoderFinish {
        command_encoder_id: CommandEncoderId,
        device_id: DeviceId,
        desc: CommandBufferDescriptor<Label<'static>>,
    },
    CopyBufferToBuffer {
        command_encoder_id: CommandEncoderId,
        source_id: BufferId,
        source_offset: BufferAddress,
        destination_id: BufferId,
        destination_offset: BufferAddress,
        size: BufferAddress,
    },
    CopyBufferToTexture {
        command_encoder_id: CommandEncoderId,
        source: TexelCopyBufferInfo,
        destination: TexelCopyTextureInfo,
        copy_size: Extent3d,
    },
    CopyTextureToBuffer {
        command_encoder_id: CommandEncoderId,
        source: TexelCopyTextureInfo,
        destination: TexelCopyBufferInfo,
        copy_size: Extent3d,
    },
    CopyTextureToTexture {
        command_encoder_id: CommandEncoderId,
        source: TexelCopyTextureInfo,
        destination: TexelCopyTextureInfo,
        copy_size: Extent3d,
    },
    CreateBindGroup {
        device_id: DeviceId,
        bind_group_id: BindGroupId,
        descriptor: BindGroupDescriptor<'static>,
    },
    CreateBindGroupLayout {
        device_id: DeviceId,
        bind_group_layout_id: BindGroupLayoutId,
        descriptor: Option<BindGroupLayoutDescriptor<'static>>,
    },
    CreateBuffer {
        device_id: DeviceId,
        buffer_id: BufferId,
        descriptor: BufferDescriptor<'static>,
    },
    CreateCommandEncoder {
        device_id: DeviceId,
        command_encoder_id: CommandEncoderId,
        desc: CommandEncoderDescriptor<Label<'static>>,
    },
    CreateComputePipeline {
        device_id: DeviceId,
        compute_pipeline_id: ComputePipelineId,
        descriptor: ComputePipelineDescriptor<'static>,
        implicit_ids: Option<(PipelineLayoutId, Vec<BindGroupLayoutId>)>,
        /// present only on ASYNC versions
        async_sender: Option<IpcSender<WebGPUComputePipelineResponse>>,
    },
    CreatePipelineLayout {
        device_id: DeviceId,
        pipeline_layout_id: PipelineLayoutId,
        descriptor: PipelineLayoutDescriptor<'static>,
    },
    CreateRenderPipeline {
        device_id: DeviceId,
        render_pipeline_id: RenderPipelineId,
        descriptor: RenderPipelineDescriptor<'static>,
        implicit_ids: Option<(PipelineLayoutId, Vec<BindGroupLayoutId>)>,
        /// present only on ASYNC versions
        async_sender: Option<IpcSender<WebGPURenderPipelineResponse>>,
    },
    CreateSampler {
        device_id: DeviceId,
        sampler_id: SamplerId,
        descriptor: SamplerDescriptor<'static>,
    },
    CreateShaderModule {
        device_id: DeviceId,
        program_id: ShaderModuleId,
        program: String,
        label: Option<String>,
        sender: IpcSender<Option<ShaderCompilationInfo>>,
    },
    /// Creates context
    CreateContext {
        buffer_ids: ArrayVec<BufferId, PRESENTATION_BUFFER_COUNT>,
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
        texture_id: TextureId,
        encoder_id: CommandEncoderId,
    },
    /// Obtains image from latest presentation buffer (same as wr update)
    GetImage {
        context_id: WebGPUContextId,
        sender: IpcSender<IpcSnapshot>,
    },
    ValidateTextureDescriptor {
        device_id: DeviceId,
        texture_id: TextureId,
        descriptor: TextureDescriptor<'static>,
    },
    DestroyContext {
        context_id: WebGPUContextId,
    },
    CreateTexture {
        device_id: DeviceId,
        texture_id: TextureId,
        descriptor: TextureDescriptor<'static>,
    },
    CreateTextureView {
        texture_id: TextureId,
        texture_view_id: TextureViewId,
        device_id: DeviceId,
        descriptor: Option<TextureViewDescriptor<'static>>,
    },
    DestroyBuffer(BufferId),
    DestroyDevice(DeviceId),
    DestroyTexture(TextureId),
    DropTexture(TextureId),
    DropAdapter(AdapterId),
    DropDevice(DeviceId),
    DropBuffer(BufferId),
    DropPipelineLayout(PipelineLayoutId),
    DropComputePipeline(ComputePipelineId),
    DropRenderPipeline(RenderPipelineId),
    DropBindGroup(BindGroupId),
    DropBindGroupLayout(BindGroupLayoutId),
    DropCommandBuffer(CommandBufferId),
    DropTextureView(TextureViewId),
    DropSampler(SamplerId),
    DropShaderModule(ShaderModuleId),
    DropRenderBundle(RenderBundleId),
    DropQuerySet(QuerySetId),
    DropComputePass(ComputePassEncoderId),
    DropRenderPass(RenderPassEncoderId),
    Exit(IpcSender<()>),
    RenderBundleEncoderFinish {
        render_bundle_encoder: RenderBundleEncoder,
        descriptor: RenderBundleDescriptor<'static>,
        render_bundle_id: RenderBundleId,
        device_id: DeviceId,
    },
    RequestAdapter {
        sender: IpcSender<WebGPUAdapterResponse>,
        options: RequestAdapterOptions,
        adapter_id: AdapterId,
    },
    RequestDevice {
        sender: IpcSender<WebGPUDeviceResponse>,
        adapter_id: WebGPUAdapter,
        descriptor: DeviceDescriptor<Option<String>>,
        device_id: DeviceId,
        queue_id: QueueId,
        pipeline_id: PipelineId,
    },
    // Compute Pass
    BeginComputePass {
        command_encoder_id: CommandEncoderId,
        compute_pass_id: ComputePassId,
        label: Label<'static>,
        device_id: DeviceId,
    },
    ComputePassSetPipeline {
        compute_pass_id: ComputePassId,
        pipeline_id: ComputePipelineId,
        device_id: DeviceId,
    },
    ComputePassSetBindGroup {
        compute_pass_id: ComputePassId,
        index: u32,
        bind_group_id: BindGroupId,
        offsets: Vec<u32>,
        device_id: DeviceId,
    },
    ComputePassDispatchWorkgroups {
        compute_pass_id: ComputePassId,
        x: u32,
        y: u32,
        z: u32,
        device_id: DeviceId,
    },
    ComputePassDispatchWorkgroupsIndirect {
        compute_pass_id: ComputePassId,
        buffer_id: BufferId,
        offset: u64,
        device_id: DeviceId,
    },
    EndComputePass {
        compute_pass_id: ComputePassId,
        device_id: DeviceId,
        command_encoder_id: CommandEncoderId,
    },
    // Render Pass
    BeginRenderPass {
        command_encoder_id: CommandEncoderId,
        render_pass_id: RenderPassId,
        label: Label<'static>,
        color_attachments: Vec<Option<RenderPassColorAttachment>>,
        depth_stencil_attachment: Option<RenderPassDepthStencilAttachment>,
        device_id: DeviceId,
    },
    RenderPassCommand {
        render_pass_id: RenderPassId,
        render_command: RenderCommand,
        device_id: DeviceId,
    },
    EndRenderPass {
        render_pass_id: RenderPassId,
        device_id: DeviceId,
        command_encoder_id: CommandEncoderId,
    },
    Submit {
        device_id: DeviceId,
        queue_id: QueueId,
        command_buffers: Vec<CommandBufferId>,
    },
    UnmapBuffer {
        buffer_id: BufferId,
        /// Return back mapping for writeback
        mapping: Option<Mapping>,
    },
    WriteBuffer {
        device_id: DeviceId,
        queue_id: QueueId,
        buffer_id: BufferId,
        buffer_offset: u64,
        data: IpcSharedMemory,
    },
    WriteTexture {
        device_id: DeviceId,
        queue_id: QueueId,
        texture_cv: TexelCopyTextureInfo,
        data_layout: TexelCopyBufferLayout,
        size: Extent3d,
        data: IpcSharedMemory,
    },
    QueueOnSubmittedWorkDone {
        sender: IpcSender<()>,
        queue_id: QueueId,
    },
    PushErrorScope {
        device_id: DeviceId,
        filter: ErrorFilter,
    },
    DispatchError {
        device_id: DeviceId,
        error: Error,
    },
    PopErrorScope {
        device_id: DeviceId,
        sender: IpcSender<WebGPUPoppedErrorScopeResponse>,
    },
    ComputeGetBindGroupLayout {
        device_id: DeviceId,
        pipeline_id: ComputePipelineId,
        index: u32,
        id: BindGroupLayoutId,
    },
    RenderGetBindGroupLayout {
        device_id: DeviceId,
        pipeline_id: RenderPipelineId,
        index: u32,
        id: BindGroupLayoutId,
    },
}
