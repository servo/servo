/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use api::{ColorF, DebugFlags, DocumentLayer, FontRenderMode, PremultipliedColorF};
use api::units::*;
use crate::batch::{BatchBuilder, AlphaBatchBuilder, AlphaBatchContainer};
use crate::clip::{ClipStore, ClipChainStack, ClipInstance};
use crate::spatial_tree::{SpatialTree, ROOT_SPATIAL_NODE_INDEX, SpatialNodeIndex};
use crate::composite::{CompositorKind, CompositeState};
use crate::debug_render::DebugItem;
use crate::gpu_cache::{GpuCache, GpuCacheHandle};
use crate::gpu_types::{PrimitiveHeaders, TransformPalette, UvRectKind, ZBufferIdGenerator};
use crate::gpu_types::TransformData;
use crate::internal_types::{FastHashMap, PlaneSplitter, SavedTargetIndex};
use crate::picture::{PictureUpdateState, SurfaceInfo, ROOT_SURFACE_INDEX, SurfaceIndex, RecordedDirtyRegion};
use crate::picture::{RetainedTiles, TileCacheInstance, DirtyRegion, SurfaceRenderTasks, SubpixelMode};
use crate::picture::{BackdropKind, TileCacheLogger};
use crate::prim_store::{SpaceMapper, PictureIndex, PrimitiveDebugId, PrimitiveScratchBuffer};
use crate::prim_store::{DeferredResolve, PrimitiveVisibilityMask};
use crate::profiler::{FrameProfileCounters, TextureCacheProfileCounters, ResourceProfileCounters};
use crate::render_backend::{DataStores, FrameStamp, FrameId};
use crate::render_target::{RenderTarget, PictureCacheTarget, TextureCacheRenderTarget};
use crate::render_target::{RenderTargetContext, RenderTargetKind};
use crate::render_task_graph::{RenderTaskId, RenderTaskGraph, RenderTaskGraphCounters};
use crate::render_task_graph::{RenderPassKind, RenderPass};
use crate::render_task::{RenderTask, RenderTaskLocation, RenderTaskKind};
use crate::resource_cache::{ResourceCache};
use crate::scene::{BuiltScene, SceneProperties};
use crate::segment::SegmentBuilder;
use std::{f32, mem};
use crate::util::MaxRect;


#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub enum ChasePrimitive {
    Nothing,
    Id(PrimitiveDebugId),
    LocalRect(LayoutRect),
}

impl Default for ChasePrimitive {
    fn default() -> Self {
        ChasePrimitive::Nothing
    }
}

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct FrameBuilderConfig {
    pub default_font_render_mode: FontRenderMode,
    pub dual_source_blending_is_supported: bool,
    pub dual_source_blending_is_enabled: bool,
    pub chase_primitive: ChasePrimitive,
    /// The immutable global picture caching enable from `RendererOptions`
    pub global_enable_picture_caching: bool,
    /// True if we're running tests (i.e. via wrench).
    pub testing: bool,
    pub gpu_supports_fast_clears: bool,
    pub gpu_supports_advanced_blend: bool,
    pub advanced_blend_is_coherent: bool,
    pub batch_lookback_count: usize,
    pub background_color: Option<ColorF>,
    pub compositor_kind: CompositorKind,
    pub tile_size_override: Option<DeviceIntSize>,
    pub max_depth_ids: i32,
    pub max_target_size: i32,
}

/// A set of common / global resources that are retained between
/// new display lists, such that any GPU cache handles can be
/// persisted even when a new display list arrives.
#[cfg_attr(feature = "capture", derive(Serialize))]
pub struct FrameGlobalResources {
    /// The image shader block for the most common / default
    /// set of image parameters (color white, stretch == rect.size).
    pub default_image_handle: GpuCacheHandle,

    /// A GPU cache config for drawing transparent rectangle primitives.
    /// This is used to 'cut out' overlay tiles where a compositor
    /// surface exists.
    pub default_transparent_rect_handle: GpuCacheHandle,
}

impl FrameGlobalResources {
    pub fn empty() -> Self {
        FrameGlobalResources {
            default_image_handle: GpuCacheHandle::new(),
            default_transparent_rect_handle: GpuCacheHandle::new(),
        }
    }

    pub fn update(
        &mut self,
        gpu_cache: &mut GpuCache,
    ) {
        if let Some(mut request) = gpu_cache.request(&mut self.default_image_handle) {
            request.push(PremultipliedColorF::WHITE);
            request.push(PremultipliedColorF::WHITE);
            request.push([
                -1.0,       // -ve means use prim rect for stretch size
                0.0,
                0.0,
                0.0,
            ]);
        }

        if let Some(mut request) = gpu_cache.request(&mut self.default_transparent_rect_handle) {
            request.push(PremultipliedColorF::TRANSPARENT);
        }
    }
}

