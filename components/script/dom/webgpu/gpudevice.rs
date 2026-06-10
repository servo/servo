/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;
use std::cell::Cell;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::context::JSContext;
use js::jsapi::{HandleObject, Heap, JSObject};
use script_bindings::cell::DomRefCell;
use script_bindings::cformat;
use script_bindings::reflector::reflect_dom_object_with_cx;
use script_bindings::script_runtime::CanGc;
use webgpu_traits::{
    PopError, WebGPU, WebGPUComputePipeline, WebGPUComputePipelineResponse, WebGPUDevice,
    WebGPUPoppedErrorScopeResponse, WebGPUQueue, WebGPURenderPipeline,
    WebGPURenderPipelineResponse, WebGPURequest,
};
use wgpu_core::id::PipelineLayoutId;
use wgpu_core::pipeline as wgpu_pipe;
use wgpu_core::pipeline::RenderPipelineDescriptor;
use wgpu_types::{self, TextureFormat};

use super::gpudevicelostinfo::GPUDeviceLostInfo;
use super::gpuerror::AsWebGpu;
use super::gpupipelineerror::GPUPipelineError;
use super::gpusupportedlimits::GPUSupportedLimits;
use crate::conversions::Convert;
use crate::dom::bindings::codegen::Bindings::EventBinding::EventInit;
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::{
    GPUAdapterMethods, GPUBindGroupDescriptor, GPUBindGroupLayoutDescriptor, GPUBufferDescriptor,
    GPUCommandEncoderDescriptor, GPUComputePipelineDescriptor, GPUDeviceLostReason,
    GPUDeviceMethods, GPUErrorFilter, GPUPipelineErrorReason, GPUPipelineLayoutDescriptor,
    GPURenderBundleEncoderDescriptor, GPURenderPipelineDescriptor, GPUSamplerDescriptor,
    GPUShaderModuleDescriptor, GPUTextureDescriptor, GPUTextureFormat, GPUUncapturedErrorEventInit,
    GPUVertexStepMode,
};
use crate::dom::bindings::codegen::UnionTypes::GPUPipelineLayoutOrGPUAutoLayoutMode;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::USVString;
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::event::Event;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::types::GPUError;
use crate::dom::webgpu::gpuadapter::GPUAdapter;
use crate::dom::webgpu::gpuadapterinfo::GPUAdapterInfo;
use crate::dom::webgpu::gpubindgroup::GPUBindGroup;
use crate::dom::webgpu::gpubindgrouplayout::GPUBindGroupLayout;
use crate::dom::webgpu::gpubuffer::GPUBuffer;
use crate::dom::webgpu::gpucommandencoder::GPUCommandEncoder;
use crate::dom::webgpu::gpucomputepipeline::GPUComputePipeline;
use crate::dom::webgpu::gpupipelinelayout::GPUPipelineLayout;
use crate::dom::webgpu::gpuqueue::GPUQueue;
use crate::dom::webgpu::gpurenderbundleencoder::GPURenderBundleEncoder;
use crate::dom::webgpu::gpurenderpipeline::GPURenderPipeline;
use crate::dom::webgpu::gpusampler::GPUSampler;
use crate::dom::webgpu::gpushadermodule::GPUShaderModule;
use crate::dom::webgpu::gpusupportedfeatures::GPUSupportedFeatures;
use crate::dom::webgpu::gputexture::GPUTexture;
use crate::dom::webgpu::gpuuncapturederrorevent::GPUUncapturedErrorEvent;
use crate::realms::InRealm;
use crate::routed_promise::{RoutedPromiseListener, callback_promise};

#[derive(JSTraceable, MallocSizeOf)]
struct DroppableGPUDevice {
    #[no_trace]
    channel: WebGPU,
    #[no_trace]
    device: WebGPUDevice,
}

impl Drop for DroppableGPUDevice {
    fn drop(&mut self) {
        if let Err(e) = self
            .channel
            .0
            .send(WebGPURequest::DropDevice(self.device.0))
        {
            warn!("Failed to send DropDevice ({:?}) ({})", self.device.0, e);
        }
    }
}

