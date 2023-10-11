/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::num::NonZeroU64;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::jsapi::{Heap, JSObject};
use webgpu::identity::WebGPUOpResult;
use webgpu::wgpu::id::{BindGroupLayoutId, PipelineLayoutId};
use webgpu::wgpu::{
    binding_model as wgpu_bind, command as wgpu_com, pipeline as wgpu_pipe, resource as wgpu_res,
};
use webgpu::{self, wgt, ErrorScopeId, WebGPU, WebGPURequest};

use super::bindings::codegen::Bindings::WebGPUBinding::{
    GPUBlendComponent, GPUBufferBindingType, GPUDeviceLostReason, GPUPrimitiveState,
    GPUSamplerBindingType, GPUStorageTextureAccess, GPUTextureSampleType, GPUVertexStepMode,
};
use super::bindings::codegen::UnionTypes::GPUPipelineLayoutOrGPUAutoLayoutMode;
use super::bindings::error::Fallible;
use super::gpudevicelostinfo::GPUDeviceLostInfo;
use super::gpusupportedlimits::GPUSupportedLimits;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::EventBinding::EventInit;
use crate::dom::bindings::codegen::Bindings::EventTargetBinding::EventTargetMethods;
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::{
    GPUAddressMode, GPUBindGroupDescriptor, GPUBindGroupLayoutDescriptor, GPUBindingResource,
    GPUBlendFactor, GPUBlendOperation, GPUBufferDescriptor, GPUCommandEncoderDescriptor,
    GPUCompareFunction, GPUComputePipelineDescriptor, GPUCullMode, GPUDeviceMethods, GPUError,
    GPUErrorFilter, GPUExtent3D, GPUExtent3DDict, GPUFilterMode, GPUFrontFace, GPUIndexFormat,
    GPUObjectDescriptorBase, GPUPipelineLayoutDescriptor, GPUPrimitiveTopology,
    GPURenderBundleEncoderDescriptor, GPURenderPipelineDescriptor, GPUSamplerDescriptor,
    GPUShaderModuleDescriptor, GPUStencilOperation, GPUSupportedLimitsMethods,
    GPUTextureDescriptor, GPUTextureDimension, GPUTextureFormat, GPUTextureViewDimension,
    GPUUncapturedErrorEventInit, GPUVertexFormat,
};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::gpuadapter::GPUAdapter;
use crate::dom::gpubindgroup::GPUBindGroup;
use crate::dom::gpubindgrouplayout::GPUBindGroupLayout;
use crate::dom::gpubuffer::{GPUBuffer, GPUBufferMapInfo, GPUBufferState};
use crate::dom::gpucommandencoder::GPUCommandEncoder;
use crate::dom::gpucomputepipeline::GPUComputePipeline;
use crate::dom::gpuoutofmemoryerror::GPUOutOfMemoryError;
use crate::dom::gpupipelinelayout::GPUPipelineLayout;
use crate::dom::gpuqueue::GPUQueue;
use crate::dom::gpurenderbundleencoder::GPURenderBundleEncoder;
use crate::dom::gpurenderpipeline::GPURenderPipeline;
use crate::dom::gpusampler::GPUSampler;
use crate::dom::gpushadermodule::GPUShaderModule;
use crate::dom::gpusupportedfeatures::GPUSupportedFeatures;
use crate::dom::gputexture::GPUTexture;
use crate::dom::gpuuncapturederrorevent::GPUUncapturedErrorEvent;
use crate::dom::gpuvalidationerror::GPUValidationError;
use crate::dom::promise::Promise;
use crate::realms::InRealm;

#[derive(JSTraceable, MallocSizeOf)]
struct ErrorScopeInfo {
    op_count: u64,
    #[ignore_malloc_size_of = "Because it is non-owning"]
    error: Option<GPUError>,
    #[ignore_malloc_size_of = "promises are hard"]
    promise: Option<Rc<Promise>>,
}

#[derive(JSTraceable, MallocSizeOf)]
struct ErrorScopeMetadata {
    id: ErrorScopeId,
    filter: GPUErrorFilter,
    popped: Cell<bool>,
}

#[derive(JSTraceable, MallocSizeOf)]
struct ScopeContext {
    error_scopes: HashMap<ErrorScopeId, ErrorScopeInfo>,
    scope_stack: Vec<ErrorScopeMetadata>,
    next_scope_id: ErrorScopeId,
}

#[dom_struct]
pub struct GPUDevice {
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
    scope_context: DomRefCell<ScopeContext>,
    #[ignore_malloc_size_of = "promises are hard"]
    lost_promise: DomRefCell<Option<Rc<Promise>>>,
    valid: Cell<bool>,
}

impl GPUDevice {
    fn new_inherited(
        channel: WebGPU,
        adapter: &GPUAdapter,
        extensions: Heap<*mut JSObject>,
        features: &GPUSupportedFeatures,
        limits: &GPUSupportedLimits,
        device: webgpu::WebGPUDevice,
        queue: &GPUQueue,
        label: String,
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
            scope_context: DomRefCell::new(ScopeContext {
                error_scopes: HashMap::new(),
                scope_stack: Vec::new(),
                next_scope_id: ErrorScopeId::new(1).unwrap(),
            }),
            lost_promise: DomRefCell::new(None),
            valid: Cell::new(true),
        }
    }

    pub fn new(
        global: &GlobalScope,
        channel: WebGPU,
        adapter: &GPUAdapter,
        extensions: Heap<*mut JSObject>,
        features: wgt::Features,
        limits: wgt::Limits,
        device: webgpu::WebGPUDevice,
        queue: webgpu::WebGPUQueue,
        label: String,
    ) -> DomRoot<Self> {
        let queue = GPUQueue::new(global, channel.clone(), queue);
        let limits = GPUSupportedLimits::new(global, limits);
        let features = GPUSupportedFeatures::Constructor(global, None, features).unwrap();
        let device = reflect_dom_object(
            Box::new(GPUDevice::new_inherited(
                channel, adapter, extensions, &features, &limits, device, &queue, label,
            )),
            global,
        );
        queue.set_device(&*device);
        device
    }
}

impl GPUDevice {
    pub fn id(&self) -> webgpu::WebGPUDevice {
        self.device
    }

