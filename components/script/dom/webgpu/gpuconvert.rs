/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;
use std::num::NonZeroU64;

use webgpu::wgc::binding_model::{BindGroupEntry, BindingResource, BufferBinding};
use webgpu::wgc::command as wgpu_com;
use webgpu::wgc::pipeline::ProgrammableStageDescriptor;
use webgpu::wgc::resource::TextureDescriptor;
use webgpu::wgt::{self, AstcBlock, AstcChannel};

use crate::conversions::{Convert, TryConvert};
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::{
    GPUAddressMode, GPUBindGroupEntry, GPUBindGroupLayoutEntry, GPUBindingResource,
    GPUBlendComponent, GPUBlendFactor, GPUBlendOperation, GPUBufferBindingType, GPUColor,
    GPUCompareFunction, GPUCullMode, GPUExtent3D, GPUFilterMode, GPUFrontFace, GPUImageCopyBuffer,
    GPUImageCopyTexture, GPUImageDataLayout, GPUIndexFormat, GPULoadOp, GPUObjectDescriptorBase,
    GPUOrigin3D, GPUPrimitiveState, GPUPrimitiveTopology, GPUProgrammableStage,
    GPUSamplerBindingType, GPUStencilOperation, GPUStorageTextureAccess, GPUStoreOp,
    GPUTextureAspect, GPUTextureDescriptor, GPUTextureDimension, GPUTextureFormat,
    GPUTextureSampleType, GPUTextureViewDimension, GPUVertexFormat,
};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::types::GPUDevice;