#[dom_struct]
pub(crate) struct GPUDevice {
    eventtarget: EventTarget,
    adapter: Dom<GPUAdapter>,
    #[ignore_malloc_size_of = "mozjs"]
    extensions: Heap<*mut JSObject>,
    features: Dom<GPUSupportedFeatures>,
    limits: Dom<GPUSupportedLimits>,
    adapter_info: Dom<GPUAdapterInfo>,
    label: DomRefCell<USVString>,
    default_queue: Dom<GPUQueue>,
    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-lost>
    #[conditional_malloc_size_of]
    lost_promise: DomRefCell<Rc<Promise>>,
    valid: Cell<bool>,
    droppable: DroppableGPUDevice,
}

pub(crate) enum PipelineLayout {
    Implicit,
    Explicit(PipelineLayoutId),
}

impl PipelineLayout {
    pub(crate) fn explicit(&self) -> Option<PipelineLayoutId> {
        match self {
            PipelineLayout::Explicit(layout_id) => Some(*layout_id),
            PipelineLayout::Implicit => None,
        }
    }
}

impl GPUDevice {
    #[allow(clippy::too_many_arguments)]
    fn new_inherited(
        channel: WebGPU,
        adapter: &GPUAdapter,
        features: &GPUSupportedFeatures,
        limits: &GPUSupportedLimits,
        adapter_info: &GPUAdapterInfo,
        device: WebGPUDevice,
        queue: &GPUQueue,
        label: String,
        lost_promise: Rc<Promise>,
    ) -> Self {
        Self {
            eventtarget: EventTarget::new_inherited(),
            adapter: Dom::from_ref(adapter),
            extensions: Heap::default(),
            features: Dom::from_ref(features),
            limits: Dom::from_ref(limits),
            adapter_info: Dom::from_ref(adapter_info),
            label: DomRefCell::new(USVString::from(label)),
            default_queue: Dom::from_ref(queue),
            lost_promise: DomRefCell::new(lost_promise),
            valid: Cell::new(true),
            droppable: DroppableGPUDevice { channel, device },
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        channel: WebGPU,
        adapter: &GPUAdapter,
        extensions: HandleObject,
        features: wgpu_types::Features,
        limits: wgpu_types::Limits,
        device: WebGPUDevice,
        queue: WebGPUQueue,
        label: String,
    ) -> DomRoot<Self> {
        let queue = GPUQueue::new(cx, global, channel.clone(), queue);
        let limits = GPUSupportedLimits::new(cx, global, limits);
        let features = GPUSupportedFeatures::Constructor(cx, global, None, features).unwrap();
        let adapter_info = GPUAdapterInfo::clone_from(cx, global, &adapter.Info());
        let lost_promise = Promise::new2(cx, global);
        let device = reflect_dom_object_with_cx(
            Box::new(GPUDevice::new_inherited(
                channel,
                adapter,
                &features,
                &limits,
                &adapter_info,
                device,
                &queue,
                label,
                lost_promise,
            )),
            global,
            cx,
        );
        queue.set_device(&device);
        device.extensions.set(*extensions);
        device
    }
}

impl GPUDevice {
    pub(crate) fn id(&self) -> WebGPUDevice {
        self.droppable.device
    }

    pub(crate) fn queue_id(&self) -> WebGPUQueue {
        self.default_queue.id()
    }

    pub(crate) fn channel(&self) -> WebGPU {
        self.droppable.channel.clone()
    }

    pub(crate) fn dispatch_error(&self, error: webgpu_traits::Error) {
        if let Err(e) = self.droppable.channel.0.send(WebGPURequest::DispatchError {
            device_id: self.id().0,
            error,
        }) {
            warn!("Failed to send WebGPURequest::DispatchError due to {e:?}");
        }
    }

    /// <https://gpuweb.github.io/gpuweb/#eventdef-gpudevice-uncapturederror>
    pub(crate) fn fire_uncaptured_error(&self, error: webgpu_traits::Error) {
        let this = Trusted::new(self);

        // Queue a global task, using the webgpu task source, to fire an event named
        // uncapturederror at a GPUDevice using GPUUncapturedErrorEvent.
        self.global().task_manager().webgpu_task_source().queue(
            task!(fire_uncaptured_error: move |cx| {
                let this = this.root();
                let error = GPUError::from_error(cx, &this.global(), error);

                let event = GPUUncapturedErrorEvent::new(cx,
                    &this.global(),
                    atom!("uncapturederror"),
                    &GPUUncapturedErrorEventInit {
                        error,
                        parent: EventInit::empty(),
                    },
                );

                event.upcast::<Event>().fire(cx, this.upcast());
            }),
        );
    }

