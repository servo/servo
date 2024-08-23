/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;
use std::num::NonZeroU64;

use script_bindings::DomTypes;
use webgpu::wgc::binding_model::{BindGroupEntry, BindingResource, BufferBinding};
use webgpu::wgc::command as wgpu_com;
use webgpu::wgc::pipeline::ProgrammableStageDescriptor;
use webgpu::wgc::resource::TextureDescriptor;
use webgpu::wgt::{self, AstcBlock, AstcChannel};

use super::bindings::codegen::Bindings::WebGPUBinding::{
    GPUProgrammableStage, GPUTextureDescriptor, GPUTextureDimension,
};
use super::bindings::error::Error;
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::{
    GPUAddressMode, GPUBindGroupEntry, GPUBindGroupLayoutEntry, GPUBindingResource,
    GPUBlendComponent, GPUBlendFactor, GPUBlendOperation, GPUBufferBindingType, GPUColor,
    GPUCompareFunction, GPUCullMode, GPUExtent3D, GPUFilterMode, GPUFrontFace, GPUImageCopyBuffer,
    GPUImageCopyTexture, GPUImageDataLayout, GPUIndexFormat, GPULoadOp, GPUObjectDescriptorBase,
    GPUOrigin3D, GPUPrimitiveState, GPUPrimitiveTopology, GPUSamplerBindingType,
    GPUStencilOperation, GPUStorageTextureAccess, GPUStoreOp, GPUTextureAspect, GPUTextureFormat,
    GPUTextureSampleType, GPUTextureViewDimension, GPUVertexFormat,
};
use crate::dom::bindings::error::Fallible;
use crate::dom::types::GPUDevice;

pub fn convert_image_copy_buffer(ic_buffer: &GPUImageCopyBuffer) -> wgpu_com::ImageCopyBuffer {
    wgpu_com::ImageCopyBuffer {
        buffer: ic_buffer.buffer.id().0,
        layout: wgt::ImageDataLayout::from(&ic_buffer.parent),
    }
}

pub fn convert_image_copy_texture(ic_texture: &GPUImageCopyTexture) -> Result<wgpu_com::ImageCopyTexture, Error> {
    Ok(wgpu_com::ImageCopyTexture {
        texture: ic_texture.texture.id().0,
        mip_level: ic_texture.mipLevel,
        origin: ic_texture
            .origin
            .as_ref()
            .map(wgt::Origin3d::try_from)
            .transpose()?
            .unwrap_or_default(),
        aspect: match ic_texture.aspect {
            GPUTextureAspect::All => wgt::TextureAspect::All,
            GPUTextureAspect::Stencil_only => wgt::TextureAspect::StencilOnly,
            GPUTextureAspect::Depth_only => wgt::TextureAspect::DepthOnly,
        },
    })
}

pub fn convert_programmable_stage<'a>(stage: &GPUProgrammableStage) -> ProgrammableStageDescriptor<'a> {
    ProgrammableStageDescriptor {
        module: stage.module.id().0,
        entry_point: stage
            .entryPoint
            .as_ref()
            .map(|ep| Cow::Owned(ep.to_string())),
        constants: Cow::Owned(
            stage
                .constants
                .as_ref()
                .map(|records| records.iter().map(|(k, v)| (k.0.clone(), **v)).collect())
                .unwrap_or_default(),
        ),
        zero_initialize_workgroup_memory: true,
    }
}

pub fn convert_bind_group_entry<'a>(entry: &GPUBindGroupEntry) -> BindGroupEntry<'a> {
    BindGroupEntry {
        binding: entry.binding,
        resource: match entry.resource {
            GPUBindingResource::GPUSampler(ref s) => BindingResource::Sampler(s.id().0),
            GPUBindingResource::GPUTextureView(ref t) => BindingResource::TextureView(t.id().0),
            GPUBindingResource::GPUBufferBinding(ref b) => {
                BindingResource::Buffer(BufferBinding {
                    buffer_id: b.buffer.id().0,
                    offset: b.offset,
                    size: b.size.and_then(wgt::BufferSize::new),
                })
            },
        },
    }
}

pub fn convert_load_op(op: Option<GPULoadOp>) -> wgpu_com::LoadOp {
    match op {
        Some(GPULoadOp::Load) => wgpu_com::LoadOp::Load,
        Some(GPULoadOp::Clear) => wgpu_com::LoadOp::Clear,
        None => wgpu_com::LoadOp::Clear,
    }
}

