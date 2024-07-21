/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::convert::TryFrom;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::jsapi::{Heap, JSObject};
use webgpu::{wgt, WebGPU, WebGPUAdapter, WebGPURequest, WebGPUResponse};

use super::gpusupportedfeatures::GPUSupportedFeatures;
use super::types::{GPUAdapterInfo, GPUSupportedLimits};
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::{
    GPUAdapterMethods, GPUDeviceDescriptor,
};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::gpu::{response_async, AsyncWGPUListener};
use crate::dom::gpudevice::GPUDevice;
use crate::dom::gpusupportedfeatures::gpu_to_wgt_feature;
use crate::dom::promise::Promise;
use crate::realms::InRealm;

#[dom_struct]
pub struct GPUAdapter {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "channels are hard"]
    #[no_trace]
    channel: WebGPU,
    name: DOMString,
    #[ignore_malloc_size_of = "mozjs"]
    extensions: Heap<*mut JSObject>,
    features: Dom<GPUSupportedFeatures>,
    limits: Dom<GPUSupportedLimits>,
    info: Dom<GPUAdapterInfo>,
    #[no_trace]
    adapter: WebGPUAdapter,
}

impl GPUAdapter {
    fn new_inherited(
        channel: WebGPU,
        name: DOMString,
        extensions: Heap<*mut JSObject>,
        features: &GPUSupportedFeatures,
        limits: &GPUSupportedLimits,
        info: &GPUAdapterInfo,
        adapter: WebGPUAdapter,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            channel,
            name,
            extensions,
            features: Dom::from_ref(features),
            limits: Dom::from_ref(limits),
            info: Dom::from_ref(info),
            adapter,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        global: &GlobalScope,
        channel: WebGPU,
        name: DOMString,
        extensions: Heap<*mut JSObject>,
        features: wgt::Features,
        limits: wgt::Limits,
        info: wgt::AdapterInfo,
        adapter: WebGPUAdapter,
    ) -> DomRoot<Self> {
        let features = GPUSupportedFeatures::Constructor(global, None, features).unwrap();
        let limits = GPUSupportedLimits::new(global, limits);
        let info = GPUAdapterInfo::new(global, info);
        reflect_dom_object(
            Box::new(GPUAdapter::new_inherited(
                channel, name, extensions, &features, &limits, &info, adapter,
            )),
            global,
        )
    }
}

impl Drop for GPUAdapter {
    fn drop(&mut self) {
        if let Err(e) = self
            .channel
            .0
            .send(WebGPURequest::DropAdapter(self.adapter.0))
        {
            warn!(
                "Failed to send WebGPURequest::DropAdapter({:?}) ({})",
                self.adapter.0, e
            );
        };
    }
}

