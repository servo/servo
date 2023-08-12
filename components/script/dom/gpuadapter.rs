/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::GPUAdapterBinding::{
    GPUAdapterMethods, GPUDeviceDescriptor, GPUFeatureName,
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
use std::convert::TryFrom;
use std::ptr::NonNull;
use std::rc::Rc;
use webgpu::{wgt, WebGPU, WebGPUAdapter, WebGPURequest, WebGPUResponse, WebGPUResponseResult};

#[dom_struct]
pub struct GPUAdapter {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "channels are hard"]
    #[no_trace]
    channel: WebGPU,
    name: DOMString,
    #[ignore_malloc_size_of = "mozjs"]
    extensions: Heap<*mut JSObject>,
    #[no_trace]
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
    /// https://gpuweb.github.io/gpuweb/#dom-gpuadapter-requestdevice
    fn RequestDevice(&self, descriptor: &GPUDeviceDescriptor, comp: InRealm) -> Rc<Promise> {
        let promise = Promise::new_in_current_realm(comp);
        let sender = response_async(&promise, self);
        let mut features = wgt::Features::empty();
        for &ext in descriptor.requiredFeatures.iter() {
            match ext {
                GPUFeatureName::Depth_clip_control => {
                    features.insert(wgt::Features::DEPTH_CLIP_CONTROL)
                },
                GPUFeatureName::Depth24unorm_stencil8 => { /* not implemented in wgpu */ },
                GPUFeatureName::Depth32float_stencil8 => {
                    features.insert(wgt::Features::DEPTH32FLOAT_STENCIL8)
                },
                GPUFeatureName::Pipeline_statistics_query => {
                    features.insert(wgt::Features::PIPELINE_STATISTICS_QUERY)
                },
                GPUFeatureName::Texture_compression_bc => {
                    features.insert(wgt::Features::TEXTURE_COMPRESSION_BC)
                },
                GPUFeatureName::Texture_compression_etc2 => {
                    features.insert(wgt::Features::TEXTURE_COMPRESSION_ETC2)
                },
                GPUFeatureName::Texture_compression_astc => {
                    features.insert(wgt::Features::TEXTURE_COMPRESSION_ASTC)
                },
                GPUFeatureName::Timestamp_query => features.insert(wgt::Features::TIMESTAMP_QUERY),
                GPUFeatureName::Indirect_first_instance => {
                    features.insert(wgt::Features::INDIRECT_FIRST_INSTANCE)
                },
            }
        }

        let mut desc = wgt::DeviceDescriptor {
            features,
            limits: wgt::Limits::default(),
            label: None,
        };
        if let Some(lim) = &descriptor.requiredLimits {
            for (k, v) in (*lim).iter() {
                let v = u32::try_from(*v).unwrap_or(u32::MAX);
                match k.as_ref() {
                    "maxTextureDimension1D" => desc.limits.max_texture_dimension_1d = v,
                    "maxTextureDimension2D" => desc.limits.max_texture_dimension_2d = v,
                    "maxTextureDimension3D" => desc.limits.max_texture_dimension_3d = v,
                    "maxTextureArrayLayers" => desc.limits.max_texture_array_layers = v,
                    "maxBindGroups" => desc.limits.max_bind_groups = v,
                    "maxDynamicUniformBuffersPerPipelineLayout" => {
                        desc.limits.max_dynamic_uniform_buffers_per_pipeline_layout = v
                    },
                    "maxDynamicStorageBuffersPerPipelineLayout" => {
                        desc.limits.max_dynamic_storage_buffers_per_pipeline_layout = v
                    },
                    "maxSampledTexturesPerShaderStage" => {
                        desc.limits.max_sampled_textures_per_shader_stage = v
                    },
                    "maxSamplersPerShaderStage" => desc.limits.max_samplers_per_shader_stage = v,
                    "maxStorageBuffersPerShaderStage" => {
                        desc.limits.max_storage_buffers_per_shader_stage = v
                    },
                    "maxStorageTexturesPerShaderStage" => {
                        desc.limits.max_storage_textures_per_shader_stage = v
                    },
                    "maxUniformBuffersPerShaderStage" => {
                        desc.limits.max_uniform_buffers_per_shader_stage = v
                    },
                    "maxUniformBufferBindingSize" => {
                        desc.limits.max_uniform_buffer_binding_size = v
                    },
                    "maxStorageBufferBindingSize" => {
                        desc.limits.max_storage_buffer_binding_size = v
                    },
                    "minUniformBufferOffsetAlignment" => {
                        desc.limits.min_uniform_buffer_offset_alignment = v
                    },
                    "minStorageBufferOffsetAlignment" => {
                        desc.limits.min_storage_buffer_offset_alignment = v
                    },
                    "maxVertexBuffers" => desc.limits.max_vertex_buffers = v,
                    "maxVertexAttributes" => desc.limits.max_vertex_attributes = v,
                    "maxVertexBufferArrayStride" => desc.limits.max_vertex_buffer_array_stride = v,
                    "maxInterStageShaderComponents" => {
                        desc.limits.max_inter_stage_shader_components = v
                    },
                    "maxComputeWorkgroupStorageSize" => {
                        desc.limits.max_compute_workgroup_storage_size = v
                    },
                    "maxComputeInvocationsPerWorkgroup" => {
                        desc.limits.max_compute_invocations_per_workgroup = v
                    },
                    "maxComputeWorkgroupSizeX" => desc.limits.max_compute_workgroup_size_x = v,
                    "maxComputeWorkgroupSizeY" => desc.limits.max_compute_workgroup_size_y = v,
                    "maxComputeWorkgroupSizeZ" => desc.limits.max_compute_workgroup_size_z = v,
                    "maxComputeWorkgroupsPerDimension" => {
                        desc.limits.max_compute_workgroups_per_dimension = v
                    },
                    _ => { /* no-op */ },
                }
            }
        }
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
                    label: desc.label.clone(), // TODO(sagudev)
                    descriptor: desc,
                    device_id: id,
                    pipeline_id,
                },
            ))
            .is_err()
        {
            promise.reject_error(Error::Operation);
        }
        promise
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuadapter-isfallbackadapter
    fn IsFallbackAdapter(&self) -> bool {
        //TODO
        false
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuadapter-requestadapterinfo
    fn RequestAdapterInfo(&self, unmaskHints: Vec<DOMString>, comp: InRealm) -> Rc<Promise> {
        let promise = Promise::new_in_current_realm(comp);
        let sender = response_async(&promise, self);
        todo!("Implemnt WPU adapter info");
        /*if self
            .channel
            .0
            .send((
                None,
                WebGPURequest::RequestAdapterInfo {
                    sender,
                    adapter_id: self.adapter,
                },
            ))
            .is_err()
        {
            promise.reject_error(Error::Operation);
        }*/
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
                let device = GPUDevice::new(
                    &self.global(),
                    self.channel.clone(),
                    &self,
                    Heap::default(),
                    wgt::Limits::default(), // TODO use real limits
                    device_id,
                    queue_id,
                    label.unwrap_or_default(),
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
