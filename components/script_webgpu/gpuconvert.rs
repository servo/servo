/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;
use std::num::NonZeroU64;

use js::context::JSContext;
use script_bindings::codegen::GenericBindings::CanvasRenderingContext2DBinding::PredefinedColorSpace;
use script_bindings::codegen::GenericBindings::WebGPUBinding::{
    GPUAddressMode, GPUBindGroupEntry, GPUBindGroupLayoutEntry, GPUBindingResource,
    GPUBlendComponent, GPUBlendFactor, GPUBlendOperation, GPUBufferBindingType, GPUColor,
    GPUCompareFunction, GPUComputePassDescriptor, GPUComputePassTimestampWrites, GPUCullMode,
    GPUExtent3D, GPUFilterMode, GPUFrontFace, GPUIndexFormat, GPULoadOp, GPUMipmapFilterMode,
    GPUObjectDescriptorBase, GPUOrigin2D, GPUOrigin3D, GPUPrimitiveState, GPUPrimitiveTopology,
    GPUProgrammableStage, GPUQuerySetDescriptor, GPUQueryType, GPURenderPassTimestampWrites,
    GPUSamplerBindingType, GPUStencilOperation, GPUStorageTextureAccess, GPUStoreOp,
    GPUTexelCopyBufferInfo, GPUTexelCopyBufferLayout, GPUTexelCopyTextureInfo, GPUTextureAspect,
    GPUTextureDescriptor, GPUTextureDimension, GPUTextureFormat, GPUTextureSampleType,
    GPUTextureViewDimension, GPUVertexFormat,
};
use script_bindings::codegen::GenericUnionTypes::GPUTextureOrGPUTextureView;
use script_bindings::interfaces::PromiseHelpers;
use webgpu_traits::{
    AddressMode, AstcBlock, AstcChannel, BindGroupEntry, BindGroupLayoutEntry, BindingResource,
    BindingType, BlendComponent, BlendFactor, BlendOperation, BufferAddress, BufferBinding,
    BufferBindingType, Color, CompareFunction, ComputePassDescriptor, Extent3d, Face, FilterMode,
    FrontFace, IndexFormat, LoadOp, MipmapFilterMode, Origin2d, Origin3d, PassTimestampWrites,
    PredefinedColorSpace as WGPUPredefinedColorSpace, PrimitiveState, PrimitiveTopology,
    ProgrammableStageDescriptor, QuerySetDescriptor, QueryType, SamplerBindingType, ShaderStages,
    StencilOperation, StorageTextureAccess, StoreOp, TexelCopyBufferInfo, TexelCopyBufferLayout,
    TexelCopyTextureInfo, TextureAspect, TextureDescriptor, TextureDimension, TextureFormat,
    TextureSampleType, TextureUsages, TextureViewDimension, VertexFormat, WebGPUTextureView,
};

use crate::dom::bindings::error::{Error, Fallible};
use crate::traits::{Equivalence, WebGPUPromise};

/// A version of the `Into<T>` trait from the standard library that can be used
/// to convert between two types that are not defined in the script crate.
/// This is intended to be used on dict/enum types generated from WebIDL once
/// those types are moved out of the script crate.
/// Only for WebGPU.
pub trait WebGPUConvert<T> {
    fn convert(self) -> T;
}

/// A version of the `TryInto<T>` trait from the standard library that can be used
/// to convert between two types that are not defined in the script crate.
/// This is intended to be used on dict/enum types generated from WebIDL once
/// those types are moved out of the script crate.
/// Only for WebGPU.
pub trait WebGPUTryConvert<T> {
    type Error;

    fn try_convert(self) -> Result<T, Self::Error>;
}

