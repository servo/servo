/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use api::{AlphaType, DocumentLayer, PremultipliedColorF, YuvFormat, YuvColorSpace};
use api::EdgeAaSegmentMask;
use api::units::*;
use crate::spatial_tree::{SpatialTree, ROOT_SPATIAL_NODE_INDEX, SpatialNodeIndex};
use crate::gpu_cache::{GpuCacheAddress, GpuDataRequest};
use crate::internal_types::FastHashMap;
use crate::render_task::RenderTaskAddress;
use crate::renderer::ShaderColorMode;
use std::i32;
use crate::util::{TransformedRectKind, MatrixHelpers};
use crate::glyph_rasterizer::SubpixelDirection;
use crate::util::pack_as_float;

// Contains type that must exactly match the same structures declared in GLSL.

pub const VECS_PER_TRANSFORM: usize = 8;

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(C)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct ZBufferId(pub i32);

const MAX_DOCUMENT_LAYERS : i8 = 1 << 3;
const MAX_DOCUMENT_LAYER_VALUE : i8 = MAX_DOCUMENT_LAYERS / 2 - 1;
const MIN_DOCUMENT_LAYER_VALUE : i8 = -MAX_DOCUMENT_LAYERS / 2;

impl ZBufferId {
    pub fn invalid() -> Self {
        ZBufferId(i32::MAX)
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct ZBufferIdGenerator {
    base: i32,
    next: i32,
    max_items_per_document_layer: i32,
}

impl ZBufferIdGenerator {
    pub fn new(layer: DocumentLayer, max_depth_ids: i32) -> Self {
        debug_assert!(layer >= MIN_DOCUMENT_LAYER_VALUE);
        debug_assert!(layer <= MAX_DOCUMENT_LAYER_VALUE);
        let max_items_per_document_layer = max_depth_ids / MAX_DOCUMENT_LAYERS as i32;
        ZBufferIdGenerator {
            base: layer as i32 * max_items_per_document_layer,
            next: 0,
            max_items_per_document_layer,
        }
    }

    pub fn next(&mut self) -> ZBufferId {
        debug_assert!(self.next < self.max_items_per_document_layer);
        let id = ZBufferId(self.next + self.base);
        self.next += 1;
        id
    }
}

/// A shader kind identifier that can be used by a generic-shader to select the behavior at runtime.
///
/// Not all brush kinds need to be present in this enum, only those we want to support in the generic
/// brush shader.
/// Do not use the 24 lowest bits. This will be packed with other information in the vertex attributes.
/// The constants must match the corresponding defines in brush_multi.glsl.
#[repr(i32)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BrushShaderKind {
    None            = 0,
    Solid           = 1,
    Image           = 2,
    Text            = 3,
    LinearGradient  = 4,
    RadialGradient  = 5,
    ConicGradient   = 6,
    Blend           = 7,
    MixBlend        = 8,
    Yuv             = 9,
    Opacity         = 10,
}

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[repr(C)]
pub enum RasterizationSpace {
    Local = 0,
    Screen = 1,
}

#[derive(Debug, Copy, Clone, MallocSizeOf)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[repr(C)]
pub enum BoxShadowStretchMode {
    Stretch = 0,
    Simple = 1,
}

#[repr(i32)]
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub enum BlurDirection {
    Horizontal = 0,
    Vertical,
}

#[derive(Debug)]
#[repr(C)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct BlurInstance {
    pub task_address: RenderTaskAddress,
    pub src_task_address: RenderTaskAddress,
    pub blur_direction: BlurDirection,
}

#[derive(Debug)]
#[repr(C)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct ScalingInstance {
    pub target_rect: DeviceRect,
    pub source_rect: DeviceIntRect,
    pub source_layer: i32,
}

#[derive(Debug)]
#[repr(C)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct SvgFilterInstance {
    pub task_address: RenderTaskAddress,
    pub input_1_task_address: RenderTaskAddress,
    pub input_2_task_address: RenderTaskAddress,
    pub kind: u16,
    pub input_count: u16,
    pub generic_int: u16,
    pub extra_data_address: GpuCacheAddress,
}

