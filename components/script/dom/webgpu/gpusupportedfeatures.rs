/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// check-tidy: no specs after this line

use dom_struct::dom_struct;
use indexmap::IndexSet;
use js::rust::HandleObject;
use wgpu_types::Features;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::{
    GPUFeatureName, GPUSupportedFeaturesMethods,
};
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::like::Setlike;
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object_with_proto};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct GPUSupportedFeatures {
    reflector: Reflector,
    // internal storage for features
    #[custom_trace]
    internal: DomRefCell<IndexSet<GPUFeatureName>>,
    #[ignore_malloc_size_of = "defined in wgpu-types"]
    #[no_trace]
    features: Features,
}

impl GPUSupportedFeatures {
    fn new(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        features: Features,
        can_gc: CanGc,
    ) -> DomRoot<GPUSupportedFeatures> {
        let mut set = IndexSet::new();
        if features.contains(Features::DEPTH_CLIP_CONTROL) {
            set.insert(GPUFeatureName::Depth_clip_control);
        }
        if features.contains(Features::DEPTH32FLOAT_STENCIL8) {
            set.insert(GPUFeatureName::Depth32float_stencil8);
        }
        if features.contains(Features::TEXTURE_COMPRESSION_BC) {
            set.insert(GPUFeatureName::Texture_compression_bc);
        }
        // TODO: texture-compression-bc-sliced-3d when wgpu supports it
        if features.contains(Features::TEXTURE_COMPRESSION_ETC2) {
            set.insert(GPUFeatureName::Texture_compression_etc2);
        }
        if features.contains(Features::TEXTURE_COMPRESSION_ASTC) {
            set.insert(GPUFeatureName::Texture_compression_astc);
        }
        if features.contains(Features::TIMESTAMP_QUERY) {
            set.insert(GPUFeatureName::Timestamp_query);
        }
        if features.contains(Features::INDIRECT_FIRST_INSTANCE) {
            set.insert(GPUFeatureName::Indirect_first_instance);
        }
        // While this feature exists in wgpu, it's not supported by naga yet
        // https://github.com/gfx-rs/wgpu/issues/4384
        /*
        if features.contains(Features::SHADER_F16) {
            set.insert(GPUFeatureName::Shader_f16);
        }
        */
        if features.contains(Features::RG11B10UFLOAT_RENDERABLE) {
            set.insert(GPUFeatureName::Rg11b10ufloat_renderable);
        }
        if features.contains(Features::BGRA8UNORM_STORAGE) {
            set.insert(GPUFeatureName::Bgra8unorm_storage);
        }
        if features.contains(Features::FLOAT32_FILTERABLE) {
            set.insert(GPUFeatureName::Float32_filterable);
        }
        // TODO: clip-distances when wgpu supports it
        if features.contains(Features::DUAL_SOURCE_BLENDING) {
            set.insert(GPUFeatureName::Dual_source_blending);
        }

        reflect_dom_object_with_proto(
            Box::new(GPUSupportedFeatures {
                reflector: Reflector::new(),
                internal: DomRefCell::new(set),
                features,
            }),
            global,
            proto,
            can_gc,
        )
    }

    #[allow(non_snake_case)]
    pub(crate) fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        features: Features,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<GPUSupportedFeatures>> {
        Ok(GPUSupportedFeatures::new(global, proto, features, can_gc))
    }
}

impl GPUSupportedFeatures {
    pub(crate) fn wgpu_features(&self) -> Features {
        self.features
    }
}

impl GPUSupportedFeaturesMethods<crate::DomTypeHolder> for GPUSupportedFeatures {
    fn Size(&self) -> u32 {
        self.internal.size()
    }
}

pub(crate) fn gpu_to_wgt_feature(feature: GPUFeatureName) -> Option<Features> {
    match feature {
        GPUFeatureName::Depth_clip_control => Some(Features::DEPTH_CLIP_CONTROL),
        GPUFeatureName::Depth32float_stencil8 => Some(Features::DEPTH32FLOAT_STENCIL8),
        GPUFeatureName::Texture_compression_bc => Some(Features::TEXTURE_COMPRESSION_BC),
        GPUFeatureName::Texture_compression_etc2 => Some(Features::TEXTURE_COMPRESSION_ETC2),
        GPUFeatureName::Texture_compression_astc => Some(Features::TEXTURE_COMPRESSION_ASTC),
        GPUFeatureName::Timestamp_query => Some(Features::TIMESTAMP_QUERY),
        GPUFeatureName::Indirect_first_instance => Some(Features::INDIRECT_FIRST_INSTANCE),
        // While this feature exists in wgpu, it's not supported by naga yet
        // https://github.com/gfx-rs/wgpu/issues/4384
        GPUFeatureName::Shader_f16 => None,
        GPUFeatureName::Rg11b10ufloat_renderable => Some(Features::RG11B10UFLOAT_RENDERABLE),
        GPUFeatureName::Bgra8unorm_storage => Some(Features::BGRA8UNORM_STORAGE),
        GPUFeatureName::Float32_filterable => Some(Features::FLOAT32_FILTERABLE),
        GPUFeatureName::Dual_source_blending => Some(Features::DUAL_SOURCE_BLENDING),
        GPUFeatureName::Texture_compression_bc_sliced_3d => None,
        GPUFeatureName::Clip_distances => None,
    }
}

impl Setlike for GPUSupportedFeatures {
    type Key = DOMString;

    #[inline(always)]
    fn get_index(&self, index: u32) -> Option<Self::Key> {
        self.internal
            .get_index(index)
            .map(|k| DOMString::from_string(k.as_str().to_owned()))
    }
    #[inline(always)]
    fn size(&self) -> u32 {
        self.internal.size()
    }
    #[inline(always)]
    fn add(&self, _key: Self::Key) {
        unreachable!("readonly");
    }
    #[inline(always)]
    fn has(&self, key: Self::Key) -> bool {
        if let Ok(key) = key.parse() {
            self.internal.has(key)
        } else {
            false
        }
    }
    #[inline(always)]
    fn clear(&self) {
        unreachable!("readonly");
    }
    #[inline(always)]
    fn delete(&self, _key: Self::Key) -> bool {
        unreachable!("readonly");
    }
}
