/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// check-tidy: no specs after this line

use std::str::FromStr;

use dom_struct::dom_struct;
use indexmap::IndexSet;
use js::rust::HandleObject;
use webgpu::wgt;

use super::bindings::like::Setlike;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::GPUFeatureNameValues::pairs;
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::{
    GPUFeatureName, GPUSupportedFeaturesMethods,
};
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;

// manual hash derived
// TODO: allow derivables in bindings.conf
impl std::hash::Hash for GPUFeatureName {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

impl Eq for GPUFeatureName {}

#[dom_struct]
pub struct GPUSupportedFeatures {
    reflector: Reflector,
    // internal storage for features
    #[custom_trace]
    internal: DomRefCell<IndexSet<GPUFeatureName>>,
}

impl GPUSupportedFeatures {
    fn new(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        features: wgt::Features,
    ) -> DomRoot<GPUSupportedFeatures> {
        let mut set = IndexSet::new();
        if features.contains(wgt::Features::DEPTH_CLIP_CONTROL) {
            set.insert(GPUFeatureName::Depth_clip_control);
        }
        if features.contains(wgt::Features::DEPTH32FLOAT_STENCIL8) {
            set.insert(GPUFeatureName::Depth32float_stencil8);
        }
        if features.contains(wgt::Features::PIPELINE_STATISTICS_QUERY) {
            set.insert(GPUFeatureName::Pipeline_statistics_query);
        }
        if features.contains(wgt::Features::TEXTURE_COMPRESSION_BC) {
            set.insert(GPUFeatureName::Texture_compression_bc);
        }
        if features.contains(wgt::Features::TEXTURE_COMPRESSION_ETC2) {
            set.insert(GPUFeatureName::Texture_compression_etc2);
        }
        if features.contains(wgt::Features::TEXTURE_COMPRESSION_ASTC) {
            set.insert(GPUFeatureName::Texture_compression_astc);
        }
        if features.contains(wgt::Features::TIMESTAMP_QUERY) {
            set.insert(GPUFeatureName::Timestamp_query);
        }
        if features.contains(wgt::Features::INDIRECT_FIRST_INSTANCE) {
            set.insert(GPUFeatureName::Indirect_first_instance);
        }
        reflect_dom_object_with_proto(
            Box::new(GPUSupportedFeatures {
                reflector: Reflector::new(),
                internal: DomRefCell::new(set),
            }),
            global,
            proto,
        )
    }

    #[allow(non_snake_case)]
    pub fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        features: wgt::Features,
    ) -> Fallible<DomRoot<GPUSupportedFeatures>> {
        Ok(GPUSupportedFeatures::new(global, proto, features))
    }
}

impl GPUSupportedFeaturesMethods for GPUSupportedFeatures {
    fn Size(&self) -> u32 {
        self.internal.size()
    }
}

pub fn gpu_to_wgt_feature(feature: GPUFeatureName) -> Option<wgt::Features> {
    match feature {
        GPUFeatureName::Depth_clip_control => Some(wgt::Features::DEPTH_CLIP_CONTROL),
        GPUFeatureName::Depth24unorm_stencil8 => None,
        GPUFeatureName::Depth32float_stencil8 => Some(wgt::Features::DEPTH32FLOAT_STENCIL8),
        GPUFeatureName::Pipeline_statistics_query => Some(wgt::Features::PIPELINE_STATISTICS_QUERY),
        GPUFeatureName::Texture_compression_bc => Some(wgt::Features::TEXTURE_COMPRESSION_BC),
        GPUFeatureName::Texture_compression_etc2 => Some(wgt::Features::TEXTURE_COMPRESSION_ETC2),
        GPUFeatureName::Texture_compression_astc => Some(wgt::Features::TEXTURE_COMPRESSION_ASTC),
        GPUFeatureName::Timestamp_query => Some(wgt::Features::TIMESTAMP_QUERY),
        GPUFeatureName::Indirect_first_instance => Some(wgt::Features::INDIRECT_FIRST_INSTANCE),
    }
}

// this should be autogenerate by bindings
impl FromStr for GPUFeatureName {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        pairs
            .iter()
            .find(|&&(key, _)| s == key)
            .map(|&(_, ev)| ev)
            .ok_or(())
    }
}

// this error is wrong because if we inline Self::Key and Self::Value all errors are gone
#[allow(crown::unrooted_must_root)]
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