/// Produces the frames that are sent to the renderer.
#[cfg_attr(feature = "capture", derive(Serialize))]
pub struct FrameBuilder {
    /// Cache of surface tiles from the previous frame builder
    /// that can optionally be consumed by this frame builder.
    pending_retained_tiles: RetainedTiles,
    pub globals: FrameGlobalResources,
}

pub struct FrameVisibilityContext<'a> {
    pub spatial_tree: &'a SpatialTree,
    pub global_screen_world_rect: WorldRect,
    pub global_device_pixel_scale: DevicePixelScale,
    pub surfaces: &'a [SurfaceInfo],
    pub debug_flags: DebugFlags,
    pub scene_properties: &'a SceneProperties,
    pub config: FrameBuilderConfig,
}

pub struct FrameVisibilityState<'a> {
    pub clip_store: &'a mut ClipStore,
    pub resource_cache: &'a mut ResourceCache,
    pub gpu_cache: &'a mut GpuCache,
    pub scratch: &'a mut PrimitiveScratchBuffer,
    pub tile_cache: Option<Box<TileCacheInstance>>,
    pub retained_tiles: &'a mut RetainedTiles,
    pub data_stores: &'a mut DataStores,
    pub clip_chain_stack: ClipChainStack,
    pub render_tasks: &'a mut RenderTaskGraph,
    pub composite_state: &'a mut CompositeState,
    /// A stack of currently active off-screen surfaces during the
    /// visibility frame traversal.
    pub surface_stack: Vec<SurfaceIndex>,
}

impl<'a> FrameVisibilityState<'a> {
    pub fn push_surface(
        &mut self,
        surface_index: SurfaceIndex,
        shared_clips: &[ClipInstance],
        spatial_tree: &SpatialTree,
    ) {
        self.surface_stack.push(surface_index);
        self.clip_chain_stack.push_surface(shared_clips, spatial_tree);
    }

    pub fn pop_surface(&mut self) {
        self.surface_stack.pop().unwrap();
        self.clip_chain_stack.pop_surface();
    }
}

pub struct FrameBuildingContext<'a> {
    pub global_device_pixel_scale: DevicePixelScale,
    pub scene_properties: &'a SceneProperties,
    pub global_screen_world_rect: WorldRect,
    pub spatial_tree: &'a SpatialTree,
    pub max_local_clip: LayoutRect,
    pub debug_flags: DebugFlags,
    pub fb_config: &'a FrameBuilderConfig,
}

pub struct FrameBuildingState<'a> {
    pub render_tasks: &'a mut RenderTaskGraph,
    pub profile_counters: &'a mut FrameProfileCounters,
    pub clip_store: &'a mut ClipStore,
    pub resource_cache: &'a mut ResourceCache,
    pub gpu_cache: &'a mut GpuCache,
    pub transforms: &'a mut TransformPalette,
    pub segment_builder: SegmentBuilder,
    pub surfaces: &'a mut Vec<SurfaceInfo>,
    pub dirty_region_stack: Vec<DirtyRegion>,
    pub composite_state: &'a mut CompositeState,
}

impl<'a> FrameBuildingState<'a> {
    /// Retrieve the current dirty region during primitive traversal.
    pub fn current_dirty_region(&self) -> &DirtyRegion {
        self.dirty_region_stack.last().unwrap()
    }

    /// Push a new dirty region for child primitives to cull / clip against.
    pub fn push_dirty_region(&mut self, region: DirtyRegion) {
        self.dirty_region_stack.push(region);
    }

    /// Pop the top dirty region from the stack.
    pub fn pop_dirty_region(&mut self) {
        self.dirty_region_stack.pop().unwrap();
    }
}

/// Immutable context of a picture when processing children.
#[derive(Debug)]
pub struct PictureContext {
    pub pic_index: PictureIndex,
    pub apply_local_clip_rect: bool,
    pub is_passthrough: bool,
    pub surface_spatial_node_index: SpatialNodeIndex,
    pub raster_spatial_node_index: SpatialNodeIndex,
    /// The surface that this picture will render on.
    pub surface_index: SurfaceIndex,
    pub dirty_region_count: usize,
    pub subpixel_mode: SubpixelMode,
}

/// Mutable state of a picture that gets modified when
/// the children are processed.
pub struct PictureState {
    pub map_local_to_pic: SpaceMapper<LayoutPixel, PicturePixel>,
    pub map_pic_to_world: SpaceMapper<PicturePixel, WorldPixel>,
    pub map_pic_to_raster: SpaceMapper<PicturePixel, RasterPixel>,
    pub map_raster_to_world: SpaceMapper<RasterPixel, WorldPixel>,
    /// If the plane splitter, the primitives get added to it instead of
    /// batching into their parent pictures.
    pub plane_splitter: Option<PlaneSplitter>,
}

