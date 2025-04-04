/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;
use std::num::NonZeroU64;

use wgpu_core::binding_model::{BindGroupEntry, BindingResource, BufferBinding};
use wgpu_core::command as wgpu_com;
use wgpu_core::pipeline::ProgrammableStageDescriptor;
use wgpu_core::resource::TextureDescriptor;
use wgpu_types::{self, AstcBlock, AstcChannel};

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

impl Convert<wgpu_types::TextureFormat> for GPUTextureFormat {
    fn convert(self) -> wgpu_types::TextureFormat {
        match self {
            GPUTextureFormat::R8unorm => wgpu_types::TextureFormat::R8Unorm,
            GPUTextureFormat::R8snorm => wgpu_types::TextureFormat::R8Snorm,
            GPUTextureFormat::R8uint => wgpu_types::TextureFormat::R8Uint,
            GPUTextureFormat::R8sint => wgpu_types::TextureFormat::R8Sint,
            GPUTextureFormat::R16uint => wgpu_types::TextureFormat::R16Uint,
            GPUTextureFormat::R16sint => wgpu_types::TextureFormat::R16Sint,
            GPUTextureFormat::R16float => wgpu_types::TextureFormat::R16Float,
            GPUTextureFormat::Rg8unorm => wgpu_types::TextureFormat::Rg8Unorm,
            GPUTextureFormat::Rg8snorm => wgpu_types::TextureFormat::Rg8Snorm,
            GPUTextureFormat::Rg8uint => wgpu_types::TextureFormat::Rg8Uint,
            GPUTextureFormat::Rg8sint => wgpu_types::TextureFormat::Rg8Sint,
            GPUTextureFormat::R32uint => wgpu_types::TextureFormat::R32Uint,
            GPUTextureFormat::R32sint => wgpu_types::TextureFormat::R32Sint,
            GPUTextureFormat::R32float => wgpu_types::TextureFormat::R32Float,
            GPUTextureFormat::Rg16uint => wgpu_types::TextureFormat::Rg16Uint,
            GPUTextureFormat::Rg16sint => wgpu_types::TextureFormat::Rg16Sint,
            GPUTextureFormat::Rg16float => wgpu_types::TextureFormat::Rg16Float,
            GPUTextureFormat::Rgba8unorm => wgpu_types::TextureFormat::Rgba8Unorm,
            GPUTextureFormat::Rgba8unorm_srgb => wgpu_types::TextureFormat::Rgba8UnormSrgb,
            GPUTextureFormat::Rgba8snorm => wgpu_types::TextureFormat::Rgba8Snorm,
            GPUTextureFormat::Rgba8uint => wgpu_types::TextureFormat::Rgba8Uint,
            GPUTextureFormat::Rgba8sint => wgpu_types::TextureFormat::Rgba8Sint,
            GPUTextureFormat::Bgra8unorm => wgpu_types::TextureFormat::Bgra8Unorm,
            GPUTextureFormat::Bgra8unorm_srgb => wgpu_types::TextureFormat::Bgra8UnormSrgb,
            GPUTextureFormat::Rgb10a2unorm => wgpu_types::TextureFormat::Rgb10a2Unorm,
            GPUTextureFormat::Rg32uint => wgpu_types::TextureFormat::Rg32Uint,
            GPUTextureFormat::Rg32sint => wgpu_types::TextureFormat::Rg32Sint,
            GPUTextureFormat::Rg32float => wgpu_types::TextureFormat::Rg32Float,
            GPUTextureFormat::Rgba16uint => wgpu_types::TextureFormat::Rgba16Uint,
            GPUTextureFormat::Rgba16sint => wgpu_types::TextureFormat::Rgba16Sint,
            GPUTextureFormat::Rgba16float => wgpu_types::TextureFormat::Rgba16Float,
            GPUTextureFormat::Rgba32uint => wgpu_types::TextureFormat::Rgba32Uint,
            GPUTextureFormat::Rgba32sint => wgpu_types::TextureFormat::Rgba32Sint,
            GPUTextureFormat::Rgba32float => wgpu_types::TextureFormat::Rgba32Float,
            GPUTextureFormat::Depth32float => wgpu_types::TextureFormat::Depth32Float,
            GPUTextureFormat::Depth24plus => wgpu_types::TextureFormat::Depth24Plus,
            GPUTextureFormat::Depth24plus_stencil8 => {
                wgpu_types::TextureFormat::Depth24PlusStencil8
            },
            GPUTextureFormat::Bc1_rgba_unorm => wgpu_types::TextureFormat::Bc1RgbaUnorm,
            GPUTextureFormat::Bc1_rgba_unorm_srgb => wgpu_types::TextureFormat::Bc1RgbaUnormSrgb,
            GPUTextureFormat::Bc2_rgba_unorm => wgpu_types::TextureFormat::Bc2RgbaUnorm,
            GPUTextureFormat::Bc2_rgba_unorm_srgb => wgpu_types::TextureFormat::Bc2RgbaUnormSrgb,
            GPUTextureFormat::Bc3_rgba_unorm => wgpu_types::TextureFormat::Bc3RgbaUnorm,
            GPUTextureFormat::Bc3_rgba_unorm_srgb => wgpu_types::TextureFormat::Bc3RgbaUnormSrgb,
            GPUTextureFormat::Bc4_r_unorm => wgpu_types::TextureFormat::Bc4RUnorm,
            GPUTextureFormat::Bc4_r_snorm => wgpu_types::TextureFormat::Bc4RSnorm,
            GPUTextureFormat::Bc5_rg_unorm => wgpu_types::TextureFormat::Bc5RgUnorm,
            GPUTextureFormat::Bc5_rg_snorm => wgpu_types::TextureFormat::Bc5RgSnorm,
            GPUTextureFormat::Bc6h_rgb_ufloat => wgpu_types::TextureFormat::Bc6hRgbUfloat,
            GPUTextureFormat::Bc7_rgba_unorm => wgpu_types::TextureFormat::Bc7RgbaUnorm,
            GPUTextureFormat::Bc7_rgba_unorm_srgb => wgpu_types::TextureFormat::Bc7RgbaUnormSrgb,
            GPUTextureFormat::Bc6h_rgb_float => wgpu_types::TextureFormat::Bc6hRgbFloat,
            GPUTextureFormat::Rgb9e5ufloat => wgpu_types::TextureFormat::Rgb9e5Ufloat,
            GPUTextureFormat::Rgb10a2uint => wgpu_types::TextureFormat::Rgb10a2Uint,
            GPUTextureFormat::Rg11b10ufloat => wgpu_types::TextureFormat::Rg11b10Ufloat,
            GPUTextureFormat::Stencil8 => wgpu_types::TextureFormat::Stencil8,
            GPUTextureFormat::Depth16unorm => wgpu_types::TextureFormat::Depth16Unorm,
            GPUTextureFormat::Depth32float_stencil8 => {
                wgpu_types::TextureFormat::Depth32FloatStencil8
            },
            GPUTextureFormat::Etc2_rgb8unorm => wgpu_types::TextureFormat::Etc2Rgb8Unorm,
            GPUTextureFormat::Etc2_rgb8unorm_srgb => wgpu_types::TextureFormat::Etc2Rgb8UnormSrgb,
            GPUTextureFormat::Etc2_rgb8a1unorm => wgpu_types::TextureFormat::Etc2Rgb8A1Unorm,
            GPUTextureFormat::Etc2_rgb8a1unorm_srgb => {
                wgpu_types::TextureFormat::Etc2Rgb8A1UnormSrgb
            },
            GPUTextureFormat::Etc2_rgba8unorm => wgpu_types::TextureFormat::Etc2Rgba8Unorm,
            GPUTextureFormat::Etc2_rgba8unorm_srgb => wgpu_types::TextureFormat::Etc2Rgba8UnormSrgb,
            GPUTextureFormat::Eac_r11unorm => wgpu_types::TextureFormat::EacR11Unorm,
            GPUTextureFormat::Eac_r11snorm => wgpu_types::TextureFormat::EacR11Snorm,
            GPUTextureFormat::Eac_rg11unorm => wgpu_types::TextureFormat::EacRg11Unorm,
            GPUTextureFormat::Eac_rg11snorm => wgpu_types::TextureFormat::EacRg11Snorm,
            GPUTextureFormat::Astc_4x4_unorm => wgpu_types::TextureFormat::Astc {
                block: AstcBlock::B4x4,
                channel: AstcChannel::Unorm,
            },
            GPUTextureFormat::Astc_4x4_unorm_srgb => wgpu_types::TextureFormat::Astc {
                block: AstcBlock::B4x4,
                channel: AstcChannel::UnormSrgb,
            },
            GPUTextureFormat::Astc_5x4_unorm => wgpu_types::TextureFormat::Astc {
                block: AstcBlock::B5x4,
                channel: AstcChannel::Unorm,
            },
            GPUTextureFormat::Astc_5x4_unorm_srgb => wgpu_types::TextureFormat::Astc {
                block: AstcBlock::B5x4,
                channel: AstcChannel::UnormSrgb,
            },
            GPUTextureFormat::Astc_5x5_unorm => wgpu_types::TextureFormat::Astc {
                block: AstcBlock::B5x5,
                channel: AstcChannel::Unorm,
            },
            GPUTextureFormat::Astc_5x5_unorm_srgb => wgpu_types::TextureFormat::Astc {
                block: AstcBlock::B5x5,
                channel: AstcChannel::UnormSrgb,
            },
            GPUTextureFormat::Astc_6x5_unorm => wgpu_types::TextureFormat::Astc {
                block: AstcBlock::B6x5,
                channel: AstcChannel::Unorm,
            },
            GPUTextureFormat::Astc_6x5_unorm_srgb => wgpu_types::TextureFormat::Astc {
                block: AstcBlock::B6x5,
                channel: AstcChannel::UnormSrgb,
            },
            GPUTextureFormat::Astc_6x6_unorm => wgpu_types::TextureFormat::Astc {
                block: AstcBlock::B6x6,
                channel: AstcChannel::Unorm,
            },
            GPUTextureFormat::Astc_6x6_unorm_srgb => wgpu_types::TextureFormat::Astc {
                block: AstcBlock::B6x6,
                channel: AstcChannel::UnormSrgb,
            },
            GPUTextureFormat::Astc_8x5_unorm => wgpu_types::TextureFormat::Astc {
                block: AstcBlock::B8x5,
                channel: AstcChannel::Unorm,
            },
            GPUTextureFormat::Astc_8x5_unorm_srgb => wgpu_types::TextureFormat::Astc {
                block: AstcBlock::B8x5,
                channel: AstcChannel::UnormSrgb,
            },
            GPUTextureFormat::Astc_8x6_unorm => wgpu_types::TextureFormat::Astc {
                block: AstcBlock::B8x6,
                channel: AstcChannel::Unorm,
            },
            GPUTextureFormat::Astc_8x6_unorm_srgb => wgpu_types::TextureFormat::Astc {
                block: AstcBlock::B8x6,
                channel: AstcChannel::UnormSrgb,
            },
            GPUTextureFormat::Astc_8x8_unorm => wgpu_types::TextureFormat::Astc {
                block: AstcBlock::B8x8,
                channel: AstcChannel::Unorm,
            },
            GPUTextureFormat::Astc_8x8_unorm_srgb => wgpu_types::TextureFormat::Astc {
                block: AstcBlock::B8x8,
                channel: AstcChannel::UnormSrgb,
            },
            GPUTextureFormat::Astc_10x5_unorm => wgpu_types::TextureFormat::Astc {
                block: AstcBlock::B10x5,
                channel: AstcChannel::Unorm,
            },
            GPUTextureFormat::Astc_10x5_unorm_srgb => wgpu_types::TextureFormat::Astc {
                block: AstcBlock::B10x5,
                channel: AstcChannel::UnormSrgb,
            },
            GPUTextureFormat::Astc_10x6_unorm => wgpu_types::TextureFormat::Astc {
                block: AstcBlock::B10x6,
                channel: AstcChannel::Unorm,
            },
            GPUTextureFormat::Astc_10x6_unorm_srgb => wgpu_types::TextureFormat::Astc {
                block: AstcBlock::B10x6,
                channel: AstcChannel::UnormSrgb,
            },
            GPUTextureFormat::Astc_10x8_unorm => wgpu_types::TextureFormat::Astc {
                block: AstcBlock::B10x8,
                channel: AstcChannel::Unorm,
            },
            GPUTextureFormat::Astc_10x8_unorm_srgb => wgpu_types::TextureFormat::Astc {
                block: AstcBlock::B10x8,
                channel: AstcChannel::UnormSrgb,
            },
            GPUTextureFormat::Astc_10x10_unorm => wgpu_types::TextureFormat::Astc {
                block: AstcBlock::B10x10,
                channel: AstcChannel::Unorm,
            },
            GPUTextureFormat::Astc_10x10_unorm_srgb => wgpu_types::TextureFormat::Astc {
                block: AstcBlock::B10x10,
                channel: AstcChannel::UnormSrgb,
            },
            GPUTextureFormat::Astc_12x10_unorm => wgpu_types::TextureFormat::Astc {
                block: AstcBlock::B12x10,
                channel: AstcChannel::Unorm,
            },
            GPUTextureFormat::Astc_12x10_unorm_srgb => wgpu_types::TextureFormat::Astc {
                block: AstcBlock::B12x10,
                channel: AstcChannel::UnormSrgb,
            },
            GPUTextureFormat::Astc_12x12_unorm => wgpu_types::TextureFormat::Astc {
                block: AstcBlock::B12x12,
                channel: AstcChannel::Unorm,
            },
            GPUTextureFormat::Astc_12x12_unorm_srgb => wgpu_types::TextureFormat::Astc {
                block: AstcBlock::B12x12,
                channel: AstcChannel::UnormSrgb,
            },
        }
    }
}