impl WebGPUConvert<TextureFormat> for GPUTextureFormat {
    fn convert(self) -> TextureFormat {
        match self {
            // 8-bit formats
            GPUTextureFormat::R8unorm => TextureFormat::R8Unorm,
            GPUTextureFormat::R8snorm => TextureFormat::R8Snorm,
            GPUTextureFormat::R8uint => TextureFormat::R8Uint,
            GPUTextureFormat::R8sint => TextureFormat::R8Sint,
            // 16-bit formats
            GPUTextureFormat::R16unorm => TextureFormat::R16Unorm,
            GPUTextureFormat::R16snorm => TextureFormat::R16Snorm,
            GPUTextureFormat::R16uint => TextureFormat::R16Uint,
            GPUTextureFormat::R16sint => TextureFormat::R16Sint,
            GPUTextureFormat::R16float => TextureFormat::R16Float,
            GPUTextureFormat::Rg8unorm => TextureFormat::Rg8Unorm,
            GPUTextureFormat::Rg8snorm => TextureFormat::Rg8Snorm,
            GPUTextureFormat::Rg8uint => TextureFormat::Rg8Uint,
            GPUTextureFormat::Rg8sint => TextureFormat::Rg8Sint,
            // 32-bit formats
            GPUTextureFormat::R32uint => TextureFormat::R32Uint,
            GPUTextureFormat::R32sint => TextureFormat::R32Sint,
            GPUTextureFormat::R32float => TextureFormat::R32Float,
            GPUTextureFormat::Rg16unorm => TextureFormat::Rg16Unorm,
            GPUTextureFormat::Rg16snorm => TextureFormat::Rg16Snorm,
            GPUTextureFormat::Rg16uint => TextureFormat::Rg16Uint,
            GPUTextureFormat::Rg16sint => TextureFormat::Rg16Sint,
            GPUTextureFormat::Rg16float => TextureFormat::Rg16Float,
            GPUTextureFormat::Rgba8unorm => TextureFormat::Rgba8Unorm,
            GPUTextureFormat::Rgba8unorm_srgb => TextureFormat::Rgba8UnormSrgb,
            GPUTextureFormat::Rgba8snorm => TextureFormat::Rgba8Snorm,
            GPUTextureFormat::Rgba8uint => TextureFormat::Rgba8Uint,
            GPUTextureFormat::Rgba8sint => TextureFormat::Rgba8Sint,
            GPUTextureFormat::Bgra8unorm => TextureFormat::Bgra8Unorm,
            GPUTextureFormat::Bgra8unorm_srgb => TextureFormat::Bgra8UnormSrgb,
            // Packed 32-bit formats
            GPUTextureFormat::Rgb9e5ufloat => TextureFormat::Rgb9e5Ufloat,
            GPUTextureFormat::Rgb10a2uint => TextureFormat::Rgb10a2Uint,
            GPUTextureFormat::Rgb10a2unorm => TextureFormat::Rgb10a2Unorm,
            GPUTextureFormat::Rg11b10ufloat => TextureFormat::Rg11b10Ufloat,
            // 64-bit formats
            GPUTextureFormat::Rg32uint => TextureFormat::Rg32Uint,
            GPUTextureFormat::Rg32sint => TextureFormat::Rg32Sint,
            GPUTextureFormat::Rg32float => TextureFormat::Rg32Float,
            GPUTextureFormat::Rgba16unorm => TextureFormat::Rgba16Unorm,
            GPUTextureFormat::Rgba16snorm => TextureFormat::Rgba16Snorm,
            GPUTextureFormat::Rgba16uint => TextureFormat::Rgba16Uint,
            GPUTextureFormat::Rgba16sint => TextureFormat::Rgba16Sint,
            GPUTextureFormat::Rgba16float => TextureFormat::Rgba16Float,
            // 96-bit formats
            GPUTextureFormat::Rgba32uint => TextureFormat::Rgba32Uint,
            GPUTextureFormat::Rgba32sint => TextureFormat::Rgba32Sint,
            GPUTextureFormat::Rgba32float => TextureFormat::Rgba32Float,
            // Depth/stencil formats
            GPUTextureFormat::Stencil8 => TextureFormat::Stencil8,
            GPUTextureFormat::Depth16unorm => TextureFormat::Depth16Unorm,
            GPUTextureFormat::Depth24plus => TextureFormat::Depth24Plus,
            GPUTextureFormat::Depth24plus_stencil8 => TextureFormat::Depth24PlusStencil8,
            GPUTextureFormat::Depth32float => TextureFormat::Depth32Float,
            // "depth32float-stencil8" feature
            GPUTextureFormat::Depth32float_stencil8 => TextureFormat::Depth32FloatStencil8,
            // BC compressed formats usable if "texture-compression-bc" is both
            // supported by the device/user agent and enabled in requestDevice.
            GPUTextureFormat::Bc1_rgba_unorm => TextureFormat::Bc1RgbaUnorm,
            GPUTextureFormat::Bc1_rgba_unorm_srgb => TextureFormat::Bc1RgbaUnormSrgb,
            GPUTextureFormat::Bc2_rgba_unorm => TextureFormat::Bc2RgbaUnorm,
            GPUTextureFormat::Bc2_rgba_unorm_srgb => TextureFormat::Bc2RgbaUnormSrgb,
            GPUTextureFormat::Bc3_rgba_unorm => TextureFormat::Bc3RgbaUnorm,
            GPUTextureFormat::Bc3_rgba_unorm_srgb => TextureFormat::Bc3RgbaUnormSrgb,
            GPUTextureFormat::Bc4_r_unorm => TextureFormat::Bc4RUnorm,
            GPUTextureFormat::Bc4_r_snorm => TextureFormat::Bc4RSnorm,
            GPUTextureFormat::Bc5_rg_unorm => TextureFormat::Bc5RgUnorm,
            GPUTextureFormat::Bc5_rg_snorm => TextureFormat::Bc5RgSnorm,
            GPUTextureFormat::Bc6h_rgb_ufloat => TextureFormat::Bc6hRgbUfloat,
            GPUTextureFormat::Bc6h_rgb_float => TextureFormat::Bc6hRgbFloat,
            GPUTextureFormat::Bc7_rgba_unorm => TextureFormat::Bc7RgbaUnorm,
            GPUTextureFormat::Bc7_rgba_unorm_srgb => TextureFormat::Bc7RgbaUnormSrgb,
            // ETC2 compressed formats usable if "texture-compression-etc2" is both
            // supported by the device/user agent and enabled in requestDevice.
            GPUTextureFormat::Etc2_rgb8unorm => TextureFormat::Etc2Rgb8Unorm,
            GPUTextureFormat::Etc2_rgb8unorm_srgb => TextureFormat::Etc2Rgb8UnormSrgb,
            GPUTextureFormat::Etc2_rgb8a1unorm => TextureFormat::Etc2Rgb8A1Unorm,
            GPUTextureFormat::Etc2_rgb8a1unorm_srgb => TextureFormat::Etc2Rgb8A1UnormSrgb,
            GPUTextureFormat::Etc2_rgba8unorm => TextureFormat::Etc2Rgba8Unorm,
            GPUTextureFormat::Etc2_rgba8unorm_srgb => TextureFormat::Etc2Rgba8UnormSrgb,
            GPUTextureFormat::Eac_r11unorm => TextureFormat::EacR11Unorm,
            GPUTextureFormat::Eac_r11snorm => TextureFormat::EacR11Snorm,
            GPUTextureFormat::Eac_rg11unorm => TextureFormat::EacRg11Unorm,
            GPUTextureFormat::Eac_rg11snorm => TextureFormat::EacRg11Snorm,
            // ASTC compressed formats usable if "texture-compression-astc" is both
            // supported by the device/user agent and enabled in requestDevice.
            GPUTextureFormat::Astc_4x4_unorm => TextureFormat::Astc {
                block: AstcBlock::B4x4,
                channel: AstcChannel::Unorm,
            },
            GPUTextureFormat::Astc_4x4_unorm_srgb => TextureFormat::Astc {
                block: AstcBlock::B4x4,
                channel: AstcChannel::UnormSrgb,
            },
            GPUTextureFormat::Astc_5x4_unorm => TextureFormat::Astc {
                block: AstcBlock::B5x4,
                channel: AstcChannel::Unorm,
            },
            GPUTextureFormat::Astc_5x4_unorm_srgb => TextureFormat::Astc {
                block: AstcBlock::B5x4,
                channel: AstcChannel::UnormSrgb,
            },
            GPUTextureFormat::Astc_5x5_unorm => TextureFormat::Astc {
                block: AstcBlock::B5x5,
                channel: AstcChannel::Unorm,
            },
            GPUTextureFormat::Astc_5x5_unorm_srgb => TextureFormat::Astc {
                block: AstcBlock::B5x5,
                channel: AstcChannel::UnormSrgb,
            },
            GPUTextureFormat::Astc_6x5_unorm => TextureFormat::Astc {
                block: AstcBlock::B6x5,
                channel: AstcChannel::Unorm,
            },
            GPUTextureFormat::Astc_6x5_unorm_srgb => TextureFormat::Astc {
                block: AstcBlock::B6x5,
                channel: AstcChannel::UnormSrgb,
            },
            GPUTextureFormat::Astc_6x6_unorm => TextureFormat::Astc {
                block: AstcBlock::B6x6,
                channel: AstcChannel::Unorm,
            },
            GPUTextureFormat::Astc_6x6_unorm_srgb => TextureFormat::Astc {
                block: AstcBlock::B6x6,
                channel: AstcChannel::UnormSrgb,
            },
            GPUTextureFormat::Astc_8x5_unorm => TextureFormat::Astc {
                block: AstcBlock::B8x5,
                channel: AstcChannel::Unorm,
            },
            GPUTextureFormat::Astc_8x5_unorm_srgb => TextureFormat::Astc {
                block: AstcBlock::B8x5,
                channel: AstcChannel::UnormSrgb,
            },
            GPUTextureFormat::Astc_8x6_unorm => TextureFormat::Astc {
                block: AstcBlock::B8x6,
                channel: AstcChannel::Unorm,
            },
            GPUTextureFormat::Astc_8x6_unorm_srgb => TextureFormat::Astc {
                block: AstcBlock::B8x6,
                channel: AstcChannel::UnormSrgb,
            },
            GPUTextureFormat::Astc_8x8_unorm => TextureFormat::Astc {
                block: AstcBlock::B8x8,
                channel: AstcChannel::Unorm,
            },
            GPUTextureFormat::Astc_8x8_unorm_srgb => TextureFormat::Astc {
                block: AstcBlock::B8x8,
                channel: AstcChannel::UnormSrgb,
            },
            GPUTextureFormat::Astc_10x5_unorm => TextureFormat::Astc {
                block: AstcBlock::B10x5,
                channel: AstcChannel::Unorm,
            },
            GPUTextureFormat::Astc_10x5_unorm_srgb => TextureFormat::Astc {
                block: AstcBlock::B10x5,
                channel: AstcChannel::UnormSrgb,
            },
            GPUTextureFormat::Astc_10x6_unorm => TextureFormat::Astc {
                block: AstcBlock::B10x6,
                channel: AstcChannel::Unorm,
            },
            GPUTextureFormat::Astc_10x6_unorm_srgb => TextureFormat::Astc {
                block: AstcBlock::B10x6,
                channel: AstcChannel::UnormSrgb,
            },
            GPUTextureFormat::Astc_10x8_unorm => TextureFormat::Astc {
                block: AstcBlock::B10x8,
                channel: AstcChannel::Unorm,
            },
            GPUTextureFormat::Astc_10x8_unorm_srgb => TextureFormat::Astc {
                block: AstcBlock::B10x8,
                channel: AstcChannel::UnormSrgb,
            },
            GPUTextureFormat::Astc_10x10_unorm => TextureFormat::Astc {
                block: AstcBlock::B10x10,
                channel: AstcChannel::Unorm,
            },
            GPUTextureFormat::Astc_10x10_unorm_srgb => TextureFormat::Astc {
                block: AstcBlock::B10x10,
                channel: AstcChannel::UnormSrgb,
            },
            GPUTextureFormat::Astc_12x10_unorm => TextureFormat::Astc {
                block: AstcBlock::B12x10,
                channel: AstcChannel::Unorm,
            },
            GPUTextureFormat::Astc_12x10_unorm_srgb => TextureFormat::Astc {
                block: AstcBlock::B12x10,
                channel: AstcChannel::UnormSrgb,
            },
            GPUTextureFormat::Astc_12x12_unorm => TextureFormat::Astc {
                block: AstcBlock::B12x12,
                channel: AstcChannel::Unorm,
            },
            GPUTextureFormat::Astc_12x12_unorm_srgb => TextureFormat::Astc {
                block: AstcBlock::B12x12,
                channel: AstcChannel::UnormSrgb,
            },
        }
    }
}

