/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

use std::borrow::Cow;
use std::cell::Cell;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::jsapi::{Heap, JSObject};
use webgpu::wgc::id::{BindGroupLayoutId, PipelineLayoutId};
use webgpu::wgc::pipeline as wgpu_pipe;
use webgpu::wgc::pipeline::RenderPipelineDescriptor;
use webgpu::wgt::TextureFormat;
use webgpu::{
    wgt, PopError, WebGPU, WebGPUComputePipeline, WebGPURenderPipeline, WebGPURequest,
    WebGPUResponse,
};

use super::gpu::AsyncWGPUListener;
use super::gpudevicelostinfo::GPUDeviceLostInfo;
use super::gpupipelineerror::GPUPipelineError;
use super::gpusupportedlimits::GPUSupportedLimits;
use crate::conversions::Convert;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::EventBinding::EventInit;
use crate::dom::bindings::codegen::Bindings::EventTargetBinding::EventTargetMethods;
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::{
    GPUBindGroupDescriptor, GPUBindGroupLayoutDescriptor, GPUBufferDescriptor,
    GPUCommandEncoderDescriptor, GPUComputePipelineDescriptor, GPUDeviceLostReason,
    GPUDeviceMethods, GPUErrorFilter, GPUPipelineErrorReason, GPUPipelineLayoutDescriptor,
    GPURenderBundleEncoderDescriptor, GPURenderPipelineDescriptor, GPUSamplerDescriptor,
    GPUShaderModuleDescriptor, GPUSupportedLimitsMethods, GPUTextureDescriptor, GPUTextureFormat,
    GPUUncapturedErrorEventInit, GPUVertexStepMode,
};
use crate::dom::bindings::codegen::UnionTypes::GPUPipelineLayoutOrGPUAutoLayoutMode;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::types::GPUError;
use crate::dom::webgpu::gpu::response_async;
use crate::dom::webgpu::gpuadapter::GPUAdapter;
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
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct GPUDevice {
    eventtarget: EventTarget,
    #[ignore_malloc_size_of = "channels are hard"]
    #[no_trace]
    channel: WebGPU,
    adapter: Dom<GPUAdapter>,
    #[ignore_malloc_size_of = "mozjs"]
    extensions: Heap<*mut JSObject>,
    features: Dom<GPUSupportedFeatures>,
    limits: Dom<GPUSupportedLimits>,
    label: DomRefCell<USVString>,
    #[no_trace]
    device: webgpu::WebGPUDevice,
    default_queue: Dom<GPUQueue>,
    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-lost>
    #[ignore_malloc_size_of = "promises are hard"]
    lost_promise: DomRefCell<Rc<Promise>>,
    valid: Cell<bool>,
}

pub(crate) enum PipelineLayout {
    Implicit(PipelineLayoutId, Vec<BindGroupLayoutId>),
    Explicit(PipelineLayoutId),
}

impl PipelineLayout {
    pub(crate) fn explicit(&self) -> Option<PipelineLayoutId> {
        match self {
            PipelineLayout::Explicit(layout_id) => Some(*layout_id),
            _ => None,
        }
    }

    pub(crate) fn implicit(self) -> Option<(PipelineLayoutId, Vec<BindGroupLayoutId>)> {
        match self {
            PipelineLayout::Implicit(layout_id, bind_group_layout_ids) => {
                Some((layout_id, bind_group_layout_ids))
            },
            _ => None,
        }
    }
}

impl GPUDevice {
    #[allow(clippy::too_many_arguments)]
    fn new_inherited(
        channel: WebGPU,
        adapter: &GPUAdapter,
        extensions: Heap<*mut JSObject>,
        features: &GPUSupportedFeatures,
        limits: &GPUSupportedLimits,
        device: webgpu::WebGPUDevice,
        queue: &GPUQueue,
        label: String,
        lost_promise: Rc<Promise>,
    ) -> Self {
        Self {
            eventtarget: EventTarget::new_inherited(),
            channel,
            adapter: Dom::from_ref(adapter),
            extensions,
            features: Dom::from_ref(features),
            limits: Dom::from_ref(limits),
            label: DomRefCell::new(USVString::from(label)),
            device,
            default_queue: Dom::from_ref(queue),
            lost_promise: DomRefCell::new(lost_promise),
            valid: Cell::new(true),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        global: &GlobalScope,
        channel: WebGPU,
        adapter: &GPUAdapter,
        extensions: Heap<*mut JSObject>,
        features: wgt::Features,
        limits: wgt::Limits,
        device: webgpu::WebGPUDevice,
        queue: webgpu::WebGPUQueue,
        label: String,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        let queue = GPUQueue::new(global, channel.clone(), queue, can_gc);
        let limits = GPUSupportedLimits::new(global, limits, can_gc);
        let features = GPUSupportedFeatures::Constructor(global, None, features, can_gc).unwrap();
        let lost_promise = Promise::new(global, can_gc);
        let device = reflect_dom_object(
            Box::new(GPUDevice::new_inherited(
                channel,
                adapter,
                extensions,
                &features,
                &limits,
                device,
                &queue,
                label,
                lost_promise,
            )),
            global,
            can_gc,
        );
        queue.set_device(&device);
        device
    }
}

impl GPUDevice {
    pub(crate) fn id(&self) -> webgpu::WebGPUDevice {
        self.device
    }

