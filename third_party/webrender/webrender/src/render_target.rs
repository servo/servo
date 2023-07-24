/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */


use api::units::*;
use api::{ColorF, PremultipliedColorF, ImageFormat, LineOrientation, BorderStyle, PipelineId};
use crate::batch::{AlphaBatchBuilder, AlphaBatchContainer, BatchTextures, resolve_image};
use crate::batch::{ClipBatcher, BatchBuilder};
use crate::spatial_tree::{SpatialTree, ROOT_SPATIAL_NODE_INDEX};
use crate::clip::ClipStore;
use crate::composite::CompositeState;
use crate::device::Texture;
use crate::frame_builder::{FrameGlobalResources};
use crate::gpu_cache::{GpuCache, GpuCacheAddress};
use crate::gpu_types::{BorderInstance, SvgFilterInstance, BlurDirection, BlurInstance, PrimitiveHeaders, ScalingInstance};
use crate::gpu_types::{TransformPalette, ZBufferIdGenerator};
use crate::internal_types::{FastHashMap, TextureSource, LayerIndex, Swizzle, SavedTargetIndex};
use crate::picture::{SurfaceInfo, ResolvedSurfaceTexture};
use crate::prim_store::{PrimitiveStore, DeferredResolve, PrimitiveScratchBuffer, PrimitiveVisibilityMask};
use crate::prim_store::gradient::GRADIENT_FP_STOPS;
use crate::render_backend::DataStores;
use crate::render_task::{RenderTaskKind, RenderTaskAddress, ClearMode, BlitSource};
use crate::render_task::{RenderTask, ScalingTask, SvgFilterInfo};
use crate::render_task_graph::{RenderTaskGraph, RenderTaskId};
use crate::resource_cache::ResourceCache;
use crate::texture_allocator::{ArrayAllocationTracker, FreeRectSlice};
use std::{cmp, mem};


const STYLE_SOLID: i32 = ((BorderStyle::Solid as i32) << 8) | ((BorderStyle::Solid as i32) << 16);
const STYLE_MASK: i32 = 0x00FF_FF00;

/// According to apitrace, textures larger than 2048 break fast clear
/// optimizations on some intel drivers. We sometimes need to go larger, but
/// we try to avoid it. This can go away when proper tiling support lands,
/// since we can then split large primitives across multiple textures.
const IDEAL_MAX_TEXTURE_DIMENSION: i32 = 2048;
/// If we ever need a larger texture than the ideal, we better round it up to a
/// reasonable number in order to have a bit of leeway in placing things inside.
const TEXTURE_DIMENSION_MASK: i32 = 0xFF;

/// A tag used to identify the output format of a `RenderTarget`.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub enum RenderTargetKind {
    Color, // RGBA8
    Alpha, // R8
}

/// Identifies a given `RenderTarget` in a `RenderTargetList`.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct RenderTargetIndex(pub usize);

pub struct RenderTargetContext<'a, 'rc> {
    pub global_device_pixel_scale: DevicePixelScale,
    pub prim_store: &'a PrimitiveStore,
    pub resource_cache: &'rc mut ResourceCache,
    pub use_dual_source_blending: bool,
    pub use_advanced_blending: bool,
    pub break_advanced_blend_batches: bool,
    pub batch_lookback_count: usize,
    pub spatial_tree: &'a SpatialTree,
    pub data_stores: &'a DataStores,
    pub surfaces: &'a [SurfaceInfo],
    pub scratch: &'a PrimitiveScratchBuffer,
    pub screen_world_rect: WorldRect,
    pub globals: &'a FrameGlobalResources,
}

/// Represents a number of rendering operations on a surface.
///
/// In graphics parlance, a "render target" usually means "a surface (texture or
/// framebuffer) bound to the output of a shader". This trait has a slightly
/// different meaning, in that it represents the operations on that surface
/// _before_ it's actually bound and rendered. So a `RenderTarget` is built by
/// the `RenderBackend` by inserting tasks, and then shipped over to the
/// `Renderer` where a device surface is resolved and the tasks are transformed
/// into draw commands on that surface.
///
/// We express this as a trait to generalize over color and alpha surfaces.
/// a given `RenderTask` will draw to one or the other, depending on its type
/// and sometimes on its parameters. See `RenderTask::target_kind`.
pub trait RenderTarget {
    /// Creates a new RenderTarget of the given type.
    fn new(
        screen_size: DeviceIntSize,
        gpu_supports_fast_clears: bool,
    ) -> Self;

    /// Optional hook to provide additional processing for the target at the
    /// end of the build phase.
    fn build(
        &mut self,
        _ctx: &mut RenderTargetContext,
        _gpu_cache: &mut GpuCache,
        _render_tasks: &mut RenderTaskGraph,
        _deferred_resolves: &mut Vec<DeferredResolve>,
        _prim_headers: &mut PrimitiveHeaders,
        _transforms: &mut TransformPalette,
        _z_generator: &mut ZBufferIdGenerator,
        _composite_state: &mut CompositeState,
    ) {
    }

