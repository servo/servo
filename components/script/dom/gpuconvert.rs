/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;

use webgpu::wgc::command as wgpu_com;
use webgpu::wgt;

use crate::dom::bindings::codegen::Bindings::WebGPUBinding::{
    GPUAddressMode, GPUBlendComponent, GPUBlendFactor, GPUBlendOperation, GPUCompareFunction,
    GPUCullMode, GPUExtent3D, GPUExtent3DDict, GPUFilterMode, GPUFrontFace, GPUImageCopyBuffer,
    GPUImageCopyTexture, GPUImageDataLayout, GPUIndexFormat, GPULoadOp, GPUObjectDescriptorBase,
    GPUOrigin3D, GPUPrimitiveState, GPUPrimitiveTopology, GPUStencilOperation, GPUStoreOp,
    GPUTextureAspect, GPUTextureFormat, GPUTextureViewDimension, GPUVertexFormat,
};

pub fn convert_texture_format(format: GPUTextureFormat) -> wgt::TextureFormat {
    match format {
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
        GPUTextureFormat::Rg11b10float => wgt::TextureFormat::Rg11b10Float,
        GPUTextureFormat::Bc6h_rgb_float => wgt::TextureFormat::Bc6hRgbFloat,
    }
}

pub fn convert_texture_view_dimension(
    dimension: GPUTextureViewDimension,
) -> wgt::TextureViewDimension {
    match dimension {
        GPUTextureViewDimension::_1d => wgt::TextureViewDimension::D1,
        GPUTextureViewDimension::_2d => wgt::TextureViewDimension::D2,
        GPUTextureViewDimension::_2d_array => wgt::TextureViewDimension::D2Array,
        GPUTextureViewDimension::Cube => wgt::TextureViewDimension::Cube,
        GPUTextureViewDimension::Cube_array => wgt::TextureViewDimension::CubeArray,
        GPUTextureViewDimension::_3d => wgt::TextureViewDimension::D3,
    }
}

pub fn convert_texture_size_to_dict(size: &GPUExtent3D) -> GPUExtent3DDict {
    match *size {
        GPUExtent3D::GPUExtent3DDict(ref dict) => GPUExtent3DDict {
            width: dict.width,
            height: dict.height,
            depthOrArrayLayers: dict.depthOrArrayLayers,
        },
        GPUExtent3D::RangeEnforcedUnsignedLongSequence(ref v) => {
            let mut w = v.clone();
            w.resize(3, 1);
            GPUExtent3DDict {
                width: w[0],
                height: w[1],
                depthOrArrayLayers: w[2],
            }
        },
    }
}

pub fn convert_texture_size_to_wgt(size: &GPUExtent3DDict) -> wgt::Extent3d {
    wgt::Extent3d {
        width: size.width,
        height: size.height,
        depth_or_array_layers: size.depthOrArrayLayers,
    }
}

pub fn convert_image_data_layout(data_layout: &GPUImageDataLayout) -> wgt::ImageDataLayout {
    wgt::ImageDataLayout {
        offset: data_layout.offset as wgt::BufferAddress,
        bytes_per_row: data_layout.bytesPerRow,
        rows_per_image: data_layout.rowsPerImage,
    }
}

