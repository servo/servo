/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use api::{
    ColorF, ColorU, ExtendMode, GradientStop,
    PremultipliedColorF, LineOrientation,
};
use api::units::{LayoutPoint, LayoutSize, LayoutVector2D};
use crate::scene_building::IsVisible;
use euclid::approxeq::ApproxEq;
use crate::frame_builder::FrameBuildingState;
use crate::gpu_cache::{GpuCacheHandle, GpuDataRequest};
use crate::intern::{Internable, InternDebug, Handle as InternHandle};
use crate::internal_types::LayoutPrimitiveInfo;
use crate::prim_store::{BrushSegment, CachedGradientSegment, GradientTileRange, VectorKey};
use crate::prim_store::{PrimitiveInstanceKind, PrimitiveOpacity};
use crate::prim_store::{PrimKeyCommonData, PrimTemplateCommonData, PrimitiveStore};
use crate::prim_store::{NinePatchDescriptor, PointKey, SizeKey, InternablePrimitive};
use std::{hash, ops::{Deref, DerefMut}};
use crate::util::pack_as_float;
use crate::texture_cache::TEXTURE_REGION_DIMENSIONS;

/// The maximum number of stops a gradient may have to use the fast path.
pub const GRADIENT_FP_STOPS: usize = 4;

/// A hashable gradient stop that can be used in primitive keys.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(Debug, Copy, Clone, MallocSizeOf, PartialEq)]
pub struct GradientStopKey {
    pub offset: f32,
    pub color: ColorU,
}

impl GradientStopKey {
    pub fn empty() -> Self {
        GradientStopKey {
            offset: 0.0,
            color: ColorU::new(0, 0, 0, 0),
        }
    }
}

impl Into<GradientStopKey> for GradientStop {
    fn into(self) -> GradientStopKey {
        GradientStopKey {
            offset: self.offset,
            color: self.color.into(),
        }
    }
}

// Convert `stop_keys` into a vector of `GradientStop`s, which is a more
// convenient representation for the current gradient builder. Compute the
// minimum stop alpha along the way.
fn stops_and_min_alpha(stop_keys: &[GradientStopKey]) -> (Vec<GradientStop>, f32) {
    let mut min_alpha: f32 = 1.0;
    let stops = stop_keys.iter().map(|stop_key| {
        let color: ColorF = stop_key.color.into();
        min_alpha = min_alpha.min(color.a);

        GradientStop {
            offset: stop_key.offset,
            color,
        }
    }).collect();

    (stops, min_alpha)
}

impl Eq for GradientStopKey {}

impl hash::Hash for GradientStopKey {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.offset.to_bits().hash(state);
        self.color.hash(state);
    }
}

/// Identifying key for a linear gradient.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(Debug, Clone, Eq, PartialEq, Hash, MallocSizeOf)]
pub struct LinearGradientKey {
    pub common: PrimKeyCommonData,
    pub extend_mode: ExtendMode,
    pub start_point: PointKey,
    pub end_point: PointKey,
    pub stretch_size: SizeKey,
    pub tile_spacing: SizeKey,
    pub stops: Vec<GradientStopKey>,
    pub reverse_stops: bool,
    pub nine_patch: Option<Box<NinePatchDescriptor>>,
}

impl LinearGradientKey {
    pub fn new(
        info: &LayoutPrimitiveInfo,
        linear_grad: LinearGradient,
    ) -> Self {
        LinearGradientKey {
            common: info.into(),
            extend_mode: linear_grad.extend_mode,
            start_point: linear_grad.start_point,
            end_point: linear_grad.end_point,
            stretch_size: linear_grad.stretch_size,
            tile_spacing: linear_grad.tile_spacing,
            stops: linear_grad.stops,
            reverse_stops: linear_grad.reverse_stops,
            nine_patch: linear_grad.nine_patch,
        }
    }
}

impl InternDebug for LinearGradientKey {}