impl TryConvert<wgpu_types::Extent3d> for &GPUExtent3D {
    type Error = Error;

    fn try_convert(self) -> Result<wgpu_types::Extent3d, Self::Error> {
        match *self {
            GPUExtent3D::GPUExtent3DDict(ref dict) => Ok(wgpu_types::Extent3d {
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
                    Ok(wgpu_types::Extent3d {
                        width: v[0],
                        height: v.get(1).copied().unwrap_or(1),
                        depth_or_array_layers: v.get(2).copied().unwrap_or(1),
                    })
                }
            },
        }
    }
}

impl Convert<wgpu_types::TexelCopyBufferLayout> for &GPUImageDataLayout {
    fn convert(self) -> wgpu_types::TexelCopyBufferLayout {
        wgpu_types::TexelCopyBufferLayout {
            offset: self.offset as wgpu_types::BufferAddress,
            bytes_per_row: self.bytesPerRow,
            rows_per_image: self.rowsPerImage,
        }
    }
}

impl Convert<wgpu_types::VertexFormat> for GPUVertexFormat {
    fn convert(self) -> wgpu_types::VertexFormat {
        match self {
            GPUVertexFormat::Uint8x2 => wgpu_types::VertexFormat::Uint8x2,
            GPUVertexFormat::Uint8x4 => wgpu_types::VertexFormat::Uint8x4,
            GPUVertexFormat::Sint8x2 => wgpu_types::VertexFormat::Sint8x2,
            GPUVertexFormat::Sint8x4 => wgpu_types::VertexFormat::Sint8x4,
            GPUVertexFormat::Unorm8x2 => wgpu_types::VertexFormat::Unorm8x2,
            GPUVertexFormat::Unorm8x4 => wgpu_types::VertexFormat::Unorm8x4,
            GPUVertexFormat::Snorm8x2 => wgpu_types::VertexFormat::Unorm8x2,
            GPUVertexFormat::Snorm8x4 => wgpu_types::VertexFormat::Unorm8x4,
            GPUVertexFormat::Uint16x2 => wgpu_types::VertexFormat::Uint16x2,
            GPUVertexFormat::Uint16x4 => wgpu_types::VertexFormat::Uint16x4,
            GPUVertexFormat::Sint16x2 => wgpu_types::VertexFormat::Sint16x2,
            GPUVertexFormat::Sint16x4 => wgpu_types::VertexFormat::Sint16x4,
            GPUVertexFormat::Unorm16x2 => wgpu_types::VertexFormat::Unorm16x2,
            GPUVertexFormat::Unorm16x4 => wgpu_types::VertexFormat::Unorm16x4,
            GPUVertexFormat::Snorm16x2 => wgpu_types::VertexFormat::Snorm16x2,
            GPUVertexFormat::Snorm16x4 => wgpu_types::VertexFormat::Snorm16x4,
            GPUVertexFormat::Float16x2 => wgpu_types::VertexFormat::Float16x2,
            GPUVertexFormat::Float16x4 => wgpu_types::VertexFormat::Float16x4,
            GPUVertexFormat::Float32 => wgpu_types::VertexFormat::Float32,
            GPUVertexFormat::Float32x2 => wgpu_types::VertexFormat::Float32x2,
            GPUVertexFormat::Float32x3 => wgpu_types::VertexFormat::Float32x3,
            GPUVertexFormat::Float32x4 => wgpu_types::VertexFormat::Float32x4,
            GPUVertexFormat::Uint32 => wgpu_types::VertexFormat::Uint32,
            GPUVertexFormat::Uint32x2 => wgpu_types::VertexFormat::Uint32x2,
            GPUVertexFormat::Uint32x3 => wgpu_types::VertexFormat::Uint32x3,
            GPUVertexFormat::Uint32x4 => wgpu_types::VertexFormat::Uint32x4,
            GPUVertexFormat::Sint32 => wgpu_types::VertexFormat::Sint32,
            GPUVertexFormat::Sint32x2 => wgpu_types::VertexFormat::Sint32x2,
            GPUVertexFormat::Sint32x3 => wgpu_types::VertexFormat::Sint32x3,
            GPUVertexFormat::Sint32x4 => wgpu_types::VertexFormat::Sint32x4,
        }
    }
}