impl FrameBuilder {
    pub fn new() -> Self {
        FrameBuilder {
            pending_retained_tiles: RetainedTiles::new(),
            globals: FrameGlobalResources::empty(),
        }
    }

    /// Provide any cached surface tiles from the previous frame builder
    /// to a new frame builder. These will be consumed or dropped the
    /// first time a new frame builder creates a frame.
    pub fn set_retained_resources(&mut self, retained_tiles: RetainedTiles) {
        // In general, the pending retained tiles are consumed by the frame
        // builder the first time a frame is built after a new scene has
        // arrived. However, if two scenes arrive in quick succession, the
        // frame builder may not have had a chance to build a frame and
        // consume the pending tiles. In this case, the pending tiles will
        // be lost, causing a full invalidation of the entire screen. To
        // avoid this, if there are still pending tiles, include them in
        // the retained tiles passed to the next frame builder.
        self.pending_retained_tiles.merge(retained_tiles);
    }

    /// Compute the contribution (bounding rectangles, and resources) of layers and their
    /// primitives in screen space.
    fn build_layer_screen_rects_and_cull_layers(
        &mut self,
        scene: &mut BuiltScene,
        global_screen_world_rect: WorldRect,
        resource_cache: &mut ResourceCache,
        gpu_cache: &mut GpuCache,
        render_tasks: &mut RenderTaskGraph,
        profile_counters: &mut FrameProfileCounters,
        global_device_pixel_scale: DevicePixelScale,
        scene_properties: &SceneProperties,
        transform_palette: &mut TransformPalette,
        data_stores: &mut DataStores,
        surfaces: &mut Vec<SurfaceInfo>,
        scratch: &mut PrimitiveScratchBuffer,
        debug_flags: DebugFlags,
        texture_cache_profile: &mut TextureCacheProfileCounters,
        composite_state: &mut CompositeState,
        tile_cache_logger: &mut TileCacheLogger,
    ) -> Option<RenderTaskId> {
        profile_scope!("build_layer_screen_rects_and_cull_layers");

        if scene.prim_store.pictures.is_empty() {
            return None
        }

        scratch.begin_frame();

        let root_spatial_node_index = scene.spatial_tree.root_reference_frame_index();

        const MAX_CLIP_COORD: f32 = 1.0e9;

        let frame_context = FrameBuildingContext {
            global_device_pixel_scale,
            scene_properties,
            global_screen_world_rect,
            spatial_tree: &scene.spatial_tree,
            max_local_clip: LayoutRect::new(
                LayoutPoint::new(-MAX_CLIP_COORD, -MAX_CLIP_COORD),
                LayoutSize::new(2.0 * MAX_CLIP_COORD, 2.0 * MAX_CLIP_COORD),
            ),
            debug_flags,
            fb_config: &scene.config,
        };

        let root_render_task_id = render_tasks.add().init(
            RenderTask::new_picture(
                RenderTaskLocation::Fixed(scene.output_rect),
                scene.output_rect.size.to_f32(),
                scene.root_pic_index,
                DeviceIntPoint::zero(),
                UvRectKind::Rect,
                ROOT_SPATIAL_NODE_INDEX,
                global_device_pixel_scale,
                PrimitiveVisibilityMask::all(),
                None,
                None,
            )
        );

        // Construct a dummy root surface, that represents the
        // main framebuffer surface.
        let root_surface = SurfaceInfo::new(
            ROOT_SPATIAL_NODE_INDEX,
            ROOT_SPATIAL_NODE_INDEX,
            0.0,
            global_screen_world_rect,
            &scene.spatial_tree,
            global_device_pixel_scale,
            (1.0, 1.0),
        );
        surfaces.push(root_surface);

        let mut retained_tiles = mem::replace(
            &mut self.pending_retained_tiles,
            RetainedTiles::new(),
        );

        // The first major pass of building a frame is to walk the picture
        // tree. This pass must be quick (it should never touch individual
        // primitives). For now, all we do here is determine which pictures
        // will create surfaces. In the future, this will be expanded to
        // set up render tasks, determine scaling of surfaces, and detect
        // which surfaces have valid cached surfaces that don't need to
        // be rendered this frame.
        PictureUpdateState::update_all(
            surfaces,
            scene.root_pic_index,
            &mut scene.prim_store.pictures,
            &frame_context,
            gpu_cache,
            &scene.clip_store,
            data_stores,
            composite_state,
        );

        {
            profile_scope!("UpdateVisibility");
            profile_marker!("UpdateVisibility");

            let visibility_context = FrameVisibilityContext {
                global_device_pixel_scale,
                spatial_tree: &scene.spatial_tree,
                global_screen_world_rect,
                surfaces,
                debug_flags,
                scene_properties,
                config: scene.config,
            };

            let mut visibility_state = FrameVisibilityState {
                resource_cache,
                gpu_cache,
                clip_store: &mut scene.clip_store,
                scratch,
                tile_cache: None,
                retained_tiles: &mut retained_tiles,
                data_stores,
                clip_chain_stack: ClipChainStack::new(),
                render_tasks,
                composite_state,
                /// Try to avoid allocating during frame traversal - it's unlikely to have a
                /// surface stack depth of > 16 in most cases.
                surface_stack: Vec::with_capacity(16),
            };

            scene.prim_store.update_visibility(
                scene.root_pic_index,
                ROOT_SURFACE_INDEX,
                &global_screen_world_rect,
                &visibility_context,
                &mut visibility_state,
            );

            // When there are tiles that are left remaining in the `retained_tiles`,
            // dirty rects are not valid.
            if !visibility_state.retained_tiles.caches.is_empty() {
              visibility_state.composite_state.dirty_rects_are_valid = false;
            }

            // When a new display list is processed by WR, the existing tiles from
            // any picture cache are stored in the `retained_tiles` field above. This
            // allows the first frame of a new display list to reuse any existing tiles
            // and surfaces that match. Once the `update_visibility` call above is
            // complete, any tiles that are left remaining in the `retained_tiles`
            // map are not needed and will be dropped. For simple compositing mode,
            // this is fine, since texture cache handles are garbage collected at
            // the end of each frame. However, if we're in native compositor mode,
            // we need to manually clean up any native compositor surfaces that were
            // allocated by these tiles.
            for (_, mut cache_state) in visibility_state.retained_tiles.caches.drain() {
                if let Some(native_surface) = cache_state.native_surface.take() {
                    visibility_state.resource_cache.destroy_compositor_surface(native_surface.opaque);
                    visibility_state.resource_cache.destroy_compositor_surface(native_surface.alpha);
                }

                for (_, external_surface) in cache_state.external_native_surface_cache.drain() {
                    visibility_state.resource_cache.destroy_compositor_surface(external_surface.native_surface_id)
                }
            }
        }

        let mut frame_state = FrameBuildingState {
            render_tasks,
            profile_counters,
            clip_store: &mut scene.clip_store,
            resource_cache,
            gpu_cache,
            transforms: transform_palette,
            segment_builder: SegmentBuilder::new(),
            surfaces,
            dirty_region_stack: Vec::new(),
            composite_state,
        };

        frame_state
            .surfaces
            .first_mut()
            .unwrap()
            .render_tasks = Some(SurfaceRenderTasks {
                root: root_render_task_id,
                port: root_render_task_id,
            });

        // Push a default dirty region which culls primitives
        // against the screen world rect, in absence of any
        // other dirty regions.
        let mut default_dirty_region = DirtyRegion::new();
        default_dirty_region.push(
            frame_context.global_screen_world_rect,
            PrimitiveVisibilityMask::all(),
        );
        frame_state.push_dirty_region(default_dirty_region);

        let (pic_context, mut pic_state, mut prim_list) = scene
            .prim_store
            .pictures[scene.root_pic_index.0]
            .take_context(
                scene.root_pic_index,
                WorldRect::max_rect(),
                root_spatial_node_index,
                root_spatial_node_index,
                ROOT_SURFACE_INDEX,
                &SubpixelMode::Allow,
                &mut frame_state,
                &frame_context,
                scratch,
                tile_cache_logger
            )
            .unwrap();

        tile_cache_logger.advance();

        {
            profile_marker!("PreparePrims");

            scene.prim_store.prepare_primitives(
                &mut prim_list,
                &pic_context,
                &mut pic_state,
                &frame_context,
                &mut frame_state,
                data_stores,
                scratch,
                tile_cache_logger,
            );
        }

        let pic = &mut scene.prim_store.pictures[scene.root_pic_index.0];
        pic.restore_context(
            ROOT_SURFACE_INDEX,
            prim_list,
            pic_context,
            pic_state,
            &mut frame_state,
        );

        frame_state.pop_dirty_region();

        {
            profile_marker!("BlockOnResources");

            resource_cache.block_until_all_resources_added(gpu_cache,
                                                           render_tasks,
                                                           texture_cache_profile);
        }

        Some(root_render_task_id)
    }

