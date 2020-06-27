/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::GPUBindGroupBinding::{
    GPUBindGroupDescriptor, GPUBindingResource,
};
use crate::dom::bindings::codegen::Bindings::GPUBindGroupLayoutBinding::{
    GPUBindGroupLayoutDescriptor, GPUBindingType,
};
use crate::dom::bindings::codegen::Bindings::GPUBufferBinding::GPUBufferDescriptor;
use crate::dom::bindings::codegen::Bindings::GPUComputePipelineBinding::GPUComputePipelineDescriptor;
use crate::dom::bindings::codegen::Bindings::GPUDeviceBinding::{
    GPUCommandEncoderDescriptor, GPUDeviceMethods,
};
use crate::dom::bindings::codegen::Bindings::GPUPipelineLayoutBinding::GPUPipelineLayoutDescriptor;
use crate::dom::bindings::codegen::Bindings::GPURenderPipelineBinding::{
    GPUBlendDescriptor, GPUBlendFactor, GPUBlendOperation, GPUCullMode, GPUFrontFace,
    GPUIndexFormat, GPUInputStepMode, GPUPrimitiveTopology, GPURenderPipelineDescriptor,
    GPUStencilOperation, GPUVertexFormat,
};
use crate::dom::bindings::codegen::Bindings::GPUSamplerBinding::{
    GPUAddressMode, GPUCompareFunction, GPUFilterMode, GPUSamplerDescriptor,
};
use crate::dom::bindings::codegen::Bindings::GPUShaderModuleBinding::GPUShaderModuleDescriptor;
use crate::dom::bindings::codegen::Bindings::GPUTextureBinding::{
    GPUExtent3D, GPUExtent3DDict, GPUTextureComponentType, GPUTextureDescriptor,
    GPUTextureDimension, GPUTextureFormat,
};
use crate::dom::bindings::codegen::Bindings::GPUTextureViewBinding::GPUTextureViewDimension;
use crate::dom::bindings::codegen::UnionTypes::Uint32ArrayOrString::{String, Uint32Array};
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::gpuadapter::GPUAdapter;
use crate::dom::gpubindgroup::GPUBindGroup;
use crate::dom::gpubindgrouplayout::GPUBindGroupLayout;
use crate::dom::gpubuffer::{GPUBuffer, GPUBufferState};
use crate::dom::gpucommandencoder::GPUCommandEncoder;
use crate::dom::gpucomputepipeline::GPUComputePipeline;
use crate::dom::gpupipelinelayout::GPUPipelineLayout;
use crate::dom::gpuqueue::GPUQueue;
use crate::dom::gpurenderpipeline::GPURenderPipeline;
use crate::dom::gpusampler::GPUSampler;
use crate::dom::gpushadermodule::GPUShaderModule;
use crate::dom::gputexture::GPUTexture;
use crate::script_runtime::JSContext as SafeJSContext;
use arrayvec::ArrayVec;
use dom_struct::dom_struct;
use js::jsapi::{Heap, JSObject};
use js::typedarray::{ArrayBuffer, CreateWith};
use std::ptr::{self, NonNull};
use webgpu::wgpu::binding_model::BufferBinding;
use webgpu::{self, wgt, WebGPU, WebGPUBindings, WebGPURequest};

#[dom_struct]
pub struct GPUDevice {
    eventtarget: EventTarget,
    #[ignore_malloc_size_of = "channels are hard"]
    channel: WebGPU,
    adapter: Dom<GPUAdapter>,
    #[ignore_malloc_size_of = "mozjs"]
    extensions: Heap<*mut JSObject>,
    #[ignore_malloc_size_of = "mozjs"]
    limits: Heap<*mut JSObject>,
    label: DomRefCell<Option<DOMString>>,
    device: webgpu::WebGPUDevice,
    default_queue: Dom<GPUQueue>,
}

