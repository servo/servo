/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Ref;
use std::rc::Rc;
use std::sync::Arc;

use euclid::default::Size2D;
use js::context::NoGC;
use pixels::Snapshot;
use script_bindings::DomTypes;
use script_bindings::callback::CallbackContainer;
use script_bindings::conversions::DerivedFrom;
use script_bindings::error::{Error, Fallible};
use script_bindings::inheritance::Castable;
use script_bindings::interfaces::PromiseHelpers;
use script_bindings::reflector::{DomGlobalGeneric, DomObject};
use script_bindings::root::DomRoot;
use script_bindings::tasks::TaskOnce;
use script_bindings::traits::DomEventTrait;
use serde_core::Serialize;
use servo_base::generic_channel::GenericCallback;
use servo_url::MutableOrigin;
use webgpu_traits::{
    Mapping, ShaderCompilationInfo, WebGPUAdapterResponse, WebGPUComputePipelineResponse,
    WebGPUDeviceResponse, WebGPUPoppedErrorScopeResponse, WebGPURenderPipelineResponse,
};
use wgpu_core::resource::BufferAccessError;

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
use crate::gpudevice::GPUDevice;
use crate::gpudevicelostinfo::GPUDeviceLostInfo;
use crate::gpuerror::GPUError;
use crate::gpuexternaltexture::GPUExternalTexture;
use crate::gpuinternalerror::GPUInternalError;
use crate::gpumapmode::GPUMapMode;
use crate::gpuoutofmemoryerror::GPUOutOfMemoryError;
use crate::gpupipelineerror::GPUPipelineError;
use crate::gpupipelinelayout::GPUPipelineLayout;
use crate::gpuqueryset::GPUQuerySet;
use crate::gpuqueue::GPUQueue;
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
pub trait Equivalence = DomTypes<
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
        GPUDevice = GPUDevice<Self>,
        GPUDeviceLostInfo = GPUDeviceLostInfo<Self>,
        GPUError = GPUError<Self>,
        GPUExternalTexture = GPUExternalTexture<Self>,
        GPUInternalError = GPUInternalError<Self>,
        GPUMapMode = GPUMapMode<Self>,
        GPUOutOfMemoryError = GPUOutOfMemoryError<Self>,
        GPUPipelineError = GPUPipelineError<Self>,
        GPUPipelineLayout = GPUPipelineLayout<Self>,
        GPUQueue = GPUQueue<Self>,
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
        WGSLLanguageFeatures = WGSLLanguageFeatures<Self>,
        // End of Equivalence
        GPU: DomGlobalGeneric<Self>,
        GPUAdapter: DomGlobalGeneric<Self>,
        GPUComputePipeline: DomGlobalGeneric<Self>,
        GPUCommandEncoder: DomGlobalGeneric<Self>,
        GPUDevice: DomGlobalGeneric<Self>,
        GPURenderBundleEncoder: DomGlobalGeneric<Self>,
        GPURenderPipeline: DomGlobalGeneric<Self>,
        GPUError: Castable,
        GPUQueue: DomGlobalGeneric<Self>,
        GPUTexture: DomGlobalGeneric<Self>,
        GPUValidationError: DerivedFrom<GPUError<Self>>,
        GPUOutOfMemoryError: DerivedFrom<GPUError<Self>>,
        GPUInternalError: DerivedFrom<GPUError<Self>>,
        // Other bounds
        HTMLVideoElement: WebGPUHTMLVideoTrait<Self>,
        // General Bounds
        GlobalScope: WebGPUGlobalTrait,
        Promise: PromiseHelpers<Self> + WebGPUTracedPromiseTrait<Self> + PartialEq,
        Event: DomEventTrait<Self>,
        EventTarget: EventTargetTrait<Self>>;

    pub trait WebGPUPromise<D: DomTypes> =
        WebGPUPromiseCallbackTrait<D, GPU<D>, WebGPUAdapterResponse>
        + WebGPUPromiseCallbackTrait<D, GPUAdapter<D>, WebGPUDeviceResponse>
        + WebGPUPromiseCallbackTrait<D, GPUBuffer<D>, Result<Mapping, BufferAccessError>>
        + WebGPUPromiseCallbackTrait<D, GPUDevice<D>, WebGPUPoppedErrorScopeResponse>
        + WebGPUPromiseCallbackTrait<D, GPUDevice<D>, WebGPUComputePipelineResponse>
        + WebGPUPromiseCallbackTrait<D, GPUDevice<D>, WebGPURenderPipelineResponse>
        + WebGPUPromiseCallbackTrait<D, GPUQueue<D>, ()>
        + WebGPUPromiseCallbackTrait<D, GPUShaderModule<D>, Option<ShaderCompilationInfo>>
        + WebGPURootedPromiseTrait<D>;
}