#[derive(Clone, Debug, Hash, MallocSizeOf, PartialEq, Eq)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct GradientCacheKey {
    pub orientation: LineOrientation,
    pub start_stop_point: VectorKey,
    pub stops: [GradientStopKey; GRADIENT_FP_STOPS],
}

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(MallocSizeOf)]
pub struct LinearGradientTemplate {
    pub common: PrimTemplateCommonData,
    pub extend_mode: ExtendMode,
    pub start_point: LayoutPoint,
    pub end_point: LayoutPoint,
    pub stretch_size: LayoutSize,
    pub tile_spacing: LayoutSize,
    pub stops_opacity: PrimitiveOpacity,
    pub stops: Vec<GradientStop>,
    pub brush_segments: Vec<BrushSegment>,
    pub reverse_stops: bool,
    pub stops_handle: GpuCacheHandle,
    /// If true, this gradient can be drawn via the fast path
    /// (cache gradient, and draw as image).
    pub supports_caching: bool,
}

impl Deref for LinearGradientTemplate {
    type Target = PrimTemplateCommonData;
    fn deref(&self) -> &Self::Target {
        &self.common
    }
}

impl DerefMut for LinearGradientTemplate {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.common
    }
}

impl From<LinearGradientKey> for LinearGradientTemplate {
    fn from(item: LinearGradientKey) -> Self {
        let common = PrimTemplateCommonData::with_key_common(item.common);

        // Check if we can draw this gradient via a fast path by caching the
        // gradient in a smaller task, and drawing as an image.
        // TODO(gw): Aim to reduce the constraints on fast path gradients in future,
        //           although this catches the vast majority of gradients on real pages.
        let mut supports_caching =
            // Gradient must cover entire primitive
            item.tile_spacing.w + item.stretch_size.w >= common.prim_rect.size.width &&
            item.tile_spacing.h + item.stretch_size.h >= common.prim_rect.size.height &&
            // Must be a vertical or horizontal gradient
            (item.start_point.x.approx_eq(&item.end_point.x) ||
             item.start_point.y.approx_eq(&item.end_point.y)) &&
            // Fast path not supported on segmented (border-image) gradients.
            item.nine_patch.is_none();

        // if we support caching and the gradient uses repeat, we might potentially
        // emit a lot of quads to cover the primitive. each quad will still cover
        // the entire gradient along the other axis, so the effect is linear in
        // display resolution, not quadratic (unlike say a tiny background image
        // tiling the display). in addition, excessive minification may lead to
        // texture trashing. so use the minification as a proxy heuristic for both
        // cases.
        //
        // note that the actual number of quads may be further increased due to
        // hard-stops and/or more than GRADIENT_FP_STOPS stops per gradient.
        if supports_caching && item.extend_mode == ExtendMode::Repeat {
            let single_repeat_size =
                if item.start_point.x.approx_eq(&item.end_point.x) {
                    item.end_point.y - item.start_point.y
                } else {
                    item.end_point.x - item.start_point.x
                };
            let downscaling = single_repeat_size as f32 / TEXTURE_REGION_DIMENSIONS as f32;
            if downscaling < 0.1 {
                // if a single copy of the gradient is this small relative to its baked
                // gradient cache, we have bad texture caching and/or too many quads.
                supports_caching = false;
            }
        }

        let (stops, min_alpha) = stops_and_min_alpha(&item.stops);

        let mut brush_segments = Vec::new();

        if let Some(ref nine_patch) = item.nine_patch {
            brush_segments = nine_patch.create_segments(common.prim_rect.size);
        }

        // Save opacity of the stops for use in
        // selecting which pass this gradient
        // should be drawn in.
        let stops_opacity = PrimitiveOpacity::from_alpha(min_alpha);

        LinearGradientTemplate {
            common,
            extend_mode: item.extend_mode,
            start_point: item.start_point.into(),
            end_point: item.end_point.into(),
            stretch_size: item.stretch_size.into(),
            tile_spacing: item.tile_spacing.into(),
            stops_opacity,
            stops,
            brush_segments,
            reverse_stops: item.reverse_stops,
            stops_handle: GpuCacheHandle::new(),
            supports_caching,
        }
    }
}