impl WebGPUTryConvert<Extent3d> for &GPUExtent3D {
    type Error = Error;

    fn try_convert(self) -> Result<Extent3d, Self::Error> {
        match *self {
            GPUExtent3D::GPUExtent3DDict(ref dict) => Ok(Extent3d {
                width: dict.width,
                height: dict.height,
                depth_or_array_layers: dict.depthOrArrayLayers,
            }),
            GPUExtent3D::RangeEnforcedUnsignedLongSequence(ref v) => {
                // https://gpuweb.github.io/gpuweb/#abstract-opdef-validate-gpuextent3d-shape
                if v.is_empty() || v.len() > 3 {
                    Err(Error::Type(
                        c"GPUExtent3D size must be between 1 and 3 (inclusive)".to_owned(),
                    ))
                } else {
                    Ok(Extent3d {
                        width: v[0],
                        height: v.get(1).copied().unwrap_or(1),
                        depth_or_array_layers: v.get(2).copied().unwrap_or(1),
                    })
                }
            },
        }
    }
}

impl WebGPUConvert<TexelCopyBufferLayout> for &GPUTexelCopyBufferLayout {
    fn convert(self) -> TexelCopyBufferLayout {
        TexelCopyBufferLayout {
            offset: self.offset as BufferAddress,
            bytes_per_row: self.bytesPerRow,
            rows_per_image: self.rowsPerImage,
        }
    }
}

