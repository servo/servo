/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub mod error;
pub mod ids;
pub mod messages;

use std::ops::Range;

use malloc_size_of_derive::MallocSizeOf;
use serde::{Deserialize, Serialize};
use servo_base::generic_channel::{GenericOneshotSender, GenericSender, GenericSharedMemory};
use webrender_api::euclid::default::Size2D;
use webrender_api::{ImageDescriptor, ImageDescriptorFlags, ImageFormat};
pub mod markers {
    pub use wgpu_core_remote_types::id::markers::{
        Adapter, BindGroup, BindGroupLayout, Buffer, CommandBuffer, CommandEncoder,
        ComputePassEncoder, ComputePipeline, Device, ExternalTexture, PipelineLayout, QuerySet,
        Queue, RenderBundle, RenderBundleEncoder, RenderPassEncoder, RenderPipeline, Sampler,
        ShaderModule, Texture, TextureView,
    };
}
pub mod id {
    pub use wgpu_core_remote_types::id::{
        AdapterId, BindGroupId, BindGroupLayoutId, BufferId, CommandBufferId, CommandEncoderId,
        ComputePassEncoderId, ComputePipelineId, DeviceId, ExternalTextureId, PipelineLayoutId,
        QuerySetId, QueueId, RenderBundleEncoderId, RenderBundleId, RenderPassEncoderId,
        RenderPipelineId, SamplerId, ShaderModuleId, TextureId, TextureViewId,
    };
}
pub use wgpu_core_remote_types::binding_model::{
    BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BindingResource, BufferBinding, BufferBindingLayout, SamplerBindingLayout,
    StorageTextureBindingLayout, TextureBindingLayout,
};
pub use wgpu_core_remote_types::encoders::{
    ComputePassDescriptor, LoadOp, PassChannel, PassTimestampWrites, RenderBundleDescriptor,
    RenderBundleEncoderDescriptor, RenderPassColorAttachment, RenderPassDepthStencilAttachment,
    RenderPassDescriptor, TexelCopyBufferInfo, TexelCopyTextureInfo, *,
};
pub use wgpu_core_remote_types::ffi::*;
use wgpu_core_remote_types::id::{ComputePipelineId, DeviceId, QueueId, RenderPipelineId};
pub use wgpu_core_remote_types::identity::IdentityManager;
pub use wgpu_core_remote_types::pipelines::{
    ComputePipelineDescriptor, FragmentState, ProgrammableStageDescriptor,
    RenderPipelineDescriptor, VertexBufferLayout, VertexState,
};
pub use wgpu_core_remote_types::{
    BufferDescriptor, BufferMapError, DeviceDescriptor, ImplementedLanguageExtension, Label,
    MapMode, PipelineError, PipelineLayoutDescriptor, QuerySetDescriptor, QueueDescriptor,
    RequestAdapterOptions, RequestDeviceError, SamplerDescriptor, ShaderModuleDescriptor,
    TextureDescriptor, TextureViewDescriptor,
};
use wgpu_types::COPY_BYTES_PER_ROW_ALIGNMENT;
pub type CompilationInfo = wgpu_types::CompilationInfo<Utf16SourceLocation>;
pub type CompilationMessage = wgpu_types::CompilationMessage<Utf16SourceLocation>;
pub use wgpu_types::{
    AdapterInfo, AddressMode, AstcBlock, AstcChannel, BlendComponent, BlendFactor, BlendOperation,
    BlendState, BufferAddress, BufferBindingType, BufferSize, BufferUsagesWebGPU as BufferUsages,
    COPY_BUFFER_ALIGNMENT, Color, ColorTargetState, ColorWrites, CommandBufferDescriptor,
    CommandEncoderDescriptor, CompareFunction, CompilationMessageType, DepthBiasState,
    DepthStencilState, DeviceLostReason, DeviceType, ExperimentalFeatures, Extent3d, Face,
    Features as FullFeatures, FeaturesWebGPU as Features, FilterMode, FrontFace,
    ImageSubresourceRange, IndexFormat, Limits, MAP_ALIGNMENT, MapMode as HostMap, MemoryHints,
    MipmapFilterMode, MultisampleState, Origin2d, Origin3d, PowerPreference, PredefinedColorSpace,
    PrimitiveState, PrimitiveTopology, QueryType, RenderBundleDepthStencil, SamplerBindingType,
    ShaderStagesWebGPU as ShaderStages, StencilFaceState, StencilOperation, StencilState,
    StorageTextureAccess, StoreOp, TexelCopyBufferLayout, TextureAspect, TextureComponentSwizzle,
    TextureDimension, TextureFormat, TextureSampleType, TextureUsages, TextureViewDimension, Trace,
    Utf16SourceLocation, VertexAttribute, VertexFormat, VertexStepMode,
};

pub fn full_features(features: Features) -> FullFeatures {
    FullFeatures::from_internal_flags(wgpu_types::FeaturesWGPU::empty(), features)
}

pub use crate::error::*;
pub use crate::ids::*;
pub use crate::messages::*;

pub const PRESENTATION_BUFFER_COUNT: usize = 10;

pub type WebGPUAdapterResponse = Option<Result<Adapter, String>>;
pub type WebGPUComputePipelineResponse = Result<Pipeline<ComputePipelineId>, PipelineError>;
pub type WebGPUPoppedErrorScopeResponse = Result<Option<Error>, ()>;
pub type WebGPURenderPipelineResponse = Result<Pipeline<RenderPipelineId>, PipelineError>;

#[derive(Clone, Debug, Deserialize, Serialize, MallocSizeOf)]
pub struct WebGPU(pub GenericSender<WebGPURequest>);

impl WebGPU {
    pub fn exit(&self, sender: GenericOneshotSender<()>) -> Result<(), &'static str> {
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
pub struct ContextConfiguration<D = DeviceId, Q = QueueId> {
    pub device: D,
    pub queue: Q,
    pub format: ImageFormat,
    pub is_opaque: bool,
    pub size: Size2D<u32>,
}

impl<D, Q> ContextConfiguration<D, Q> {
    pub fn stride(&self) -> u32 {
        (self.size.width * self.format.bytes_per_pixel() as u32)
            .next_multiple_of(COPY_BYTES_PER_ROW_ALIGNMENT)
    }

    pub fn buffer_size(&self) -> u64 {
        self.stride() as u64 * self.size.height as u64
    }
}

impl<D, Q> From<ContextConfiguration<D, Q>> for ImageDescriptor {
    fn from(config: ContextConfiguration<D, Q>) -> Self {
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
    Result<(DeviceDescriptor<'static>, bool), RequestDeviceError>,
);

#[derive(Debug, Deserialize, Serialize)]
pub enum BufferUpdate {
    Read(GenericSharedMemory),
    Write(GenericSharedMemory, Range<u64>),
}