impl LinearGradientTemplate {
    /// Update the GPU cache for a given primitive template. This may be called multiple
    /// times per frame, by each primitive reference that refers to this interned
    /// template. The initial request call to the GPU cache ensures that work is only
    /// done if the cache entry is invalid (due to first use or eviction).
    pub fn update(
        &mut self,
        frame_state: &mut FrameBuildingState,
    ) {
        if let Some(mut request) =
            frame_state.gpu_cache.request(&mut self.common.gpu_cache_handle) {
            // write_prim_gpu_blocks
            request.push([
                self.start_point.x,
                self.start_point.y,
                self.end_point.x,
                self.end_point.y,
            ]);
            request.push([
                pack_as_float(self.extend_mode as u32),
                self.stretch_size.width,
                self.stretch_size.height,
                0.0,
            ]);

            // write_segment_gpu_blocks
            for segment in &self.brush_segments {
                // has to match VECS_PER_SEGMENT
                request.write_segment(
                    segment.local_rect,
                    segment.extra_data,
                );
            }
        }

        if let Some(mut request) = frame_state.gpu_cache.request(&mut self.stops_handle) {
            GradientGpuBlockBuilder::build(
                self.reverse_stops,
                &mut request,
                &self.stops,
            );
        }

        self.opacity = {
            // If the coverage of the gradient extends to or beyond
            // the primitive rect, then the opacity can be determined
            // by the colors of the stops. If we have tiling / spacing
            // then we just assume the gradient is translucent for now.
            // (In the future we could consider segmenting in some cases).
            let stride = self.stretch_size + self.tile_spacing;
            if stride.width >= self.common.prim_rect.size.width &&
               stride.height >= self.common.prim_rect.size.height {
                self.stops_opacity
            } else {
               PrimitiveOpacity::translucent()
            }
        }
    }
}

pub type LinearGradientDataHandle = InternHandle<LinearGradient>;

#[derive(Debug, MallocSizeOf)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct LinearGradient {
    pub extend_mode: ExtendMode,
    pub start_point: PointKey,
    pub end_point: PointKey,
    pub stretch_size: SizeKey,
    pub tile_spacing: SizeKey,
    pub stops: Vec<GradientStopKey>,
    pub reverse_stops: bool,
    pub nine_patch: Option<Box<NinePatchDescriptor>>,
}

impl Internable for LinearGradient {
    type Key = LinearGradientKey;
    type StoreData = LinearGradientTemplate;
    type InternData = ();
}

impl InternablePrimitive for LinearGradient {
    fn into_key(
        self,
        info: &LayoutPrimitiveInfo,
    ) -> LinearGradientKey {
        LinearGradientKey::new(info, self)
    }

    fn make_instance_kind(
        _key: LinearGradientKey,
        data_handle: LinearGradientDataHandle,
        prim_store: &mut PrimitiveStore,
        _reference_frame_relative_offset: LayoutVector2D,
    ) -> PrimitiveInstanceKind {
        let gradient_index = prim_store.linear_gradients.push(LinearGradientPrimitive {
            cache_segments: Vec::new(),
            visible_tiles_range: GradientTileRange::empty(),
        });

        PrimitiveInstanceKind::LinearGradient {
            data_handle,
            gradient_index,
        }
    }
}

impl IsVisible for LinearGradient {
    fn is_visible(&self) -> bool {
        true
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "capture", derive(Serialize))]
pub struct LinearGradientPrimitive {
    pub cache_segments: Vec<CachedGradientSegment>,
    pub visible_tiles_range: GradientTileRange,
}

////////////////////////////////////////////////////////////////////////////////

/// Hashable radial gradient parameters, for use during prim interning.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(Debug, Clone, MallocSizeOf, PartialEq)]
pub struct RadialGradientParams {
    pub start_radius: f32,
    pub end_radius: f32,
    pub ratio_xy: f32,
}

impl Eq for RadialGradientParams {}

impl hash::Hash for RadialGradientParams {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.start_radius.to_bits().hash(state);
        self.end_radius.to_bits().hash(state);
        self.ratio_xy.to_bits().hash(state);
    }
}

