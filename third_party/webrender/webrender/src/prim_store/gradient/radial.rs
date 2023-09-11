/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Radial gradients
//!
//! Specification: https://drafts.csswg.org/css-images-4/#radial-gradients
//!
//! Radial gradients are rendered via cached render tasks and composited with the image brush.

use euclid::{vec2, size2};
use api::{ExtendMode, GradientStop, PremultipliedColorF, ColorU};
use api::units::*;
use crate::scene_building::IsVisible;
use crate::frame_builder::FrameBuildingState;
use crate::gpu_cache::{GpuCache, GpuCacheHandle};
use crate::intern::{Internable, InternDebug, Handle as InternHandle};
use crate::internal_types::LayoutPrimitiveInfo;
use crate::prim_store::{BrushSegment, GradientTileRange, InternablePrimitive};
use crate::prim_store::{PrimitiveInstanceKind, PrimitiveOpacity};
use crate::prim_store::{PrimKeyCommonData, PrimTemplateCommonData, PrimitiveStore};
use crate::prim_store::{NinePatchDescriptor, PointKey, SizeKey, FloatKey};
use crate::render_task::{RenderTask, RenderTaskKind};
use crate::render_task_graph::RenderTaskId;
use crate::render_task_cache::{RenderTaskCacheKeyKind, RenderTaskCacheKey, RenderTaskParent};
use crate::picture::{SurfaceIndex};

use std::{hash, ops::{Deref, DerefMut}};
use super::{
    stops_and_min_alpha, GradientStopKey, GradientGpuBlockBuilder,
    apply_gradient_local_clip,
};

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
#[derive(Debug)]
pub struct RadialGradientTemplate {
    pub common: PrimTemplateCommonData,
    pub extend_mode: ExtendMode,
    pub params: RadialGradientParams,
    pub center: DevicePoint,
    pub task_size: DeviceIntSize,
    pub scale: DeviceVector2D,
    pub stretch_size: LayoutSize,
    pub tile_spacing: LayoutSize,
    pub brush_segments: Vec<BrushSegment>,
    pub stops_opacity: PrimitiveOpacity,
    pub stops: Vec<GradientStop>,
    pub stops_handle: GpuCacheHandle,
    pub src_color: Option<RenderTaskId>,
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

        let mut stretch_size: LayoutSize = item.stretch_size.into();
        stretch_size.width = stretch_size.width.min(common.prim_rect.size.width);
        stretch_size.height = stretch_size.height.min(common.prim_rect.size.height);

        // Avoid rendering enormous gradients. Radial gradients are mostly made of soft transitions,
        // so it is unlikely that rendering at a higher resolution that 1024 would produce noticeable
        // differences, especially with 8 bits per channel.
        const MAX_SIZE: f32 = 1024.0;
        let mut task_size: DeviceSize = stretch_size.cast_unit();
        let mut scale = vec2(1.0, 1.0);
        if task_size.width > MAX_SIZE {
            scale.x = task_size.width/ MAX_SIZE;
            task_size.width = MAX_SIZE;
        }
        if task_size.height > MAX_SIZE {
            scale.y = task_size.height /MAX_SIZE;
            task_size.height = MAX_SIZE;
        }

