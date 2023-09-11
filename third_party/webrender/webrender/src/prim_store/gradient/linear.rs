/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Linear gradients
//!
//! Specification: https://drafts.csswg.org/css-images-4/#linear-gradients
//!
//! Linear gradients are rendered via cached render tasks and composited with the image brush.

use euclid::approxeq::ApproxEq;
use euclid::{point2, vec2, size2};
use api::{ExtendMode, GradientStop, LineOrientation, PremultipliedColorF, ColorF, ColorU};
use api::units::*;
use crate::scene_building::IsVisible;
use crate::frame_builder::FrameBuildingState;
use crate::gpu_cache::{GpuCache, GpuCacheHandle};
use crate::intern::{Internable, InternDebug, Handle as InternHandle};
use crate::internal_types::LayoutPrimitiveInfo;
use crate::image_tiling::simplify_repeated_primitive;
use crate::prim_store::{BrushSegment, GradientTileRange};
use crate::prim_store::{PrimitiveInstanceKind, PrimitiveOpacity};
use crate::prim_store::{PrimKeyCommonData, PrimTemplateCommonData, PrimitiveStore};
use crate::prim_store::{NinePatchDescriptor, PointKey, SizeKey, InternablePrimitive};
use crate::render_task::{RenderTask, RenderTaskKind};
use crate::render_task_graph::RenderTaskId;
use crate::render_task_cache::{RenderTaskCacheKeyKind, RenderTaskCacheKey, RenderTaskParent};
use crate::picture::{SurfaceIndex};
use crate::util::pack_as_float;
use super::{stops_and_min_alpha, GradientStopKey, GradientGpuBlockBuilder, apply_gradient_local_clip};
use std::ops::{Deref, DerefMut};
use std::mem::swap;

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
    pub cached: bool,
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
            cached: linear_grad.cached,
            nine_patch: linear_grad.nine_patch,
        }
    }
}

