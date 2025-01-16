/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use num_traits::bounds::UpperBounded;
use webgpu::wgt::Limits;
use GPUSupportedLimits_Binding::GPUSupportedLimitsMethods;

use crate::dom::bindings::codegen::Bindings::WebGPUBinding::GPUSupportedLimits_Binding;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct GPUSupportedLimits {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "defined in wgpu-types"]
    #[no_trace]
    limits: Limits,
}

impl GPUSupportedLimits {
    fn new_inherited(limits: Limits) -> Self {
        Self {
            reflector_: Reflector::new(),
            limits,
        }
    }

    pub(crate) fn new(global: &GlobalScope, limits: Limits) -> DomRoot<Self> {
        reflect_dom_object(Box::new(Self::new_inherited(limits)), global, CanGc::note())
    }
}

impl GPUSupportedLimitsMethods<crate::DomTypeHolder> for GPUSupportedLimits {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-maxtexturedimension1d>
    fn MaxTextureDimension1D(&self) -> u32 {
        self.limits.max_texture_dimension_1d
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-maxtexturedimension2d>
    fn MaxTextureDimension2D(&self) -> u32 {
        self.limits.max_texture_dimension_2d
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-maxtexturedimension3d>
    fn MaxTextureDimension3D(&self) -> u32 {
        self.limits.max_texture_dimension_3d
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-maxtexturearraylayers>
    fn MaxTextureArrayLayers(&self) -> u32 {
        self.limits.max_texture_array_layers
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-maxbindgroups>
    fn MaxBindGroups(&self) -> u32 {
        self.limits.max_bind_groups
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-maxbindingsperbindgroup>
    fn MaxBindingsPerBindGroup(&self) -> u32 {
        self.limits.max_bindings_per_bind_group
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-maxdynamicuniformbuffersperpipelinelayout>
    fn MaxDynamicUniformBuffersPerPipelineLayout(&self) -> u32 {
        self.limits.max_dynamic_uniform_buffers_per_pipeline_layout
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-maxdynamicstoragebuffersperpipelinelayout>
    fn MaxDynamicStorageBuffersPerPipelineLayout(&self) -> u32 {
        self.limits.max_dynamic_storage_buffers_per_pipeline_layout
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-maxsampledtexturespershaderstage>
    fn MaxSampledTexturesPerShaderStage(&self) -> u32 {
        self.limits.max_sampled_textures_per_shader_stage
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-maxsamplerspershaderstage>
    fn MaxSamplersPerShaderStage(&self) -> u32 {
        self.limits.max_samplers_per_shader_stage
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-maxstoragebufferspershaderstage>
    fn MaxStorageBuffersPerShaderStage(&self) -> u32 {
        self.limits.max_storage_buffers_per_shader_stage
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-maxstoragetexturespershaderstage>
    fn MaxStorageTexturesPerShaderStage(&self) -> u32 {
        self.limits.max_storage_textures_per_shader_stage
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-maxuniformbufferspershaderstage>
    fn MaxUniformBuffersPerShaderStage(&self) -> u32 {
        self.limits.max_uniform_buffers_per_shader_stage
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-maxuniformbufferbindingsize>
    fn MaxUniformBufferBindingSize(&self) -> u64 {
        self.limits.max_uniform_buffer_binding_size as u64
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-maxstoragebufferbindingsize>
    fn MaxStorageBufferBindingSize(&self) -> u64 {
        self.limits.max_storage_buffer_binding_size as u64
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-minuniformbufferoffsetalignment>
    fn MinUniformBufferOffsetAlignment(&self) -> u32 {
        self.limits.min_uniform_buffer_offset_alignment
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-minstoragebufferoffsetalignment>
    fn MinStorageBufferOffsetAlignment(&self) -> u32 {
        self.limits.min_storage_buffer_offset_alignment
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-maxvertexbuffers>
    fn MaxVertexBuffers(&self) -> u32 {
        self.limits.max_vertex_buffers
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-maxbuffersize>
    fn MaxBufferSize(&self) -> u64 {
        self.limits.max_buffer_size
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-maxvertexattributes>
    fn MaxVertexAttributes(&self) -> u32 {
        self.limits.max_vertex_attributes
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-maxvertexbufferarraystride>
    fn MaxVertexBufferArrayStride(&self) -> u32 {
        self.limits.max_vertex_buffer_array_stride
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-maxinterstageshadercomponents>
    fn MaxInterStageShaderComponents(&self) -> u32 {
        self.limits.max_inter_stage_shader_components
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-maxcomputeworkgroupstoragesize>
    fn MaxComputeWorkgroupStorageSize(&self) -> u32 {
        self.limits.max_compute_workgroup_storage_size
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-maxcomputeinvocationsperworkgroup>
    fn MaxComputeInvocationsPerWorkgroup(&self) -> u32 {
        self.limits.max_compute_invocations_per_workgroup
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-maxcomputeworkgroupsizex>
    fn MaxComputeWorkgroupSizeX(&self) -> u32 {
        self.limits.max_compute_workgroup_size_x
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-maxcomputeworkgroupsizey>
    fn MaxComputeWorkgroupSizeY(&self) -> u32 {
        self.limits.max_compute_workgroup_size_y
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-maxcomputeworkgroupsizez>
    fn MaxComputeWorkgroupSizeZ(&self) -> u32 {
        self.limits.max_compute_workgroup_size_z
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-maxcomputeworkgroupsperdimension>
    fn MaxComputeWorkgroupsPerDimension(&self) -> u32 {
        self.limits.max_compute_workgroups_per_dimension
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-maxbindgroupsplusvertexbuffers>
    fn MaxBindGroupsPlusVertexBuffers(&self) -> u32 {
        // Not on wgpu yet, so we craft it manually
        self.limits.max_bind_groups + self.limits.max_vertex_buffers
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-maxinterstageshadervariables>
    fn MaxInterStageShaderVariables(&self) -> u32 {
        // Not in wgpu yet, so we use default value from spec
        16
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-maxcolorattachments>
    fn MaxColorAttachments(&self) -> u32 {
        self.limits.max_color_attachments
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpusupportedlimits-maxcolorattachmentbytespersample>
    fn MaxColorAttachmentBytesPerSample(&self) -> u32 {
        self.limits.max_color_attachment_bytes_per_sample
    }
}

/// Returns false if unknown limit or other value error
pub(crate) fn set_limit(limits: &mut Limits, limit: &str, value: u64) -> bool {
    /// per spec defaults are lower bounds for values
    ///
    /// <https://www.w3.org/TR/webgpu/#limit-class-maximum>
    fn set_maximum<T>(limit: &mut T, value: u64) -> bool
    where
        T: Ord + Copy + TryFrom<u64> + UpperBounded,
    {
        if let Ok(value) = T::try_from(value) {
            *limit = value.max(*limit);
            true
        } else {
            false
        }
    }

    /// per spec defaults are higher bounds for values
    ///
    /// <https://www.w3.org/TR/webgpu/#limit-class-alignment>
    fn set_alignment<T>(limit: &mut T, value: u64) -> bool
    where
        T: Ord + Copy + TryFrom<u64> + UpperBounded,
    {
        if !value.is_power_of_two() {
            return false;
        }
        if let Ok(value) = T::try_from(value) {
            *limit = value.min(*limit);
            true
        } else {
            false
        }
    }

    match limit {
        "maxTextureDimension1D" => set_maximum(&mut limits.max_texture_dimension_1d, value),
        "maxTextureDimension2D" => set_maximum(&mut limits.max_texture_dimension_2d, value),
        "maxTextureDimension3D" => set_maximum(&mut limits.max_texture_dimension_3d, value),
        "maxTextureArrayLayers" => set_maximum(&mut limits.max_texture_array_layers, value),
        "maxBindGroups" => set_maximum(&mut limits.max_bind_groups, value),
        "maxBindGroupsPlusVertexBuffers" => {
            // not in wgpu but we're allowed to give back better limits than requested.
            // we use dummy value to still produce value verification
            let mut v: u32 = 0;
            set_maximum(&mut v, value)
        },
        "maxBindingsPerBindGroup" => set_maximum(&mut limits.max_bindings_per_bind_group, value),
        "maxDynamicUniformBuffersPerPipelineLayout" => set_maximum(
            &mut limits.max_dynamic_uniform_buffers_per_pipeline_layout,
            value,
        ),
        "maxDynamicStorageBuffersPerPipelineLayout" => set_maximum(
            &mut limits.max_dynamic_storage_buffers_per_pipeline_layout,
            value,
        ),
        "maxSampledTexturesPerShaderStage" => {
            set_maximum(&mut limits.max_sampled_textures_per_shader_stage, value)
        },
        "maxSamplersPerShaderStage" => {
            set_maximum(&mut limits.max_samplers_per_shader_stage, value)
        },
        "maxStorageBuffersPerShaderStage" => {
            set_maximum(&mut limits.max_storage_buffers_per_shader_stage, value)
        },
        "maxStorageTexturesPerShaderStage" => {
            set_maximum(&mut limits.max_storage_textures_per_shader_stage, value)
        },
        "maxUniformBuffersPerShaderStage" => {
            set_maximum(&mut limits.max_uniform_buffers_per_shader_stage, value)
        },
        "maxUniformBufferBindingSize" => {
            set_maximum(&mut limits.max_uniform_buffer_binding_size, value)
        },
        "maxStorageBufferBindingSize" => {
            set_maximum(&mut limits.max_storage_buffer_binding_size, value)
        },
        "minUniformBufferOffsetAlignment" => {
            set_alignment(&mut limits.min_uniform_buffer_offset_alignment, value)
        },
        "minStorageBufferOffsetAlignment" => {
            set_alignment(&mut limits.min_storage_buffer_offset_alignment, value)
        },
        "maxVertexBuffers" => set_maximum(&mut limits.max_vertex_buffers, value),
        "maxBufferSize" => set_maximum(&mut limits.max_buffer_size, value),
        "maxVertexAttributes" => set_maximum(&mut limits.max_vertex_attributes, value),
        "maxVertexBufferArrayStride" => {
            set_maximum(&mut limits.max_vertex_buffer_array_stride, value)
        },
        "maxInterStageShaderComponents" => {
            set_maximum(&mut limits.max_inter_stage_shader_components, value)
        },
        "maxInterStageShaderVariables" => {
            // not in wgpu but we're allowed to give back better limits than requested.
            // we use dummy value to still produce value verification
            let mut v: u32 = 0;
            set_maximum(&mut v, value)
        },
        "maxColorAttachments" => set_maximum(&mut limits.max_color_attachments, value),
        "maxColorAttachmentBytesPerSample" => {
            set_maximum(&mut limits.max_color_attachment_bytes_per_sample, value)
        },
        "maxComputeWorkgroupStorageSize" => {
            set_maximum(&mut limits.max_compute_workgroup_storage_size, value)
        },
        "maxComputeInvocationsPerWorkgroup" => {
            set_maximum(&mut limits.max_compute_invocations_per_workgroup, value)
        },
        "maxComputeWorkgroupSizeX" => set_maximum(&mut limits.max_compute_workgroup_size_x, value),
        "maxComputeWorkgroupSizeY" => set_maximum(&mut limits.max_compute_workgroup_size_y, value),
        "maxComputeWorkgroupSizeZ" => set_maximum(&mut limits.max_compute_workgroup_size_z, value),
        "maxComputeWorkgroupsPerDimension" => {
            set_maximum(&mut limits.max_compute_workgroups_per_dimension, value)
        },
        _ => false,
    }
}
