/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;
use std::sync::Arc;

use js::context::JSContext;
use script_bindings::DomTypes;
use script_bindings::codegen::GenericBindings::WebGPUBinding::GPUTextureFormat;
use script_bindings::error::Fallible;
use servo_base::generic_channel::GenericCallback;
use webgpu_traits::{
    Mapping, WebGPU, WebGPUAdapterResponse, WebGPUDevice, WebGPUDeviceResponse,
    WebGPUExternalTexture, WebGPUQuerySet, WebGPUSampler, WebGPUShaderModule, WebGPUTexture,
    WebGPUTextureView,
};
use wgpu_core::resource::BufferAccessError;
use wgpu_types::TextureFormat;

use crate::gpu::GPU;
use crate::gpuadapter::GPUAdapter;
use crate::gpuadapterinfo::GPUAdapterInfo;
use crate::gpubindgroup::GPUBindGroup;
use crate::gpubindgrouplayout::GPUBindGroupLayout;
use crate::gpubuffer::GPUBuffer;
use crate::gpubufferusage::GPUBufferUsage;
use crate::gpucolorwrite::GPUColorWrite;
use crate::gpucommandbuffer::GPUCommandBuffer;
use crate::gpucompilationinfo::GPUCompilationInfo;
use crate::gpucompilationmessage::GPUCompilationMessage;
use crate::gpudevicelostinfo::GPUDeviceLostInfo;
use crate::gpumapmode::GPUMapMode;
use crate::gpurenderbundle::GPURenderBundle;
use crate::gpushaderstage::GPUShaderStage;
use crate::gpusupportedfeatures::GPUSupportedFeatures;
use crate::gpusupportedlimits::GPUSupportedLimits;
use crate::gputextureusage::GPUTextureUsage;
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
        GPUCompilationInfo = GPUCompilationInfo<Self>,
        GPUCompilationMessage = GPUCompilationMessage<Self>,
        GPUDeviceLostInfo = GPUDeviceLostInfo<Self>,
        GPUMapMode = GPUMapMode<Self>,
        GPURenderBundle = GPURenderBundle<Self>,
        GPUShaderStage = GPUShaderStage<Self>,
        GPUSupportedFeatures = GPUSupportedFeatures<Self>,
        GPUSupportedLimits = GPUSupportedLimits<Self>,
        GPUTextureUsage = GPUTextureUsage<Self>,
        WGSLLanguageFeatures = WGSLLanguageFeatures<Self>>;
}

/// The main trait for creating and using promises in script_webgpu.
pub trait WebGPUPromiseTrait<D: DomTypes> {
    fn callback_promise_adapter(
        self: &Rc<Self>,
        d: &GPUAdapter<D>,
    ) -> GenericCallback<WebGPUDeviceResponse>;

    fn callback_promise_gpubuffer(
        self: &Rc<Self>,
        d: &GPUBuffer<D>,
    ) -> GenericCallback<Result<Mapping, BufferAccessError>>;

    fn callback_promise_gpu(self: &Rc<Self>, d: &GPU<D>) -> GenericCallback<WebGPUAdapterResponse>;
}

pub trait WebGPUGlobalTrait {
    fn global_wgpu_id_hub(&self) -> Arc<IdentityHub>;
}

pub trait GPUDeviceTrait<D: DomTypes> {
    fn is_lost(&self) -> bool;
    fn id(&self) -> WebGPUDevice;
    fn channel(&self) -> WebGPU;
    fn dispatch_error(&self, error: webgpu_traits::Error);
    fn validate_texture_format_required_features(
        &self,
        gpu_texture_format: &GPUTextureFormat,
    ) -> Fallible<TextureFormat>;
}

pub trait GPUTextureTrait {
    fn id(&self) -> WebGPUTexture;
    fn get_default_view(&self, cx: &mut JSContext) -> WebGPUTextureView;
}

pub trait GPUTextureViewTrait {
    fn id(&self) -> WebGPUTextureView;
}

pub trait GPUSamplerTrait {
    fn id(&self) -> WebGPUSampler;
}

pub trait GPUExternalTextureTrait {
    fn id(&self) -> WebGPUExternalTexture;
}

pub trait GPUQuerySetTrait {
    fn id(&self) -> WebGPUQuerySet;
}

pub trait GPUShaderModuleTrait {
    fn id(&self) -> WebGPUShaderModule;
}