pub fn convert_vertex_format(format: GPUVertexFormat) -> wgt::VertexFormat {
    match format {
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

pub fn convert_primitive_state(primitive_state: &GPUPrimitiveState) -> wgt::PrimitiveState {
    wgt::PrimitiveState {
        topology: convert_primitive_topology(&primitive_state.topology),
        strip_index_format: primitive_state.stripIndexFormat.map(
            |index_format| match index_format {
                GPUIndexFormat::Uint16 => wgt::IndexFormat::Uint16,
                GPUIndexFormat::Uint32 => wgt::IndexFormat::Uint32,
            },
        ),
        front_face: match primitive_state.frontFace {
            GPUFrontFace::Ccw => wgt::FrontFace::Ccw,
            GPUFrontFace::Cw => wgt::FrontFace::Cw,
        },
        cull_mode: match primitive_state.cullMode {
            GPUCullMode::None => None,
            GPUCullMode::Front => Some(wgt::Face::Front),
            GPUCullMode::Back => Some(wgt::Face::Back),
        },
        unclipped_depth: primitive_state.clampDepth,
        ..Default::default()
    }
}

pub fn convert_primitive_topology(
    primitive_topology: &GPUPrimitiveTopology,
) -> wgt::PrimitiveTopology {
    match primitive_topology {
        GPUPrimitiveTopology::Point_list => wgt::PrimitiveTopology::PointList,
        GPUPrimitiveTopology::Line_list => wgt::PrimitiveTopology::LineList,
        GPUPrimitiveTopology::Line_strip => wgt::PrimitiveTopology::LineStrip,
        GPUPrimitiveTopology::Triangle_list => wgt::PrimitiveTopology::TriangleList,
        GPUPrimitiveTopology::Triangle_strip => wgt::PrimitiveTopology::TriangleStrip,
    }
}

pub fn convert_address_mode(address_mode: GPUAddressMode) -> wgt::AddressMode {
    match address_mode {
        GPUAddressMode::Clamp_to_edge => wgt::AddressMode::ClampToEdge,
        GPUAddressMode::Repeat => wgt::AddressMode::Repeat,
        GPUAddressMode::Mirror_repeat => wgt::AddressMode::MirrorRepeat,
    }
}

pub fn convert_filter_mode(filter_mode: GPUFilterMode) -> wgt::FilterMode {
    match filter_mode {
        GPUFilterMode::Nearest => wgt::FilterMode::Nearest,
        GPUFilterMode::Linear => wgt::FilterMode::Linear,
    }
}

pub fn convert_view_dimension(
    view_dimension: GPUTextureViewDimension,
) -> wgt::TextureViewDimension {
    match view_dimension {
        GPUTextureViewDimension::_1d => wgt::TextureViewDimension::D1,
        GPUTextureViewDimension::_2d => wgt::TextureViewDimension::D2,
        GPUTextureViewDimension::_2d_array => wgt::TextureViewDimension::D2Array,
        GPUTextureViewDimension::Cube => wgt::TextureViewDimension::Cube,
        GPUTextureViewDimension::Cube_array => wgt::TextureViewDimension::CubeArray,
        GPUTextureViewDimension::_3d => wgt::TextureViewDimension::D3,
    }
}

pub fn convert_compare_function(compare: GPUCompareFunction) -> wgt::CompareFunction {
    match compare {
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

pub fn convert_blend_factor(factor: &GPUBlendFactor) -> wgt::BlendFactor {
    match factor {
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

pub fn convert_blend_component(blend_component: &GPUBlendComponent) -> wgt::BlendComponent {
    wgt::BlendComponent {
        src_factor: convert_blend_factor(&blend_component.srcFactor),
        dst_factor: convert_blend_factor(&blend_component.dstFactor),
        operation: match blend_component.operation {
            GPUBlendOperation::Add => wgt::BlendOperation::Add,
            GPUBlendOperation::Subtract => wgt::BlendOperation::Subtract,
            GPUBlendOperation::Reverse_subtract => wgt::BlendOperation::ReverseSubtract,
            GPUBlendOperation::Min => wgt::BlendOperation::Min,
            GPUBlendOperation::Max => wgt::BlendOperation::Max,
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

pub fn convert_stencil_op(operation: GPUStencilOperation) -> wgt::StencilOperation {
    match operation {
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

pub fn convert_ic_buffer(ic_buffer: &GPUImageCopyBuffer) -> wgpu_com::ImageCopyBuffer {
    wgpu_com::ImageCopyBuffer {
        buffer: ic_buffer.buffer.id().0,
        layout: convert_image_data_layout(&ic_buffer.parent),
    }
}

pub fn convert_ic_texture(ic_texture: &GPUImageCopyTexture) -> wgpu_com::ImageCopyTexture {
    wgpu_com::ImageCopyTexture {
        texture: ic_texture.texture.id().0,
        mip_level: ic_texture.mipLevel,
        origin: match ic_texture.origin {
            Some(GPUOrigin3D::RangeEnforcedUnsignedLongSequence(ref v)) => {
                let mut w = v.clone();
                w.resize(3, 0);
                wgt::Origin3d {
                    x: w[0],
                    y: w[1],
                    z: w[2],
                }
            },
            Some(GPUOrigin3D::GPUOrigin3DDict(ref d)) => wgt::Origin3d {
                x: d.x,
                y: d.y,
                z: d.z,
            },
            None => wgt::Origin3d::default(),
        },
        aspect: match ic_texture.aspect {
            GPUTextureAspect::All => wgt::TextureAspect::All,
            GPUTextureAspect::Stencil_only => wgt::TextureAspect::StencilOnly,
            GPUTextureAspect::Depth_only => wgt::TextureAspect::DepthOnly,
        },
    }
}

pub fn convert_label(parent: &GPUObjectDescriptorBase) -> Option<Cow<'static, str>> {
    parent.label.as_ref().map(|s| Cow::Owned(s.to_string()))
}