impl Convert<wgt::TextureFormat> for GPUTextureFormat {
    fn convert(self) -> wgt::TextureFormat {
        match self {
            GPUTextureFormat::R8unorm => wgt::TextureFormat::R8Unorm,
            GPUTextureFormat::R8snorm => wgt::TextureFormat::R8Snorm,
            GPUTextureFormat::R8uint => wgt::TextureFormat::R8Uint,
            GPUTextureFormat::R8sint => wgt::TextureFormat::R8Sint,
            GPUTextureFormat::R16uint => wgt::TextureFormat::R16Uint,
            GPUTextureFormat::R16sint => wgt::TextureFormat::R16Sint,
            GPUTextureFormat::R16float => wgt::TextureFormat::R16Float,
            GPUTextureFormat::Rg8unorm => wgt::TextureFormat::Rg8Unorm,
            GPUTextureFormat::Rg8snorm => wgt::TextureFormat::Rg8Snorm,
            GPUTextureFormat::Rg8uint => wgt::TextureFormat::Rg8Uint,
            GPUTextureFormat::Rg8sint => wgt::TextureFormat::Rg8Sint,
            GPUTextureFormat::R32uint => wgt::TextureFormat::R32Uint,
            GPUTextureFormat::R32sint => wgt::TextureFormat::R32Sint,
            GPUTextureFormat::R32float => wgt::TextureFormat::R32Float,
            GPUTextureFormat::Rg16uint => wgt::TextureFormat::Rg16Uint,
            GPUTextureFormat::Rg16sint => wgt::TextureFormat::Rg16Sint,
            GPUTextureFormat::Rg16float => wgt::TextureFormat::Rg16Float,
            GPUTextureFormat::Rgba8unorm => wgt::TextureFormat::Rgba8Unorm,
            GPUTextureFormat::Rgba8unorm_srgb => wgt::TextureFormat::Rgba8UnormSrgb,
            GPUTextureFormat::Rgba8snorm => wgt::TextureFormat::Rgba8Snorm,
            GPUTextureFormat::Rgba8uint => wgt::TextureFormat::Rgba8Uint,
            GPUTextureFormat::Rgba8sint => wgt::TextureFormat::Rgba8Sint,
            GPUTextureFormat::Bgra8unorm => wgt::TextureFormat::Bgra8Unorm,
            GPUTextureFormat::Bgra8unorm_srgb => wgt::TextureFormat::Bgra8UnormSrgb,
            GPUTextureFormat::Rgb10a2unorm => wgt::TextureFormat::Rgb10a2Unorm,
            GPUTextureFormat::Rg32uint => wgt::TextureFormat::Rg32Uint,
            GPUTextureFormat::Rg32sint => wgt::TextureFormat::Rg32Sint,
            GPUTextureFormat::Rg32float => wgt::TextureFormat::Rg32Float,
            GPUTextureFormat::Rgba16uint => wgt::TextureFormat::Rgba16Uint,
            GPUTextureFormat::Rgba16sint => wgt::TextureFormat::Rgba16Sint,
            GPUTextureFormat::Rgba16float => wgt::TextureFormat::Rgba16Float,
            GPUTextureFormat::Rgba32uint => wgt::TextureFormat::Rgba32Uint,
            GPUTextureFormat::Rgba32sint => wgt::TextureFormat::Rgba32Sint,
            GPUTextureFormat::Rgba32float => wgt::TextureFormat::Rgba32Float,
            GPUTextureFormat::Depth32float => wgt::TextureFormat::Depth32Float,
            GPUTextureFormat::Depth24plus => wgt::TextureFormat::Depth24Plus,
            GPUTextureFormat::Depth24plus_stencil8 => wgt::TextureFormat::Depth24PlusStencil8,
            GPUTextureFormat::Bc1_rgba_unorm => wgt::TextureFormat::Bc1RgbaUnorm,
            GPUTextureFormat::Bc1_rgba_unorm_srgb => wgt::TextureFormat::Bc1RgbaUnormSrgb,
            GPUTextureFormat::Bc2_rgba_unorm => wgt::TextureFormat::Bc2RgbaUnorm,
            GPUTextureFormat::Bc2_rgba_unorm_srgb => wgt::TextureFormat::Bc2RgbaUnormSrgb,
            GPUTextureFormat::Bc3_rgba_unorm => wgt::TextureFormat::Bc3RgbaUnorm,
            GPUTextureFormat::Bc3_rgba_unorm_srgb => wgt::TextureFormat::Bc3RgbaUnormSrgb,
            GPUTextureFormat::Bc4_r_unorm => wgt::TextureFormat::Bc4RUnorm,
            GPUTextureFormat::Bc4_r_snorm => wgt::TextureFormat::Bc4RSnorm,
            GPUTextureFormat::Bc5_rg_unorm => wgt::TextureFormat::Bc5RgUnorm,
            GPUTextureFormat::Bc5_rg_snorm => wgt::TextureFormat::Bc5RgSnorm,
            GPUTextureFormat::Bc6h_rgb_ufloat => wgt::TextureFormat::Bc6hRgbUfloat,
            GPUTextureFormat::Bc7_rgba_unorm => wgt::TextureFormat::Bc7RgbaUnorm,
            GPUTextureFormat::Bc7_rgba_unorm_srgb => wgt::TextureFormat::Bc7RgbaUnormSrgb,
            GPUTextureFormat::Bc6h_rgb_float => wgt::TextureFormat::Bc6hRgbFloat,
            GPUTextureFormat::Rgb9e5ufloat => wgt::TextureFormat::Rgb9e5Ufloat,
            GPUTextureFormat::Rgb10a2uint => wgt::TextureFormat::Rgb10a2Uint,
            GPUTextureFormat::Rg11b10ufloat => wgt::TextureFormat::Rg11b10Ufloat,
            GPUTextureFormat::Stencil8 => wgt::TextureFormat::Stencil8,
            GPUTextureFormat::Depth16unorm => wgt::TextureFormat::Depth16Unorm,
            GPUTextureFormat::Depth32float_stencil8 => wgt::TextureFormat::Depth32FloatStencil8,
            GPUTextureFormat::Etc2_rgb8unorm => wgt::TextureFormat::Etc2Rgb8Unorm,
            GPUTextureFormat::Etc2_rgb8unorm_srgb => wgt::TextureFormat::Etc2Rgb8UnormSrgb,
            GPUTextureFormat::Etc2_rgb8a1unorm => wgt::TextureFormat::Etc2Rgb8A1Unorm,
            GPUTextureFormat::Etc2_rgb8a1unorm_srgb => wgt::TextureFormat::Etc2Rgb8A1UnormSrgb,
            GPUTextureFormat::Etc2_rgba8unorm => wgt::TextureFormat::Etc2Rgba8Unorm,
            GPUTextureFormat::Etc2_rgba8unorm_srgb => wgt::TextureFormat::Etc2Rgba8UnormSrgb,
            GPUTextureFormat::Eac_r11unorm => wgt::TextureFormat::EacR11Unorm,
            GPUTextureFormat::Eac_r11snorm => wgt::TextureFormat::EacR11Snorm,
            GPUTextureFormat::Eac_rg11unorm => wgt::TextureFormat::EacRg11Unorm,
            GPUTextureFormat::Eac_rg11snorm => wgt::TextureFormat::EacRg11Snorm,
            GPUTextureFormat::Astc_4x4_unorm => wgt::TextureFormat::Astc {
                block: AstcBlock::B4x4,
                channel: AstcChannel::Unorm,
            },
            GPUTextureFormat::Astc_4x4_unorm_srgb => wgt::TextureFormat::Astc {
                block: AstcBlock::B4x4,
                channel: AstcChannel::UnormSrgb,
            },
            GPUTextureFormat::Astc_5x4_unorm => wgt::TextureFormat::Astc {
                block: AstcBlock::B5x4,
                channel: AstcChannel::Unorm,
            },
            GPUTextureFormat::Astc_5x4_unorm_srgb => wgt::TextureFormat::Astc {
                block: AstcBlock::B5x4,
                channel: AstcChannel::UnormSrgb,
            },
            GPUTextureFormat::Astc_5x5_unorm => wgt::TextureFormat::Astc {
                block: AstcBlock::B5x5,
                channel: AstcChannel::Unorm,
            },
            GPUTextureFormat::Astc_5x5_unorm_srgb => wgt::TextureFormat::Astc {
                block: AstcBlock::B5x5,
                channel: AstcChannel::UnormSrgb,
            },
            GPUTextureFormat::Astc_6x5_unorm => wgt::TextureFormat::Astc {
                block: AstcBlock::B6x5,
                channel: AstcChannel::Unorm,
            },
            GPUTextureFormat::Astc_6x5_unorm_srgb => wgt::TextureFormat::Astc {
                block: AstcBlock::B6x5,
                channel: AstcChannel::UnormSrgb,
            },
            GPUTextureFormat::Astc_6x6_unorm => wgt::TextureFormat::Astc {
                block: AstcBlock::B6x6,
                channel: AstcChannel::Unorm,
            },
            GPUTextureFormat::Astc_6x6_unorm_srgb => wgt::TextureFormat::Astc {
                block: AstcBlock::B6x6,
                channel: AstcChannel::UnormSrgb,
            },
            GPUTextureFormat::Astc_8x5_unorm => wgt::TextureFormat::Astc {
                block: AstcBlock::B8x5,
                channel: AstcChannel::Unorm,
            },
            GPUTextureFormat::Astc_8x5_unorm_srgb => wgt::TextureFormat::Astc {
                block: AstcBlock::B8x5,
                channel: AstcChannel::UnormSrgb,
            },
            GPUTextureFormat::Astc_8x6_unorm => wgt::TextureFormat::Astc {
                block: AstcBlock::B8x6,
                channel: AstcChannel::Unorm,
            },
            GPUTextureFormat::Astc_8x6_unorm_srgb => wgt::TextureFormat::Astc {
                block: AstcBlock::B8x6,
                channel: AstcChannel::UnormSrgb,
            },
            GPUTextureFormat::Astc_8x8_unorm => wgt::TextureFormat::Astc {
                block: AstcBlock::B8x8,
                channel: AstcChannel::Unorm,
            },
            GPUTextureFormat::Astc_8x8_unorm_srgb => wgt::TextureFormat::Astc {
                block: AstcBlock::B8x8,
                channel: AstcChannel::UnormSrgb,
            },
            GPUTextureFormat::Astc_10x5_unorm => wgt::TextureFormat::Astc {
                block: AstcBlock::B10x5,
                channel: AstcChannel::Unorm,
            },
            GPUTextureFormat::Astc_10x5_unorm_srgb => wgt::TextureFormat::Astc {
                block: AstcBlock::B10x5,
                channel: AstcChannel::UnormSrgb,
            },
            GPUTextureFormat::Astc_10x6_unorm => wgt::TextureFormat::Astc {
                block: AstcBlock::B10x6,
                channel: AstcChannel::Unorm,
            },
            GPUTextureFormat::Astc_10x6_unorm_srgb => wgt::TextureFormat::Astc {
                block: AstcBlock::B10x6,
                channel: AstcChannel::UnormSrgb,
            },
            GPUTextureFormat::Astc_10x8_unorm => wgt::TextureFormat::Astc {
                block: AstcBlock::B10x8,
                channel: AstcChannel::Unorm,
            },
            GPUTextureFormat::Astc_10x8_unorm_srgb => wgt::TextureFormat::Astc {
                block: AstcBlock::B10x8,
                channel: AstcChannel::UnormSrgb,
            },
            GPUTextureFormat::Astc_10x10_unorm => wgt::TextureFormat::Astc {
                block: AstcBlock::B10x10,
                channel: AstcChannel::Unorm,
            },
            GPUTextureFormat::Astc_10x10_unorm_srgb => wgt::TextureFormat::Astc {
                block: AstcBlock::B10x10,
                channel: AstcChannel::UnormSrgb,
            },
            GPUTextureFormat::Astc_12x10_unorm => wgt::TextureFormat::Astc {
                block: AstcBlock::B12x10,
                channel: AstcChannel::Unorm,
            },
            GPUTextureFormat::Astc_12x10_unorm_srgb => wgt::TextureFormat::Astc {
                block: AstcBlock::B12x10,
                channel: AstcChannel::UnormSrgb,
            },
            GPUTextureFormat::Astc_12x12_unorm => wgt::TextureFormat::Astc {
                block: AstcBlock::B12x12,
                channel: AstcChannel::Unorm,
            },
            GPUTextureFormat::Astc_12x12_unorm_srgb => wgt::TextureFormat::Astc {
                block: AstcBlock::B12x12,
                channel: AstcChannel::UnormSrgb,
            },
        }
    }
}