    /// Associates a `RenderTask` with this target. That task must be assigned
    /// to a region returned by invoking `allocate()` on this target.
    ///
    /// TODO(gw): It's a bit odd that we need the deferred resolves and mutable
    /// GPU cache here. They are typically used by the build step above. They
    /// are used for the blit jobs to allow resolve_image to be called. It's a
    /// bit of extra overhead to store the image key here and the resolve them
    /// in the build step separately.  BUT: if/when we add more texture cache
    /// target jobs, we might want to tidy this up.
    fn add_task(
        &mut self,
        task_id: RenderTaskId,
        ctx: &RenderTargetContext,
        gpu_cache: &mut GpuCache,
        render_tasks: &RenderTaskGraph,
        clip_store: &ClipStore,
        transforms: &mut TransformPalette,
        deferred_resolves: &mut Vec<DeferredResolve>,
    );

    fn needs_depth(&self) -> bool;

    fn used_rect(&self) -> DeviceIntRect;
    fn add_used(&mut self, rect: DeviceIntRect);
}

/// A series of `RenderTarget` instances, serving as the high-level container
/// into which `RenderTasks` are assigned.
///
/// During the build phase, we iterate over the tasks in each `RenderPass`. For
/// each task, we invoke `allocate()` on the `RenderTargetList`, which in turn
/// attempts to allocate an output region in the last `RenderTarget` in the
/// list. If allocation fails (or if the list is empty), a new `RenderTarget` is
/// created and appended to the list. The build phase then assign the task into
/// the target associated with the final allocation.
///
/// The result is that each `RenderPass` is associated with one or two
/// `RenderTargetLists`, depending on whether we have all our tasks have the
/// same `RenderTargetKind`. The lists are then shipped to the `Renderer`, which
/// allocates a device texture array, with one slice per render target in the
/// list.
///
/// The upshot of this scheme is that it maximizes batching. In a given pass,
/// we need to do a separate batch for each individual render target. But with
/// the texture array, we can expose the entirety of the previous pass to each
/// task in the current pass in a single batch, which generally allows each
/// task to be drawn in a single batch regardless of how many results from the
/// previous pass it depends on.
///
/// Note that in some cases (like drop-shadows), we can depend on the output of
/// a pass earlier than the immediately-preceding pass. See `SavedTargetIndex`.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct RenderTargetList<T> {
    screen_size: DeviceIntSize,
    pub format: ImageFormat,
    /// The maximum width and height of any single primitive we've encountered
    /// that will be drawn to a dynamic location.
    ///
    /// We initially create our per-slice allocators with a width and height of
    /// IDEAL_MAX_TEXTURE_DIMENSION. If we encounter a larger primitive, the
    /// allocation will fail, but we'll bump max_dynamic_size, which will cause the
    /// allocator for the next slice to be just large enough to accomodate it.
    pub max_dynamic_size: DeviceIntSize,
    pub targets: Vec<T>,
    pub saved_index: Option<SavedTargetIndex>,
    pub alloc_tracker: ArrayAllocationTracker,
    gpu_supports_fast_clears: bool,
}

impl<T: RenderTarget> RenderTargetList<T> {
    pub fn new(
        screen_size: DeviceIntSize,
        format: ImageFormat,
        gpu_supports_fast_clears: bool,
    ) -> Self {
        RenderTargetList {
            screen_size,
            format,
            max_dynamic_size: DeviceIntSize::new(0, 0),
            targets: Vec::new(),
            saved_index: None,
            alloc_tracker: ArrayAllocationTracker::new(),
            gpu_supports_fast_clears,
        }
    }

    pub fn build(
        &mut self,
        ctx: &mut RenderTargetContext,
        gpu_cache: &mut GpuCache,
        render_tasks: &mut RenderTaskGraph,
        deferred_resolves: &mut Vec<DeferredResolve>,
        saved_index: Option<SavedTargetIndex>,
        prim_headers: &mut PrimitiveHeaders,
        transforms: &mut TransformPalette,
        z_generator: &mut ZBufferIdGenerator,
        composite_state: &mut CompositeState,
    ) {
        debug_assert_eq!(None, self.saved_index);
        self.saved_index = saved_index;

        for target in &mut self.targets {
            target.build(
                ctx,
                gpu_cache,
                render_tasks,
                deferred_resolves,
                prim_headers,
                transforms,
                z_generator,
                composite_state,
            );
        }
    }