impl GPUAdapterMethods for GPUAdapter {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpuadapter-requestdevice>
    fn RequestDevice(&self, descriptor: &GPUDeviceDescriptor, comp: InRealm) -> Rc<Promise> {
        // Step 2
        let promise = Promise::new_in_current_realm(comp);
        let sender = response_async(&promise, self);
        let mut features = wgt::Features::empty();
        for &ext in descriptor.requiredFeatures.iter() {
            if let Some(feature) = gpu_to_wgt_feature(ext) {
                features.insert(feature);
            } else {
                promise.reject_error(Error::Type(format!(
                    "{} is not supported feature",
                    ext.as_str()
                )));
                return promise;
            }
        }

        let mut desc = wgt::DeviceDescriptor {
            required_features: features,
            required_limits: wgt::Limits::default(),
            label: None,
        };
        if let Some(limits) = &descriptor.requiredLimits {
            for (limit, value) in (*limits).iter() {
                let v = u32::try_from(*value).unwrap_or(u32::MAX);
                match limit.as_ref() {
                    "maxTextureDimension1D" => desc.required_limits.max_texture_dimension_1d = v,
                    "maxTextureDimension2D" => desc.required_limits.max_texture_dimension_2d = v,
                    "maxTextureDimension3D" => desc.required_limits.max_texture_dimension_3d = v,
                    "maxTextureArrayLayers" => desc.required_limits.max_texture_array_layers = v,
                    "maxBindGroups" => desc.required_limits.max_bind_groups = v,
                    "maxBindingsPerBindGroup" => {
                        desc.required_limits.max_bindings_per_bind_group = v
                    },
                    "maxDynamicUniformBuffersPerPipelineLayout" => {
                        desc.required_limits
                            .max_dynamic_uniform_buffers_per_pipeline_layout = v
                    },
                    "maxDynamicStorageBuffersPerPipelineLayout" => {
                        desc.required_limits
                            .max_dynamic_storage_buffers_per_pipeline_layout = v
                    },
                    "maxSampledTexturesPerShaderStage" => {
                        desc.required_limits.max_sampled_textures_per_shader_stage = v
                    },
                    "maxSamplersPerShaderStage" => {
                        desc.required_limits.max_samplers_per_shader_stage = v
                    },
                    "maxStorageBuffersPerShaderStage" => {
                        desc.required_limits.max_storage_buffers_per_shader_stage = v
                    },
                    "maxStorageTexturesPerShaderStage" => {
                        desc.required_limits.max_storage_textures_per_shader_stage = v
                    },
                    "maxUniformBuffersPerShaderStage" => {
                        desc.required_limits.max_uniform_buffers_per_shader_stage = v
                    },
                    "maxUniformBufferBindingSize" => {
                        desc.required_limits.max_uniform_buffer_binding_size = v
                    },
                    "maxStorageBufferBindingSize" => {
                        desc.required_limits.max_storage_buffer_binding_size = v
                    },
                    "minUniformBufferOffsetAlignment" => {
                        desc.required_limits.min_uniform_buffer_offset_alignment = v
                    },
                    "minStorageBufferOffsetAlignment" => {
                        desc.required_limits.min_storage_buffer_offset_alignment = v
                    },
                    "maxVertexBuffers" => desc.required_limits.max_vertex_buffers = v,
                    "maxBufferSize" => desc.required_limits.max_buffer_size = *value,
                    "maxVertexAttributes" => desc.required_limits.max_vertex_attributes = v,
                    "maxVertexBufferArrayStride" => {
                        desc.required_limits.max_vertex_buffer_array_stride = v
                    },
                    "maxInterStageShaderComponents" => {
                        desc.required_limits.max_inter_stage_shader_components = v
                    },
                    "maxComputeWorkgroupStorageSize" => {
                        desc.required_limits.max_compute_workgroup_storage_size = v
                    },
                    "maxComputeInvocationsPerWorkgroup" => {
                        desc.required_limits.max_compute_invocations_per_workgroup = v
                    },
                    "maxComputeWorkgroupSizeX" => {
                        desc.required_limits.max_compute_workgroup_size_x = v
                    },
                    "maxComputeWorkgroupSizeY" => {
                        desc.required_limits.max_compute_workgroup_size_y = v
                    },
                    "maxComputeWorkgroupSizeZ" => {
                        desc.required_limits.max_compute_workgroup_size_z = v
                    },
                    "maxComputeWorkgroupsPerDimension" => {
                        desc.required_limits.max_compute_workgroups_per_dimension = v
                    },
                    _ => {
                        error!("Unknown required limit: {limit} with value {value}");
                        // we should reject but spec is still evolving
                        // promise.reject_error(Error::Operation);
                        // return promise;
                    },
                }
            }
        }
        let id = self
            .global()
            .wgpu_id_hub()
            .create_device_id(self.adapter.0.backend());
        let pipeline_id = self.global().pipeline_id();
        if self
            .channel
            .0
            .send(WebGPURequest::RequestDevice {
                sender,
                adapter_id: self.adapter,
                descriptor: desc,
                device_id: id,
                pipeline_id,
            })
            .is_err()
        {
            promise.reject_error(Error::Operation);
        }
        // Step 5
        promise
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuadapter-isfallbackadapter>
    fn IsFallbackAdapter(&self) -> bool {
        //TODO
        false
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuadapter-requestadapterinfo>
    fn RequestAdapterInfo(&self, unmask_hints: Vec<DOMString>, comp: InRealm) -> Rc<Promise> {
        // XXX: Adapter info should be generated here ...
        // Step 1
        let promise = Promise::new_in_current_realm(comp);
        // Step 4
        if !unmask_hints.is_empty() {
            todo!("unmaskHints on RequestAdapterInfo");
        }
        promise.resolve_native(&*self.info);
        // Step 5
        promise
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuadapter-features>
    fn Features(&self) -> DomRoot<GPUSupportedFeatures> {
        DomRoot::from_ref(&self.features)
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuadapter-limits>
    fn Limits(&self) -> DomRoot<GPUSupportedLimits> {
        DomRoot::from_ref(&self.limits)
    }
}

impl AsyncWGPUListener for GPUAdapter {
    fn handle_response(&self, response: WebGPUResponse, promise: &Rc<Promise>) {
        match response {
            WebGPUResponse::Device(Ok(device)) => {
                let descriptor = device.descriptor;
                let device = GPUDevice::new(
                    &self.global(),
                    self.channel.clone(),
                    self,
                    Heap::default(),
                    descriptor.required_features,
                    descriptor.required_limits,
                    device.device_id,
                    device.queue_id,
                    descriptor.label.unwrap_or_default(),
                );
                self.global().add_gpu_device(&device);
                promise.resolve_native(&device);
            },
            WebGPUResponse::Device(Err(e)) => {
                warn!("Could not get GPUDevice({:?})", e);
                promise.reject_error(Error::Operation);
            },
            WebGPUResponse::None => unreachable!("Failed to get a response for RequestDevice"),
            _ => unreachable!("GPUAdapter received wrong WebGPUResponse"),
        }
    }
}
