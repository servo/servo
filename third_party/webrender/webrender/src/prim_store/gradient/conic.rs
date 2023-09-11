/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Conic gradients
//!
//! Specification: https://drafts.csswg.org/css-images-4/#conic-gradients
//!
//! Conic gradients are rendered via cached render tasks and composited with the image brush.

use euclid::vec2;
use api::{ExtendMode, GradientStop, PremultipliedColorF};
use api::units::*;
use crate::scene_building::IsVisible;
use crate::frame_builder::FrameBuildingState;
use crate::gpu_cache::{GpuCache, GpuCacheHandle};
use crate::intern::{Internable, InternDebug, Handle as InternHandle};
use crate::internal_types::LayoutPrimitiveInfo;
use crate::prim_store::{BrushSegment, GradientTileRange};
use crate::prim_store::{PrimitiveInstanceKind, PrimitiveOpacity, FloatKey};
use crate::prim_store::{PrimKeyCommonData, PrimTemplateCommonData, PrimitiveStore};
use crate::prim_store::{NinePatchDescriptor, PointKey, SizeKey, InternablePrimitive};
use crate::render_task::{RenderTask, RenderTaskKind};
use crate::render_task_graph::RenderTaskId;
use crate::render_task_cache::{RenderTaskCacheKeyKind, RenderTaskCacheKey, RenderTaskParent};
use crate::picture::{SurfaceIndex};

use std::{hash, ops::{Deref, DerefMut}};
use super::{stops_and_min_alpha, GradientStopKey, GradientGpuBlockBuilder};

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
    pub center: DevicePoint,
    pub params: ConicGradientParams,
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
            scale.x = task_size.width / MAX_SIZE;
            task_size.width = MAX_SIZE;
        }
        if task_size.height > MAX_SIZE {
            scale.y = task_size.height / MAX_SIZE;
            task_size.height = MAX_SIZE;
        }

        ConicGradientTemplate {
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

impl ConicGradientTemplate {
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

        let cache_key = ConicGradientCacheKey {
            size: self.task_size,
            center: PointKey { x: self.center.x, y: self.center.y },
            scale: PointKey { x: self.scale.x, y: self.scale.y },
            start_offset: FloatKey(self.params.start_offset),
            end_offset: FloatKey(self.params.end_offset),
            angle: FloatKey(self.params.angle),
            extend_mode: self.extend_mode,
            stops: self.stops.iter().map(|stop| (*stop).into()).collect(),
        };

        let task_id = frame_state.resource_cache.request_render_task(
            RenderTaskCacheKey {
                size: self.task_size,
                kind: RenderTaskCacheKeyKind::ConicGradient(cache_key),
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
                    RenderTaskKind::ConicGradient(ConicGradientTask {
                        extend_mode: self.extend_mode,
                        scale: self.scale,
                        center: self.center,
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
    const PROFILE_COUNTER: usize = crate::profiler::INTERNED_CONIC_GRADIENTS;
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

#[derive(Debug)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct ConicGradientTask {
    pub extend_mode: ExtendMode,
    pub center: DevicePoint,
    pub scale: DeviceVector2D,
    pub params: ConicGradientParams,
    pub stops: GpuCacheHandle,
}

impl ConicGradientTask {
    pub fn to_instance(&self, target_rect: &DeviceIntRect, gpu_cache: &mut GpuCache) -> ConicGradientInstance {
        ConicGradientInstance {
            task_rect: target_rect.to_f32(),
            center: self.center,
            scale: self.scale,
            start_offset: self.params.start_offset,
            end_offset: self.params.end_offset,
            angle: self.params.angle,
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
pub struct ConicGradientInstance {
    pub task_rect: DeviceRect,
    pub center: DevicePoint,
    pub scale: DeviceVector2D,
    pub start_offset: f32,
    pub end_offset: f32,
    pub angle: f32,
    pub extend_mode: i32,
    pub gradient_stops_address: i32,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct ConicGradientCacheKey {
    pub size: DeviceIntSize,
    pub center: PointKey,
    pub scale: PointKey,
    pub start_offset: FloatKey,
    pub end_offset: FloatKey,
    pub angle: FloatKey,
    pub extend_mode: ExtendMode,
    pub stops: Vec<GradientStopKey>,
}