    pub fn allocate(
        &mut self,
        alloc_size: DeviceIntSize,
    ) -> (RenderTargetIndex, DeviceIntPoint) {
        let (free_rect_slice, origin) = match self.alloc_tracker.allocate(&alloc_size) {
            Some(allocation) => allocation,
            None => {
                // Have the allocator restrict slice sizes to our max ideal
                // dimensions, unless we've already gone bigger on a previous
                // slice.
                let rounded_dimensions = DeviceIntSize::new(
                    (self.max_dynamic_size.width + TEXTURE_DIMENSION_MASK) & !TEXTURE_DIMENSION_MASK,
                    (self.max_dynamic_size.height + TEXTURE_DIMENSION_MASK) & !TEXTURE_DIMENSION_MASK,
                );
                let allocator_dimensions = DeviceIntSize::new(
                    cmp::max(IDEAL_MAX_TEXTURE_DIMENSION, rounded_dimensions.width),
                    cmp::max(IDEAL_MAX_TEXTURE_DIMENSION, rounded_dimensions.height),
                );

                assert!(alloc_size.width <= allocator_dimensions.width &&
                    alloc_size.height <= allocator_dimensions.height);
                let slice = FreeRectSlice(self.targets.len() as u32);
                self.targets.push(T::new(self.screen_size, self.gpu_supports_fast_clears));

                self.alloc_tracker.extend(
                    slice,
                    allocator_dimensions,
                    alloc_size,
                );

                (slice, DeviceIntPoint::zero())
            }
        };

        if alloc_size.is_empty() && self.targets.is_empty() {
            // push an unused target here, only if we don't have any
            self.targets.push(T::new(self.screen_size, self.gpu_supports_fast_clears));
        }

        self.targets[free_rect_slice.0 as usize]
            .add_used(DeviceIntRect::new(origin, alloc_size));

        (RenderTargetIndex(free_rect_slice.0 as usize), origin)
    }

    pub fn needs_depth(&self) -> bool {
        self.targets.iter().any(|target| target.needs_depth())
    }

    pub fn check_ready(&self, t: &Texture) {
        let dimensions = t.get_dimensions();
        assert!(dimensions.width >= self.max_dynamic_size.width);
        assert!(dimensions.height >= self.max_dynamic_size.height);
        assert_eq!(t.get_format(), self.format);
        assert_eq!(t.get_layer_count() as usize, self.targets.len());
        assert!(t.supports_depth() >= self.needs_depth());
    }
}


/// Contains the work (in the form of instance arrays) needed to fill a color
/// color output surface (RGBA8).
///
/// See `RenderTarget`.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct ColorRenderTarget {
    pub alpha_batch_containers: Vec<AlphaBatchContainer>,
    // List of blur operations to apply for this render target.
    pub vertical_blurs: Vec<BlurInstance>,
    pub horizontal_blurs: Vec<BlurInstance>,
    pub readbacks: Vec<DeviceIntRect>,
    pub scalings: FastHashMap<TextureSource, Vec<ScalingInstance>>,
    pub svg_filters: Vec<(BatchTextures, Vec<SvgFilterInstance>)>,
    pub blits: Vec<BlitJob>,
    // List of frame buffer outputs for this render target.
    pub outputs: Vec<FrameOutput>,
    alpha_tasks: Vec<RenderTaskId>,
    screen_size: DeviceIntSize,
    // Track the used rect of the render target, so that
    // we can set a scissor rect and only clear to the
    // used portion of the target as an optimization.
    pub used_rect: DeviceIntRect,
}

impl RenderTarget for ColorRenderTarget {
    fn new(
        screen_size: DeviceIntSize,
        _: bool,
    ) -> Self {
        ColorRenderTarget {
            alpha_batch_containers: Vec::new(),
            vertical_blurs: Vec::new(),
            horizontal_blurs: Vec::new(),
            readbacks: Vec::new(),
            scalings: FastHashMap::default(),
            svg_filters: Vec::new(),
            blits: Vec::new(),
            outputs: Vec::new(),
            alpha_tasks: Vec::new(),
            screen_size,
            used_rect: DeviceIntRect::zero(),
        }
    }

