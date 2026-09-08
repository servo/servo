/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::Arc;

use script_bindings::DomTypes;
use script_bindings::codegen::GenericBindings::WebGPUBinding::GPUTextureFormat;
use script_bindings::codegen::GenericUnionTypes::GPUPipelineLayoutOrGPUAutoLayoutMode;
use script_bindings::error::Fallible;
use script_bindings::reflector::DomGlobalGeneric;
use script_bindings::str::DOMString;
use servo_base::generic_channel::GenericCallback;
use stylo_atoms::Atom;
use webgpu_traits::{
    Mapping, ShaderCompilationInfo, WebGPU, WebGPUAdapterResponse, WebGPUDevice,
    WebGPUDeviceResponse, WebGPUExternalTexture, WebGPUQueue,
};
use wgpu_core::resource::BufferAccessError;
use wgpu_types::TextureFormat;

use crate::PipelineLayout;
use crate::gpu::GPU;
use crate::gpuadapter::GPUAdapter;
use crate::gpuadapterinfo::GPUAdapterInfo;
use crate::gpubindgroup::GPUBindGroup;
use crate::gpubindgrouplayout::GPUBindGroupLayout;
use crate::gpubuffer::GPUBuffer;
use crate::gpubufferusage::GPUBufferUsage;
use crate::gpucolorwrite::GPUColorWrite;
use crate::gpucommandbuffer::GPUCommandBuffer;
use crate::gpucommandencoder::GPUCommandEncoder;
use crate::gpucompilationinfo::GPUCompilationInfo;
use crate::gpucompilationmessage::GPUCompilationMessage;
use crate::gpucomputepassencoder::GPUComputePassEncoder;
use crate::gpucomputepipeline::GPUComputePipeline;
use crate::gpudevicelostinfo::GPUDeviceLostInfo;
use crate::gpuerror::GPUError;
use crate::gpuinternalerror::GPUInternalError;
use crate::gpumapmode::GPUMapMode;
use crate::gpuoutofmemoryerror::GPUOutOfMemoryError;
use crate::gpupipelineerror::GPUPipelineError;
use crate::gpupipelinelayout::GPUPipelineLayout;
use crate::gpuqueryset::GPUQuerySet;
use crate::gpurenderbundle::GPURenderBundle;
use crate::gpurenderbundleencoder::GPURenderBundleEncoder;
use crate::gpurenderpassencoder::GPURenderPassEncoder;
use crate::gpurenderpipeline::GPURenderPipeline;
use crate::gpusampler::GPUSampler;
use crate::gpushadermodule::GPUShaderModule;
use crate::gpushaderstage::GPUShaderStage;
use crate::gpusupportedfeatures::GPUSupportedFeatures;
use crate::gpusupportedlimits::GPUSupportedLimits;
use crate::gputexture::GPUTexture;
use crate::gputextureusage::GPUTextureUsage;
use crate::gputextureview::GPUTextureView;
use crate::gpuuncapturederrorevent::GPUUncapturedErrorEvent;
use crate::gpuvalidationerror::GPUValidationError;
use crate::identityhub::IdentityHub;
use crate::wgsllanguagefeatures::WGSLLanguageFeatures;

