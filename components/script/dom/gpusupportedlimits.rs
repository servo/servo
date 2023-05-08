/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::reflector::Reflector;
use crate::dom::bindings::trace::JSTraceable;
use dom_struct::dom_struct;
use js::jsapi::JSTracer;
use GPUSupportedLimitsBinding::GPUSupportedLimitsMethods;

use super::bindings::codegen::Bindings::GPUSupportedLimitsBinding::GPUSupportedLimitsBinding;
use super::bindings::reflector::reflect_dom_object;
use super::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;

use webgpu::{
    identity::WebGPUOpResult, wgpu::resource, wgt, WebGPU, WebGPURequest, WebGPUTexture,
    WebGPUTextureView,
};

pub struct Limits(wgt::Limits);

#[allow(unsafe_code)]
unsafe impl JSTraceable for Limits {
    unsafe fn trace(&self, _: *mut JSTracer) {
        // do nothing
    }
}

#[dom_struct]
pub struct GPUSupportedLimits {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "defined in wgpu-types"]
    limits: Limits,
}

impl GPUSupportedLimits {
    fn new_inherited(limits: wgt::Limits) -> Self {
        Self {
            reflector_: Reflector::new(),
            limits: Limits(limits),
        }
    }

    pub fn new(global: &GlobalScope, limits: wgt::Limits) -> DomRoot<Self> {
        reflect_dom_object(Box::new(Self::new_inherited(limits)), global)
    }
}

impl GPUSupportedLimitsMethods for GPUSupportedLimits {
    /// https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-maxtexturedimension1d
    fn MaxTextureDimension1D(&self) -> u32 {
        self.limits.0.max_texture_dimension_1d
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-maxtexturedimension2d
    fn MaxTextureDimension2D(&self) -> u32 {
        self.limits.0.max_texture_dimension_2d
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-maxtexturedimension3d
    fn MaxTextureDimension3D(&self) -> u32 {
        self.limits.0.max_texture_dimension_3d
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-maxtexturearraylayers
    fn MaxTextureArrayLayers(&self) -> u32 {
        self.limits.0.max_texture_array_layers
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-maxbindgroups
    fn MaxBindGroups(&self) -> u32 {
        self.limits.0.max_bind_groups
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-maxdynamicuniformbuffersperpipelinelayout
    fn MaxDynamicUniformBuffersPerPipelineLayout(&self) -> u32 {
        self.limits
            .0
            .max_dynamic_uniform_buffers_per_pipeline_layout
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-maxdynamicstoragebuffersperpipelinelayout
    fn MaxDynamicStorageBuffersPerPipelineLayout(&self) -> u32 {
        self.limits
            .0
            .max_dynamic_storage_buffers_per_pipeline_layout
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-maxsampledtexturespershaderstage
    fn MaxSampledTexturesPerShaderStage(&self) -> u32 {
        self.limits.0.max_sampled_textures_per_shader_stage
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-maxsamplerspershaderstage
    fn MaxSamplersPerShaderStage(&self) -> u32 {
        self.limits.0.max_samplers_per_shader_stage
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-maxstoragebufferspershaderstage
    fn MaxStorageBuffersPerShaderStage(&self) -> u32 {
        self.limits.0.max_storage_buffers_per_shader_stage
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-maxstoragetexturespershaderstage
    fn MaxStorageTexturesPerShaderStage(&self) -> u32 {
        self.limits.0.max_storage_textures_per_shader_stage
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-maxuniformbufferspershaderstage
    fn MaxUniformBuffersPerShaderStage(&self) -> u32 {
        self.limits.0.max_uniform_buffers_per_shader_stage
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-maxuniformbufferbindingsize
    fn MaxUniformBufferBindingSize(&self) -> u32 {
        self.limits.0.max_uniform_buffer_binding_size
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-maxstoragebufferbindingsize
    fn MaxStorageBufferBindingSize(&self) -> u32 {
        self.limits.0.max_storage_buffer_binding_size
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-minuniformbufferoffsetalignment
    fn MinUniformBufferOffsetAlignment(&self) -> u32 {
        self.limits.0.min_uniform_buffer_offset_alignment
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-minstoragebufferoffsetalignment
    fn MinStorageBufferOffsetAlignment(&self) -> u32 {
        self.limits.0.min_storage_buffer_offset_alignment
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-maxvertexbuffers
    fn MaxVertexBuffers(&self) -> u32 {
        self.limits.0.max_vertex_buffers
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-maxvertexattributes
    fn MaxVertexAttributes(&self) -> u32 {
        self.limits.0.max_vertex_attributes
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-maxvertexbufferarraystride
    fn MaxVertexBufferArrayStride(&self) -> u32 {
        self.limits.0.max_vertex_buffer_array_stride
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-maxinterstageshadercomponents
    fn MaxInterStageShaderComponents(&self) -> u32 {
        self.limits.0.max_inter_stage_shader_components
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-maxcomputeworkgroupstoragesize
    fn MaxComputeWorkgroupStorageSize(&self) -> u32 {
        self.limits.0.max_compute_workgroup_storage_size
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-maxcomputeinvocationsperworkgroup
    fn MaxComputeInvocationsPerWorkgroup(&self) -> u32 {
        self.limits.0.max_compute_invocations_per_workgroup
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-maxcomputeworkgroupsizex
    fn MaxComputeWorkgroupSizeX(&self) -> u32 {
        self.limits.0.max_compute_workgroup_size_x
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-maxcomputeworkgroupsizey
    fn MaxComputeWorkgroupSizeY(&self) -> u32 {
        self.limits.0.max_compute_workgroup_size_y
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-maxcomputeworkgroupsizez
    fn MaxComputeWorkgroupSizeZ(&self) -> u32 {
        self.limits.0.max_compute_workgroup_size_z
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-maxcomputeworkgroupsperdimension
    fn MaxComputeWorkgroupsPerDimension(&self) -> u32 {
        self.limits.0.max_compute_workgroups_per_dimension
    }
}