    pub fn build(
        &mut self,
        scene: &mut BuiltScene,
        resource_cache: &mut ResourceCache,
        gpu_cache: &mut GpuCache,
        stamp: FrameStamp,
        global_device_pixel_scale: DevicePixelScale,
        layer: DocumentLayer,
        device_origin: DeviceIntPoint,
        pan: WorldPoint,
        resource_profile: &mut ResourceProfileCounters,
        scene_properties: &SceneProperties,
        data_stores: &mut DataStores,
        scratch: &mut PrimitiveScratchBuffer,
        render_task_counters: &mut RenderTaskGraphCounters,
        debug_flags: DebugFlags,
        tile_cache_logger: &mut TileCacheLogger,
    ) -> Frame {
        profile_scope!("build");
        profile_marker!("BuildFrame");

        let mut profile_counters = FrameProfileCounters::new();
        profile_counters
            .total_primitives
            .set(scene.prim_store.prim_count());
        resource_profile.content_slices.set(scene.content_slice_count);
        resource_cache.begin_frame(stamp);
        gpu_cache.begin_frame(stamp);

        self.globals.update(gpu_cache);

        scene.spatial_tree.update_tree(
            pan,
            global_device_pixel_scale,
            scene_properties,
        );
        let mut transform_palette = scene.spatial_tree.build_transform_palette();
        scene.clip_store.clear_old_instances();

        let mut render_tasks = RenderTaskGraph::new(
            stamp.frame_id(),
            render_task_counters,
        );
        let mut surfaces = Vec::new();

        let output_size = scene.output_rect.size.to_i32();
        let screen_world_rect = (scene.output_rect.to_f32() / global_device_pixel_scale).round_out();

        // Determine if we will draw this frame with picture caching enabled. This depends on:
        // (1) If globally enabled when WR was initialized
        // (2) If current debug flags allow picture caching
        // (3) Whether we are currently pinch zooming
        // (4) If any picture cache spatial nodes are not in the root coordinate system
        let picture_caching_is_enabled =
            scene.config.global_enable_picture_caching &&
            !debug_flags.contains(DebugFlags::DISABLE_PICTURE_CACHING) &&
            !scene.picture_cache_spatial_nodes.iter().any(|spatial_node_index| {
                let spatial_node = &scene
                    .spatial_tree
                    .spatial_nodes[spatial_node_index.0 as usize];
                spatial_node.is_ancestor_or_self_zooming
            });

        let mut composite_state = CompositeState::new(
            scene.config.compositor_kind,
            picture_caching_is_enabled,
            global_device_pixel_scale,
            scene.config.max_depth_ids,
        );

        let main_render_task_id = self.build_layer_screen_rects_and_cull_layers(
            scene,
            screen_world_rect,
            resource_cache,
            gpu_cache,
            &mut render_tasks,
            &mut profile_counters,
            global_device_pixel_scale,
            scene_properties,
            &mut transform_palette,
            data_stores,
            &mut surfaces,
            scratch,
            debug_flags,
            &mut resource_profile.texture_cache,
            &mut composite_state,
            tile_cache_logger,
        );

        let mut passes;
        let mut deferred_resolves = vec![];
        let mut has_texture_cache_tasks = false;
        let mut prim_headers = PrimitiveHeaders::new();

        {
            profile_marker!("Batching");

            passes = render_tasks.generate_passes(
                main_render_task_id,
                output_size,
                scene.config.gpu_supports_fast_clears,
            );

            // Used to generated a unique z-buffer value per primitive.
            let mut z_generator = ZBufferIdGenerator::new(layer, scene.config.max_depth_ids);
            let use_dual_source_blending = scene.config.dual_source_blending_is_enabled &&
                                           scene.config.dual_source_blending_is_supported;

            for pass in &mut passes {
                let mut ctx = RenderTargetContext {
                    global_device_pixel_scale,
                    prim_store: &scene.prim_store,
                    resource_cache,
                    use_dual_source_blending,
                    use_advanced_blending: scene.config.gpu_supports_advanced_blend,
                    break_advanced_blend_batches: !scene.config.advanced_blend_is_coherent,
                    batch_lookback_count: scene.config.batch_lookback_count,
                    spatial_tree: &scene.spatial_tree,
                    data_stores,
                    surfaces: &surfaces,
                    scratch,
                    screen_world_rect,
                    globals: &self.globals,
                };

                build_render_pass(
                    pass,
                    &mut ctx,
                    gpu_cache,
                    &mut render_tasks,
                    &mut deferred_resolves,
                    &scene.clip_store,
                    &mut transform_palette,
                    &mut prim_headers,
                    &mut z_generator,
                    &mut composite_state,
                );

                match pass.kind {
                    RenderPassKind::MainFramebuffer { .. } => {}
                    RenderPassKind::OffScreen {
                        ref texture_cache,
                        ref picture_cache,
                        ..
                    } => {
                        has_texture_cache_tasks |= !texture_cache.is_empty();
                        has_texture_cache_tasks |= !picture_cache.is_empty();
                    }
                }
            }
        }

        let gpu_cache_frame_id = gpu_cache.end_frame(&mut resource_profile.gpu_cache).frame_id();

        render_tasks.write_task_data();
        *render_task_counters = render_tasks.counters();
        resource_cache.end_frame(&mut resource_profile.texture_cache);

        Frame {
            content_origin: scene.output_rect.origin,
            device_rect: DeviceIntRect::new(
                device_origin,
                scene.output_rect.size,
            ),
            layer,
            profile_counters,
            passes,
            transform_palette: transform_palette.finish(),
            render_tasks,
            deferred_resolves,
            gpu_cache_frame_id,
            has_been_rendered: false,
            has_texture_cache_tasks,
            prim_headers,
            recorded_dirty_regions: mem::replace(&mut scratch.recorded_dirty_regions, Vec::new()),
            debug_items: mem::replace(&mut scratch.debug_items, Vec::new()),
            composite_state,
        }
    }
}