// This trait enforces the equivalence of all local types with the types in DomTypes.
trait_set::trait_set! {
pub trait Equivalence =  DomTypes<
    GPU = GPU<Self>,
        GPUAdapter = GPUAdapter<Self>,
        GPUAdapterInfo = GPUAdapterInfo<Self>,
        GPUBindGroup = GPUBindGroup<Self>,
        GPUBindGroupLayout = GPUBindGroupLayout<Self>,
        GPUBuffer = GPUBuffer<Self>,
        GPUBufferUsage = GPUBufferUsage<Self>,
        GPUColorWrite = GPUColorWrite<Self>,
        GPUCommandBuffer = GPUCommandBuffer<Self>,
        GPUCommandEncoder = GPUCommandEncoder<Self>,
        GPUCompilationInfo = GPUCompilationInfo<Self>,
        GPUCompilationMessage = GPUCompilationMessage<Self>,
        GPUComputePassEncoder = GPUComputePassEncoder<Self>,
        GPUComputePipeline = GPUComputePipeline<Self>,
        GPUDeviceLostInfo = GPUDeviceLostInfo<Self>,
        GPUError = GPUError<Self>,
        GPUInternalError = GPUInternalError<Self>,
        GPUMapMode = GPUMapMode<Self>,
        GPUOutOfMemoryError = GPUOutOfMemoryError<Self>,
        GPUPipelineError = GPUPipelineError<Self>,
        GPUPipelineLayout = GPUPipelineLayout<Self>,
        GPUQuerySet = GPUQuerySet<Self>,
        GPURenderBundle = GPURenderBundle<Self>,
        GPURenderBundleEncoder = GPURenderBundleEncoder<Self>,
        GPURenderPassEncoder = GPURenderPassEncoder<Self>,
        GPURenderPipeline = GPURenderPipeline<Self>,
        GPUSampler = GPUSampler<Self>,
        GPUShaderModule = GPUShaderModule<Self>,
        GPUShaderStage = GPUShaderStage<Self>,
        GPUSupportedFeatures = GPUSupportedFeatures<Self>,
        GPUSupportedLimits = GPUSupportedLimits<Self>,
        GPUTexture = GPUTexture<Self>,
        GPUTextureUsage = GPUTextureUsage<Self>,
        GPUTextureView = GPUTextureView<Self>,
        GPUUncapturedErrorEvent = GPUUncapturedErrorEvent<Self>,
        GPUValidationError = GPUValidationError<Self>,
        WGSLLanguageFeatures = WGSLLanguageFeatures<Self>>;
}

/// The main trait for creating and using promises in script_webgpu.
pub trait WebGPUPromiseTrait<D: DomTypes> {
    fn callback_promise_adapter(&self, d: &GPUAdapter<D>) -> GenericCallback<WebGPUDeviceResponse>;

    fn callback_promise_gpubuffer(
        &self,
        d: &GPUBuffer<D>,
    ) -> GenericCallback<Result<Mapping, BufferAccessError>>;

    fn callback_promise_gpu(&self, d: &GPU<D>) -> GenericCallback<WebGPUAdapterResponse>;

    fn callback_promise_gpushadermodule(
        &self,
        d: &GPUShaderModule<D>,
    ) -> GenericCallback<Option<ShaderCompilationInfo>>;
}

pub trait WebGPUGlobalTrait {
    fn global_wgpu_id_hub(&self) -> Arc<IdentityHub>;
}

pub trait GPUDeviceTrait<D: DomTypes>: DomGlobalGeneric<D> {
    fn is_lost(&self) -> bool;
    fn id(&self) -> WebGPUDevice;
    fn channel(&self) -> WebGPU;
    fn dispatch_error(&self, error: webgpu_traits::Error);
    fn validate_texture_format_required_features(
        &self,
        gpu_texture_format: &GPUTextureFormat,
    ) -> Fallible<TextureFormat>;
    fn get_pipeline_layout_data(
        &self,
        layout: &GPUPipelineLayoutOrGPUAutoLayoutMode<D>,
    ) -> PipelineLayout;
    fn queue_id(&self) -> WebGPUQueue;
}

pub trait GPUExternalTextureTrait<D: DomTypes> {
    fn id(&self) -> WebGPUExternalTexture;
}

pub trait WebGPUEventTrait {
    fn new_inherited() -> Self;
    fn init_event(&self, type_: Atom, bubbles: bool, cancelable: bool);
    #[expect(non_snake_case)]
    fn IsTrusted(&self) -> bool;
}

pub trait WebGPUDomExceptionTrait {
    fn new_inherited(message: DOMString, name: DOMString) -> Self;
}