#[derive(Copy, Clone, Debug, Hash, MallocSizeOf, PartialEq, Eq)]
#[repr(C)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub enum BorderSegment {
    TopLeft,
    TopRight,
    BottomRight,
    BottomLeft,
    Left,
    Top,
    Right,
    Bottom,
}

#[derive(Debug, Clone)]
#[repr(C)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct BorderInstance {
    pub task_origin: DevicePoint,
    pub local_rect: DeviceRect,
    pub color0: PremultipliedColorF,
    pub color1: PremultipliedColorF,
    pub flags: i32,
    pub widths: DeviceSize,
    pub radius: DeviceSize,
    pub clip_params: [f32; 8],
}

/// A clipping primitive drawn into the clipping mask.
/// Could be an image or a rectangle, which defines the
/// way `address` is treated.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[repr(C)]
pub struct ClipMaskInstance {
    pub clip_transform_id: TransformPaletteId,
    pub prim_transform_id: TransformPaletteId,
    pub clip_data_address: GpuCacheAddress,
    pub resource_address: GpuCacheAddress,
    pub local_pos: LayoutPoint,
    pub tile_rect: LayoutRect,
    pub sub_rect: DeviceRect,
    pub task_origin: DevicePoint,
    pub screen_origin: DevicePoint,
    pub device_pixel_scale: f32,
}

/// A border corner dot or dash drawn into the clipping mask.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[repr(C)]
pub struct ClipMaskBorderCornerDotDash {
    pub clip_mask_instance: ClipMaskInstance,
    pub dot_dash_data: [f32; 8],
}

// 16 bytes per instance should be enough for anyone!
#[derive(Debug, Clone)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct PrimitiveInstanceData {
    data: [i32; 4],
}

/// Vertex format for resolve style operations with pixel local storage.
#[derive(Debug, Clone)]
#[repr(C)]
pub struct ResolveInstanceData {
    rect: [f32; 4],
}

impl ResolveInstanceData {
    pub fn new(rect: DeviceIntRect) -> Self {
        ResolveInstanceData {
            rect: [
                rect.origin.x as f32,
                rect.origin.y as f32,
                rect.size.width as f32,
                rect.size.height as f32,
            ],
        }
    }
}

/// Vertex format for picture cache composite shader.
/// When editing the members, update desc::COMPOSITE
/// so its list of instance_attributes matches:
#[derive(Debug, Clone)]
#[repr(C)]
pub struct CompositeInstance {
    // Device space rectangle of surface
    rect: DeviceRect,
    // Device space clip rect for this surface
    clip_rect: DeviceRect,
    // Color for solid color tiles, white otherwise
    color: PremultipliedColorF,

    // Packed into a single vec4 (aParams)
    z_id: f32,
    color_space_or_uv_type: f32, // YuvColorSpace for YUV;
                                 // UV coordinate space for RGB
    yuv_format: f32,            // YuvFormat
    yuv_rescale: f32,

    // UV rectangles (pixel space) for color / yuv texture planes
    uv_rects: [TexelRect; 3],

    // Texture array layers for color / yuv texture planes
    texture_layers: [f32; 3],
}

impl CompositeInstance {
    pub fn new(
        rect: DeviceRect,
        clip_rect: DeviceRect,
        color: PremultipliedColorF,
        layer: f32,
        z_id: ZBufferId,
    ) -> Self {
        let uv = TexelRect::new(0.0, 0.0, 1.0, 1.0);
        CompositeInstance {
            rect,
            clip_rect,
            color,
            z_id: z_id.0 as f32,
            color_space_or_uv_type: pack_as_float(0u32),
            yuv_format: 0.0,
            yuv_rescale: 0.0,
            texture_layers: [layer, 0.0, 0.0],
            uv_rects: [uv, uv, uv],
        }
    }