impl Convert<wgpu_types::PrimitiveState> for &GPUPrimitiveState {
    fn convert(self) -> wgpu_types::PrimitiveState {
        wgpu_types::PrimitiveState {
            topology: self.topology.convert(),
            strip_index_format: self
                .stripIndexFormat
                .map(|index_format| match index_format {
                    GPUIndexFormat::Uint16 => wgpu_types::IndexFormat::Uint16,
                    GPUIndexFormat::Uint32 => wgpu_types::IndexFormat::Uint32,
                }),
            front_face: match self.frontFace {
                GPUFrontFace::Ccw => wgpu_types::FrontFace::Ccw,
                GPUFrontFace::Cw => wgpu_types::FrontFace::Cw,
            },
            cull_mode: match self.cullMode {
                GPUCullMode::None => None,
                GPUCullMode::Front => Some(wgpu_types::Face::Front),
                GPUCullMode::Back => Some(wgpu_types::Face::Back),
            },
            unclipped_depth: self.clampDepth,
            ..Default::default()
        }
    }
}

impl Convert<wgpu_types::PrimitiveTopology> for &GPUPrimitiveTopology {
    fn convert(self) -> wgpu_types::PrimitiveTopology {
        match self {
            GPUPrimitiveTopology::Point_list => wgpu_types::PrimitiveTopology::PointList,
            GPUPrimitiveTopology::Line_list => wgpu_types::PrimitiveTopology::LineList,
            GPUPrimitiveTopology::Line_strip => wgpu_types::PrimitiveTopology::LineStrip,
            GPUPrimitiveTopology::Triangle_list => wgpu_types::PrimitiveTopology::TriangleList,
            GPUPrimitiveTopology::Triangle_strip => wgpu_types::PrimitiveTopology::TriangleStrip,
        }
    }
}

