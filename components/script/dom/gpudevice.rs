/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

use std::borrow::Cow;
use std::cell::Cell;
use std::collections::HashMap;
use std::num::NonZeroU64;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use dom_struct::dom_struct;
use js::jsapi::{Heap, JSObject};
use webgpu::wgc::id::{BindGroupLayoutId, PipelineLayoutId};
use webgpu::wgc::{
    binding_model as wgpu_bind, command as wgpu_com, pipeline as wgpu_pipe, resource as wgpu_res,
};
use webgpu::{self, wgt, PopError, WebGPU, WebGPURequest, WebGPUResponse};

use super::bindings::codegen::UnionTypes::GPUPipelineLayoutOrGPUAutoLayoutMode;
use super::bindings::error::Fallible;
use super::gpu::AsyncWGPUListener;
use super::gpudevicelostinfo::GPUDeviceLostInfo;
use super::gpusupportedlimits::GPUSupportedLimits;
use super::types::GPUError;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::EventBinding::EventInit;
use crate::dom::bindings::codegen::Bindings::EventTargetBinding::EventTargetMethods;
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::{
    GPUBindGroupDescriptor, GPUBindGroupLayoutDescriptor, GPUBindingResource, GPUBufferBindingType,
    GPUBufferDescriptor, GPUCommandEncoderDescriptor, GPUComputePipelineDescriptor,
    GPUDeviceLostReason, GPUDeviceMethods, GPUErrorFilter, GPUPipelineLayoutDescriptor,
    GPURenderBundleEncoderDescriptor, GPURenderPipelineDescriptor, GPUSamplerBindingType,
    GPUSamplerDescriptor, GPUShaderModuleDescriptor, GPUStorageTextureAccess,
    GPUSupportedLimitsMethods, GPUTextureDescriptor, GPUTextureDimension, GPUTextureSampleType,
    GPUUncapturedErrorEventInit, GPUVertexStepMode,
};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::gpu::response_async;
use crate::dom::gpuadapter::GPUAdapter;
use crate::dom::gpubindgroup::GPUBindGroup;
use crate::dom::gpubindgrouplayout::GPUBindGroupLayout;
use crate::dom::gpubuffer::{GPUBuffer, GPUBufferMapInfo, GPUBufferState};
use crate::dom::gpucommandencoder::GPUCommandEncoder;
use crate::dom::gpucomputepipeline::GPUComputePipeline;
use crate::dom::gpuconvert::{
    convert_address_mode, convert_blend_component, convert_compare_function, convert_filter_mode,
    convert_label, convert_primitive_state, convert_stencil_op, convert_texture_format,
    convert_texture_size_to_dict, convert_texture_size_to_wgt, convert_vertex_format,
    convert_view_dimension,
};
use crate::dom::gpupipelinelayout::GPUPipelineLayout;
use crate::dom::gpuqueue::GPUQueue;
use crate::dom::gpurenderbundleencoder::GPURenderBundleEncoder;
use crate::dom::gpurenderpipeline::GPURenderPipeline;
use crate::dom::gpusampler::GPUSampler;
use crate::dom::gpushadermodule::GPUShaderModule;
use crate::dom::gpusupportedfeatures::GPUSupportedFeatures;
use crate::dom::gputexture::GPUTexture;
use crate::dom::gpuuncapturederrorevent::GPUUncapturedErrorEvent;
use crate::dom::promise::Promise;
use crate::realms::InRealm;

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
    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-lost>
    #[ignore_malloc_size_of = "promises are hard"]
    lost_promise: DomRefCell<Rc<Promise>>,
    valid: Cell<bool>,
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
        let lost_promise = Promise::new(global);
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
        );
        queue.set_device(&device);
        device
    }
}

impl GPUDevice {
    pub fn id(&self) -> webgpu::WebGPUDevice {
        self.device
    }

    pub fn channel(&self) -> WebGPU {
        self.channel.clone()
    }

    pub fn dispatch_error(&self, error: webgpu::Error) {
        if let Err(e) = self.channel.0.send(WebGPURequest::DispatchError {
            device_id: self.device.0,
            error,
        }) {
            warn!("Failed to send WebGPURequest::DispatchError due to {e:?}");
        }
    }