        RadialGradientTemplate {
            common,
            center: DevicePoint::new(item.center.x, item.center.y),
            extend_mode: item.extend_mode,
            params: item.params,
            stretch_size,
            task_size: task_size.ceil().to_i32(),
            scale,
            tile_spacing: item.tile_spacing.into(),
            brush_segments,
            stops_opacity,
            stops,
            stops_handle: GpuCacheHandle::new(),
            src_color: None,
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
        parent_surface: SurfaceIndex,
    ) {
        if let Some(mut request) =
            frame_state.gpu_cache.request(&mut self.common.gpu_cache_handle) {
            // write_prim_gpu_blocks
            request.push(PremultipliedColorF::WHITE);
            request.push(PremultipliedColorF::WHITE);
            request.push([
                self.stretch_size.width,
                self.stretch_size.height,
                0.0,
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
                false,
                &mut request,
                &self.stops,
            );
        }

        let task_size = self.task_size;
        let cache_key = RadialGradientCacheKey {
            size: task_size,
            center: PointKey { x: self.center.x, y: self.center.y },
            scale: PointKey { x: self.scale.x, y: self.scale.y },
            start_radius: FloatKey(self.params.start_radius),
            end_radius: FloatKey(self.params.end_radius),
            ratio_xy: FloatKey(self.params.ratio_xy),
            extend_mode: self.extend_mode,
            stops: self.stops.iter().map(|stop| (*stop).into()).collect(),
        };

        let task_id = frame_state.resource_cache.request_render_task(
            RenderTaskCacheKey {
                size: task_size,
                kind: RenderTaskCacheKeyKind::RadialGradient(cache_key),
            },
            frame_state.gpu_cache,
            frame_state.rg_builder,
            None,
            false,
            RenderTaskParent::Surface(parent_surface),
            frame_state.surfaces,
            |rg_builder| {
                rg_builder.add().init(RenderTask::new_dynamic(
                    task_size,
                    RenderTaskKind::RadialGradient(RadialGradientTask {
                        extend_mode: self.extend_mode,
                        center: self.center,
                        scale: self.scale,
                        params: self.params.clone(),
                        stops: self.stops_handle,
                    }),
                ))
            }
        );

        self.src_color = Some(task_id);

        // Tile spacing is always handled by decomposing into separate draw calls so the
        // primitive opacity is equivalent to stops opacity. This might change to being
        // set to non-opaque in the presence of tile spacing if/when tile spacing is handled
        // in the same way as with the image primitive.
        self.opacity = self.stops_opacity;
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
    const PROFILE_COUNTER: usize = crate::profiler::INTERNED_RADIAL_GRADIENTS;
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

#[derive(Debug)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct RadialGradientTask {
    pub extend_mode: ExtendMode,
    pub center: DevicePoint,
    pub scale: DeviceVector2D,
    pub params: RadialGradientParams,
    pub stops: GpuCacheHandle,
}

impl RadialGradientTask {
    pub fn to_instance(&self, target_rect: &DeviceIntRect, gpu_cache: &mut GpuCache) -> RadialGradientInstance {
        RadialGradientInstance {
            task_rect: target_rect.to_f32(),
            center: self.center,
            scale: self.scale,
            start_radius: self.params.start_radius,
            end_radius: self.params.end_radius,
            ratio_xy: self.params.ratio_xy,
            extend_mode: self.extend_mode as i32,
            gradient_stops_address: self.stops.as_int(gpu_cache),
        }
    }
}

/// The per-instance shader input of a radial gradient render task.
///
/// Must match the RADIAL_GRADIENT instance description in renderer/vertex.rs.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[repr(C)]
#[derive(Clone, Debug)]
pub struct RadialGradientInstance {
    pub task_rect: DeviceRect,
    pub center: DevicePoint,
    pub scale: DeviceVector2D,
    pub start_radius: f32,
    pub end_radius: f32,
    pub ratio_xy: f32,
    pub extend_mode: i32,
    pub gradient_stops_address: i32,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct RadialGradientCacheKey {
    pub size: DeviceIntSize,
    pub center: PointKey,
    pub scale: PointKey,
    pub start_radius: FloatKey,
    pub end_radius: FloatKey,
    pub ratio_xy: FloatKey,
    pub extend_mode: ExtendMode,
    pub stops: Vec<GradientStopKey>,
}

/// Avoid invoking the radial gradient shader on large areas where the color is
/// constant.
///
/// If the extend mode is set to clamp, the "interesting" part
/// of the gradient is only in the bounds of the gradient's ellipse, and the rest
/// is the color of the last gradient stop.
///
/// Sometimes we run into radial gradient with a small radius compared to the
/// primitive bounds, which means a large area of the primitive is a constant color
/// This function tries to detect that, potentially shrink the gradient primitive to only
/// the useful part and if needed insert solid color primitives around the gradient where
/// parts of it have been removed.
pub fn optimize_radial_gradient(
    prim_rect: &mut LayoutRect,
    stretch_size: &mut LayoutSize,
    center: &mut LayoutPoint,
    tile_spacing: &mut LayoutSize,
    clip_rect: &LayoutRect,
    radius: LayoutSize,
    end_offset: f32,
    extend_mode: ExtendMode,
    stops: &[GradientStopKey],
    solid_parts: &mut dyn FnMut(&LayoutRect, ColorU),
) {
    let offset = apply_gradient_local_clip(
        prim_rect,
        stretch_size,
        tile_spacing,
        clip_rect
    );

    *center += offset;

    if extend_mode != ExtendMode::Clamp || stops.is_empty() {
        return;
    }

    // Bounding box of the "interesting" part of the gradient.
    let min = prim_rect.origin + center.to_vector() - radius.to_vector() * end_offset;
    let max = prim_rect.origin + center.to_vector() + radius.to_vector() * end_offset;

    // The (non-repeated) gradient primitive rect.
    let gradient_rect = LayoutRect {
        origin: prim_rect.origin,
        size: *stretch_size,
    };

    // How much internal margin between the primitive bounds and the gradient's
    // bounding rect (areas that are a constant color).
    let mut l = (min.x - gradient_rect.min_x()).max(0.0).floor();
    let mut t = (min.y - gradient_rect.min_y()).max(0.0).floor();
    let mut r = (gradient_rect.max_x() - max.x).max(0.0).floor();
    let mut b = (gradient_rect.max_y() - max.y).max(0.0).floor();

    let is_tiled = prim_rect.size.width > stretch_size.width + tile_spacing.width
        || prim_rect.size.height > stretch_size.height + tile_spacing.height;

    let bg_color = stops.last().unwrap().color;

    if bg_color.a != 0 && is_tiled {
        // If the primitive has repetitions, it's not enough to insert solid rects around it,
        // so bail out.
        return;
    }

    // If the background is fully transparent, shrinking the primitive bounds as much as possible
    // is always a win. If the background is not transparent, we have to insert solid rectangles
    // around the shrunk parts.
    // If the background is transparent and the primitive is tiled, the optimization may introduce
    // tile spacing which forces the tiling to be manually decomposed.
    // Either way, don't bother optimizing unless it saves a significant amount of pixels.
    if bg_color.a != 0 || (is_tiled && tile_spacing.is_empty()) {
        let threshold = 128.0;
        if l < threshold { l = 0.0 }
        if t < threshold { t = 0.0 }
        if r < threshold { r = 0.0 }
        if b < threshold { b = 0.0 }
    }

    if l + t + r + b == 0.0 {
        // No adjustment to make;
        return;
    }

    // Insert solid rectangles around the gradient, in the places where the primitive will be
    // shrunk.
    if bg_color.a != 0 {
        if l != 0.0 && t != 0.0 {
            let solid_rect = LayoutRect {
                origin: gradient_rect.origin,
                size: size2(l, t),
            };
            solid_parts(&solid_rect, bg_color);
        }

        if l != 0.0 && b != 0.0 {
            let solid_rect = LayoutRect {
                origin: gradient_rect.bottom_left() - vec2(0.0, b),
                size: size2(l, b),
            };
            solid_parts(&solid_rect, bg_color);
        }

        if t != 0.0 && r != 0.0 {
            let solid_rect = LayoutRect {
                origin: gradient_rect.top_right() - vec2(r, 0.0),
                size: size2(r, t),
            };
            solid_parts(&solid_rect, bg_color);
        }

        if r != 0.0 && b != 0.0 {
            let solid_rect = LayoutRect {
                origin: gradient_rect.bottom_right() - vec2(r, b),
                size: size2(r, b),
            };
            solid_parts(&solid_rect, bg_color);
        }

        if l != 0.0 {
            let solid_rect = LayoutRect {
                origin: gradient_rect.origin + vec2(0.0, t),
                size: size2(l, gradient_rect.size.height - t - b),
            };
            solid_parts(&solid_rect, bg_color);
        }

        if r != 0.0 {
            let solid_rect = LayoutRect {
                origin: gradient_rect.top_right() + vec2(-r, t),
                size: size2(r, gradient_rect.size.height - t - b),
            };
            solid_parts(&solid_rect, bg_color);
        }

        if t != 0.0 {
            let solid_rect = LayoutRect {
                origin: gradient_rect.origin + vec2(l, 0.0),
                size: size2(gradient_rect.size.width - l - r, t),
            };
            solid_parts(&solid_rect, bg_color);
        }

        if b != 0.0 {
            let solid_rect = LayoutRect {
                origin: gradient_rect.bottom_left() + vec2(l, -b),
                size: size2(gradient_rect.size.width - l - r, b),
            };
            solid_parts(&solid_rect, bg_color);
        }
    }

    // Shrink the gradient primitive.

    prim_rect.origin.x += l;
    prim_rect.origin.y += t;
    prim_rect.size.width -= l;
    prim_rect.size.height -= t;

    stretch_size.width -= l + r;
    stretch_size.height -= b + t;

    center.x -= l;
    center.y -= t;

    tile_spacing.width += l + r;
    tile_spacing.height += t + b;
}