impl Convert<wgpu_types::AddressMode> for GPUAddressMode {
    fn convert(self) -> wgpu_types::AddressMode {
        match self {
            GPUAddressMode::Clamp_to_edge => wgpu_types::AddressMode::ClampToEdge,
            GPUAddressMode::Repeat => wgpu_types::AddressMode::Repeat,
            GPUAddressMode::Mirror_repeat => wgpu_types::AddressMode::MirrorRepeat,
        }
    }
}

impl Convert<wgpu_types::FilterMode> for GPUFilterMode {
    fn convert(self) -> wgpu_types::FilterMode {
        match self {
            GPUFilterMode::Nearest => wgpu_types::FilterMode::Nearest,
            GPUFilterMode::Linear => wgpu_types::FilterMode::Linear,
        }
    }
}

impl Convert<wgpu_types::TextureViewDimension> for GPUTextureViewDimension {
    fn convert(self) -> wgpu_types::TextureViewDimension {
        match self {
            GPUTextureViewDimension::_1d => wgpu_types::TextureViewDimension::D1,
            GPUTextureViewDimension::_2d => wgpu_types::TextureViewDimension::D2,
            GPUTextureViewDimension::_2d_array => wgpu_types::TextureViewDimension::D2Array,
            GPUTextureViewDimension::Cube => wgpu_types::TextureViewDimension::Cube,
            GPUTextureViewDimension::Cube_array => wgpu_types::TextureViewDimension::CubeArray,
            GPUTextureViewDimension::_3d => wgpu_types::TextureViewDimension::D3,
        }
    }
}

