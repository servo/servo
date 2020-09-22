/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::GPUAdapterBinding::{
    GPUAdapterMethods, GPUDeviceDescriptor, GPUExtensionName, GPULimits,
};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::gpu::response_async;
use crate::dom::gpu::AsyncWGPUListener;
use crate::dom::gpudevice::GPUDevice;
use crate::dom::promise::Promise;
use crate::realms::InRealm;
use crate::script_runtime::JSContext as SafeJSContext;
use dom_struct::dom_struct;
use js::jsapi::{Heap, JSObject};
use std::ptr::NonNull;
use std::rc::Rc;
use webgpu::{wgt, WebGPU, WebGPUAdapter, WebGPURequest, WebGPUResponse, WebGPUResponseResult};

#[dom_struct]
pub struct GPUAdapter {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "channels are hard"]
    channel: WebGPU,
    name: DOMString,
    #[ignore_malloc_size_of = "mozjs"]
    extensions: Heap<*mut JSObject>,
    adapter: WebGPUAdapter,
}

impl GPUAdapter {
    fn new_inherited(
        channel: WebGPU,
        name: DOMString,
        extensions: Heap<*mut JSObject>,
        adapter: WebGPUAdapter,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            channel,
            name,
            extensions,
            adapter,
        }
    }

    pub fn new(
        global: &GlobalScope,
        channel: WebGPU,
        name: DOMString,
        extensions: Heap<*mut JSObject>,
        adapter: WebGPUAdapter,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(GPUAdapter::new_inherited(
                channel, name, extensions, adapter,
            )),
            global,
        )
    }
}

impl GPUAdapterMethods for GPUAdapter {
    // https://gpuweb.github.io/gpuweb/#dom-gpuadapter-name
    fn Name(&self) -> DOMString {
        self.name.clone()
    }

    // https://gpuweb.github.io/gpuweb/#dom-gpuadapter-extensions
    fn Extensions(&self, _cx: SafeJSContext) -> NonNull<JSObject> {
        NonNull::new(self.extensions.get()).unwrap()
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuadapter-requestdevice
    fn RequestDevice(&self, descriptor: &GPUDeviceDescriptor, comp: InRealm) -> Rc<Promise> {
        let promise = Promise::new_in_current_realm(&self.global(), comp);
        let sender = response_async(&promise, self);
        let mut features = wgt::Features::empty();
        for &ext in descriptor.extensions.iter() {
            if ext == GPUExtensionName::Depth_clamping {
                features.insert(wgt::Features::DEPTH_CLAMPING);
            } else if ext == GPUExtensionName::Texture_compression_bc {
                features.insert(wgt::Features::TEXTURE_COMPRESSION_BC)
            }
        }

        let desc = wgt::DeviceDescriptor {
            features,
            limits: wgt::Limits {
                max_bind_groups: descriptor.limits.maxBindGroups,
                max_dynamic_uniform_buffers_per_pipeline_layout: descriptor
                    .limits
                    .maxDynamicUniformBuffersPerPipelineLayout,
                max_dynamic_storage_buffers_per_pipeline_layout: descriptor
                    .limits
                    .maxDynamicStorageBuffersPerPipelineLayout,
                max_sampled_textures_per_shader_stage: descriptor
                    .limits
                    .maxSampledTexturesPerShaderStage,
                max_samplers_per_shader_stage: descriptor.limits.maxSamplersPerShaderStage,
                max_storage_buffers_per_shader_stage: descriptor
                    .limits
                    .maxStorageBuffersPerShaderStage,
                max_storage_textures_per_shader_stage: descriptor
                    .limits
                    .maxStorageTexturesPerShaderStage,
                max_uniform_buffers_per_shader_stage: descriptor
                    .limits
                    .maxUniformBuffersPerShaderStage,
                max_uniform_buffer_binding_size: descriptor.limits.maxUniformBufferBindingSize,
                ..Default::default()
            },
            shader_validation: true,
        };
        let id = self
            .global()
            .wgpu_id_hub()
            .lock()
            .create_device_id(self.adapter.0.backend());
        let pipeline_id = self.global().pipeline_id();
        if self
            .channel
            .0
            .send((
                None,
                WebGPURequest::RequestDevice {
                    sender,
                    adapter_id: self.adapter,
                    descriptor: desc,
                    device_id: id,
                    pipeline_id,
                    label: descriptor.parent.label.as_ref().map(|s| s.to_string()),
                },
            ))
            .is_err()
        {
            promise.reject_error(Error::Operation);
        }
        promise
    }
}

impl AsyncWGPUListener for GPUAdapter {
    fn handle_response(&self, response: WebGPUResponseResult, promise: &Rc<Promise>) {
        match response {
            Ok(WebGPUResponse::RequestDevice {
                device_id,
                queue_id,
                descriptor,
                label,
            }) => {
                let limits = GPULimits {
                    maxBindGroups: descriptor.limits.max_bind_groups,
                    maxDynamicStorageBuffersPerPipelineLayout: descriptor
                        .limits
                        .max_dynamic_storage_buffers_per_pipeline_layout,
                    maxDynamicUniformBuffersPerPipelineLayout: descriptor
                        .limits
                        .max_dynamic_uniform_buffers_per_pipeline_layout,
                    maxSampledTexturesPerShaderStage: descriptor
                        .limits
                        .max_sampled_textures_per_shader_stage,
                    maxSamplersPerShaderStage: descriptor.limits.max_samplers_per_shader_stage,
                    maxStorageBuffersPerShaderStage: descriptor
                        .limits
                        .max_storage_buffers_per_shader_stage,
                    maxStorageTexturesPerShaderStage: descriptor
                        .limits
                        .max_storage_textures_per_shader_stage,
                    maxUniformBufferBindingSize: descriptor.limits.max_uniform_buffer_binding_size,
                    maxUniformBuffersPerShaderStage: descriptor
                        .limits
                        .max_uniform_buffers_per_shader_stage,
                };
                let device = GPUDevice::new(
                    &self.global(),
                    self.channel.clone(),
                    &self,
                    Heap::default(),
                    limits,
                    device_id,
                    queue_id,
                    label,
                );
                self.global().add_gpu_device(&device);
                promise.resolve_native(&device);
            },
            Err(e) => {
                warn!("Could not get GPUDevice({:?})", e);
                promise.reject_error(Error::Operation);
            },
            _ => {
                warn!("GPUAdapter received wrong WebGPUResponse");
                promise.reject_error(Error::Operation);
            },
        }
    }
}