    pub fn fire_uncaptured_error(&self, error: webgpu::Error) {
        let error = GPUError::from_error(&self.global(), error);
        let ev = GPUUncapturedErrorEvent::new(
            &self.global(),
            DOMString::from("uncapturederror"),
            &GPUUncapturedErrorEventInit {
                error,
                parent: EventInit::empty(),
            },
        );
        let _ = self.eventtarget.DispatchEvent(ev.event());
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
                .create_pipeline_layout_id(self.device.0.backend());
            let max_bind_grps = self.limits.MaxBindGroups();
            let mut bgls = Vec::with_capacity(max_bind_grps as usize);
            let mut bgl_ids = Vec::with_capacity(max_bind_grps as usize);
            for _ in 0..max_bind_grps {
                let bgl = self
                    .global()
                    .wgpu_id_hub()
                    .create_bind_group_layout_id(self.device.0.backend());
                bgls.push(webgpu::WebGPUBindGroupLayout(bgl));
                bgl_ids.push(bgl);
            }
            (None, Some((layout_id, bgl_ids)), bgls)
        }
    }

    /// <https://gpuweb.github.io/gpuweb/#lose-the-device>
    pub fn lose(&self, reason: GPUDeviceLostReason, msg: String) {
        let lost_promise = &(*self.lost_promise.borrow());
        let global = &self.global();
        let lost = GPUDeviceLostInfo::new(global, msg.into(), reason);
        lost_promise.resolve_native(&*lost);
    }
}

impl GPUDeviceMethods for GPUDevice {
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
            .create_buffer_id(self.device.0.backend());

        if desc.is_none() {
            self.dispatch_error(webgpu::Error::Validation(String::from(
                "Invalid GPUBufferUsage",
            )));
        }

        self.channel
            .0
            .send(WebGPURequest::CreateBuffer {
                device_id: self.device.0,
                buffer_id: id,
                descriptor: desc,
            })
            .expect("Failed to create WebGPU buffer");