impl Convert<wgpu_types::CompareFunction> for GPUCompareFunction {
    fn convert(self) -> wgpu_types::CompareFunction {
        match self {
            GPUCompareFunction::Never => wgpu_types::CompareFunction::Never,
            GPUCompareFunction::Less => wgpu_types::CompareFunction::Less,
            GPUCompareFunction::Equal => wgpu_types::CompareFunction::Equal,
            GPUCompareFunction::Less_equal => wgpu_types::CompareFunction::LessEqual,
            GPUCompareFunction::Greater => wgpu_types::CompareFunction::Greater,
            GPUCompareFunction::Not_equal => wgpu_types::CompareFunction::NotEqual,
            GPUCompareFunction::Greater_equal => wgpu_types::CompareFunction::GreaterEqual,
            GPUCompareFunction::Always => wgpu_types::CompareFunction::Always,
        }
    }
}

impl Convert<wgpu_types::BlendFactor> for &GPUBlendFactor {
    fn convert(self) -> wgpu_types::BlendFactor {
        match self {
            GPUBlendFactor::Zero => wgpu_types::BlendFactor::Zero,
            GPUBlendFactor::One => wgpu_types::BlendFactor::One,
            GPUBlendFactor::Src => wgpu_types::BlendFactor::Src,
            GPUBlendFactor::One_minus_src => wgpu_types::BlendFactor::OneMinusSrc,
            GPUBlendFactor::Src_alpha => wgpu_types::BlendFactor::SrcAlpha,
            GPUBlendFactor::One_minus_src_alpha => wgpu_types::BlendFactor::OneMinusSrcAlpha,
            GPUBlendFactor::Dst => wgpu_types::BlendFactor::Dst,
            GPUBlendFactor::One_minus_dst => wgpu_types::BlendFactor::OneMinusDst,
            GPUBlendFactor::Dst_alpha => wgpu_types::BlendFactor::DstAlpha,
            GPUBlendFactor::One_minus_dst_alpha => wgpu_types::BlendFactor::OneMinusDstAlpha,
            GPUBlendFactor::Src_alpha_saturated => wgpu_types::BlendFactor::SrcAlphaSaturated,
            GPUBlendFactor::Constant => wgpu_types::BlendFactor::Constant,
            GPUBlendFactor::One_minus_constant => wgpu_types::BlendFactor::OneMinusConstant,
        }
    }
}

