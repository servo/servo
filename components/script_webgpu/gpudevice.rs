/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;
use std::cell::Cell;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::context::{JSContext, NoGC};
use js::jsapi::{HandleObject, Heap, JSObject};
use js::realm::CurrentRealm;
use jstraceable_derive::JSTraceable;
use log::warn;
use malloc_size_of_derive::MallocSizeOf;
use script_bindings::callback::CallbackContainer;
use script_bindings::cell::DomRefCell;
use script_bindings::codegen::GenericBindings::EventBinding::EventInit;
use script_bindings::codegen::GenericBindings::EventHandlerBinding::EventHandlerNonNull;
use script_bindings::codegen::GenericBindings::WebGPUBinding::{
    GPUAdapterMethods, GPUBindGroupDescriptor, GPUBindGroupLayoutDescriptor, GPUBufferDescriptor,
    GPUCommandEncoderDescriptor, GPUComputePipelineDescriptor, GPUDeviceLostReason,
    GPUDeviceMethods, GPUDeviceWrap, GPUErrorFilter, GPUExternalTextureDescriptor,
    GPUPipelineLayoutDescriptor, GPUQuerySetDescriptor, GPURenderBundleEncoderDescriptor,
    GPURenderPipelineDescriptor, GPUSamplerDescriptor, GPUShaderModuleDescriptor,
    GPUTextureDescriptor, GPUTextureFormat, GPUUncapturedErrorEventInit, GPUVertexStepMode,
};
use script_bindings::codegen::GenericUnionTypes::GPUPipelineLayoutOrGPUAutoLayoutMode;
use script_bindings::error::Error;
use script_bindings::inheritance::Castable;
use script_bindings::interfaces::{
    HeapTracedPromiseHelpers, PromiseHelpers, StackRootPromiseHelpers,
};
use script_bindings::reflector::{
    DomGlobalGeneric, reflect_weak_referenceable_dom_object_with_cx_and_wrap,
};
use script_bindings::traits::DomEventTrait;
use script_bindings::{DomTypes, cformat};
use stylo_atoms::atom;
use webgpu_traits::{WebGPU, WebGPUDevice, WebGPUQueue, WebGPURequest};
use wgpu_core::pipeline as wgpu_pipe;
use wgpu_core::pipeline::RenderPipelineDescriptor;
use wgpu_types::{self, TextureFormat};

use super::gpudevicelostinfo::GPUDeviceLostInfo;
use crate::PipelineLayout;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::USVString;
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::gpuadapter::GPUAdapter;
use crate::gpuadapterinfo::GPUAdapterInfo;
use crate::gpubindgroup::GPUBindGroup;
use crate::gpubindgrouplayout::GPUBindGroupLayout;
use crate::gpubuffer::GPUBuffer;
use crate::gpucommandencoder::GPUCommandEncoder;
use crate::gpucomputepipeline::GPUComputePipeline;
use crate::gpuconvert::WebGPUConvert;
use crate::gpuerror::{AsWebGpu, GPUError};
use crate::gpuexternaltexture::GPUExternalTexture;
use crate::gpupipelinelayout::GPUPipelineLayout;
use crate::gpuqueryset::GPUQuerySet;
use crate::gpurenderbundleencoder::GPURenderBundleEncoder;
use crate::gpurenderpipeline::GPURenderPipeline;
use crate::gpusampler::GPUSampler;
use crate::gpushadermodule::GPUShaderModule;
use crate::gpusupportedfeatures::GPUSupportedFeatures;
use crate::gpusupportedlimits::GPUSupportedLimits;
use crate::gputexture::GPUTexture;
use crate::gpuuncapturederrorevent::GPUUncapturedErrorEvent;
use crate::traits::{
    Equivalence, EventTargetTrait, GPUQueueTrait, WebGPUGlobalTrait, WebGPUPromise,
    WebGPUPromiseCallbackTrait, WebGPURootedPromiseTrait, WebGPUTracedPromiseTrait,
};

macro_rules! event_handler(
    ($event_type: ident, $getter: ident, $setter: ident) => (
        define_event_handler!(
            script_bindings::codegen::GenericBindings::EventHandlerBinding::EventHandlerNonNull<D>,
            $event_type,
            $getter,
            $setter,
            set_event_handler_common
        );
    )
);