impl TryConvert<wgt::Extent3d> for &GPUExtent3D {
    type Error = Error;

    fn try_convert(self) -> Result<wgt::Extent3d, Self::Error> {
        match *self {
            GPUExtent3D::GPUExtent3DDict(ref dict) => Ok(wgt::Extent3d {
                width: dict.width,
                height: dict.height,
                depth_or_array_layers: dict.depthOrArrayLayers,
            }),
            GPUExtent3D::RangeEnforcedUnsignedLongSequence(ref v) => {
                // https://gpuweb.github.io/gpuweb/#abstract-opdef-validate-gpuextent3d-shape
                if v.is_empty() || v.len() > 3 {
                    Err(Error::Type(
                        "GPUExtent3D size must be between 1 and 3 (inclusive)".to_string(),
                    ))
                } else {
                    Ok(wgt::Extent3d {
                        width: v[0],
                        height: v.get(1).copied().unwrap_or(1),
                        depth_or_array_layers: v.get(2).copied().unwrap_or(1),
                    })
                }
            },
        }
    }
}

impl Convert<wgt::TexelCopyBufferLayout> for &GPUImageDataLayout {
    fn convert(self) -> wgt::TexelCopyBufferLayout {
        wgt::TexelCopyBufferLayout {
            offset: self.offset as wgt::BufferAddress,
            bytes_per_row: self.bytesPerRow,
            rows_per_image: self.rowsPerImage,
        }
    }
}