impl WebGPUConvert<VertexFormat> for GPUVertexFormat {
    fn convert(self) -> VertexFormat {
        match self {
            GPUVertexFormat::Uint8 => VertexFormat::Uint8,
            GPUVertexFormat::Uint8x2 => VertexFormat::Uint8x2,
            GPUVertexFormat::Uint8x4 => VertexFormat::Uint8x4,
            GPUVertexFormat::Sint8 => VertexFormat::Sint8,
            GPUVertexFormat::Sint8x2 => VertexFormat::Sint8x2,
            GPUVertexFormat::Sint8x4 => VertexFormat::Sint8x4,
            GPUVertexFormat::Unorm8 => VertexFormat::Unorm8,
            GPUVertexFormat::Unorm8x2 => VertexFormat::Unorm8x2,
            GPUVertexFormat::Unorm8x4 => VertexFormat::Unorm8x4,
            GPUVertexFormat::Snorm8 => VertexFormat::Snorm8,
            GPUVertexFormat::Snorm8x2 => VertexFormat::Snorm8x2,
            GPUVertexFormat::Snorm8x4 => VertexFormat::Snorm8x4,
            GPUVertexFormat::Uint16 => VertexFormat::Uint16,
            GPUVertexFormat::Uint16x2 => VertexFormat::Uint16x2,
            GPUVertexFormat::Uint16x4 => VertexFormat::Uint16x4,
            GPUVertexFormat::Sint16 => VertexFormat::Sint16,
            GPUVertexFormat::Sint16x2 => VertexFormat::Sint16x2,
            GPUVertexFormat::Sint16x4 => VertexFormat::Sint16x4,
            GPUVertexFormat::Unorm16 => VertexFormat::Unorm16,
            GPUVertexFormat::Unorm16x2 => VertexFormat::Unorm16x2,
            GPUVertexFormat::Unorm16x4 => VertexFormat::Unorm16x4,
            GPUVertexFormat::Snorm16 => VertexFormat::Snorm16,
            GPUVertexFormat::Snorm16x2 => VertexFormat::Snorm16x2,
            GPUVertexFormat::Snorm16x4 => VertexFormat::Snorm16x4,
            GPUVertexFormat::Float16 => VertexFormat::Float16,
            GPUVertexFormat::Float16x2 => VertexFormat::Float16x2,
            GPUVertexFormat::Float16x4 => VertexFormat::Float16x4,
            GPUVertexFormat::Float32 => VertexFormat::Float32,
            GPUVertexFormat::Float32x2 => VertexFormat::Float32x2,
            GPUVertexFormat::Float32x3 => VertexFormat::Float32x3,
            GPUVertexFormat::Float32x4 => VertexFormat::Float32x4,
            GPUVertexFormat::Uint32 => VertexFormat::Uint32,
            GPUVertexFormat::Uint32x2 => VertexFormat::Uint32x2,
            GPUVertexFormat::Uint32x3 => VertexFormat::Uint32x3,
            GPUVertexFormat::Uint32x4 => VertexFormat::Uint32x4,
            GPUVertexFormat::Sint32 => VertexFormat::Sint32,
            GPUVertexFormat::Sint32x2 => VertexFormat::Sint32x2,
            GPUVertexFormat::Sint32x3 => VertexFormat::Sint32x3,
            GPUVertexFormat::Sint32x4 => VertexFormat::Sint32x4,
            GPUVertexFormat::Unorm10_10_10_2 => VertexFormat::Unorm10_10_10_2,
            GPUVertexFormat::Unorm8x4_bgra => VertexFormat::Unorm8x4Bgra,
        }
    }
}