    pub fn new_rgb(
        rect: DeviceRect,
        clip_rect: DeviceRect,
        color: PremultipliedColorF,
        layer: f32,
        z_id: ZBufferId,
        uv_rect: TexelRect,
    ) -> Self {
        CompositeInstance {
            rect,
            clip_rect,
            color,
            z_id: z_id.0 as f32,
            color_space_or_uv_type: pack_as_float(1u32),
            yuv_format: 0.0,
            yuv_rescale: 0.0,
            texture_layers: [layer, 0.0, 0.0],
            uv_rects: [uv_rect, uv_rect, uv_rect],
        }
    }

    pub fn new_yuv(
        rect: DeviceRect,
        clip_rect: DeviceRect,
        z_id: ZBufferId,
        yuv_color_space: YuvColorSpace,
        yuv_format: YuvFormat,
        yuv_rescale: f32,
        texture_layers: [f32; 3],
        uv_rects: [TexelRect; 3],
    ) -> Self {
        CompositeInstance {
            rect,
            clip_rect,
            color: PremultipliedColorF::WHITE,
            z_id: z_id.0 as f32,
            color_space_or_uv_type: pack_as_float(yuv_color_space as u32),
            yuv_format: pack_as_float(yuv_format as u32),
            yuv_rescale,
            texture_layers,
            uv_rects,
        }
    }
}

/// Vertex format for issuing colored quads.
#[derive(Debug, Clone)]
#[repr(C)]
pub struct ClearInstance {
    pub rect: [f32; 4],
    pub color: [f32; 4],
}

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct PrimitiveHeaderIndex(pub i32);

#[derive(Debug)]
#[repr(C)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct PrimitiveHeaders {
    // The integer-type headers for a primitive.
    pub headers_int: Vec<PrimitiveHeaderI>,
    // The float-type headers for a primitive.
    pub headers_float: Vec<PrimitiveHeaderF>,
}

impl PrimitiveHeaders {
    pub fn new() -> PrimitiveHeaders {
        PrimitiveHeaders {
            headers_int: Vec::new(),
            headers_float: Vec::new(),
        }
    }

    // Add a new primitive header.
    pub fn push(
        &mut self,
        prim_header: &PrimitiveHeader,
        z: ZBufferId,
        user_data: [i32; 4],
    ) -> PrimitiveHeaderIndex {
        debug_assert_eq!(self.headers_int.len(), self.headers_float.len());
        let id = self.headers_float.len();

        self.headers_float.push(PrimitiveHeaderF {
            local_rect: prim_header.local_rect,
            local_clip_rect: prim_header.local_clip_rect,
        });

        self.headers_int.push(PrimitiveHeaderI {
            z,
            unused: 0,
            specific_prim_address: prim_header.specific_prim_address.as_int(),
            transform_id: prim_header.transform_id,
            user_data,
        });

        PrimitiveHeaderIndex(id as i32)
    }
}

// This is a convenience type used to make it easier to pass
// the common parts around during batching.
#[derive(Debug)]
pub struct PrimitiveHeader {
    pub local_rect: LayoutRect,
    pub local_clip_rect: LayoutRect,
    pub specific_prim_address: GpuCacheAddress,
    pub transform_id: TransformPaletteId,
}

// f32 parts of a primitive header
#[derive(Debug)]
#[repr(C)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct PrimitiveHeaderF {
    pub local_rect: LayoutRect,
    pub local_clip_rect: LayoutRect,
}

// i32 parts of a primitive header
// TODO(gw): Compress parts of these down to u16
#[derive(Debug)]
#[repr(C)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct PrimitiveHeaderI {
    pub z: ZBufferId,
    pub specific_prim_address: i32,
    pub transform_id: TransformPaletteId,
    pub unused: i32,                    // To ensure required 16 byte alignment of vertex textures
    pub user_data: [i32; 4],
}

pub struct GlyphInstance {
    pub prim_header_index: PrimitiveHeaderIndex,
}