    /// <https://gpuweb.github.io/gpuweb/#abstract-opdef-validate-texture-format-required-features>
    ///
    /// Validates that the device suppports required features,
    /// and if so returns an ok containing wgpu's `TextureFormat`
    pub(crate) fn validate_texture_format_required_features(
        &self,
        format: &GPUTextureFormat,
    ) -> Fallible<TextureFormat> {
        let texture_format: TextureFormat = (*format).convert();
        if self
            .features
            .wgpu_features()
            .contains(texture_format.required_features())
        {
            Ok(texture_format)
        } else {
            Err(Error::Type(cformat!(
                "{texture_format:?} is not supported by this GPUDevice"
            )))
        }
    }

    pub(crate) fn is_lost(&self) -> bool {
        self.lost_promise.borrow().is_fulfilled()
    }

    pub(crate) fn get_pipeline_layout_data(
        &self,
        layout: &GPUPipelineLayoutOrGPUAutoLayoutMode,
    ) -> PipelineLayout {
        if let GPUPipelineLayoutOrGPUAutoLayoutMode::GPUPipelineLayout(layout) = layout {
            PipelineLayout::Explicit(layout.id().0)
        } else {
            PipelineLayout::Implicit
        }
    }

    pub(crate) fn parse_render_pipeline<'a>(
        &self,
        descriptor: &GPURenderPipelineDescriptor,
    ) -> Fallible<RenderPipelineDescriptor<'a>> {
        let pipeline_layout = self.get_pipeline_layout_data(&descriptor.parent.layout);
        let desc = wgpu_pipe::RenderPipelineDescriptor {
            label: (&descriptor.parent.parent).convert(),
            layout: pipeline_layout.explicit(),
            cache: None,
            vertex: wgpu_pipe::VertexState {
                stage: (&descriptor.vertex.parent).convert(),
                buffers: Cow::Owned(
                    descriptor
                        .vertex
                        .buffers
                        .iter()
                        .map(|buffer| wgpu_pipe::VertexBufferLayout {
                            array_stride: buffer.arrayStride,
                            step_mode: match buffer.stepMode {
                                GPUVertexStepMode::Vertex => wgpu_types::VertexStepMode::Vertex,
                                GPUVertexStepMode::Instance => wgpu_types::VertexStepMode::Instance,
                            },
                            attributes: Cow::Owned(
                                buffer
                                    .attributes
                                    .iter()
                                    .map(|att| wgpu_types::VertexAttribute {
                                        format: att.format.convert(),
                                        offset: att.offset,
                                        shader_location: att.shaderLocation,
                                    })
                                    .collect::<Vec<_>>(),
                            ),
                        })
                        .collect::<Vec<_>>(),
                ),
            },
            fragment: descriptor
                .fragment
                .as_ref()
                .map(|stage| -> Fallible<wgpu_pipe::FragmentState> {
                    Ok(wgpu_pipe::FragmentState {
                        stage: (&stage.parent).convert(),
                        targets: Cow::Owned(
                            stage
                                .targets
                                .iter()
                                .map(|state| {
                                    self.validate_texture_format_required_features(&state.format)
                                        .map(|format| {
                                            Some(wgpu_types::ColorTargetState {
                                                format,
                                                write_mask:
                                                    wgpu_types::ColorWrites::from_bits_retain(
                                                        state.writeMask,
                                                    ),
                                                blend: state.blend.as_ref().map(|blend| {
                                                    wgpu_types::BlendState {
                                                        color: (&blend.color).convert(),
                                                        alpha: (&blend.alpha).convert(),
                                                    }
                                                }),
                                            })
                                        })
                                })
                                .collect::<Result<Vec<_>, _>>()?,
                        ),
                    })
                })
                .transpose()?,
            primitive: (&descriptor.primitive).convert(),
            depth_stencil: descriptor
                .depthStencil
                .as_ref()
                .map(|dss_desc| {
                    self.validate_texture_format_required_features(&dss_desc.format)
                        .map(|format| wgpu_types::DepthStencilState {
                            format,
                            depth_write_enabled: dss_desc.depthWriteEnabled,
                            depth_compare: dss_desc.depthCompare.map(|dc| dc.convert()),
                            stencil: wgpu_types::StencilState {
                                front: wgpu_types::StencilFaceState {
                                    compare: dss_desc.stencilFront.compare.convert(),

                                    fail_op: dss_desc.stencilFront.failOp.convert(),
                                    depth_fail_op: dss_desc.stencilFront.depthFailOp.convert(),
                                    pass_op: dss_desc.stencilFront.passOp.convert(),
                                },
                                back: wgpu_types::StencilFaceState {
                                    compare: dss_desc.stencilBack.compare.convert(),
                                    fail_op: dss_desc.stencilBack.failOp.convert(),
                                    depth_fail_op: dss_desc.stencilBack.depthFailOp.convert(),
                                    pass_op: dss_desc.stencilBack.passOp.convert(),
                                },
                                read_mask: dss_desc.stencilReadMask,
                                write_mask: dss_desc.stencilWriteMask,
                            },
                            bias: wgpu_types::DepthBiasState {
                                constant: dss_desc.depthBias,
                                slope_scale: *dss_desc.depthBiasSlopeScale,
                                clamp: *dss_desc.depthBiasClamp,
                            },
                        })
                })
                .transpose()?,
            multisample: wgpu_types::MultisampleState {
                count: descriptor.multisample.count,
                mask: descriptor.multisample.mask as u64,
                alpha_to_coverage_enabled: descriptor.multisample.alphaToCoverageEnabled,
            },
            multiview_mask: None,
        };
        Ok(desc)
    }

    /// <https://gpuweb.github.io/gpuweb/#lose-the-device>
    pub(crate) fn lose(&self, reason: GPUDeviceLostReason, msg: String) {
        let this = Trusted::new(self);

        // Queue a global task, using the webgpu task source, to resolve device.lost
        // promise with a new GPUDeviceLostInfo with reason and message.
        self.global().task_manager().webgpu_task_source().queue(
            task!(resolve_device_lost: move || {
                let this = this.root();

                let lost_promise = &(*this.lost_promise.borrow());
                let lost = GPUDeviceLostInfo::new(&this.global(), msg.into(), reason, CanGc::deprecated_note());
                lost_promise.resolve_native(&*lost, CanGc::deprecated_note());
            }),
        );
    }
}