impl WebGPUConvert<PrimitiveState> for &GPUPrimitiveState {
    fn convert(self) -> PrimitiveState {
        PrimitiveState {
            topology: self.topology.convert(),
            strip_index_format: self
                .stripIndexFormat
                .map(|index_format| match index_format {
                    GPUIndexFormat::Uint16 => IndexFormat::Uint16,
                    GPUIndexFormat::Uint32 => IndexFormat::Uint32,
                }),
            front_face: match self.frontFace {
                GPUFrontFace::Ccw => FrontFace::Ccw,
                GPUFrontFace::Cw => FrontFace::Cw,
            },
            cull_mode: match self.cullMode {
                GPUCullMode::None => None,
                GPUCullMode::Front => Some(Face::Front),
                GPUCullMode::Back => Some(Face::Back),
            },
            unclipped_depth: self.clampDepth,
            ..Default::default()
        }
    }
}

impl WebGPUConvert<PrimitiveTopology> for &GPUPrimitiveTopology {
    fn convert(self) -> PrimitiveTopology {
        match self {
            GPUPrimitiveTopology::Point_list => PrimitiveTopology::PointList,
            GPUPrimitiveTopology::Line_list => PrimitiveTopology::LineList,
            GPUPrimitiveTopology::Line_strip => PrimitiveTopology::LineStrip,
            GPUPrimitiveTopology::Triangle_list => PrimitiveTopology::TriangleList,
            GPUPrimitiveTopology::Triangle_strip => PrimitiveTopology::TriangleStrip,
        }
    }
}

impl WebGPUConvert<AddressMode> for GPUAddressMode {
    fn convert(self) -> AddressMode {
        match self {
            GPUAddressMode::Clamp_to_edge => AddressMode::ClampToEdge,
            GPUAddressMode::Repeat => AddressMode::Repeat,
            GPUAddressMode::Mirror_repeat => AddressMode::MirrorRepeat,
        }
    }
}

impl WebGPUConvert<FilterMode> for GPUFilterMode {
    fn convert(self) -> FilterMode {
        match self {
            GPUFilterMode::Nearest => FilterMode::Nearest,
            GPUFilterMode::Linear => FilterMode::Linear,
        }
    }
}

impl WebGPUConvert<MipmapFilterMode> for GPUMipmapFilterMode {
    fn convert(self) -> MipmapFilterMode {
        match self {
            GPUMipmapFilterMode::Nearest => MipmapFilterMode::Nearest,
            GPUMipmapFilterMode::Linear => MipmapFilterMode::Linear,
        }
    }
}

impl WebGPUConvert<TextureViewDimension> for GPUTextureViewDimension {
    fn convert(self) -> TextureViewDimension {
        match self {
            GPUTextureViewDimension::_1d => TextureViewDimension::D1,
            GPUTextureViewDimension::_2d => TextureViewDimension::D2,
            GPUTextureViewDimension::_2d_array => TextureViewDimension::D2Array,
            GPUTextureViewDimension::Cube => TextureViewDimension::Cube,
            GPUTextureViewDimension::Cube_array => TextureViewDimension::CubeArray,
            GPUTextureViewDimension::_3d => TextureViewDimension::D3,
        }
    }
}

impl WebGPUConvert<CompareFunction> for GPUCompareFunction {
    fn convert(self) -> CompareFunction {
        match self {
            GPUCompareFunction::Never => CompareFunction::Never,
            GPUCompareFunction::Less => CompareFunction::Less,
            GPUCompareFunction::Equal => CompareFunction::Equal,
            GPUCompareFunction::Less_equal => CompareFunction::LessEqual,
            GPUCompareFunction::Greater => CompareFunction::Greater,
            GPUCompareFunction::Not_equal => CompareFunction::NotEqual,
            GPUCompareFunction::Greater_equal => CompareFunction::GreaterEqual,
            GPUCompareFunction::Always => CompareFunction::Always,
        }
    }
}

impl WebGPUConvert<BlendFactor> for &GPUBlendFactor {
    fn convert(self) -> BlendFactor {
        match self {
            GPUBlendFactor::Zero => BlendFactor::Zero,
            GPUBlendFactor::One => BlendFactor::One,
            GPUBlendFactor::Src => BlendFactor::Src,
            GPUBlendFactor::One_minus_src => BlendFactor::OneMinusSrc,
            GPUBlendFactor::Src_alpha => BlendFactor::SrcAlpha,
            GPUBlendFactor::One_minus_src_alpha => BlendFactor::OneMinusSrcAlpha,
            GPUBlendFactor::Dst => BlendFactor::Dst,
            GPUBlendFactor::One_minus_dst => BlendFactor::OneMinusDst,
            GPUBlendFactor::Dst_alpha => BlendFactor::DstAlpha,
            GPUBlendFactor::One_minus_dst_alpha => BlendFactor::OneMinusDstAlpha,
            GPUBlendFactor::Src_alpha_saturated => BlendFactor::SrcAlphaSaturated,
            GPUBlendFactor::Constant => BlendFactor::Constant,
            GPUBlendFactor::One_minus_constant => BlendFactor::OneMinusConstant,
            GPUBlendFactor::Src1 => BlendFactor::Src1,
            GPUBlendFactor::One_minus_src1 => BlendFactor::OneMinusSrc1,
            GPUBlendFactor::Src1_alpha => BlendFactor::Src1Alpha,
            GPUBlendFactor::One_minus_src1_alpha => BlendFactor::OneMinusSrc1Alpha,
        }
    }
}