    fn build(
        &mut self,
        ctx: &mut RenderTargetContext,
        gpu_cache: &mut GpuCache,
        render_tasks: &mut RenderTaskGraph,
        deferred_resolves: &mut Vec<DeferredResolve>,
        prim_headers: &mut PrimitiveHeaders,
        transforms: &mut TransformPalette,
        z_generator: &mut ZBufferIdGenerator,
        composite_state: &mut CompositeState,
    ) {
        profile_scope!("build");
        let mut merged_batches = AlphaBatchContainer::new(None);

        for task_id in &self.alpha_tasks {
            profile_scope!("alpha_task");
            let task = &render_tasks[*task_id];

            match task.clear_mode {
                ClearMode::One |
                ClearMode::Zero => {
                    panic!("bug: invalid clear mode for color task");
                }
                ClearMode::DontCare |
                ClearMode::Transparent => {}
            }

            match task.kind {
                RenderTaskKind::Picture(ref pic_task) => {
                    let pic = &ctx.prim_store.pictures[pic_task.pic_index.0];

                    let raster_spatial_node_index = match pic.raster_config {
                        Some(ref raster_config) => {
                            let surface = &ctx.surfaces[raster_config.surface_index.0];
                            surface.raster_spatial_node_index
                        }
                        None => {
                            // This must be the main framebuffer
                            ROOT_SPATIAL_NODE_INDEX
                        }
                    };

                    let (target_rect, _) = task.get_target_rect();

                    let scissor_rect = if pic_task.can_merge {
                        None
                    } else {
                        Some(target_rect)
                    };

                    // TODO(gw): The type names of AlphaBatchBuilder and BatchBuilder
                    //           are still confusing. Once more of the picture caching
                    //           improvement code lands, the AlphaBatchBuilder and
                    //           AlphaBatchList types will be collapsed into one, which
                    //           should simplify coming up with better type names.
                    let alpha_batch_builder = AlphaBatchBuilder::new(
                        self.screen_size,
                        ctx.break_advanced_blend_batches,
                        ctx.batch_lookback_count,
                        *task_id,
                        render_tasks.get_task_address(*task_id),
                        PrimitiveVisibilityMask::all(),
                    );

                    let mut batch_builder = BatchBuilder::new(
                        vec![alpha_batch_builder],
                    );

                    batch_builder.add_pic_to_batch(
                        pic,
                        ctx,
                        gpu_cache,
                        render_tasks,
                        deferred_resolves,
                        prim_headers,
                        transforms,
                        raster_spatial_node_index,
                        pic_task.surface_spatial_node_index,
                        z_generator,
                        composite_state,
                    );

                    let alpha_batch_builders = batch_builder.finalize();

                    for batcher in alpha_batch_builders {
                        batcher.build(
                            &mut self.alpha_batch_containers,
                            &mut merged_batches,
                            target_rect,
                            scissor_rect,
                        );
                    }
                }
                _ => {
                    unreachable!();
                }
            }
        }

        if !merged_batches.is_empty() {
            self.alpha_batch_containers.push(merged_batches);
        }
    }

    fn add_task(
        &mut self,
        task_id: RenderTaskId,
        ctx: &RenderTargetContext,
        gpu_cache: &mut GpuCache,
        render_tasks: &RenderTaskGraph,
        _: &ClipStore,
        _: &mut TransformPalette,
        deferred_resolves: &mut Vec<DeferredResolve>,
    ) {
        profile_scope!("add_task");
        let task = &render_tasks[task_id];

        match task.kind {
            RenderTaskKind::VerticalBlur(..) => {
                add_blur_instances(
                    &mut self.vertical_blurs,
                    BlurDirection::Vertical,
                    render_tasks.get_task_address(task_id),
                    render_tasks.get_task_address(task.children[0]),
                );
            }
            RenderTaskKind::HorizontalBlur(..) => {
                add_blur_instances(
                    &mut self.horizontal_blurs,
                    BlurDirection::Horizontal,
                    render_tasks.get_task_address(task_id),
                    render_tasks.get_task_address(task.children[0]),
                );
            }
            RenderTaskKind::Picture(ref task_info) => {
                let pic = &ctx.prim_store.pictures[task_info.pic_index.0];
                self.alpha_tasks.push(task_id);

                // If this pipeline is registered as a frame output
                // store the information necessary to do the copy.
                if let Some(pipeline_id) = pic.frame_output_pipeline_id {
                    self.outputs.push(FrameOutput {
                        pipeline_id,
                        task_id,
                    });
                }
            }
            RenderTaskKind::SvgFilter(ref task_info) => {
                add_svg_filter_instances(
                    &mut self.svg_filters,
                    render_tasks,
                    &task_info.info,
                    task_id,
                    task.children.get(0).cloned(),
                    task.children.get(1).cloned(),
                    task_info.extra_gpu_cache_handle.map(|handle| gpu_cache.get_address(&handle)),
                )
            }
            RenderTaskKind::ClipRegion(..) |
            RenderTaskKind::Border(..) |
            RenderTaskKind::CacheMask(..) |
            RenderTaskKind::Gradient(..) |
            RenderTaskKind::LineDecoration(..) => {
                panic!("Should not be added to color target!");
            }
            RenderTaskKind::Readback(device_rect) => {
                self.readbacks.push(device_rect);
            }
            RenderTaskKind::Scaling(ref info) => {
                add_scaling_instances(
                    info,
                    &mut self.scalings,
                    task,
                    task.children.first().map(|&child| &render_tasks[child]),
                    ctx.resource_cache,
                    gpu_cache,
                    deferred_resolves,
                );
            }
            RenderTaskKind::Blit(ref task_info) => {
                let source = match task_info.source {
                    BlitSource::Image { key } => {
                        // Get the cache item for the source texture.
                        let cache_item = resolve_image(
                            key.request,
                            ctx.resource_cache,
                            gpu_cache,
                            deferred_resolves,
                        );

                        // Work out a source rect to copy from the texture, depending on whether
                        // a sub-rect is present or not.
                        let source_rect = key.texel_rect.map_or(cache_item.uv_rect.to_i32(), |sub_rect| {
                            DeviceIntRect::new(
                                DeviceIntPoint::new(
                                    cache_item.uv_rect.origin.x as i32 + sub_rect.origin.x,
                                    cache_item.uv_rect.origin.y as i32 + sub_rect.origin.y,
                                ),
                                sub_rect.size,
                            )
                        });

                        // Store the blit job for the renderer to execute, including
                        // the allocated destination rect within this target.
                        BlitJobSource::Texture(
                            cache_item.texture_id,
                            cache_item.texture_layer,
                            source_rect,
                        )
                    }
                    BlitSource::RenderTask { task_id } => {
                        BlitJobSource::RenderTask(task_id)
                    }
                };

                let target_rect = task
                    .get_target_rect()
                    .0
                    .inner_rect(task_info.padding);
                self.blits.push(BlitJob {
                    source,
                    target_rect,
                });
            }
            #[cfg(test)]
            RenderTaskKind::Test(..) => {}
        }
    }