impl GlyphInstance {
    pub fn new(
        prim_header_index: PrimitiveHeaderIndex,
    ) -> Self {
        GlyphInstance {
            prim_header_index,
        }
    }

    // TODO(gw): Some of these fields can be moved to the primitive
    //           header since they are constant, and some can be
    //           compressed to a smaller size.
    pub fn build(&self,
        render_task: RenderTaskAddress,
        clip_task: RenderTaskAddress,
        subpx_dir: SubpixelDirection,
        glyph_index_in_text_run: i32,
        glyph_uv_rect: GpuCacheAddress,
        color_mode: ShaderColorMode,
    ) -> PrimitiveInstanceData {
        PrimitiveInstanceData {
            data: [
                self.prim_header_index.0 as i32,
                ((render_task.0 as i32) << 16)
                | clip_task.0 as i32,
                (subpx_dir as u32 as i32) << 24
                | (color_mode as u32 as i32) << 16
                | glyph_index_in_text_run,
                glyph_uv_rect.as_int()
                | ((BrushShaderKind::Text as i32) << 24),
            ],
        }
    }
}

pub struct SplitCompositeInstance {
    pub prim_header_index: PrimitiveHeaderIndex,
    pub polygons_address: GpuCacheAddress,
    pub z: ZBufferId,
    pub render_task_address: RenderTaskAddress,
}

impl From<SplitCompositeInstance> for PrimitiveInstanceData {
    fn from(instance: SplitCompositeInstance) -> Self {
        PrimitiveInstanceData {
            data: [
                instance.prim_header_index.0,
                instance.polygons_address.as_int(),
                instance.z.0,
                instance.render_task_address.0 as i32,
            ],
        }
    }
}

bitflags! {
    /// Flags that define how the common brush shader
    /// code should process this instance.
    #[cfg_attr(feature = "capture", derive(Serialize))]
    #[cfg_attr(feature = "replay", derive(Deserialize))]
    #[derive(MallocSizeOf)]
    pub struct BrushFlags: u8 {
        /// Apply perspective interpolation to UVs
        const PERSPECTIVE_INTERPOLATION = 1;
        /// Do interpolation relative to segment rect,
        /// rather than primitive rect.
        const SEGMENT_RELATIVE = 2;
        /// Repeat UVs horizontally.
        const SEGMENT_REPEAT_X = 4;
        /// Repeat UVs vertically.
        const SEGMENT_REPEAT_Y = 8;
        /// Horizontally follow border-image-repeat: round.
        const SEGMENT_REPEAT_X_ROUND = 16;
        /// Vertically follow border-image-repeat: round.
        const SEGMENT_REPEAT_Y_ROUND = 32;
        /// Middle (fill) area of a border-image-repeat.
        const SEGMENT_NINEPATCH_MIDDLE = 64;
        /// The extra segment data is a texel rect.
        const SEGMENT_TEXEL_RECT = 128;
    }
}

/// Convenience structure to encode into PrimitiveInstanceData.
pub struct BrushInstance {
    pub prim_header_index: PrimitiveHeaderIndex,
    pub render_task_address: RenderTaskAddress,
    pub clip_task_address: RenderTaskAddress,
    pub segment_index: i32,
    pub edge_flags: EdgeAaSegmentMask,
    pub brush_flags: BrushFlags,
    pub resource_address: i32,
    pub brush_kind: BrushShaderKind,
}

impl From<BrushInstance> for PrimitiveInstanceData {
    fn from(instance: BrushInstance) -> Self {
        PrimitiveInstanceData {
            data: [
                instance.prim_header_index.0,
                ((instance.render_task_address.0 as i32) << 16)
                | instance.clip_task_address.0 as i32,
                instance.segment_index
                | ((instance.edge_flags.bits() as i32) << 16)
                | ((instance.brush_flags.bits() as i32) << 24),
                instance.resource_address
                | ((instance.brush_kind as i32) << 24),
            ]
        }
    }
}