impl Convert<wgt::VertexFormat> for GPUVertexFormat {
    fn convert(self) -> wgt::VertexFormat {
        match self {
            GPUVertexFormat::Uint8x2 => wgt::VertexFormat::Uint8x2,
            GPUVertexFormat::Uint8x4 => wgt::VertexFormat::Uint8x4,
            GPUVertexFormat::Sint8x2 => wgt::VertexFormat::Sint8x2,
            GPUVertexFormat::Sint8x4 => wgt::VertexFormat::Sint8x4,
            GPUVertexFormat::Unorm8x2 => wgt::VertexFormat::Unorm8x2,
            GPUVertexFormat::Unorm8x4 => wgt::VertexFormat::Unorm8x4,
            GPUVertexFormat::Snorm8x2 => wgt::VertexFormat::Unorm8x2,
            GPUVertexFormat::Snorm8x4 => wgt::VertexFormat::Unorm8x4,
            GPUVertexFormat::Uint16x2 => wgt::VertexFormat::Uint16x2,
            GPUVertexFormat::Uint16x4 => wgt::VertexFormat::Uint16x4,
            GPUVertexFormat::Sint16x2 => wgt::VertexFormat::Sint16x2,
            GPUVertexFormat::Sint16x4 => wgt::VertexFormat::Sint16x4,
            GPUVertexFormat::Unorm16x2 => wgt::VertexFormat::Unorm16x2,
            GPUVertexFormat::Unorm16x4 => wgt::VertexFormat::Unorm16x4,
            GPUVertexFormat::Snorm16x2 => wgt::VertexFormat::Snorm16x2,
            GPUVertexFormat::Snorm16x4 => wgt::VertexFormat::Snorm16x4,
            GPUVertexFormat::Float16x2 => wgt::VertexFormat::Float16x2,
            GPUVertexFormat::Float16x4 => wgt::VertexFormat::Float16x4,
            GPUVertexFormat::Float32 => wgt::VertexFormat::Float32,
            GPUVertexFormat::Float32x2 => wgt::VertexFormat::Float32x2,
            GPUVertexFormat::Float32x3 => wgt::VertexFormat::Float32x3,
            GPUVertexFormat::Float32x4 => wgt::VertexFormat::Float32x4,
            GPUVertexFormat::Uint32 => wgt::VertexFormat::Uint32,
            GPUVertexFormat::Uint32x2 => wgt::VertexFormat::Uint32x2,
            GPUVertexFormat::Uint32x3 => wgt::VertexFormat::Uint32x3,
            GPUVertexFormat::Uint32x4 => wgt::VertexFormat::Uint32x4,
            GPUVertexFormat::Sint32 => wgt::VertexFormat::Sint32,
            GPUVertexFormat::Sint32x2 => wgt::VertexFormat::Sint32x2,
            GPUVertexFormat::Sint32x3 => wgt::VertexFormat::Sint32x3,
            GPUVertexFormat::Sint32x4 => wgt::VertexFormat::Sint32x4,
        }
    }
}