    fn needs_depth(&self) -> bool {
        self.alpha_batch_containers.iter().any(|ab| {
            !ab.opaque_batches.is_empty()
        })
    }

    fn used_rect(&self) -> DeviceIntRect {
        self.used_rect
    }

    fn add_used(&mut self, rect: DeviceIntRect) {
        self.used_rect = self.used_rect.union(&rect);
    }
}

/// Contains the work (in the form of instance arrays) needed to fill an alpha
/// output surface (R8).
///
/// See `RenderTarget`.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct AlphaRenderTarget {
    pub clip_batcher: ClipBatcher,
    // List of blur operations to apply for this render target.
    pub vertical_blurs: Vec<BlurInstance>,
    pub horizontal_blurs: Vec<BlurInstance>,
    pub scalings: FastHashMap<TextureSource, Vec<ScalingInstance>>,
    pub zero_clears: Vec<RenderTaskId>,
    pub one_clears: Vec<RenderTaskId>,
    // Track the used rect of the render target, so that
    // we can set a scissor rect and only clear to the
    // used portion of the target as an optimization.
    pub used_rect: DeviceIntRect,
}

impl RenderTarget for AlphaRenderTarget {
    fn new(
        _: DeviceIntSize,
        gpu_supports_fast_clears: bool,
    ) -> Self {
        AlphaRenderTarget {
            clip_batcher: ClipBatcher::new(gpu_supports_fast_clears),
            vertical_blurs: Vec::new(),
            horizontal_blurs: Vec::new(),
            scalings: FastHashMap::default(),
            zero_clears: Vec::new(),
            one_clears: Vec::new(),
            used_rect: DeviceIntRect::zero(),
        }
    }