    pub(crate) fn queue_id(&self) -> webgpu::WebGPUQueue {
        self.default_queue.id()
    }

    pub(crate) fn channel(&self) -> WebGPU {
        self.channel.clone()
    }

    pub(crate) fn dispatch_error(&self, error: webgpu::Error) {
        if let Err(e) = self.channel.0.send(WebGPURequest::DispatchError {
            device_id: self.device.0,
            error,
        }) {
            warn!("Failed to send WebGPURequest::DispatchError due to {e:?}");
        }
    }

    pub(crate) fn fire_uncaptured_error(&self, error: webgpu::Error, can_gc: CanGc) {
        let error = GPUError::from_error(&self.global(), error, can_gc);
        let ev = GPUUncapturedErrorEvent::new(
            &self.global(),
            DOMString::from("uncapturederror"),
            &GPUUncapturedErrorEventInit {
                error,
                parent: EventInit::empty(),
            },
            can_gc,
        );
        let _ = self.eventtarget.DispatchEvent(ev.event(), can_gc);
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
            Err(Error::Type(format!(
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
        if let GPUPipelineLayoutOrGPUAutoLayoutMode::GPUPipelineLayout(ref layout) = layout {
            PipelineLayout::Explicit(layout.id().0)
        } else {
            let layout_id = self.global().wgpu_id_hub().create_pipeline_layout_id();
            let max_bind_grps = self.limits.MaxBindGroups();
            let mut bgl_ids = Vec::with_capacity(max_bind_grps as usize);
            for _ in 0..max_bind_grps {
                let bgl = self.global().wgpu_id_hub().create_bind_group_layout_id();
                bgl_ids.push(bgl);
            }
            PipelineLayout::Implicit(layout_id, bgl_ids)
        }
    }

    pub(crate) fn parse_render_pipeline<'a>(
        &self,
        descriptor: &GPURenderPipelineDescriptor,
    ) -> Fallible<(PipelineLayout, RenderPipelineDescriptor<'a>)> {
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
                                GPUVertexStepMode::Vertex => wgt::VertexStepMode::Vertex,
                                GPUVertexStepMode::Instance => wgt::VertexStepMode::Instance,
                            },
                            attributes: Cow::Owned(
                                buffer
                                    .attributes
                                    .iter()
                                    .map(|att| wgt::VertexAttribute {
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
                                            Some(wgt::ColorTargetState {
                                                format,
                                                write_mask: wgt::ColorWrites::from_bits_retain(
                                                    state.writeMask,
                                                ),
                                                blend: state.blend.as_ref().map(|blend| {
                                                    wgt::BlendState {
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
                        .map(|format| wgt::DepthStencilState {
                            format,
                            depth_write_enabled: dss_desc.depthWriteEnabled,
                            depth_compare: dss_desc.depthCompare.convert(),
                            stencil: wgt::StencilState {
                                front: wgt::StencilFaceState {
                                    compare: dss_desc.stencilFront.compare.convert(),

                                    fail_op: dss_desc.stencilFront.failOp.convert(),
                                    depth_fail_op: dss_desc.stencilFront.depthFailOp.convert(),
                                    pass_op: dss_desc.stencilFront.passOp.convert(),
                                },
                                back: wgt::StencilFaceState {
                                    compare: dss_desc.stencilBack.compare.convert(),
                                    fail_op: dss_desc.stencilBack.failOp.convert(),
                                    depth_fail_op: dss_desc.stencilBack.depthFailOp.convert(),
                                    pass_op: dss_desc.stencilBack.passOp.convert(),
                                },
                                read_mask: dss_desc.stencilReadMask,
                                write_mask: dss_desc.stencilWriteMask,
                            },
                            bias: wgt::DepthBiasState {
                                constant: dss_desc.depthBias,
                                slope_scale: *dss_desc.depthBiasSlopeScale,
                                clamp: *dss_desc.depthBiasClamp,
                            },
                        })
                })
                .transpose()?,
            multisample: wgt::MultisampleState {
                count: descriptor.multisample.count,
                mask: descriptor.multisample.mask as u64,
                alpha_to_coverage_enabled: descriptor.multisample.alphaToCoverageEnabled,
            },
            multiview: None,
        };
        Ok((pipeline_layout, desc))
    }

    /// <https://gpuweb.github.io/gpuweb/#lose-the-device>
    pub(crate) fn lose(&self, reason: GPUDeviceLostReason, msg: String) {
        let lost_promise = &(*self.lost_promise.borrow());
        let global = &self.global();
        let lost = GPUDeviceLostInfo::new(global, msg.into(), reason, CanGc::note());
        lost_promise.resolve_native(&*lost);
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
    fn CreateBuffer(&self, descriptor: &GPUBufferDescriptor) -> Fallible<DomRoot<GPUBuffer>> {
        GPUBuffer::create(self, descriptor)
    }

    /// <https://gpuweb.github.io/gpuweb/#GPUDevice-createBindGroupLayout>
    #[allow(non_snake_case)]
    fn CreateBindGroupLayout(
        &self,
        descriptor: &GPUBindGroupLayoutDescriptor,
    ) -> Fallible<DomRoot<GPUBindGroupLayout>> {
        GPUBindGroupLayout::create(self, descriptor)
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createpipelinelayout>
    fn CreatePipelineLayout(
        &self,
        descriptor: &GPUPipelineLayoutDescriptor,
    ) -> DomRoot<GPUPipelineLayout> {
        GPUPipelineLayout::create(self, descriptor)
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createbindgroup>
    fn CreateBindGroup(&self, descriptor: &GPUBindGroupDescriptor) -> DomRoot<GPUBindGroup> {
        GPUBindGroup::create(self, descriptor)
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createshadermodule>
    fn CreateShaderModule(
        &self,
        descriptor: RootedTraceableBox<GPUShaderModuleDescriptor>,
        comp: InRealm,
        can_gc: CanGc,
    ) -> DomRoot<GPUShaderModule> {
        GPUShaderModule::create(self, descriptor, comp, can_gc)
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createcomputepipeline>
    fn CreateComputePipeline(
        &self,
        descriptor: &GPUComputePipelineDescriptor,
    ) -> DomRoot<GPUComputePipeline> {
        let compute_pipeline = GPUComputePipeline::create(self, descriptor, None);
        GPUComputePipeline::new(
            &self.global(),
            compute_pipeline,
            descriptor.parent.parent.label.clone(),
            self,
            CanGc::note(),
        )
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createcomputepipelineasync>
    fn CreateComputePipelineAsync(
        &self,
        descriptor: &GPUComputePipelineDescriptor,
        comp: InRealm,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        let promise = Promise::new_in_current_realm(comp, can_gc);
        let sender = response_async(&promise, self);
        GPUComputePipeline::create(self, descriptor, Some(sender));
        promise
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createcommandencoder>
    fn CreateCommandEncoder(
        &self,
        descriptor: &GPUCommandEncoderDescriptor,
    ) -> DomRoot<GPUCommandEncoder> {
        GPUCommandEncoder::create(self, descriptor)
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createtexture>
    fn CreateTexture(&self, descriptor: &GPUTextureDescriptor) -> Fallible<DomRoot<GPUTexture>> {
        GPUTexture::create(self, descriptor)
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createsampler>
    fn CreateSampler(&self, descriptor: &GPUSamplerDescriptor) -> DomRoot<GPUSampler> {
        GPUSampler::create(self, descriptor)
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createrenderpipeline>
    fn CreateRenderPipeline(
        &self,
        descriptor: &GPURenderPipelineDescriptor,
    ) -> Fallible<DomRoot<GPURenderPipeline>> {
        let (pipeline_layout, desc) = self.parse_render_pipeline(descriptor)?;
        let render_pipeline = GPURenderPipeline::create(self, pipeline_layout, desc, None)?;
        Ok(GPURenderPipeline::new(
            &self.global(),
            render_pipeline,
            descriptor.parent.parent.label.clone(),
            self,
            CanGc::note(),
        ))
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createrenderpipelineasync>
    fn CreateRenderPipelineAsync(
        &self,
        descriptor: &GPURenderPipelineDescriptor,
        comp: InRealm,
        can_gc: CanGc,
    ) -> Fallible<Rc<Promise>> {
        let (implicit_ids, desc) = self.parse_render_pipeline(descriptor)?;
        let promise = Promise::new_in_current_realm(comp, can_gc);
        let sender = response_async(&promise, self);
        GPURenderPipeline::create(self, implicit_ids, desc, Some(sender))?;
        Ok(promise)
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createrenderbundleencoder>
    fn CreateRenderBundleEncoder(
        &self,
        descriptor: &GPURenderBundleEncoderDescriptor,
    ) -> Fallible<DomRoot<GPURenderBundleEncoder>> {
        GPURenderBundleEncoder::create(self, descriptor)
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-pusherrorscope>
    fn PushErrorScope(&self, filter: GPUErrorFilter) {
        if self
            .channel
            .0
            .send(WebGPURequest::PushErrorScope {
                device_id: self.device.0,
                filter: filter.as_webgpu(),
            })
            .is_err()
        {
            warn!("Failed sending WebGPURequest::PushErrorScope");
        }
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-poperrorscope>
    fn PopErrorScope(&self, comp: InRealm, can_gc: CanGc) -> Rc<Promise> {
        let promise = Promise::new_in_current_realm(comp, can_gc);
        let sender = response_async(&promise, self);
        if self
            .channel
            .0
            .send(WebGPURequest::PopErrorScope {
                device_id: self.device.0,
                sender,
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
                .channel
                .0
                .send(WebGPURequest::DestroyDevice(self.device.0))
            {
                warn!("Failed to send DestroyDevice ({:?}) ({})", self.device.0, e);
            }
        }
    }
}

impl AsyncWGPUListener for GPUDevice {
    fn handle_response(&self, response: WebGPUResponse, promise: &Rc<Promise>, can_gc: CanGc) {
        match response {
            WebGPUResponse::PoppedErrorScope(result) => match result {
                Ok(None) | Err(PopError::Lost) => promise.resolve_native(&None::<Option<GPUError>>),
                Err(PopError::Empty) => promise.reject_error(Error::Operation),
                Ok(Some(error)) => {
                    let error = GPUError::from_error(&self.global(), error, can_gc);
                    promise.resolve_native(&error);
                },
            },
            WebGPUResponse::ComputePipeline(result) => match result {
                Ok(pipeline) => promise.resolve_native(&GPUComputePipeline::new(
                    &self.global(),
                    WebGPUComputePipeline(pipeline.id),
                    pipeline.label.into(),
                    self,
                    can_gc,
                )),
                Err(webgpu::Error::Validation(msg)) => {
                    promise.reject_native(&GPUPipelineError::new(
                        &self.global(),
                        msg.into(),
                        GPUPipelineErrorReason::Validation,
                        can_gc,
                    ))
                },
                Err(webgpu::Error::OutOfMemory(msg) | webgpu::Error::Internal(msg)) => promise
                    .reject_native(&GPUPipelineError::new(
                        &self.global(),
                        msg.into(),
                        GPUPipelineErrorReason::Internal,
                        can_gc,
                    )),
            },
            WebGPUResponse::RenderPipeline(result) => match result {
                Ok(pipeline) => promise.resolve_native(&GPURenderPipeline::new(
                    &self.global(),
                    WebGPURenderPipeline(pipeline.id),
                    pipeline.label.into(),
                    self,
                    can_gc,
                )),
                Err(webgpu::Error::Validation(msg)) => {
                    promise.reject_native(&GPUPipelineError::new(
                        &self.global(),
                        msg.into(),
                        GPUPipelineErrorReason::Validation,
                        can_gc,
                    ))
                },
                Err(webgpu::Error::OutOfMemory(msg) | webgpu::Error::Internal(msg)) => promise
                    .reject_native(&GPUPipelineError::new(
                        &self.global(),
                        msg.into(),
                        GPUPipelineErrorReason::Internal,
                        can_gc,
                    )),
            },
            _ => unreachable!("Wrong response received on AsyncWGPUListener for GPUDevice"),
        }
    }
}

impl Drop for GPUDevice {
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