/// Convenience structure to encode into the image brush's user data.
#[derive(Copy, Clone, Debug)]
pub struct ImageBrushData {
    pub color_mode: ShaderColorMode,
    pub alpha_type: AlphaType,
    pub raster_space: RasterizationSpace,
    pub opacity: f32,
}

impl ImageBrushData {
    #[inline]
    pub fn encode(&self) -> [i32; 4] {
        [
            self.color_mode as i32 | ((self.alpha_type as i32) << 16),
            self.raster_space as i32,
            get_shader_opacity(self.opacity),
            0,
        ]
    }
}

// Represents the information about a transform palette
// entry that is passed to shaders. It includes an index
// into the transform palette, and a set of flags. The
// only flag currently used determines whether the
// transform is axis-aligned (and this should have
// pixel snapping applied).
#[derive(Copy, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[repr(C)]
pub struct TransformPaletteId(pub u32);

impl TransformPaletteId {
    /// Identity transform ID.
    pub const IDENTITY: Self = TransformPaletteId(0);

    /// Extract the transform kind from the id.
    pub fn transform_kind(&self) -> TransformedRectKind {
        if (self.0 >> 24) == 0 {
            TransformedRectKind::AxisAligned
        } else {
            TransformedRectKind::Complex
        }
    }
}

/// The GPU data payload for a transform palette entry.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[repr(C)]
pub struct TransformData {
    transform: LayoutToPictureTransform,
    inv_transform: PictureToLayoutTransform,
}

impl TransformData {
    fn invalid() -> Self {
        TransformData {
            transform: LayoutToPictureTransform::identity(),
            inv_transform: PictureToLayoutTransform::identity(),
        }
    }
}

// Extra data stored about each transform palette entry.
#[derive(Clone)]
pub struct TransformMetadata {
    transform_kind: TransformedRectKind,
}

impl TransformMetadata {
    pub fn invalid() -> Self {
        TransformMetadata {
            transform_kind: TransformedRectKind::AxisAligned,
        }
    }
}

#[derive(Debug, Hash, Eq, PartialEq)]
struct RelativeTransformKey {
    from_index: SpatialNodeIndex,
    to_index: SpatialNodeIndex,
}

// Stores a contiguous list of TransformData structs, that
// are ready for upload to the GPU.
// TODO(gw): For now, this only stores the complete local
//           to world transform for each spatial node. In
//           the future, the transform palette will support
//           specifying a coordinate system that the transform
//           should be relative to.
pub struct TransformPalette {
    transforms: Vec<TransformData>,
    metadata: Vec<TransformMetadata>,
    map: FastHashMap<RelativeTransformKey, usize>,
}

impl TransformPalette {
    pub fn new(count: usize) -> Self {
        let _ = VECS_PER_TRANSFORM;
        TransformPalette {
            transforms: vec![TransformData::invalid(); count],
            metadata: vec![TransformMetadata::invalid(); count],
            map: FastHashMap::default(),
        }
    }

    pub fn finish(self) -> Vec<TransformData> {
        self.transforms
    }

    pub fn set_world_transform(
        &mut self,
        index: SpatialNodeIndex,
        transform: LayoutToWorldTransform,
    ) {
        register_transform(
            &mut self.metadata,
            &mut self.transforms,
            index,
            ROOT_SPATIAL_NODE_INDEX,
            // We know the root picture space == world space
            transform.with_destination::<PicturePixel>(),
        );
    }

    fn get_index(
        &mut self,
        child_index: SpatialNodeIndex,
        parent_index: SpatialNodeIndex,
        spatial_tree: &SpatialTree,
    ) -> usize {
        if parent_index == ROOT_SPATIAL_NODE_INDEX {
            child_index.0 as usize
        } else if child_index == parent_index {
            0
        } else {
            let key = RelativeTransformKey {
                from_index: child_index,
                to_index: parent_index,
            };

            let metadata = &mut self.metadata;
            let transforms = &mut self.transforms;

            *self.map
                .entry(key)
                .or_insert_with(|| {
                    let transform = spatial_tree.get_relative_transform(
                        child_index,
                        parent_index,
                    )
                    .into_transform()
                    .with_destination::<PicturePixel>();

                    register_transform(
                        metadata,
                        transforms,
                        child_index,
                        parent_index,
                        transform,
                    )
                })
        }
    }

