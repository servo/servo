/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::GPUAdapterBinding::GPULimits;
use crate::dom::bindings::codegen::Bindings::GPUBindGroupLayoutBinding::{
    GPUBindGroupLayoutBindings, GPUBindGroupLayoutDescriptor, GPUBindingType,
};
use crate::dom::bindings::codegen::Bindings::GPUBufferBinding::GPUBufferDescriptor;
use crate::dom::bindings::codegen::Bindings::GPUDeviceBinding::{self, GPUDeviceMethods};
use crate::dom::bindings::codegen::Bindings::GPUPipelineLayoutBinding::GPUPipelineLayoutDescriptor;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowBinding::WindowMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::gpuadapter::GPUAdapter;
use crate::dom::gpubindgrouplayout::GPUBindGroupLayout;
use crate::dom::gpubuffer::{GPUBuffer, GPUBufferState};
use crate::dom::gpupipelinelayout::GPUPipelineLayout;
use crate::dom::window::Window;
use crate::script_runtime::JSContext as SafeJSContext;
use dom_struct::dom_struct;
use ipc_channel::ipc;
use js::jsapi::{Heap, JSObject};
use js::jsval::{JSVal, ObjectValue, UndefinedValue};
use js::typedarray::{ArrayBuffer, CreateWith};
use std::collections::{HashMap, HashSet};
use std::ptr::{self, NonNull};
use webgpu::wgpu::binding_model::{BindGroupLayoutBinding, BindingType, ShaderStage};
use webgpu::wgpu::resource::{BufferDescriptor, BufferUsage};
use webgpu::{WebGPU, WebGPUBuffer, WebGPUDevice, WebGPURequest};

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
    device: WebGPUDevice,
}

impl GPUDevice {
    fn new_inherited(
        channel: WebGPU,
        adapter: &GPUAdapter,
        extensions: Heap<*mut JSObject>,
        limits: Heap<*mut JSObject>,
        device: WebGPUDevice,
    ) -> GPUDevice {
        Self {
            eventtarget: EventTarget::new_inherited(),
            channel,
            adapter: Dom::from_ref(adapter),
            extensions,
            limits,
            label: DomRefCell::new(None),
            device,
        }
    }

    #[allow(unsafe_code)]
    pub fn new(
        global: &GlobalScope,
        channel: WebGPU,
        adapter: &GPUAdapter,
        extensions: Heap<*mut JSObject>,
        limits: Heap<*mut JSObject>,
        device: WebGPUDevice,
    ) -> DomRoot<GPUDevice> {
        reflect_dom_object(
            Box::new(GPUDevice::new_inherited(
                channel, adapter, extensions, limits, device,
            )),
            global,
            GPUDeviceBinding::Wrap,
        )
    }
}

impl GPUDevice {
    unsafe fn resolve_create_buffer_mapped(
        &self,
        cx: SafeJSContext,
        gpu_buffer: WebGPUBuffer,
        array_buffer: Vec<u8>,
        descriptor: BufferDescriptor,
        valid: bool,
    ) -> Vec<JSVal> {
        rooted!(in(*cx) let mut js_array_buffer = ptr::null_mut::<JSObject>());
        let mut out = Vec::new();
        assert!(ArrayBuffer::create(
            *cx,
            CreateWith::Slice(array_buffer.as_slice()),
            js_array_buffer.handle_mut(),
        )
        .is_ok());

        let buff = GPUBuffer::new(
            &self.global(),
            self.channel.clone(),
            gpu_buffer,
            self.device,
            GPUBufferState::Mapped,
            descriptor.size,
            descriptor.usage.bits(),
            valid,
        );
        out.push(ObjectValue(buff.reflector().get_jsobject().get()));
        out.push(ObjectValue(js_array_buffer.get()));
        out
    }

