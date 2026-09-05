/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::Arc;

use script_webgpu::traits::{WebGPUGlobalTrait, WebGPUPromiseTrait};
use webgpu_traits::Mapping;
use wgpu_core::resource::BufferAccessError;

use crate::dom::GlobalScope;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::gpu::GPU;
use crate::dom::promise::RootedPromise;
use crate::dom::types::{GPUAdapter, GPUBuffer, GPUShaderModule};
use crate::routed_promise::callback_promise;

pub(crate) mod gpu_promise_listener;
pub(crate) mod gpu {
    #[expect(clippy::upper_case_acronyms)]
    pub(crate) type GPU = script_webgpu::gpu::GPU<crate::DomTypeHolder>;
}
pub(crate) mod gpuadapter_promise_listener;
pub(crate) mod gpuadapter {
    pub(crate) type GPUAdapter = script_webgpu::gpuadapter::GPUAdapter<crate::DomTypeHolder>;
}
pub(crate) mod gpuadapterinfo {
    pub(crate) type GPUAdapterInfo =
        script_webgpu::gpuadapterinfo::GPUAdapterInfo<crate::DomTypeHolder>;
}
pub(crate) mod gpubindgroup {
    pub(crate) type GPUBindGroup = script_webgpu::gpubindgroup::GPUBindGroup<crate::DomTypeHolder>;
}
pub(crate) mod gpubindgrouplayout {
    pub(crate) type GPUBindGroupLayout =
        script_webgpu::gpubindgrouplayout::GPUBindGroupLayout<crate::DomTypeHolder>;
}
pub(crate) mod gpubuffer_promise_listener;
pub(crate) mod gpubuffer {
    pub(crate) type GPUBuffer = script_webgpu::gpubuffer::GPUBuffer<crate::DomTypeHolder>;
}
pub(crate) mod gpubufferusage {
    pub(crate) type GPUBufferUsage =
        script_webgpu::gpubufferusage::GPUBufferUsage<crate::DomTypeHolder>;
}
pub(crate) mod gpucanvascontext;
pub(crate) mod gpucolorwrite {
    pub(crate) type GPUColorWrite =
        script_webgpu::gpucolorwrite::GPUColorWrite<crate::DomTypeHolder>;
}
pub(crate) mod gpucommandbuffer {
    pub(crate) type GPUCommandBuffer =
        script_webgpu::gpucommandbuffer::GPUCommandBuffer<crate::DomTypeHolder>;
}
pub(crate) mod gpucommandencoder {
    pub(crate) type GPUCommandEncoder =
        script_webgpu::gpucommandencoder::GPUCommandEncoder<crate::DomTypeHolder>;
}
pub(crate) mod gpucompilationinfo {
    pub(crate) type GPUCompilationInfo =
        script_webgpu::gpucompilationinfo::GPUCompilationInfo<crate::DomTypeHolder>;
}
pub(crate) mod gpucompilationmessage {
    pub(crate) type GPUCompilationMessage =
        script_webgpu::gpucompilationmessage::GPUCompilationMessage<crate::DomTypeHolder>;
}
pub(crate) mod gpucomputepassencoder {
    pub(crate) type GPUComputePassEncoder =
        script_webgpu::gpucomputepassencoder::GPUComputePassEncoder<crate::DomTypeHolder>;
}
pub(crate) mod gpucomputepipeline {
    pub(crate) type GPUComputePipeline =
        script_webgpu::gpucomputepipeline::GPUComputePipeline<crate::DomTypeHolder>;
}
pub(crate) mod gpudevice;
pub(crate) mod gpudevicelostinfo {
    pub(crate) type GPUDeviceLostInfo =
        script_webgpu::gpudevicelostinfo::GPUDeviceLostInfo<crate::DomTypeHolder>;
}
pub(crate) mod gpuerror;
pub(crate) mod gpuexternaltexture;
pub(crate) mod gpuinternalerror;
pub(crate) mod gpumapmode {
    pub(crate) type GPUMapMode = script_webgpu::gpumapmode::GPUMapMode<crate::DomTypeHolder>;
}
pub(crate) mod gpuoutofmemoryerror;
pub(crate) mod gpupipelineerror;
pub(crate) mod gpupipelinelayout {
    pub(crate) type GPUPipelineLayout =
        script_webgpu::gpupipelinelayout::GPUPipelineLayout<crate::DomTypeHolder>;
}
pub(crate) mod gpuqueryset {
    pub(crate) type GPUQuerySet = script_webgpu::gpuqueryset::GPUQuerySet<crate::DomTypeHolder>;
}
pub(crate) mod gpuqueue;
pub(crate) mod gpurenderbundle {
    pub(crate) type GPURenderBundle =
        script_webgpu::gpurenderbundle::GPURenderBundle<crate::DomTypeHolder>;
}
pub(crate) mod gpurenderbundleencoder {
    pub(crate) type GPURenderBundleEncoder =
        script_webgpu::gpurenderbundleencoder::GPURenderBundleEncoder<crate::DomTypeHolder>;
}
pub(crate) mod gpurenderpassencoder {
    pub(crate) type GPURenderPassEncoder =
        script_webgpu::gpurenderpassencoder::GPURenderPassEncoder<crate::DomTypeHolder>;
}
pub(crate) mod gpurenderpipeline {
    pub(crate) type GPURenderPipeline =
        script_webgpu::gpurenderpipeline::GPURenderPipeline<crate::DomTypeHolder>;
}
pub(crate) mod gpusampler {
    pub(crate) type GPUSampler = script_webgpu::gpusampler::GPUSampler<crate::DomTypeHolder>;
}
pub(crate) mod gpushadermodule_promise_listener;
pub(crate) mod gpushadermodule {
    pub(crate) type GPUShaderModule =
        script_webgpu::gpushadermodule::GPUShaderModule<crate::DomTypeHolder>;
}
pub(crate) mod gpushaderstage {
    pub(crate) type GPUShaderStage =
        script_webgpu::gpushaderstage::GPUShaderStage<crate::DomTypeHolder>;
}
pub(crate) mod gpusupportedfeatures {
    pub(crate) type GPUSupportedFeatures =
        script_webgpu::gpusupportedfeatures::GPUSupportedFeatures<crate::DomTypeHolder>;
}
pub(crate) mod gpusupportedlimits {
    pub(crate) type GPUSupportedLimits =
        script_webgpu::gpusupportedlimits::GPUSupportedLimits<crate::DomTypeHolder>;
}
pub(crate) mod gputexture {
    pub(crate) type GPUTexture = script_webgpu::gputexture::GPUTexture<crate::DomTypeHolder>;
}
pub(crate) mod gputextureusage {
    pub(crate) type GPUTextureUsage =
        script_webgpu::gputextureusage::GPUTextureUsage<crate::DomTypeHolder>;
}
pub(crate) mod gputextureview {
    pub(crate) type GPUTextureView =
        script_webgpu::gputextureview::GPUTextureView<crate::DomTypeHolder>;
}
pub(crate) mod gpuuncapturederrorevent;
pub(crate) mod gpuvalidationerror;
pub(crate) mod identityhub {
    pub(crate) type IdentityHub = script_webgpu::identityhub::IdentityHub;
}
pub(crate) mod wgsllanguagefeatures {
    pub(crate) type WGSLLanguageFeatures =
        script_webgpu::wgsllanguagefeatures::WGSLLanguageFeatures<crate::DomTypeHolder>;
}