/// These are used to generate a event handler which has no special case.
macro_rules! define_event_handler(
    ($handler: ty, $event_type: ident, $getter: ident, $setter: ident, $setter_fn: ident) => (
        fn $getter(&self, cx: &mut js::context::JSContext) -> Option<::std::rc::Rc<$handler>> {
            use crate::dom::bindings::inheritance::Castable;
            let eventtarget = self.upcast::<D::EventTarget>();
            D::EventTarget::get_event_handler_common(eventtarget, cx, stringify!($event_type))
        }

        fn $setter(&self, cx: &mut js::context::JSContext, listener: Option<::std::rc::Rc<$handler>>) {
            use crate::dom::bindings::inheritance::Castable;
            let eventtarget = self.upcast::<D::EventTarget>();
            eventtarget.$setter_fn(cx, stringify!($event_type), listener)
        }
    )
);

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
pub struct GPUDevice<D: DomTypes> {
    eventtarget: D::EventTarget,
    adapter: Dom<GPUAdapter<D>>,
    #[ignore_malloc_size_of = "mozjs"]
    extensions: Heap<*mut JSObject>,
    features: Dom<GPUSupportedFeatures<D>>,
    limits: Dom<GPUSupportedLimits<D>>,
    adapter_info: Dom<GPUAdapterInfo<D>>,
    label: DomRefCell<USVString>,
    default_queue: Dom<D::GPUQueue>,
    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-lost>
    lost_promise: DomRefCell<<D::Promise as PromiseHelpers<D>>::HeapTraced>,
    valid: Cell<bool>,
    droppable: DroppableGPUDevice,
}

