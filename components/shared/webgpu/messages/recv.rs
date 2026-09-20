/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! IPC messages that are received in the WebGPU thread
//! (usually from the ScriptThread, and more specifically from DOM objects)

use arrayvec::ArrayVec;
use pixels::{SharedSnapshot, SnapshotPixelFormat};
use serde::{Deserialize, Serialize};
use servo_base::Epoch;
use servo_base::generic_channel::{
    GenericCallback, GenericOneshotSender, GenericSender, GenericSharedMemory,
};
use servo_base::id::PipelineId;
use webrender_api::ImageKey;
use webrender_api::euclid::default::Size2D;
use webrender_api::units::DeviceIntSize;

use crate::id::*;
use crate::{
    BindGroupDescriptor, BindGroupLayoutDescriptor, BufferAccessError, BufferDescriptor,
    CommandBufferDescriptor, CommandEncoderCommand, CommandEncoderDescriptor,
    ComputePassEncoderCommand, ComputePipelineDescriptor, ContextConfiguration, DeviceDescriptor,
    Error, ErrorFilter, Extent3d, HostMap, Label, Mapping, PRESENTATION_BUFFER_COUNT,
    PassTimestampWrites, PipelineLayoutDescriptor, QuerySetDescriptor, RenderBundleDescriptor,
    RenderBundleEncoderCommand, RenderBundleEncoderDescriptor, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, RenderPassEncoderCommand, RenderPipelineDescriptor,
    RequestAdapterOptions, SamplerDescriptor, ShaderCompilationInfo, TexelCopyBufferLayout,
    TexelCopyTextureInfo, TextureDescriptor, TextureViewDescriptor, WebGPUAdapter,
    WebGPUAdapterResponse, WebGPUComputePipelineResponse, WebGPUContextId, WebGPUDeviceResponse,
    WebGPUPoppedErrorScopeResponse, WebGPURenderPipelineResponse,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct PendingTexture {
    pub texture_id: TextureId,
    pub encoder_id: CommandEncoderId,
    pub command_buffer_id: CommandBufferId,
    pub configuration: ContextConfiguration,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum WebGPURequest {
    SetImageKey {
        context_id: WebGPUContextId,
        image_key: ImageKey,
    },
    BufferMapAsync {
        callback: GenericCallback<Result<Mapping, BufferAccessError>>,
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
        command_buffer_id: CommandBufferId,
    },
    CommandEncoderCommand {
        command_encoder_id: CommandEncoderId,
        command: CommandEncoderCommand,
        device_id: DeviceId,
    },
    CopyExternalImageToTexture {
        device_id: DeviceId,
        queue_id: QueueId,
        usable_source: Option<SharedSnapshot>,
        destination: TexelCopyTextureInfo,
        dest_tex_descriptor: TextureDescriptor<'static>,
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
        /// present only on ASYNC versions
        async_sender: Option<GenericCallback<WebGPUComputePipelineResponse>>,
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
        /// present only on ASYNC versions
        async_sender: Option<GenericCallback<WebGPURenderPipelineResponse>>,
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
        callback: GenericCallback<Option<ShaderCompilationInfo>>,
    },
    /// Creates context
    CreateContext {
        buffer_ids: ArrayVec<BufferId, PRESENTATION_BUFFER_COUNT>,
        size: DeviceIntSize,
        sender: GenericSender<WebGPUContextId>,
    },
    /// Present texture to WebRender
    Present {
        context_id: WebGPUContextId,
        pending_texture: Option<PendingTexture>,
        size: Size2D<u32>,
        canvas_epoch: Epoch,
    },
    /// Create [`pixels::Snapshot`] with contents of the last present operation
    /// or provided pending texture and send it over provided [`IpcSender`].
    GetImage {
        context_id: WebGPUContextId,
        pending_texture: Option<PendingTexture>,
        sender: GenericSender<SharedSnapshot>,
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
    DropCommandEncoder(CommandEncoderId),
    DropCommandBuffer(CommandBufferId),
    DropTextureView(TextureViewId),
    DropSampler(SamplerId),
    DropShaderModule(ShaderModuleId),
    DropRenderBundle(RenderBundleId),
    DropQuerySet(QuerySetId),
    DropComputePass(ComputePassEncoderId),
    DropRenderPass(RenderPassEncoderId),
    Exit(GenericOneshotSender<()>),
    RenderBundleEncoderFinish {
        render_bundle_encoder_id: RenderBundleEncoderId,
        descriptor: RenderBundleDescriptor<'static>,
        render_bundle_id: RenderBundleId,
        device_id: DeviceId,
    },
    RequestAdapter {
        sender: GenericCallback<WebGPUAdapterResponse>,
        options: RequestAdapterOptions,
        adapter_id: AdapterId,
    },
    RequestDevice {
        sender: GenericCallback<WebGPUDeviceResponse>,
        adapter_id: WebGPUAdapter,
        descriptor: DeviceDescriptor<Option<String>>,
        device_id: DeviceId,
        queue_id: QueueId,
        pipeline_id: PipelineId,
    },
    // Compute Pass
    BeginComputePass {
        command_encoder_id: CommandEncoderId,
        compute_pass_id: ComputePassEncoderId,
        label: Label<'static>,
        timestamp_writes: Option<PassTimestampWrites>,
        device_id: DeviceId,
    },
    ComputePassCommand {
        compute_pass_id: ComputePassEncoderId,
        compute_command: ComputePassEncoderCommand,
        device_id: DeviceId,
    },
    EndComputePass {
        compute_pass_id: ComputePassEncoderId,
        device_id: DeviceId,
    },
    // Render Pass
    BeginRenderPass {
        command_encoder_id: CommandEncoderId,
        render_pass_id: RenderPassEncoderId,
        label: Label<'static>,
        color_attachments: Vec<Option<RenderPassColorAttachment>>,
        depth_stencil_attachment: Option<RenderPassDepthStencilAttachment<TextureViewId>>,
        timestamp_writes: Option<PassTimestampWrites>,
        device_id: DeviceId,
    },
    RenderPassCommand {
        render_pass_id: RenderPassEncoderId,
        render_command: RenderPassEncoderCommand,
        device_id: DeviceId,
    },
    EndRenderPass {
        render_pass_id: RenderPassEncoderId,
        device_id: DeviceId,
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
        data: GenericSharedMemory,
    },
    WriteTexture {
        device_id: DeviceId,
        queue_id: QueueId,
        texture_cv: TexelCopyTextureInfo,
        data_layout: TexelCopyBufferLayout,
        size: Extent3d,
        data: GenericSharedMemory,
    },
    QueueOnSubmittedWorkDone {
        sender: GenericCallback<()>,
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
        callback: GenericCallback<WebGPUPoppedErrorScopeResponse>,
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
    CreateQuerySet {
        device_id: DeviceId,
        query_set_id: QuerySetId,
        descriptor: QuerySetDescriptor<'static>,
    },
    /// Create planar texture and view to be imported as external texture
    CreatePlanarTexture {
        device_id: DeviceId,
        size: Size2D<u32>,
        format: SnapshotPixelFormat,
        texture_id: TextureId,
        /// aka plane
        texture_view_id: TextureViewId,
    },
    UpdatePlanarTexture {
        device_id: DeviceId,
        queue_id: QueueId,
        texture_id: TextureId,
        snapshot: SharedSnapshot,
    },
    DropPlanarTexture(TextureId, TextureViewId),
    /// Import plane as external texture, if plane not provided it creates invalid external texture
    ImportExternalTexture {
        device_id: DeviceId,
        external_texture_id: ExternalTextureId,
        label: String,
        size: Size2D<u32>,
        plane0: Option<TextureViewId>,
    },
    DestroyExternalTexture(ExternalTextureId),
    DropExternalTexture(ExternalTextureId),
    DestroyQuerySet(QuerySetId),
    CreateRenderBundleEncoder {
        device_id: DeviceId,
        render_bundle_encoder_id: RenderBundleEncoderId,
        desc: RenderBundleEncoderDescriptor<'static>,
    },
    RenderBundleEncoderCommand {
        render_bundle_encoder_id: RenderBundleEncoderId,
        render_command: RenderBundleEncoderCommand,
        device_id: DeviceId,
    },
    DropRenderBundleEncoder(RenderBundleEncoderId),
}