/// Identifying key for a radial gradient.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(Debug, Clone, Eq, PartialEq, Hash, MallocSizeOf)]
pub struct RadialGradientKey {
    pub common: PrimKeyCommonData,
    pub extend_mode: ExtendMode,
    pub center: PointKey,
    pub params: RadialGradientParams,
    pub stretch_size: SizeKey,
    pub stops: Vec<GradientStopKey>,
    pub tile_spacing: SizeKey,
    pub nine_patch: Option<Box<NinePatchDescriptor>>,
}

impl RadialGradientKey {
    pub fn new(
        info: &LayoutPrimitiveInfo,
        radial_grad: RadialGradient,
    ) -> Self {
        RadialGradientKey {
            common: info.into(),
            extend_mode: radial_grad.extend_mode,
            center: radial_grad.center,
            params: radial_grad.params,
            stretch_size: radial_grad.stretch_size,
            stops: radial_grad.stops,
            tile_spacing: radial_grad.tile_spacing,
            nine_patch: radial_grad.nine_patch,
        }
    }
}

impl InternDebug for RadialGradientKey {}

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(MallocSizeOf)]
pub struct RadialGradientTemplate {
    pub common: PrimTemplateCommonData,
    pub extend_mode: ExtendMode,
    pub center: LayoutPoint,
    pub params: RadialGradientParams,
    pub stretch_size: LayoutSize,
    pub tile_spacing: LayoutSize,
    pub brush_segments: Vec<BrushSegment>,
    pub stops_opacity: PrimitiveOpacity,
    pub stops: Vec<GradientStop>,
    pub stops_handle: GpuCacheHandle,
}

impl Deref for RadialGradientTemplate {
    type Target = PrimTemplateCommonData;
    fn deref(&self) -> &Self::Target {
        &self.common
    }
}

impl DerefMut for RadialGradientTemplate {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.common
    }
}

impl From<RadialGradientKey> for RadialGradientTemplate {
    fn from(item: RadialGradientKey) -> Self {
        let common = PrimTemplateCommonData::with_key_common(item.common);
        let mut brush_segments = Vec::new();

        if let Some(ref nine_patch) = item.nine_patch {
            brush_segments = nine_patch.create_segments(common.prim_rect.size);
        }

        let (stops, min_alpha) = stops_and_min_alpha(&item.stops);

        // Save opacity of the stops for use in
        // selecting which pass this gradient
        // should be drawn in.
        let stops_opacity = PrimitiveOpacity::from_alpha(min_alpha);

        RadialGradientTemplate {
            common,
            center: item.center.into(),
            extend_mode: item.extend_mode,
            params: item.params,
            stretch_size: item.stretch_size.into(),
            tile_spacing: item.tile_spacing.into(),
            brush_segments,
            stops_opacity,
            stops,
            stops_handle: GpuCacheHandle::new(),
        }
    }
}

impl RadialGradientTemplate {
    /// Update the GPU cache for a given primitive template. This may be called multiple
    /// times per frame, by each primitive reference that refers to this interned
    /// template. The initial request call to the GPU cache ensures that work is only
    /// done if the cache entry is invalid (due to first use or eviction).
    pub fn update(
        &mut self,
        frame_state: &mut FrameBuildingState,
    ) {
        if let Some(mut request) =
            frame_state.gpu_cache.request(&mut self.common.gpu_cache_handle) {
            // write_prim_gpu_blocks
            request.push([
                self.center.x,
                self.center.y,
                self.params.start_radius,
                self.params.end_radius,
            ]);
            request.push([
                self.params.ratio_xy,
                pack_as_float(self.extend_mode as u32),
                self.stretch_size.width,
                self.stretch_size.height,
            ]);

            // write_segment_gpu_blocks
            for segment in &self.brush_segments {
                // has to match VECS_PER_SEGMENT
                request.write_segment(
                    segment.local_rect,
                    segment.extra_data,
                );
            }
        }

        if let Some(mut request) = frame_state.gpu_cache.request(&mut self.stops_handle) {
            GradientGpuBlockBuilder::build(
                false,
                &mut request,
                &self.stops,
            );
        }

        self.opacity = PrimitiveOpacity::translucent();
    }
}

