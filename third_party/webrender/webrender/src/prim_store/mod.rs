/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use api::{BorderRadius, ClipMode, ColorF, ColorU};
use api::{ImageRendering, RepeatMode, PrimitiveFlags};
use api::{PremultipliedColorF, PropertyBinding, Shadow, GradientStop};
use api::{BoxShadowClipMode, LineStyle, LineOrientation, BorderStyle};
use api::{PrimitiveKeyKind, ExtendMode, EdgeAaSegmentMask};
use api::image_tiling::{self, Repetition};
use api::units::*;
use crate::border::{get_max_scale_for_border, build_border_instances};
use crate::border::BorderSegmentCacheKey;
use crate::clip::{ClipStore};
use crate::spatial_tree::{ROOT_SPATIAL_NODE_INDEX, SpatialTree, CoordinateSpaceMapping, SpatialNodeIndex, VisibleFace};
use crate::clip::{ClipDataStore, ClipNodeFlags, ClipChainId, ClipChainInstance, ClipItemKind};
use crate::debug_colors;
use crate::debug_render::DebugItem;
use crate::scene_building::{CreateShadow, IsVisible};
use euclid::{SideOffsets2D, Transform3D, Rect, Scale, Size2D, Point2D, Vector2D};
use euclid::approxeq::ApproxEq;
use crate::frame_builder::{FrameBuildingContext, FrameBuildingState, PictureContext, PictureState};
use crate::frame_builder::{FrameVisibilityContext, FrameVisibilityState};
use crate::glyph_rasterizer::GlyphKey;
use crate::gpu_cache::{GpuCache, GpuCacheAddress, GpuCacheHandle, GpuDataRequest, ToGpuBlocks};
use crate::gpu_types::{BrushFlags};
use crate::intern;
use crate::internal_types::PlaneSplitAnchor;
use malloc_size_of::MallocSizeOf;
use crate::picture::{PictureCompositeMode, PicturePrimitive, ClusterFlags, TileCacheLogger};
use crate::picture::{PrimitiveList, RecordedDirtyRegion, SurfaceIndex, RetainedTiles, RasterConfig};
use crate::prim_store::backdrop::BackdropDataHandle;
use crate::prim_store::borders::{ImageBorderDataHandle, NormalBorderDataHandle};
use crate::prim_store::gradient::{GRADIENT_FP_STOPS, GradientCacheKey, GradientStopKey};
use crate::prim_store::gradient::{LinearGradientPrimitive, LinearGradientDataHandle, RadialGradientDataHandle, ConicGradientDataHandle};
use crate::prim_store::image::{ImageDataHandle, ImageInstance, VisibleImageTile, YuvImageDataHandle};
use crate::prim_store::line_dec::{LineDecorationDataHandle,MAX_LINE_DECORATION_RESOLUTION};
use crate::prim_store::picture::PictureDataHandle;
use crate::prim_store::text_run::{TextRunDataHandle, TextRunPrimitive};
#[cfg(debug_assertions)]
use crate::render_backend::{FrameId};
use crate::render_backend::DataStores;
use crate::render_task_graph::RenderTaskId;
use crate::render_task_cache::{RenderTaskCacheKeyKind, RenderTaskCacheEntryHandle, RenderTaskCacheKey, to_cache_size};
use crate::render_task::RenderTask;
use crate::renderer::{MAX_VERTEX_TEXTURE_WIDTH};
use crate::resource_cache::{ImageProperties, ImageRequest};
use crate::scene::SceneProperties;
use crate::segment::SegmentBuilder;
use std::{cmp, fmt, hash, ops, u32, usize, mem};
#[cfg(debug_assertions)]
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::storage;
use crate::texture_cache::TEXTURE_REGION_DIMENSIONS;
use crate::util::{MatrixHelpers, MaxRect, Recycler, ScaleOffset, RectHelpers, PointHelpers};
use crate::util::{clamp_to_scale_factor, pack_as_float, project_rect, raster_rect_to_device_pixels};
use crate::internal_types::{LayoutPrimitiveInfo, Filter};
use smallvec::SmallVec;

pub mod backdrop;
pub mod borders;
pub mod gradient;
pub mod image;
pub mod line_dec;
pub mod picture;
pub mod text_run;
pub mod interned;

/// Counter for unique primitive IDs for debug tracing.
#[cfg(debug_assertions)]
static NEXT_PRIM_ID: AtomicUsize = AtomicUsize::new(0);

#[cfg(debug_assertions)]
static PRIM_CHASE_ID: AtomicUsize = AtomicUsize::new(usize::MAX);

#[cfg(debug_assertions)]
pub fn register_prim_chase_id(id: PrimitiveDebugId) {
    PRIM_CHASE_ID.store(id.0, Ordering::SeqCst);
}

#[cfg(not(debug_assertions))]
pub fn register_prim_chase_id(_: PrimitiveDebugId) {
}

const MIN_BRUSH_SPLIT_AREA: f32 = 128.0 * 128.0;
pub const VECS_PER_SEGMENT: usize = 2;

const MAX_MASK_SIZE: f32 = 4096.0;

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(Debug, Copy, Clone, MallocSizeOf)]
pub struct PrimitiveOpacity {
    pub is_opaque: bool,
}

impl PrimitiveOpacity {
    pub fn opaque() -> PrimitiveOpacity {
        PrimitiveOpacity { is_opaque: true }
    }

    pub fn translucent() -> PrimitiveOpacity {
        PrimitiveOpacity { is_opaque: false }
    }

    pub fn from_alpha(alpha: f32) -> PrimitiveOpacity {
        PrimitiveOpacity {
            is_opaque: alpha >= 1.0,
        }
    }

    pub fn combine(self, other: PrimitiveOpacity) -> PrimitiveOpacity {
        PrimitiveOpacity{
            is_opaque: self.is_opaque && other.is_opaque
        }
    }
}

#[derive(Clone, Debug)]
pub struct SpaceSnapper {
    pub ref_spatial_node_index: SpatialNodeIndex,
    current_target_spatial_node_index: SpatialNodeIndex,
    snapping_transform: Option<ScaleOffset>,
    pub device_pixel_scale: DevicePixelScale,
}

impl SpaceSnapper {
    pub fn new(
        ref_spatial_node_index: SpatialNodeIndex,
        device_pixel_scale: DevicePixelScale,
    ) -> Self {
        SpaceSnapper {
            ref_spatial_node_index,
            current_target_spatial_node_index: SpatialNodeIndex::INVALID,
            snapping_transform: None,
            device_pixel_scale,
        }
    }

    pub fn new_with_target(
        ref_spatial_node_index: SpatialNodeIndex,
        target_node_index: SpatialNodeIndex,
        device_pixel_scale: DevicePixelScale,
        spatial_tree: &SpatialTree,
    ) -> Self {
        let mut snapper = SpaceSnapper {
            ref_spatial_node_index,
            current_target_spatial_node_index: SpatialNodeIndex::INVALID,
            snapping_transform: None,
            device_pixel_scale,
        };

        snapper.set_target_spatial_node(target_node_index, spatial_tree);
        snapper
    }

    pub fn set_target_spatial_node(
        &mut self,
        target_node_index: SpatialNodeIndex,
        spatial_tree: &SpatialTree,
    ) {
        if target_node_index == self.current_target_spatial_node_index {
            return
        }

        let ref_spatial_node = &spatial_tree.spatial_nodes[self.ref_spatial_node_index.0 as usize];
        let target_spatial_node = &spatial_tree.spatial_nodes[target_node_index.0 as usize];

        self.current_target_spatial_node_index = target_node_index;
        self.snapping_transform = match (ref_spatial_node.snapping_transform, target_spatial_node.snapping_transform) {
            (Some(ref ref_scale_offset), Some(ref target_scale_offset)) => {
                Some(ref_scale_offset
                    .inverse()
                    .accumulate(target_scale_offset)
                    .scale(self.device_pixel_scale.0))
            }
            _ => None,
        };
    }

    pub fn snap_rect<F>(&self, rect: &Rect<f32, F>) -> Rect<f32, F> where F: fmt::Debug {
        debug_assert!(self.current_target_spatial_node_index != SpatialNodeIndex::INVALID);
        match self.snapping_transform {
            Some(ref scale_offset) => {
                let snapped_device_rect : DeviceRect = scale_offset.map_rect(rect).snap();
                scale_offset.unmap_rect(&snapped_device_rect)
            }
            None => *rect,
        }
    }

    pub fn snap_point<F>(&self, point: &Point2D<f32, F>) -> Point2D<f32, F> where F: fmt::Debug {
        debug_assert!(self.current_target_spatial_node_index != SpatialNodeIndex::INVALID);
        match self.snapping_transform {
            Some(ref scale_offset) => {
                let snapped_device_vector : DevicePoint = scale_offset.map_point(point).snap();
                scale_offset.unmap_point(&snapped_device_vector)
            }
            None => *point,
        }
    }

    pub fn snap_size<F>(&self, size: &Size2D<f32, F>) -> Size2D<f32, F> where F: fmt::Debug {
        debug_assert!(self.current_target_spatial_node_index != SpatialNodeIndex::INVALID);
        match self.snapping_transform {
            Some(ref scale_offset) => {
                let rect = Rect::<f32, F>::new(Point2D::<f32, F>::zero(), *size);
                let snapped_device_rect : DeviceRect = scale_offset.map_rect(&rect).snap();
                scale_offset.unmap_rect(&snapped_device_rect).size
            }
            None => *size,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SpaceMapper<F, T> {
    kind: CoordinateSpaceMapping<F, T>,
    pub ref_spatial_node_index: SpatialNodeIndex,
    pub current_target_spatial_node_index: SpatialNodeIndex,
    pub bounds: Rect<f32, T>,
    visible_face: VisibleFace,
}

impl<F, T> SpaceMapper<F, T> where F: fmt::Debug {
    pub fn new(
        ref_spatial_node_index: SpatialNodeIndex,
        bounds: Rect<f32, T>,
    ) -> Self {
        SpaceMapper {
            kind: CoordinateSpaceMapping::Local,
            ref_spatial_node_index,
            current_target_spatial_node_index: ref_spatial_node_index,
            bounds,
            visible_face: VisibleFace::Front,
        }
    }

    pub fn new_with_target(
        ref_spatial_node_index: SpatialNodeIndex,
        target_node_index: SpatialNodeIndex,
        bounds: Rect<f32, T>,
        spatial_tree: &SpatialTree,
    ) -> Self {
        let mut mapper = Self::new(ref_spatial_node_index, bounds);
        mapper.set_target_spatial_node(target_node_index, spatial_tree);
        mapper
    }

    pub fn set_target_spatial_node(
        &mut self,
        target_node_index: SpatialNodeIndex,
        spatial_tree: &SpatialTree,
    ) {
        if target_node_index == self.current_target_spatial_node_index {
            return
        }

        let ref_spatial_node = &spatial_tree.spatial_nodes[self.ref_spatial_node_index.0 as usize];
        let target_spatial_node = &spatial_tree.spatial_nodes[target_node_index.0 as usize];

        self.kind = if self.ref_spatial_node_index == target_node_index {
            CoordinateSpaceMapping::Local
        } else if ref_spatial_node.coordinate_system_id == target_spatial_node.coordinate_system_id {
            let scale_offset = ref_spatial_node.content_transform
                .inverse()
                .accumulate(&target_spatial_node.content_transform);
            CoordinateSpaceMapping::ScaleOffset(scale_offset)
        } else {
            let transform = spatial_tree
                .get_relative_transform(target_node_index, self.ref_spatial_node_index)
                .into_transform()
                .with_source::<F>()
                .with_destination::<T>();
            CoordinateSpaceMapping::Transform(transform)
        };

        self.visible_face = self.kind.visible_face();
        self.current_target_spatial_node_index = target_node_index;
    }

    pub fn get_transform(&self) -> Transform3D<f32, F, T> {
        match self.kind {
            CoordinateSpaceMapping::Local => {
                Transform3D::identity()
            }
            CoordinateSpaceMapping::ScaleOffset(ref scale_offset) => {
                scale_offset.to_transform()
            }
            CoordinateSpaceMapping::Transform(transform) => {
                transform
            }
        }
    }

    pub fn unmap(&self, rect: &Rect<f32, T>) -> Option<Rect<f32, F>> {
        match self.kind {
            CoordinateSpaceMapping::Local => {
                Some(rect.cast_unit())
            }
            CoordinateSpaceMapping::ScaleOffset(ref scale_offset) => {
                Some(scale_offset.unmap_rect(rect))
            }
            CoordinateSpaceMapping::Transform(ref transform) => {
                transform.inverse_rect_footprint(rect)
            }
        }
    }

    pub fn map(&self, rect: &Rect<f32, F>) -> Option<Rect<f32, T>> {
        match self.kind {
            CoordinateSpaceMapping::Local => {
                Some(rect.cast_unit())
            }
            CoordinateSpaceMapping::ScaleOffset(ref scale_offset) => {
                Some(scale_offset.map_rect(rect))
            }
            CoordinateSpaceMapping::Transform(ref transform) => {
                match project_rect(transform, rect, &self.bounds) {
                    Some(bounds) => {
                        Some(bounds)
                    }
                    None => {
                        warn!("parent relative transform can't transform the primitive rect for {:?}", rect);
                        None
                    }
                }
            }
        }
    }

    pub fn map_vector(&self, v: Vector2D<f32, F>) -> Vector2D<f32, T> {
        match self.kind {
            CoordinateSpaceMapping::Local => {
                v.cast_unit()
            }
            CoordinateSpaceMapping::ScaleOffset(ref scale_offset) => {
                scale_offset.map_vector(&v)
            }
            CoordinateSpaceMapping::Transform(ref transform) => {
                transform.transform_vector2d(v)
            }
        }
    }
}

/// For external images, it's not possible to know the
/// UV coords of the image (or the image data itself)
/// until the render thread receives the frame and issues
/// callbacks to the client application. For external
/// images that are visible, a DeferredResolve is created
/// that is stored in the frame. This allows the render
/// thread to iterate this list and update any changed
/// texture data and update the UV rect. Any filtering
/// is handled externally for NativeTexture external
/// images.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct DeferredResolve {
    pub address: GpuCacheAddress,
    pub image_properties: ImageProperties,
    pub rendering: ImageRendering,
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "capture", derive(Serialize))]
pub struct ClipTaskIndex(pub u16);

impl ClipTaskIndex {
    pub const INVALID: ClipTaskIndex = ClipTaskIndex(0);
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, MallocSizeOf, Ord, PartialOrd)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct PictureIndex(pub usize);

impl GpuCacheHandle {
    pub fn as_int(self, gpu_cache: &GpuCache) -> i32 {
        gpu_cache.get_address(&self).as_int()
    }
}

impl GpuCacheAddress {
    pub fn as_int(self) -> i32 {
        // TODO(gw): Temporarily encode GPU Cache addresses as a single int.
        //           In the future, we can change the PrimitiveInstanceData struct
        //           to use 2x u16 for the vertex attribute instead of an i32.
        self.v as i32 * MAX_VERTEX_TEXTURE_WIDTH as i32 + self.u as i32
    }
}

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(Copy, Debug, Clone, MallocSizeOf, PartialEq)]
pub struct RectangleKey {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl RectangleKey {
    pub fn intersects(&self, other: &Self) -> bool {
        self.x < other.x + other.w
            && other.x < self.x + self.w
            && self.y < other.y + other.h
            && other.y < self.y + self.h
    }
}

impl Eq for RectangleKey {}

impl hash::Hash for RectangleKey {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.x.to_bits().hash(state);
        self.y.to_bits().hash(state);
        self.w.to_bits().hash(state);
        self.h.to_bits().hash(state);
    }
}

impl From<RectangleKey> for LayoutRect {
    fn from(key: RectangleKey) -> LayoutRect {
        LayoutRect {
            origin: LayoutPoint::new(key.x, key.y),
            size: LayoutSize::new(key.w, key.h),
        }
    }
}

impl From<RectangleKey> for WorldRect {
    fn from(key: RectangleKey) -> WorldRect {
        WorldRect {
            origin: WorldPoint::new(key.x, key.y),
            size: WorldSize::new(key.w, key.h),
        }
    }
}

impl From<LayoutRect> for RectangleKey {
    fn from(rect: LayoutRect) -> RectangleKey {
        RectangleKey {
            x: rect.origin.x,
            y: rect.origin.y,
            w: rect.size.width,
            h: rect.size.height,
        }
    }
}

impl From<PictureRect> for RectangleKey {
    fn from(rect: PictureRect) -> RectangleKey {
        RectangleKey {
            x: rect.origin.x,
            y: rect.origin.y,
            w: rect.size.width,
            h: rect.size.height,
        }
    }
}

impl From<WorldRect> for RectangleKey {
    fn from(rect: WorldRect) -> RectangleKey {
        RectangleKey {
            x: rect.origin.x,
            y: rect.origin.y,
            w: rect.size.width,
            h: rect.size.height,
        }
    }
}

/// A hashable SideOffset2D that can be used in primitive keys.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(Debug, Clone, MallocSizeOf, PartialEq)]
pub struct SideOffsetsKey {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Eq for SideOffsetsKey {}

impl hash::Hash for SideOffsetsKey {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.top.to_bits().hash(state);
        self.right.to_bits().hash(state);
        self.bottom.to_bits().hash(state);
        self.left.to_bits().hash(state);
    }
}

impl From<SideOffsetsKey> for LayoutSideOffsets {
    fn from(key: SideOffsetsKey) -> LayoutSideOffsets {
        LayoutSideOffsets::new(
            key.top,
            key.right,
            key.bottom,
            key.left,
        )
    }
}

impl<U> From<SideOffsets2D<f32, U>> for SideOffsetsKey {
    fn from(offsets: SideOffsets2D<f32, U>) -> SideOffsetsKey {
        SideOffsetsKey {
            top: offsets.top,
            right: offsets.right,
            bottom: offsets.bottom,
            left: offsets.left,
        }
    }
}

/// A hashable size for using as a key during primitive interning.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(Copy, Debug, Clone, MallocSizeOf, PartialEq)]
pub struct SizeKey {
    w: f32,
    h: f32,
}

impl Eq for SizeKey {}

impl hash::Hash for SizeKey {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.w.to_bits().hash(state);
        self.h.to_bits().hash(state);
    }
}

impl From<SizeKey> for LayoutSize {
    fn from(key: SizeKey) -> LayoutSize {
        LayoutSize::new(key.w, key.h)
    }
}

impl<U> From<Size2D<f32, U>> for SizeKey {
    fn from(size: Size2D<f32, U>) -> SizeKey {
        SizeKey {
            w: size.width,
            h: size.height,
        }
    }
}

/// A hashable vec for using as a key during primitive interning.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(Copy, Debug, Clone, MallocSizeOf, PartialEq)]
pub struct VectorKey {
    pub x: f32,
    pub y: f32,
}

impl Eq for VectorKey {}

impl hash::Hash for VectorKey {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.x.to_bits().hash(state);
        self.y.to_bits().hash(state);
    }
}

impl From<VectorKey> for LayoutVector2D {
    fn from(key: VectorKey) -> LayoutVector2D {
        LayoutVector2D::new(key.x, key.y)
    }
}

impl From<VectorKey> for WorldVector2D {
    fn from(key: VectorKey) -> WorldVector2D {
        WorldVector2D::new(key.x, key.y)
    }
}