    fn add_task(
        &mut self,
        task_id: RenderTaskId,
        ctx: &RenderTargetContext,
        gpu_cache: &mut GpuCache,
        render_tasks: &RenderTaskGraph,
        clip_store: &ClipStore,
        transforms: &mut TransformPalette,
        deferred_resolves: &mut Vec<DeferredResolve>,
    ) {
        profile_scope!("add_task");
        let task = &render_tasks[task_id];
        let (target_rect, _) = task.get_target_rect();

        match task.clear_mode {
            ClearMode::Zero => {
                self.zero_clears.push(task_id);
            }
            ClearMode::One => {
                self.one_clears.push(task_id);
            }
            ClearMode::DontCare => {}
            ClearMode::Transparent => {
                panic!("bug: invalid clear mode for alpha task");
            }
        }

        match task.kind {
            RenderTaskKind::Readback(..) |
            RenderTaskKind::Picture(..) |
            RenderTaskKind::Blit(..) |
            RenderTaskKind::Border(..) |
            RenderTaskKind::LineDecoration(..) |
            RenderTaskKind::Gradient(..) |
            RenderTaskKind::SvgFilter(..) => {
                panic!("BUG: should not be added to alpha target!");
            }
            RenderTaskKind::VerticalBlur(..) => {
                add_blur_instances(
                    &mut self.vertical_blurs,
                    BlurDirection::Vertical,
                    render_tasks.get_task_address(task_id),
                    render_tasks.get_task_address(task.children[0]),
                );
            }
            RenderTaskKind::HorizontalBlur(..) => {
                add_blur_instances(
                    &mut self.horizontal_blurs,
                    BlurDirection::Horizontal,
                    render_tasks.get_task_address(task_id),
                    render_tasks.get_task_address(task.children[0]),
                );
            }
            RenderTaskKind::CacheMask(ref task_info) => {
                self.clip_batcher.add(
                    task_info.clip_node_range,
                    task_info.root_spatial_node_index,
                    ctx.resource_cache,
                    gpu_cache,
                    clip_store,
                    ctx.spatial_tree,
                    transforms,
                    &ctx.data_stores.clip,
                    task_info.actual_rect,
                    &ctx.screen_world_rect,
                    task_info.device_pixel_scale,
                    target_rect.origin.to_f32(),
                    task_info.actual_rect.origin.to_f32(),
                );
            }
            RenderTaskKind::ClipRegion(ref region_task) => {
                let device_rect = DeviceRect::new(
                    DevicePoint::zero(),
                    target_rect.size.to_f32(),
                );
                self.clip_batcher.add_clip_region(
                    region_task.clip_data_address,
                    region_task.local_pos,
                    device_rect,
                    target_rect.origin.to_f32(),
                    DevicePoint::zero(),
                    region_task.device_pixel_scale.0,
                );
            }
            RenderTaskKind::Scaling(ref info) => {
                add_scaling_instances(
                    info,
                    &mut self.scalings,
                    task,
                    task.children.first().map(|&child| &render_tasks[child]),
                    ctx.resource_cache,
                    gpu_cache,
                    deferred_resolves,
                );
            }
            #[cfg(test)]
            RenderTaskKind::Test(..) => {}
        }
    }

    fn needs_depth(&self) -> bool {
        false
    }

    fn used_rect(&self) -> DeviceIntRect {
        self.used_rect
    }

    fn add_used(&mut self, rect: DeviceIntRect) {
        self.used_rect = self.used_rect.union(&rect);
    }
}

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct PictureCacheTarget {
    pub surface: ResolvedSurfaceTexture,
    pub alpha_batch_container: AlphaBatchContainer,
    pub clear_color: Option<ColorF>,
    pub dirty_rect: DeviceIntRect,
    pub valid_rect: DeviceIntRect,
}

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct TextureCacheRenderTarget {
    pub target_kind: RenderTargetKind,
    pub horizontal_blurs: Vec<BlurInstance>,
    pub blits: Vec<BlitJob>,
    pub border_segments_complex: Vec<BorderInstance>,
    pub border_segments_solid: Vec<BorderInstance>,
    pub clears: Vec<DeviceIntRect>,
    pub line_decorations: Vec<LineDecorationJob>,
    pub gradients: Vec<GradientJob>,
}

impl TextureCacheRenderTarget {
    pub fn new(target_kind: RenderTargetKind) -> Self {
        TextureCacheRenderTarget {
            target_kind,
            horizontal_blurs: vec![],
            blits: vec![],
            border_segments_complex: vec![],
            border_segments_solid: vec![],
            clears: vec![],
            line_decorations: vec![],
            gradients: vec![],
        }
    }