impl GPUDeviceMethods<crate::DomTypeHolder> for GPUDevice {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-features>
    fn Features(&self) -> DomRoot<GPUSupportedFeatures> {
        DomRoot::from_ref(&self.features)
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-limits>
    fn Limits(&self) -> DomRoot<GPUSupportedLimits> {
        DomRoot::from_ref(&self.limits)
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-adapterinfo>
    fn AdapterInfo(&self) -> DomRoot<GPUAdapterInfo> {
        DomRoot::from_ref(&self.adapter_info)
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-queue>
    fn GetQueue(&self) -> DomRoot<GPUQueue> {
        DomRoot::from_ref(&self.default_queue)
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn Label(&self) -> USVString {
        self.label.borrow().clone()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn SetLabel(&self, value: USVString) {
        *self.label.borrow_mut() = value;
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-lost>
    fn Lost(&self) -> Rc<Promise> {
        self.lost_promise.borrow().clone()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createbuffer>
    fn CreateBuffer(
        &self,
        cx: &mut JSContext,
        descriptor: &GPUBufferDescriptor,
    ) -> Fallible<DomRoot<GPUBuffer>> {
        GPUBuffer::create(cx, self, descriptor)
    }

    /// <https://gpuweb.github.io/gpuweb/#GPUDevice-createBindGroupLayout>
    fn CreateBindGroupLayout(
        &self,
        cx: &mut JSContext,
        descriptor: &GPUBindGroupLayoutDescriptor,
    ) -> Fallible<DomRoot<GPUBindGroupLayout>> {
        GPUBindGroupLayout::create(cx, self, descriptor)
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createpipelinelayout>
    fn CreatePipelineLayout(
        &self,
        cx: &mut JSContext,
        descriptor: &GPUPipelineLayoutDescriptor,
    ) -> DomRoot<GPUPipelineLayout> {
        GPUPipelineLayout::create(cx, self, descriptor)
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createbindgroup>
    fn CreateBindGroup(
        &self,
        cx: &mut JSContext,
        descriptor: &GPUBindGroupDescriptor,
    ) -> DomRoot<GPUBindGroup> {
        GPUBindGroup::create(cx, self, descriptor)
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createshadermodule>
    fn CreateShaderModule(
        &self,
        descriptor: RootedTraceableBox<GPUShaderModuleDescriptor>,
        comp: InRealm,
    ) -> DomRoot<GPUShaderModule> {
        GPUShaderModule::create(self, descriptor, comp, CanGc::deprecated_note())
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createcomputepipeline>
    fn CreateComputePipeline(
        &self,
        cx: &mut JSContext,
        descriptor: &GPUComputePipelineDescriptor,
    ) -> DomRoot<GPUComputePipeline> {
        let compute_pipeline = GPUComputePipeline::create(self, descriptor, None);
        GPUComputePipeline::new(
            cx,
            &self.global(),
            compute_pipeline,
            descriptor.parent.parent.label.clone(),
            self,
        )
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createcomputepipelineasync>
    fn CreateComputePipelineAsync(
        &self,
        descriptor: &GPUComputePipelineDescriptor,
        comp: InRealm,
    ) -> Rc<Promise> {
        let promise = Promise::new_in_current_realm(comp, CanGc::deprecated_note());
        let callback = callback_promise(
            &promise,
            self,
            self.global().task_manager().dom_manipulation_task_source(),
        );
        GPUComputePipeline::create(self, descriptor, Some(callback));
        promise
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createcommandencoder>
    fn CreateCommandEncoder(
        &self,
        cx: &mut JSContext,
        descriptor: &GPUCommandEncoderDescriptor,
    ) -> DomRoot<GPUCommandEncoder> {
        GPUCommandEncoder::create(cx, self, descriptor)
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createtexture>
    fn CreateTexture(
        &self,
        cx: &mut JSContext,
        descriptor: &GPUTextureDescriptor,
    ) -> Fallible<DomRoot<GPUTexture>> {
        GPUTexture::create(cx, self, descriptor)
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createsampler>
    fn CreateSampler(
        &self,
        cx: &mut JSContext,
        descriptor: &GPUSamplerDescriptor,
    ) -> DomRoot<GPUSampler> {
        GPUSampler::create(cx, self, descriptor)
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createrenderpipeline>
    fn CreateRenderPipeline(
        &self,
        cx: &mut JSContext,
        descriptor: &GPURenderPipelineDescriptor,
    ) -> Fallible<DomRoot<GPURenderPipeline>> {
        let desc = self.parse_render_pipeline(descriptor)?;
        let render_pipeline = GPURenderPipeline::create(self, desc, None)?;
        Ok(GPURenderPipeline::new(
            cx,
            &self.global(),
            render_pipeline,
            descriptor.parent.parent.label.clone(),
            self,
        ))
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createrenderpipelineasync>
    fn CreateRenderPipelineAsync(
        &self,
        descriptor: &GPURenderPipelineDescriptor,
        comp: InRealm,
    ) -> Fallible<Rc<Promise>> {
        let desc = self.parse_render_pipeline(descriptor)?;
        let promise = Promise::new_in_current_realm(comp, CanGc::deprecated_note());
        let callback = callback_promise(
            &promise,
            self,
            self.global().task_manager().dom_manipulation_task_source(),
        );
        GPURenderPipeline::create(self, desc, Some(callback))?;
        Ok(promise)
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createrenderbundleencoder>
    fn CreateRenderBundleEncoder(
        &self,
        cx: &mut JSContext,
        descriptor: &GPURenderBundleEncoderDescriptor,
    ) -> Fallible<DomRoot<GPURenderBundleEncoder>> {
        GPURenderBundleEncoder::create(cx, self, descriptor)
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-pusherrorscope>
    fn PushErrorScope(&self, filter: GPUErrorFilter) {
        if self
            .droppable
            .channel
            .0
            .send(WebGPURequest::PushErrorScope {
                device_id: self.id().0,
                filter: filter.as_webgpu(),
            })
            .is_err()
        {
            warn!("Failed sending WebGPURequest::PushErrorScope");
        }
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-poperrorscope>
    fn PopErrorScope(&self, comp: InRealm) -> Rc<Promise> {
        let promise = Promise::new_in_current_realm(comp, CanGc::deprecated_note());
        let callback = callback_promise(
            &promise,
            self,
            self.global().task_manager().dom_manipulation_task_source(),
        );
        if self
            .droppable
            .channel
            .0
            .send(WebGPURequest::PopErrorScope {
                device_id: self.id().0,
                callback,
            })
            .is_err()
        {
            warn!("Error when sending WebGPURequest::PopErrorScope");
        }
        promise
    }

    // https://gpuweb.github.io/gpuweb/#dom-gpudevice-onuncapturederror
    event_handler!(uncapturederror, GetOnuncapturederror, SetOnuncapturederror);

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-destroy>
    fn Destroy(&self) {
        if self.valid.get() {
            self.valid.set(false);

            if let Err(e) = self
                .droppable
                .channel
                .0
                .send(WebGPURequest::DestroyDevice(self.id().0))
            {
                warn!("Failed to send DestroyDevice ({:?}) ({})", self.id().0, e);
            }
        }
    }
}

impl RoutedPromiseListener<WebGPUPoppedErrorScopeResponse> for GPUDevice {
    fn handle_response(
        &self,
        cx: &mut js::context::JSContext,
        response: WebGPUPoppedErrorScopeResponse,
        promise: &Rc<Promise>,
    ) {
        match response {
            Ok(None) | Err(PopError::Lost) => {
                promise.resolve_native_with_cx(cx, &None::<Option<GPUError>>)
            },
            Err(PopError::Empty) => promise.reject_error_with_cx(cx, Error::Operation(None)),
            Ok(Some(error)) => {
                let error = GPUError::from_error(cx, &self.global(), error);
                promise.resolve_native_with_cx(cx, &error);
            },
        }
    }
}

impl RoutedPromiseListener<WebGPUComputePipelineResponse> for GPUDevice {
    fn handle_response(
        &self,
        cx: &mut js::context::JSContext,
        response: WebGPUComputePipelineResponse,
        promise: &Rc<Promise>,
    ) {
        match response {
            Ok(pipeline) => {
                let gpu_compute_pipeline = GPUComputePipeline::new(
                    cx,
                    &self.global(),
                    WebGPUComputePipeline(pipeline.id),
                    pipeline.label.into(),
                    self,
                );
                promise.resolve_native_with_cx(cx, &gpu_compute_pipeline)
            },
            Err(webgpu_traits::Error::Validation(msg)) => {
                let gpu_pipeline_error = GPUPipelineError::new(
                    cx,
                    &self.global(),
                    msg.into(),
                    GPUPipelineErrorReason::Validation,
                );
                promise.reject_native_with_cx(cx, &gpu_pipeline_error)
            },
            Err(webgpu_traits::Error::OutOfMemory(msg) | webgpu_traits::Error::Internal(msg)) => {
                let gpu_pipeline_error = GPUPipelineError::new(
                    cx,
                    &self.global(),
                    msg.into(),
                    GPUPipelineErrorReason::Internal,
                );
                promise.reject_native_with_cx(cx, &gpu_pipeline_error)
            },
        }
    }
}

impl RoutedPromiseListener<WebGPURenderPipelineResponse> for GPUDevice {
    fn handle_response(
        &self,
        cx: &mut js::context::JSContext,
        response: WebGPURenderPipelineResponse,
        promise: &Rc<Promise>,
    ) {
        match response {
            Ok(pipeline) => {
                let gpu_pipeline = GPURenderPipeline::new(
                    cx,
                    &self.global(),
                    WebGPURenderPipeline(pipeline.id),
                    pipeline.label.into(),
                    self,
                );
                promise.resolve_native_with_cx(cx, &gpu_pipeline)
            },
            Err(webgpu_traits::Error::Validation(msg)) => {
                let pipeline_error = GPUPipelineError::new(
                    cx,
                    &self.global(),
                    msg.into(),
                    GPUPipelineErrorReason::Validation,
                );

                promise.reject_native_with_cx(cx, &pipeline_error)
            },
            Err(webgpu_traits::Error::OutOfMemory(msg) | webgpu_traits::Error::Internal(msg)) => {
                let pipeline_error = GPUPipelineError::new(
                    cx,
                    &self.global(),
                    msg.into(),
                    GPUPipelineErrorReason::Internal,
                );
                promise.reject_native_with_cx(cx, &pipeline_error)
            },
        }
    }
}