impl Convert<wgt::PrimitiveState> for &GPUPrimitiveState {
    fn convert(self) -> wgt::PrimitiveState {
        wgt::PrimitiveState {
            topology: self.topology.convert(),
            strip_index_format: self
                .stripIndexFormat
                .map(|index_format| match index_format {
                    GPUIndexFormat::Uint16 => wgt::IndexFormat::Uint16,
                    GPUIndexFormat::Uint32 => wgt::IndexFormat::Uint32,
                }),
            front_face: match self.frontFace {
                GPUFrontFace::Ccw => wgt::FrontFace::Ccw,
                GPUFrontFace::Cw => wgt::FrontFace::Cw,
            },
            cull_mode: match self.cullMode {
                GPUCullMode::None => None,
                GPUCullMode::Front => Some(wgt::Face::Front),
                GPUCullMode::Back => Some(wgt::Face::Back),
            },
            unclipped_depth: self.clampDepth,
            ..Default::default()
        }
    }
}

impl Convert<wgt::PrimitiveTopology> for &GPUPrimitiveTopology {
    fn convert(self) -> wgt::PrimitiveTopology {
        match self {
            GPUPrimitiveTopology::Point_list => wgt::PrimitiveTopology::PointList,
            GPUPrimitiveTopology::Line_list => wgt::PrimitiveTopology::LineList,
            GPUPrimitiveTopology::Line_strip => wgt::PrimitiveTopology::LineStrip,
            GPUPrimitiveTopology::Triangle_list => wgt::PrimitiveTopology::TriangleList,
            GPUPrimitiveTopology::Triangle_strip => wgt::PrimitiveTopology::TriangleStrip,
        }
    }
}