/// Trait for Rooted Promise
pub trait WebGPURootedPromiseTrait<D: DomTypes> {
    fn new_rooted(
        cx: &mut js::context::JSContext,
        global: &D::GlobalScope,
    ) -> <D::Promise as PromiseHelpers<D>>::StackRoot;
}

/// Trait for sending Promise callbacks
pub trait WebGPUPromiseCallbackTrait<D: DomTypes, S, T: Serialize + 'static + Send> {
    fn callback_promise_dom_manipulation_task_source(&self, d: &S) -> GenericCallback<T>;
}

/// Trait that needs to be implemented for TracedPromise
pub trait WebGPUTracedPromiseTrait<D: DomTypes> {
    fn is_fulfilled(&self) -> bool;
}

pub trait WebGPUGlobalTrait: Sized + DomObject {
    fn global_wgpu_id_hub(&self) -> Arc<IdentityHub>;
    fn queue_webgpu_task_source(&self, task: impl TaskOnce + 'static);
    fn entry() -> DomRoot<Self>;
    fn origin(&self) -> MutableOrigin;
}

#[expect(clippy::type_complexity)]
pub trait WebGPUHTMLVideoTrait<D: DomTypes>: OriginIsCleanTrait {
    fn planar_video_for_webgpu(
        &self,
        device: &GPUDevice<D>,
    ) -> Fallible<(
        Size2D<u32>,
        Option<Rc<crate::gpuexternaltexture::PlanarTexture<D>>>,
    )>;
    fn is_usable(&self) -> bool;
    fn get_current_frame_data(&self) -> Option<Snapshot>;
}

pub trait EventTargetTrait<D: DomTypes> {
    fn new_inherited() -> D::EventTarget;
    fn get_event_handler_common<T: CallbackContainer<D>>(
        &self,
        cx: &mut js::context::JSContext,
        ty: &str,
    ) -> Option<Rc<T>>;
    fn set_event_handler_common<T: CallbackContainer<D>>(
        &self,
        cx: &mut js::context::JSContext,
        ty: &str,
        listener: Option<Rc<T>>,
    );
}

pub trait OriginIsCleanTrait {
    fn origin_is_clean(&self) -> bool;
}

pub trait ImageBitmapTrait: OriginIsCleanTrait {
    fn bitmap_data(&self) -> Ref<'_, Option<Snapshot>>;
}

pub trait ImageDataTrait {
    fn is_detached(&self, cx: &mut js::context::JSContext) -> bool;
    fn get_snapshot(&self, no_gc: &NoGC) -> Snapshot;
}

pub trait HtmlImageElementTrait {
    fn is_usable(&self) -> Result<bool, Error>;
    fn get_raster_image_data(&self) -> Option<Snapshot>;
    fn same_origin(&self, origin: &MutableOrigin) -> bool;
}

pub trait OffscreenCanvasTrait: OriginIsCleanTrait {
    fn get_image_data(&self) -> Option<Snapshot>;
}

pub trait HtmlCanvasElementTrait: OriginIsCleanTrait {
    fn is_valid(&self) -> bool;
    fn get_image_data(&self) -> Option<Snapshot>;
}