        let buffer = webgpu::WebGPUBuffer(id);
        let map_info;
        let state;
        if descriptor.mappedAtCreation {
            let buf_data = vec![0u8; descriptor.size as usize];
            map_info = DomRefCell::new(Some(GPUBufferMapInfo {
                mapping: Arc::new(Mutex::new(buf_data)),
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
            self,
            state,
            descriptor.size,
            map_info,
            descriptor.parent.label.clone().unwrap_or_default(),
        ))
    }

    /// <https://gpuweb.github.io/gpuweb/#GPUDevice-createBindGroupLayout>
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
                    visibility,
                    ty,
                    count: None,
                }
            })
            .collect::<Vec<_>>();

        let desc = if valid {
            Some(wgpu_bind::BindGroupLayoutDescriptor {
                label: convert_label(&descriptor.parent),
                entries: Cow::Owned(entries),
            })
        } else {
            self.dispatch_error(webgpu::Error::Validation(String::from(
                "Invalid GPUShaderStage",
            )));
            None
        };

        let bind_group_layout_id = self
            .global()
            .wgpu_id_hub()
            .create_bind_group_layout_id(self.device.0.backend());
        self.channel
            .0
            .send(WebGPURequest::CreateBindGroupLayout {
                device_id: self.device.0,
                bind_group_layout_id,
                descriptor: desc,
            })
            .expect("Failed to create WebGPU BindGroupLayout");

        let bgl = webgpu::WebGPUBindGroupLayout(bind_group_layout_id);

        GPUBindGroupLayout::new(
            &self.global(),
            self.channel.clone(),
            bgl,
            descriptor.parent.label.clone().unwrap_or_default(),
        )
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createpipelinelayout>
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

        let pipeline_layout_id = self
            .global()
            .wgpu_id_hub()
            .create_pipeline_layout_id(self.device.0.backend());
        self.channel
            .0
            .send(WebGPURequest::CreatePipelineLayout {
                device_id: self.device.0,
                pipeline_layout_id,
                descriptor: desc,
            })
            .expect("Failed to create WebGPU PipelineLayout");

        let bgls = descriptor
            .bindGroupLayouts
            .iter()
            .map(|each| each.id())
            .collect::<Vec<_>>();
        let pipeline_layout = webgpu::WebGPUPipelineLayout(pipeline_layout_id);
        GPUPipelineLayout::new(
            &self.global(),
            self.channel.clone(),
            pipeline_layout,
            descriptor.parent.label.clone().unwrap_or_default(),
            bgls,
        )
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createbindgroup>
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

        let bind_group_id = self
            .global()
            .wgpu_id_hub()
            .create_bind_group_id(self.device.0.backend());
        self.channel
            .0
            .send(WebGPURequest::CreateBindGroup {
                device_id: self.device.0,
                bind_group_id,
                descriptor: desc,
            })
            .expect("Failed to create WebGPU BindGroup");

        let bind_group = webgpu::WebGPUBindGroup(bind_group_id);

        GPUBindGroup::new(
            &self.global(),
            self.channel.clone(),
            bind_group,
            self.device,
            &descriptor.layout,
            descriptor.parent.label.clone().unwrap_or_default(),
        )
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createshadermodule>
    fn CreateShaderModule(
        &self,
        descriptor: RootedTraceableBox<GPUShaderModuleDescriptor>,
        comp: InRealm,
    ) -> DomRoot<GPUShaderModule> {
        let program_id = self
            .global()
            .wgpu_id_hub()
            .create_shader_module_id(self.device.0.backend());
        let promise = Promise::new_in_current_realm(comp);
        let shader_module = GPUShaderModule::new(
            &self.global(),
            self.channel.clone(),
            webgpu::WebGPUShaderModule(program_id),
            descriptor.parent.label.clone().unwrap_or_default(),
            promise.clone(),
        );
        let sender = response_async(&promise, &*shader_module);
        self.channel
            .0
            .send(WebGPURequest::CreateShaderModule {
                device_id: self.device.0,
                program_id,
                program: descriptor.code.0.clone(),
                label: None,
                sender,
            })
            .expect("Failed to create WebGPU ShaderModule");
        shader_module
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createcomputepipeline>
    fn CreateComputePipeline(
        &self,
        descriptor: &GPUComputePipelineDescriptor,
    ) -> DomRoot<GPUComputePipeline> {
        let compute_pipeline_id = self
            .global()
            .wgpu_id_hub()
            .create_compute_pipeline_id(self.device.0.backend());

        let (layout, implicit_ids, bgls) = self.get_pipeline_layout_data(&descriptor.parent.layout);

        let desc = wgpu_pipe::ComputePipelineDescriptor {
            label: convert_label(&descriptor.parent.parent),
            layout,
            stage: wgpu_pipe::ProgrammableStageDescriptor {
                module: descriptor.compute.module.id().0,
                entry_point: Some(Cow::Owned(descriptor.compute.entryPoint.to_string())),
                constants: Cow::Owned(HashMap::new()),
                zero_initialize_workgroup_memory: true,
                vertex_pulling_transform: false,
            },
            cache: None,
        };

        self.channel
            .0
            .send(WebGPURequest::CreateComputePipeline {
                device_id: self.device.0,
                compute_pipeline_id,
                descriptor: desc,
                implicit_ids,
            })
            .expect("Failed to create WebGPU ComputePipeline");

        let compute_pipeline = webgpu::WebGPUComputePipeline(compute_pipeline_id);
        GPUComputePipeline::new(
            &self.global(),
            self.channel.clone(),
            compute_pipeline,
            descriptor.parent.parent.label.clone().unwrap_or_default(),
            bgls,
            self,
        )
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createcomputepipelineasync>
    fn CreateComputePipelineAsync(
        &self,
        descriptor: &GPUComputePipelineDescriptor,
        comp: InRealm,
    ) -> Rc<Promise> {
        let promise = Promise::new_in_current_realm(comp);
        promise.resolve_native(&self.CreateComputePipeline(descriptor));
        promise
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createcommandencoder>
    fn CreateCommandEncoder(
        &self,
        descriptor: &GPUCommandEncoderDescriptor,
    ) -> DomRoot<GPUCommandEncoder> {
        let command_encoder_id = self
            .global()
            .wgpu_id_hub()
            .create_command_encoder_id(self.device.0.backend());
        self.channel
            .0
            .send(WebGPURequest::CreateCommandEncoder {
                device_id: self.device.0,
                command_encoder_id,
                label: convert_label(&descriptor.parent),
            })
            .expect("Failed to create WebGPU command encoder");

        let encoder = webgpu::WebGPUCommandEncoder(command_encoder_id);

        GPUCommandEncoder::new(
            &self.global(),
            self.channel.clone(),
            self,
            encoder,
            descriptor.parent.label.clone().unwrap_or_default(),
        )
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createtexture>
    fn CreateTexture(&self, descriptor: &GPUTextureDescriptor) -> Fallible<DomRoot<GPUTexture>> {
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
            .create_texture_id(self.device.0.backend());

        if desc.is_none() {
            return Err(Error::Type(String::from("Invalid GPUTextureUsage")));
        }
        self.channel
            .0
            .send(WebGPURequest::CreateTexture {
                device_id: self.device.0,
                texture_id,
                descriptor: desc,
            })
            .expect("Failed to create WebGPU Texture");

        let texture = webgpu::WebGPUTexture(texture_id);

        Ok(GPUTexture::new(
            &self.global(),
            texture,
            self,
            self.channel.clone(),
            size,
            descriptor.mipLevelCount,
            descriptor.sampleCount,
            descriptor.dimension,
            descriptor.format,
            descriptor.usage,
            descriptor.parent.label.clone().unwrap_or_default(),
        ))
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createsampler>
    fn CreateSampler(&self, descriptor: &GPUSamplerDescriptor) -> DomRoot<GPUSampler> {
        let sampler_id = self
            .global()
            .wgpu_id_hub()
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
            compare: descriptor.compare.map(convert_compare_function),
            anisotropy_clamp: 1,
            border_color: None,
        };

        self.channel
            .0
            .send(WebGPURequest::CreateSampler {
                device_id: self.device.0,
                sampler_id,
                descriptor: desc,
            })
            .expect("Failed to create WebGPU sampler");

        let sampler = webgpu::WebGPUSampler(sampler_id);

        GPUSampler::new(
            &self.global(),
            self.channel.clone(),
            self.device,
            compare_enable,
            sampler,
            descriptor.parent.label.clone().unwrap_or_default(),
        )
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createrenderpipeline>
    fn CreateRenderPipeline(
        &self,
        descriptor: &GPURenderPipelineDescriptor,
    ) -> DomRoot<GPURenderPipeline> {
        let mut valid = true;

        let (layout, implicit_ids, bgls) = self.get_pipeline_layout_data(&descriptor.parent.layout);

        let desc = if valid {
            Some(wgpu_pipe::RenderPipelineDescriptor {
                label: convert_label(&descriptor.parent.parent),
                layout,
                cache: None,
                vertex: wgpu_pipe::VertexState {
                    stage: wgpu_pipe::ProgrammableStageDescriptor {
                        module: descriptor.vertex.parent.module.id().0,
                        entry_point: Some(Cow::Owned(
                            descriptor.vertex.parent.entryPoint.to_string(),
                        )),
                        constants: Cow::Owned(HashMap::new()),
                        zero_initialize_workgroup_memory: true,
                        vertex_pulling_transform: false,
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
                            entry_point: Some(Cow::Owned(stage.parent.entryPoint.to_string())),
                            constants: Cow::Owned(HashMap::new()),
                            zero_initialize_workgroup_memory: true,
                            vertex_pulling_transform: false,
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
            self.dispatch_error(webgpu::Error::Validation(String::from(
                "Invalid GPUColorWriteFlags",
            )));
            None
        };

        let render_pipeline_id = self
            .global()
            .wgpu_id_hub()
            .create_render_pipeline_id(self.device.0.backend());

        self.channel
            .0
            .send(WebGPURequest::CreateRenderPipeline {
                device_id: self.device.0,
                render_pipeline_id,
                descriptor: desc,
                implicit_ids,
            })
            .expect("Failed to create WebGPU render pipeline");

        let render_pipeline = webgpu::WebGPURenderPipeline(render_pipeline_id);

        GPURenderPipeline::new(
            &self.global(),
            render_pipeline,
            descriptor.parent.parent.label.clone().unwrap_or_default(),
            bgls,
            self,
        )
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createrenderpipelineasync>
    fn CreateRenderPipelineAsync(
        &self,
        descriptor: &GPURenderPipelineDescriptor,
        comp: InRealm,
    ) -> Rc<Promise> {
        let promise = Promise::new_in_current_realm(comp);
        promise.resolve_native(&self.CreateRenderPipeline(descriptor));
        promise
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createrenderbundleencoder>
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
            depth_stencil: descriptor.parent.depthStencilFormat.map(|dsf| {
                wgt::RenderBundleDepthStencil {
                    format: convert_texture_format(dsf),
                    depth_read_only: descriptor.depthReadOnly,
                    stencil_read_only: descriptor.stencilReadOnly,
                }
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
            self,
            self.channel.clone(),
            descriptor.parent.parent.label.clone().unwrap_or_default(),
        )
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
    fn PopErrorScope(&self, comp: InRealm) -> Rc<Promise> {
        let promise = Promise::new_in_current_realm(comp);
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
    fn handle_response(&self, response: WebGPUResponse, promise: &Rc<Promise>) {
        match response {
            WebGPUResponse::PoppedErrorScope(result) => match result {
                Ok(None) | Err(PopError::Lost) => promise.resolve_native(&None::<Option<GPUError>>),
                Err(PopError::Empty) => promise.reject_error(Error::Operation),
                Ok(Some(error)) => {
                    let error = GPUError::from_error(&self.global(), error);
                    promise.resolve_native(&error);
                },
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