    // Get a transform palette id for the given spatial node.
    // TODO(gw): In the future, it will be possible to specify
    //           a coordinate system id here, to allow retrieving
    //           transforms in the local space of a given spatial node.
    pub fn get_id(
        &mut self,
        from_index: SpatialNodeIndex,
        to_index: SpatialNodeIndex,
        spatial_tree: &SpatialTree,
    ) -> TransformPaletteId {
        let index = self.get_index(
            from_index,
            to_index,
            spatial_tree,
        );
        let transform_kind = self.metadata[index].transform_kind as u32;
        TransformPaletteId(
            (index as u32) |
            (transform_kind << 24)
        )
    }
}

// Texture cache resources can be either a simple rect, or define
// a polygon within a rect by specifying a UV coordinate for each
// corner. This is useful for rendering screen-space rasterized
// off-screen surfaces.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub enum UvRectKind {
    // The 2d bounds of the texture cache entry define the
    // valid UV space for this texture cache entry.
    Rect,
    // The four vertices below define a quad within
    // the texture cache entry rect. The shader can
    // use a bilerp() to correctly interpolate a
    // UV coord in the vertex shader.
    Quad {
        top_left: DeviceHomogeneousVector,
        top_right: DeviceHomogeneousVector,
        bottom_left: DeviceHomogeneousVector,
        bottom_right: DeviceHomogeneousVector,
    },
}

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct ImageSource {
    pub p0: DevicePoint,
    pub p1: DevicePoint,
    pub texture_layer: f32,
    pub user_data: [f32; 3],
    pub uv_rect_kind: UvRectKind,
}

impl ImageSource {
    pub fn write_gpu_blocks(&self, request: &mut GpuDataRequest) {
        // see fetch_image_resource in GLSL
        // has to be VECS_PER_IMAGE_RESOURCE vectors
        request.push([
            self.p0.x,
            self.p0.y,
            self.p1.x,
            self.p1.y,
        ]);
        request.push([
            self.texture_layer,
            self.user_data[0],
            self.user_data[1],
            self.user_data[2],
        ]);

        // If this is a polygon uv kind, then upload the four vertices.
        if let UvRectKind::Quad { top_left, top_right, bottom_left, bottom_right } = self.uv_rect_kind {
            // see fetch_image_resource_extra in GLSL
            //Note: we really need only 3 components per point here: X, Y, and W
            request.push(top_left);
            request.push(top_right);
            request.push(bottom_left);
            request.push(bottom_right);
        }
    }
}

// Set the local -> world transform for a given spatial
// node in the transform palette.
fn register_transform(
    metadatas: &mut Vec<TransformMetadata>,
    transforms: &mut Vec<TransformData>,
    from_index: SpatialNodeIndex,
    to_index: SpatialNodeIndex,
    transform: LayoutToPictureTransform,
) -> usize {
    // TODO: refactor the calling code to not even try
    // registering a non-invertible transform.
    let inv_transform = transform
        .inverse()
        .unwrap_or_else(PictureToLayoutTransform::identity);

    let metadata = TransformMetadata {
        transform_kind: transform.transform_kind()
    };
    let data = TransformData {
        transform,
        inv_transform,
    };

    if to_index == ROOT_SPATIAL_NODE_INDEX {
        let index = from_index.0 as usize;
        metadatas[index] = metadata;
        transforms[index] = data;
        index
    } else {
        let index = transforms.len();
        metadatas.push(metadata);
        transforms.push(data);
        index
    }
}

pub fn get_shader_opacity(opacity: f32) -> i32 {
    (opacity * 65535.0).round() as i32
}