impl Convert<wgpu_types::BlendComponent> for &GPUBlendComponent {
    fn convert(self) -> wgpu_types::BlendComponent {
        wgpu_types::BlendComponent {
            src_factor: self.srcFactor.convert(),
            dst_factor: self.dstFactor.convert(),
            operation: match self.operation {
                GPUBlendOperation::Add => wgpu_types::BlendOperation::Add,
                GPUBlendOperation::Subtract => wgpu_types::BlendOperation::Subtract,
                GPUBlendOperation::Reverse_subtract => wgpu_types::BlendOperation::ReverseSubtract,
                GPUBlendOperation::Min => wgpu_types::BlendOperation::Min,
                GPUBlendOperation::Max => wgpu_types::BlendOperation::Max,
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

impl Convert<wgpu_types::StencilOperation> for GPUStencilOperation {
    fn convert(self) -> wgpu_types::StencilOperation {
        match self {
            GPUStencilOperation::Keep => wgpu_types::StencilOperation::Keep,
            GPUStencilOperation::Zero => wgpu_types::StencilOperation::Zero,
            GPUStencilOperation::Replace => wgpu_types::StencilOperation::Replace,
            GPUStencilOperation::Invert => wgpu_types::StencilOperation::Invert,
            GPUStencilOperation::Increment_clamp => wgpu_types::StencilOperation::IncrementClamp,
            GPUStencilOperation::Decrement_clamp => wgpu_types::StencilOperation::DecrementClamp,
            GPUStencilOperation::Increment_wrap => wgpu_types::StencilOperation::IncrementWrap,
            GPUStencilOperation::Decrement_wrap => wgpu_types::StencilOperation::DecrementWrap,
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

impl TryConvert<wgpu_types::Origin3d> for &GPUOrigin3D {
    type Error = Error;

    fn try_convert(self) -> Result<wgpu_types::Origin3d, Self::Error> {
        match self {
            GPUOrigin3D::RangeEnforcedUnsignedLongSequence(v) => {
                // https://gpuweb.github.io/gpuweb/#abstract-opdef-validate-gpuorigin3d-shape
                if v.len() > 3 {
                    Err(Error::Type(
                        "sequence is too long for GPUOrigin3D".to_string(),
                    ))
                } else {
                    Ok(wgpu_types::Origin3d {
                        x: v.first().copied().unwrap_or(0),
                        y: v.get(1).copied().unwrap_or(0),
                        z: v.get(2).copied().unwrap_or(0),
                    })
                }
            },
            GPUOrigin3D::GPUOrigin3DDict(d) => Ok(wgpu_types::Origin3d {
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
                .map(TryConvert::<wgpu_types::Origin3d>::try_convert)
                .transpose()?
                .unwrap_or_default(),
            aspect: match self.aspect {
                GPUTextureAspect::All => wgpu_types::TextureAspect::All,
                GPUTextureAspect::Stencil_only => wgpu_types::TextureAspect::StencilOnly,
                GPUTextureAspect::Depth_only => wgpu_types::TextureAspect::DepthOnly,
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
) -> Fallible<Result<wgpu_types::BindGroupLayoutEntry, webgpu_traits::Error>> {
    let number_of_provided_bindings = bgle.buffer.is_some() as u8 +
        bgle.sampler.is_some() as u8 +
        bgle.storageTexture.is_some() as u8 +
        bgle.texture.is_some() as u8;
    let ty = if let Some(buffer) = &bgle.buffer {
        Some(wgpu_types::BindingType::Buffer {
            ty: match buffer.type_ {
                GPUBufferBindingType::Uniform => wgpu_types::BufferBindingType::Uniform,
                GPUBufferBindingType::Storage => {
                    wgpu_types::BufferBindingType::Storage { read_only: false }
                },
                GPUBufferBindingType::Read_only_storage => {
                    wgpu_types::BufferBindingType::Storage { read_only: true }
                },
            },
            has_dynamic_offset: buffer.hasDynamicOffset,
            min_binding_size: NonZeroU64::new(buffer.minBindingSize),
        })
    } else if let Some(sampler) = &bgle.sampler {
        Some(wgpu_types::BindingType::Sampler(match sampler.type_ {
            GPUSamplerBindingType::Filtering => wgpu_types::SamplerBindingType::Filtering,
            GPUSamplerBindingType::Non_filtering => wgpu_types::SamplerBindingType::NonFiltering,
            GPUSamplerBindingType::Comparison => wgpu_types::SamplerBindingType::Comparison,
        }))
    } else if let Some(storage) = &bgle.storageTexture {
        Some(wgpu_types::BindingType::StorageTexture {
            access: match storage.access {
                GPUStorageTextureAccess::Write_only => wgpu_types::StorageTextureAccess::WriteOnly,
                GPUStorageTextureAccess::Read_only => wgpu_types::StorageTextureAccess::ReadOnly,
                GPUStorageTextureAccess::Read_write => wgpu_types::StorageTextureAccess::ReadWrite,
            },
            format: device.validate_texture_format_required_features(&storage.format)?,
            view_dimension: storage.viewDimension.convert(),
        })
    } else if let Some(texture) = &bgle.texture {
        Some(wgpu_types::BindingType::Texture {
            sample_type: match texture.sampleType {
                GPUTextureSampleType::Float => {
                    wgpu_types::TextureSampleType::Float { filterable: true }
                },
                GPUTextureSampleType::Unfilterable_float => {
                    wgpu_types::TextureSampleType::Float { filterable: false }
                },
                GPUTextureSampleType::Depth => wgpu_types::TextureSampleType::Depth,
                GPUTextureSampleType::Sint => wgpu_types::TextureSampleType::Sint,
                GPUTextureSampleType::Uint => wgpu_types::TextureSampleType::Uint,
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
    .ok_or(webgpu_traits::Error::Validation(
        "Exactly on entry type must be provided".to_string(),
    ));

    Ok(ty.map(|ty| wgpu_types::BindGroupLayoutEntry {
        binding: bgle.binding,
        visibility: wgpu_types::ShaderStages::from_bits_retain(bgle.visibility),
        ty,
        count: None,
    }))
}

pub(crate) fn convert_texture_descriptor(
    descriptor: &GPUTextureDescriptor,
    device: &GPUDevice,
) -> Fallible<(TextureDescriptor<'static>, wgpu_types::Extent3d)> {
    let size = (&descriptor.size).try_convert()?;
    let desc = TextureDescriptor {
        label: (&descriptor.parent).convert(),
        size,
        mip_level_count: descriptor.mipLevelCount,
        sample_count: descriptor.sampleCount,
        dimension: descriptor.dimension.convert(),
        format: device.validate_texture_format_required_features(&descriptor.format)?,
        usage: wgpu_types::TextureUsages::from_bits_retain(descriptor.usage),
        view_formats: descriptor
            .viewFormats
            .iter()
            .map(|tf| device.validate_texture_format_required_features(tf))
            .collect::<Fallible<_>>()?,
    };
    Ok((desc, size))
}

impl TryConvert<wgpu_types::Color> for &GPUColor {
    type Error = Error;

    fn try_convert(self) -> Result<wgpu_types::Color, Self::Error> {
        match self {
            GPUColor::DoubleSequence(s) => {
                // https://gpuweb.github.io/gpuweb/#abstract-opdef-validate-gpucolor-shape
                if s.len() != 4 {
                    Err(Error::Type("GPUColor sequence must be len 4".to_string()))
                } else {
                    Ok(wgpu_types::Color {
                        r: *s[0],
                        g: *s[1],
                        b: *s[2],
                        a: *s[3],
                    })
                }
            },
            GPUColor::GPUColorDict(d) => Ok(wgpu_types::Color {
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
                        size: b.size.and_then(wgpu_types::BufferSize::new),
                    })
                },
            },
        }
    }
}

impl Convert<wgpu_types::TextureDimension> for GPUTextureDimension {
    fn convert(self) -> wgpu_types::TextureDimension {
        match self {
            GPUTextureDimension::_1d => wgpu_types::TextureDimension::D1,
            GPUTextureDimension::_2d => wgpu_types::TextureDimension::D2,
            GPUTextureDimension::_3d => wgpu_types::TextureDimension::D3,
        }
    }
}