impl Convert<wgt::AddressMode> for GPUAddressMode {
    fn convert(self) -> wgt::AddressMode {
        match self {
            GPUAddressMode::Clamp_to_edge => wgt::AddressMode::ClampToEdge,
            GPUAddressMode::Repeat => wgt::AddressMode::Repeat,
            GPUAddressMode::Mirror_repeat => wgt::AddressMode::MirrorRepeat,
        }
    }
}

impl Convert<wgt::FilterMode> for GPUFilterMode {
    fn convert(self) -> wgt::FilterMode {
        match self {
            GPUFilterMode::Nearest => wgt::FilterMode::Nearest,
            GPUFilterMode::Linear => wgt::FilterMode::Linear,
        }
    }
}

impl Convert<wgt::TextureViewDimension> for GPUTextureViewDimension {
    fn convert(self) -> wgt::TextureViewDimension {
        match self {
            GPUTextureViewDimension::_1d => wgt::TextureViewDimension::D1,
            GPUTextureViewDimension::_2d => wgt::TextureViewDimension::D2,
            GPUTextureViewDimension::_2d_array => wgt::TextureViewDimension::D2Array,
            GPUTextureViewDimension::Cube => wgt::TextureViewDimension::Cube,
            GPUTextureViewDimension::Cube_array => wgt::TextureViewDimension::CubeArray,
            GPUTextureViewDimension::_3d => wgt::TextureViewDimension::D3,
        }
    }
}

impl Convert<wgt::CompareFunction> for GPUCompareFunction {
    fn convert(self) -> wgt::CompareFunction {
        match self {
            GPUCompareFunction::Never => wgt::CompareFunction::Never,
            GPUCompareFunction::Less => wgt::CompareFunction::Less,
            GPUCompareFunction::Equal => wgt::CompareFunction::Equal,
            GPUCompareFunction::Less_equal => wgt::CompareFunction::LessEqual,
            GPUCompareFunction::Greater => wgt::CompareFunction::Greater,
            GPUCompareFunction::Not_equal => wgt::CompareFunction::NotEqual,
            GPUCompareFunction::Greater_equal => wgt::CompareFunction::GreaterEqual,
            GPUCompareFunction::Always => wgt::CompareFunction::Always,
        }
    }
}

impl Convert<wgt::BlendFactor> for &GPUBlendFactor {
    fn convert(self) -> wgt::BlendFactor {
        match self {
            GPUBlendFactor::Zero => wgt::BlendFactor::Zero,
            GPUBlendFactor::One => wgt::BlendFactor::One,
            GPUBlendFactor::Src => wgt::BlendFactor::Src,
            GPUBlendFactor::One_minus_src => wgt::BlendFactor::OneMinusSrc,
            GPUBlendFactor::Src_alpha => wgt::BlendFactor::SrcAlpha,
            GPUBlendFactor::One_minus_src_alpha => wgt::BlendFactor::OneMinusSrcAlpha,
            GPUBlendFactor::Dst => wgt::BlendFactor::Dst,
            GPUBlendFactor::One_minus_dst => wgt::BlendFactor::OneMinusDst,
            GPUBlendFactor::Dst_alpha => wgt::BlendFactor::DstAlpha,
            GPUBlendFactor::One_minus_dst_alpha => wgt::BlendFactor::OneMinusDstAlpha,
            GPUBlendFactor::Src_alpha_saturated => wgt::BlendFactor::SrcAlphaSaturated,
            GPUBlendFactor::Constant => wgt::BlendFactor::Constant,
            GPUBlendFactor::One_minus_constant => wgt::BlendFactor::OneMinusConstant,
        }
    }
}

