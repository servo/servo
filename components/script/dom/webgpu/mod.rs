/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;
use std::sync::Arc;

use euclid::default::Size2D;
use script_bindings::DomTypes;
use script_bindings::callback::CallbackContainer;
use script_bindings::error::Fallible;
use script_bindings::interfaces::PromiseHelpers;
use script_bindings::reflector::{DomGlobalGeneric, DomObject};
use script_bindings::root::DomRoot;
use script_webgpu::traits::{
    EventTargetTrait, GPUQueueTrait, WebGPUGlobalTrait, WebGPUHTMLVideoTrait,
    WebGPUPromiseCallbackTrait, WebGPURootedPromiseTrait, WebGPUTracedPromiseTrait,
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use servo_base::generic_channel::GenericCallback;

use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::promise::RootedPromise;
use crate::dom::types::{EventTarget, GPUDevice, GPUQueue, HTMLVideoElement};
use crate::dom::{GlobalScope, Promise};
use crate::routed_promise::{RoutedPromiseListener, callback_promise};
use crate::tasks::task::TaskOnce;

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
pub(crate) mod gpudevice_promise_listener;
pub(crate) mod gpudevice {
    pub(crate) type GPUDevice = script_webgpu::gpudevice::GPUDevice<crate::DomTypeHolder>;
}
pub(crate) mod gpudevicelostinfo {
    pub(crate) type GPUDeviceLostInfo =
        script_webgpu::gpudevicelostinfo::GPUDeviceLostInfo<crate::DomTypeHolder>;
}
pub(crate) mod gpuerror {
    pub(crate) type GPUError = script_webgpu::gpuerror::GPUError<crate::DomTypeHolder>;
}
pub(crate) mod gpuexternaltexture {
    pub(crate) type GPUExternalTexture =
        script_webgpu::gpuexternaltexture::GPUExternalTexture<crate::DomTypeHolder>;
    pub(crate) type PlanarTexture =
        script_webgpu::gpuexternaltexture::PlanarTexture<crate::DomTypeHolder>;
}
pub(crate) mod gpuinternalerror {
    pub(crate) type GPUInternalError =
        script_webgpu::gpuinternalerror::GPUInternalError<crate::DomTypeHolder>;
}
pub(crate) mod gpumapmode {
    pub(crate) type GPUMapMode = script_webgpu::gpumapmode::GPUMapMode<crate::DomTypeHolder>;
}
pub(crate) mod gpuoutofmemoryerror {
    pub(crate) type GPUOutOfMemoryError =
        script_webgpu::gpuoutofmemoryerror::GPUOutOfMemoryError<crate::DomTypeHolder>;
}
pub(crate) mod gpupipelineerror {
    pub(crate) type GPUPipelineError =
        script_webgpu::gpupipelineerror::GPUPipelineError<crate::DomTypeHolder>;
}
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
pub(crate) mod gpuuncapturederrorevent {
    pub(crate) type GPUUncapturedErrorEvent =
        script_webgpu::gpuuncapturederrorevent::GPUUncapturedErrorEvent<crate::DomTypeHolder>;
}
pub(crate) mod gpuvalidationerror {
    pub(crate) type GPUValidationError =
        script_webgpu::gpuvalidationerror::GPUValidationError<crate::DomTypeHolder>;
}
pub(crate) mod identityhub {
    pub(crate) type IdentityHub = script_webgpu::identityhub::IdentityHub;
}
pub(crate) mod wgsllanguagefeatures {
    pub(crate) type WGSLLanguageFeatures =
        script_webgpu::wgsllanguagefeatures::WGSLLanguageFeatures<crate::DomTypeHolder>;
}

impl<S, T> WebGPUPromiseCallbackTrait<crate::DomTypeHolder, S, T> for RootedPromise
where
    S: DomObject + DomGlobalGeneric<crate::DomTypeHolder> + RoutedPromiseListener<T>,
    T: Serialize + 'static + Send + DeserializeOwned,
{
    fn callback_promise_dom_manipulation_task_source(&self, d: &S) -> GenericCallback<T> {
        let task_manager = <S as DomGlobal>::global(d).task_manager();
        callback_promise(self, d, task_manager.dom_manipulation_task_source())
    }
}

impl WebGPUTracedPromiseTrait<crate::DomTypeHolder> for Promise {
    fn is_fulfilled(&self) -> bool {
        Promise::is_fulfilled(self)
    }
}

impl WebGPURootedPromiseTrait<crate::DomTypeHolder> for RootedPromise {
    fn new_rooted(
        cx: &mut js::context::JSContext,
        global: &<crate::DomTypeHolder as DomTypes>::GlobalScope,
    ) -> <<crate::DomTypeHolder as DomTypes>::Promise as PromiseHelpers<crate::DomTypeHolder>>::StackRoot{
        Promise::new_rooted(cx, global)
    }
}

impl GPUQueueTrait<crate::DomTypeHolder> for GPUQueue {
    fn new(
        cx: &mut js::context::JSContext,
        global: &GlobalScope,
        channel: webgpu_traits::WebGPU,
        queue: webgpu_traits::WebGPUQueue,
    ) -> DomRoot<GPUQueue> {
        GPUQueue::new(cx, global, channel, queue)
    }

    fn id(&self) -> webgpu_traits::WebGPUQueue {
        GPUQueue::id(self)
    }

    fn set_device(&self, cx: &mut js::context::JSContext, device: &GPUDevice) {
        GPUQueue::set_device(self, cx, device);
    }
}

struct WebGPUTaskSource<F: FnOnce(&mut js::context::JSContext)> {
    _name: &'static str,
    task_fn: F,
}

impl<F: FnOnce(&mut js::context::JSContext) + Send> TaskOnce for WebGPUTaskSource<F> {
    fn run_once(self, cx: &mut js::context::JSContext) {
        (self.task_fn)(cx)
    }
}

impl WebGPUGlobalTrait for GlobalScope {
    fn global_wgpu_id_hub(&self) -> Arc<script_webgpu::identityhub::IdentityHub> {
        self.wgpu_id_hub()
    }

    fn queue_webgpu_task_source<F: FnOnce(&mut js::context::JSContext) + Send + 'static>(
        &self,
        name: &'static str,
        task_fn: F,
    ) {
        self.task_manager()
            .webgpu_task_source()
            .queue(WebGPUTaskSource {
                _name: name,
                task_fn,
            });
    }
}