    fn validate_buffer_descriptor(
        &self,
        descriptor: &GPUBufferDescriptor,
    ) -> (bool, BufferDescriptor) {
        // TODO: Record a validation error in the current scope if the descriptor is invalid.
        let wgpu_usage = BufferUsage::from_bits(descriptor.usage);
        let valid = wgpu_usage.is_some() && descriptor.size > 0;

        if valid {
            (
                true,
                BufferDescriptor {
                    size: descriptor.size,
                    usage: wgpu_usage.unwrap(),
                },
            )
        } else {
            (
                false,
                BufferDescriptor {
                    size: 0,
                    usage: BufferUsage::STORAGE,
                },
            )
        }
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
        let (valid, wgpu_descriptor) = self.validate_buffer_descriptor(descriptor);
        let (sender, receiver) = ipc::channel().unwrap();
        if let Some(window) = self.global().downcast::<Window>() {
            let id = window.Navigator().create_buffer_id(self.device.0.backend());
            self.channel
                .0
                .send(WebGPURequest::CreateBuffer(
                    sender,
                    self.device,
                    id,
                    wgpu_descriptor,
                ))
                .expect("Failed to create WebGPU buffer");
        } else {
            unimplemented!()
        };

        let buffer = receiver.recv().unwrap();

        GPUBuffer::new(
            &self.global(),
            self.channel.clone(),
            buffer,
            self.device,
            GPUBufferState::Unmapped,
            descriptor.size,
            descriptor.usage,
            valid,
        )
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpudevice-createbuffermapped
    fn CreateBufferMapped(
        &self,
        cx: SafeJSContext,
        descriptor: &GPUBufferDescriptor,
    ) -> Vec<JSVal> {
        let (valid, wgpu_descriptor) = self.validate_buffer_descriptor(descriptor);
        let (sender, receiver) = ipc::channel().unwrap();
        rooted!(in(*cx) let js_val = UndefinedValue());
        if let Some(window) = self.global().downcast::<Window>() {
            let id = window.Navigator().create_buffer_id(self.device.0.backend());
            self.channel
                .0
                .send(WebGPURequest::CreateBufferMapped(
                    sender,
                    self.device,
                    id,
                    wgpu_descriptor.clone(),
                ))
                .expect("Failed to create WebGPU buffer");
        } else {
            return vec![js_val.get()];
        };

        let (buffer, array_buffer) = receiver.recv().unwrap();

        unsafe {
            self.resolve_create_buffer_mapped(cx, buffer, array_buffer, wgpu_descriptor, valid)
        }
    }

    /// https://gpuweb.github.io/gpuweb/#GPUDevice-createBindGroupLayout
    #[allow(non_snake_case)]
    fn CreateBindGroupLayout(
        &self,
        descriptor: &GPUBindGroupLayoutDescriptor,
    ) -> DomRoot<GPUBindGroupLayout> {
        #[derive(Clone)]
        struct MaxLimits {
            max_uniform_buffers_per_shader_stage: i32,
            max_storage_buffers_per_shader_stage: i32,
            max_sampled_textures_per_shader_stage: i32,
            max_storage_textures_per_shader_stage: i32,
            max_samplers_per_shader_stage: i32,
        }
        let mut storeBindings = HashSet::new();
        // TODO: We should have these limits on device creation
        let limits = GPULimits::empty();

        let mut validation_map = HashMap::new();
        let maxLimits = MaxLimits {
            max_uniform_buffers_per_shader_stage: limits.maxUniformBuffersPerShaderStage as i32,
            max_storage_buffers_per_shader_stage: limits.maxStorageBuffersPerShaderStage as i32,
            max_sampled_textures_per_shader_stage: limits.maxSampledTexturesPerShaderStage as i32,
            max_storage_textures_per_shader_stage: limits.maxStorageTexturesPerShaderStage as i32,
            max_samplers_per_shader_stage: limits.maxSamplersPerShaderStage as i32,
        };
        validation_map.insert(
            webgpu::wgpu::binding_model::ShaderStage::VERTEX,
            maxLimits.clone(),
        );
        validation_map.insert(
            webgpu::wgpu::binding_model::ShaderStage::FRAGMENT,
            maxLimits.clone(),
        );
        validation_map.insert(
            webgpu::wgpu::binding_model::ShaderStage::COMPUTE,
            maxLimits.clone(),
        );
        let mut max_dynamic_uniform_buffers_per_pipeline_layout =
            limits.maxDynamicUniformBuffersPerPipelineLayout as i32;
        let mut max_dynamic_storage_buffers_per_pipeline_layout =
            limits.maxDynamicStorageBuffersPerPipelineLayout as i32;
        let mut valid = true;

        let bindings = descriptor
            .bindings
            .iter()
            .map(|bind| {
                // TODO: binding must be >= 0
                storeBindings.insert(bind.binding);
                let visibility = match ShaderStage::from_bits(bind.visibility) {
                    Some(visibility) => visibility,
                    None => {
                        valid = false;
                        ShaderStage::from_bits(0).unwrap()
                    },
                };
                let ty = match bind.type_ {
                    GPUBindingType::Uniform_buffer => {
                        if let Some(limit) = validation_map.get_mut(&visibility) {
                            limit.max_uniform_buffers_per_shader_stage -= 1;
                        }
                        if bind.hasDynamicOffset {
                            max_dynamic_uniform_buffers_per_pipeline_layout -= 1;
                        };
                        BindingType::UniformBuffer
                    },
                    GPUBindingType::Storage_buffer => {
                        if let Some(limit) = validation_map.get_mut(&visibility) {
                            limit.max_storage_buffers_per_shader_stage -= 1;
                        }
                        if bind.hasDynamicOffset {
                            max_dynamic_storage_buffers_per_pipeline_layout -= 1;
                        };
                        BindingType::StorageBuffer
                    },
                    GPUBindingType::Readonly_storage_buffer => {
                        if let Some(limit) = validation_map.get_mut(&visibility) {
                            limit.max_storage_buffers_per_shader_stage -= 1;
                        }
                        if bind.hasDynamicOffset {
                            max_dynamic_storage_buffers_per_pipeline_layout -= 1;
                        };
                        BindingType::ReadonlyStorageBuffer
                    },
                    GPUBindingType::Sampled_texture => {
                        if let Some(limit) = validation_map.get_mut(&visibility) {
                            limit.max_sampled_textures_per_shader_stage -= 1;
                        }
                        if bind.hasDynamicOffset {
                            valid = false
                        };
                        BindingType::SampledTexture
                    },
                    GPUBindingType::Storage_texture => {
                        if let Some(limit) = validation_map.get_mut(&visibility) {
                            limit.max_storage_textures_per_shader_stage -= 1;
                        }
                        if bind.hasDynamicOffset {
                            valid = false
                        };
                        BindingType::StorageTexture
                    },
                    GPUBindingType::Sampler => {
                        if let Some(limit) = validation_map.get_mut(&visibility) {
                            limit.max_samplers_per_shader_stage -= 1;
                        }
                        if bind.hasDynamicOffset {
                            valid = false
                        };
                        BindingType::Sampler
                    },
                };

                BindGroupLayoutBinding {
                    binding: bind.binding,
                    visibility,
                    ty,
                    dynamic: bind.hasDynamicOffset,
                    multisampled: bind.multisampled,
                    texture_dimension: webgpu::wgpu::resource::TextureViewDimension::D2, // Use as default for now
                }
            })
            .collect::<Vec<BindGroupLayoutBinding>>();

        // bindings are unique
        valid &= storeBindings.len() == bindings.len();

        // Ensure that values do not exceed the max limit for each ShaderStage.
        valid &= validation_map.values().all(|stage| {
            stage.max_uniform_buffers_per_shader_stage >= 0 &&
                stage.max_storage_buffers_per_shader_stage >= 0 &&
                stage.max_sampled_textures_per_shader_stage >= 0 &&
                stage.max_storage_textures_per_shader_stage >= 0 &&
                stage.max_samplers_per_shader_stage >= 0
        });

        // DynamicValues does not exceed the max limit for the pipeline
        valid &= max_dynamic_uniform_buffers_per_pipeline_layout >= 0 &&
            max_dynamic_storage_buffers_per_pipeline_layout >= 0;

        let (sender, receiver) = ipc::channel().unwrap();
        if let Some(window) = self.global().downcast::<Window>() {
            let id = window
                .Navigator()
                .create_bind_group_layout_id(self.device.0.backend());
            self.channel
                .0
                .send(WebGPURequest::CreateBindGroupLayout(
                    sender,
                    self.device,
                    id,
                    bindings.clone(),
                ))
                .expect("Failed to create WebGPU BindGroupLayout");
        }
        let bgl = receiver.recv().unwrap();

        let binds = descriptor
            .bindings
            .iter()
            .map(|bind| GPUBindGroupLayoutBindings {
                binding: bind.binding,
                hasDynamicOffset: bind.hasDynamicOffset,
                multisampled: bind.multisampled,
                type_: bind.type_,
                visibility: bind.visibility,
                //texture_dimension: bind.texture_dimension
            })
            .collect::<Vec<_>>();

        GPUBindGroupLayout::new(&self.global(), self.channel.clone(), bgl, binds, valid)
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpudevice-createpipelinelayout
    fn CreatePipelineLayout(
        &self,
        descriptor: &GPUPipelineLayoutDescriptor,
    ) -> DomRoot<GPUPipelineLayout> {
        // TODO: We should have these limits on device creation
        let limits = GPULimits::empty();
        let mut bind_group_layouts = Vec::new();
        let mut bgl_ids = Vec::new();
        let mut max_dynamic_uniform_buffers_per_pipeline_layout =
            limits.maxDynamicUniformBuffersPerPipelineLayout as i32;
        let mut max_dynamic_storage_buffers_per_pipeline_layout =
            limits.maxDynamicStorageBuffersPerPipelineLayout as i32;
        descriptor.bindGroupLayouts.iter().for_each(|each| {
            if each.is_valid() {
                let id = each.id();
                bind_group_layouts.push(id);
                bgl_ids.push(id.0);
            }
            each.bindings().iter().for_each(|bind| {
                match bind.type_ {
                    GPUBindingType::Uniform_buffer => {
                        if bind.hasDynamicOffset {
                            max_dynamic_uniform_buffers_per_pipeline_layout -= 1;
                        };
                    },
                    GPUBindingType::Storage_buffer => {
                        if bind.hasDynamicOffset {
                            max_dynamic_storage_buffers_per_pipeline_layout -= 1;
                        };
                    },
                    GPUBindingType::Readonly_storage_buffer => {
                        if bind.hasDynamicOffset {
                            max_dynamic_storage_buffers_per_pipeline_layout -= 1;
                        };
                    },
                    _ => {},
                };
            });
        });

        let valid = descriptor.bindGroupLayouts.len() <= limits.maxBindGroups as usize &&
            descriptor.bindGroupLayouts.len() == bind_group_layouts.len() &&
            max_dynamic_uniform_buffers_per_pipeline_layout >= 0 &&
            max_dynamic_storage_buffers_per_pipeline_layout >= 0;

        let (sender, receiver) = ipc::channel().unwrap();
        if let Some(window) = self.global().downcast::<Window>() {
            let id = window
                .Navigator()
                .create_pipeline_layout_id(self.device.0.backend());
            self.channel
                .0
                .send(WebGPURequest::CreatePipelineLayout(
                    sender,
                    self.device,
                    id,
                    bgl_ids,
                ))
                .expect("Failed to create WebGPU PipelineLayout");
        }
        let pipeline_layout = receiver.recv().unwrap();
        GPUPipelineLayout::new(&self.global(), bind_group_layouts, pipeline_layout, valid)
    }
}