    pub fn handle_server_msg(&self, scope: Option<ErrorScopeId>, result: WebGPUOpResult) {
        let result = match result {
            WebGPUOpResult::Success => Ok(()),
            WebGPUOpResult::ValidationError(m) => {
                let val_err = GPUValidationError::new(&self.global(), DOMString::from_string(m));
                Err((
                    GPUError::GPUValidationError(val_err),
                    GPUErrorFilter::Validation,
                ))
            },
            WebGPUOpResult::OutOfMemoryError => {
                let oom_err = GPUOutOfMemoryError::new(&self.global());
                Err((
                    GPUError::GPUOutOfMemoryError(oom_err),
                    GPUErrorFilter::Out_of_memory,
                ))
            },
        };

        if let Some(s_id) = scope {
            if let Err((err, filter)) = result {
                let scop = self
                    .scope_context
                    .borrow()
                    .scope_stack
                    .iter()
                    .rev()
                    .find(|meta| meta.id <= s_id && meta.filter == filter)
                    .map(|meta| meta.id);
                if let Some(s) = scop {
                    self.handle_error(s, err);
                } else {
                    self.fire_uncaptured_error(err);
                }
            }
            self.try_remove_scope(s_id);
        } else {
            if let Err((err, _)) = result {
                self.fire_uncaptured_error(err);
            }
        }
    }

    fn handle_error(&self, scope: ErrorScopeId, error: GPUError) {
        let mut context = self.scope_context.borrow_mut();
        if let Some(mut err_scope) = context.error_scopes.get_mut(&scope) {
            if err_scope.error.is_none() {
                err_scope.error = Some(error);
            }
        } else {
            warn!("Could not find ErrorScope with Id({})", scope);
        }
    }

    fn try_remove_scope(&self, scope: ErrorScopeId) {
        let mut context = self.scope_context.borrow_mut();
        let remove = if let Some(mut err_scope) = context.error_scopes.get_mut(&scope) {
            err_scope.op_count -= 1;
            if let Some(ref promise) = err_scope.promise {
                if !promise.is_fulfilled() {
                    if let Some(ref e) = err_scope.error {
                        promise.resolve_native(e);
                    } else if err_scope.op_count == 0 {
                        promise.resolve_native(&None::<GPUError>);
                    }
                }
            }
            err_scope.op_count == 0 && err_scope.promise.is_some()
        } else {
            warn!("Could not find ErrorScope with Id({})", scope);
            false
        };
        if remove {
            let _ = context.error_scopes.remove(&scope);
            context.scope_stack.retain(|meta| meta.id != scope);
        }
    }

    fn fire_uncaptured_error(&self, err: GPUError) {
        let ev = GPUUncapturedErrorEvent::new(
            &self.global(),
            DOMString::from("uncapturederror"),
            &GPUUncapturedErrorEventInit {
                error: err,
                parent: EventInit::empty(),
            },
        );
        let _ = self.eventtarget.DispatchEvent(ev.event());
    }

    pub fn use_current_scope(&self) -> Option<ErrorScopeId> {
        let mut context = self.scope_context.borrow_mut();
        let scope_id = context
            .scope_stack
            .iter()
            .rev()
            .find(|meta| !meta.popped.get())
            .map(|meta| meta.id);
        scope_id.and_then(|s_id| {
            context.error_scopes.get_mut(&s_id).map(|mut scope| {
                scope.op_count += 1;
                s_id
            })
        })
    }

    fn get_pipeline_layout_data(
        &self,
        layout: &GPUPipelineLayoutOrGPUAutoLayoutMode,
    ) -> (
        Option<PipelineLayoutId>,
        Option<(PipelineLayoutId, Vec<BindGroupLayoutId>)>,
        Vec<webgpu::WebGPUBindGroupLayout>,
    ) {
        if let GPUPipelineLayoutOrGPUAutoLayoutMode::GPUPipelineLayout(ref layout) = layout {
            (Some(layout.id().0), None, layout.bind_group_layouts())
        } else {
            let layout_id = self
                .global()
                .wgpu_id_hub()
                .lock()
                .create_pipeline_layout_id(self.device.0.backend());
            let max_bind_grps = self.limits.MaxBindGroups();
            let mut bgls = Vec::with_capacity(max_bind_grps as usize);
            let mut bgl_ids = Vec::with_capacity(max_bind_grps as usize);
            for _ in 0..max_bind_grps {
                let bgl = self
                    .global()
                    .wgpu_id_hub()
                    .lock()
                    .create_bind_group_layout_id(self.device.0.backend());
                bgls.push(webgpu::WebGPUBindGroupLayout(bgl));
                bgl_ids.push(bgl);
            }
            (None, Some((layout_id, bgl_ids)), bgls)
        }
    }

    /// https://gpuweb.github.io/gpuweb/#lose-the-device
    pub fn lose(&self, reason: GPUDeviceLostReason) {
        if let Some(ref lost_promise) = *self.lost_promise.borrow() {
            let global = &self.global();
            let msg = match reason {
                GPUDeviceLostReason::Unknown => "Unknown reason for your device loss.",
                GPUDeviceLostReason::Destroyed => {
                    "Device self-destruction sequence activated successfully!"
                },
            };
            let lost = GPUDeviceLostInfo::new(global, msg.into(), reason);
            lost_promise.resolve_native(&*lost);
        }
    }
}