/// Processes this pass to prepare it for rendering.
///
/// Among other things, this allocates output regions for each of our tasks
/// (added via `add_render_task`) in a RenderTarget and assigns it into that
/// target.
pub fn build_render_pass(
    pass: &mut RenderPass,
    ctx: &mut RenderTargetContext,
    gpu_cache: &mut GpuCache,
    render_tasks: &mut RenderTaskGraph,
    deferred_resolves: &mut Vec<DeferredResolve>,
    clip_store: &ClipStore,
    transforms: &mut TransformPalette,
    prim_headers: &mut PrimitiveHeaders,
    z_generator: &mut ZBufferIdGenerator,
    composite_state: &mut CompositeState,
) {
    profile_scope!("build_render_pass");

    match pass.kind {
        RenderPassKind::MainFramebuffer { ref mut main_target, .. } => {
            profile_scope!("MainFrameBuffer");
            for &task_id in &pass.tasks {
                profile_scope!("task");
                assert_eq!(render_tasks[task_id].target_kind(), RenderTargetKind::Color);
                main_target.add_task(
                    task_id,
                    ctx,
                    gpu_cache,
                    render_tasks,
                    clip_store,
                    transforms,
                    deferred_resolves,
                );
            }
            main_target.build(
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
        RenderPassKind::OffScreen {
            ref mut color,
            ref mut alpha,
            ref mut texture_cache,
            ref mut picture_cache,
        } => {
            profile_scope!("OffScreen");
            let saved_color = if pass.tasks.iter().any(|&task_id| {
                let t = &render_tasks[task_id];
                t.target_kind() == RenderTargetKind::Color && t.saved_index.is_some()
            }) {
                Some(render_tasks.save_target())
            } else {
                None
            };
            let saved_alpha = if pass.tasks.iter().any(|&task_id| {
                let t = &render_tasks[task_id];
                t.target_kind() == RenderTargetKind::Alpha && t.saved_index.is_some()
            }) {
                Some(render_tasks.save_target())
            } else {
                None
            };

            // Collect a list of picture cache tasks, keyed by picture index.
            // This allows us to only walk that picture root once, adding the
            // primitives to all relevant batches at the same time.
            let mut picture_cache_tasks = FastHashMap::default();

            // Step through each task, adding to batches as appropriate.
            for &task_id in &pass.tasks {
                let (target_kind, texture_target, layer) = {
                    let task = &mut render_tasks[task_id];
                    let target_kind = task.target_kind();

                    // Find a target to assign this task to, or create a new
                    // one if required.
                    let (texture_target, layer) = match task.location {
                        RenderTaskLocation::TextureCache { texture, layer, .. } => {
                            (Some(texture), layer)
                        }
                        RenderTaskLocation::Fixed(..) => {
                            (None, 0)
                        }
                        RenderTaskLocation::Dynamic(ref mut origin, size) => {
                            let (target_index, alloc_origin) =  match target_kind {
                                RenderTargetKind::Color => color.allocate(size),
                                RenderTargetKind::Alpha => alpha.allocate(size),
                            };
                            *origin = Some((alloc_origin, target_index));
                            (None, target_index.0)
                        }
                        RenderTaskLocation::PictureCache { .. } => {
                            // For picture cache tiles, just store them in the map
                            // of picture cache tasks, to be handled below.
                            let pic_index = match task.kind {
                                RenderTaskKind::Picture(ref info) => {
                                    info.pic_index
                                }
                                _ => {
                                    unreachable!();
                                }
                            };

                            picture_cache_tasks
                                .entry(pic_index)
                                .or_insert_with(Vec::new)
                                .push(task_id);

                            continue;
                        }
                    };

                    // Replace the pending saved index with a real one
                    if let Some(index) = task.saved_index {
                        assert_eq!(index, SavedTargetIndex::PENDING);
                        task.saved_index = match target_kind {
                            RenderTargetKind::Color => saved_color,
                            RenderTargetKind::Alpha => saved_alpha,
                        };
                    }

                    // Give the render task an opportunity to add any
                    // information to the GPU cache, if appropriate.
                    task.write_gpu_blocks(gpu_cache);

                    (target_kind, texture_target, layer)
                };

                match texture_target {
                    Some(texture_target) => {
                        let texture = texture_cache
                            .entry((texture_target, layer))
                            .or_insert_with(||
                                TextureCacheRenderTarget::new(target_kind)
                            );
                        texture.add_task(task_id, render_tasks);
                    }
                    None => {
                        match target_kind {
                            RenderTargetKind::Color => {
                                color.targets[layer].add_task(
                                    task_id,
                                    ctx,
                                    gpu_cache,
                                    render_tasks,
                                    clip_store,
                                    transforms,
                                    deferred_resolves,
                                )
                            }
                            RenderTargetKind::Alpha => {
                                alpha.targets[layer].add_task(
                                    task_id,
                                    ctx,
                                    gpu_cache,
                                    render_tasks,
                                    clip_store,
                                    transforms,
                                    deferred_resolves,
                                )
                            }
                        }
                    }
                }
            }

            // For each picture in this pass that has picture cache tiles, create
            // a batcher per task, and then build batches for each of the tasks
            // at the same time.
            for (pic_index, task_ids) in picture_cache_tasks {
                profile_scope!("picture_cache_task");
                let pic = &ctx.prim_store.pictures[pic_index.0];
                let tile_cache = pic.tile_cache.as_ref().expect("bug");

                // Extract raster/surface spatial nodes for this surface.
                let (root_spatial_node_index, surface_spatial_node_index) = match pic.raster_config {
                    Some(ref rc) => {
                        let surface = &ctx.surfaces[rc.surface_index.0];
                        (surface.raster_spatial_node_index, surface.surface_spatial_node_index)
                    }
                    None => {
                        unreachable!();
                    }
                };

                // Determine the clear color for this picture cache.
                // If the entire tile cache is opaque, we can skip clear completely.
                // If it's the first layer, clear it to white to allow subpixel AA on that
                // first layer even if it's technically transparent.
                // Otherwise, clear to transparent and composite with alpha.
                // TODO(gw): We can detect per-tile opacity for the clear color here
                //           which might be a significant win on some pages?
                let forced_opaque = match tile_cache.background_color {
                    Some(color) => color.a >= 1.0,
                    None => false,
                };
                let mut clear_color = if forced_opaque {
                    Some(ColorF::WHITE)
                } else {
                    Some(ColorF::TRANSPARENT)
                };

                // If this picture cache has a valid color backdrop, we will use
                // that as the clear color, skipping the draw of the backdrop
                // primitive (and anything prior to it) during batching.
                if let Some(BackdropKind::Color { color }) = tile_cache.backdrop.kind {
                    clear_color = Some(color);
                }

                // Create an alpha batcher for each of the tasks of this picture.
                let mut batchers = Vec::new();
                for task_id in &task_ids {
                    let task_id = *task_id;
                    let vis_mask = match render_tasks[task_id].kind {
                        RenderTaskKind::Picture(ref info) => info.vis_mask,
                        _ => unreachable!(),
                    };
                    batchers.push(AlphaBatchBuilder::new(
                        pass.screen_size,
                        ctx.break_advanced_blend_batches,
                        ctx.batch_lookback_count,
                        task_id,
                        render_tasks.get_task_address(task_id),
                        vis_mask,
                    ));
                }

                // Run the batch creation code for this picture, adding items to
                // all relevant per-task batchers.
                let mut batch_builder = BatchBuilder::new(batchers);
                {
                profile_scope!("add_pic_to_batch");
                batch_builder.add_pic_to_batch(
                    pic,
                    ctx,
                    gpu_cache,
                    render_tasks,
                    deferred_resolves,
                    prim_headers,
                    transforms,
                    root_spatial_node_index,
                    surface_spatial_node_index,
                    z_generator,
                    composite_state,
                );
                }

                // Create picture cache targets, one per render task, and assign
                // the correct batcher to them.
                let batchers = batch_builder.finalize();
                for (task_id, batcher) in task_ids.into_iter().zip(batchers.into_iter()) {
                    profile_scope!("task");
                    let task = &render_tasks[task_id];
                    let (target_rect, _) = task.get_target_rect();

                    match task.location {
                        RenderTaskLocation::PictureCache { ref surface, .. } => {
                            // TODO(gw): The interface here is a bit untidy since it's
                            //           designed to support batch merging, which isn't
                            //           relevant for picture cache targets. We
                            //           can restructure / tidy this up a bit.
                            let (scissor_rect, valid_rect)  = match render_tasks[task_id].kind {
                                RenderTaskKind::Picture(ref info) => {
                                    (
                                        info.scissor_rect.expect("bug: must be set for cache tasks"),
                                        info.valid_rect.expect("bug: must be set for cache tasks"),
                                    )
                                }
                                _ => unreachable!(),
                            };
                            let mut batch_containers = Vec::new();
                            let mut alpha_batch_container = AlphaBatchContainer::new(Some(scissor_rect));
                            batcher.build(
                                &mut batch_containers,
                                &mut alpha_batch_container,
                                target_rect,
                                None,
                            );
                            debug_assert!(batch_containers.is_empty());

                            let target = PictureCacheTarget {
                                surface: surface.clone(),
                                clear_color,
                                alpha_batch_container,
                                dirty_rect: scissor_rect,
                                valid_rect,
                            };

                            picture_cache.push(target);
                        }
                        _ => {
                            unreachable!()
                        }
                    }
                }
            }

            color.build(
                ctx,
                gpu_cache,
                render_tasks,
                deferred_resolves,
                saved_color,
                prim_headers,
                transforms,
                z_generator,
                composite_state,
            );
            alpha.build(
                ctx,
                gpu_cache,
                render_tasks,
                deferred_resolves,
                saved_alpha,
                prim_headers,
                transforms,
                z_generator,
                composite_state,
            );
        }
    }
}

/// A rendering-oriented representation of the frame built by the render backend
/// and presented to the renderer.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct Frame {
    /// The origin on content produced by the render tasks.
    pub content_origin: DeviceIntPoint,
    /// The rectangle to show the frame in, on screen.
    pub device_rect: DeviceIntRect,
    pub layer: DocumentLayer,
    pub passes: Vec<RenderPass>,
    #[cfg_attr(any(feature = "capture", feature = "replay"), serde(default = "FrameProfileCounters::new", skip))]
    pub profile_counters: FrameProfileCounters,

    pub transform_palette: Vec<TransformData>,
    pub render_tasks: RenderTaskGraph,
    pub prim_headers: PrimitiveHeaders,

    /// The GPU cache frame that the contents of Self depend on
    pub gpu_cache_frame_id: FrameId,

    /// List of textures that we don't know about yet
    /// from the backend thread. The render thread
    /// will use a callback to resolve these and
    /// patch the data structures.
    pub deferred_resolves: Vec<DeferredResolve>,

    /// True if this frame contains any render tasks
    /// that write to the texture cache.
    pub has_texture_cache_tasks: bool,

    /// True if this frame has been drawn by the
    /// renderer.
    pub has_been_rendered: bool,

    /// Dirty regions recorded when generating this frame. Empty when not in
    /// testing.
    #[cfg_attr(feature = "serde", serde(skip))]
    pub recorded_dirty_regions: Vec<RecordedDirtyRegion>,

    /// Debugging information to overlay for this frame.
    pub debug_items: Vec<DebugItem>,

    /// Contains picture cache tiles, and associated information.
    /// Used by the renderer to composite tiles into the framebuffer,
    /// or hand them off to an OS compositor.
    pub composite_state: CompositeState,
}

impl Frame {
    // This frame must be flushed if it writes to the
    // texture cache, and hasn't been drawn yet.
    pub fn must_be_drawn(&self) -> bool {
        self.has_texture_cache_tasks && !self.has_been_rendered
    }

    // Returns true if this frame doesn't alter what is on screen currently.
    pub fn is_nop(&self) -> bool {
        // If picture caching is disabled, we don't have enough information
        // to know if this frame is a nop, so it gets drawn unconditionally.
        if !self.composite_state.picture_caching_is_enabled {
            return false;
        }

        // When picture caching is enabled, the first (main framebuffer) pass
        // consists of compositing tiles only (whether via the simple compositor
        // or the native OS compositor). If there are no other passes, that
        // implies that none of the picture cache tiles were updated, and thus
        // the frame content must be exactly the same as last frame. If this is
        // true, drawing this frame is a no-op and can be skipped.

        if self.passes.len() > 1 {
            return false;
        }

        true
    }
}