impl From<LayoutVector2D> for VectorKey {
    fn from(vec: LayoutVector2D) -> VectorKey {
        VectorKey {
            x: vec.x,
            y: vec.y,
        }
    }
}

impl From<WorldVector2D> for VectorKey {
    fn from(vec: WorldVector2D) -> VectorKey {
        VectorKey {
            x: vec.x,
            y: vec.y,
        }
    }
}

/// A hashable point for using as a key during primitive interning.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(Debug, Copy, Clone, MallocSizeOf, PartialEq)]
pub struct PointKey {
    pub x: f32,
    pub y: f32,
}

impl Eq for PointKey {}

impl hash::Hash for PointKey {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.x.to_bits().hash(state);
        self.y.to_bits().hash(state);
    }
}

impl From<PointKey> for LayoutPoint {
    fn from(key: PointKey) -> LayoutPoint {
        LayoutPoint::new(key.x, key.y)
    }
}

impl From<LayoutPoint> for PointKey {
    fn from(p: LayoutPoint) -> PointKey {
        PointKey {
            x: p.x,
            y: p.y,
        }
    }
}

impl From<PicturePoint> for PointKey {
    fn from(p: PicturePoint) -> PointKey {
        PointKey {
            x: p.x,
            y: p.y,
        }
    }
}

impl From<WorldPoint> for PointKey {
    fn from(p: WorldPoint) -> PointKey {
        PointKey {
            x: p.x,
            y: p.y,
        }
    }
}

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(Debug, Clone, Eq, MallocSizeOf, PartialEq, Hash)]
pub struct PrimKeyCommonData {
    pub flags: PrimitiveFlags,
    pub prim_rect: RectangleKey,
}

impl From<&LayoutPrimitiveInfo> for PrimKeyCommonData {
    fn from(info: &LayoutPrimitiveInfo) -> Self {
        PrimKeyCommonData {
            flags: info.flags,
            prim_rect: info.rect.into(),
        }
    }
}

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(Debug, Clone, Eq, MallocSizeOf, PartialEq, Hash)]
pub struct PrimKey<T: MallocSizeOf> {
    pub common: PrimKeyCommonData,
    pub kind: T,
}

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(Debug, Clone, Eq, MallocSizeOf, PartialEq, Hash)]
pub struct PrimitiveKey {
    pub common: PrimKeyCommonData,
    pub kind: PrimitiveKeyKind,
}

impl PrimitiveKey {
    pub fn new(
        info: &LayoutPrimitiveInfo,
        kind: PrimitiveKeyKind,
    ) -> Self {
        PrimitiveKey {
            common: info.into(),
            kind,
        }
    }
}

impl intern::InternDebug for PrimitiveKey {}

/// The shared information for a given primitive. This is interned and retained
/// both across frames and display lists, by comparing the matching PrimitiveKey.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(MallocSizeOf)]
pub enum PrimitiveTemplateKind {
    Rectangle {
        color: PropertyBinding<ColorF>,
    },
    Clear,
}

/// Construct the primitive template data from a primitive key. This
/// is invoked when a primitive key is created and the interner
/// doesn't currently contain a primitive with this key.
impl From<PrimitiveKeyKind> for PrimitiveTemplateKind {
    fn from(kind: PrimitiveKeyKind) -> Self {
        match kind {
            PrimitiveKeyKind::Clear => {
                PrimitiveTemplateKind::Clear
            }
            PrimitiveKeyKind::Rectangle { color, .. } => {
                PrimitiveTemplateKind::Rectangle {
                    color: color.into(),
                }
            }
        }
    }
}

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(MallocSizeOf)]
pub struct PrimTemplateCommonData {
    pub flags: PrimitiveFlags,
    pub may_need_repetition: bool,
    pub prim_rect: LayoutRect,
    pub opacity: PrimitiveOpacity,
    /// The GPU cache handle for a primitive template. Since this structure
    /// is retained across display lists by interning, this GPU cache handle
    /// also remains valid, which reduces the number of updates to the GPU
    /// cache when a new display list is processed.
    pub gpu_cache_handle: GpuCacheHandle,
}

impl PrimTemplateCommonData {
    pub fn with_key_common(common: PrimKeyCommonData) -> Self {
        PrimTemplateCommonData {
            flags: common.flags,
            may_need_repetition: true,
            prim_rect: common.prim_rect.into(),
            gpu_cache_handle: GpuCacheHandle::new(),
            opacity: PrimitiveOpacity::translucent(),
        }
    }
}

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(MallocSizeOf)]
pub struct PrimTemplate<T> {
    pub common: PrimTemplateCommonData,
    pub kind: T,
}

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(MallocSizeOf)]
pub struct PrimitiveTemplate {
    pub common: PrimTemplateCommonData,
    pub kind: PrimitiveTemplateKind,
}

impl ops::Deref for PrimitiveTemplate {
    type Target = PrimTemplateCommonData;
    fn deref(&self) -> &Self::Target {
        &self.common
    }
}

impl ops::DerefMut for PrimitiveTemplate {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.common
    }
}

impl From<PrimitiveKey> for PrimitiveTemplate {
    fn from(item: PrimitiveKey) -> Self {
        PrimitiveTemplate {
            common: PrimTemplateCommonData::with_key_common(item.common),
            kind: item.kind.into(),
        }
    }
}

impl PrimitiveTemplateKind {
    /// Write any GPU blocks for the primitive template to the given request object.
    fn write_prim_gpu_blocks(
        &self,
        request: &mut GpuDataRequest,
        scene_properties: &SceneProperties,
    ) {
        match *self {
            PrimitiveTemplateKind::Clear => {
                // Opaque black with operator dest out
                request.push(PremultipliedColorF::BLACK);
            }
            PrimitiveTemplateKind::Rectangle { ref color, .. } => {
                request.push(scene_properties.resolve_color(color).premultiplied())
            }
        }
    }
}

impl PrimitiveTemplate {
    /// Update the GPU cache for a given primitive template. This may be called multiple
    /// times per frame, by each primitive reference that refers to this interned
    /// template. The initial request call to the GPU cache ensures that work is only
    /// done if the cache entry is invalid (due to first use or eviction).
    pub fn update(
        &mut self,
        frame_state: &mut FrameBuildingState,
        scene_properties: &SceneProperties,
    ) {
        if let Some(mut request) = frame_state.gpu_cache.request(&mut self.common.gpu_cache_handle) {
            self.kind.write_prim_gpu_blocks(&mut request, scene_properties);
        }

        self.opacity = match self.kind {
            PrimitiveTemplateKind::Clear => {
                PrimitiveOpacity::translucent()
            }
            PrimitiveTemplateKind::Rectangle { ref color, .. } => {
                PrimitiveOpacity::from_alpha(scene_properties.resolve_color(color).a)
            }
        };
    }
}

type PrimitiveDataHandle = intern::Handle<PrimitiveKeyKind>;

impl intern::Internable for PrimitiveKeyKind {
    type Key = PrimitiveKey;
    type StoreData = PrimitiveTemplate;
    type InternData = ();
}

impl InternablePrimitive for PrimitiveKeyKind {
    fn into_key(
        self,
        info: &LayoutPrimitiveInfo,
    ) -> PrimitiveKey {
        PrimitiveKey::new(info, self)
    }

    fn make_instance_kind(
        key: PrimitiveKey,
        data_handle: PrimitiveDataHandle,
        prim_store: &mut PrimitiveStore,
        _reference_frame_relative_offset: LayoutVector2D,
    ) -> PrimitiveInstanceKind {
        match key.kind {
            PrimitiveKeyKind::Clear => {
                PrimitiveInstanceKind::Clear {
                    data_handle
                }
            }
            PrimitiveKeyKind::Rectangle { color, .. } => {
                let color_binding_index = match color {
                    PropertyBinding::Binding(..) => {
                        prim_store.color_bindings.push(color)
                    }
                    PropertyBinding::Value(..) => ColorBindingIndex::INVALID,
                };
                PrimitiveInstanceKind::Rectangle {
                    data_handle,
                    opacity_binding_index: OpacityBindingIndex::INVALID,
                    segment_instance_index: SegmentInstanceIndex::INVALID,
                    color_binding_index,
                }
            }
        }
    }
}

// Maintains a list of opacity bindings that have been collapsed into
// the color of a single primitive. This is an important optimization
// that avoids allocating an intermediate surface for most common
// uses of opacity filters.
#[derive(Debug)]
#[cfg_attr(feature = "capture", derive(Serialize))]
pub struct OpacityBinding {
    pub bindings: Vec<PropertyBinding<f32>>,
    pub current: f32,
}

impl OpacityBinding {
    pub fn new() -> OpacityBinding {
        OpacityBinding {
            bindings: Vec::new(),
            current: 1.0,
        }
    }

    // Add a new opacity value / binding to the list
    pub fn push(&mut self, binding: PropertyBinding<f32>) {
        self.bindings.push(binding);
    }

    // Resolve the current value of each opacity binding, and
    // store that as a single combined opacity.
    pub fn update(&mut self, scene_properties: &SceneProperties) {
        let mut new_opacity = 1.0;

        for binding in &self.bindings {
            let opacity = scene_properties.resolve_float(binding);
            new_opacity = new_opacity * opacity;
        }

        self.current = new_opacity;
    }
}

#[derive(Debug, MallocSizeOf)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct VisibleMaskImageTile {
    pub tile_offset: TileOffset,
    pub tile_rect: LayoutRect,
}

#[derive(Debug)]
#[cfg_attr(feature = "capture", derive(Serialize))]
pub struct VisibleGradientTile {
    pub handle: GpuCacheHandle,
    pub local_rect: LayoutRect,
    pub local_clip_rect: LayoutRect,
}

#[derive(Debug)]
#[cfg_attr(feature = "capture", derive(Serialize))]
pub struct CachedGradientSegment {
    pub handle: RenderTaskCacheEntryHandle,
    pub local_rect: LayoutRect,
}

/// Information about how to cache a border segment,
/// along with the current render task cache entry.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(Debug, MallocSizeOf)]
pub struct BorderSegmentInfo {
    pub local_task_size: LayoutSize,
    pub cache_key: BorderSegmentCacheKey,
}

/// Represents the visibility state of a segment (wrt clip masks).
#[cfg_attr(feature = "capture", derive(Serialize))]
#[derive(Debug, Clone)]
pub enum ClipMaskKind {
    /// The segment has a clip mask, specified by the render task.
    Mask(RenderTaskId),
    /// The segment has no clip mask.
    None,
    /// The segment is made invisible / clipped completely.
    Clipped,
}

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(Debug, Clone, MallocSizeOf)]
pub struct BrushSegment {
    pub local_rect: LayoutRect,
    pub may_need_clip_mask: bool,
    pub edge_flags: EdgeAaSegmentMask,
    pub extra_data: [f32; 4],
    pub brush_flags: BrushFlags,
}

impl BrushSegment {
    pub fn new(
        local_rect: LayoutRect,
        may_need_clip_mask: bool,
        edge_flags: EdgeAaSegmentMask,
        extra_data: [f32; 4],
        brush_flags: BrushFlags,
    ) -> Self {
        Self {
            local_rect,
            may_need_clip_mask,
            edge_flags,
            extra_data,
            brush_flags,
        }
    }

    /// Write out to the clip mask instances array the correct clip mask
    /// config for this segment.
    pub fn update_clip_task(
        &self,
        clip_chain: Option<&ClipChainInstance>,
        prim_bounding_rect: WorldRect,
        root_spatial_node_index: SpatialNodeIndex,
        surface_index: SurfaceIndex,
        pic_state: &mut PictureState,
        frame_context: &FrameBuildingContext,
        frame_state: &mut FrameBuildingState,
        clip_data_store: &mut ClipDataStore,
        unclipped: &DeviceRect,
        device_pixel_scale: DevicePixelScale,
    ) -> ClipMaskKind {
        match clip_chain {
            Some(clip_chain) => {
                if !clip_chain.needs_mask ||
                   (!self.may_need_clip_mask && !clip_chain.has_non_local_clips) {
                    return ClipMaskKind::None;
                }

                let segment_world_rect = match pic_state.map_pic_to_world.map(&clip_chain.pic_clip_rect) {
                    Some(rect) => rect,
                    None => return ClipMaskKind::Clipped,
                };

                let segment_world_rect = match segment_world_rect.intersection(&prim_bounding_rect) {
                    Some(rect) => rect,
                    None => return ClipMaskKind::Clipped,
                };

                // Get a minimal device space rect, clipped to the screen that we
                // need to allocate for the clip mask, as well as interpolated
                // snap offsets.
                let device_rect = match get_clipped_device_rect(
                    unclipped,
                    &pic_state.map_raster_to_world,
                    segment_world_rect,
                    device_pixel_scale,
                ) {
                    Some(info) => info,
                    None => {
                        return ClipMaskKind::Clipped;
                    }
                };

                let (device_rect, device_pixel_scale) = adjust_mask_scale_for_max_size(device_rect, device_pixel_scale);

                let clip_task_id = RenderTask::new_mask(
                    device_rect.to_i32(),
                    clip_chain.clips_range,
                    root_spatial_node_index,
                    frame_state.clip_store,
                    frame_state.gpu_cache,
                    frame_state.resource_cache,
                    frame_state.render_tasks,
                    clip_data_store,
                    device_pixel_scale,
                    frame_context.fb_config,
                );
                let port = frame_state
                    .surfaces[surface_index.0]
                    .render_tasks
                    .unwrap_or_else(|| panic!("bug: no task for surface {:?}", surface_index))
                    .port;
                frame_state.render_tasks.add_dependency(port, clip_task_id);
                ClipMaskKind::Mask(clip_task_id)
            }
            None => {
                ClipMaskKind::Clipped
            }
        }
    }
}

#[derive(Debug)]
#[repr(C)]
struct ClipRect {
    rect: LayoutRect,
    mode: f32,
}

#[derive(Debug)]
#[repr(C)]
struct ClipCorner {
    rect: LayoutRect,
    outer_radius_x: f32,
    outer_radius_y: f32,
    inner_radius_x: f32,
    inner_radius_y: f32,
}

impl ToGpuBlocks for ClipCorner {
    fn write_gpu_blocks(&self, mut request: GpuDataRequest) {
        self.write(&mut request)
    }
}

impl ClipCorner {
    fn write(&self, request: &mut GpuDataRequest) {
        request.push(self.rect);
        request.push([
            self.outer_radius_x,
            self.outer_radius_y,
            self.inner_radius_x,
            self.inner_radius_y,
        ]);
    }