pub type RadialGradientDataHandle = InternHandle<RadialGradient>;

#[derive(Debug, MallocSizeOf)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct RadialGradient {
    pub extend_mode: ExtendMode,
    pub center: PointKey,
    pub params: RadialGradientParams,
    pub stretch_size: SizeKey,
    pub stops: Vec<GradientStopKey>,
    pub tile_spacing: SizeKey,
    pub nine_patch: Option<Box<NinePatchDescriptor>>,
}

impl Internable for RadialGradient {
    type Key = RadialGradientKey;
    type StoreData = RadialGradientTemplate;
    type InternData = ();
}

impl InternablePrimitive for RadialGradient {
    fn into_key(
        self,
        info: &LayoutPrimitiveInfo,
    ) -> RadialGradientKey {
        RadialGradientKey::new(info, self)
    }

    fn make_instance_kind(
        _key: RadialGradientKey,
        data_handle: RadialGradientDataHandle,
        _prim_store: &mut PrimitiveStore,
        _reference_frame_relative_offset: LayoutVector2D,
    ) -> PrimitiveInstanceKind {
        PrimitiveInstanceKind::RadialGradient {
            data_handle,
            visible_tiles_range: GradientTileRange::empty(),
        }
    }
}

impl IsVisible for RadialGradient {
    fn is_visible(&self) -> bool {
        true
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Conic gradients

/// Hashable conic gradient parameters, for use during prim interning.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(Debug, Clone, MallocSizeOf, PartialEq)]
pub struct ConicGradientParams {
    pub angle: f32, // in radians
    pub start_offset: f32,
    pub end_offset: f32,
}

impl Eq for ConicGradientParams {}

impl hash::Hash for ConicGradientParams {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.angle.to_bits().hash(state);
        self.start_offset.to_bits().hash(state);
        self.end_offset.to_bits().hash(state);
    }
}

/// Identifying key for a line decoration.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(Debug, Clone, Eq, PartialEq, Hash, MallocSizeOf)]
pub struct ConicGradientKey {
    pub common: PrimKeyCommonData,
    pub extend_mode: ExtendMode,
    pub center: PointKey,
    pub params: ConicGradientParams,
    pub stretch_size: SizeKey,
    pub stops: Vec<GradientStopKey>,
    pub tile_spacing: SizeKey,
    pub nine_patch: Option<Box<NinePatchDescriptor>>,
}

impl ConicGradientKey {
    pub fn new(
        info: &LayoutPrimitiveInfo,
        conic_grad: ConicGradient,
    ) -> Self {
        ConicGradientKey {
            common: info.into(),
            extend_mode: conic_grad.extend_mode,
            center: conic_grad.center,
            params: conic_grad.params,
            stretch_size: conic_grad.stretch_size,
            stops: conic_grad.stops,
            tile_spacing: conic_grad.tile_spacing,
            nine_patch: conic_grad.nine_patch,
        }
    }
}

impl InternDebug for ConicGradientKey {}

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(MallocSizeOf)]
pub struct ConicGradientTemplate {
    pub common: PrimTemplateCommonData,
    pub extend_mode: ExtendMode,
    pub center: LayoutPoint,
    pub params: ConicGradientParams,
    pub stretch_size: LayoutSize,
    pub tile_spacing: LayoutSize,
    pub brush_segments: Vec<BrushSegment>,
    pub stops_opacity: PrimitiveOpacity,
    pub stops: Vec<GradientStop>,
    pub stops_handle: GpuCacheHandle,
}

impl Deref for ConicGradientTemplate {
    type Target = PrimTemplateCommonData;
    fn deref(&self) -> &Self::Target {
        &self.common
    }
}

impl DerefMut for ConicGradientTemplate {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.common
    }
}