    pub fn add_task(
        &mut self,
        task_id: RenderTaskId,
        render_tasks: &mut RenderTaskGraph,
    ) {
        profile_scope!("add_task");
        let task_address = render_tasks.get_task_address(task_id);
        let src_task_address = render_tasks[task_id].children.get(0).map(|src_task_id| {
            render_tasks.get_task_address(*src_task_id)
        });

        let task = &mut render_tasks[task_id];
        let target_rect = task.get_target_rect();

        match task.kind {
            RenderTaskKind::LineDecoration(ref info) => {
                self.clears.push(target_rect.0);

                self.line_decorations.push(LineDecorationJob {
                    task_rect: target_rect.0.to_f32(),
                    local_size: info.local_size,
                    style: info.style as i32,
                    axis_select: match info.orientation {
                        LineOrientation::Horizontal => 0.0,
                        LineOrientation::Vertical => 1.0,
                    },
                    wavy_line_thickness: info.wavy_line_thickness,
                });
            }
            RenderTaskKind::HorizontalBlur(..) => {
                add_blur_instances(
                    &mut self.horizontal_blurs,
                    BlurDirection::Horizontal,
                    task_address,
                    src_task_address.unwrap(),
                );
            }
            RenderTaskKind::Blit(ref task_info) => {
                match task_info.source {
                    BlitSource::Image { .. } => {
                        // reading/writing from the texture cache at the same time
                        // is undefined behavior.
                        panic!("bug: a single blit cannot be to/from texture cache");
                    }
                    BlitSource::RenderTask { task_id } => {
                        // Add a blit job to copy from an existing render
                        // task to this target.
                        self.blits.push(BlitJob {
                            source: BlitJobSource::RenderTask(task_id),
                            target_rect: target_rect.0.inner_rect(task_info.padding),
                        });
                    }
                }
            }
            RenderTaskKind::Border(ref mut task_info) => {
                self.clears.push(target_rect.0);

                let task_origin = target_rect.0.origin.to_f32();
                let instances = mem::replace(&mut task_info.instances, Vec::new());
                for mut instance in instances {
                    // TODO(gw): It may be better to store the task origin in
                    //           the render task data instead of per instance.
                    instance.task_origin = task_origin;
                    if instance.flags & STYLE_MASK == STYLE_SOLID {
                        self.border_segments_solid.push(instance);
                    } else {
                        self.border_segments_complex.push(instance);
                    }
                }
            }
            RenderTaskKind::Gradient(ref task_info) => {
                let mut stops = [0.0; 4];
                let mut colors = [PremultipliedColorF::BLACK; 4];

                let axis_select = match task_info.orientation {
                    LineOrientation::Horizontal => 0.0,
                    LineOrientation::Vertical => 1.0,
                };

                for (stop, (offset, color)) in task_info.stops.iter().zip(stops.iter_mut().zip(colors.iter_mut())) {
                    *offset = stop.offset;
                    *color = ColorF::from(stop.color).premultiplied();
                }

                self.gradients.push(GradientJob {
                    task_rect: target_rect.0.to_f32(),
                    axis_select,
                    stops,
                    colors,
                    start_stop: [task_info.start_point, task_info.end_point],
                });
            }
            RenderTaskKind::VerticalBlur(..) |
            RenderTaskKind::Picture(..) |
            RenderTaskKind::ClipRegion(..) |
            RenderTaskKind::CacheMask(..) |
            RenderTaskKind::Readback(..) |
            RenderTaskKind::Scaling(..) |
            RenderTaskKind::SvgFilter(..) => {
                panic!("BUG: unexpected task kind for texture cache target");
            }
            #[cfg(test)]
            RenderTaskKind::Test(..) => {}
        }
    }
}

fn add_blur_instances(
    instances: &mut Vec<BlurInstance>,
    blur_direction: BlurDirection,
    task_address: RenderTaskAddress,
    src_task_address: RenderTaskAddress,
) {
    let instance = BlurInstance {
        task_address,
        src_task_address,
        blur_direction,
    };

    instances.push(instance);
}

fn add_scaling_instances(
    task: &ScalingTask,
    instances: &mut FastHashMap<TextureSource, Vec<ScalingInstance>>,
    target_task: &RenderTask,
    source_task: Option<&RenderTask>,
    resource_cache: &ResourceCache,
    gpu_cache: &mut GpuCache,
    deferred_resolves: &mut Vec<DeferredResolve>,
) {
    let target_rect = target_task
        .get_target_rect()
        .0
        .inner_rect(task.padding)
        .to_f32();

    let (source, (source_rect, source_layer)) = match task.image {
        Some(key) => {
            assert!(source_task.is_none());

            // Get the cache item for the source texture.
            let cache_item = resolve_image(
                key.request,
                resource_cache,
                gpu_cache,
                deferred_resolves,
            );

            // Work out a source rect to copy from the texture, depending on whether
            // a sub-rect is present or not.
            let source_rect = key.texel_rect.map_or(cache_item.uv_rect, |sub_rect| {
                DeviceIntRect::new(
                    DeviceIntPoint::new(
                        cache_item.uv_rect.origin.x + sub_rect.origin.x,
                        cache_item.uv_rect.origin.y + sub_rect.origin.y,
                    ),
                    sub_rect.size,
                )
            });

            (
                cache_item.texture_id,
                (source_rect, cache_item.texture_layer as LayerIndex),
            )
        }
        None => {
            (
                match task.target_kind {
                    RenderTargetKind::Color => TextureSource::PrevPassColor,
                    RenderTargetKind::Alpha => TextureSource::PrevPassAlpha,
                },
                source_task.unwrap().location.to_source_rect(),
            )
        }
    };

    instances
        .entry(source)
        .or_insert(Vec::new())
        .push(ScalingInstance {
            target_rect,
            source_rect,
            source_layer: source_layer as i32,
        });
}