impl WebGPUPromiseTrait<crate::DomTypeHolder> for RootedPromise {
    fn callback_promise_adapter(
        &self,
        d: &GPUAdapter,
    ) -> servo_base::generic_channel::GenericCallback<webgpu_traits::WebGPUDeviceResponse> {
        let task_manager = <GPUAdapter as DomGlobal>::global(d).task_manager();
        callback_promise(self, d, task_manager.dom_manipulation_task_source())
    }

    fn callback_promise_gpubuffer(
        &self,
        d: &GPUBuffer,
    ) -> servo_base::generic_channel::GenericCallback<Result<Mapping, BufferAccessError>> {
        let task_manager = <GPUBuffer as DomGlobal>::global(d).task_manager();
        callback_promise(self, d, task_manager.dom_manipulation_task_source())
    }

    fn callback_promise_gpu(
        &self,
        d: &GPU,
    ) -> servo_base::generic_channel::GenericCallback<webgpu_traits::WebGPUAdapterResponse> {
        let task_manager = <GPU as DomGlobal>::global(d).task_manager();
        callback_promise(self, d, task_manager.dom_manipulation_task_source())
    }

    fn callback_promise_gpushadermodule(
        &self,
        d: &script_webgpu::gpushadermodule::GPUShaderModule<crate::DomTypeHolder>,
    ) -> servo_base::generic_channel::GenericCallback<Option<webgpu_traits::ShaderCompilationInfo>>
    {
        let task_manager = <GPUShaderModule as DomGlobal>::global(d).task_manager();
        callback_promise(self, d, task_manager.dom_manipulation_task_source())
    }
}

impl WebGPUGlobalTrait for GlobalScope {
    fn global_wgpu_id_hub(&self) -> Arc<script_webgpu::identityhub::IdentityHub> {
        self.wgpu_id_hub()
    }
}