impl WebGPUHTMLVideoTrait<crate::DomTypeHolder> for HTMLVideoElement {
    fn planar_video_for_webgpu(
        &self,
        device: &script_webgpu::gpudevice::GPUDevice<crate::DomTypeHolder>,
    ) -> Fallible<(
        Size2D<u32>,
        Option<Rc<script_webgpu::gpuexternaltexture::PlanarTexture<crate::DomTypeHolder>>>,
    )> {
        HTMLVideoElement::planar_video_for_webgpu(self, device)
    }
}

impl EventTargetTrait<crate::DomTypeHolder> for EventTarget {
    fn new_inherited() -> <crate::DomTypeHolder as DomTypes>::EventTarget {
        EventTarget::new_inherited()
    }

    fn get_event_handler_common<T: CallbackContainer<crate::DomTypeHolder>>(
        &self,
        cx: &mut js::context::JSContext,
        ty: &str,
    ) -> Option<std::rc::Rc<T>> {
        EventTarget::get_event_handler_common(self, cx, ty)
    }

    fn set_event_handler_common<T: CallbackContainer<crate::DomTypeHolder>>(
        &self,
        cx: &mut js::context::JSContext,
        ty: &str,
        listener: Option<Rc<T>>,
    ) {
        EventTarget::set_event_handler_common(self, cx, ty, listener);
    }
}