impl From<ConicGradientKey> for ConicGradientTemplate {
    fn from(item: ConicGradientKey) -> Self {
        let common = PrimTemplateCommonData::with_key_common(item.common);
        let mut brush_segments = Vec::new();

        if let Some(ref nine_patch) = item.nine_patch {
            brush_segments = nine_patch.create_segments(common.prim_rect.size);
        }

        let (stops, min_alpha) = stops_and_min_alpha(&item.stops);

        // Save opacity of the stops for use in
        // selecting which pass this gradient
        // should be drawn in.
        let stops_opacity = PrimitiveOpacity::from_alpha(min_alpha);

        ConicGradientTemplate {
            common,
            center: item.center.into(),
            extend_mode: item.extend_mode,
            params: item.params,
            stretch_size: item.stretch_size.into(),
            tile_spacing: item.tile_spacing.into(),
            brush_segments,
            stops_opacity,
            stops,
            stops_handle: GpuCacheHandle::new(),
        }
    }
}

impl ConicGradientTemplate {
    /// Update the GPU cache for a given primitive template. This may be called multiple
    /// times per frame, by each primitive reference that refers to this interned
    /// template. The initial request call to the GPU cache ensures that work is only
    /// done if the cache entry is invalid (due to first use or eviction).
    pub fn update(
        &mut self,
        frame_state: &mut FrameBuildingState,
    ) {
        if let Some(mut request) =
            frame_state.gpu_cache.request(&mut self.common.gpu_cache_handle) {
            // write_prim_gpu_blocks
            request.push([
                self.center.x,
                self.center.y,
                self.params.start_offset,
                self.params.end_offset,
            ]);
            request.push([
                self.params.angle,
                pack_as_float(self.extend_mode as u32),
                self.stretch_size.width,
                self.stretch_size.height,
            ]);

            // write_segment_gpu_blocks
            for segment in &self.brush_segments {
                // has to match VECS_PER_SEGMENT
                request.write_segment(
                    segment.local_rect,
                    segment.extra_data,
                );
            }
        }

        if let Some(mut request) = frame_state.gpu_cache.request(&mut self.stops_handle) {
            GradientGpuBlockBuilder::build(
                false,
                &mut request,
                &self.stops,
            );
        }

        self.opacity = PrimitiveOpacity::translucent();
    }
}

pub type ConicGradientDataHandle = InternHandle<ConicGradient>;

#[derive(Debug, MallocSizeOf)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct ConicGradient {
    pub extend_mode: ExtendMode,
    pub center: PointKey,
    pub params: ConicGradientParams,
    pub stretch_size: SizeKey,
    pub stops: Vec<GradientStopKey>,
    pub tile_spacing: SizeKey,
    pub nine_patch: Option<Box<NinePatchDescriptor>>,
}

impl Internable for ConicGradient {
    type Key = ConicGradientKey;
    type StoreData = ConicGradientTemplate;
    type InternData = ();
}

impl InternablePrimitive for ConicGradient {
    fn into_key(
        self,
        info: &LayoutPrimitiveInfo,
    ) -> ConicGradientKey {
        ConicGradientKey::new(info, self)
    }

    fn make_instance_kind(
        _key: ConicGradientKey,
        data_handle: ConicGradientDataHandle,
        _prim_store: &mut PrimitiveStore,
        _reference_frame_relative_offset: LayoutVector2D,
    ) -> PrimitiveInstanceKind {
        PrimitiveInstanceKind::ConicGradient {
            data_handle,
            visible_tiles_range: GradientTileRange::empty(),
        }
    }
}

impl IsVisible for ConicGradient {
    fn is_visible(&self) -> bool {
        true
    }
}

////////////////////////////////////////////////////////////////////////////////

// The gradient entry index for the first color stop
pub const GRADIENT_DATA_FIRST_STOP: usize = 0;
// The gradient entry index for the last color stop
pub const GRADIENT_DATA_LAST_STOP: usize = GRADIENT_DATA_SIZE - 1;

// The start of the gradient data table
pub const GRADIENT_DATA_TABLE_BEGIN: usize = GRADIENT_DATA_FIRST_STOP + 1;
// The exclusive bound of the gradient data table
pub const GRADIENT_DATA_TABLE_END: usize = GRADIENT_DATA_LAST_STOP;
// The number of entries in the gradient data table.
pub const GRADIENT_DATA_TABLE_SIZE: usize = 128;