fn add_svg_filter_instances(
    instances: &mut Vec<(BatchTextures, Vec<SvgFilterInstance>)>,
    render_tasks: &RenderTaskGraph,
    filter: &SvgFilterInfo,
    task_id: RenderTaskId,
    input_1_task: Option<RenderTaskId>,
    input_2_task: Option<RenderTaskId>,
    extra_data_address: Option<GpuCacheAddress>,
) {
    let mut textures = BatchTextures::no_texture();

    if let Some(saved_index) = input_1_task.map(|id| &render_tasks[id].saved_index) {
        textures.colors[0] = match saved_index {
            Some(saved_index) => TextureSource::RenderTaskCache(*saved_index, Swizzle::default()),
            None => TextureSource::PrevPassColor,
        };
    }

    if let Some(saved_index) = input_2_task.map(|id| &render_tasks[id].saved_index) {
        textures.colors[1] = match saved_index {
            Some(saved_index) => TextureSource::RenderTaskCache(*saved_index, Swizzle::default()),
            None => TextureSource::PrevPassColor,
        };
    }

    let kind = match filter {
        SvgFilterInfo::Blend(..) => 0,
        SvgFilterInfo::Flood(..) => 1,
        SvgFilterInfo::LinearToSrgb => 2,
        SvgFilterInfo::SrgbToLinear => 3,
        SvgFilterInfo::Opacity(..) => 4,
        SvgFilterInfo::ColorMatrix(..) => 5,
        SvgFilterInfo::DropShadow(..) => 6,
        SvgFilterInfo::Offset(..) => 7,
        SvgFilterInfo::ComponentTransfer(..) => 8,
        SvgFilterInfo::Identity => 9,
        SvgFilterInfo::Composite(..) => 10,
    };

    let input_count = match filter {
        SvgFilterInfo::Flood(..) => 0,

        SvgFilterInfo::LinearToSrgb |
        SvgFilterInfo::SrgbToLinear |
        SvgFilterInfo::Opacity(..) |
        SvgFilterInfo::ColorMatrix(..) |
        SvgFilterInfo::Offset(..) |
        SvgFilterInfo::ComponentTransfer(..) |
        SvgFilterInfo::Identity => 1,

        // Not techincally a 2 input filter, but we have 2 inputs here: original content & blurred content.
        SvgFilterInfo::DropShadow(..) |
        SvgFilterInfo::Blend(..) |
        SvgFilterInfo::Composite(..) => 2,
    };

    let generic_int = match filter {
        SvgFilterInfo::Blend(mode) => *mode as u16,
        SvgFilterInfo::ComponentTransfer(data) =>
            (data.r_func.to_int() << 12 |
              data.g_func.to_int() << 8 |
              data.b_func.to_int() << 4 |
              data.a_func.to_int()) as u16,
        SvgFilterInfo::Composite(operator) =>
            operator.as_int() as u16,
        SvgFilterInfo::LinearToSrgb |
        SvgFilterInfo::SrgbToLinear |
        SvgFilterInfo::Flood(..) |
        SvgFilterInfo::Opacity(..) |
        SvgFilterInfo::ColorMatrix(..) |
        SvgFilterInfo::DropShadow(..) |
        SvgFilterInfo::Offset(..) |
        SvgFilterInfo::Identity => 0,
    };

    let instance = SvgFilterInstance {
        task_address: render_tasks.get_task_address(task_id),
        input_1_task_address: input_1_task.map(|id| render_tasks.get_task_address(id)).unwrap_or(RenderTaskAddress(0)),
        input_2_task_address: input_2_task.map(|id| render_tasks.get_task_address(id)).unwrap_or(RenderTaskAddress(0)),
        kind,
        input_count,
        generic_int,
        extra_data_address: extra_data_address.unwrap_or(GpuCacheAddress::INVALID),
    };

    for (ref mut batch_textures, ref mut batch) in instances.iter_mut() {
        if let Some(combined_textures) = batch_textures.combine_textures(textures) {
            batch.push(instance);
            // Update the batch textures to the newly combined batch textures
            *batch_textures = combined_textures;
            return;
        }
    }

    instances.push((textures, vec![instance]));
}

// Defines where the source data for a blit job can be found.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub enum BlitJobSource {
    Texture(TextureSource, i32, DeviceIntRect),
    RenderTask(RenderTaskId),
}

// Information required to do a blit from a source to a target.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct BlitJob {
    pub source: BlitJobSource,
    pub target_rect: DeviceIntRect,
}

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(Debug)]
pub struct LineDecorationJob {
    pub task_rect: DeviceRect,
    pub local_size: LayoutSize,
    pub wavy_line_thickness: f32,
    pub style: i32,
    pub axis_select: f32,
}

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[repr(C)]
pub struct GradientJob {
    pub task_rect: DeviceRect,
    pub stops: [f32; GRADIENT_FP_STOPS],
    pub colors: [PremultipliedColorF; GRADIENT_FP_STOPS],
    pub axis_select: f32,
    pub start_stop: [f32; 2],
}

/// Frame output information for a given pipeline ID.
/// Storing the task ID allows the renderer to find
/// the target rect within the render target that this
/// pipeline exists at.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct FrameOutput {
    pub task_id: RenderTaskId,
    pub pipeline_id: PipelineId,
}