impl Convert<wgt::BlendComponent> for &GPUBlendComponent {
    fn convert(self) -> wgt::BlendComponent {
        wgt::BlendComponent {
            src_factor: self.srcFactor.convert(),
            dst_factor: self.dstFactor.convert(),
            operation: match self.operation {
                GPUBlendOperation::Add => wgt::BlendOperation::Add,
                GPUBlendOperation::Subtract => wgt::BlendOperation::Subtract,
                GPUBlendOperation::Reverse_subtract => wgt::BlendOperation::ReverseSubtract,
                GPUBlendOperation::Min => wgt::BlendOperation::Min,
                GPUBlendOperation::Max => wgt::BlendOperation::Max,
            },
        }
    }
}

pub(crate) fn convert_load_op<T>(load: &GPULoadOp, clear: T) -> wgpu_com::LoadOp<T> {
    match load {
        GPULoadOp::Load => wgpu_com::LoadOp::Load,
        GPULoadOp::Clear => wgpu_com::LoadOp::Clear(clear),
    }
}

impl Convert<wgpu_com::StoreOp> for &GPUStoreOp {
    fn convert(self) -> wgpu_com::StoreOp {
        match self {
            GPUStoreOp::Store => wgpu_com::StoreOp::Store,
            GPUStoreOp::Discard => wgpu_com::StoreOp::Discard,
        }
    }
}

impl Convert<wgt::StencilOperation> for GPUStencilOperation {
    fn convert(self) -> wgt::StencilOperation {
        match self {
            GPUStencilOperation::Keep => wgt::StencilOperation::Keep,
            GPUStencilOperation::Zero => wgt::StencilOperation::Zero,
            GPUStencilOperation::Replace => wgt::StencilOperation::Replace,
            GPUStencilOperation::Invert => wgt::StencilOperation::Invert,
            GPUStencilOperation::Increment_clamp => wgt::StencilOperation::IncrementClamp,
            GPUStencilOperation::Decrement_clamp => wgt::StencilOperation::DecrementClamp,
            GPUStencilOperation::Increment_wrap => wgt::StencilOperation::IncrementWrap,
            GPUStencilOperation::Decrement_wrap => wgt::StencilOperation::DecrementWrap,
        }
    }
}

impl Convert<wgpu_com::TexelCopyBufferInfo> for &GPUImageCopyBuffer {
    fn convert(self) -> wgpu_com::TexelCopyBufferInfo {
        wgpu_com::TexelCopyBufferInfo {
            buffer: self.buffer.id().0,
            layout: self.parent.convert(),
        }
    }
}

impl TryConvert<wgt::Origin3d> for &GPUOrigin3D {
    type Error = Error;

    fn try_convert(self) -> Result<wgt::Origin3d, Self::Error> {
        match self {
            GPUOrigin3D::RangeEnforcedUnsignedLongSequence(v) => {
                // https://gpuweb.github.io/gpuweb/#abstract-opdef-validate-gpuorigin3d-shape
                if v.len() > 3 {
                    Err(Error::Type(
                        "sequence is too long for GPUOrigin3D".to_string(),
                    ))
                } else {
                    Ok(wgt::Origin3d {
                        x: v.first().copied().unwrap_or(0),
                        y: v.get(1).copied().unwrap_or(0),
                        z: v.get(2).copied().unwrap_or(0),
                    })
                }
            },
            GPUOrigin3D::GPUOrigin3DDict(d) => Ok(wgt::Origin3d {
                x: d.x,
                y: d.y,
                z: d.z,
            }),
        }
    }
}

impl TryConvert<wgpu_com::TexelCopyTextureInfo> for &GPUImageCopyTexture {
    type Error = Error;