// The number of entries in a gradient data: GRADIENT_DATA_TABLE_SIZE + first stop entry + last stop entry
pub const GRADIENT_DATA_SIZE: usize = GRADIENT_DATA_TABLE_SIZE + 2;

/// An entry in a gradient data table representing a segment of the gradient
/// color space.
#[derive(Debug, Copy, Clone)]
#[repr(C)]
struct GradientDataEntry {
    start_color: PremultipliedColorF,
    end_color: PremultipliedColorF,
}

impl GradientDataEntry {
    fn white() -> Self {
        Self {
            start_color: PremultipliedColorF::WHITE,
            end_color: PremultipliedColorF::WHITE,
        }
    }
}

// TODO(gw): Tidy this up to be a free function / module?
struct GradientGpuBlockBuilder {}

impl GradientGpuBlockBuilder {
    /// Generate a color ramp filling the indices in [start_idx, end_idx) and interpolating
    /// from start_color to end_color.
    fn fill_colors(
        start_idx: usize,
        end_idx: usize,
        start_color: &PremultipliedColorF,
        end_color: &PremultipliedColorF,
        entries: &mut [GradientDataEntry; GRADIENT_DATA_SIZE],
    ) {
        // Calculate the color difference for individual steps in the ramp.
        let inv_steps = 1.0 / (end_idx - start_idx) as f32;
        let step_r = (end_color.r - start_color.r) * inv_steps;
        let step_g = (end_color.g - start_color.g) * inv_steps;
        let step_b = (end_color.b - start_color.b) * inv_steps;
        let step_a = (end_color.a - start_color.a) * inv_steps;

        let mut cur_color = *start_color;

        // Walk the ramp writing start and end colors for each entry.
        for index in start_idx .. end_idx {
            let entry = &mut entries[index];
            entry.start_color = cur_color;
            cur_color.r += step_r;
            cur_color.g += step_g;
            cur_color.b += step_b;
            cur_color.a += step_a;
            entry.end_color = cur_color;
        }
    }

    /// Compute an index into the gradient entry table based on a gradient stop offset. This
    /// function maps offsets from [0, 1] to indices in [GRADIENT_DATA_TABLE_BEGIN, GRADIENT_DATA_TABLE_END].
    #[inline]
    fn get_index(offset: f32) -> usize {
        (offset.max(0.0).min(1.0) * GRADIENT_DATA_TABLE_SIZE as f32 +
            GRADIENT_DATA_TABLE_BEGIN as f32)
            .round() as usize
    }