impl WebGPUConvert<BlendComponent> for &GPUBlendComponent {
    fn convert(self) -> BlendComponent {
        BlendComponent {
            src_factor: self.srcFactor.convert(),
            dst_factor: self.dstFactor.convert(),
            operation: match self.operation {
                GPUBlendOperation::Add => BlendOperation::Add,
                GPUBlendOperation::Subtract => BlendOperation::Subtract,
                GPUBlendOperation::Reverse_subtract => BlendOperation::ReverseSubtract,
                GPUBlendOperation::Min => BlendOperation::Min,
                GPUBlendOperation::Max => BlendOperation::Max,
            },
        }
    }
}

pub fn convert_load_op<T>(load: &GPULoadOp, clear: T) -> LoadOp<T> {
    match load {
        GPULoadOp::Load => LoadOp::Load,
        GPULoadOp::Clear => LoadOp::Clear(clear),
    }
}

impl WebGPUConvert<StoreOp> for &GPUStoreOp {
    fn convert(self) -> StoreOp {
        match self {
            GPUStoreOp::Store => StoreOp::Store,
            GPUStoreOp::Discard => StoreOp::Discard,
        }
    }
}

impl WebGPUConvert<StencilOperation> for GPUStencilOperation {
    fn convert(self) -> StencilOperation {
        match self {
            GPUStencilOperation::Keep => StencilOperation::Keep,
            GPUStencilOperation::Zero => StencilOperation::Zero,
            GPUStencilOperation::Replace => StencilOperation::Replace,
            GPUStencilOperation::Invert => StencilOperation::Invert,
            GPUStencilOperation::Increment_clamp => StencilOperation::IncrementClamp,
            GPUStencilOperation::Decrement_clamp => StencilOperation::DecrementClamp,
            GPUStencilOperation::Increment_wrap => StencilOperation::IncrementWrap,
            GPUStencilOperation::Decrement_wrap => StencilOperation::DecrementWrap,
        }
    }
}

impl<D> WebGPUConvert<TexelCopyBufferInfo> for &GPUTexelCopyBufferInfo<D>
where
    D: Equivalence,
    <D::Promise as PromiseHelpers<D>>::StackRoot: WebGPUPromise<D>,
{
    fn convert(self) -> TexelCopyBufferInfo {
        TexelCopyBufferInfo {
            buffer: self.buffer.id().0,
            layout: self.parent.convert(),
        }
    }
}

impl WebGPUTryConvert<Origin3d> for &GPUOrigin3D {
    type Error = Error;

    fn try_convert(self) -> Result<Origin3d, Self::Error> {
        match self {
            GPUOrigin3D::RangeEnforcedUnsignedLongSequence(v) => {
                // https://gpuweb.github.io/gpuweb/#abstract-opdef-validate-gpuorigin3d-shape
                if v.len() > 3 {
                    Err(Error::Type(
                        c"sequence is too long for GPUOrigin3D".to_owned(),
                    ))
                } else {
                    Ok(Origin3d {
                        x: v.first().copied().unwrap_or(0),
                        y: v.get(1).copied().unwrap_or(0),
                        z: v.get(2).copied().unwrap_or(0),
                    })
                }
            },
            GPUOrigin3D::GPUOrigin3DDict(d) => Ok(Origin3d {
                x: d.x,
                y: d.y,
                z: d.z,
            }),
        }
    }
}

impl WebGPUTryConvert<Origin2d> for &GPUOrigin2D {
    type Error = Error;

    /// <https://gpuweb.github.io/gpuweb/#abstract-opdef-validate-gpuorigin2d-shape>
    fn try_convert(self) -> Result<Origin2d, Self::Error> {
        match self {
            GPUOrigin2D::RangeEnforcedUnsignedLongSequence(v) => {
                if v.len() > 2 {
                    Err(Error::Type(
                        c"sequence is too long for GPUOrigin2D".to_owned(),
                    ))
                } else {
                    Ok(Origin2d {
                        x: v.first().copied().unwrap_or(0),
                        y: v.get(1).copied().unwrap_or(0),
                    })
                }
            },
            GPUOrigin2D::GPUOrigin2DDict(d) => Ok(Origin2d { x: d.x, y: d.y }),
        }
    }
}

impl<D> WebGPUTryConvert<TexelCopyTextureInfo> for &GPUTexelCopyTextureInfo<D>
where
    D: Equivalence,
    <D::Promise as PromiseHelpers<D>>::StackRoot: WebGPUPromise<D>,
{
    type Error = Error;

    fn try_convert(self) -> Result<TexelCopyTextureInfo, Self::Error> {
        Ok(TexelCopyTextureInfo {
            texture: self.texture.id().0,
            mip_level: self.mipLevel,
            origin: self
                .origin
                .as_ref()
                .map(WebGPUTryConvert::<Origin3d>::try_convert)
                .transpose()?
                .unwrap_or_default(),
            aspect: match self.aspect {
                GPUTextureAspect::All => TextureAspect::All,
                GPUTextureAspect::Stencil_only => TextureAspect::StencilOnly,
                GPUTextureAspect::Depth_only => TextureAspect::DepthOnly,
            },
        })
    }
}