    fn try_convert(self) -> Result<wgpu_com::TexelCopyTextureInfo, Self::Error> {
        Ok(wgpu_com::TexelCopyTextureInfo {
            texture: self.texture.id().0,
            mip_level: self.mipLevel,
            origin: self
                .origin
                .as_ref()
                .map(TryConvert::<wgt::Origin3d>::try_convert)
                .transpose()?
                .unwrap_or_default(),
            aspect: match self.aspect {
                GPUTextureAspect::All => wgt::TextureAspect::All,
                GPUTextureAspect::Stencil_only => wgt::TextureAspect::StencilOnly,
                GPUTextureAspect::Depth_only => wgt::TextureAspect::DepthOnly,
            },
        })
    }
}

impl<'a> Convert<Option<Cow<'a, str>>> for &GPUObjectDescriptorBase {
    fn convert(self) -> Option<Cow<'a, str>> {
        if self.label.is_empty() {
            None
        } else {
            Some(Cow::Owned(self.label.to_string()))
        }
    }
}

pub(crate) fn convert_bind_group_layout_entry(
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
            view_dimension: storage.viewDimension.convert(),
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
            view_dimension: texture.viewDimension.convert(),
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

pub(crate) fn convert_texture_descriptor(
    descriptor: &GPUTextureDescriptor,
    device: &GPUDevice,
) -> Fallible<(TextureDescriptor<'static>, wgt::Extent3d)> {
    let size = (&descriptor.size).try_convert()?;
    let desc = TextureDescriptor {
        label: (&descriptor.parent).convert(),
        size,
        mip_level_count: descriptor.mipLevelCount,
        sample_count: descriptor.sampleCount,
        dimension: descriptor.dimension.convert(),
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

impl TryConvert<wgt::Color> for &GPUColor {
    type Error = Error;

    fn try_convert(self) -> Result<wgt::Color, Self::Error> {
        match self {
            GPUColor::DoubleSequence(s) => {
                // https://gpuweb.github.io/gpuweb/#abstract-opdef-validate-gpucolor-shape
                if s.len() != 4 {
                    Err(Error::Type("GPUColor sequence must be len 4".to_string()))
                } else {
                    Ok(wgt::Color {
                        r: *s[0],
                        g: *s[1],
                        b: *s[2],
                        a: *s[3],
                    })
                }
            },
            GPUColor::GPUColorDict(d) => Ok(wgt::Color {
                r: *d.r,
                g: *d.g,
                b: *d.b,
                a: *d.a,
            }),
        }
    }
}

impl<'a> Convert<ProgrammableStageDescriptor<'a>> for &GPUProgrammableStage {
    fn convert(self) -> ProgrammableStageDescriptor<'a> {
        ProgrammableStageDescriptor {
            module: self.module.id().0,
            entry_point: self
                .entryPoint
                .as_ref()
                .map(|ep| Cow::Owned(ep.to_string())),
            constants: self
                .constants
                .as_ref()
                .map(|records| records.iter().map(|(k, v)| (k.0.clone(), **v)).collect())
                .unwrap_or_default(),
            zero_initialize_workgroup_memory: true,
        }
    }
}

impl<'a> Convert<BindGroupEntry<'a>> for &GPUBindGroupEntry {
    fn convert(self) -> BindGroupEntry<'a> {
        BindGroupEntry {
            binding: self.binding,
            resource: match self.resource {
                GPUBindingResource::GPUSampler(ref s) => BindingResource::Sampler(s.id().0),
                GPUBindingResource::GPUTextureView(ref t) => BindingResource::TextureView(t.id().0),
                GPUBindingResource::GPUBufferBinding(ref b) => {
                    BindingResource::Buffer(BufferBinding {
                        buffer: b.buffer.id().0,
                        offset: b.offset,
                        size: b.size.and_then(wgt::BufferSize::new),
                    })
                },
            },
        }
    }
}

impl Convert<wgt::TextureDimension> for GPUTextureDimension {
    fn convert(self) -> wgt::TextureDimension {
        match self {
            GPUTextureDimension::_1d => wgt::TextureDimension::D1,
            GPUTextureDimension::_2d => wgt::TextureDimension::D2,
            GPUTextureDimension::_3d => wgt::TextureDimension::D3,
        }
    }
}