impl InternDebug for LinearGradientKey {}

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(Debug, MallocSizeOf)]
pub struct LinearGradientTemplate {
    pub common: PrimTemplateCommonData,
    pub extend_mode: ExtendMode,
    pub start_point: DevicePoint,
    pub end_point: DevicePoint,
    pub task_size: DeviceIntSize,
    pub scale: DeviceVector2D,
    pub stretch_size: LayoutSize,
    pub tile_spacing: LayoutSize,
    pub stops_opacity: PrimitiveOpacity,
    pub stops: Vec<GradientStop>,
    pub brush_segments: Vec<BrushSegment>,
    pub reverse_stops: bool,
    pub is_fast_path: bool,
    pub cached: bool,
    pub stops_handle: GpuCacheHandle,
    pub src_color: Option<RenderTaskId>,
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

/// Perform a few optimizations to the gradient that are relevant to scene building.
///
/// Returns true if the gradient was decomposed into fast-path primitives, indicating
/// that we shouldn't emit a regular gradient primitive after this returns.
pub fn optimize_linear_gradient(
    prim_rect: &mut LayoutRect,
    tile_size: &mut LayoutSize,
    mut tile_spacing: LayoutSize,
    clip_rect: &LayoutRect,
    start: &mut LayoutPoint,
    end: &mut LayoutPoint,
    extend_mode: ExtendMode,
    stops: &mut [GradientStopKey],
    // Callback called for each fast-path segment (rect, start end, stops).
    callback: &mut dyn FnMut(&LayoutRect, LayoutPoint, LayoutPoint, &[GradientStopKey])
) -> bool {
    // First sanitize the gradient parameters. See if we can remove repetitions,
    // tighten the primitive bounds, etc.

    // The size of gradient render tasks depends on the tile_size. No need to generate
    // large stretch sizes that will be clipped to the bounds of the primitive.
    tile_size.width = tile_size.width.min(prim_rect.size.width);
    tile_size.height = tile_size.height.min(prim_rect.size.height);

    simplify_repeated_primitive(&tile_size, &mut tile_spacing, prim_rect);

    let vertical = start.x.approx_eq(&end.x);
    let horizontal = start.y.approx_eq(&end.y);

    let mut horizontally_tiled = prim_rect.size.width > tile_size.width;
    let mut vertically_tiled = prim_rect.size.height > tile_size.height;

    // Check whether the tiling is equivalent to stretching on either axis.
    // Stretching the gradient is more efficient than repeating it.
    if vertically_tiled && horizontal && tile_spacing.height == 0.0 {
        tile_size.height = prim_rect.size.height;
        vertically_tiled = false;
    }

    if horizontally_tiled && vertical && tile_spacing.width == 0.0 {
        tile_size.width = prim_rect.size.width;
        horizontally_tiled = false;
    }

    let offset = apply_gradient_local_clip(
        prim_rect,
        &tile_size,
        &tile_spacing,
        &clip_rect
    );

    *start += offset;
    *end += offset;

    // Next, in the case of axis-aligned gradients, see if it is worth
    // decomposing the gradient into multiple gradients with only two
    // gradient stops per segment to get a faster shader.

    if extend_mode != ExtendMode::Clamp || stops.is_empty() {
        return false;
    }

    if !vertical && !horizontal {
        return false;
    }

    if vertical && horizontal {
        return false;
    }

    if !tile_spacing.is_empty() || vertically_tiled || horizontally_tiled {
        return false;
    }

    // If the gradient is small, no need to bother with decomposing it.
    if (horizontal && tile_size.width < 256.0)
        || (vertical && tile_size.height < 256.0) {

        return false;
    }

    // Flip x and y if need be so that we only deal with the horizontal case.

    // From now on don't return false. We are going modifying the caller's
    // variables and not bother to restore them. If the control flow changes,
    // Make sure to to restore &mut parameters to sensible values before
    // returning false.

    let adjust_rect = &mut |rect: &mut LayoutRect| {
        if vertical {
            swap(&mut rect.origin.x, &mut rect.origin.y);
            swap(&mut rect.size.width, &mut rect.size.height);
        }
    };

    let adjust_size = &mut |size: &mut LayoutSize| {
        if vertical { swap(&mut size.width, &mut size.height); }
    };

    let adjust_point = &mut |p: &mut LayoutPoint| {
        if vertical { swap(&mut p.x, &mut p.y); }
    };

    let clip_rect = match clip_rect.intersection(prim_rect) {
        Some(clip) => clip,
        None => {
            return false;
        }
    };

    adjust_rect(prim_rect);
    adjust_point(start);
    adjust_point(end);
    adjust_size(tile_size);

    let length = (end.x - start.x).abs();

    // Decompose the gradient into simple segments. This lets us:
    // - separate opaque from semi-transparent segments,
    // - compress long segments into small render tasks,
    // - make sure hard stops stay so even if the primitive is large.

    let reverse_stops = start.x > end.x;

    // Handle reverse stops so we can assume stops are arranged in increasing x.
    if reverse_stops {
        stops.reverse();
        swap(start, end);
    }

    // Use fake gradient stop to emulate the potential constant color sections
    // before and after the gradient endpoints.
    let mut prev = *stops.first().unwrap();
    let mut last = *stops.last().unwrap();

    // Set the offsets of the fake stops to position them at the edges of the primitive.
    prev.offset = -start.x / length;
    last.offset = (tile_size.width - start.x) / length;
    if reverse_stops {
        prev.offset = 1.0 - prev.offset;
        last.offset = 1.0 - last.offset;
    }

    for stop in stops.iter().chain((&[last]).iter()) {
        let prev_stop = prev;
        prev = *stop;

        if prev_stop.color.a == 0 && stop.color.a == 0 {
            continue;
        }


        let prev_offset = if reverse_stops { 1.0 - prev_stop.offset } else { prev_stop.offset };
        let offset = if reverse_stops { 1.0 - stop.offset } else { stop.offset };

        // In layout space, relative to the primitive.
        let segment_start = start.x + prev_offset * length;
        let segment_end = start.x + offset * length;
        let segment_length = segment_end - segment_start;

        if segment_length <= 0.0 {
            continue;
        }

        let mut segment_rect = *prim_rect;
        segment_rect.origin.x += segment_start;
        segment_rect.size.width = segment_length;

        let mut start = point2(0.0, 0.0);
        let mut end = point2(segment_length, 0.0);

        adjust_point(&mut start);
        adjust_point(&mut end);
        adjust_rect(&mut segment_rect);

        let origin_before_clip = segment_rect.origin;
        segment_rect = match segment_rect.intersection(&clip_rect) {
            Some(rect) => rect,
            None => {
                continue;
            }
        };
        let offset = segment_rect.origin - origin_before_clip;

        // Account for the clipping since start and end are relative to the origin.
        start -= offset;
        end -= offset;

        callback(
            &segment_rect,
            start,
            end,
            &[
                GradientStopKey { offset: 0.0, .. prev_stop },
                GradientStopKey { offset: 1.0, .. *stop },
            ],
        );
    }

    true
}

impl From<LinearGradientKey> for LinearGradientTemplate {
    fn from(item: LinearGradientKey) -> Self {

        let common = PrimTemplateCommonData::with_key_common(item.common);

        let (mut stops, min_alpha) = stops_and_min_alpha(&item.stops);

        let mut brush_segments = Vec::new();

        if let Some(ref nine_patch) = item.nine_patch {
            brush_segments = nine_patch.create_segments(common.prim_rect.size);
        }

        // Save opacity of the stops for use in
        // selecting which pass this gradient
        // should be drawn in.
        let stops_opacity = PrimitiveOpacity::from_alpha(min_alpha);

        let start_point = DevicePoint::new(item.start_point.x, item.start_point.y);
        let end_point = DevicePoint::new(item.end_point.x, item.end_point.y);
        let tile_spacing: LayoutSize = item.tile_spacing.into();
        let stretch_size: LayoutSize = item.stretch_size.into();
        let mut task_size: DeviceSize = stretch_size.cast_unit();

        let horizontal = start_point.y.approx_eq(&end_point.y);
        let vertical = start_point.x.approx_eq(&end_point.x);

        if horizontal {
            // Completely horizontal, we can stretch the gradient vertically.
            task_size.height = 1.0;
        }

        if vertical {
            // Completely vertical, we can stretch the gradient horizontally.
            task_size.width = 1.0;
        }

        // See if we can render the gradient using a special fast-path shader.
        // The fast path path only works with two gradient stops.
        let mut is_fast_path = false;
        if item.cached && stops.len() == 2 && brush_segments.is_empty() {
            if horizontal
                && stretch_size.width >= common.prim_rect.width()
                && start_point.x.approx_eq(&0.0)
                && end_point.x.approx_eq(&stretch_size.width) {
                is_fast_path = true;
                task_size.width = task_size.width.min(256.0);
            }
            if vertical
                && stretch_size.height >= common.prim_rect.height()
                && start_point.y.approx_eq(&0.0)
                && end_point.y.approx_eq(&stretch_size.height) {
                is_fast_path = true;
                task_size.height = task_size.height.min(256.0);
            }

            if stops[0].color == stops[1].color {
                is_fast_path = true;
                task_size = size2(1.0, 1.0);
            }

            if is_fast_path && item.reverse_stops {
                // The fast path doesn't use the gradient gpu blocks builder so handle
                // reversed stops here.
                stops.swap(0, 1);
            }
        }

        // Avoid rendering enormous gradients. Radial gradients are mostly made of soft transitions,
        // so it is unlikely that rendering at a higher resolution than 1024 would produce noticeable
        // differences, especially with 8 bits per channel.
        const MAX_SIZE: f32 = 1024.0;

        let mut scale = vec2(1.0, 1.0);

        if task_size.width > MAX_SIZE {
            scale.x = task_size.width / MAX_SIZE;
            task_size.width = MAX_SIZE;
        }

        if task_size.height > MAX_SIZE {
            scale.y = task_size.height / MAX_SIZE;
            task_size.height = MAX_SIZE;
        }

        LinearGradientTemplate {
            common,
            extend_mode: item.extend_mode,
            start_point,
            end_point,
            task_size: task_size.ceil().to_i32(),
            scale,
            stretch_size,
            tile_spacing,
            stops_opacity,
            stops,
            brush_segments,
            reverse_stops: item.reverse_stops,
            is_fast_path,
            cached: item.cached,
            stops_handle: GpuCacheHandle::new(),
            src_color: None,
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
        parent_surface: SurfaceIndex,
    ) {
        if let Some(mut request) = frame_state.gpu_cache.request(
            &mut self.common.gpu_cache_handle
        ) {

            // Write_prim_gpu_blocks
            if self.cached {
                // We are using the image brush.
                request.push(PremultipliedColorF::WHITE);
                request.push(PremultipliedColorF::WHITE);
                request.push([
                    self.stretch_size.width,
                    self.stretch_size.height,
                    0.0,
                    0.0,
                ]);
            } else {
                // We are using the gradient brush.
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
            }

            // write_segment_gpu_blocks
            for segment in &self.brush_segments {
                // has to match VECS_PER_SEGMENT
                request.write_segment(
                    segment.local_rect,
                    segment.extra_data,
                );
            }
        }

        // The fast path passes gradient stops as vertex attributes directly.
        if !self.is_fast_path {
            if let Some(mut request) = frame_state.gpu_cache.request(&mut self.stops_handle) {
                GradientGpuBlockBuilder::build(
                    self.reverse_stops,
                    &mut request,
                    &self.stops,
                );
            }
        }

        // Tile spacing is always handled by decomposing into separate draw calls so the
        // primitive opacity is equivalent to stops opacity. This might change to being
        // set to non-opaque in the presence of tile spacing if/when tile spacing is handled
        // in the same way as with the image primitive.
        self.opacity = self.stops_opacity;

        if !self.cached {
            return;
        }

        let task_id = if self.is_fast_path {
            let orientation = if self.task_size.width > self.task_size.height {
                LineOrientation::Horizontal
            } else {
                LineOrientation::Vertical
            };

            let gradient = FastLinearGradientTask {
                color0: self.stops[0].color.into(),
                color1: self.stops[1].color.into(),
                orientation,
            };

            frame_state.resource_cache.request_render_task(
                RenderTaskCacheKey {
                    size: self.task_size,
                    kind: RenderTaskCacheKeyKind::FastLinearGradient(gradient),
                },
                frame_state.gpu_cache,
                frame_state.rg_builder,
                None,
                false,
                RenderTaskParent::Surface(parent_surface),
                frame_state.surfaces,
                |rg_builder| {
                    rg_builder.add().init(RenderTask::new_dynamic(
                        self.task_size,
                        RenderTaskKind::FastLinearGradient(gradient),
                    ))
                }
            )
        } else {
            let cache_key = LinearGradientCacheKey {
                size: self.task_size,
                start: PointKey { x: self.start_point.x, y: self.start_point.y },
                end: PointKey { x: self.end_point.x, y: self.end_point.y },
                scale: PointKey { x: self.scale.x, y: self.scale.y },
                extend_mode: self.extend_mode,
                stops: self.stops.iter().map(|stop| (*stop).into()).collect(),
                reversed_stops: self.reverse_stops,
            };

            frame_state.resource_cache.request_render_task(
                RenderTaskCacheKey {
                    size: self.task_size,
                    kind: RenderTaskCacheKeyKind::LinearGradient(cache_key),
                },
                frame_state.gpu_cache,
                frame_state.rg_builder,
                None,
                false,
                RenderTaskParent::Surface(parent_surface),
                frame_state.surfaces,
                |rg_builder| {
                    rg_builder.add().init(RenderTask::new_dynamic(
                        self.task_size,
                        RenderTaskKind::LinearGradient(LinearGradientTask {
                            start: self.start_point,
                            end: self.end_point,
                            scale: self.scale,
                            extend_mode: self.extend_mode,
                            stops: self.stops_handle,
                        }),
                    ))
                }
            )
        };

        self.src_color = Some(task_id);
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
    pub cached: bool,
}

impl Internable for LinearGradient {
    type Key = LinearGradientKey;
    type StoreData = LinearGradientTemplate;
    type InternData = ();
    const PROFILE_COUNTER: usize = crate::profiler::INTERNED_LINEAR_GRADIENTS;
}

impl InternablePrimitive for LinearGradient {
    fn into_key(
        self,
        info: &LayoutPrimitiveInfo,
    ) -> LinearGradientKey {
        LinearGradientKey::new(info, self)
    }

    fn make_instance_kind(
        key: LinearGradientKey,
        data_handle: LinearGradientDataHandle,
        _prim_store: &mut PrimitiveStore,
        _reference_frame_relative_offset: LayoutVector2D,
    ) -> PrimitiveInstanceKind {
        if key.cached {
            PrimitiveInstanceKind::CachedLinearGradient {
                data_handle,
                visible_tiles_range: GradientTileRange::empty(),
            }
        } else {
            PrimitiveInstanceKind::LinearGradient {
                data_handle,
                visible_tiles_range: GradientTileRange::empty(),
            }
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

#[derive(Debug)]
#[cfg_attr(feature = "capture", derive(Serialize))]
pub struct CachedGradientSegment {
    pub render_task: RenderTaskId,
    pub local_rect: LayoutRect,
}


#[derive(Copy, Clone, Debug, Hash, MallocSizeOf, PartialEq, Eq)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct FastLinearGradientTask {
    pub color0: ColorU,
    pub color1: ColorU,
    pub orientation: LineOrientation,
}

impl FastLinearGradientTask {
    pub fn to_instance(&self, target_rect: &DeviceIntRect) -> FastLinearGradientInstance {
        FastLinearGradientInstance {
            task_rect: target_rect.to_f32(),
            color0: ColorF::from(self.color0).premultiplied(),
            color1: ColorF::from(self.color1).premultiplied(),
            axis_select: match self.orientation {
                LineOrientation::Horizontal => 0.0,
                LineOrientation::Vertical => 1.0,
            },
        }
    }
}

pub type FastLinearGradientCacheKey = FastLinearGradientTask;

/// The per-instance shader input of a fast-path linear gradient render task.
///
/// Must match the FAST_LINEAR_GRADIENT instance description in renderer/vertex.rs.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[repr(C)]
#[derive(Clone, Debug)]
pub struct FastLinearGradientInstance {
    pub task_rect: DeviceRect,
    pub color0: PremultipliedColorF,
    pub color1: PremultipliedColorF,
    pub axis_select: f32,
}

#[derive(Debug)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct LinearGradientTask {
    pub start: DevicePoint,
    pub end: DevicePoint,
    pub scale: DeviceVector2D,
    pub extend_mode: ExtendMode,
    pub stops: GpuCacheHandle,
}

impl LinearGradientTask {
    pub fn to_instance(&self, target_rect: &DeviceIntRect, gpu_cache: &mut GpuCache) -> LinearGradientInstance {
        LinearGradientInstance {
            task_rect: target_rect.to_f32(),
            start: self.start,
            end: self.end,
            scale: self.scale,
            extend_mode: self.extend_mode as i32,
            gradient_stops_address: self.stops.as_int(gpu_cache),
        }
    }
}

/// The per-instance shader input of a linear gradient render task.
///
/// Must match the LINEAR_GRADIENT instance description in renderer/vertex.rs.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[repr(C)]
#[derive(Clone, Debug)]
pub struct LinearGradientInstance {
    pub task_rect: DeviceRect,
    pub start: DevicePoint,
    pub end: DevicePoint,
    pub scale: DeviceVector2D,
    pub extend_mode: i32,
    pub gradient_stops_address: i32,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct LinearGradientCacheKey {
    pub size: DeviceIntSize,
    pub start: PointKey,
    pub end: PointKey,
    pub scale: PointKey,
    pub extend_mode: ExtendMode,
    pub stops: Vec<GradientStopKey>,
    pub reversed_stops: bool,
}