pub fn convert_store_op(op: Option<GPUStoreOp>) -> wgpu_com::StoreOp {
    match op {
        Some(GPUStoreOp::Store) => wgpu_com::StoreOp::Store,
        Some(GPUStoreOp::Discard) => wgpu_com::StoreOp::Discard,
        None => wgpu_com::StoreOp::Discard,
    }
}

pub fn convert_bind_group_layout_entry(
    bgle: &GPUBindGroupLayoutEntry,
    device: &GPUDevice,
) -> Fallible<Result<wgt::BindGroupLayoutEntry, webgpu::Error>> {
    let number_of_provided_bindings = bgle.buffer.is_some() as u8 +
        bgle.sampler.is_some() as u8 +
        bgle.storageTexture.is_some() as u8 +
        bgle.texture.is_some() as u8;
    let ty = if let Some(buffer) = &bgle.buffer {
        Some(wgt::BindingType::Buffer {
            ty: match buffer.type_ {
                GPUBufferBindingType::Uniform => wgt::BufferBindingType::Uniform,
                GPUBufferBindingType::Storage => {
                    wgt::BufferBindingType::Storage { read_only: false }
                },
                GPUBufferBindingType::Read_only_storage => {
                    wgt::BufferBindingType::Storage { read_only: true }
                },
            },
            has_dynamic_offset: buffer.hasDynamicOffset,
            min_binding_size: NonZeroU64::new(buffer.minBindingSize),
        })
    } else if let Some(sampler) = &bgle.sampler {
        Some(wgt::BindingType::Sampler(match sampler.type_ {
            GPUSamplerBindingType::Filtering => wgt::SamplerBindingType::Filtering,
            GPUSamplerBindingType::Non_filtering => wgt::SamplerBindingType::NonFiltering,
            GPUSamplerBindingType::Comparison => wgt::SamplerBindingType::Comparison,
        }))
    } else if let Some(storage) = &bgle.storageTexture {
        Some(wgt::BindingType::StorageTexture {
            access: match storage.access {
                GPUStorageTextureAccess::Write_only => wgt::StorageTextureAccess::WriteOnly,
                GPUStorageTextureAccess::Read_only => wgt::StorageTextureAccess::ReadOnly,
                GPUStorageTextureAccess::Read_write => wgt::StorageTextureAccess::ReadWrite,
            },
            format: device.validate_texture_format_required_features(&storage.format)?,
            view_dimension: storage.viewDimension.into(),
        })
    } else if let Some(texture) = &bgle.texture {
        Some(wgt::BindingType::Texture {
            sample_type: match texture.sampleType {
                GPUTextureSampleType::Float => wgt::TextureSampleType::Float { filterable: true },
                GPUTextureSampleType::Unfilterable_float => {
                    wgt::TextureSampleType::Float { filterable: false }
                },
                GPUTextureSampleType::Depth => wgt::TextureSampleType::Depth,
                GPUTextureSampleType::Sint => wgt::TextureSampleType::Sint,
                GPUTextureSampleType::Uint => wgt::TextureSampleType::Uint,
            },
            view_dimension: texture.viewDimension.into(),
            multisampled: texture.multisampled,
        })
    } else {
        assert_eq!(number_of_provided_bindings, 0);
        None
    };
    // Check for number of bindings should actually be done in device-timeline,
    // but we do it last on content-timeline to have some visible effect
    let ty = if number_of_provided_bindings != 1 {
        None
    } else {
        ty
    }
    .ok_or(webgpu::Error::Validation(
        "Exactly on entry type must be provided".to_string(),
    ));

    Ok(ty.map(|ty| wgt::BindGroupLayoutEntry {
        binding: bgle.binding,
        visibility: wgt::ShaderStages::from_bits_retain(bgle.visibility),
        ty,
        count: None,
    }))
}

pub fn convert_texture_descriptor(
    descriptor: &GPUTextureDescriptor,
    device: &GPUDevice,
) -> Fallible<(TextureDescriptor<'static>, wgt::Extent3d)> {
    let size = (&descriptor.size).try_into()?;
    let desc = TextureDescriptor {
        label: (&descriptor.parent).into(),
        size,
        mip_level_count: descriptor.mipLevelCount,
        sample_count: descriptor.sampleCount,
        dimension: descriptor.dimension.into(),
        format: device.validate_texture_format_required_features(&descriptor.format)?,
        usage: wgt::TextureUsages::from_bits_retain(descriptor.usage),
        view_formats: descriptor
            .viewFormats
            .iter()
            .map(|tf| device.validate_texture_format_required_features(tf))
            .collect::<Fallible<_>>()?,
    };
    Ok((desc, size))
}