    // Build the gradient data from the supplied stops, reversing them if necessary.
    fn build(
        reverse_stops: bool,
        request: &mut GpuDataRequest,
        src_stops: &[GradientStop],
    ) {
        // Preconditions (should be ensured by DisplayListBuilder):
        // * we have at least two stops
        // * first stop has offset 0.0
        // * last stop has offset 1.0
        let mut src_stops = src_stops.into_iter();
        let mut cur_color = match src_stops.next() {
            Some(stop) => {
                debug_assert_eq!(stop.offset, 0.0);
                stop.color.premultiplied()
            }
            None => {
                error!("Zero gradient stops found!");
                PremultipliedColorF::BLACK
            }
        };

        // A table of gradient entries, with two colors per entry, that specify the start and end color
        // within the segment of the gradient space represented by that entry. To lookup a gradient result,
        // first the entry index is calculated to determine which two colors to interpolate between, then
        // the offset within that entry bucket is used to interpolate between the two colors in that entry.
        // This layout preserves hard stops, as the end color for a given entry can differ from the start
        // color for the following entry, despite them being adjacent. Colors are stored within in BGRA8
        // format for texture upload. This table requires the gradient color stops to be normalized to the
        // range [0, 1]. The first and last entries hold the first and last color stop colors respectively,
        // while the entries in between hold the interpolated color stop values for the range [0, 1].
        let mut entries = [GradientDataEntry::white(); GRADIENT_DATA_SIZE];

        if reverse_stops {
            // Fill in the first entry (for reversed stops) with the first color stop
            GradientGpuBlockBuilder::fill_colors(
                GRADIENT_DATA_LAST_STOP,
                GRADIENT_DATA_LAST_STOP + 1,
                &cur_color,
                &cur_color,
                &mut entries,
            );

            // Fill in the center of the gradient table, generating a color ramp between each consecutive pair
            // of gradient stops. Each iteration of a loop will fill the indices in [next_idx, cur_idx). The
            // loop will then fill indices in [GRADIENT_DATA_TABLE_BEGIN, GRADIENT_DATA_TABLE_END).
            let mut cur_idx = GRADIENT_DATA_TABLE_END;
            for next in src_stops {
                let next_color = next.color.premultiplied();
                let next_idx = Self::get_index(1.0 - next.offset);

                if next_idx < cur_idx {
                    GradientGpuBlockBuilder::fill_colors(
                        next_idx,
                        cur_idx,
                        &next_color,
                        &cur_color,
                        &mut entries,
                    );
                    cur_idx = next_idx;
                }

                cur_color = next_color;
            }
            if cur_idx != GRADIENT_DATA_TABLE_BEGIN {
                error!("Gradient stops abruptly at {}, auto-completing to white", cur_idx);
            }

            // Fill in the last entry (for reversed stops) with the last color stop
            GradientGpuBlockBuilder::fill_colors(
                GRADIENT_DATA_FIRST_STOP,
                GRADIENT_DATA_FIRST_STOP + 1,
                &cur_color,
                &cur_color,
                &mut entries,
            );
        } else {
            // Fill in the first entry with the first color stop
            GradientGpuBlockBuilder::fill_colors(
                GRADIENT_DATA_FIRST_STOP,
                GRADIENT_DATA_FIRST_STOP + 1,
                &cur_color,
                &cur_color,
                &mut entries,
            );

            // Fill in the center of the gradient table, generating a color ramp between each consecutive pair
            // of gradient stops. Each iteration of a loop will fill the indices in [cur_idx, next_idx). The
            // loop will then fill indices in [GRADIENT_DATA_TABLE_BEGIN, GRADIENT_DATA_TABLE_END).
            let mut cur_idx = GRADIENT_DATA_TABLE_BEGIN;
            for next in src_stops {
                let next_color = next.color.premultiplied();
                let next_idx = Self::get_index(next.offset);

                if next_idx > cur_idx {
                    GradientGpuBlockBuilder::fill_colors(
                        cur_idx,
                        next_idx,
                        &cur_color,
                        &next_color,
                        &mut entries,
                    );
                    cur_idx = next_idx;
                }

                cur_color = next_color;
            }
            if cur_idx != GRADIENT_DATA_TABLE_END {
                error!("Gradient stops abruptly at {}, auto-completing to white", cur_idx);
            }

            // Fill in the last entry with the last color stop
            GradientGpuBlockBuilder::fill_colors(
                GRADIENT_DATA_LAST_STOP,
                GRADIENT_DATA_LAST_STOP + 1,
                &cur_color,
                &cur_color,
                &mut entries,
            );
        }

        for entry in entries.iter() {
            request.push(entry.start_color);
            request.push(entry.end_color);
        }
    }
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
    assert_eq!(mem::size_of::<LinearGradient>(), 72, "LinearGradient size changed");
    assert_eq!(mem::size_of::<LinearGradientTemplate>(), 120, "LinearGradientTemplate size changed");
    assert_eq!(mem::size_of::<LinearGradientKey>(), 88, "LinearGradientKey size changed");

    assert_eq!(mem::size_of::<RadialGradient>(), 72, "RadialGradient size changed");
    assert_eq!(mem::size_of::<RadialGradientTemplate>(), 128, "RadialGradientTemplate size changed");
    assert_eq!(mem::size_of::<RadialGradientKey>(), 96, "RadialGradientKey size changed");

    assert_eq!(mem::size_of::<ConicGradient>(), 72, "ConicGradient size changed");
    assert_eq!(mem::size_of::<ConicGradientTemplate>(), 128, "ConicGradientTemplate size changed");
    assert_eq!(mem::size_of::<ConicGradientKey>(), 96, "ConicGradientKey size changed");
}
