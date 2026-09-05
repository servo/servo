/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![cfg_attr(crown, feature(register_tool))]
// Register the linter `crown`, which is the Servo-specific linter for the script crate.
#![cfg_attr(crown, register_tool(crown))]

pub mod datablock;
pub mod gpu;
pub mod gpuadapter;
pub mod gpuadapterinfo;
pub mod gpubindgroup;
pub mod gpubindgrouplayout;
pub mod gpubuffer;
pub mod gpubufferusage;
pub mod gpucolorwrite;
pub mod gpucommandbuffer;
pub mod gpucommandencoder;
pub mod gpucompilationinfo;
pub mod gpucompilationmessage;
pub mod gpucomputepassencoder;
pub mod gpucomputepipeline;
pub mod gpuconvert;
pub mod gpudevicelostinfo;
pub mod gpumapmode;
pub mod gpupipelinelayout;
pub mod gpuqueryset;
pub mod gpurenderbundle;
pub mod gpurenderbundleencoder;
pub mod gpurenderpassencoder;
pub mod gpurenderpipeline;
pub mod gpusampler;
pub mod gpushadermodule;
pub mod gpushaderstage;
pub mod gpusupportedfeatures;
pub mod gpusupportedlimits;
pub mod gputexture;
pub mod gputextureusage;
pub mod gputextureview;
pub mod identityhub;
pub mod traits;
pub mod wgsllanguagefeatures;

pub(crate) use js::gc::Traceable as JSTraceable;
pub(crate) use jstraceable_derive::JSTraceable;
pub(crate) use script_bindings::reflector::{DomObject, MutDomObject, Reflector};
pub(crate) use script_bindings::trace::CustomTraceable;
use wgpu_core::id::PipelineLayoutId;

pub(crate) use crate::dom::bindings::inheritance::HasParent;

pub enum PipelineLayout {
    Implicit,
    Explicit(PipelineLayoutId),
}

impl PipelineLayout {
    pub fn explicit(&self) -> Option<PipelineLayoutId> {
        match self {
            PipelineLayout::Explicit(layout_id) => Some(*layout_id),
            PipelineLayout::Implicit => None,
        }
    }
}

// Reexports
pub(crate) mod dom {
    pub(crate) mod types {}
    pub(crate) mod bindings {
        pub(crate) use script_bindings::*;
    }
}

/// Generated JS-Rust bindings.
#[allow(missing_docs, non_snake_case)]
pub(crate) mod codegen {
    #[expect(unused)]
    pub(crate) mod Bindings {
        use std::ptr;

        use js::context::JSContext;
        use js::gc::HandleObject;
        pub(crate) use script_bindings::DomTypes;
        use script_bindings::conversions::IDLInterface;
        use script_bindings::reflector::DomObjectWrap;
        pub(crate) use script_bindings::reflector::Reflector;
        use script_bindings::root::{Dom, DomRoot, Root};
        use script_bindings::utils::DOMClass;

        use crate::gpu::GPU;
        use crate::gpuadapter::GPUAdapter;
        use crate::gpuadapterinfo::GPUAdapterInfo;
        use crate::gpubindgroup::GPUBindGroup;
        use crate::gpubindgrouplayout::GPUBindGroupLayout;
        use crate::gpubuffer::GPUBuffer;
        use crate::gpubufferusage::GPUBufferUsage;
        use crate::gpucolorwrite::GPUColorWrite;
        use crate::gpucommandbuffer::GPUCommandBuffer;
        use crate::gpucommandencoder::GPUCommandEncoder;
        use crate::gpucompilationinfo::GPUCompilationInfo;
        use crate::gpucompilationmessage::GPUCompilationMessage;
        use crate::gpucomputepassencoder::GPUComputePassEncoder;
        use crate::gpucomputepipeline::GPUComputePipeline;
        use crate::gpudevicelostinfo::GPUDeviceLostInfo;
        use crate::gpumapmode::GPUMapMode;
        use crate::gpupipelinelayout::GPUPipelineLayout;
        use crate::gpuqueryset::GPUQuerySet;
        use crate::gpurenderbundle::GPURenderBundle;
        use crate::gpurenderbundleencoder::GPURenderBundleEncoder;
        use crate::gpurenderpassencoder::GPURenderPassEncoder;
        use crate::gpurenderpipeline::GPURenderPipeline;
        use crate::gpusampler::GPUSampler;
        use crate::gpushadermodule::GPUShaderModule;
        use crate::gpushaderstage::GPUShaderStage;
        use crate::gpusupportedfeatures::GPUSupportedFeatures;
        use crate::gpusupportedlimits::GPUSupportedLimits;
        use crate::gputexture::GPUTexture;
        use crate::gputextureusage::GPUTextureUsage;
        use crate::gputextureview::GPUTextureView;
        use crate::wgsllanguagefeatures::WGSLLanguageFeatures;
        include!(concat!(
            env!("OUT_DIR"),
            "/ConcreteBindings/WebGPUBinding.rs"
        ));
    }
}