impl<D> GPUDevice<D>
where
    D: Equivalence,
    <D::Promise as PromiseHelpers<D>>::StackRoot: WebGPUPromise<D>,
    EventHandlerNonNull<D>: CallbackContainer<D>,
{
    #[allow(clippy::too_many_arguments)]
    fn new_inherited(
        channel: WebGPU,
        adapter: &GPUAdapter<D>,
        features: &GPUSupportedFeatures<D>,
        limits: &GPUSupportedLimits<D>,
        adapter_info: &GPUAdapterInfo<D>,
        device: WebGPUDevice,
        queue: &D::GPUQueue,
        label: String,
        lost_promise: &<D::Promise as PromiseHelpers<D>>::StackRoot,
    ) -> Self {
        Self {
            eventtarget: D::EventTarget::new_inherited(),
            adapter: Dom::from_ref(adapter),
            extensions: Heap::default(),
            features: Dom::from_ref(features),
            limits: Dom::from_ref(limits),
            adapter_info: Dom::from_ref(adapter_info),
            label: DomRefCell::new(USVString::from(label)),
            default_queue: Dom::from_ref(queue),
            lost_promise: DomRefCell::new(lost_promise.to_traced()),
            valid: Cell::new(true),
            droppable: DroppableGPUDevice { channel, device },
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        cx: &mut JSContext,
        global: &D::GlobalScope,
        channel: WebGPU,
        adapter: &GPUAdapter<D>,
        extensions: HandleObject,
        features: wgpu_types::Features,
        limits: wgpu_types::Limits,
        device: WebGPUDevice,
        queue: WebGPUQueue,
        label: String,
    ) -> DomRoot<Self> {
        let queue = D::GPUQueue::new(cx, global, channel.clone(), queue);
        let limits = GPUSupportedLimits::new(cx, global, limits);
        let features = GPUSupportedFeatures::Constructor(cx, global, None, features).unwrap();
        let adapter_info = GPUAdapterInfo::clone_from(cx, global, &adapter.Info());
        let lost_promise = <D::Promise as PromiseHelpers<D>>::StackRoot::new_rooted(cx, global);
        let device = reflect_weak_referenceable_dom_object_with_cx_and_wrap::<D, _, _>(
            cx,
            Rc::new(GPUDevice::new_inherited(
                channel,
                adapter,
                &features,
                &limits,
                &adapter_info,
                device,
                &queue,
                label,
                &lost_promise,
            )),
            global,
            GPUDeviceWrap::<D>,
        );
        queue.set_device(cx, &device);
        device.extensions.set(*extensions);
        device
    }
}

impl<D> GPUDevice<D>
where
    D: Equivalence,
    <D::Promise as PromiseHelpers<D>>::StackRoot: WebGPUPromise<D>,
    EventHandlerNonNull<D>: CallbackContainer<D>,
{
    pub fn id(&self) -> WebGPUDevice {
        self.droppable.device
    }

    pub fn queue_id(&self) -> WebGPUQueue {
        self.default_queue.id()
    }

    pub fn channel(&self) -> WebGPU {
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
    pub fn fire_uncaptured_error(&self, error: webgpu_traits::Error) {
        let this = Trusted::new(self);

        // Queue a global task, using the webgpu task source, to fire an event named
        // uncapturederror at a GPUDevice using GPUUncapturedErrorEvent.
        self.global_from_reflector()
            .queue_webgpu_task_source("fire_uncaptured_error", move |cx| {
                let this = this.root();
                let error = GPUError::from_error(cx, &*this.global_from_reflector(), error);

                let event = GPUUncapturedErrorEvent::new(
                    cx,
                    &*this.global_from_reflector(),
                    atom!("uncapturederror"),
                    &GPUUncapturedErrorEventInit::<D> {
                        error,
                        parent: EventInit::empty(),
                    },
                );

                event.upcast::<D::Event>().fire(cx, this.upcast());
            });
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
        layout: &GPUPipelineLayoutOrGPUAutoLayoutMode<D>,
    ) -> PipelineLayout {
        if let GPUPipelineLayoutOrGPUAutoLayoutMode::GPUPipelineLayout(layout) = layout {
            PipelineLayout::Explicit(layout.id().0)
        } else {
            PipelineLayout::Implicit
        }
    }

    pub(crate) fn parse_render_pipeline<'a>(
        &self,
        descriptor: &GPURenderPipelineDescriptor<D>,
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
                        // FIXME: webidl has `sequence<GPUVertexBufferLayout?> buffers`
                        // but we get no option here so it must be eaten by codegen
                        .map(|buffer| {
                            Some(wgpu_pipe::VertexBufferLayout {
                                array_stride: buffer.arrayStride,
                                step_mode: match buffer.stepMode {
                                    GPUVertexStepMode::Vertex => wgpu_types::VertexStepMode::Vertex,
                                    GPUVertexStepMode::Instance => {
                                        wgpu_types::VertexStepMode::Instance
                                    },
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
    pub fn lose(&self, reason: GPUDeviceLostReason, msg: String) {
        let this = Trusted::new(self);

        // Queue a global task, using the webgpu task source, to resolve device.lost
        // promise with a new GPUDeviceLostInfo with reason and message.
        let global: DomRoot<D::GlobalScope> = self.global_from_reflector();
        global.queue_webgpu_task_source("resolve_device_lost", move |cx| {
            let this = this.root();

            let lost_promise = &(*this.lost_promise.borrow());
            let lost =
                GPUDeviceLostInfo::<D>::new(cx, &*this.global_from_reflector(), msg.into(), reason);
            lost_promise.resolve_native(cx, &*lost);
        });
    }
}

impl<D> GPUDeviceMethods<D> for GPUDevice<D>
where
    D: Equivalence,
    <D::Promise as PromiseHelpers<D>>::StackRoot: WebGPUPromise<D>,
    <D::Promise as PromiseHelpers<D>>::HeapTraced: HeapTracedPromiseHelpers<D>,
{
    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-features>
    fn Features(&self) -> DomRoot<GPUSupportedFeatures<D>> {
        DomRoot::from_ref(&self.features)
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-limits>
    fn Limits(&self) -> DomRoot<GPUSupportedLimits<D>> {
        DomRoot::from_ref(&*self.limits)
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-adapterinfo>
    fn AdapterInfo(&self) -> DomRoot<GPUAdapterInfo<D>> {
        DomRoot::from_ref(&self.adapter_info)
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-queue>
    fn GetQueue(&self) -> DomRoot<D::GPUQueue> {
        DomRoot::from_ref(&self.default_queue)
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn Label(&self) -> USVString {
        self.label.borrow().clone()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn SetLabel(&self, no_gc: &NoGC, value: USVString) {
        *self.label.safe_borrow_mut(no_gc) = value;
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-lost>
    fn Lost(&self) -> <<D as script_bindings::DomTypes>::Promise as script_bindings::interfaces::PromiseHelpers<D>>::StackRoot{
        self.lost_promise.borrow().root()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createbuffer>
    fn CreateBuffer(
        &self,
        cx: &mut JSContext,
        descriptor: &GPUBufferDescriptor,
    ) -> Fallible<DomRoot<GPUBuffer<D>>> {
        GPUBuffer::create(cx, self, descriptor)
    }

    /// <https://gpuweb.github.io/gpuweb/#GPUDevice-createBindGroupLayout>
    fn CreateBindGroupLayout(
        &self,
        cx: &mut JSContext,
        descriptor: &GPUBindGroupLayoutDescriptor,
    ) -> Fallible<DomRoot<GPUBindGroupLayout<D>>> {
        GPUBindGroupLayout::create(cx, self, descriptor)
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createpipelinelayout>
    fn CreatePipelineLayout(
        &self,
        cx: &mut JSContext,
        descriptor: &GPUPipelineLayoutDescriptor<D>,
    ) -> DomRoot<GPUPipelineLayout<D>> {
        GPUPipelineLayout::create(cx, self, descriptor)
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createbindgroup>
    fn CreateBindGroup(
        &self,
        cx: &mut JSContext,
        descriptor: &GPUBindGroupDescriptor<D>,
    ) -> DomRoot<GPUBindGroup<D>> {
        GPUBindGroup::create(cx, self, descriptor)
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createshadermodule>
    fn CreateShaderModule(
        &self,
        cx: &mut CurrentRealm<'_>,
        descriptor: RootedTraceableBox<GPUShaderModuleDescriptor>,
    ) -> DomRoot<GPUShaderModule<D>> {
        GPUShaderModule::create(cx, self, descriptor)
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createcomputepipeline>
    fn CreateComputePipeline(
        &self,
        cx: &mut JSContext,
        descriptor: &GPUComputePipelineDescriptor<D>,
    ) -> DomRoot<GPUComputePipeline<D>> {
        let compute_pipeline = GPUComputePipeline::create(self, descriptor, None);
        GPUComputePipeline::new(
            cx,
            &*self.global_from_reflector(),
            compute_pipeline,
            descriptor.parent.parent.label.clone(),
            self,
        )
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createcomputepipelineasync>
    fn CreateComputePipelineAsync(
        &self,
        cx: &mut CurrentRealm<'_>,
        descriptor: &GPUComputePipelineDescriptor<D>,
    ) -> <<D as script_bindings::DomTypes>::Promise as script_bindings::interfaces::PromiseHelpers<D>>::StackRoot{
        let promise = D::Promise::new_in_realm_rooted(cx);
        let callback =
            <D::Promise as PromiseHelpers<D>>::StackRoot::callback_promise_dom_manipulation_task_source(&promise, self);
        GPUComputePipeline::create(self, descriptor, Some(callback));
        promise
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createcommandencoder>
    fn CreateCommandEncoder(
        &self,
        cx: &mut JSContext,
        descriptor: &GPUCommandEncoderDescriptor,
    ) -> DomRoot<GPUCommandEncoder<D>> {
        GPUCommandEncoder::create(cx, self, descriptor)
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createtexture>
    fn CreateTexture(
        &self,
        cx: &mut JSContext,
        descriptor: &GPUTextureDescriptor,
    ) -> Fallible<DomRoot<GPUTexture<D>>> {
        GPUTexture::create(cx, self, descriptor)
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createsampler>
    fn CreateSampler(
        &self,
        cx: &mut JSContext,
        descriptor: &GPUSamplerDescriptor,
    ) -> DomRoot<GPUSampler<D>> {
        GPUSampler::create(cx, self, descriptor)
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createrenderpipeline>
    fn CreateRenderPipeline(
        &self,
        cx: &mut JSContext,
        descriptor: &GPURenderPipelineDescriptor<D>,
    ) -> Fallible<DomRoot<GPURenderPipeline<D>>> {
        let desc = self.parse_render_pipeline(descriptor)?;
        let render_pipeline = GPURenderPipeline::create(self, desc, None)?;
        Ok(GPURenderPipeline::new(
            cx,
            &*self.global_from_reflector(),
            render_pipeline,
            descriptor.parent.parent.label.clone(),
            self,
        ))
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createrenderpipelineasync>
    fn CreateRenderPipelineAsync(
        &self,
        cx: &mut CurrentRealm<'_>,
        descriptor: &GPURenderPipelineDescriptor<D>,
    ) -> Fallible<<<D as script_bindings::DomTypes>::Promise as script_bindings::interfaces::PromiseHelpers<D>>::StackRoot>{
        let desc = self.parse_render_pipeline(descriptor)?;
        let promise = D::Promise::new_in_realm_rooted(cx);
        let callback = <D::Promise as PromiseHelpers<D>>::StackRoot::callback_promise_dom_manipulation_task_source(
            &promise, self,
        );
        GPURenderPipeline::create(self, desc, Some(callback))?;
        Ok(promise)
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createrenderbundleencoder>
    fn CreateRenderBundleEncoder(
        &self,
        cx: &mut JSContext,
        descriptor: &GPURenderBundleEncoderDescriptor,
    ) -> Fallible<DomRoot<GPURenderBundleEncoder<D>>> {
        GPURenderBundleEncoder::create(cx, self, descriptor)
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createqueryset>
    fn CreateQuerySet(
        &self,
        cx: &mut JSContext,
        descriptor: &GPUQuerySetDescriptor,
    ) -> Fallible<DomRoot<GPUQuerySet<D>>> {
        GPUQuerySet::create(cx, self, descriptor)
    }

    /// <https://www.w3.org/TR/webgpu/#dom-gpudevice-importexternaltexture>
    fn ImportExternalTexture(
        &self,
        cx: &mut JSContext,
        descriptor: &GPUExternalTextureDescriptor<D>,
    ) -> Fallible<DomRoot<GPUExternalTexture<D>>> {
        GPUExternalTexture::create(cx, self, descriptor)
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
    fn PopErrorScope(&self, cx: &mut CurrentRealm<'_>) -> <<D as script_bindings::DomTypes>::Promise as script_bindings::interfaces::PromiseHelpers<D>>::StackRoot{
        let promise = D::Promise::new_in_realm_rooted(cx);
        let callback = <D::Promise as PromiseHelpers<D>>::StackRoot::callback_promise_dom_manipulation_task_source(
            &promise, self,
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