impl GPUDevice {
    fn new_inherited(
        channel: WebGPU,
        adapter: &GPUAdapter,
        extensions: Heap<*mut JSObject>,
        limits: Heap<*mut JSObject>,
        device: webgpu::WebGPUDevice,
        queue: &GPUQueue,
    ) -> Self {
        Self {
            eventtarget: EventTarget::new_inherited(),
            channel,
            adapter: Dom::from_ref(adapter),
            extensions,
            limits,
            label: DomRefCell::new(None),
            device,
            default_queue: Dom::from_ref(queue),
        }
    }

    pub fn new(
        global: &GlobalScope,
        channel: WebGPU,
        adapter: &GPUAdapter,
        extensions: Heap<*mut JSObject>,
        limits: Heap<*mut JSObject>,
        device: webgpu::WebGPUDevice,
        queue: webgpu::WebGPUQueue,
    ) -> DomRoot<Self> {
        let queue = GPUQueue::new(global, channel.clone(), queue);
        reflect_dom_object(
            Box::new(GPUDevice::new_inherited(
                channel, adapter, extensions, limits, device, &queue,
            )),
            global,
        )
    }
}

impl GPUDevice {
    pub fn id(&self) -> webgpu::WebGPUDevice {
        self.device
    }
}

impl GPUDeviceMethods for GPUDevice {
    /// https://gpuweb.github.io/gpuweb/#dom-gpudevice-adapter
    fn Adapter(&self) -> DomRoot<GPUAdapter> {
        DomRoot::from_ref(&self.adapter)
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpudevice-extensions
    fn Extensions(&self, _cx: SafeJSContext) -> NonNull<JSObject> {
        NonNull::new(self.extensions.get()).unwrap()
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpudevice-limits
    fn Limits(&self, _cx: SafeJSContext) -> NonNull<JSObject> {
        NonNull::new(self.extensions.get()).unwrap()
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpudevice-defaultqueue
    fn DefaultQueue(&self) -> DomRoot<GPUQueue> {
        DomRoot::from_ref(&self.default_queue)
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn GetLabel(&self) -> Option<DOMString> {
        self.label.borrow().clone()
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn SetLabel(&self, value: Option<DOMString>) {
        *self.label.borrow_mut() = value;
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpudevice-createbuffer
    fn CreateBuffer(&self, descriptor: &GPUBufferDescriptor) -> DomRoot<GPUBuffer> {
        let wgpu_descriptor = wgt::BufferDescriptor {
            label: Default::default(),
            size: descriptor.size,
            usage: match wgt::BufferUsage::from_bits(descriptor.usage) {
                Some(u) => u,
                None => wgt::BufferUsage::empty(),
            },
            mapped_at_creation: descriptor.mappedAtCreation,
        };
        let id = self
            .global()
            .wgpu_id_hub()
            .lock()
            .create_buffer_id(self.device.0.backend());
        self.channel
            .0
            .send(WebGPURequest::CreateBuffer {
                device_id: self.device.0,
                buffer_id: id,
                descriptor: wgpu_descriptor,
            })
            .expect("Failed to create WebGPU buffer");

        let buffer = webgpu::WebGPUBuffer(id);
        let mapping = RootedTraceableBox::new(Heap::default());
        let state;
        let mapping_range;
        if descriptor.mappedAtCreation {
            let cx = self.global().get_cx();
            rooted!(in(*cx) let mut array_buffer = ptr::null_mut::<JSObject>());
            unsafe {
                assert!(ArrayBuffer::create(
                    *cx,
                    CreateWith::Length(descriptor.size as u32),
                    array_buffer.handle_mut(),
                )
                .is_ok());
            }
            mapping.set(array_buffer.get());
            state = GPUBufferState::MappedAtCreation;
            mapping_range = 0..descriptor.size;
        } else {
            state = GPUBufferState::Unmapped;
            mapping_range = 0..0;
        }

        GPUBuffer::new(
            &self.global(),
            self.channel.clone(),
            buffer,
            self.device,
            state,
            descriptor.size,
            true,
            mapping,
            mapping_range,
        )
    }

    /// https://gpuweb.github.io/gpuweb/#GPUDevice-createBindGroupLayout
    #[allow(non_snake_case)]
    fn CreateBindGroupLayout(
        &self,
        descriptor: &GPUBindGroupLayoutDescriptor,
    ) -> DomRoot<GPUBindGroupLayout> {
        let entries = descriptor
            .entries
            .iter()
            .map(|bind| {
                let visibility = match wgt::ShaderStage::from_bits(bind.visibility) {
                    Some(visibility) => visibility,
                    None => wgt::ShaderStage::from_bits(0).unwrap(),
                };
                let ty = match bind.type_ {
                    GPUBindingType::Uniform_buffer => wgt::BindingType::UniformBuffer {
                        dynamic: bind.hasDynamicOffset,
                        min_binding_size: wgt::BufferSize::new(bind.minBufferBindingSize),
                    },
                    GPUBindingType::Storage_buffer => wgt::BindingType::StorageBuffer {
                        dynamic: bind.hasDynamicOffset,
                        min_binding_size: wgt::BufferSize::new(bind.minBufferBindingSize),
                        readonly: false,
                    },
                    GPUBindingType::Readonly_storage_buffer => wgt::BindingType::StorageBuffer {
                        dynamic: bind.hasDynamicOffset,
                        min_binding_size: wgt::BufferSize::new(bind.minBufferBindingSize),
                        readonly: true,
                    },
                    GPUBindingType::Sampled_texture => wgt::BindingType::SampledTexture {
                        dimension: bind
                            .viewDimension
                            .map_or(wgt::TextureViewDimension::D2, |v| {
                                convert_texture_view_dimension(v)
                            }),
                        component_type: if let Some(c) = bind.textureComponentType {
                            match c {
                                GPUTextureComponentType::Float => wgt::TextureComponentType::Float,
                                GPUTextureComponentType::Sint => wgt::TextureComponentType::Sint,
                                GPUTextureComponentType::Uint => wgt::TextureComponentType::Uint,
                            }
                        } else {
                            wgt::TextureComponentType::Float
                        },
                        multisampled: bind.multisampled,
                    },
                    GPUBindingType::Readonly_storage_texture => wgt::BindingType::StorageTexture {
                        dimension: bind
                            .viewDimension
                            .map_or(wgt::TextureViewDimension::D2, |v| {
                                convert_texture_view_dimension(v)
                            }),
                        format: bind
                            .storageTextureFormat
                            .map_or(wgt::TextureFormat::Bgra8UnormSrgb, |f| {
                                convert_texture_format(f)
                            }),
                        readonly: true,
                    },
                    GPUBindingType::Writeonly_storage_texture => wgt::BindingType::StorageTexture {
                        dimension: bind
                            .viewDimension
                            .map_or(wgt::TextureViewDimension::D2, |v| {
                                convert_texture_view_dimension(v)
                            }),
                        format: bind
                            .storageTextureFormat
                            .map_or(wgt::TextureFormat::Bgra8UnormSrgb, |f| {
                                convert_texture_format(f)
                            }),
                        readonly: true,
                    },
                    GPUBindingType::Sampler => wgt::BindingType::Sampler { comparison: false },
                    GPUBindingType::Comparison_sampler => {
                        wgt::BindingType::Sampler { comparison: true }
                    },
                };

                wgt::BindGroupLayoutEntry::new(bind.binding, visibility, ty)
            })
            .collect::<Vec<_>>();

        let bind_group_layout_id = self
            .global()
            .wgpu_id_hub()
            .lock()
            .create_bind_group_layout_id(self.device.0.backend());
        self.channel
            .0
            .send(WebGPURequest::CreateBindGroupLayout {
                device_id: self.device.0,
                bind_group_layout_id,
                entries,
            })
            .expect("Failed to create WebGPU BindGroupLayout");

        let bgl = webgpu::WebGPUBindGroupLayout(bind_group_layout_id);

        GPUBindGroupLayout::new(&self.global(), bgl, true)
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpudevice-createpipelinelayout
    fn CreatePipelineLayout(
        &self,
        descriptor: &GPUPipelineLayoutDescriptor,
    ) -> DomRoot<GPUPipelineLayout> {
        let mut bgl_ids = Vec::new();
        descriptor.bindGroupLayouts.iter().for_each(|each| {
            if each.is_valid() {
                bgl_ids.push(each.id().0);
            }
        });

        let pipeline_layout_id = self
            .global()
            .wgpu_id_hub()
            .lock()
            .create_pipeline_layout_id(self.device.0.backend());
        self.channel
            .0
            .send(WebGPURequest::CreatePipelineLayout {
                device_id: self.device.0,
                pipeline_layout_id,
                bind_group_layouts: bgl_ids,
            })
            .expect("Failed to create WebGPU PipelineLayout");

        let pipeline_layout = webgpu::WebGPUPipelineLayout(pipeline_layout_id);
        GPUPipelineLayout::new(&self.global(), pipeline_layout, true)
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpudevice-createbindgroup
    fn CreateBindGroup(&self, descriptor: &GPUBindGroupDescriptor) -> DomRoot<GPUBindGroup> {
        let mut valid = descriptor.layout.is_valid();
        let entries = descriptor
            .entries
            .iter()
            .map(|bind| {
                (
                    bind.binding,
                    match bind.resource {
                        GPUBindingResource::GPUSampler(ref s) => {
                            valid &= s.is_valid();
                            WebGPUBindings::Sampler(s.id().0)
                        },
                        GPUBindingResource::GPUTextureView(ref t) => {
                            valid &= t.is_valid();
                            WebGPUBindings::TextureView(t.id().0)
                        },
                        GPUBindingResource::GPUBufferBindings(ref b) => {
                            valid &= b.buffer.is_valid();
                            WebGPUBindings::Buffer(BufferBinding {
                                buffer_id: b.buffer.id().0,
                                offset: b.offset,
                                size: b.size.and_then(wgt::BufferSize::new),
                            })
                        },
                    },
                )
            })
            .collect::<Vec<_>>();

        let bind_group_id = self
            .global()
            .wgpu_id_hub()
            .lock()
            .create_bind_group_id(self.device.0.backend());
        self.channel
            .0
            .send(WebGPURequest::CreateBindGroup {
                device_id: self.device.0,
                bind_group_id,
                bind_group_layout_id: descriptor.layout.id().0,
                entries,
            })
            .expect("Failed to create WebGPU BindGroup");

        let bind_group = webgpu::WebGPUBindGroup(bind_group_id);
        GPUBindGroup::new(
            &self.global(),
            bind_group,
            self.device,
            valid,
            &*descriptor.layout,
        )
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpudevice-createshadermodule
    fn CreateShaderModule(
        &self,
        descriptor: RootedTraceableBox<GPUShaderModuleDescriptor>,
    ) -> DomRoot<GPUShaderModule> {
        let program: Vec<u32> = match &descriptor.code {
            Uint32Array(program) => program.to_vec(),
            String(program) => program.chars().map(|c| c as u32).collect::<Vec<u32>>(),
        };
        let program_id = self
            .global()
            .wgpu_id_hub()
            .lock()
            .create_shader_module_id(self.device.0.backend());
        self.channel
            .0
            .send(WebGPURequest::CreateShaderModule {
                device_id: self.device.0,
                program_id,
                program,
            })
            .expect("Failed to create WebGPU ShaderModule");

        let shader_module = webgpu::WebGPUShaderModule(program_id);
        GPUShaderModule::new(&self.global(), shader_module)
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpudevice-createcomputepipeline
    fn CreateComputePipeline(
        &self,
        descriptor: &GPUComputePipelineDescriptor,
    ) -> DomRoot<GPUComputePipeline> {
        let valid = descriptor.parent.layout.is_valid();
        let pipeline = descriptor.parent.layout.id();
        let program = descriptor.computeStage.module.id();
        let entry_point = descriptor.computeStage.entryPoint.to_string();
        let compute_pipeline_id = self
            .global()
            .wgpu_id_hub()
            .lock()
            .create_compute_pipeline_id(self.device.0.backend());

        self.channel
            .0
            .send(WebGPURequest::CreateComputePipeline {
                device_id: self.device.0,
                compute_pipeline_id,
                pipeline_layout_id: pipeline.0,
                program_id: program.0,
                entry_point,
            })
            .expect("Failed to create WebGPU ComputePipeline");

        let compute_pipeline = webgpu::WebGPUComputePipeline(compute_pipeline_id);
        GPUComputePipeline::new(&self.global(), compute_pipeline, valid)
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpudevice-createcommandencoder
    fn CreateCommandEncoder(
        &self,
        _descriptor: &GPUCommandEncoderDescriptor,
    ) -> DomRoot<GPUCommandEncoder> {
        let command_encoder_id = self
            .global()
            .wgpu_id_hub()
            .lock()
            .create_command_encoder_id(self.device.0.backend());
        self.channel
            .0
            .send(WebGPURequest::CreateCommandEncoder {
                device_id: self.device.0,
                command_encoder_id,
            })
            .expect("Failed to create WebGPU command encoder");

        let encoder = webgpu::WebGPUCommandEncoder(command_encoder_id);

        GPUCommandEncoder::new(
            &self.global(),
            self.channel.clone(),
            self.device,
            encoder,
            true,
        )
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpudevice-createtexture
    fn CreateTexture(&self, descriptor: &GPUTextureDescriptor) -> DomRoot<GPUTexture> {
        let mut valid = true;
        let size = convert_texture_size_to_dict(&descriptor.size);
        let desc = wgt::TextureDescriptor {
            label: Default::default(),
            size: convert_texture_size_to_wgt(&size),
            mip_level_count: descriptor.mipLevelCount,
            sample_count: descriptor.sampleCount,
            dimension: match descriptor.dimension {
                GPUTextureDimension::_1d => wgt::TextureDimension::D1,
                GPUTextureDimension::_2d => wgt::TextureDimension::D2,
                GPUTextureDimension::_3d => wgt::TextureDimension::D3,
            },
            format: convert_texture_format(descriptor.format),
            usage: match wgt::TextureUsage::from_bits(descriptor.usage) {
                Some(t) => t,
                None => {
                    valid = false;
                    wgt::TextureUsage::empty()
                },
            },
        };

        let texture_id = self
            .global()
            .wgpu_id_hub()
            .lock()
            .create_texture_id(self.device.0.backend());

        self.channel
            .0
            .send(WebGPURequest::CreateTexture {
                device_id: self.device.0,
                texture_id,
                descriptor: desc,
            })
            .expect("Failed to create WebGPU Texture");

        let texture = webgpu::WebGPUTexture(texture_id);

        GPUTexture::new(
            &self.global(),
            texture,
            self.device,
            self.channel.clone(),
            size,
            descriptor.mipLevelCount,
            descriptor.sampleCount,
            descriptor.dimension,
            descriptor.format,
            descriptor.usage,
            valid,
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
        let desc = wgt::SamplerDescriptor {
            label: Default::default(),
            address_mode_u: convert_address_mode(descriptor.addressModeU),
            address_mode_v: convert_address_mode(descriptor.addressModeV),
            address_mode_w: convert_address_mode(descriptor.addressModeW),
            mag_filter: convert_filter_mode(descriptor.magFilter),
            min_filter: convert_filter_mode(descriptor.minFilter),
            mipmap_filter: convert_filter_mode(descriptor.mipmapFilter),
            lod_min_clamp: *descriptor.lodMinClamp,
            lod_max_clamp: *descriptor.lodMaxClamp,
            compare: descriptor.compare.map(|c| convert_compare_function(c)),
            anisotropy_clamp: None,
            ..Default::default()
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

        GPUSampler::new(&self.global(), self.device, compare_enable, sampler, true)
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpudevice-createrenderpipeline
    fn CreateRenderPipeline(
        &self,
        descriptor: &GPURenderPipelineDescriptor,
    ) -> DomRoot<GPURenderPipeline> {
        let valid = descriptor.parent.layout.is_valid();
        let vertex_module = descriptor.vertexStage.module.id().0;
        let vertex_entry_point = descriptor.vertexStage.entryPoint.to_string();
        let (fragment_module, fragment_entry_point) = match descriptor.fragmentStage {
            Some(ref frag) => (Some(frag.module.id().0), Some(frag.entryPoint.to_string())),
            None => (None, None),
        };

        let primitive_topology = match descriptor.primitiveTopology {
            GPUPrimitiveTopology::Point_list => wgt::PrimitiveTopology::PointList,
            GPUPrimitiveTopology::Line_list => wgt::PrimitiveTopology::LineList,
            GPUPrimitiveTopology::Line_strip => wgt::PrimitiveTopology::LineStrip,
            GPUPrimitiveTopology::Triangle_list => wgt::PrimitiveTopology::TriangleList,
            GPUPrimitiveTopology::Triangle_strip => wgt::PrimitiveTopology::TriangleStrip,
        };

        let ref rs_desc = descriptor.rasterizationState;
        let rasterization_state = wgt::RasterizationStateDescriptor {
            front_face: match rs_desc.frontFace {
                GPUFrontFace::Ccw => wgt::FrontFace::Ccw,
                GPUFrontFace::Cw => wgt::FrontFace::Cw,
            },
            cull_mode: match rs_desc.cullMode {
                GPUCullMode::None => wgt::CullMode::None,
                GPUCullMode::Front => wgt::CullMode::Front,
                GPUCullMode::Back => wgt::CullMode::Back,
            },
            depth_bias: rs_desc.depthBias,
            depth_bias_slope_scale: *rs_desc.depthBiasSlopeScale,
            depth_bias_clamp: *rs_desc.depthBiasClamp,
        };

        let color_states = descriptor
            .colorStates
            .iter()
            .map(|state| wgt::ColorStateDescriptor {
                format: convert_texture_format(state.format),
                alpha_blend: convert_blend_descriptor(&state.alphaBlend),
                color_blend: convert_blend_descriptor(&state.colorBlend),
                write_mask: match wgt::ColorWrite::from_bits(state.writeMask) {
                    Some(mask) => mask,
                    None => wgt::ColorWrite::empty(),
                },
            })
            .collect::<ArrayVec<_>>();

        let depth_stencil_state = if let Some(ref dss_desc) = descriptor.depthStencilState {
            Some(wgt::DepthStencilStateDescriptor {
                format: convert_texture_format(dss_desc.format),
                depth_write_enabled: dss_desc.depthWriteEnabled,
                depth_compare: convert_compare_function(dss_desc.depthCompare),
                stencil_front: wgt::StencilStateFaceDescriptor {
                    compare: convert_compare_function(dss_desc.stencilFront.compare),
                    fail_op: convert_stencil_op(dss_desc.stencilFront.failOp),
                    depth_fail_op: convert_stencil_op(dss_desc.stencilFront.depthFailOp),
                    pass_op: convert_stencil_op(dss_desc.stencilFront.passOp),
                },
                stencil_back: wgt::StencilStateFaceDescriptor {
                    compare: convert_compare_function(dss_desc.stencilBack.compare),
                    fail_op: convert_stencil_op(dss_desc.stencilBack.failOp),
                    depth_fail_op: convert_stencil_op(dss_desc.stencilBack.depthFailOp),
                    pass_op: convert_stencil_op(dss_desc.stencilBack.passOp),
                },
                stencil_read_mask: dss_desc.stencilReadMask,
                stencil_write_mask: dss_desc.stencilWriteMask,
            })
        } else {
            None
        };

        let ref vs_desc = descriptor.vertexState;
        let vertex_state = (
            match vs_desc.indexFormat {
                GPUIndexFormat::Uint16 => wgt::IndexFormat::Uint16,
                GPUIndexFormat::Uint32 => wgt::IndexFormat::Uint32,
            },
            vs_desc
                .vertexBuffers
                .iter()
                .map(|buffer| {
                    (
                        buffer.arrayStride,
                        match buffer.stepMode {
                            GPUInputStepMode::Vertex => wgt::InputStepMode::Vertex,
                            GPUInputStepMode::Instance => wgt::InputStepMode::Instance,
                        },
                        buffer
                            .attributes
                            .iter()
                            .map(|att| wgt::VertexAttributeDescriptor {
                                format: convert_vertex_format(att.format),
                                offset: att.offset,
                                shader_location: att.shaderLocation,
                            })
                            .collect::<Vec<_>>(),
                    )
                })
                .collect::<Vec<_>>(),
        );

        let render_pipeline_id = self
            .global()
            .wgpu_id_hub()
            .lock()
            .create_render_pipeline_id(self.device.0.backend());

        self.channel
            .0
            .send(WebGPURequest::CreateRenderPipeline {
                device_id: self.device.0,
                render_pipeline_id,
                pipeline_layout_id: descriptor.parent.layout.id().0,
                vertex_module,
                vertex_entry_point,
                fragment_module,
                fragment_entry_point,
                primitive_topology,
                rasterization_state,
                color_states,
                depth_stencil_state,
                vertex_state,
                sample_count: descriptor.sampleCount,
                sample_mask: descriptor.sampleMask,
                alpha_to_coverage_enabled: descriptor.alphaToCoverageEnabled,
            })
            .expect("Failed to create WebGPU render pipeline");

        let render_pipeline = webgpu::WebGPURenderPipeline(render_pipeline_id);

        GPURenderPipeline::new(&self.global(), render_pipeline, self.device, valid)
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

fn convert_blend_descriptor(desc: &GPUBlendDescriptor) -> wgt::BlendDescriptor {
    wgt::BlendDescriptor {
        src_factor: convert_blend_factor(desc.srcFactor),
        dst_factor: convert_blend_factor(desc.dstFactor),
        operation: match desc.operation {
            GPUBlendOperation::Add => wgt::BlendOperation::Add,
            GPUBlendOperation::Subtract => wgt::BlendOperation::Subtract,
            GPUBlendOperation::Reverse_subtract => wgt::BlendOperation::ReverseSubtract,
            GPUBlendOperation::Min => wgt::BlendOperation::Min,
            GPUBlendOperation::Max => wgt::BlendOperation::Max,
        },
    }
}

fn convert_blend_factor(factor: GPUBlendFactor) -> wgt::BlendFactor {
    match factor {
        GPUBlendFactor::Zero => wgt::BlendFactor::Zero,
        GPUBlendFactor::One => wgt::BlendFactor::One,
        GPUBlendFactor::Src_color => wgt::BlendFactor::SrcColor,
        GPUBlendFactor::One_minus_src_color => wgt::BlendFactor::OneMinusSrcColor,
        GPUBlendFactor::Src_alpha => wgt::BlendFactor::SrcAlpha,
        GPUBlendFactor::One_minus_src_alpha => wgt::BlendFactor::OneMinusSrcAlpha,
        GPUBlendFactor::Dst_color => wgt::BlendFactor::DstColor,
        GPUBlendFactor::One_minus_dst_color => wgt::BlendFactor::OneMinusDstColor,
        GPUBlendFactor::Dst_alpha => wgt::BlendFactor::DstAlpha,
        GPUBlendFactor::One_minus_dst_alpha => wgt::BlendFactor::OneMinusDstAlpha,
        GPUBlendFactor::Src_alpha_saturated => wgt::BlendFactor::SrcAlphaSaturated,
        GPUBlendFactor::Blend_color => wgt::BlendFactor::BlendColor,
        GPUBlendFactor::One_minus_blend_color => wgt::BlendFactor::OneMinusBlendColor,
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
        GPUVertexFormat::Uchar2 => wgt::VertexFormat::Uchar2,
        GPUVertexFormat::Uchar4 => wgt::VertexFormat::Uchar4,
        GPUVertexFormat::Char2 => wgt::VertexFormat::Char2,
        GPUVertexFormat::Char4 => wgt::VertexFormat::Char4,
        GPUVertexFormat::Uchar2norm => wgt::VertexFormat::Uchar2Norm,
        GPUVertexFormat::Uchar4norm => wgt::VertexFormat::Uchar4Norm,
        GPUVertexFormat::Char2norm => wgt::VertexFormat::Char2Norm,
        GPUVertexFormat::Char4norm => wgt::VertexFormat::Char4Norm,
        GPUVertexFormat::Ushort2 => wgt::VertexFormat::Ushort2,
        GPUVertexFormat::Ushort4 => wgt::VertexFormat::Ushort4,
        GPUVertexFormat::Short2 => wgt::VertexFormat::Short2,
        GPUVertexFormat::Short4 => wgt::VertexFormat::Short4,
        GPUVertexFormat::Ushort2norm => wgt::VertexFormat::Ushort2Norm,
        GPUVertexFormat::Ushort4norm => wgt::VertexFormat::Ushort4Norm,
        GPUVertexFormat::Short2norm => wgt::VertexFormat::Short2Norm,
        GPUVertexFormat::Short4norm => wgt::VertexFormat::Short4Norm,
        GPUVertexFormat::Half2 => wgt::VertexFormat::Half2,
        GPUVertexFormat::Half4 => wgt::VertexFormat::Half4,
        GPUVertexFormat::Float => wgt::VertexFormat::Float,
        GPUVertexFormat::Float2 => wgt::VertexFormat::Float2,
        GPUVertexFormat::Float3 => wgt::VertexFormat::Float3,
        GPUVertexFormat::Float4 => wgt::VertexFormat::Float4,
        GPUVertexFormat::Uint => wgt::VertexFormat::Uint,
        GPUVertexFormat::Uint2 => wgt::VertexFormat::Uint2,
        GPUVertexFormat::Uint3 => wgt::VertexFormat::Uint3,
        GPUVertexFormat::Uint4 => wgt::VertexFormat::Uint4,
        GPUVertexFormat::Int => wgt::VertexFormat::Int,
        GPUVertexFormat::Int2 => wgt::VertexFormat::Int2,
        GPUVertexFormat::Int3 => wgt::VertexFormat::Int3,
        GPUVertexFormat::Int4 => wgt::VertexFormat::Int4,
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
        GPUTextureFormat::Rg11b10float => wgt::TextureFormat::Rg11b10Float,
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

fn convert_texture_size_to_dict(size: &GPUExtent3D) -> GPUExtent3DDict {
    match *size {
        GPUExtent3D::GPUExtent3DDict(ref dict) => GPUExtent3DDict {
            width: dict.width,
            height: dict.height,
            depth: dict.depth,
        },
        GPUExtent3D::RangeEnforcedUnsignedLongSequence(ref v) => GPUExtent3DDict {
            width: v[0],
            height: v[1],
            depth: v[2],
        },
    }
}

fn convert_texture_size_to_wgt(size: &GPUExtent3DDict) -> wgt::Extent3d {
    wgt::Extent3d {
        width: size.width,
        height: size.height,
        depth: size.depth,
    }
}