impl GPUDeviceMethods for GPUDevice {
    /// https://gpuweb.github.io/gpuweb/#dom-gpudevice-features
    fn Features(&self) -> DomRoot<GPUSupportedFeatures> {
        DomRoot::from_ref(&self.features)
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpudevice-limits
    fn Limits(&self) -> DomRoot<GPUSupportedLimits> {
        DomRoot::from_ref(&self.limits)
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpudevice-queue
    fn GetQueue(&self) -> DomRoot<GPUQueue> {
        DomRoot::from_ref(&self.default_queue)
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn Label(&self) -> USVString {
        self.label.borrow().clone()
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn SetLabel(&self, value: USVString) {
        *self.label.borrow_mut() = value;
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpudevice-lost
    fn GetLost(&self, comp: InRealm) -> Fallible<Rc<Promise>> {
        let promise = Promise::new_in_current_realm(comp);
        *self.lost_promise.borrow_mut() = Some(promise.clone());
        Ok(promise)
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpudevice-createbuffer
    fn CreateBuffer(&self, descriptor: &GPUBufferDescriptor) -> Fallible<DomRoot<GPUBuffer>> {
        let desc =
            wgt::BufferUsages::from_bits(descriptor.usage).map(|usg| wgpu_res::BufferDescriptor {
                label: convert_label(&descriptor.parent),
                size: descriptor.size as wgt::BufferAddress,
                usage: usg,
                mapped_at_creation: descriptor.mappedAtCreation,
            });
        let id = self
            .global()
            .wgpu_id_hub()
            .lock()
            .create_buffer_id(self.device.0.backend());

        let scope_id = self.use_current_scope();
        if desc.is_none() {
            self.handle_server_msg(
                scope_id,
                WebGPUOpResult::ValidationError(String::from("Invalid GPUBufferUsage")),
            );
        }

        self.channel
            .0
            .send((
                scope_id,
                WebGPURequest::CreateBuffer {
                    device_id: self.device.0,
                    buffer_id: id,
                    descriptor: desc,
                },
            ))
            .expect("Failed to create WebGPU buffer");

        let buffer = webgpu::WebGPUBuffer(id);
        let map_info;
        let state;
        if descriptor.mappedAtCreation {
            let buf_data = vec![0u8; descriptor.size as usize];
            map_info = DomRefCell::new(Some(GPUBufferMapInfo {
                mapping: Rc::new(RefCell::new(buf_data)),
                mapping_range: 0..descriptor.size,
                mapped_ranges: Vec::new(),
                js_buffers: Vec::new(),
                map_mode: None,
            }));
            state = GPUBufferState::MappedAtCreation;
        } else {
            map_info = DomRefCell::new(None);
            state = GPUBufferState::Unmapped;
        }

        Ok(GPUBuffer::new(
            &self.global(),
            self.channel.clone(),
            buffer,
            &self,
            state,
            descriptor.size,
            map_info,
            descriptor.parent.label.clone().unwrap_or_default(),
        ))
    }

    /// https://gpuweb.github.io/gpuweb/#GPUDevice-createBindGroupLayout
    #[allow(non_snake_case)]
    fn CreateBindGroupLayout(
        &self,
        descriptor: &GPUBindGroupLayoutDescriptor,
    ) -> DomRoot<GPUBindGroupLayout> {
        let mut valid = true;
        let entries = descriptor
            .entries
            .iter()
            .map(|bind| {
                let visibility = match wgt::ShaderStages::from_bits(bind.visibility) {
                    Some(visibility) => visibility,
                    None => {
                        valid = false;
                        wgt::ShaderStages::empty()
                    },
                };
                let ty = if let Some(buffer) = &bind.buffer {
                    wgt::BindingType::Buffer {
                        ty: match buffer.type_ {
                            GPUBufferBindingType::Uniform => wgt::BufferBindingType::Uniform,
                            GPUBufferBindingType::Storage => {
                                wgt::BufferBindingType::Storage { read_only: false }
                            },
                            GPUBufferBindingType::Read_only_storage => {
                                wgt::BufferBindingType::Storage { read_only: true }
                            },
                        },
                        has_dynamic_offset: buffer.hasDynamicOffset,
                        min_binding_size: NonZeroU64::new(buffer.minBindingSize),
                    }
                } else if let Some(sampler) = &bind.sampler {
                    wgt::BindingType::Sampler(match sampler.type_ {
                        GPUSamplerBindingType::Filtering => wgt::SamplerBindingType::Filtering,
                        GPUSamplerBindingType::Non_filtering => {
                            wgt::SamplerBindingType::NonFiltering
                        },
                        GPUSamplerBindingType::Comparison => wgt::SamplerBindingType::Comparison,
                    })
                } else if let Some(storage) = &bind.storageTexture {
                    wgt::BindingType::StorageTexture {
                        access: match storage.access {
                            GPUStorageTextureAccess::Write_only => {
                                wgt::StorageTextureAccess::WriteOnly
                            },
                        },
                        format: convert_texture_format(storage.format),
                        view_dimension: convert_view_dimension(storage.viewDimension),
                    }
                } else if let Some(texture) = &bind.texture {
                    wgt::BindingType::Texture {
                        sample_type: match texture.sampleType {
                            GPUTextureSampleType::Float => {
                                wgt::TextureSampleType::Float { filterable: true }
                            },
                            GPUTextureSampleType::Unfilterable_float => {
                                wgt::TextureSampleType::Float { filterable: false }
                            },
                            GPUTextureSampleType::Depth => wgt::TextureSampleType::Depth,
                            GPUTextureSampleType::Sint => wgt::TextureSampleType::Sint,
                            GPUTextureSampleType::Uint => wgt::TextureSampleType::Uint,
                        },
                        view_dimension: convert_view_dimension(texture.viewDimension),
                        multisampled: texture.multisampled,
                    }
                } else {
                    valid = false;
                    todo!("Handle error");
                };

                wgt::BindGroupLayoutEntry {
                    binding: bind.binding,
                    visibility: visibility,
                    ty,
                    count: None,
                }
            })
            .collect::<Vec<_>>();

        let scope_id = self.use_current_scope();

        let desc = if valid {
            Some(wgpu_bind::BindGroupLayoutDescriptor {
                label: convert_label(&descriptor.parent),
                entries: Cow::Owned(entries),
            })
        } else {
            self.handle_server_msg(
                scope_id,
                WebGPUOpResult::ValidationError(String::from("Invalid GPUShaderStage")),
            );
            None
        };

        let bind_group_layout_id = self
            .global()
            .wgpu_id_hub()
            .lock()
            .create_bind_group_layout_id(self.device.0.backend());
        self.channel
            .0
            .send((
                scope_id,
                WebGPURequest::CreateBindGroupLayout {
                    device_id: self.device.0,
                    bind_group_layout_id,
                    descriptor: desc,
                },
            ))
            .expect("Failed to create WebGPU BindGroupLayout");

        let bgl = webgpu::WebGPUBindGroupLayout(bind_group_layout_id);

        GPUBindGroupLayout::new(
            &self.global(),
            bgl,
            descriptor.parent.label.clone().unwrap_or_default(),
        )
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpudevice-createpipelinelayout
    fn CreatePipelineLayout(
        &self,
        descriptor: &GPUPipelineLayoutDescriptor,
    ) -> DomRoot<GPUPipelineLayout> {
        let desc = wgpu_bind::PipelineLayoutDescriptor {
            label: convert_label(&descriptor.parent),
            bind_group_layouts: Cow::Owned(
                descriptor
                    .bindGroupLayouts
                    .iter()
                    .map(|each| each.id().0)
                    .collect::<Vec<_>>(),
            ),
            push_constant_ranges: Cow::Owned(vec![]),
        };

        let scope_id = self.use_current_scope();

        let pipeline_layout_id = self
            .global()
            .wgpu_id_hub()
            .lock()
            .create_pipeline_layout_id(self.device.0.backend());
        self.channel
            .0
            .send((
                scope_id,
                WebGPURequest::CreatePipelineLayout {
                    device_id: self.device.0,
                    pipeline_layout_id,
                    descriptor: desc,
                },
            ))
            .expect("Failed to create WebGPU PipelineLayout");

        let bgls = descriptor
            .bindGroupLayouts
            .iter()
            .map(|each| each.id())
            .collect::<Vec<_>>();
        let pipeline_layout = webgpu::WebGPUPipelineLayout(pipeline_layout_id);
        GPUPipelineLayout::new(
            &self.global(),
            pipeline_layout,
            descriptor.parent.label.clone().unwrap_or_default(),
            bgls,
        )
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpudevice-createbindgroup
    fn CreateBindGroup(&self, descriptor: &GPUBindGroupDescriptor) -> DomRoot<GPUBindGroup> {
        let entries = descriptor
            .entries
            .iter()
            .map(|bind| wgpu_bind::BindGroupEntry {
                binding: bind.binding,
                resource: match bind.resource {
                    GPUBindingResource::GPUSampler(ref s) => {
                        wgpu_bind::BindingResource::Sampler(s.id().0)
                    },
                    GPUBindingResource::GPUTextureView(ref t) => {
                        wgpu_bind::BindingResource::TextureView(t.id().0)
                    },
                    GPUBindingResource::GPUBufferBinding(ref b) => {
                        wgpu_bind::BindingResource::Buffer(wgpu_bind::BufferBinding {
                            buffer_id: b.buffer.id().0,
                            offset: b.offset,
                            size: b.size.and_then(wgt::BufferSize::new),
                        })
                    },
                },
            })
            .collect::<Vec<_>>();

        let desc = wgpu_bind::BindGroupDescriptor {
            label: convert_label(&descriptor.parent),
            layout: descriptor.layout.id().0,
            entries: Cow::Owned(entries),
        };

        let scope_id = self.use_current_scope();

        let bind_group_id = self
            .global()
            .wgpu_id_hub()
            .lock()
            .create_bind_group_id(self.device.0.backend());
        self.channel
            .0
            .send((
                scope_id,
                WebGPURequest::CreateBindGroup {
                    device_id: self.device.0,
                    bind_group_id,
                    descriptor: desc,
                },
            ))
            .expect("Failed to create WebGPU BindGroup");

        let bind_group = webgpu::WebGPUBindGroup(bind_group_id);

        GPUBindGroup::new(
            &self.global(),
            bind_group,
            self.device,
            &*descriptor.layout,
            descriptor.parent.label.clone().unwrap_or_default(),
        )
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpudevice-createshadermodule
    fn CreateShaderModule(
        &self,
        descriptor: RootedTraceableBox<GPUShaderModuleDescriptor>,
    ) -> DomRoot<GPUShaderModule> {
        let program_id = self
            .global()
            .wgpu_id_hub()
            .lock()
            .create_shader_module_id(self.device.0.backend());

        let scope_id = self.use_current_scope();
        self.channel
            .0
            .send((
                scope_id,
                WebGPURequest::CreateShaderModule {
                    device_id: self.device.0,
                    program_id,
                    program: descriptor.code.0.clone(),
                    label: None,
                },
            ))
            .expect("Failed to create WebGPU ShaderModule");

        let shader_module = webgpu::WebGPUShaderModule(program_id);
        GPUShaderModule::new(
            &self.global(),
            shader_module,
            descriptor.parent.label.clone().unwrap_or_default(),
        )
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpudevice-createcomputepipeline
    fn CreateComputePipeline(
        &self,
        descriptor: &GPUComputePipelineDescriptor,
    ) -> DomRoot<GPUComputePipeline> {
        let compute_pipeline_id = self
            .global()
            .wgpu_id_hub()
            .lock()
            .create_compute_pipeline_id(self.device.0.backend());

        let scope_id = self.use_current_scope();
        let (layout, implicit_ids, bgls) = self.get_pipeline_layout_data(&descriptor.parent.layout);

        let desc = wgpu_pipe::ComputePipelineDescriptor {
            label: convert_label(&descriptor.parent.parent),
            layout,
            stage: wgpu_pipe::ProgrammableStageDescriptor {
                module: descriptor.compute.module.id().0,
                entry_point: Cow::Owned(descriptor.compute.entryPoint.to_string()),
            },
        };

        self.channel
            .0
            .send((
                scope_id,
                WebGPURequest::CreateComputePipeline {
                    device_id: self.device.0,
                    compute_pipeline_id,
                    descriptor: desc,
                    implicit_ids,
                },
            ))
            .expect("Failed to create WebGPU ComputePipeline");

        let compute_pipeline = webgpu::WebGPUComputePipeline(compute_pipeline_id);
        GPUComputePipeline::new(
            &self.global(),
            compute_pipeline,
            descriptor.parent.parent.label.clone().unwrap_or_default(),
            bgls,
            &self,
        )
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpudevice-createcommandencoder
    fn CreateCommandEncoder(
        &self,
        descriptor: &GPUCommandEncoderDescriptor,
    ) -> DomRoot<GPUCommandEncoder> {
        let command_encoder_id = self
            .global()
            .wgpu_id_hub()
            .lock()
            .create_command_encoder_id(self.device.0.backend());
        let scope_id = self.use_current_scope();
        self.channel
            .0
            .send((
                scope_id,
                WebGPURequest::CreateCommandEncoder {
                    device_id: self.device.0,
                    command_encoder_id,
                    label: convert_label(&descriptor.parent),
                },
            ))
            .expect("Failed to create WebGPU command encoder");

        let encoder = webgpu::WebGPUCommandEncoder(command_encoder_id);

        GPUCommandEncoder::new(
            &self.global(),
            self.channel.clone(),
            &self,
            encoder,
            descriptor.parent.label.clone().unwrap_or_default(),
        )
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpudevice-createtexture
    fn CreateTexture(&self, descriptor: &GPUTextureDescriptor) -> DomRoot<GPUTexture> {
        let size = convert_texture_size_to_dict(&descriptor.size);
        let desc = wgt::TextureUsages::from_bits(descriptor.usage).map(|usg| {
            wgpu_res::TextureDescriptor {
                label: convert_label(&descriptor.parent),
                size: convert_texture_size_to_wgt(&size),
                mip_level_count: descriptor.mipLevelCount,
                sample_count: descriptor.sampleCount,
                dimension: match descriptor.dimension {
                    GPUTextureDimension::_1d => wgt::TextureDimension::D1,
                    GPUTextureDimension::_2d => wgt::TextureDimension::D2,
                    GPUTextureDimension::_3d => wgt::TextureDimension::D3,
                },
                format: convert_texture_format(descriptor.format),
                usage: usg,
                view_formats: descriptor
                    .viewFormats
                    .iter()
                    .map(|tf| convert_texture_format(*tf))
                    .collect(),
            }
        });

        let texture_id = self
            .global()
            .wgpu_id_hub()
            .lock()
            .create_texture_id(self.device.0.backend());

        let scope_id = self.use_current_scope();
        if desc.is_none() {
            self.handle_server_msg(
                scope_id,
                WebGPUOpResult::ValidationError(String::from("Invalid GPUTextureUsage")),
            );
        }
        self.channel
            .0
            .send((
                scope_id,
                WebGPURequest::CreateTexture {
                    device_id: self.device.0,
                    texture_id,
                    descriptor: desc,
                },
            ))
            .expect("Failed to create WebGPU Texture");

        let texture = webgpu::WebGPUTexture(texture_id);

        GPUTexture::new(
            &self.global(),
            texture,
            &self,
            self.channel.clone(),
            size,
            descriptor.mipLevelCount,
            descriptor.sampleCount,
            descriptor.dimension,
            descriptor.format,
            descriptor.usage,
            descriptor.parent.label.clone().unwrap_or_default(),
        )
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpudevice-createsampler
    fn CreateSampler(&self, descriptor: &GPUSamplerDescriptor) -> DomRoot<GPUSampler> {
        let sampler_id = self
            .global()
            .wgpu_id_hub()
            .lock()
            .create_sampler_id(self.device.0.backend());
        let compare_enable = descriptor.compare.is_some();
        let desc = wgpu_res::SamplerDescriptor {
            label: convert_label(&descriptor.parent),
            address_modes: [
                convert_address_mode(descriptor.addressModeU),
                convert_address_mode(descriptor.addressModeV),
                convert_address_mode(descriptor.addressModeW),
            ],
            mag_filter: convert_filter_mode(descriptor.magFilter),
            min_filter: convert_filter_mode(descriptor.minFilter),
            mipmap_filter: convert_filter_mode(descriptor.mipmapFilter),
            lod_min_clamp: *descriptor.lodMinClamp,
            lod_max_clamp: *descriptor.lodMaxClamp,
            compare: descriptor.compare.map(|c| convert_compare_function(c)),
            anisotropy_clamp: 1,
            border_color: None,
        };

        let scope_id = self.use_current_scope();
        self.channel
            .0
            .send((
                scope_id,
                WebGPURequest::CreateSampler {
                    device_id: self.device.0,
                    sampler_id,
                    descriptor: desc,
                },
            ))
            .expect("Failed to create WebGPU sampler");

        let sampler = webgpu::WebGPUSampler(sampler_id);

        GPUSampler::new(
            &self.global(),
            self.device,
            compare_enable,
            sampler,
            descriptor.parent.label.clone().unwrap_or_default(),
        )
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpudevice-createrenderpipeline
    fn CreateRenderPipeline(
        &self,
        descriptor: &GPURenderPipelineDescriptor,
    ) -> DomRoot<GPURenderPipeline> {
        let scope_id = self.use_current_scope();
        let mut valid = true;

        let (layout, implicit_ids, bgls) = self.get_pipeline_layout_data(&descriptor.parent.layout);

        let desc = if valid {
            Some(wgpu_pipe::RenderPipelineDescriptor {
                label: convert_label(&descriptor.parent.parent),
                layout,
                vertex: wgpu_pipe::VertexState {
                    stage: wgpu_pipe::ProgrammableStageDescriptor {
                        module: descriptor.vertex.parent.module.id().0,
                        entry_point: Cow::Owned(descriptor.vertex.parent.entryPoint.to_string()),
                    },
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
                                            format: convert_vertex_format(att.format),
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
                    .map(|stage| wgpu_pipe::FragmentState {
                        stage: wgpu_pipe::ProgrammableStageDescriptor {
                            module: stage.parent.module.id().0,
                            entry_point: Cow::Owned(stage.parent.entryPoint.to_string()),
                        },
                        targets: Cow::Owned(
                            stage
                                .targets
                                .iter()
                                .map(|state| {
                                    Some(wgt::ColorTargetState {
                                        format: convert_texture_format(state.format),
                                        write_mask: match wgt::ColorWrites::from_bits(
                                            state.writeMask,
                                        ) {
                                            Some(mask) => mask,
                                            None => {
                                                valid = false;
                                                wgt::ColorWrites::empty()
                                            },
                                        },
                                        blend: state.blend.as_ref().map(|blend| wgt::BlendState {
                                            color: convert_blend_component(&blend.color),
                                            alpha: convert_blend_component(&blend.alpha),
                                        }),
                                    })
                                })
                                .collect::<Vec<_>>(),
                        ),
                    }),
                primitive: convert_primitive_state(&descriptor.primitive),
                depth_stencil: descriptor.depthStencil.as_ref().map(|dss_desc| {
                    wgt::DepthStencilState {
                        format: convert_texture_format(dss_desc.format),
                        depth_write_enabled: dss_desc.depthWriteEnabled,
                        depth_compare: convert_compare_function(dss_desc.depthCompare),
                        stencil: wgt::StencilState {
                            front: wgt::StencilFaceState {
                                compare: convert_compare_function(dss_desc.stencilFront.compare),
                                fail_op: convert_stencil_op(dss_desc.stencilFront.failOp),
                                depth_fail_op: convert_stencil_op(
                                    dss_desc.stencilFront.depthFailOp,
                                ),
                                pass_op: convert_stencil_op(dss_desc.stencilFront.passOp),
                            },
                            back: wgt::StencilFaceState {
                                compare: convert_compare_function(dss_desc.stencilBack.compare),
                                fail_op: convert_stencil_op(dss_desc.stencilBack.failOp),
                                depth_fail_op: convert_stencil_op(dss_desc.stencilBack.depthFailOp),
                                pass_op: convert_stencil_op(dss_desc.stencilBack.passOp),
                            },
                            read_mask: dss_desc.stencilReadMask,
                            write_mask: dss_desc.stencilWriteMask,
                        },
                        bias: wgt::DepthBiasState {
                            constant: dss_desc.depthBias,
                            slope_scale: *dss_desc.depthBiasSlopeScale,
                            clamp: *dss_desc.depthBiasClamp,
                        },
                    }
                }),
                multisample: wgt::MultisampleState {
                    count: descriptor.multisample.count,
                    mask: descriptor.multisample.mask as u64,
                    alpha_to_coverage_enabled: descriptor.multisample.alphaToCoverageEnabled,
                },
                multiview: None,
            })
        } else {
            self.handle_server_msg(
                scope_id,
                WebGPUOpResult::ValidationError(String::from("Invalid GPUColorWriteFlags")),
            );
            None
        };

        let render_pipeline_id = self
            .global()
            .wgpu_id_hub()
            .lock()
            .create_render_pipeline_id(self.device.0.backend());

        self.channel
            .0
            .send((
                scope_id,
                WebGPURequest::CreateRenderPipeline {
                    device_id: self.device.0,
                    render_pipeline_id,
                    descriptor: desc,
                    implicit_ids,
                },
            ))
            .expect("Failed to create WebGPU render pipeline");

        let render_pipeline = webgpu::WebGPURenderPipeline(render_pipeline_id);

        GPURenderPipeline::new(
            &self.global(),
            render_pipeline,
            descriptor.parent.parent.label.clone().unwrap_or_default(),
            bgls,
            &self,
        )
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpudevice-createrenderbundleencoder
    fn CreateRenderBundleEncoder(
        &self,
        descriptor: &GPURenderBundleEncoderDescriptor,
    ) -> DomRoot<GPURenderBundleEncoder> {
        let desc = wgpu_com::RenderBundleEncoderDescriptor {
            label: convert_label(&descriptor.parent.parent),
            color_formats: Cow::Owned(
                descriptor
                    .parent
                    .colorFormats
                    .iter()
                    .map(|f| Some(convert_texture_format(*f)))
                    .collect::<Vec<_>>(),
            ),
            depth_stencil: Some(wgt::RenderBundleDepthStencil {
                format: convert_texture_format(descriptor.parent.depthStencilFormat.unwrap()),
                depth_read_only: descriptor.depthReadOnly,
                stencil_read_only: descriptor.stencilReadOnly,
            }),
            sample_count: descriptor.parent.sampleCount,
            multiview: None,
        };

        // Handle error gracefully
        let render_bundle_encoder =
            wgpu_com::RenderBundleEncoder::new(&desc, self.device.0, None).unwrap();

        GPURenderBundleEncoder::new(
            &self.global(),
            render_bundle_encoder,
            &self,
            self.channel.clone(),
            descriptor.parent.parent.label.clone().unwrap_or_default(),
        )
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpudevice-pusherrorscope
    fn PushErrorScope(&self, filter: GPUErrorFilter) {
        let mut context = self.scope_context.borrow_mut();
        let scope_id = context.next_scope_id;
        context.next_scope_id = ErrorScopeId::new(scope_id.get() + 1).unwrap();
        let err_scope = ErrorScopeInfo {
            op_count: 0,
            error: None,
            promise: None,
        };
        let res = context.error_scopes.insert(scope_id, err_scope);
        context.scope_stack.push(ErrorScopeMetadata {
            id: scope_id,
            filter,
            popped: Cell::new(false),
        });
        assert!(res.is_none());
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpudevice-poperrorscope
    fn PopErrorScope(&self, comp: InRealm) -> Rc<Promise> {
        let mut context = self.scope_context.borrow_mut();
        let promise = Promise::new_in_current_realm(comp);
        let scope_id =
            if let Some(meta) = context.scope_stack.iter().rev().find(|m| !m.popped.get()) {
                meta.popped.set(true);
                meta.id
            } else {
                promise.reject_error(Error::Operation);
                return promise;
            };
        let remove = if let Some(mut err_scope) = context.error_scopes.get_mut(&scope_id) {
            if let Some(ref e) = err_scope.error {
                promise.resolve_native(e);
            } else if err_scope.op_count == 0 {
                promise.resolve_native(&None::<GPUError>);
            }
            err_scope.promise = Some(promise.clone());
            err_scope.op_count == 0
        } else {
            error!("Could not find ErrorScope with Id({})", scope_id);
            false
        };
        if remove {
            let _ = context.error_scopes.remove(&scope_id);
            context.scope_stack.retain(|meta| meta.id != scope_id);
        }
        promise
    }

    // https://gpuweb.github.io/gpuweb/#dom-gpudevice-onuncapturederror
    event_handler!(uncapturederror, GetOnuncapturederror, SetOnuncapturederror);

    /// https://gpuweb.github.io/gpuweb/#dom-gpudevice-destroy
    fn Destroy(&self) {
        if self.valid.get() {
            self.valid.set(false);

            self.lose(GPUDeviceLostReason::Destroyed);

            if let Err(e) = self
                .channel
                .0
                .send((None, WebGPURequest::DestroyDevice(self.device.0)))
            {
                warn!("Failed to send DestroyDevice ({:?}) ({})", self.device.0, e);
            }
        }
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpudevice-createcomputepipelineasync
    fn CreateComputePipelineAsync(
        &self,
        _descriptor: &GPUComputePipelineDescriptor,
    ) -> Rc<Promise> {
        todo!()
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpudevice-createrenderpipelineasync
    fn CreateRenderPipelineAsync(&self, _descriptor: &GPURenderPipelineDescriptor) -> Rc<Promise> {
        todo!()
    }
}

impl Drop for GPUDevice {
    // not sure about this but this is non failable version of destroy
    fn drop(&mut self) {
        self.Destroy()
    }
}

fn convert_blend_component(blend_component: &GPUBlendComponent) -> wgt::BlendComponent {
    wgt::BlendComponent {
        src_factor: convert_blend_factor(&blend_component.srcFactor),
        dst_factor: convert_blend_factor(&blend_component.dstFactor),
        operation: match blend_component.operation {
            GPUBlendOperation::Add => wgt::BlendOperation::Add,
            GPUBlendOperation::Subtract => wgt::BlendOperation::Subtract,
            GPUBlendOperation::Reverse_subtract => wgt::BlendOperation::ReverseSubtract,
            GPUBlendOperation::Min => wgt::BlendOperation::Min,
            GPUBlendOperation::Max => wgt::BlendOperation::Max,
        },
    }
}

fn convert_primitive_state(primitive_state: &GPUPrimitiveState) -> wgt::PrimitiveState {
    wgt::PrimitiveState {
        topology: convert_primitive_topology(&primitive_state.topology),
        strip_index_format: primitive_state.stripIndexFormat.map(
            |index_format| match index_format {
                GPUIndexFormat::Uint16 => wgt::IndexFormat::Uint16,
                GPUIndexFormat::Uint32 => wgt::IndexFormat::Uint32,
            },
        ),
        front_face: match primitive_state.frontFace {
            GPUFrontFace::Ccw => wgt::FrontFace::Ccw,
            GPUFrontFace::Cw => wgt::FrontFace::Cw,
        },
        cull_mode: match primitive_state.cullMode {
            GPUCullMode::None => None,
            GPUCullMode::Front => Some(wgt::Face::Front),
            GPUCullMode::Back => Some(wgt::Face::Back),
        },
        unclipped_depth: primitive_state.clampDepth,
        ..Default::default()
    }
}

fn convert_primitive_topology(primitive_topology: &GPUPrimitiveTopology) -> wgt::PrimitiveTopology {
    match primitive_topology {
        GPUPrimitiveTopology::Point_list => wgt::PrimitiveTopology::PointList,
        GPUPrimitiveTopology::Line_list => wgt::PrimitiveTopology::LineList,
        GPUPrimitiveTopology::Line_strip => wgt::PrimitiveTopology::LineStrip,
        GPUPrimitiveTopology::Triangle_list => wgt::PrimitiveTopology::TriangleList,
        GPUPrimitiveTopology::Triangle_strip => wgt::PrimitiveTopology::TriangleStrip,
    }
}

fn convert_view_dimension(view_dimension: GPUTextureViewDimension) -> wgt::TextureViewDimension {
    match view_dimension {
        GPUTextureViewDimension::_1d => wgt::TextureViewDimension::D1,
        GPUTextureViewDimension::_2d => wgt::TextureViewDimension::D2,
        GPUTextureViewDimension::_2d_array => wgt::TextureViewDimension::D2Array,
        GPUTextureViewDimension::Cube => wgt::TextureViewDimension::Cube,
        GPUTextureViewDimension::Cube_array => wgt::TextureViewDimension::CubeArray,
        GPUTextureViewDimension::_3d => wgt::TextureViewDimension::D3,
    }
}

fn convert_address_mode(address_mode: GPUAddressMode) -> wgt::AddressMode {
    match address_mode {
        GPUAddressMode::Clamp_to_edge => wgt::AddressMode::ClampToEdge,
        GPUAddressMode::Repeat => wgt::AddressMode::Repeat,
        GPUAddressMode::Mirror_repeat => wgt::AddressMode::MirrorRepeat,
    }
}

fn convert_filter_mode(filter_mode: GPUFilterMode) -> wgt::FilterMode {
    match filter_mode {
        GPUFilterMode::Nearest => wgt::FilterMode::Nearest,
        GPUFilterMode::Linear => wgt::FilterMode::Linear,
    }
}

fn convert_compare_function(compare: GPUCompareFunction) -> wgt::CompareFunction {
    match compare {
        GPUCompareFunction::Never => wgt::CompareFunction::Never,
        GPUCompareFunction::Less => wgt::CompareFunction::Less,
        GPUCompareFunction::Equal => wgt::CompareFunction::Equal,
        GPUCompareFunction::Less_equal => wgt::CompareFunction::LessEqual,
        GPUCompareFunction::Greater => wgt::CompareFunction::Greater,
        GPUCompareFunction::Not_equal => wgt::CompareFunction::NotEqual,
        GPUCompareFunction::Greater_equal => wgt::CompareFunction::GreaterEqual,
        GPUCompareFunction::Always => wgt::CompareFunction::Always,
    }
}

fn convert_blend_factor(factor: &GPUBlendFactor) -> wgt::BlendFactor {
    match factor {
        GPUBlendFactor::Zero => wgt::BlendFactor::Zero,
        GPUBlendFactor::One => wgt::BlendFactor::One,
        GPUBlendFactor::Src => wgt::BlendFactor::Src,
        GPUBlendFactor::One_minus_src => wgt::BlendFactor::OneMinusSrc,
        GPUBlendFactor::Src_alpha => wgt::BlendFactor::SrcAlpha,
        GPUBlendFactor::One_minus_src_alpha => wgt::BlendFactor::OneMinusSrcAlpha,
        GPUBlendFactor::Dst => wgt::BlendFactor::Dst,
        GPUBlendFactor::One_minus_dst => wgt::BlendFactor::OneMinusDst,
        GPUBlendFactor::Dst_alpha => wgt::BlendFactor::DstAlpha,
        GPUBlendFactor::One_minus_dst_alpha => wgt::BlendFactor::OneMinusDstAlpha,
        GPUBlendFactor::Src_alpha_saturated => wgt::BlendFactor::SrcAlphaSaturated,
        GPUBlendFactor::Constant => wgt::BlendFactor::Constant,
        GPUBlendFactor::One_minus_constant => wgt::BlendFactor::OneMinusConstant,
    }
}

fn convert_stencil_op(operation: GPUStencilOperation) -> wgt::StencilOperation {
    match operation {
        GPUStencilOperation::Keep => wgt::StencilOperation::Keep,
        GPUStencilOperation::Zero => wgt::StencilOperation::Zero,
        GPUStencilOperation::Replace => wgt::StencilOperation::Replace,
        GPUStencilOperation::Invert => wgt::StencilOperation::Invert,
        GPUStencilOperation::Increment_clamp => wgt::StencilOperation::IncrementClamp,
        GPUStencilOperation::Decrement_clamp => wgt::StencilOperation::DecrementClamp,
        GPUStencilOperation::Increment_wrap => wgt::StencilOperation::IncrementWrap,
        GPUStencilOperation::Decrement_wrap => wgt::StencilOperation::DecrementWrap,
    }
}

fn convert_vertex_format(format: GPUVertexFormat) -> wgt::VertexFormat {
    match format {
        GPUVertexFormat::Uint8x2 => wgt::VertexFormat::Uint8x2,
        GPUVertexFormat::Uint8x4 => wgt::VertexFormat::Uint8x4,
        GPUVertexFormat::Sint8x2 => wgt::VertexFormat::Sint8x2,
        GPUVertexFormat::Sint8x4 => wgt::VertexFormat::Sint8x4,
        GPUVertexFormat::Unorm8x2 => wgt::VertexFormat::Unorm8x2,
        GPUVertexFormat::Unorm8x4 => wgt::VertexFormat::Unorm8x4,
        GPUVertexFormat::Snorm8x2 => wgt::VertexFormat::Unorm8x2,
        GPUVertexFormat::Snorm8x4 => wgt::VertexFormat::Unorm8x4,
        GPUVertexFormat::Uint16x2 => wgt::VertexFormat::Uint16x2,
        GPUVertexFormat::Uint16x4 => wgt::VertexFormat::Uint16x4,
        GPUVertexFormat::Sint16x2 => wgt::VertexFormat::Sint16x2,
        GPUVertexFormat::Sint16x4 => wgt::VertexFormat::Sint16x4,
        GPUVertexFormat::Unorm16x2 => wgt::VertexFormat::Unorm16x2,
        GPUVertexFormat::Unorm16x4 => wgt::VertexFormat::Unorm16x4,
        GPUVertexFormat::Snorm16x2 => wgt::VertexFormat::Snorm16x2,
        GPUVertexFormat::Snorm16x4 => wgt::VertexFormat::Snorm16x4,
        GPUVertexFormat::Float16x2 => wgt::VertexFormat::Float16x2,
        GPUVertexFormat::Float16x4 => wgt::VertexFormat::Float16x4,
        GPUVertexFormat::Float32 => wgt::VertexFormat::Float32,
        GPUVertexFormat::Float32x2 => wgt::VertexFormat::Float32x2,
        GPUVertexFormat::Float32x3 => wgt::VertexFormat::Float32x3,
        GPUVertexFormat::Float32x4 => wgt::VertexFormat::Float32x4,
        GPUVertexFormat::Uint32 => wgt::VertexFormat::Uint32,
        GPUVertexFormat::Uint32x2 => wgt::VertexFormat::Uint32x2,
        GPUVertexFormat::Uint32x3 => wgt::VertexFormat::Uint32x3,
        GPUVertexFormat::Uint32x4 => wgt::VertexFormat::Uint32x4,
        GPUVertexFormat::Sint32 => wgt::VertexFormat::Sint32,
        GPUVertexFormat::Sint32x2 => wgt::VertexFormat::Sint32x2,
        GPUVertexFormat::Sint32x3 => wgt::VertexFormat::Sint32x3,
        GPUVertexFormat::Sint32x4 => wgt::VertexFormat::Sint32x4,
    }
}

pub fn convert_texture_format(format: GPUTextureFormat) -> wgt::TextureFormat {
    match format {
        GPUTextureFormat::R8unorm => wgt::TextureFormat::R8Unorm,
        GPUTextureFormat::R8snorm => wgt::TextureFormat::R8Snorm,
        GPUTextureFormat::R8uint => wgt::TextureFormat::R8Uint,
        GPUTextureFormat::R8sint => wgt::TextureFormat::R8Sint,
        GPUTextureFormat::R16uint => wgt::TextureFormat::R16Uint,
        GPUTextureFormat::R16sint => wgt::TextureFormat::R16Sint,
        GPUTextureFormat::R16float => wgt::TextureFormat::R16Float,
        GPUTextureFormat::Rg8unorm => wgt::TextureFormat::Rg8Unorm,
        GPUTextureFormat::Rg8snorm => wgt::TextureFormat::Rg8Snorm,
        GPUTextureFormat::Rg8uint => wgt::TextureFormat::Rg8Uint,
        GPUTextureFormat::Rg8sint => wgt::TextureFormat::Rg8Sint,
        GPUTextureFormat::R32uint => wgt::TextureFormat::R32Uint,
        GPUTextureFormat::R32sint => wgt::TextureFormat::R32Sint,
        GPUTextureFormat::R32float => wgt::TextureFormat::R32Float,
        GPUTextureFormat::Rg16uint => wgt::TextureFormat::Rg16Uint,
        GPUTextureFormat::Rg16sint => wgt::TextureFormat::Rg16Sint,
        GPUTextureFormat::Rg16float => wgt::TextureFormat::Rg16Float,
        GPUTextureFormat::Rgba8unorm => wgt::TextureFormat::Rgba8Unorm,
        GPUTextureFormat::Rgba8unorm_srgb => wgt::TextureFormat::Rgba8UnormSrgb,
        GPUTextureFormat::Rgba8snorm => wgt::TextureFormat::Rgba8Snorm,
        GPUTextureFormat::Rgba8uint => wgt::TextureFormat::Rgba8Uint,
        GPUTextureFormat::Rgba8sint => wgt::TextureFormat::Rgba8Sint,
        GPUTextureFormat::Bgra8unorm => wgt::TextureFormat::Bgra8Unorm,
        GPUTextureFormat::Bgra8unorm_srgb => wgt::TextureFormat::Bgra8UnormSrgb,
        GPUTextureFormat::Rgb10a2unorm => wgt::TextureFormat::Rgb10a2Unorm,
        GPUTextureFormat::Rg32uint => wgt::TextureFormat::Rg32Uint,
        GPUTextureFormat::Rg32sint => wgt::TextureFormat::Rg32Sint,
        GPUTextureFormat::Rg32float => wgt::TextureFormat::Rg32Float,
        GPUTextureFormat::Rgba16uint => wgt::TextureFormat::Rgba16Uint,
        GPUTextureFormat::Rgba16sint => wgt::TextureFormat::Rgba16Sint,
        GPUTextureFormat::Rgba16float => wgt::TextureFormat::Rgba16Float,
        GPUTextureFormat::Rgba32uint => wgt::TextureFormat::Rgba32Uint,
        GPUTextureFormat::Rgba32sint => wgt::TextureFormat::Rgba32Sint,
        GPUTextureFormat::Rgba32float => wgt::TextureFormat::Rgba32Float,
        GPUTextureFormat::Depth32float => wgt::TextureFormat::Depth32Float,
        GPUTextureFormat::Depth24plus => wgt::TextureFormat::Depth24Plus,
        GPUTextureFormat::Depth24plus_stencil8 => wgt::TextureFormat::Depth24PlusStencil8,
        GPUTextureFormat::Bc1_rgba_unorm => wgt::TextureFormat::Bc1RgbaUnorm,
        GPUTextureFormat::Bc1_rgba_unorm_srgb => wgt::TextureFormat::Bc1RgbaUnormSrgb,
        GPUTextureFormat::Bc2_rgba_unorm => wgt::TextureFormat::Bc2RgbaUnorm,
        GPUTextureFormat::Bc2_rgba_unorm_srgb => wgt::TextureFormat::Bc2RgbaUnormSrgb,
        GPUTextureFormat::Bc3_rgba_unorm => wgt::TextureFormat::Bc3RgbaUnorm,
        GPUTextureFormat::Bc3_rgba_unorm_srgb => wgt::TextureFormat::Bc3RgbaUnormSrgb,
        GPUTextureFormat::Bc4_r_unorm => wgt::TextureFormat::Bc4RUnorm,
        GPUTextureFormat::Bc4_r_snorm => wgt::TextureFormat::Bc4RSnorm,
        GPUTextureFormat::Bc5_rg_unorm => wgt::TextureFormat::Bc5RgUnorm,
        GPUTextureFormat::Bc5_rg_snorm => wgt::TextureFormat::Bc5RgSnorm,
        GPUTextureFormat::Bc6h_rgb_ufloat => wgt::TextureFormat::Bc6hRgbUfloat,
        GPUTextureFormat::Bc7_rgba_unorm => wgt::TextureFormat::Bc7RgbaUnorm,
        GPUTextureFormat::Bc7_rgba_unorm_srgb => wgt::TextureFormat::Bc7RgbaUnormSrgb,
        GPUTextureFormat::Rg11b10float => wgt::TextureFormat::Rg11b10Float,
        GPUTextureFormat::Bc6h_rgb_float => wgt::TextureFormat::Bc6hRgbFloat,
    }
}

pub fn convert_texture_view_dimension(
    dimension: GPUTextureViewDimension,
) -> wgt::TextureViewDimension {
    match dimension {
        GPUTextureViewDimension::_1d => wgt::TextureViewDimension::D1,
        GPUTextureViewDimension::_2d => wgt::TextureViewDimension::D2,
        GPUTextureViewDimension::_2d_array => wgt::TextureViewDimension::D2Array,
        GPUTextureViewDimension::Cube => wgt::TextureViewDimension::Cube,
        GPUTextureViewDimension::Cube_array => wgt::TextureViewDimension::CubeArray,
        GPUTextureViewDimension::_3d => wgt::TextureViewDimension::D3,
    }
}

pub fn convert_texture_size_to_dict(size: &GPUExtent3D) -> GPUExtent3DDict {
    match *size {
        GPUExtent3D::GPUExtent3DDict(ref dict) => GPUExtent3DDict {
            width: dict.width,
            height: dict.height,
            depthOrArrayLayers: dict.depthOrArrayLayers,
        },
        GPUExtent3D::RangeEnforcedUnsignedLongSequence(ref v) => {
            let mut w = v.clone();
            w.resize(3, 1);
            GPUExtent3DDict {
                width: w[0],
                height: w[1],
                depthOrArrayLayers: w[2],
            }
        },
    }
}

pub fn convert_texture_size_to_wgt(size: &GPUExtent3DDict) -> wgt::Extent3d {
    wgt::Extent3d {
        width: size.width,
        height: size.height,
        depth_or_array_layers: size.depthOrArrayLayers,
    }
}

pub fn convert_label(parent: &GPUObjectDescriptorBase) -> Option<Cow<'static, str>> {
    parent.label.as_ref().map(|s| Cow::Owned(s.to_string()))
}