impl<'a> WebGPUConvert<Option<Cow<'a, str>>> for &GPUObjectDescriptorBase {
    fn convert(self) -> Option<Cow<'a, str>> {
        if self.label.is_empty() {
            None
        } else {
            Some(Cow::Owned(self.label.to_string()))
        }
    }
}

pub(crate) fn convert_bind_group_layout_entry<D>(
    bgle: &GPUBindGroupLayoutEntry,
    device: &D::GPUDevice,
) -> Fallible<Result<BindGroupLayoutEntry, webgpu_traits::Error>>
where
    D: Equivalence,
    <D::Promise as PromiseHelpers<D>>::StackRoot: WebGPUPromise<D>,
{
    let number_of_provided_bindings = bgle.buffer.is_some() as u8 +
        bgle.sampler.is_some() as u8 +
        bgle.storageTexture.is_some() as u8 +
        bgle.texture.is_some() as u8;
    let ty = if let Some(buffer) = &bgle.buffer {
        Some(BindingType::Buffer {
            ty: match buffer.type_ {
                GPUBufferBindingType::Uniform => BufferBindingType::Uniform,
                GPUBufferBindingType::Storage => BufferBindingType::Storage { read_only: false },
                GPUBufferBindingType::Read_only_storage => {
                    BufferBindingType::Storage { read_only: true }
                },
            },
            has_dynamic_offset: buffer.hasDynamicOffset,
            min_binding_size: NonZeroU64::new(buffer.minBindingSize),
        })
    } else if let Some(sampler) = &bgle.sampler {
        Some(BindingType::Sampler(match sampler.type_ {
            GPUSamplerBindingType::Filtering => SamplerBindingType::Filtering,
            GPUSamplerBindingType::Non_filtering => SamplerBindingType::NonFiltering,
            GPUSamplerBindingType::Comparison => SamplerBindingType::Comparison,
        }))
    } else if let Some(storage) = &bgle.storageTexture {
        Some(BindingType::StorageTexture {
            access: match storage.access {
                GPUStorageTextureAccess::Write_only => StorageTextureAccess::WriteOnly,
                GPUStorageTextureAccess::Read_only => StorageTextureAccess::ReadOnly,
                GPUStorageTextureAccess::Read_write => StorageTextureAccess::ReadWrite,
            },
            format: device.validate_texture_format_required_features(&storage.format)?,
            view_dimension: storage.viewDimension.convert(),
        })
    } else if let Some(texture) = &bgle.texture {
        Some(BindingType::Texture {
            sample_type: match texture.sampleType {
                GPUTextureSampleType::Float => TextureSampleType::Float { filterable: true },
                GPUTextureSampleType::Unfilterable_float => {
                    TextureSampleType::Float { filterable: false }
                },
                GPUTextureSampleType::Depth => TextureSampleType::Depth,
                GPUTextureSampleType::Sint => TextureSampleType::Sint,
                GPUTextureSampleType::Uint => TextureSampleType::Uint,
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

    Ok(ty.map(|ty| BindGroupLayoutEntry {
        binding: bgle.binding,
        visibility: ShaderStages::from_bits_retain(bgle.visibility),
        ty,
        count: None,
    }))
}

pub fn convert_texture_descriptor<D>(
    descriptor: &GPUTextureDescriptor,
    device: &D::GPUDevice,
) -> Fallible<(TextureDescriptor<'static>, Extent3d)>
where
    D: Equivalence,
    <D::Promise as PromiseHelpers<D>>::StackRoot: WebGPUPromise<D>,
{
    let size = (&descriptor.size).try_convert()?;
    let desc = TextureDescriptor {
        label: (&descriptor.parent).convert(),
        size,
        mip_level_count: descriptor.mipLevelCount,
        sample_count: descriptor.sampleCount,
        dimension: descriptor.dimension.convert(),
        format: device.validate_texture_format_required_features(&descriptor.format)?,
        usage: TextureUsages::from_bits_retain(descriptor.usage),
        view_formats: descriptor
            .viewFormats
            .iter()
            .map(|tf| device.validate_texture_format_required_features(tf))
            .collect::<Fallible<_>>()?,
    };
    Ok((desc, size))
}

impl WebGPUTryConvert<Color> for &GPUColor {
    type Error = Error;

    fn try_convert(self) -> Result<Color, Self::Error> {
        match self {
            GPUColor::DoubleSequence(s) => {
                // https://gpuweb.github.io/gpuweb/#abstract-opdef-validate-gpucolor-shape
                if s.len() != 4 {
                    Err(Error::Type(c"GPUColor sequence must be len 4".to_owned()))
                } else {
                    Ok(Color {
                        r: *s[0],
                        g: *s[1],
                        b: *s[2],
                        a: *s[3],
                    })
                }
            },
            GPUColor::GPUColorDict(d) => Ok(Color {
                r: *d.r,
                g: *d.g,
                b: *d.b,
                a: *d.a,
            }),
        }
    }
}

impl<'a, D> WebGPUConvert<ProgrammableStageDescriptor<'a>> for &GPUProgrammableStage<D>
where
    D: Equivalence,
    <D::Promise as PromiseHelpers<D>>::StackRoot: WebGPUPromise<D>,
{
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

pub fn convert_texture_for_wgpu_with_cx<D>(
    cx: &mut JSContext,
    texture_view: &GPUTextureOrGPUTextureView<D>,
) -> WebGPUTextureView
where
    D: Equivalence,
    <D::Promise as PromiseHelpers<D>>::StackRoot: WebGPUPromise<D>,
{
    match texture_view {
        GPUTextureOrGPUTextureView::GPUTextureView(view) => view.id(),
        GPUTextureOrGPUTextureView::GPUTexture(texture) => texture.get_default_view(cx),
    }
}

pub(crate) fn convert_bind_group_entry<'a, D>(
    cx: &mut JSContext,
    bind_group: &GPUBindGroupEntry<D>,
) -> BindGroupEntry<'a>
where
    D: Equivalence,
    <D::Promise as PromiseHelpers<D>>::StackRoot: WebGPUPromise<D>,
{
    BindGroupEntry {
        binding: bind_group.binding,
        resource: match bind_group.resource {
            GPUBindingResource::GPUSampler(ref s) => BindingResource::Sampler(s.id().0),
            GPUBindingResource::GPUTextureView(ref t) => BindingResource::TextureView(t.id().0),
            GPUBindingResource::GPUTexture(ref t) => {
                BindingResource::TextureView(t.get_default_view(cx).0)
            },
            GPUBindingResource::GPUBufferBinding(ref b) => BindingResource::Buffer(BufferBinding {
                buffer: b.buffer.id().0,
                offset: b.offset,
                size: b.size,
            }),
            GPUBindingResource::GPUBuffer(ref b) => BindingResource::Buffer(BufferBinding {
                buffer: b.id().0,
                offset: 0,
                size: None,
            }),
            GPUBindingResource::GPUExternalTexture(ref t) => {
                BindingResource::ExternalTexture(t.id().0)
            },
        },
    }
}

impl WebGPUConvert<TextureDimension> for GPUTextureDimension {
    fn convert(self) -> TextureDimension {
        match self {
            GPUTextureDimension::_1d => TextureDimension::D1,
            GPUTextureDimension::_2d => TextureDimension::D2,
            GPUTextureDimension::_3d => TextureDimension::D3,
        }
    }
}

impl WebGPUConvert<WGPUPredefinedColorSpace> for PredefinedColorSpace {
    fn convert(self) -> WGPUPredefinedColorSpace {
        match self {
            PredefinedColorSpace::Srgb | PredefinedColorSpace::Srgb_linear => {
                WGPUPredefinedColorSpace::Srgb
            },
            PredefinedColorSpace::Display_p3 | PredefinedColorSpace::Display_p3_linear => {
                WGPUPredefinedColorSpace::DisplayP3
            },
        }
    }
}

impl WebGPUConvert<QuerySetDescriptor<'static>> for &GPUQuerySetDescriptor {
    fn convert(self) -> QuerySetDescriptor<'static> {
        QuerySetDescriptor {
            label: (&self.parent).convert(),
            count: self.count,
            ty: match self.type_ {
                GPUQueryType::Occlusion => QueryType::Occlusion,
                GPUQueryType::Timestamp => QueryType::Timestamp,
            },
        }
    }
}

impl<D> WebGPUConvert<PassTimestampWrites> for &GPUComputePassTimestampWrites<D>
where
    D: Equivalence,
    <D::Promise as PromiseHelpers<D>>::StackRoot: WebGPUPromise<D>,
{
    fn convert(self) -> PassTimestampWrites {
        PassTimestampWrites {
            query_set: self.querySet.id().0,
            beginning_of_pass_write_index: self.beginningOfPassWriteIndex,
            end_of_pass_write_index: self.endOfPassWriteIndex,
        }
    }
}

impl<D> WebGPUConvert<PassTimestampWrites> for &GPURenderPassTimestampWrites<D>
where
    D: Equivalence,
    <D::Promise as PromiseHelpers<D>>::StackRoot: WebGPUPromise<D>,
{
    fn convert(self) -> PassTimestampWrites {
        PassTimestampWrites {
            query_set: self.querySet.id().0,
            beginning_of_pass_write_index: self.beginningOfPassWriteIndex,
            end_of_pass_write_index: self.endOfPassWriteIndex,
        }
    }
}

impl<D> WebGPUConvert<ComputePassDescriptor<'static>> for &GPUComputePassDescriptor<D>
where
    D: Equivalence,
    <D::Promise as PromiseHelpers<D>>::StackRoot: WebGPUPromise<D>,
{
    fn convert(self) -> ComputePassDescriptor<'static> {
        ComputePassDescriptor {
            label: (&self.parent).convert(),
            timestamp_writes: self.timestampWrites.as_ref().map(WebGPUConvert::convert),
        }
    }
}

impl WebGPUConvert<IndexFormat> for GPUIndexFormat {
    fn convert(self) -> IndexFormat {
        match self {
            GPUIndexFormat::Uint16 => IndexFormat::Uint16,
            GPUIndexFormat::Uint32 => IndexFormat::Uint32,
        }
    }
}