    fn uniform(rect: LayoutRect, outer_radius: f32, inner_radius: f32) -> ClipCorner {
        ClipCorner {
            rect,
            outer_radius_x: outer_radius,
            outer_radius_y: outer_radius,
            inner_radius_x: inner_radius,
            inner_radius_y: inner_radius,
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct ImageMaskData {
    /// The local size of the whole masked area.
    pub local_mask_size: LayoutSize,
}

impl ToGpuBlocks for ImageMaskData {
    fn write_gpu_blocks(&self, mut request: GpuDataRequest) {
        request.push([
            self.local_mask_size.width,
            self.local_mask_size.height,
            0.0,
            0.0,
        ]);
    }
}

#[derive(Debug)]
pub struct ClipData {
    rect: ClipRect,
    top_left: ClipCorner,
    top_right: ClipCorner,
    bottom_left: ClipCorner,
    bottom_right: ClipCorner,
}

impl ClipData {
    pub fn rounded_rect(size: LayoutSize, radii: &BorderRadius, mode: ClipMode) -> ClipData {
        // TODO(gw): For simplicity, keep most of the clip GPU structs the
        //           same as they were, even though the origin is now always
        //           zero, since they are in the clip's local space. In future,
        //           we could reduce the GPU cache size of ClipData.
        let rect = LayoutRect::new(
            LayoutPoint::zero(),
            size,
        );

        ClipData {
            rect: ClipRect {
                rect,
                mode: mode as u32 as f32,
            },
            top_left: ClipCorner {
                rect: LayoutRect::new(
                    LayoutPoint::new(rect.origin.x, rect.origin.y),
                    LayoutSize::new(radii.top_left.width, radii.top_left.height),
                ),
                outer_radius_x: radii.top_left.width,
                outer_radius_y: radii.top_left.height,
                inner_radius_x: 0.0,
                inner_radius_y: 0.0,
            },
            top_right: ClipCorner {
                rect: LayoutRect::new(
                    LayoutPoint::new(
                        rect.origin.x + rect.size.width - radii.top_right.width,
                        rect.origin.y,
                    ),
                    LayoutSize::new(radii.top_right.width, radii.top_right.height),
                ),
                outer_radius_x: radii.top_right.width,
                outer_radius_y: radii.top_right.height,
                inner_radius_x: 0.0,
                inner_radius_y: 0.0,
            },
            bottom_left: ClipCorner {
                rect: LayoutRect::new(
                    LayoutPoint::new(
                        rect.origin.x,
                        rect.origin.y + rect.size.height - radii.bottom_left.height,
                    ),
                    LayoutSize::new(radii.bottom_left.width, radii.bottom_left.height),
                ),
                outer_radius_x: radii.bottom_left.width,
                outer_radius_y: radii.bottom_left.height,
                inner_radius_x: 0.0,
                inner_radius_y: 0.0,
            },
            bottom_right: ClipCorner {
                rect: LayoutRect::new(
                    LayoutPoint::new(
                        rect.origin.x + rect.size.width - radii.bottom_right.width,
                        rect.origin.y + rect.size.height - radii.bottom_right.height,
                    ),
                    LayoutSize::new(radii.bottom_right.width, radii.bottom_right.height),
                ),
                outer_radius_x: radii.bottom_right.width,
                outer_radius_y: radii.bottom_right.height,
                inner_radius_x: 0.0,
                inner_radius_y: 0.0,
            },
        }
    }

    pub fn uniform(size: LayoutSize, radius: f32, mode: ClipMode) -> ClipData {
        // TODO(gw): For simplicity, keep most of the clip GPU structs the
        //           same as they were, even though the origin is now always
        //           zero, since they are in the clip's local space. In future,
        //           we could reduce the GPU cache size of ClipData.
        let rect = LayoutRect::new(
            LayoutPoint::zero(),
            size,
        );

        ClipData {
            rect: ClipRect {
                rect,
                mode: mode as u32 as f32,
            },
            top_left: ClipCorner::uniform(
                LayoutRect::new(
                    LayoutPoint::new(rect.origin.x, rect.origin.y),
                    LayoutSize::new(radius, radius),
                ),
                radius,
                0.0,
            ),
            top_right: ClipCorner::uniform(
                LayoutRect::new(
                    LayoutPoint::new(rect.origin.x + rect.size.width - radius, rect.origin.y),
                    LayoutSize::new(radius, radius),
                ),
                radius,
                0.0,
            ),
            bottom_left: ClipCorner::uniform(
                LayoutRect::new(
                    LayoutPoint::new(rect.origin.x, rect.origin.y + rect.size.height - radius),
                    LayoutSize::new(radius, radius),
                ),
                radius,
                0.0,
            ),
            bottom_right: ClipCorner::uniform(
                LayoutRect::new(
                    LayoutPoint::new(
                        rect.origin.x + rect.size.width - radius,
                        rect.origin.y + rect.size.height - radius,
                    ),
                    LayoutSize::new(radius, radius),
                ),
                radius,
                0.0,
            ),
        }
    }

    pub fn write(&self, request: &mut GpuDataRequest) {
        request.push(self.rect.rect);
        request.push([self.rect.mode, 0.0, 0.0, 0.0]);
        for corner in &[
            &self.top_left,
            &self.top_right,
            &self.bottom_left,
            &self.bottom_right,
        ] {
            corner.write(request);
        }
    }
}

/// A hashable descriptor for nine-patches, used by image and
/// gradient borders.
#[derive(Debug, Clone, PartialEq, Eq, Hash, MallocSizeOf)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct NinePatchDescriptor {
    pub width: i32,
    pub height: i32,
    pub slice: DeviceIntSideOffsets,
    pub fill: bool,
    pub repeat_horizontal: RepeatMode,
    pub repeat_vertical: RepeatMode,
    pub outset: SideOffsetsKey,
    pub widths: SideOffsetsKey,
}

impl IsVisible for PrimitiveKeyKind {
    // Return true if the primary primitive is visible.
    // Used to trivially reject non-visible primitives.
    // TODO(gw): Currently, primitives other than those
    //           listed here are handled before the
    //           add_primitive() call. In the future
    //           we should move the logic for all other
    //           primitive types to use this.
    fn is_visible(&self) -> bool {
        match *self {
            PrimitiveKeyKind::Clear => {
                true
            }
            PrimitiveKeyKind::Rectangle { ref color, .. } => {
                match *color {
                    PropertyBinding::Value(value) => value.a > 0,
                    PropertyBinding::Binding(..) => true,
                }
            }
        }
    }
}

impl CreateShadow for PrimitiveKeyKind {
    // Create a clone of this PrimitiveContainer, applying whatever
    // changes are necessary to the primitive to support rendering
    // it as part of the supplied shadow.
    fn create_shadow(
        &self,
        shadow: &Shadow,
    ) -> PrimitiveKeyKind {
        match *self {
            PrimitiveKeyKind::Rectangle { .. } => {
                PrimitiveKeyKind::Rectangle {
                    color: PropertyBinding::Value(shadow.color.into()),
                }
            }
            PrimitiveKeyKind::Clear => {
                panic!("bug: this prim is not supported in shadow contexts");
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct PrimitiveDebugId(pub usize);

#[derive(Clone, Debug)]
#[cfg_attr(feature = "capture", derive(Serialize))]
pub enum PrimitiveInstanceKind {
    /// Direct reference to a Picture
    Picture {
        /// Handle to the common interned data for this primitive.
        data_handle: PictureDataHandle,
        pic_index: PictureIndex,
        segment_instance_index: SegmentInstanceIndex,
    },
    /// A run of glyphs, with associated font parameters.
    TextRun {
        /// Handle to the common interned data for this primitive.
        data_handle: TextRunDataHandle,
        /// Index to the per instance scratch data for this primitive.
        run_index: TextRunIndex,
    },
    /// A line decoration. cache_handle refers to a cached render
    /// task handle, if this line decoration is not a simple solid.
    LineDecoration {
        /// Handle to the common interned data for this primitive.
        data_handle: LineDecorationDataHandle,
        // TODO(gw): For now, we need to store some information in
        //           the primitive instance that is created during
        //           prepare_prims and read during the batching pass.
        //           Once we unify the prepare_prims and batching to
        //           occur at the same time, we can remove most of
        //           the things we store here in the instance, and
        //           use them directly. This will remove cache_handle,
        //           but also the opacity, clip_task_id etc below.
        cache_handle: Option<RenderTaskCacheEntryHandle>,
    },
    NormalBorder {
        /// Handle to the common interned data for this primitive.
        data_handle: NormalBorderDataHandle,
        cache_handles: storage::Range<RenderTaskCacheEntryHandle>,
    },
    ImageBorder {
        /// Handle to the common interned data for this primitive.
        data_handle: ImageBorderDataHandle,
    },
    Rectangle {
        /// Handle to the common interned data for this primitive.
        data_handle: PrimitiveDataHandle,
        opacity_binding_index: OpacityBindingIndex,
        segment_instance_index: SegmentInstanceIndex,
        color_binding_index: ColorBindingIndex,
    },
    YuvImage {
        /// Handle to the common interned data for this primitive.
        data_handle: YuvImageDataHandle,
        segment_instance_index: SegmentInstanceIndex,
        is_compositor_surface: bool,
    },
    Image {
        /// Handle to the common interned data for this primitive.
        data_handle: ImageDataHandle,
        image_instance_index: ImageInstanceIndex,
        is_compositor_surface: bool,
    },
    LinearGradient {
        /// Handle to the common interned data for this primitive.
        data_handle: LinearGradientDataHandle,
        gradient_index: LinearGradientIndex,
    },
    RadialGradient {
        /// Handle to the common interned data for this primitive.
        data_handle: RadialGradientDataHandle,
        visible_tiles_range: GradientTileRange,
    },
    ConicGradient {
        /// Handle to the common interned data for this primitive.
        data_handle: ConicGradientDataHandle,
        visible_tiles_range: GradientTileRange,
    },
    /// Clear out a rect, used for special effects.
    Clear {
        /// Handle to the common interned data for this primitive.
        data_handle: PrimitiveDataHandle,
    },
    /// Render a portion of a specified backdrop.
    Backdrop {
        data_handle: BackdropDataHandle,
    },
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "capture", derive(Serialize))]
pub struct PrimitiveVisibilityIndex(pub u32);

impl PrimitiveVisibilityIndex {
    pub const INVALID: PrimitiveVisibilityIndex = PrimitiveVisibilityIndex(u32::MAX);
}

/// A bit mask describing which dirty regions a primitive is visible in.
/// A value of 0 means not visible in any region, while a mask of 0xffff
/// would be considered visible in all regions.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct PrimitiveVisibilityMask {
    bits: u16,
}

impl PrimitiveVisibilityMask {
    /// Construct a default mask, where no regions are considered visible
    pub fn empty() -> Self {
        PrimitiveVisibilityMask {
            bits: 0,
        }
    }

    pub fn all() -> Self {
        PrimitiveVisibilityMask {
            bits: !0,
        }
    }

    pub fn include(&mut self, other: PrimitiveVisibilityMask) {
        self.bits |= other.bits;
    }

    pub fn intersects(&self, other: PrimitiveVisibilityMask) -> bool {
        (self.bits & other.bits) != 0
    }

    /// Mark a given region index as visible
    pub fn set_visible(&mut self, region_index: usize) {
        debug_assert!(region_index < PrimitiveVisibilityMask::MAX_DIRTY_REGIONS);
        self.bits |= 1 << region_index;
    }

    /// Returns true if there are no visible regions
    pub fn is_empty(&self) -> bool {
        self.bits == 0
    }

    /// The maximum number of supported dirty regions.
    pub const MAX_DIRTY_REGIONS: usize = 8 * mem::size_of::<PrimitiveVisibilityMask>();
}

bitflags! {
    /// A set of bitflags that can be set in the visibility information
    /// for a primitive instance. This can be used to control how primitives
    /// are treated during batching.
    // TODO(gw): We should also move `is_compositor_surface` to be part of
    //           this flags struct.
    #[cfg_attr(feature = "capture", derive(Serialize))]
    pub struct PrimitiveVisibilityFlags: u16 {
        /// Implies that this primitive covers the entire picture cache slice,
        /// and can thus be dropped during batching and drawn with clear color.
        const IS_BACKDROP = 1;
    }
}

/// Information stored for a visible primitive about the visible
/// rect and associated clip information.
#[cfg_attr(feature = "capture", derive(Serialize))]
pub struct PrimitiveVisibility {
    /// The clip chain instance that was built for this primitive.
    pub clip_chain: ClipChainInstance,

    /// The current world rect, clipped to screen / dirty rect boundaries.
    // TODO(gw): This is only used by a small number of primitives.
    //           It's probably faster to not store this and recalculate
    //           on demand in those cases?
    pub clipped_world_rect: WorldRect,

    /// An index into the clip task instances array in the primitive
    /// store. If this is ClipTaskIndex::INVALID, then the primitive
    /// has no clip mask. Otherwise, it may store the offset of the
    /// global clip mask task for this primitive, or the first of
    /// a list of clip task ids (one per segment).
    pub clip_task_index: ClipTaskIndex,

    /// A set of flags that define how this primitive should be handled
    /// during batching of visibile primitives.
    pub flags: PrimitiveVisibilityFlags,

    /// A mask defining which of the dirty regions this primitive is visible in.
    pub visibility_mask: PrimitiveVisibilityMask,

    /// The current combined local clip for this primitive, from
    /// the primitive local clip above and the current clip chain.
    pub combined_local_clip_rect: LayoutRect,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "capture", derive(Serialize))]
pub struct PrimitiveInstance {
    /// Identifies the kind of primitive this
    /// instance is, and references to where
    /// the relevant information for the primitive
    /// can be found.
    pub kind: PrimitiveInstanceKind,

    /// Local space clip rect for this instance
    pub local_clip_rect: LayoutRect,

    #[cfg(debug_assertions)]
    pub id: PrimitiveDebugId,

    /// The last frame ID (of the `RenderTaskGraph`) this primitive
    /// was prepared for rendering in.
    #[cfg(debug_assertions)]
    pub prepared_frame_id: FrameId,

    /// If this primitive is visible, an index into the instance
    /// visibility scratch buffer. If not visible, INVALID.
    pub visibility_info: PrimitiveVisibilityIndex,

    /// ID of the clip chain that this primitive is clipped by.
    pub clip_chain_id: ClipChainId,
}

impl PrimitiveInstance {
    pub fn new(
        local_clip_rect: LayoutRect,
        kind: PrimitiveInstanceKind,
        clip_chain_id: ClipChainId,
    ) -> Self {
        PrimitiveInstance {
            local_clip_rect,
            kind,
            #[cfg(debug_assertions)]
            prepared_frame_id: FrameId::INVALID,
            #[cfg(debug_assertions)]
            id: PrimitiveDebugId(NEXT_PRIM_ID.fetch_add(1, Ordering::Relaxed)),
            visibility_info: PrimitiveVisibilityIndex::INVALID,
            clip_chain_id,
        }
    }

    // Reset any pre-frame state for this primitive.
    pub fn reset(&mut self) {
        self.visibility_info = PrimitiveVisibilityIndex::INVALID;
    }

    #[cfg(debug_assertions)]
    pub fn is_chased(&self) -> bool {
        PRIM_CHASE_ID.load(Ordering::SeqCst) == self.id.0
    }

    #[cfg(not(debug_assertions))]
    pub fn is_chased(&self) -> bool {
        false
    }

    pub fn uid(&self) -> intern::ItemUid {
        match &self.kind {
            PrimitiveInstanceKind::Clear { data_handle, .. } |
            PrimitiveInstanceKind::Rectangle { data_handle, .. } => {
                data_handle.uid()
            }
            PrimitiveInstanceKind::Image { data_handle, .. } => {
                data_handle.uid()
            }
            PrimitiveInstanceKind::ImageBorder { data_handle, .. } => {
                data_handle.uid()
            }
            PrimitiveInstanceKind::LineDecoration { data_handle, .. } => {
                data_handle.uid()
            }
            PrimitiveInstanceKind::LinearGradient { data_handle, .. } => {
                data_handle.uid()
            }
            PrimitiveInstanceKind::NormalBorder { data_handle, .. } => {
                data_handle.uid()
            }
            PrimitiveInstanceKind::Picture { data_handle, .. } => {
                data_handle.uid()
            }
            PrimitiveInstanceKind::RadialGradient { data_handle, .. } => {
                data_handle.uid()
            }
            PrimitiveInstanceKind::ConicGradient { data_handle, .. } => {
                data_handle.uid()
            }
            PrimitiveInstanceKind::TextRun { data_handle, .. } => {
                data_handle.uid()
            }
            PrimitiveInstanceKind::YuvImage { data_handle, .. } => {
                data_handle.uid()
            }
            PrimitiveInstanceKind::Backdrop { data_handle, .. } => {
                data_handle.uid()
            }
        }
    }
}

#[cfg_attr(feature = "capture", derive(Serialize))]
#[derive(Debug)]
pub struct SegmentedInstance {
    pub gpu_cache_handle: GpuCacheHandle,
    pub segments_range: SegmentsRange,
}

pub type GlyphKeyStorage = storage::Storage<GlyphKey>;
pub type TextRunIndex = storage::Index<TextRunPrimitive>;
pub type TextRunStorage = storage::Storage<TextRunPrimitive>;
pub type OpacityBindingIndex = storage::Index<OpacityBinding>;
pub type OpacityBindingStorage = storage::Storage<OpacityBinding>;
pub type ColorBindingIndex = storage::Index<PropertyBinding<ColorU>>;
pub type ColorBindingStorage = storage::Storage<PropertyBinding<ColorU>>;
pub type BorderHandleStorage = storage::Storage<RenderTaskCacheEntryHandle>;
pub type SegmentStorage = storage::Storage<BrushSegment>;
pub type SegmentsRange = storage::Range<BrushSegment>;
pub type SegmentInstanceStorage = storage::Storage<SegmentedInstance>;
pub type SegmentInstanceIndex = storage::Index<SegmentedInstance>;
pub type ImageInstanceStorage = storage::Storage<ImageInstance>;
pub type ImageInstanceIndex = storage::Index<ImageInstance>;
pub type GradientTileStorage = storage::Storage<VisibleGradientTile>;
pub type GradientTileRange = storage::Range<VisibleGradientTile>;
pub type LinearGradientIndex = storage::Index<LinearGradientPrimitive>;
pub type LinearGradientStorage = storage::Storage<LinearGradientPrimitive>;

/// Contains various vecs of data that is used only during frame building,
/// where we want to recycle the memory each new display list, to avoid constantly
/// re-allocating and moving memory around. Written during primitive preparation,
/// and read during batching.
#[cfg_attr(feature = "capture", derive(Serialize))]
pub struct PrimitiveScratchBuffer {
    /// Contains a list of clip mask instance parameters
    /// per segment generated.
    pub clip_mask_instances: Vec<ClipMaskKind>,

    /// List of glyphs keys that are allocated by each
    /// text run instance.
    pub glyph_keys: GlyphKeyStorage,

    /// List of render task handles for border segment instances
    /// that have been added this frame.
    pub border_cache_handles: BorderHandleStorage,

    /// A list of brush segments that have been built for this scene.
    pub segments: SegmentStorage,

    /// A list of segment ranges and GPU cache handles for prim instances
    /// that have opted into segment building. In future, this should be
    /// removed in favor of segment building during primitive interning.
    pub segment_instances: SegmentInstanceStorage,

    /// A list of visible tiles that tiled gradients use to store
    /// per-tile information.
    pub gradient_tiles: GradientTileStorage,

    /// List of the visibility information for currently visible primitives.
    pub prim_info: Vec<PrimitiveVisibility>,

    /// List of dirty regions for the cached pictures in this document, used to
    /// verify invalidation in wrench reftests. Only collected in testing.
    pub recorded_dirty_regions: Vec<RecordedDirtyRegion>,

    /// List of debug display items for rendering.
    pub debug_items: Vec<DebugItem>,
}

impl PrimitiveScratchBuffer {
    pub fn new() -> Self {
        PrimitiveScratchBuffer {
            clip_mask_instances: Vec::new(),
            glyph_keys: GlyphKeyStorage::new(0),
            border_cache_handles: BorderHandleStorage::new(0),
            segments: SegmentStorage::new(0),
            segment_instances: SegmentInstanceStorage::new(0),
            gradient_tiles: GradientTileStorage::new(0),
            recorded_dirty_regions: Vec::new(),
            debug_items: Vec::new(),
            prim_info: Vec::new(),
        }
    }

    pub fn recycle(&mut self, recycler: &mut Recycler) {
        recycler.recycle_vec(&mut self.clip_mask_instances);
        recycler.recycle_vec(&mut self.prim_info);
        self.glyph_keys.recycle(recycler);
        self.border_cache_handles.recycle(recycler);
        self.segments.recycle(recycler);
        self.segment_instances.recycle(recycler);
        self.gradient_tiles.recycle(recycler);
        recycler.recycle_vec(&mut self.debug_items);
    }

    pub fn begin_frame(&mut self) {
        // Clear the clip mask tasks for the beginning of the frame. Append
        // a single kind representing no clip mask, at the ClipTaskIndex::INVALID
        // location.
        self.clip_mask_instances.clear();
        self.clip_mask_instances.push(ClipMaskKind::None);

        self.border_cache_handles.clear();

        // TODO(gw): As in the previous code, the gradient tiles store GPU cache
        //           handles that are cleared (and thus invalidated + re-uploaded)
        //           every frame. This maintains the existing behavior, but we
        //           should fix this in the future to retain handles.
        self.gradient_tiles.clear();

        self.prim_info.clear();

        self.debug_items.clear();

        assert!(self.recorded_dirty_regions.is_empty(), "Should have sent to Renderer");
    }

    #[allow(dead_code)]
    pub fn push_debug_rect(
        &mut self,
        rect: DeviceRect,
        outer_color: ColorF,
        inner_color: ColorF,
    ) {
        self.debug_items.push(DebugItem::Rect {
            rect,
            outer_color,
            inner_color,
        });
    }

    #[allow(dead_code)]
    pub fn push_debug_string(
        &mut self,
        position: DevicePoint,
        color: ColorF,
        msg: String,
    ) {
        self.debug_items.push(DebugItem::Text {
            position,
            color,
            msg,
        });
    }
}

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(Clone, Debug)]
pub struct PrimitiveStoreStats {
    picture_count: usize,
    text_run_count: usize,
    opacity_binding_count: usize,
    image_count: usize,
    linear_gradient_count: usize,
    color_binding_count: usize,
}

impl PrimitiveStoreStats {
    pub fn empty() -> Self {
        PrimitiveStoreStats {
            picture_count: 0,
            text_run_count: 0,
            opacity_binding_count: 0,
            image_count: 0,
            linear_gradient_count: 0,
            color_binding_count: 0,
        }
    }
}

#[cfg_attr(feature = "capture", derive(Serialize))]
pub struct PrimitiveStore {
    pub pictures: Vec<PicturePrimitive>,
    pub text_runs: TextRunStorage,
    pub linear_gradients: LinearGradientStorage,

    /// A list of image instances. These are stored separately as
    /// storing them inline in the instance makes the structure bigger
    /// for other types.
    pub images: ImageInstanceStorage,

    /// List of animated opacity bindings for a primitive.
    pub opacity_bindings: OpacityBindingStorage,
    /// animated color bindings for this primitive.
    pub color_bindings: ColorBindingStorage,
}

impl PrimitiveStore {
    pub fn new(stats: &PrimitiveStoreStats) -> PrimitiveStore {
        PrimitiveStore {
            pictures: Vec::with_capacity(stats.picture_count),
            text_runs: TextRunStorage::new(stats.text_run_count),
            images: ImageInstanceStorage::new(stats.image_count),
            opacity_bindings: OpacityBindingStorage::new(stats.opacity_binding_count),
            color_bindings: ColorBindingStorage::new(stats.color_binding_count),
            linear_gradients: LinearGradientStorage::new(stats.linear_gradient_count),
        }
    }

    pub fn get_stats(&self) -> PrimitiveStoreStats {
        PrimitiveStoreStats {
            picture_count: self.pictures.len(),
            text_run_count: self.text_runs.len(),
            image_count: self.images.len(),
            opacity_binding_count: self.opacity_bindings.len(),
            linear_gradient_count: self.linear_gradients.len(),
            color_binding_count: self.color_bindings.len(),
        }
    }

    #[allow(unused)]
    pub fn print_picture_tree(&self, root: PictureIndex) {
        use crate::print_tree::PrintTree;
        let mut pt = PrintTree::new("picture tree");
        self.pictures[root.0].print(&self.pictures, root, &mut pt);
    }

    /// Destroy an existing primitive store. This is called just before
    /// a primitive store is replaced with a newly built scene.
    pub fn destroy(
        &mut self,
        retained_tiles: &mut RetainedTiles,
    ) {
        for pic in &mut self.pictures {
            pic.destroy(
                retained_tiles,
            );
        }
    }

    /// Returns the total count of primitive instances contained in pictures.
    pub fn prim_count(&self) -> usize {
        let mut prim_count = 0;
        for pic in &self.pictures {
            for cluster in &pic.prim_list.clusters {
                prim_count += cluster.prim_instances.len();
            }
        }
        prim_count
    }

    /// Update visibility pass - update each primitive visibility struct, and
    /// build the clip chain instance if appropriate.
    pub fn update_visibility(
        &mut self,
        pic_index: PictureIndex,
        parent_surface_index: SurfaceIndex,
        world_culling_rect: &WorldRect,
        frame_context: &FrameVisibilityContext,
        frame_state: &mut FrameVisibilityState,
    ) -> Option<PictureRect> {
        profile_scope!("update_visibility");
        let (mut prim_list, surface_index, apply_local_clip_rect, world_culling_rect, is_composite) = {
            let pic = &mut self.pictures[pic_index.0];
            let mut world_culling_rect = *world_culling_rect;

            let prim_list = mem::replace(&mut pic.prim_list, PrimitiveList::empty());
            let (surface_index, is_composite) = match pic.raster_config {
                Some(ref raster_config) => (raster_config.surface_index, true),
                None => (parent_surface_index, false)
            };

            match pic.raster_config {
                Some(RasterConfig { composite_mode: PictureCompositeMode::TileCache { .. }, .. }) => {
                    let mut tile_cache = pic.tile_cache.take().unwrap();
                    debug_assert!(frame_state.tile_cache.is_none());

                    // If we have a tile cache for this picture, see if any of the
                    // relative transforms have changed, which means we need to
                    // re-map the dependencies of any child primitives.
                    world_culling_rect = tile_cache.pre_update(
                        layout_rect_as_picture_rect(&pic.estimated_local_rect),
                        surface_index,
                        frame_context,
                        frame_state,
                    );

                    // Push a new surface, supplying the list of clips that should be
                    // ignored, since they are handled by clipping when drawing this surface.
                    frame_state.push_surface(
                        surface_index,
                        &tile_cache.shared_clips,
                        frame_context.spatial_tree,
                    );
                    frame_state.tile_cache = Some(tile_cache);
                }
                _ => {
                    if is_composite {
                        frame_state.push_surface(
                            surface_index,
                            &[],
                            frame_context.spatial_tree,
                        );
                    }
                }
            }

            (prim_list, surface_index, pic.apply_local_clip_rect, world_culling_rect, is_composite)
        };

        let surface = &frame_context.surfaces[surface_index.0 as usize];

        let mut map_local_to_surface = surface
            .map_local_to_surface
            .clone();

        let map_surface_to_world = SpaceMapper::new_with_target(
            ROOT_SPATIAL_NODE_INDEX,
            surface.surface_spatial_node_index,
            frame_context.global_screen_world_rect,
            frame_context.spatial_tree,
        );

        let mut surface_rect = PictureRect::zero();

        for cluster in &mut prim_list.clusters {
            profile_scope!("cluster");
            // Get the cluster and see if is visible
            if !cluster.flags.contains(ClusterFlags::IS_VISIBLE) {
                // Each prim instance must have reset called each frame, to clear
                // indices into various scratch buffers. If this doesn't occur,
                // the primitive may incorrectly be considered visible, which can
                // cause unexpected conditions to occur later during the frame.
                // Primitive instances are normally reset in the main loop below,
                // but we must also reset them in the rare case that the cluster
                // visibility has changed (due to an invalid transform and/or
                // backface visibility changing for this cluster).
                // TODO(gw): This is difficult to test for in CI - as a follow up,
                //           we should add a debug flag that validates the prim
                //           instance is always reset every frame to catch similar
                //           issues in future.
                for prim_instance in &mut cluster.prim_instances {
                    prim_instance.reset();
                }
                continue;
            }

            map_local_to_surface.set_target_spatial_node(
                cluster.spatial_node_index,
                frame_context.spatial_tree,
            );

            for prim_instance in &mut cluster.prim_instances {
                prim_instance.reset();

                if prim_instance.is_chased() {
                    #[cfg(debug_assertions)] // needed for ".id" part
                    println!("\tpreparing {:?} in {:?}", prim_instance.id, pic_index);
                    println!("\t{:?}", prim_instance.kind);
                }

                let (is_passthrough, prim_local_rect, prim_shadowed_rect) = match prim_instance.kind {
                    PrimitiveInstanceKind::Picture { pic_index, .. } => {
                        if !self.pictures[pic_index.0].is_visible() {
                            continue;
                        }

                        frame_state.clip_chain_stack.push_clip(
                            prim_instance.clip_chain_id,
                            frame_state.clip_store,
                        );

                        let pic_surface_rect = self.update_visibility(
                            pic_index,
                            surface_index,
                            &world_culling_rect,
                            frame_context,
                            frame_state,
                        );

                        frame_state.clip_chain_stack.pop_clip();

                        let pic = &self.pictures[pic_index.0];

                        if prim_instance.is_chased() && pic.estimated_local_rect != pic.precise_local_rect {
                            println!("\testimate {:?} adjusted to {:?}", pic.estimated_local_rect, pic.precise_local_rect);
                        }

                        let mut shadow_rect = pic.precise_local_rect;
                        match pic.raster_config {
                            Some(ref rc) => match rc.composite_mode {
                                // If we have a drop shadow filter, we also need to include the shadow in
                                // our shadowed local rect for the purpose of calculating the size of the
                                // picture.
                                PictureCompositeMode::Filter(Filter::DropShadows(ref shadows)) => {
                                    for shadow in shadows {
                                        shadow_rect = shadow_rect.union(&pic.precise_local_rect.translate(shadow.offset));
                                    }
                                }
                                _ => {}
                            }
                            None => {
                                // If the primitive does not have its own raster config, we need to
                                // propogate the surface rect calculation to the parent.
                                if let Some(ref rect) = pic_surface_rect {
                                    surface_rect = surface_rect.union(rect);
                                }
                            }
                        }

                        (pic.raster_config.is_none(), pic.precise_local_rect, shadow_rect)
                    }
                    _ => {
                        let prim_data = &frame_state.data_stores.as_common_data(&prim_instance);

                        (false, prim_data.prim_rect, prim_data.prim_rect)
                    }
                };

                if is_passthrough {
                    let vis_index = PrimitiveVisibilityIndex(frame_state.scratch.prim_info.len() as u32);

                    frame_state.scratch.prim_info.push(
                        PrimitiveVisibility {
                            clipped_world_rect: WorldRect::max_rect(),
                            clip_chain: ClipChainInstance::empty(),
                            clip_task_index: ClipTaskIndex::INVALID,
                            combined_local_clip_rect: LayoutRect::zero(),
                            visibility_mask: PrimitiveVisibilityMask::empty(),
                            flags: PrimitiveVisibilityFlags::empty(),
                        }
                    );

                    prim_instance.visibility_info = vis_index;
                } else {
                    if prim_local_rect.size.width <= 0.0 || prim_local_rect.size.height <= 0.0 {
                        if prim_instance.is_chased() {
                            println!("\tculled for zero local rectangle");
                        }
                        continue;
                    }

                    // Inflate the local rect for this primitive by the inflation factor of
                    // the picture context and include the shadow offset. This ensures that
                    // even if the primitive itself is not visible, any effects from the
                    // blur radius or shadow will be correctly taken into account.
                    let inflation_factor = surface.inflation_factor;
                    let local_rect = prim_shadowed_rect
                        .inflate(inflation_factor, inflation_factor)
                        .intersection(&prim_instance.local_clip_rect);
                    let local_rect = match local_rect {
                        Some(local_rect) => local_rect,
                        None => {
                            if prim_instance.is_chased() {
                                println!("\tculled for being out of the local clip rectangle: {:?}",
                                         prim_instance.local_clip_rect);
                            }
                            continue;
                        }
                    };

                    // Include the clip chain for this primitive in the current stack.
                    frame_state.clip_chain_stack.push_clip(
                        prim_instance.clip_chain_id,
                        frame_state.clip_store,
                    );

                    frame_state.clip_store.set_active_clips(
                        prim_instance.local_clip_rect,
                        cluster.spatial_node_index,
                        frame_state.clip_chain_stack.current_clips_array(),
                        &frame_context.spatial_tree,
                        &frame_state.data_stores.clip,
                    );

                    let clip_chain = frame_state
                        .clip_store
                        .build_clip_chain_instance(
                            local_rect,
                            &map_local_to_surface,
                            &map_surface_to_world,
                            &frame_context.spatial_tree,
                            frame_state.gpu_cache,
                            frame_state.resource_cache,
                            surface.device_pixel_scale,
                            &world_culling_rect,
                            &mut frame_state.data_stores.clip,
                            true,
                            prim_instance.is_chased(),
                        );

                    // Primitive visibility flags default to empty, but may be supplied
                    // by the `update_prim_dependencies` method below when picture caching
                    // is active.
                    let mut vis_flags = PrimitiveVisibilityFlags::empty();

                    if let Some(ref mut tile_cache) = frame_state.tile_cache {
                        // TODO(gw): Refactor how tile_cache is stored in frame_state
                        //           so that we can pass frame_state directly to
                        //           update_prim_dependencies, rather than splitting borrows.
                        match tile_cache.update_prim_dependencies(
                            prim_instance,
                            cluster.spatial_node_index,
                            clip_chain.as_ref(),
                            prim_local_rect,
                            frame_context,
                            frame_state.data_stores,
                            frame_state.clip_store,
                            &self.pictures,
                            frame_state.resource_cache,
                            &self.opacity_bindings,
                            &self.color_bindings,
                            &self.images,
                            &frame_state.surface_stack,
                            &mut frame_state.composite_state,
                        ) {
                            Some(flags) => {
                                vis_flags = flags;
                            }
                            None => {
                                prim_instance.visibility_info = PrimitiveVisibilityIndex::INVALID;
                                // Ensure the primitive clip is popped - perhaps we can use
                                // some kind of scope to do this automatically in future.
                                frame_state.clip_chain_stack.pop_clip();
                                continue;
                            }
                        }
                    }

                    // Ensure the primitive clip is popped
                    frame_state.clip_chain_stack.pop_clip();

                    let clip_chain = match clip_chain {
                        Some(clip_chain) => clip_chain,
                        None => {
                            if prim_instance.is_chased() {
                                println!("\tunable to build the clip chain, skipping");
                            }
                            prim_instance.visibility_info = PrimitiveVisibilityIndex::INVALID;
                            continue;
                        }
                    };

                    if prim_instance.is_chased() {
                        println!("\teffective clip chain from {:?} {}",
                                 clip_chain.clips_range,
                                 if apply_local_clip_rect { "(applied)" } else { "" },
                        );
                        println!("\tpicture rect {:?} @{:?}",
                                 clip_chain.pic_clip_rect,
                                 clip_chain.pic_spatial_node_index,
                        );
                    }

                    // Check if the clip bounding rect (in pic space) is visible on screen
                    // This includes both the prim bounding rect + local prim clip rect!
                    let world_rect = match map_surface_to_world.map(&clip_chain.pic_clip_rect) {
                        Some(world_rect) => world_rect,
                        None => {
                            continue;
                        }
                    };

                    let clipped_world_rect = match world_rect.intersection(&world_culling_rect) {
                        Some(rect) => rect,
                        None => {
                            continue;
                        }
                    };

                    let combined_local_clip_rect = if apply_local_clip_rect {
                        clip_chain.local_clip_rect
                    } else {
                        prim_instance.local_clip_rect
                    };

                    if combined_local_clip_rect.size.is_empty() {
                        debug_assert!(combined_local_clip_rect.size.width >= 0.0 &&
                            combined_local_clip_rect.size.height >= 0.0);
                        if prim_instance.is_chased() {
                            println!("\tculled for zero local clip rectangle");
                        }
                        prim_instance.visibility_info = PrimitiveVisibilityIndex::INVALID;
                        continue;
                    }

                    // Include the visible area for primitive, including any shadows, in
                    // the area affected by the surface.
                    match combined_local_clip_rect.intersection(&local_rect) {
                        Some(visible_rect) => {
                            if let Some(rect) = map_local_to_surface.map(&visible_rect) {
                                surface_rect = surface_rect.union(&rect);
                            }
                        }
                        None => {
                            if prim_instance.is_chased() {
                                println!("\tculled for zero visible rectangle");
                            }
                            prim_instance.visibility_info = PrimitiveVisibilityIndex::INVALID;
                            continue;
                        }
                    }

                    // When the debug display is enabled, paint a colored rectangle around each
                    // primitive.
                    if frame_context.debug_flags.contains(::api::DebugFlags::PRIMITIVE_DBG) {
                        let debug_color = match prim_instance.kind {
                            PrimitiveInstanceKind::Picture { .. } => ColorF::TRANSPARENT,
                            PrimitiveInstanceKind::TextRun { .. } => debug_colors::RED,
                            PrimitiveInstanceKind::LineDecoration { .. } => debug_colors::PURPLE,
                            PrimitiveInstanceKind::NormalBorder { .. } |
                            PrimitiveInstanceKind::ImageBorder { .. } => debug_colors::ORANGE,
                            PrimitiveInstanceKind::Rectangle { .. } => ColorF { r: 0.8, g: 0.8, b: 0.8, a: 0.5 },
                            PrimitiveInstanceKind::YuvImage { .. } => debug_colors::BLUE,
                            PrimitiveInstanceKind::Image { .. } => debug_colors::BLUE,
                            PrimitiveInstanceKind::LinearGradient { .. } => debug_colors::PINK,
                            PrimitiveInstanceKind::RadialGradient { .. } => debug_colors::PINK,
                            PrimitiveInstanceKind::ConicGradient { .. } => debug_colors::PINK,
                            PrimitiveInstanceKind::Clear { .. } => debug_colors::CYAN,
                            PrimitiveInstanceKind::Backdrop { .. } => debug_colors::MEDIUMAQUAMARINE,
                        };
                        if debug_color.a != 0.0 {
                            let debug_rect = clipped_world_rect * frame_context.global_device_pixel_scale;
                            frame_state.scratch.push_debug_rect(debug_rect, debug_color, debug_color.scale_alpha(0.5));
                        }
                    } else if frame_context.debug_flags.contains(::api::DebugFlags::OBSCURE_IMAGES) {
                        let is_image = matches!(
                            prim_instance.kind,
                            PrimitiveInstanceKind::Image { .. } | PrimitiveInstanceKind::YuvImage { .. }
                        );
                        if is_image {
                            // We allow "small" images, since they're generally UI elements.
                            let rect = clipped_world_rect * frame_context.global_device_pixel_scale;
                            if rect.size.width > 70.0 && rect.size.height > 70.0 {
                                frame_state.scratch.push_debug_rect(rect, debug_colors::PURPLE, debug_colors::PURPLE);
                            }
                        }
                    }

                    let vis_index = PrimitiveVisibilityIndex(frame_state.scratch.prim_info.len() as u32);
                    if prim_instance.is_chased() {
                        println!("\tvisible {:?} with {:?}", vis_index, combined_local_clip_rect);
                    }

                    frame_state.scratch.prim_info.push(
                        PrimitiveVisibility {
                            clipped_world_rect,
                            clip_chain,
                            clip_task_index: ClipTaskIndex::INVALID,
                            combined_local_clip_rect,
                            visibility_mask: PrimitiveVisibilityMask::empty(),
                            flags: vis_flags,
                        }
                    );

                    prim_instance.visibility_info = vis_index;

                    self.request_resources_for_prim(
                        prim_instance,
                        cluster.spatial_node_index,
                        clipped_world_rect,
                        frame_context,
                        frame_state,
                    );
                }
            }
        }

        // Similar to above, pop either the clip chain or root entry off the current clip stack.
        if is_composite {
            frame_state.pop_surface();
        }

        let pic = &mut self.pictures[pic_index.0];
        pic.prim_list = prim_list;

        // If the local rect changed (due to transforms in child primitives) then
        // invalidate the GPU cache location to re-upload the new local rect
        // and stretch size. Drop shadow filters also depend on the local rect
        // size for the extra GPU cache data handle.
        // TODO(gw): In future, if we support specifying a flag which gets the
        //           stretch size from the segment rect in the shaders, we can
        //           remove this invalidation here completely.
        if let Some(ref rc) = pic.raster_config {
            // Inflate the local bounding rect if required by the filter effect.
            // This inflaction factor is to be applied to the surface itself.
            if pic.options.inflate_if_required {
                // The picture's local rect is calculated as the union of the
                // snapped primitive rects, which should result in a snapped
                // local rect, unless it was inflated. This is also done during
                // surface configuration when calculating the picture's
                // estimated local rect.
                let snap_pic_to_raster = SpaceSnapper::new_with_target(
                    surface.raster_spatial_node_index,
                    pic.spatial_node_index,
                    surface.device_pixel_scale,
                    frame_context.spatial_tree,
                );

                surface_rect = rc.composite_mode.inflate_picture_rect(surface_rect, surface.scale_factors);
                surface_rect = snap_pic_to_raster.snap_rect(&surface_rect);
            }

            // Layout space for the picture is picture space from the
            // perspective of its child primitives.
            let pic_local_rect = surface_rect * Scale::new(1.0);
            if pic.precise_local_rect != pic_local_rect {
                match rc.composite_mode {
                    PictureCompositeMode::Filter(Filter::DropShadows(..)) => {
                        for handle in &pic.extra_gpu_data_handles {
                            frame_state.gpu_cache.invalidate(handle);
                        }
                    }
                    _ => {}
                }
                // Invalidate any segments built for this picture, since the local
                // rect has changed.
                pic.segments_are_valid = false;
                pic.precise_local_rect = pic_local_rect;
            }

            if let PictureCompositeMode::TileCache { .. } = rc.composite_mode {
                let mut tile_cache = frame_state.tile_cache.take().unwrap();

                // Build the dirty region(s) for this tile cache.
                tile_cache.post_update(
                    frame_context,
                    frame_state,
                );

                pic.tile_cache = Some(tile_cache);
            }

            None
        } else {
            let parent_surface = &frame_context.surfaces[parent_surface_index.0 as usize];
            let map_surface_to_parent_surface = SpaceMapper::new_with_target(
                parent_surface.surface_spatial_node_index,
                surface.surface_spatial_node_index,
                PictureRect::max_rect(),
                frame_context.spatial_tree,
            );
            map_surface_to_parent_surface.map(&surface_rect)
        }
    }

    fn request_resources_for_prim(
        &mut self,
        prim_instance: &mut PrimitiveInstance,
        prim_spatial_node_index: SpatialNodeIndex,
        prim_world_rect: WorldRect,
        frame_context: &FrameVisibilityContext,
        frame_state: &mut FrameVisibilityState,
    ) {
        profile_scope!("request_resources_for_prim");
        match prim_instance.kind {
            PrimitiveInstanceKind::TextRun { .. } => {
                // Text runs can't request resources early here, as we don't
                // know until TileCache::post_update() whether we are drawing
                // on an opaque surface.
                // TODO(gw): We might be able to detect simple cases of this earlier,
                //           during the picture traversal. But it's probably not worth it?
            }
            PrimitiveInstanceKind::Image { data_handle, image_instance_index, .. } => {
                let prim_data = &mut frame_state.data_stores.image[data_handle];
                let common_data = &mut prim_data.common;
                let image_data = &mut prim_data.kind;
                let image_instance = &mut self.images[image_instance_index];

                let image_properties = frame_state
                    .resource_cache
                    .get_image_properties(image_data.key);

                let request = ImageRequest {
                    key: image_data.key,
                    rendering: image_data.image_rendering,
                    tile: None,
                };

                match image_properties {
                    Some(ImageProperties { tiling: None, .. }) => {

                        frame_state.resource_cache.request_image(
                            request,
                            frame_state.gpu_cache,
                        );
                    }
                    Some(ImageProperties { tiling: Some(tile_size), visible_rect, .. }) => {
                        image_instance.visible_tiles.clear();
                        // TODO: rename the blob's visible_rect into something that doesn't conflict
                        // with the terminology we use during culling since it's not really the same
                        // thing.
                        let active_rect = visible_rect;

                        // Tighten the clip rect because decomposing the repeated image can
                        // produce primitives that are partially covering the original image
                        // rect and we want to clip these extra parts out.
                        let prim_info = &frame_state.scratch.prim_info[prim_instance.visibility_info.0 as usize];
                        let tight_clip_rect = prim_info
                            .combined_local_clip_rect
                            .intersection(&common_data.prim_rect).unwrap();
                        image_instance.tight_local_clip_rect = tight_clip_rect;

                        let map_local_to_world = SpaceMapper::new_with_target(
                            ROOT_SPATIAL_NODE_INDEX,
                            prim_spatial_node_index,
                            frame_context.global_screen_world_rect,
                            frame_context.spatial_tree,
                        );

                        let visible_rect = compute_conservative_visible_rect(
                            &tight_clip_rect,
                            prim_world_rect,
                            &map_local_to_world,
                        );

                        let base_edge_flags = edge_flags_for_tile_spacing(&image_data.tile_spacing);

                        let stride = image_data.stretch_size + image_data.tile_spacing;

                        // We are performing the decomposition on the CPU here, no need to
                        // have it in the shader.
                        common_data.may_need_repetition = false;

                        let repetitions = image_tiling::repetitions(
                            &common_data.prim_rect,
                            &visible_rect,
                            stride,
                        );

                        for Repetition { origin, edge_flags } in repetitions {
                            let edge_flags = base_edge_flags | edge_flags;

                            let layout_image_rect = LayoutRect {
                                origin,
                                size: image_data.stretch_size,
                            };

                            let tiles = image_tiling::tiles(
                                &layout_image_rect,
                                &visible_rect,
                                &active_rect,
                                tile_size as i32,
                            );

                            for tile in tiles {
                                frame_state.resource_cache.request_image(
                                    request.with_tile(tile.offset),
                                    frame_state.gpu_cache,
                                );

                                image_instance.visible_tiles.push(VisibleImageTile {
                                    tile_offset: tile.offset,
                                    edge_flags: tile.edge_flags & edge_flags,
                                    local_rect: tile.rect,
                                    local_clip_rect: tight_clip_rect,
                                });
                            }
                        }

                        if image_instance.visible_tiles.is_empty() {
                            // Mark as invisible
                            prim_instance.visibility_info = PrimitiveVisibilityIndex::INVALID;
                        }
                    }
                    None => {}
                }
            }
            PrimitiveInstanceKind::ImageBorder { data_handle, .. } => {
                let prim_data = &mut frame_state.data_stores.image_border[data_handle];
                prim_data.kind.request_resources(
                    frame_state.resource_cache,
                    frame_state.gpu_cache,
                );
            }
            PrimitiveInstanceKind::YuvImage { data_handle, .. } => {
                let prim_data = &mut frame_state.data_stores.yuv_image[data_handle];
                prim_data.kind.request_resources(
                    frame_state.resource_cache,
                    frame_state.gpu_cache,
                );
            }
            _ => {}
        }
    }

    pub fn get_opacity_binding(
        &self,
        opacity_binding_index: OpacityBindingIndex,
    ) -> f32 {
        if opacity_binding_index == OpacityBindingIndex::INVALID {
            1.0
        } else {
            self.opacity_bindings[opacity_binding_index].current
        }
    }

    // Internal method that retrieves the primitive index of a primitive
    // that can be the target for collapsing parent opacity filters into.
    fn get_opacity_collapse_prim(
        &self,
        pic_index: PictureIndex,
    ) -> Option<PictureIndex> {
        let pic = &self.pictures[pic_index.0];

        // We can only collapse opacity if there is a single primitive, otherwise
        // the opacity needs to be applied to the primitives as a group.
        if pic.prim_list.clusters.len() != 1 {
            return None;
        }

        let cluster = &pic.prim_list.clusters[0];
        if cluster.prim_instances.len() != 1 {
            return None;
        }

        let prim_instance = &cluster.prim_instances[0];

        // For now, we only support opacity collapse on solid rects and images.
        // This covers the most common types of opacity filters that can be
        // handled by this optimization. In the future, we can easily extend
        // this to other primitives, such as text runs and gradients.
        match prim_instance.kind {
            // If we find a single rect or image, we can use that
            // as the primitive to collapse the opacity into.
            PrimitiveInstanceKind::Rectangle { .. } |
            PrimitiveInstanceKind::Image { .. } => {
                return Some(pic_index);
            }
            PrimitiveInstanceKind::Clear { .. } |
            PrimitiveInstanceKind::TextRun { .. } |
            PrimitiveInstanceKind::NormalBorder { .. } |
            PrimitiveInstanceKind::ImageBorder { .. } |
            PrimitiveInstanceKind::YuvImage { .. } |
            PrimitiveInstanceKind::LinearGradient { .. } |
            PrimitiveInstanceKind::RadialGradient { .. } |
            PrimitiveInstanceKind::ConicGradient { .. } |
            PrimitiveInstanceKind::LineDecoration { .. } |
            PrimitiveInstanceKind::Backdrop { .. } => {
                // These prims don't support opacity collapse
            }
            PrimitiveInstanceKind::Picture { pic_index, .. } => {
                let pic = &self.pictures[pic_index.0];

                // If we encounter a picture that is a pass-through
                // (i.e. no composite mode), then we can recurse into
                // that to try and find a primitive to collapse to.
                if pic.requested_composite_mode.is_none() {
                    return self.get_opacity_collapse_prim(pic_index);
                }
            }
        }

        None
    }

    // Apply any optimizations to drawing this picture. Currently,
    // we just support collapsing pictures with an opacity filter
    // by pushing that opacity value into the color of a primitive
    // if that picture contains one compatible primitive.
    pub fn optimize_picture_if_possible(
        &mut self,
        pic_index: PictureIndex,
    ) {
        // Only handle opacity filters for now.
        let binding = match self.pictures[pic_index.0].requested_composite_mode {
            Some(PictureCompositeMode::Filter(Filter::Opacity(binding, _))) => {
                binding
            }
            _ => {
                return;
            }
        };

        // See if this picture contains a single primitive that supports
        // opacity collapse.
        match self.get_opacity_collapse_prim(pic_index) {
            Some(pic_index) => {
                let pic = &mut self.pictures[pic_index.0];
                let prim_instance = &mut pic.prim_list.clusters[0].prim_instances[0];
                match prim_instance.kind {
                    PrimitiveInstanceKind::Image { image_instance_index, .. } => {
                        let image_instance = &mut self.images[image_instance_index];
                        // By this point, we know we should only have found a primitive
                        // that supports opacity collapse.
                        if image_instance.opacity_binding_index == OpacityBindingIndex::INVALID {
                            image_instance.opacity_binding_index = self.opacity_bindings.push(OpacityBinding::new());
                        }
                        let opacity_binding = &mut self.opacity_bindings[image_instance.opacity_binding_index];
                        opacity_binding.push(binding);
                    }
                    PrimitiveInstanceKind::Rectangle { ref mut opacity_binding_index, .. } => {
                        // By this point, we know we should only have found a primitive
                        // that supports opacity collapse.
                        if *opacity_binding_index == OpacityBindingIndex::INVALID {
                            *opacity_binding_index = self.opacity_bindings.push(OpacityBinding::new());
                        }
                        let opacity_binding = &mut self.opacity_bindings[*opacity_binding_index];
                        opacity_binding.push(binding);
                    }
                    _ => {
                        unreachable!();
                    }
                }
            }
            None => {
                return;
            }
        }

        // The opacity filter has been collapsed, so mark this picture
        // as a pass though. This means it will no longer allocate an
        // intermediate surface or incur an extra blend / blit. Instead,
        // the collapsed primitive will be drawn directly into the
        // parent picture.
        self.pictures[pic_index.0].requested_composite_mode = None;
    }

    fn prepare_prim_for_render(
        &mut self,
        prim_instance: &mut PrimitiveInstance,
        prim_spatial_node_index: SpatialNodeIndex,
        pic_context: &PictureContext,
        pic_state: &mut PictureState,
        frame_context: &FrameBuildingContext,
        frame_state: &mut FrameBuildingState,
        plane_split_anchor: PlaneSplitAnchor,
        data_stores: &mut DataStores,
        scratch: &mut PrimitiveScratchBuffer,
        tile_cache_log: &mut TileCacheLogger,
    ) -> bool {
        profile_scope!("prepare_prim_for_render");
        // If we have dependencies, we need to prepare them first, in order
        // to know the actual rect of this primitive.
        // For example, scrolling may affect the location of an item in
        // local space, which may force us to render this item on a larger
        // picture target, if being composited.
        let pic_info = {
            match prim_instance.kind {
                PrimitiveInstanceKind::Picture { pic_index ,.. } => {
                    let pic = &mut self.pictures[pic_index.0];

                    let clipped_prim_bounding_rect = scratch
                        .prim_info[prim_instance.visibility_info.0 as usize]
                        .clipped_world_rect;

                    match pic.take_context(
                        pic_index,
                        clipped_prim_bounding_rect,
                        pic_context.surface_spatial_node_index,
                        pic_context.raster_spatial_node_index,
                        pic_context.surface_index,
                        &pic_context.subpixel_mode,
                        frame_state,
                        frame_context,
                        scratch,
                        tile_cache_log,
                    ) {
                        Some(info) => Some(info),
                        None => {
                            if prim_instance.is_chased() {
                                println!("\tculled for carrying an invisible composite filter");
                            }

                            prim_instance.visibility_info = PrimitiveVisibilityIndex::INVALID;

                            return false;
                        }
                    }
                }
                PrimitiveInstanceKind::TextRun { .. } |
                PrimitiveInstanceKind::Rectangle { .. } |
                PrimitiveInstanceKind::LineDecoration { .. } |
                PrimitiveInstanceKind::NormalBorder { .. } |
                PrimitiveInstanceKind::ImageBorder { .. } |
                PrimitiveInstanceKind::YuvImage { .. } |
                PrimitiveInstanceKind::Image { .. } |
                PrimitiveInstanceKind::LinearGradient { .. } |
                PrimitiveInstanceKind::RadialGradient { .. } |
                PrimitiveInstanceKind::ConicGradient { .. } |
                PrimitiveInstanceKind::Clear { .. } |
                PrimitiveInstanceKind::Backdrop { .. } => {
                    None
                }
            }
        };

        let is_passthrough = match pic_info {
            Some((pic_context_for_children, mut pic_state_for_children, mut prim_list)) => {
                let is_passthrough = pic_context_for_children.is_passthrough;

                self.prepare_primitives(
                    &mut prim_list,
                    &pic_context_for_children,
                    &mut pic_state_for_children,
                    frame_context,
                    frame_state,
                    data_stores,
                    scratch,
                    tile_cache_log,
                );

                // Restore the dependencies (borrow check dance)
                self.pictures[pic_context_for_children.pic_index.0]
                    .restore_context(
                        pic_context.surface_index,
                        prim_list,
                        pic_context_for_children,
                        pic_state_for_children,
                        frame_state,
                    );

                is_passthrough
            }
            None => {
                false
            }
        };

        let prim_rect = data_stores.get_local_prim_rect(
            prim_instance,
            self,
        );

        if !is_passthrough {
            prim_instance.update_clip_task(
                &prim_rect.origin,
                prim_spatial_node_index,
                pic_context.raster_spatial_node_index,
                pic_context,
                pic_state,
                frame_context,
                frame_state,
                self,
                data_stores,
                scratch,
            );

            if prim_instance.is_chased() {
                println!("\tconsidered visible and ready with local pos {:?}", prim_rect.origin);
            }
        }

        #[cfg(debug_assertions)]
        {
            prim_instance.prepared_frame_id = frame_state.render_tasks.frame_id();
        }

        self.prepare_interned_prim_for_render(
            prim_instance,
            prim_spatial_node_index,
            plane_split_anchor,
            pic_context,
            pic_state,
            frame_context,
            frame_state,
            data_stores,
            scratch,
        );

        true
    }

    pub fn prepare_primitives(
        &mut self,
        prim_list: &mut PrimitiveList,
        pic_context: &PictureContext,
        pic_state: &mut PictureState,
        frame_context: &FrameBuildingContext,
        frame_state: &mut FrameBuildingState,
        data_stores: &mut DataStores,
        scratch: &mut PrimitiveScratchBuffer,
        tile_cache_log: &mut TileCacheLogger,
    ) {
        profile_scope!("prepare_primitives");
        for (cluster_index, cluster) in prim_list.clusters.iter_mut().enumerate() {
            profile_scope!("cluster");
            pic_state.map_local_to_pic.set_target_spatial_node(
                cluster.spatial_node_index,
                frame_context.spatial_tree,
            );

            for (prim_instance_index, prim_instance) in cluster.prim_instances.iter_mut().enumerate() {
                if prim_instance.visibility_info == PrimitiveVisibilityIndex::INVALID {
                    continue;
                }

                // The original clipped world rect was calculated during the initial visibility pass.
                // However, it's possible that the dirty rect has got smaller, if tiles were not
                // dirty. Intersecting with the dirty rect here eliminates preparing any primitives
                // outside the dirty rect, and reduces the size of any off-screen surface allocations
                // for clip masks / render tasks that we make.
                {
                    let visibility_info = &mut scratch.prim_info[prim_instance.visibility_info.0 as usize];
                    let dirty_region = frame_state.current_dirty_region();

                    for dirty_region in &dirty_region.dirty_rects {
                        if visibility_info.clipped_world_rect.intersects(&dirty_region.world_rect) {
                            visibility_info.visibility_mask.include(dirty_region.visibility_mask);
                        }
                    }

                    if visibility_info.visibility_mask.is_empty() {
                        prim_instance.visibility_info = PrimitiveVisibilityIndex::INVALID;
                        continue;
                    }
                }

                let plane_split_anchor = PlaneSplitAnchor::new(cluster_index, prim_instance_index);

                if self.prepare_prim_for_render(
                    prim_instance,
                    cluster.spatial_node_index,
                    pic_context,
                    pic_state,
                    frame_context,
                    frame_state,
                    plane_split_anchor,
                    data_stores,
                    scratch,
                    tile_cache_log,
                ) {
                    frame_state.profile_counters.visible_primitives.inc();
                }
            }
        }
    }

    /// Prepare an interned primitive for rendering, by requesting
    /// resources, render tasks etc. This is equivalent to the
    /// prepare_prim_for_render_inner call for old style primitives.
    fn prepare_interned_prim_for_render(
        &mut self,
        prim_instance: &mut PrimitiveInstance,
        prim_spatial_node_index: SpatialNodeIndex,
        plane_split_anchor: PlaneSplitAnchor,
        pic_context: &PictureContext,
        pic_state: &mut PictureState,
        frame_context: &FrameBuildingContext,
        frame_state: &mut FrameBuildingState,
        data_stores: &mut DataStores,
        scratch: &mut PrimitiveScratchBuffer,
    ) {
        let is_chased = prim_instance.is_chased();
        let device_pixel_scale = frame_state.surfaces[pic_context.surface_index.0].device_pixel_scale;

        match &mut prim_instance.kind {
            PrimitiveInstanceKind::LineDecoration { data_handle, ref mut cache_handle, .. } => {
                profile_scope!("LineDecoration");
                let prim_data = &mut data_stores.line_decoration[*data_handle];
                let common_data = &mut prim_data.common;
                let line_dec_data = &mut prim_data.kind;

                // Update the template this instane references, which may refresh the GPU
                // cache with any shared template data.
                line_dec_data.update(common_data, frame_state);

                // Work out the device pixel size to be used to cache this line decoration.
                if is_chased {
                    println!("\tline decoration key={:?}", line_dec_data.cache_key);
                }

                // If we have a cache key, it's a wavy / dashed / dotted line. Otherwise, it's
                // a simple solid line.
                if let Some(cache_key) = line_dec_data.cache_key.as_ref() {
                    // TODO(gw): Do we ever need / want to support scales for text decorations
                    //           based on the current transform?
                    let scale_factor = Scale::new(1.0) * device_pixel_scale;
                    let mut task_size = (LayoutSize::from_au(cache_key.size) * scale_factor).ceil().to_i32();
                    if task_size.width > MAX_LINE_DECORATION_RESOLUTION as i32 ||
                       task_size.height > MAX_LINE_DECORATION_RESOLUTION as i32 {
                         let max_extent = cmp::max(task_size.width, task_size.height);
                         let task_scale_factor = Scale::new(MAX_LINE_DECORATION_RESOLUTION as f32 / max_extent as f32);
                         task_size = (LayoutSize::from_au(cache_key.size) * scale_factor * task_scale_factor)
                                        .ceil().to_i32();
                    }

                    // Request a pre-rendered image task.
                    // TODO(gw): This match is a bit untidy, but it should disappear completely
                    //           once the prepare_prims and batching are unified. When that
                    //           happens, we can use the cache handle immediately, and not need
                    //           to temporarily store it in the primitive instance.
                    *cache_handle = Some(frame_state.resource_cache.request_render_task(
                        RenderTaskCacheKey {
                            size: task_size,
                            kind: RenderTaskCacheKeyKind::LineDecoration(cache_key.clone()),
                        },
                        frame_state.gpu_cache,
                        frame_state.render_tasks,
                        None,
                        false,
                        |render_tasks| {
                            render_tasks.add().init(RenderTask::new_line_decoration(
                                task_size,
                                cache_key.style,
                                cache_key.orientation,
                                cache_key.wavy_line_thickness.to_f32_px(),
                                LayoutSize::from_au(cache_key.size),
                            ))
                        }
                    ));
                }
            }
            PrimitiveInstanceKind::TextRun { run_index, data_handle, .. } => {
                profile_scope!("TextRun");
                let prim_data = &mut data_stores.text_run[*data_handle];
                let run = &mut self.text_runs[*run_index];

                prim_data.common.may_need_repetition = false;

                // The glyph transform has to match `glyph_transform` in "ps_text_run" shader.
                // It's relative to the rasterizing space of a glyph.
                let transform = frame_context.spatial_tree
                    .get_relative_transform(
                        prim_spatial_node_index,
                        pic_context.raster_spatial_node_index,
                    )
                    .into_fast_transform();
                let prim_offset = prim_data.common.prim_rect.origin.to_vector() - run.reference_frame_relative_offset;

                let pic = &self.pictures[pic_context.pic_index.0];
                let raster_space = pic.get_raster_space(frame_context.spatial_tree);
                let surface = &frame_state.surfaces[pic_context.surface_index.0];
                let prim_info = &scratch.prim_info[prim_instance.visibility_info.0 as usize];
                let root_scaling_factor = match pic.raster_config {
                    Some(ref raster_config) => raster_config.root_scaling_factor,
                    None => 1.0
                };

                run.request_resources(
                    prim_offset,
                    prim_info.clip_chain.pic_clip_rect,
                    &prim_data.font,
                    &prim_data.glyphs,
                    &transform.to_transform().with_destination::<_>(),
                    surface,
                    prim_spatial_node_index,
                    raster_space,
                    root_scaling_factor,
                    &pic_context.subpixel_mode,
                    frame_state.resource_cache,
                    frame_state.gpu_cache,
                    frame_state.render_tasks,
                    frame_context.spatial_tree,
                    scratch,
                );

                // Update the template this instane references, which may refresh the GPU
                // cache with any shared template data.
                prim_data.update(frame_state);
            }
            PrimitiveInstanceKind::Clear { data_handle, .. } => {
                profile_scope!("Clear");
                let prim_data = &mut data_stores.prim[*data_handle];

                prim_data.common.may_need_repetition = false;

                // Update the template this instane references, which may refresh the GPU
                // cache with any shared template data.
                prim_data.update(frame_state, frame_context.scene_properties);
            }
            PrimitiveInstanceKind::NormalBorder { data_handle, ref mut cache_handles, .. } => {
                profile_scope!("NormalBorder");
                let prim_data = &mut data_stores.normal_border[*data_handle];
                let common_data = &mut prim_data.common;
                let border_data = &mut prim_data.kind;

                common_data.may_need_repetition =
                    matches!(border_data.border.top.style, BorderStyle::Dotted | BorderStyle::Dashed) ||
                    matches!(border_data.border.right.style, BorderStyle::Dotted | BorderStyle::Dashed) ||
                    matches!(border_data.border.bottom.style, BorderStyle::Dotted | BorderStyle::Dashed) ||
                    matches!(border_data.border.left.style, BorderStyle::Dotted | BorderStyle::Dashed);


                // Update the template this instance references, which may refresh the GPU
                // cache with any shared template data.
                border_data.update(common_data, frame_state);

                // TODO(gw): For now, the scale factors to rasterize borders at are
                //           based on the true world transform of the primitive. When
                //           raster roots with local scale are supported in future,
                //           that will need to be accounted for here.
                let scale = frame_context
                    .spatial_tree
                    .get_world_transform(prim_spatial_node_index)
                    .scale_factors();

                // Scale factors are normalized to a power of 2 to reduce the number of
                // resolution changes.
                // For frames with a changing scale transform round scale factors up to
                // nearest power-of-2 boundary so that we don't keep having to redraw
                // the content as it scales up and down. Rounding up to nearest
                // power-of-2 boundary ensures we never scale up, only down --- avoiding
                // jaggies. It also ensures we never scale down by more than a factor of
                // 2, avoiding bad downscaling quality.
                let scale_width = clamp_to_scale_factor(scale.0, false);
                let scale_height = clamp_to_scale_factor(scale.1, false);
                // Pick the maximum dimension as scale
                let world_scale = LayoutToWorldScale::new(scale_width.max(scale_height));
                let mut scale = world_scale * device_pixel_scale;
                let max_scale = get_max_scale_for_border(border_data);
                scale.0 = scale.0.min(max_scale.0);

                // For each edge and corner, request the render task by content key
                // from the render task cache. This ensures that the render task for
                // this segment will be available for batching later in the frame.
                let mut handles: SmallVec<[RenderTaskCacheEntryHandle; 8]> = SmallVec::new();

                for segment in &border_data.border_segments {
                    // Update the cache key device size based on requested scale.
                    let cache_size = to_cache_size(segment.local_task_size * scale);
                    let cache_key = RenderTaskCacheKey {
                        kind: RenderTaskCacheKeyKind::BorderSegment(segment.cache_key.clone()),
                        size: cache_size,
                    };

                    handles.push(frame_state.resource_cache.request_render_task(
                        cache_key,
                        frame_state.gpu_cache,
                        frame_state.render_tasks,
                        None,
                        false,          // TODO(gw): We don't calculate opacity for borders yet!
                        |render_tasks| {
                            render_tasks.add().init(RenderTask::new_border_segment(
                                cache_size,
                                build_border_instances(
                                    &segment.cache_key,
                                    cache_size,
                                    &border_data.border,
                                    scale,
                                ),
                            ))
                        }
                    ));
                }

                *cache_handles = scratch
                    .border_cache_handles
                    .extend(handles);
            }
            PrimitiveInstanceKind::ImageBorder { data_handle, .. } => {
                profile_scope!("ImageBorder");
                let prim_data = &mut data_stores.image_border[*data_handle];

                // TODO: get access to the ninepatch and to check whwther we need support
                // for repetitions in the shader.

                // Update the template this instane references, which may refresh the GPU
                // cache with any shared template data.
                prim_data.kind.update(&mut prim_data.common, frame_state);
            }
            PrimitiveInstanceKind::Rectangle { data_handle, segment_instance_index, opacity_binding_index, color_binding_index, .. } => {
                profile_scope!("Rectangle");
                let prim_data = &mut data_stores.prim[*data_handle];
                prim_data.common.may_need_repetition = false;

                if *color_binding_index != ColorBindingIndex::INVALID {
                    match self.color_bindings[*color_binding_index] {
                        PropertyBinding::Binding(..) => {
                            // We explicitly invalidate the gpu cache
                            // if the color is animating.
                            let gpu_cache_handle =
                                if *segment_instance_index == SegmentInstanceIndex::INVALID {
                                    None
                                } else if *segment_instance_index == SegmentInstanceIndex::UNUSED {
                                    Some(&prim_data.common.gpu_cache_handle)
                                } else {
                                    Some(&scratch.segment_instances[*segment_instance_index].gpu_cache_handle)
                                };
                            if let Some(gpu_cache_handle) = gpu_cache_handle {
                                frame_state.gpu_cache.invalidate(gpu_cache_handle);
                            }
                        }
                        PropertyBinding::Value(..) => {},
                    }
                }

                // Update the template this instane references, which may refresh the GPU
                // cache with any shared template data.
                prim_data.update(
                    frame_state,
                    frame_context.scene_properties,
                );

                update_opacity_binding(
                    &mut self.opacity_bindings,
                    *opacity_binding_index,
                    frame_context.scene_properties,
                );

                write_segment(
                    *segment_instance_index,
                    frame_state,
                    &mut scratch.segments,
                    &mut scratch.segment_instances,
                    |request| {
                        prim_data.kind.write_prim_gpu_blocks(
                            request,
                            frame_context.scene_properties,
                        );
                    }
                );
            }
            PrimitiveInstanceKind::YuvImage { data_handle, segment_instance_index, .. } => {
                profile_scope!("YuvImage");
                let prim_data = &mut data_stores.yuv_image[*data_handle];
                let common_data = &mut prim_data.common;
                let yuv_image_data = &mut prim_data.kind;

                common_data.may_need_repetition = false;

                // Update the template this instane references, which may refresh the GPU
                // cache with any shared template data.
                yuv_image_data.update(common_data, frame_state);

                write_segment(
                    *segment_instance_index,
                    frame_state,
                    &mut scratch.segments,
                    &mut scratch.segment_instances,
                    |request| {
                        yuv_image_data.write_prim_gpu_blocks(request);
                    }
                );
            }
            PrimitiveInstanceKind::Image { data_handle, image_instance_index, .. } => {
                profile_scope!("Image");
                let prim_data = &mut data_stores.image[*data_handle];
                let common_data = &mut prim_data.common;
                let image_data = &mut prim_data.kind;

                if image_data.stretch_size.width >= common_data.prim_rect.size.width &&
                    image_data.stretch_size.height >= common_data.prim_rect.size.height {

                    common_data.may_need_repetition = false;
                }

                // Update the template this instane references, which may refresh the GPU
                // cache with any shared template data.
                image_data.update(common_data, frame_state);

                let image_instance = &mut self.images[*image_instance_index];

                update_opacity_binding(
                    &mut self.opacity_bindings,
                    image_instance.opacity_binding_index,
                    frame_context.scene_properties,
                );

                write_segment(
                    image_instance.segment_instance_index,
                    frame_state,
                    &mut scratch.segments,
                    &mut scratch.segment_instances,
                    |request| {
                        image_data.write_prim_gpu_blocks(request);
                    },
                );
            }
            PrimitiveInstanceKind::LinearGradient { data_handle, gradient_index, .. } => {
                profile_scope!("LinearGradient");
                let prim_data = &mut data_stores.linear_grad[*data_handle];
                let gradient = &mut self.linear_gradients[*gradient_index];

                // Update the template this instane references, which may refresh the GPU
                // cache with any shared template data.
                prim_data.update(frame_state);

                if prim_data.stretch_size.width >= prim_data.common.prim_rect.size.width &&
                    prim_data.stretch_size.height >= prim_data.common.prim_rect.size.height {

                    prim_data.common.may_need_repetition = false;
                }

                if prim_data.supports_caching {
                    let gradient_size = (prim_data.end_point - prim_data.start_point).to_size();

                    // Calculate what the range of the gradient is that covers this
                    // primitive. These values are included in the cache key. The
                    // size of the gradient task is the length of a texture cache
                    // region, for maximum accuracy, and a minimal size on the
                    // axis that doesn't matter.
                    let (size, orientation, prim_start_offset, prim_end_offset) =
                        if prim_data.start_point.x.approx_eq(&prim_data.end_point.x) {
                            let prim_start_offset = -prim_data.start_point.y / gradient_size.height;
                            let prim_end_offset = (prim_data.common.prim_rect.size.height - prim_data.start_point.y)
                                                    / gradient_size.height;
                            let size = DeviceIntSize::new(16, TEXTURE_REGION_DIMENSIONS);
                            (size, LineOrientation::Vertical, prim_start_offset, prim_end_offset)
                        } else {
                            let prim_start_offset = -prim_data.start_point.x / gradient_size.width;
                            let prim_end_offset = (prim_data.common.prim_rect.size.width - prim_data.start_point.x)
                                                    / gradient_size.width;
                            let size = DeviceIntSize::new(TEXTURE_REGION_DIMENSIONS, 16);
                            (size, LineOrientation::Horizontal, prim_start_offset, prim_end_offset)
                        };

                    // Build the cache key, including information about the stops.
                    let mut stops = vec![GradientStopKey::empty(); prim_data.stops.len()];

                    // Reverse the stops as required, same as the gradient builder does
                    // for the slow path.
                    if prim_data.reverse_stops {
                        for (src, dest) in prim_data.stops.iter().rev().zip(stops.iter_mut()) {
                            let stop = GradientStop {
                                offset: 1.0 - src.offset,
                                color: src.color,
                            };
                            *dest = stop.into();
                        }
                    } else {
                        for (src, dest) in prim_data.stops.iter().zip(stops.iter_mut()) {
                            *dest = (*src).into();
                        }
                    }

                    gradient.cache_segments.clear();

                    // emit render task caches and image rectangles to draw a gradient
                    // with offsets from start_offset to end_offset.
                    //
                    // the primitive is covered by a gradient that ranges from
                    // prim_start_offset to prim_end_offset.
                    //
                    // when clamping, these two pairs of offsets will always be the same.
                    // when repeating, however, we march across the primitive, blitting
                    // copies of the gradient along the way.  each copy has a range from
                    // 0.0 to 1.0 (assuming it's fully visible), but where it appears on
                    // the primitive changes as we go.  this position is also expressed
                    // as an offset: gradient_offset_base.  that is, in terms of stops,
                    // we draw a gradient from start_offset to end_offset.  its actual
                    // location on the primitive is at start_offset + gradient_offset_base.
                    //
                    // either way, we need a while-loop to draw the gradient as well
                    // because it might have more than 4 stops (the maximum of a cached
                    // segment) and/or hard stops. so we have a walk-within-the-walk from
                    // start_offset to end_offset caching up to GRADIENT_FP_STOPS stops at a
                    // time.
                    fn emit_segments(start_offset: f32, // start and end offset together are
                                     end_offset: f32,   // always a subrange of 0..1
                                     gradient_offset_base: f32,
                                     prim_start_offset: f32, // the offsets of the entire gradient as it
                                     prim_end_offset: f32,   // covers the entire primitive.
                                     prim_origin_in: LayoutPoint,
                                     prim_size_in: LayoutSize,
                                     task_size: DeviceIntSize,
                                     is_opaque: bool,
                                     stops: &[GradientStopKey],
                                     orientation: LineOrientation,
                                     frame_state: &mut FrameBuildingState,
                                     gradient: &mut LinearGradientPrimitive)
                    {
                        // these prints are used to generate documentation examples, so
                        // leaving them in but commented out:
                        //println!("emit_segments call:");
                        //println!("\tstart_offset: {}, end_offset: {}", start_offset, end_offset);
                        //println!("\tprim_start_offset: {}, prim_end_offset: {}", prim_start_offset, prim_end_offset);
                        //println!("\tgradient_offset_base: {}", gradient_offset_base);
                        let mut first_stop = 0;
                        // look for an inclusive range of stops [first_stop, last_stop].
                        // once first_stop points at (or past) the last stop, we're done.
                        while first_stop < stops.len()-1 {

                            // if the entire sub-gradient starts at an offset that's past the
                            // segment's end offset, we're done.
                            if stops[first_stop].offset > end_offset {
                                return;
                            }

                            // accumulate stops until we have GRADIENT_FP_STOPS of them, or we hit
                            // a hard stop:
                            let mut last_stop = first_stop;
                            let mut hard_stop = false;   // did we stop on a hard stop?
                            while last_stop < stops.len()-1 &&
                                  last_stop - first_stop + 1 < GRADIENT_FP_STOPS
                            {
                                if stops[last_stop+1].offset == stops[last_stop].offset {
                                    hard_stop = true;
                                    break;
                                }

                                last_stop = last_stop + 1;
                            }

                            let num_stops = last_stop - first_stop + 1;

                            // repeated hard stops at the same offset, skip
                            if num_stops == 0 {
                                first_stop = last_stop + 1;
                                continue;
                            }

                            // if the last_stop offset is before start_offset, the segment's not visible:
                            if stops[last_stop].offset < start_offset {
                                first_stop = if hard_stop { last_stop+1 } else { last_stop };
                                continue;
                            }

                            let segment_start_point = start_offset.max(stops[first_stop].offset);
                            let segment_end_point   = end_offset  .min(stops[last_stop ].offset);

                            let mut segment_stops = [GradientStopKey::empty(); GRADIENT_FP_STOPS];
                            for i in 0..num_stops {
                                segment_stops[i] = stops[first_stop + i];
                            }

                            let cache_key = GradientCacheKey {
                                orientation,
                                start_stop_point: VectorKey {
                                    x: segment_start_point,
                                    y: segment_end_point,
                                },
                                stops: segment_stops,
                            };

                            let mut prim_origin = prim_origin_in;
                            let mut prim_size   = prim_size_in;

                            // the primitive is covered by a segment from overall_start to
                            // overall_end; scale and shift based on the length of the actual
                            // segment that we're drawing:
                            let inv_length = 1.0 / ( prim_end_offset - prim_start_offset );
                            if orientation == LineOrientation::Horizontal {
                                prim_origin.x    += ( segment_start_point + gradient_offset_base - prim_start_offset )
                                                    * inv_length * prim_size.width;
                                prim_size.width  *= ( segment_end_point - segment_start_point )
                                                    * inv_length; // 2 gradient_offset_bases cancel out
                            } else {
                                prim_origin.y    += ( segment_start_point + gradient_offset_base - prim_start_offset )
                                                    * inv_length * prim_size.height;
                                prim_size.height *= ( segment_end_point - segment_start_point )
                                                    * inv_length; // 2 gradient_offset_bases cancel out
                            }

                            // <= 0 can happen if a hardstop lands exactly on an edge
                            if prim_size.area() > 0.0 {
                                let local_rect = LayoutRect::new( prim_origin, prim_size );

                                // documentation example traces:
                                //println!("\t\tcaching from offset {} to {}", segment_start_point, segment_end_point);
                                //println!("\t\tand blitting to {:?}", local_rect);

                                // Request the render task each frame.
                                gradient.cache_segments.push(
                                    CachedGradientSegment {
                                        handle: frame_state.resource_cache.request_render_task(
                                            RenderTaskCacheKey {
                                                size: task_size,
                                                kind: RenderTaskCacheKeyKind::Gradient(cache_key),
                                            },
                                            frame_state.gpu_cache,
                                            frame_state.render_tasks,
                                            None,
                                            is_opaque,
                                            |render_tasks| {
                                                render_tasks.add().init(RenderTask::new_gradient(
                                                    task_size,
                                                    segment_stops,
                                                    orientation,
                                                    segment_start_point,
                                                    segment_end_point,
                                                ))
                                            }),
                                        local_rect: local_rect,
                                    }
                                );
                            }

                            // if ending on a hardstop, skip past it for the start of the next run:
                            first_stop = if hard_stop { last_stop + 1 } else { last_stop };
                        }
                    }

                    if prim_data.extend_mode == ExtendMode::Clamp ||
                       ( prim_start_offset >= 0.0 && prim_end_offset <= 1.0 )  // repeat doesn't matter
                    {
                        // To support clamping, we need to make sure that quads are emitted for the
                        // segments before and after the 0.0...1.0 range of offsets.  emit_segments
                        // can handle that by duplicating the first and last point if necessary:
                        if prim_start_offset < 0.0 {
                            stops.insert(0, GradientStopKey {
                                offset: prim_start_offset,
                                color : stops[0].color
                            });
                        }

                        if prim_end_offset > 1.0 {
                            stops.push( GradientStopKey {
                                offset: prim_end_offset,
                                color : stops[stops.len()-1].color
                            });
                        }

                        emit_segments(prim_start_offset, prim_end_offset,
                                      0.0,
                                      prim_start_offset, prim_end_offset,
                                      prim_data.common.prim_rect.origin,
                                      prim_data.common.prim_rect.size,
                                      size,
                                      prim_data.stops_opacity.is_opaque,
                                      &stops,
                                      orientation,
                                      frame_state,
                                      gradient);
                    }
                    else
                    {
                        let mut segment_start_point = prim_start_offset;
                        while segment_start_point < prim_end_offset {

                            // gradient stops are expressed in the range 0.0 ... 1.0, so to blit
                            // a copy of the gradient, snap to the integer just before the offset
                            // we want ...
                            let gradient_offset_base = segment_start_point.floor();
                            // .. and then draw from a start offset in range 0 to 1 ...
                            let repeat_start = segment_start_point - gradient_offset_base;
                            // .. up to the next integer, but clamped to the primitive's real
                            // end offset:
                            let repeat_end = (gradient_offset_base + 1.0).min(prim_end_offset) - gradient_offset_base;

                            emit_segments(repeat_start, repeat_end,
                                          gradient_offset_base,
                                          prim_start_offset, prim_end_offset,
                                          prim_data.common.prim_rect.origin,
                                          prim_data.common.prim_rect.size,
                                          size,
                                          prim_data.stops_opacity.is_opaque,
                                          &stops,
                                          orientation,
                                          frame_state,
                                          gradient);

                            segment_start_point = repeat_end + gradient_offset_base;
                        }
                    }
                }

                if prim_data.tile_spacing != LayoutSize::zero() {
                    // We are performing the decomposition on the CPU here, no need to
                    // have it in the shader.
                    prim_data.common.may_need_repetition = false;

                    let prim_info = &scratch.prim_info[prim_instance.visibility_info.0 as usize];

                    let map_local_to_world = SpaceMapper::new_with_target(
                        ROOT_SPATIAL_NODE_INDEX,
                        prim_spatial_node_index,
                        frame_context.global_screen_world_rect,
                        frame_context.spatial_tree,
                    );

                    gradient.visible_tiles_range = decompose_repeated_primitive(
                        &prim_info.combined_local_clip_rect,
                        &prim_data.common.prim_rect,
                        prim_info.clipped_world_rect,
                        &prim_data.stretch_size,
                        &prim_data.tile_spacing,
                        frame_state,
                        &mut scratch.gradient_tiles,
                        &map_local_to_world,
                        &mut |_, mut request| {
                            request.push([
                                prim_data.start_point.x,
                                prim_data.start_point.y,
                                prim_data.end_point.x,
                                prim_data.end_point.y,
                            ]);
                            request.push([
                                pack_as_float(prim_data.extend_mode as u32),
                                prim_data.stretch_size.width,
                                prim_data.stretch_size.height,
                                0.0,
                            ]);
                        }
                    );

                    if gradient.visible_tiles_range.is_empty() {
                        prim_instance.visibility_info = PrimitiveVisibilityIndex::INVALID;
                    }
                }

                // TODO(gw): Consider whether it's worth doing segment building
                //           for gradient primitives.
            }
            PrimitiveInstanceKind::RadialGradient { data_handle, ref mut visible_tiles_range, .. } => {
                profile_scope!("RadialGradient");
                let prim_data = &mut data_stores.radial_grad[*data_handle];

                if prim_data.stretch_size.width >= prim_data.common.prim_rect.size.width &&
                    prim_data.stretch_size.height >= prim_data.common.prim_rect.size.height {

                    // We are performing the decomposition on the CPU here, no need to
                    // have it in the shader.
                    prim_data.common.may_need_repetition = false;
                }

                // Update the template this instane references, which may refresh the GPU
                // cache with any shared template data.
                prim_data.update(frame_state);

                if prim_data.tile_spacing != LayoutSize::zero() {
                    let prim_info = &scratch.prim_info[prim_instance.visibility_info.0 as usize];

                    let map_local_to_world = SpaceMapper::new_with_target(
                        ROOT_SPATIAL_NODE_INDEX,
                        prim_spatial_node_index,
                        frame_context.global_screen_world_rect,
                        frame_context.spatial_tree,
                    );

                    prim_data.common.may_need_repetition = false;

                    *visible_tiles_range = decompose_repeated_primitive(
                        &prim_info.combined_local_clip_rect,
                        &prim_data.common.prim_rect,
                        prim_info.clipped_world_rect,
                        &prim_data.stretch_size,
                        &prim_data.tile_spacing,
                        frame_state,
                        &mut scratch.gradient_tiles,
                        &map_local_to_world,
                        &mut |_, mut request| {
                            request.push([
                                prim_data.center.x,
                                prim_data.center.y,
                                prim_data.params.start_radius,
                                prim_data.params.end_radius,
                            ]);
                            request.push([
                                prim_data.params.ratio_xy,
                                pack_as_float(prim_data.extend_mode as u32),
                                prim_data.stretch_size.width,
                                prim_data.stretch_size.height,
                            ]);
                        },
                    );

                    if visible_tiles_range.is_empty() {
                        prim_instance.visibility_info = PrimitiveVisibilityIndex::INVALID;
                    }
                }

                // TODO(gw): Consider whether it's worth doing segment building
                //           for gradient primitives.
            }
            PrimitiveInstanceKind::ConicGradient { data_handle, ref mut visible_tiles_range, .. } => {
                profile_scope!("ConicGradient");
                let prim_data = &mut data_stores.conic_grad[*data_handle];

                if prim_data.stretch_size.width >= prim_data.common.prim_rect.size.width &&
                    prim_data.stretch_size.height >= prim_data.common.prim_rect.size.height {

                    // We are performing the decomposition on the CPU here, no need to
                    // have it in the shader.
                    prim_data.common.may_need_repetition = false;
                }

                // Update the template this instane references, which may refresh the GPU
                // cache with any shared template data.
                prim_data.update(frame_state);

                if prim_data.tile_spacing != LayoutSize::zero() {
                    let prim_info = &scratch.prim_info[prim_instance.visibility_info.0 as usize];

                    let map_local_to_world = SpaceMapper::new_with_target(
                        ROOT_SPATIAL_NODE_INDEX,
                        prim_spatial_node_index,
                        frame_context.global_screen_world_rect,
                        frame_context.spatial_tree,
                    );

                    prim_data.common.may_need_repetition = false;

                    *visible_tiles_range = decompose_repeated_primitive(
                        &prim_info.combined_local_clip_rect,
                        &prim_data.common.prim_rect,
                        prim_info.clipped_world_rect,
                        &prim_data.stretch_size,
                        &prim_data.tile_spacing,
                        frame_state,
                        &mut scratch.gradient_tiles,
                        &map_local_to_world,
                        &mut |_, mut request| {
                            request.push([
                                prim_data.center.x,
                                prim_data.center.y,
                                prim_data.params.start_offset,
                                prim_data.params.end_offset,
                            ]);
                            request.push([
                                prim_data.params.angle,
                                pack_as_float(prim_data.extend_mode as u32),
                                prim_data.stretch_size.width,
                                prim_data.stretch_size.height,
                            ]);
                        },
                    );

                    if visible_tiles_range.is_empty() {
                        prim_instance.visibility_info = PrimitiveVisibilityIndex::INVALID;
                    }
                }

                // TODO(gw): Consider whether it's worth doing segment building
                //           for gradient primitives.
            }
            PrimitiveInstanceKind::Picture { pic_index, segment_instance_index, .. } => {
                profile_scope!("Picture");
                let pic = &mut self.pictures[pic_index.0];
                let prim_info = &scratch.prim_info[prim_instance.visibility_info.0 as usize];

                if pic.prepare_for_render(
                    frame_context,
                    frame_state,
                    data_stores,
                ) {
                    if let Some(ref mut splitter) = pic_state.plane_splitter {
                        PicturePrimitive::add_split_plane(
                            splitter,
                            frame_context.spatial_tree,
                            prim_spatial_node_index,
                            pic.precise_local_rect,
                            &prim_info.combined_local_clip_rect,
                            frame_state.current_dirty_region().combined,
                            plane_split_anchor,
                        );
                    }

                    // If this picture uses segments, ensure the GPU cache is
                    // up to date with segment local rects.
                    // TODO(gw): This entire match statement above can now be
                    //           refactored into prepare_interned_prim_for_render.
                    if pic.can_use_segments() {
                        write_segment(
                            *segment_instance_index,
                            frame_state,
                            &mut scratch.segments,
                            &mut scratch.segment_instances,
                            |request| {
                                request.push(PremultipliedColorF::WHITE);
                                request.push(PremultipliedColorF::WHITE);
                                request.push([
                                    -1.0,       // -ve means use prim rect for stretch size
                                    0.0,
                                    0.0,
                                    0.0,
                                ]);
                            }
                        );
                    }
                } else {
                    prim_instance.visibility_info = PrimitiveVisibilityIndex::INVALID;
                }
            }
            PrimitiveInstanceKind::Backdrop { data_handle } => {
                profile_scope!("Backdrop");
                let backdrop_pic_index = data_stores.backdrop[*data_handle].kind.pic_index;

                // Setup a dependency on the backdrop picture to ensure it is rendered prior to rendering this primitive.
                let backdrop_surface_index = self.pictures[backdrop_pic_index.0].raster_config.as_ref().unwrap().surface_index;
                if let Some(backdrop_tasks) = frame_state.surfaces[backdrop_surface_index.0].render_tasks {
                    let picture_task_id = frame_state.surfaces[pic_context.surface_index.0].render_tasks.as_ref().unwrap().port;
                    frame_state.render_tasks.add_dependency(picture_task_id, backdrop_tasks.root);
                } else {
                    if prim_instance.is_chased() {
                        println!("\tBackdrop primitive culled because backdrop task was not assigned render tasks");
                    }
                    prim_instance.visibility_info = PrimitiveVisibilityIndex::INVALID;
                }
            }
        };
    }
}

fn write_segment<F>(
    segment_instance_index: SegmentInstanceIndex,
    frame_state: &mut FrameBuildingState,
    segments: &mut SegmentStorage,
    segment_instances: &mut SegmentInstanceStorage,
    f: F,
) where F: Fn(&mut GpuDataRequest) {
    debug_assert_ne!(segment_instance_index, SegmentInstanceIndex::INVALID);
    if segment_instance_index != SegmentInstanceIndex::UNUSED {
        let segment_instance = &mut segment_instances[segment_instance_index];

        if let Some(mut request) = frame_state.gpu_cache.request(&mut segment_instance.gpu_cache_handle) {
            let segments = &segments[segment_instance.segments_range];

            f(&mut request);

            for segment in segments {
                request.write_segment(
                    segment.local_rect,
                    [0.0; 4],
                );
            }
        }
    }
}

fn decompose_repeated_primitive(
    combined_local_clip_rect: &LayoutRect,
    prim_local_rect: &LayoutRect,
    prim_world_rect: WorldRect,
    stretch_size: &LayoutSize,
    tile_spacing: &LayoutSize,
    frame_state: &mut FrameBuildingState,
    gradient_tiles: &mut GradientTileStorage,
    map_local_to_world: &SpaceMapper<LayoutPixel, WorldPixel>,
    callback: &mut dyn FnMut(&LayoutRect, GpuDataRequest),
) -> GradientTileRange {
    let mut visible_tiles = Vec::new();

    // Tighten the clip rect because decomposing the repeated image can
    // produce primitives that are partially covering the original image
    // rect and we want to clip these extra parts out.
    let tight_clip_rect = combined_local_clip_rect
        .intersection(prim_local_rect).unwrap();

    let visible_rect = compute_conservative_visible_rect(
        &tight_clip_rect,
        prim_world_rect,
        map_local_to_world,
    );
    let stride = *stretch_size + *tile_spacing;

    let repetitions = image_tiling::repetitions(prim_local_rect, &visible_rect, stride);
    for Repetition { origin, .. } in repetitions {
        let mut handle = GpuCacheHandle::new();
        let rect = LayoutRect {
            origin,
            size: *stretch_size,
        };

        if let Some(request) = frame_state.gpu_cache.request(&mut handle) {
            callback(&rect, request);
        }

        visible_tiles.push(VisibleGradientTile {
            local_rect: rect,
            local_clip_rect: tight_clip_rect,
            handle
        });
    }

    // At this point if we don't have tiles to show it means we could probably
    // have done a better a job at culling during an earlier stage.
    // Clearing the screen rect has the effect of "culling out" the primitive
    // from the point of view of the batch builder, and ensures we don't hit
    // assertions later on because we didn't request any image.
    if visible_tiles.is_empty() {
        GradientTileRange::empty()
    } else {
        gradient_tiles.extend(visible_tiles)
    }
}

fn compute_conservative_visible_rect(
    local_clip_rect: &LayoutRect,
    world_culling_rect: WorldRect,
    map_local_to_world: &SpaceMapper<LayoutPixel, WorldPixel>,
) -> LayoutRect {
    if let Some(local_bounds) = map_local_to_world.unmap(&world_culling_rect) {
        return local_clip_rect.intersection(&local_bounds).unwrap_or_else(LayoutRect::zero)
    }

    *local_clip_rect
}

fn edge_flags_for_tile_spacing(tile_spacing: &LayoutSize) -> EdgeAaSegmentMask {
    let mut flags = EdgeAaSegmentMask::empty();

    if tile_spacing.width > 0.0 {
        flags |= EdgeAaSegmentMask::LEFT | EdgeAaSegmentMask::RIGHT;
    }
    if tile_spacing.height > 0.0 {
        flags |= EdgeAaSegmentMask::TOP | EdgeAaSegmentMask::BOTTOM;
    }

    flags
}

impl<'a> GpuDataRequest<'a> {
    // Write the GPU cache data for an individual segment.
    fn write_segment(
        &mut self,
        local_rect: LayoutRect,
        extra_data: [f32; 4],
    ) {
        let _ = VECS_PER_SEGMENT;
        self.push(local_rect);
        self.push(extra_data);
    }
}

    fn write_brush_segment_description(
        prim_local_rect: LayoutRect,
        prim_local_clip_rect: LayoutRect,
        clip_chain: &ClipChainInstance,
        segment_builder: &mut SegmentBuilder,
        clip_store: &ClipStore,
        data_stores: &DataStores,
    ) -> bool {
        // If the brush is small, we want to skip building segments
        // and just draw it as a single primitive with clip mask.
        if prim_local_rect.size.area() < MIN_BRUSH_SPLIT_AREA {
            return false;
        }

        segment_builder.initialize(
            prim_local_rect,
            None,
            prim_local_clip_rect
        );

        // Segment the primitive on all the local-space clip sources that we can.
        for i in 0 .. clip_chain.clips_range.count {
            let clip_instance = clip_store
                .get_instance_from_range(&clip_chain.clips_range, i);
            let clip_node = &data_stores.clip[clip_instance.handle];

            // If this clip item is positioned by another positioning node, its relative position
            // could change during scrolling. This means that we would need to resegment. Instead
            // of doing that, only segment with clips that have the same positioning node.
            // TODO(mrobinson, #2858): It may make sense to include these nodes, resegmenting only
            // when necessary while scrolling.
            if !clip_instance.flags.contains(ClipNodeFlags::SAME_SPATIAL_NODE) {
                continue;
            }

            let (local_clip_rect, radius, mode) = match clip_node.item.kind {
                ClipItemKind::RoundedRectangle { rect, radius, mode } => {
                    (rect, Some(radius), mode)
                }
                ClipItemKind::Rectangle { rect, mode } => {
                    (rect, None, mode)
                }
                ClipItemKind::BoxShadow { ref source } => {
                    // For inset box shadows, we can clip out any
                    // pixels that are inside the shadow region
                    // and are beyond the inner rect, as they can't
                    // be affected by the blur radius.
                    let inner_clip_mode = match source.clip_mode {
                        BoxShadowClipMode::Outset => None,
                        BoxShadowClipMode::Inset => Some(ClipMode::ClipOut),
                    };

                    // Push a region into the segment builder where the
                    // box-shadow can have an effect on the result. This
                    // ensures clip-mask tasks get allocated for these
                    // pixel regions, even if no other clips affect them.
                    segment_builder.push_mask_region(
                        source.prim_shadow_rect,
                        source.prim_shadow_rect.inflate(
                            -0.5 * source.original_alloc_size.width,
                            -0.5 * source.original_alloc_size.height,
                        ),
                        inner_clip_mode,
                    );

                    continue;
                }
                ClipItemKind::Image { .. } => {
                    // If we encounter an image mask, bail out from segment building.
                    // It's not possible to know which parts of the primitive are affected
                    // by the mask (without inspecting the pixels). We could do something
                    // better here in the future if it ever shows up as a performance issue
                    // (for instance, at least segment based on the bounding rect of the
                    // image mask if it's non-repeating).
                    return false;
                }
            };

            segment_builder.push_clip_rect(local_clip_rect, radius, mode);
        }

        true
    }

impl PrimitiveInstance {
    fn build_segments_if_needed(
        &mut self,
        prim_info: &PrimitiveVisibility,
        frame_state: &mut FrameBuildingState,
        prim_store: &mut PrimitiveStore,
        data_stores: &DataStores,
        segments_store: &mut SegmentStorage,
        segment_instances_store: &mut SegmentInstanceStorage,
    ) {
        let prim_clip_chain = &prim_info.clip_chain;

        // Usually, the primitive rect can be found from information
        // in the instance and primitive template.
        let prim_local_rect = data_stores.get_local_prim_rect(
            self,
            prim_store,
        );

        let segment_instance_index = match self.kind {
            PrimitiveInstanceKind::Rectangle { ref mut segment_instance_index, .. } |
            PrimitiveInstanceKind::YuvImage { ref mut segment_instance_index, .. } => {
                segment_instance_index
            }
            PrimitiveInstanceKind::Image { data_handle, image_instance_index, .. } => {
                let image_data = &data_stores.image[data_handle].kind;
                let image_instance = &mut prim_store.images[image_instance_index];
                //Note: tiled images don't support automatic segmentation,
                // they strictly produce one segment per visible tile instead.
                if frame_state
                    .resource_cache
                    .get_image_properties(image_data.key)
                    .and_then(|properties| properties.tiling)
                    .is_some()
                {
                    image_instance.segment_instance_index = SegmentInstanceIndex::UNUSED;
                    return;
                }
                &mut image_instance.segment_instance_index
            }
            PrimitiveInstanceKind::Picture { ref mut segment_instance_index, pic_index, .. } => {
                let pic = &mut prim_store.pictures[pic_index.0];

                // If this picture supports segment rendering
                if pic.can_use_segments() {
                    // If the segments have been invalidated, ensure the current
                    // index of segments is invalid. This ensures that the segment
                    // building logic below will be run.
                    if !pic.segments_are_valid {
                        *segment_instance_index = SegmentInstanceIndex::INVALID;
                        pic.segments_are_valid = true;
                    }

                    segment_instance_index
                } else {
                    return;
                }
            }
            PrimitiveInstanceKind::TextRun { .. } |
            PrimitiveInstanceKind::NormalBorder { .. } |
            PrimitiveInstanceKind::ImageBorder { .. } |
            PrimitiveInstanceKind::Clear { .. } |
            PrimitiveInstanceKind::LinearGradient { .. } |
            PrimitiveInstanceKind::RadialGradient { .. } |
            PrimitiveInstanceKind::ConicGradient { .. } |
            PrimitiveInstanceKind::LineDecoration { .. } |
            PrimitiveInstanceKind::Backdrop { .. } => {
                // These primitives don't support / need segments.
                return;
            }
        };

        if *segment_instance_index == SegmentInstanceIndex::INVALID {
            let mut segments: SmallVec<[BrushSegment; 8]> = SmallVec::new();

            if write_brush_segment_description(
                prim_local_rect,
                self.local_clip_rect,
                prim_clip_chain,
                &mut frame_state.segment_builder,
                frame_state.clip_store,
                data_stores,
            ) {
                frame_state.segment_builder.build(|segment| {
                    segments.push(
                        BrushSegment::new(
                            segment.rect.translate(-prim_local_rect.origin.to_vector()),
                            segment.has_mask,
                            segment.edge_flags,
                            [0.0; 4],
                            BrushFlags::PERSPECTIVE_INTERPOLATION,
                        ),
                    );
                });
            }

            // If only a single segment is produced, there is no benefit to writing
            // a segment instance array. Instead, just use the main primitive rect
            // written into the GPU cache.
            // TODO(gw): This is (sortof) a bandaid - due to a limitation in the current
            //           brush encoding, we can only support a total of up to 2^16 segments.
            //           This should be (more than) enough for any real world case, so for
            //           now we can handle this by skipping cases where we were generating
            //           segments where there is no benefit. The long term / robust fix
            //           for this is to move the segment building to be done as a more
            //           limited nine-patch system during scene building, removing arbitrary
            //           segmentation during frame-building (see bug #1617491).
            if segments.len() <= 1 {
                *segment_instance_index = SegmentInstanceIndex::UNUSED;
            } else {
                let segments_range = segments_store.extend(segments);

                let instance = SegmentedInstance {
                    segments_range,
                    gpu_cache_handle: GpuCacheHandle::new(),
                };

                *segment_instance_index = segment_instances_store.push(instance);
            };
        }
    }

    fn update_clip_task_for_brush(
        &self,
        prim_origin: &LayoutPoint,
        prim_info: &mut PrimitiveVisibility,
        prim_spatial_node_index: SpatialNodeIndex,
        root_spatial_node_index: SpatialNodeIndex,
        pic_context: &PictureContext,
        pic_state: &mut PictureState,
        frame_context: &FrameBuildingContext,
        frame_state: &mut FrameBuildingState,
        prim_store: &PrimitiveStore,
        data_stores: &mut DataStores,
        segments_store: &mut SegmentStorage,
        segment_instances_store: &mut SegmentInstanceStorage,
        clip_mask_instances: &mut Vec<ClipMaskKind>,
        unclipped: &DeviceRect,
        device_pixel_scale: DevicePixelScale,
    ) -> bool {
        let segments = match self.kind {
            PrimitiveInstanceKind::TextRun { .. } |
            PrimitiveInstanceKind::Clear { .. } |
            PrimitiveInstanceKind::LineDecoration { .. } |
            PrimitiveInstanceKind::Backdrop { .. } => {
                return false;
            }
            PrimitiveInstanceKind::Image { image_instance_index, .. } => {
                let segment_instance_index = prim_store
                    .images[image_instance_index]
                    .segment_instance_index;

                if segment_instance_index == SegmentInstanceIndex::UNUSED {
                    return false;
                }

                let segment_instance = &segment_instances_store[segment_instance_index];

                &segments_store[segment_instance.segments_range]
            }
            PrimitiveInstanceKind::Picture { segment_instance_index, .. } => {
                // Pictures may not support segment rendering at all (INVALID)
                // or support segment rendering but choose not to due to size
                // or some other factor (UNUSED).
                if segment_instance_index == SegmentInstanceIndex::UNUSED ||
                   segment_instance_index == SegmentInstanceIndex::INVALID {
                    return false;
                }

                let segment_instance = &segment_instances_store[segment_instance_index];
                &segments_store[segment_instance.segments_range]
            }
            PrimitiveInstanceKind::YuvImage { segment_instance_index, .. } |
            PrimitiveInstanceKind::Rectangle { segment_instance_index, .. } => {
                debug_assert!(segment_instance_index != SegmentInstanceIndex::INVALID);

                if segment_instance_index == SegmentInstanceIndex::UNUSED {
                    return false;
                }

                let segment_instance = &segment_instances_store[segment_instance_index];

                &segments_store[segment_instance.segments_range]
            }
            PrimitiveInstanceKind::ImageBorder { data_handle, .. } => {
                let border_data = &data_stores.image_border[data_handle].kind;

                // TODO: This is quite messy - once we remove legacy primitives we
                //       can change this to be a tuple match on (instance, template)
                border_data.brush_segments.as_slice()
            }
            PrimitiveInstanceKind::NormalBorder { data_handle, .. } => {
                let border_data = &data_stores.normal_border[data_handle].kind;

                // TODO: This is quite messy - once we remove legacy primitives we
                //       can change this to be a tuple match on (instance, template)
                border_data.brush_segments.as_slice()
            }
            PrimitiveInstanceKind::LinearGradient { data_handle, .. } => {
                let prim_data = &data_stores.linear_grad[data_handle];

                // TODO: This is quite messy - once we remove legacy primitives we
                //       can change this to be a tuple match on (instance, template)
                if prim_data.brush_segments.is_empty() {
                    return false;
                }

                prim_data.brush_segments.as_slice()
            }
            PrimitiveInstanceKind::RadialGradient { data_handle, .. } => {
                let prim_data = &data_stores.radial_grad[data_handle];

                // TODO: This is quite messy - once we remove legacy primitives we
                //       can change this to be a tuple match on (instance, template)
                if prim_data.brush_segments.is_empty() {
                    return false;
                }

                prim_data.brush_segments.as_slice()
            }
            PrimitiveInstanceKind::ConicGradient { data_handle, .. } => {
                let prim_data = &data_stores.conic_grad[data_handle];

                // TODO: This is quite messy - once we remove legacy primitives we
                //       can change this to be a tuple match on (instance, template)
                if prim_data.brush_segments.is_empty() {
                    return false;
                }

                prim_data.brush_segments.as_slice()
            }
        };

        // If there are no segments, early out to avoid setting a valid
        // clip task instance location below.
        if segments.is_empty() {
            return true;
        }

        // Set where in the clip mask instances array the clip mask info
        // can be found for this primitive. Each segment will push the
        // clip mask information for itself in update_clip_task below.
        prim_info.clip_task_index = ClipTaskIndex(clip_mask_instances.len() as _);

        // If we only built 1 segment, there is no point in re-running
        // the clip chain builder. Instead, just use the clip chain
        // instance that was built for the main primitive. This is a
        // significant optimization for the common case.
        if segments.len() == 1 {
            let clip_mask_kind = segments[0].update_clip_task(
                Some(&prim_info.clip_chain),
                prim_info.clipped_world_rect,
                root_spatial_node_index,
                pic_context.surface_index,
                pic_state,
                frame_context,
                frame_state,
                &mut data_stores.clip,
                unclipped,
                device_pixel_scale,
            );
            clip_mask_instances.push(clip_mask_kind);
        } else {
            let dirty_world_rect = frame_state.current_dirty_region().combined;

            for segment in segments {
                // Build a clip chain for the smaller segment rect. This will
                // often manage to eliminate most/all clips, and sometimes
                // clip the segment completely.
                frame_state.clip_store.set_active_clips_from_clip_chain(
                    &prim_info.clip_chain,
                    prim_spatial_node_index,
                    &frame_context.spatial_tree,
                );

                let segment_clip_chain = frame_state
                    .clip_store
                    .build_clip_chain_instance(
                        segment.local_rect.translate(prim_origin.to_vector()),
                        &pic_state.map_local_to_pic,
                        &pic_state.map_pic_to_world,
                        &frame_context.spatial_tree,
                        frame_state.gpu_cache,
                        frame_state.resource_cache,
                        device_pixel_scale,
                        &dirty_world_rect,
                        &mut data_stores.clip,
                        false,
                        self.is_chased(),
                    );

                let clip_mask_kind = segment.update_clip_task(
                    segment_clip_chain.as_ref(),
                    prim_info.clipped_world_rect,
                    root_spatial_node_index,
                    pic_context.surface_index,
                    pic_state,
                    frame_context,
                    frame_state,
                    &mut data_stores.clip,
                    unclipped,
                    device_pixel_scale,
                );
                clip_mask_instances.push(clip_mask_kind);
            }
        }

        true
    }

    fn update_clip_task(
        &mut self,
        prim_origin: &LayoutPoint,
        prim_spatial_node_index: SpatialNodeIndex,
        root_spatial_node_index: SpatialNodeIndex,
        pic_context: &PictureContext,
        pic_state: &mut PictureState,
        frame_context: &FrameBuildingContext,
        frame_state: &mut FrameBuildingState,
        prim_store: &mut PrimitiveStore,
        data_stores: &mut DataStores,
        scratch: &mut PrimitiveScratchBuffer,
    ) {
        let prim_info = &mut scratch.prim_info[self.visibility_info.0 as usize];
        let device_pixel_scale = frame_state.surfaces[pic_context.surface_index.0].device_pixel_scale;

        if self.is_chased() {
            println!("\tupdating clip task with pic rect {:?}", prim_info.clip_chain.pic_clip_rect);
        }

        // Get the device space rect for the primitive if it was unclipped.
        let unclipped = match get_unclipped_device_rect(
            prim_info.clip_chain.pic_clip_rect,
            &pic_state.map_pic_to_raster,
            device_pixel_scale,
        ) {
            Some(rect) => rect,
            None => return,
        };

        self.build_segments_if_needed(
            &prim_info,
            frame_state,
            prim_store,
            data_stores,
            &mut scratch.segments,
            &mut scratch.segment_instances,
        );

        // First try to  render this primitive's mask using optimized brush rendering.
        if self.update_clip_task_for_brush(
            prim_origin,
            prim_info,
            prim_spatial_node_index,
            root_spatial_node_index,
            pic_context,
            pic_state,
            frame_context,
            frame_state,
            prim_store,
            data_stores,
            &mut scratch.segments,
            &mut scratch.segment_instances,
            &mut scratch.clip_mask_instances,
            &unclipped,
            device_pixel_scale,
        ) {
            if self.is_chased() {
                println!("\tsegment tasks have been created for clipping");
            }
            return;
        }

        if prim_info.clip_chain.needs_mask {
            // Get a minimal device space rect, clipped to the screen that we
            // need to allocate for the clip mask, as well as interpolated
            // snap offsets.
            if let Some(device_rect) = get_clipped_device_rect(
                &unclipped,
                &pic_state.map_raster_to_world,
                prim_info.clipped_world_rect,
                device_pixel_scale,
            ) {
                let (device_rect, device_pixel_scale) = adjust_mask_scale_for_max_size(device_rect, device_pixel_scale);

                let clip_task_id = RenderTask::new_mask(
                    device_rect,
                    prim_info.clip_chain.clips_range,
                    root_spatial_node_index,
                    frame_state.clip_store,
                    frame_state.gpu_cache,
                    frame_state.resource_cache,
                    frame_state.render_tasks,
                    &mut data_stores.clip,
                    device_pixel_scale,
                    frame_context.fb_config,
                );
                if self.is_chased() {
                    println!("\tcreated task {:?} with device rect {:?}",
                        clip_task_id, device_rect);
                }
                // Set the global clip mask instance for this primitive.
                let clip_task_index = ClipTaskIndex(scratch.clip_mask_instances.len() as _);
                scratch.clip_mask_instances.push(ClipMaskKind::Mask(clip_task_id));
                prim_info.clip_task_index = clip_task_index;
                frame_state.render_tasks.add_dependency(
                    frame_state.surfaces[pic_context.surface_index.0].render_tasks.unwrap().port,
                    clip_task_id,
                );
            }
        }
    }
}

// Ensures that the size of mask render tasks are within MAX_MASK_SIZE.
fn adjust_mask_scale_for_max_size(device_rect: DeviceRect, device_pixel_scale: DevicePixelScale) -> (DeviceIntRect, DevicePixelScale) {
    if device_rect.width() > MAX_MASK_SIZE || device_rect.height() > MAX_MASK_SIZE {
        // round_out will grow by 1 integer pixel if origin is on a
        // fractional position, so keep that margin for error with -1:
        let scale = (MAX_MASK_SIZE - 1.0) /
            f32::max(device_rect.width(), device_rect.height());
        let new_device_pixel_scale = device_pixel_scale * Scale::new(scale);
        let new_device_rect = (device_rect.to_f32() * Scale::new(scale))
            .round_out()
            .to_i32();
        (new_device_rect, new_device_pixel_scale)
    } else {
        (device_rect.to_i32(), device_pixel_scale)
    }
}

/// Retrieve the exact unsnapped device space rectangle for a primitive.
fn get_unclipped_device_rect(
    prim_rect: PictureRect,
    map_to_raster: &SpaceMapper<PicturePixel, RasterPixel>,
    device_pixel_scale: DevicePixelScale,
) -> Option<DeviceRect> {
    let raster_rect = map_to_raster.map(&prim_rect)?;
    let world_rect = raster_rect * Scale::new(1.0);
    Some(world_rect * device_pixel_scale)
}

/// Given an unclipped device rect, try to find a minimal device space
/// rect to allocate a clip mask for, by clipping to the screen. This
/// function is very similar to get_raster_rects below. It is far from
/// ideal, and should be refactored as part of the support for setting
/// scale per-raster-root.
fn get_clipped_device_rect(
    unclipped: &DeviceRect,
    map_to_world: &SpaceMapper<RasterPixel, WorldPixel>,
    prim_bounding_rect: WorldRect,
    device_pixel_scale: DevicePixelScale,
) -> Option<DeviceRect> {
    let unclipped_raster_rect = {
        let world_rect = *unclipped * Scale::new(1.0);
        let raster_rect = world_rect * device_pixel_scale.inverse();

        raster_rect.cast_unit()
    };

    let unclipped_world_rect = map_to_world.map(&unclipped_raster_rect)?;

    let clipped_world_rect = unclipped_world_rect.intersection(&prim_bounding_rect)?;

    let clipped_raster_rect = map_to_world.unmap(&clipped_world_rect)?;

    let clipped_raster_rect = clipped_raster_rect.intersection(&unclipped_raster_rect)?;

    // Ensure that we won't try to allocate a zero-sized clip render task.
    if clipped_raster_rect.is_empty() {
        return None;
    }

    let clipped = raster_rect_to_device_pixels(
        clipped_raster_rect,
        device_pixel_scale,
    );

    Some(clipped)
}

pub fn get_raster_rects(
    pic_rect: PictureRect,
    map_to_raster: &SpaceMapper<PicturePixel, RasterPixel>,
    map_to_world: &SpaceMapper<RasterPixel, WorldPixel>,
    prim_bounding_rect: WorldRect,
    device_pixel_scale: DevicePixelScale,
) -> Option<(DeviceRect, DeviceRect)> {
    let unclipped_raster_rect = map_to_raster.map(&pic_rect)?;

    let unclipped = raster_rect_to_device_pixels(
        unclipped_raster_rect,
        device_pixel_scale,
    );

    let unclipped_world_rect = map_to_world.map(&unclipped_raster_rect)?;

    let clipped_world_rect = unclipped_world_rect.intersection(&prim_bounding_rect)?;

    let clipped_raster_rect = map_to_world.unmap(&clipped_world_rect)?;

    let clipped_raster_rect = clipped_raster_rect.intersection(&unclipped_raster_rect)?;

    let clipped = raster_rect_to_device_pixels(
        clipped_raster_rect,
        device_pixel_scale,
    );

    // Ensure that we won't try to allocate a zero-sized clip render task.
    if clipped.is_empty() {
        return None;
    }

    Some((clipped, unclipped))
}

/// Choose the decoration mask tile size for a given line.
///
/// Given a line with overall size `rect_size` and the given `orientation`,
/// return the dimensions of a single mask tile for the decoration pattern
/// described by `style` and `wavy_line_thickness`.
///
/// If `style` is `Solid`, no mask tile is necessary; return `None`. The other
/// styles each have their own characteristic periods of repetition, so for each
/// one, this function returns a `LayoutSize` with the right aspect ratio and
/// whose specific size is convenient for the `cs_line_decoration.glsl` fragment
/// shader to work with. The shader uses a local coordinate space in which the
/// tile fills a rectangle with one corner at the origin, and with the size this
/// function returns.
///
/// The returned size is not necessarily in pixels; device scaling and other
/// concerns can still affect the actual task size.
///
/// Regardless of whether `orientation` is `Vertical` or `Horizontal`, the
/// `width` and `height` of the returned size are always horizontal and
/// vertical, respectively.
pub fn get_line_decoration_size(
    rect_size: &LayoutSize,
    orientation: LineOrientation,
    style: LineStyle,
    wavy_line_thickness: f32,
) -> Option<LayoutSize> {
    let h = match orientation {
        LineOrientation::Horizontal => rect_size.height,
        LineOrientation::Vertical => rect_size.width,
    };

    // TODO(gw): The formulae below are based on the existing gecko and line
    //           shader code. They give reasonable results for most inputs,
    //           but could definitely do with a detailed pass to get better
    //           quality on a wider range of inputs!
    //           See nsCSSRendering::PaintDecorationLine in Gecko.

    let (parallel, perpendicular) = match style {
        LineStyle::Solid => {
            return None;
        }
        LineStyle::Dashed => {
            let dash_length = (3.0 * h).min(64.0).max(1.0);

            (2.0 * dash_length, 4.0)
        }
        LineStyle::Dotted => {
            let diameter = h.min(64.0).max(1.0);
            let period = 2.0 * diameter;

            (period, diameter)
        }
        LineStyle::Wavy => {
            let line_thickness = wavy_line_thickness.max(1.0);
            let slope_length = h - line_thickness;
            let flat_length = ((line_thickness - 1.0) * 2.0).max(1.0);
            let approx_period = 2.0 * (slope_length + flat_length);

            (approx_period, h)
        }
    };

    Some(match orientation {
        LineOrientation::Horizontal => LayoutSize::new(parallel, perpendicular),
        LineOrientation::Vertical => LayoutSize::new(perpendicular, parallel),
    })
}

fn update_opacity_binding(
    opacity_bindings: &mut OpacityBindingStorage,
    opacity_binding_index: OpacityBindingIndex,
    scene_properties: &SceneProperties,
) {
    if opacity_binding_index != OpacityBindingIndex::INVALID {
        let binding = &mut opacity_bindings[opacity_binding_index];
        binding.update(scene_properties);
    }
}

/// Trait for primitives that are directly internable.
/// see SceneBuilder::add_primitive<P>
pub trait InternablePrimitive: intern::Internable<InternData = ()> + Sized {
    /// Build a new key from self with `info`.
    fn into_key(
        self,
        info: &LayoutPrimitiveInfo,
    ) -> Self::Key;

    fn make_instance_kind(
        key: Self::Key,
        data_handle: intern::Handle<Self>,
        prim_store: &mut PrimitiveStore,
        reference_frame_relative_offset: LayoutVector2D,
    ) -> PrimitiveInstanceKind;
}


#[test]
#[cfg(target_pointer_width = "64")]
fn test_struct_sizes() {
    use std::mem;
    // The sizes of these structures are critical for performance on a number of
    // talos stress tests. If you get a failure here on CI, there's two possibilities:
    // (a) You made a structure smaller than it currently is. Great work! Update the
    //     test expectations and move on.
    // (b) You made a structure larger. This is not necessarily a problem, but should only
    //     be done with care, and after checking if talos performance regresses badly.
    assert_eq!(mem::size_of::<PrimitiveInstance>(), 80, "PrimitiveInstance size changed");
    assert_eq!(mem::size_of::<PrimitiveInstanceKind>(), 40, "PrimitiveInstanceKind size changed");
    assert_eq!(mem::size_of::<PrimitiveTemplate>(), 56, "PrimitiveTemplate size changed");
    assert_eq!(mem::size_of::<PrimitiveTemplateKind>(), 28, "PrimitiveTemplateKind size changed");
    assert_eq!(mem::size_of::<PrimitiveKey>(), 36, "PrimitiveKey size changed");
    assert_eq!(mem::size_of::<PrimitiveKeyKind>(), 16, "PrimitiveKeyKind size changed");
}
