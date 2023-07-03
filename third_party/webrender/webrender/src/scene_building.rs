/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use api::{AlphaType, BorderDetails, BorderDisplayItem, BuiltDisplayListIter, PrimitiveFlags};
use api::{ClipId, ColorF, CommonItemProperties, ComplexClipRegion, ComponentTransferFuncType, RasterSpace};
use api::{DisplayItem, DisplayItemRef, ExtendMode, ExternalScrollId, FilterData, SharedFontInstanceMap};
use api::{FilterOp, FilterPrimitive, FontInstanceKey, FontSize, GlyphInstance, GlyphOptions, GradientStop};
use api::{IframeDisplayItem, ImageKey, ImageRendering, ItemRange, ColorDepth, QualitySettings};
use api::{LineOrientation, LineStyle, NinePatchBorderSource, PipelineId, MixBlendMode, StackingContextFlags};
use api::{PropertyBinding, ReferenceFrame, ReferenceFrameKind, ScrollFrameDisplayItem, ScrollSensitivity};
use api::{Shadow, SpaceAndClipInfo, SpatialId, StackingContext, StickyFrameDisplayItem, ImageMask};
use api::{ClipMode, PrimitiveKeyKind, TransformStyle, YuvColorSpace, ColorRange, YuvData, TempFilterData};
use api::image_tiling::simplify_repeated_primitive;
use api::units::*;
use crate::clip::{ClipChainId, ClipRegion, ClipItemKey, ClipStore, ClipItemKeyKind};
use crate::clip::{ClipInternData, ClipNodeKind, ClipInstance};
use crate::spatial_tree::{ROOT_SPATIAL_NODE_INDEX, SpatialTree, SpatialNodeIndex};
use crate::frame_builder::{ChasePrimitive, FrameBuilderConfig};
use crate::glyph_rasterizer::FontInstance;
use crate::hit_test::{HitTestingItem, HitTestingScene};
use crate::intern::Interner;
use crate::internal_types::{FastHashMap, FastHashSet, LayoutPrimitiveInfo, Filter};
use crate::picture::{Picture3DContext, PictureCompositeMode, PicturePrimitive, PictureOptions};
use crate::picture::{BlitReason, OrderedPictureChild, PrimitiveList, TileCacheInstance, ClusterFlags};
use crate::prim_store::PrimitiveInstance;
use crate::prim_store::{PrimitiveInstanceKind, NinePatchDescriptor, PrimitiveStore};
use crate::prim_store::{InternablePrimitive, SegmentInstanceIndex, PictureIndex};
use crate::prim_store::{register_prim_chase_id, get_line_decoration_size};
use crate::prim_store::{SpaceSnapper};
use crate::prim_store::backdrop::Backdrop;
use crate::prim_store::borders::{ImageBorder, NormalBorderPrim};
use crate::prim_store::gradient::{GradientStopKey, LinearGradient, RadialGradient, RadialGradientParams, ConicGradient, ConicGradientParams};
use crate::prim_store::image::{Image, YuvImage};
use crate::prim_store::line_dec::{LineDecoration, LineDecorationCacheKey};
use crate::prim_store::picture::{Picture, PictureCompositeKey, PictureKey};
use crate::prim_store::text_run::TextRun;
use crate::render_backend::SceneView;
use crate::resource_cache::ImageRequest;
use crate::scene::{Scene, BuiltScene, SceneStats, StackingContextHelpers};
use crate::scene_builder_thread::Interners;
use crate::spatial_node::{StickyFrameInfo, ScrollFrameKind};
use euclid::approxeq::ApproxEq;
use std::{f32, mem, usize, ops};
use std::collections::vec_deque::VecDeque;
use std::sync::Arc;
use crate::util::{MaxRect, VecHelper};
use crate::filterdata::{SFilterDataComponent, SFilterData, SFilterDataKey};
use smallvec::SmallVec;

/// The offset stack for a given reference frame.
struct ReferenceFrameState {
    /// A stack of current offsets from the current reference frame scope.
    offsets: Vec<LayoutVector2D>,
}

/// Maps from stacking context layout coordinates into reference frame
/// relative coordinates.
struct ReferenceFrameMapper {
    /// A stack of reference frame scopes.
    frames: Vec<ReferenceFrameState>,
}

impl ReferenceFrameMapper {
    fn new() -> Self {
        ReferenceFrameMapper {
            frames: vec![
                ReferenceFrameState {
                    offsets: vec![
                        LayoutVector2D::zero(),
                    ],
                }
            ],
        }
    }

    /// Push a new scope. This resets the current offset to zero, and is
    /// used when a new reference frame or iframe is pushed.
    fn push_scope(&mut self) {
        self.frames.push(ReferenceFrameState {
            offsets: vec![
                LayoutVector2D::zero(),
            ],
        });
    }

    /// Pop a reference frame scope off the stack.
    fn pop_scope(&mut self) {
        self.frames.pop().unwrap();
    }

    /// Push a new offset for the current scope. This is used when
    /// a new stacking context is pushed.
    fn push_offset(&mut self, offset: LayoutVector2D) {
        let frame = self.frames.last_mut().unwrap();
        let current_offset = *frame.offsets.last().unwrap();
        frame.offsets.push(current_offset + offset);
    }

    /// Pop a local stacking context offset from the current scope.
    fn pop_offset(&mut self) {
        let frame = self.frames.last_mut().unwrap();
        frame.offsets.pop().unwrap();
    }

    /// Retrieve the current offset to allow converting a stacking context
    /// relative coordinate to be relative to the owing reference frame.
    /// TODO(gw): We could perhaps have separate coordinate spaces for this,
    ///           however that's going to either mean a lot of changes to
    ///           public API code, or a lot of changes to internal code.
    ///           Before doing that, we should revisit how Gecko would
    ///           prefer to provide coordinates.
    /// TODO(gw): For now, this includes only the reference frame relative
    ///           offset. Soon, we will expand this to include the initial
    ///           scroll offsets that are now available on scroll nodes. This
    ///           will allow normalizing the coordinates even between display
    ///           lists where APZ has scrolled the content.
    fn current_offset(&self) -> LayoutVector2D {
        *self.frames.last().unwrap().offsets.last().unwrap()
    }
}

/// Offsets primitives (and clips) by the external scroll offset
/// supplied to scroll nodes.
pub struct ScrollOffsetMapper {
    pub current_spatial_node: SpatialNodeIndex,
    pub current_offset: LayoutVector2D,
}

impl ScrollOffsetMapper {
    fn new() -> Self {
        ScrollOffsetMapper {
            current_spatial_node: SpatialNodeIndex::INVALID,
            current_offset: LayoutVector2D::zero(),
        }
    }

    /// Return the accumulated external scroll offset for a spatial
    /// node. This caches the last result, which is the common case,
    /// or defers to the spatial tree to build the value.
    fn external_scroll_offset(
        &mut self,
        spatial_node_index: SpatialNodeIndex,
        spatial_tree: &SpatialTree,
    ) -> LayoutVector2D {
        if spatial_node_index != self.current_spatial_node {
            self.current_spatial_node = spatial_node_index;
            self.current_offset = spatial_tree.external_scroll_offset(spatial_node_index);
        }

        self.current_offset
    }
}

/// A data structure that keeps track of mapping between API Ids for spatials and the indices
/// used internally in the SpatialTree to avoid having to do HashMap lookups for primitives
/// and clips during frame building.
#[derive(Default)]
pub struct NodeIdToIndexMapper {
    spatial_node_map: FastHashMap<SpatialId, SpatialNodeIndex>,
}

impl NodeIdToIndexMapper {
    fn add_spatial_node(&mut self, id: SpatialId, index: SpatialNodeIndex) {
        let _old_value = self.spatial_node_map.insert(id, index);
        debug_assert!(_old_value.is_none());
    }

    fn get_spatial_node_index(&self, id: SpatialId) -> SpatialNodeIndex {
        self.spatial_node_map[&id]
    }
}

#[derive(Debug, Clone, Default)]
pub struct CompositeOps {
    // Requires only a single texture as input (e.g. most filters)
    pub filters: Vec<Filter>,
    pub filter_datas: Vec<FilterData>,
    pub filter_primitives: Vec<FilterPrimitive>,

    // Requires two source textures (e.g. mix-blend-mode)
    pub mix_blend_mode: Option<MixBlendMode>,
}

impl CompositeOps {
    pub fn new(
        filters: Vec<Filter>,
        filter_datas: Vec<FilterData>,
        filter_primitives: Vec<FilterPrimitive>,
        mix_blend_mode: Option<MixBlendMode>
    ) -> Self {
        CompositeOps {
            filters,
            filter_datas,
            filter_primitives,
            mix_blend_mode,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.filters.is_empty() &&
            self.filter_primitives.is_empty() &&
            self.mix_blend_mode.is_none()
    }
}

bitflags! {
    /// Slice flags
    pub struct SliceFlags : u8 {
        /// Slice created by a cluster that has ClusterFlags::SCROLLBAR_CONTAINER
        const IS_SCROLLBAR = 1;
    }
}

/// Information about a set of primitive clusters that will form a picture cache slice.
struct Slice {
    /// The spatial node root of the picture cache. If this is None, the slice
    /// will not be cached and instead drawn directly to the parent surface. This
    /// is a temporary measure until we enable caching all slices.
    cache_scroll_root: Option<SpatialNodeIndex>,
    /// List of primitive clusters that make up this slice
    prim_list: PrimitiveList,
    /// A list of clips that are shared by all primitives in the slice. These can be
    /// filtered out and applied when the tile cache is composited rather than per-item.
    shared_clips: Option<Vec<ClipInstance>>,
    /// Various flags describing properties of this slice
    pub flags: SliceFlags,
}

/// A structure that converts a serialized display list into a form that WebRender
/// can use to later build a frame. This structure produces a BuiltScene. Public
/// members are typically those that are destructured into the BuiltScene.
pub struct SceneBuilder<'a> {
    /// The scene that we are currently building.
    scene: &'a Scene,

    /// The map of all font instances.
    font_instances: SharedFontInstanceMap,

    /// A set of pipelines that the caller has requested be made available as
    /// output textures.
    output_pipelines: &'a FastHashSet<PipelineId>,

    /// The data structure that converts between ClipId/SpatialId and the various
    /// index types that the SpatialTree uses.
    id_to_index_mapper: NodeIdToIndexMapper,

    /// A stack of stacking context properties.
    sc_stack: Vec<FlattenedStackingContext>,

    /// Maintains state for any currently active shadows
    pending_shadow_items: VecDeque<ShadowItem>,

    /// The SpatialTree that we are currently building during building.
    pub spatial_tree: SpatialTree,

    /// The store of primitives.
    pub prim_store: PrimitiveStore,

    /// Information about all primitives involved in hit testing.
    pub hit_testing_scene: HitTestingScene,

    /// The store which holds all complex clipping information.
    pub clip_store: ClipStore,

    /// The configuration to use for the FrameBuilder. We consult this in
    /// order to determine the default font.
    pub config: FrameBuilderConfig,

    /// Reference to the set of data that is interned across display lists.
    interners: &'a mut Interners,

    /// The root picture index for this builder. This is the picture
    /// to start the culling phase from.
    pub root_pic_index: PictureIndex,

    /// Helper struct to map stacking context coords <-> reference frame coords.
    rf_mapper: ReferenceFrameMapper,

    /// Helper struct to map spatial nodes to external scroll offsets.
    external_scroll_mapper: ScrollOffsetMapper,

    /// If true, picture caching setup has already been completed.
    picture_caching_initialized: bool,

    /// The current recursion depth of iframes encountered. Used to restrict picture
    /// caching slices to only the top-level content frame.
    iframe_depth: usize,

    /// The number of picture cache slices that were created for content.
    content_slice_count: usize,

    /// A set of any spatial nodes that are attached to either a picture cache
    /// root, or a clip node on the picture cache primitive. These are used
    /// to detect cases where picture caching must be disabled. This is mostly
    /// a temporary workaround for some existing wrench tests. I don't think
    /// Gecko ever produces picture cache slices with complex transforms, so
    /// in future we should prevent this in the public API and remove this hack.
    picture_cache_spatial_nodes: FastHashSet<SpatialNodeIndex>,

    /// The current quality / performance settings for this scene.
    quality_settings: QualitySettings,
}

impl<'a> SceneBuilder<'a> {
    pub fn build(
        scene: &Scene,
        font_instances: SharedFontInstanceMap,
        view: &SceneView,
        output_pipelines: &FastHashSet<PipelineId>,
        frame_builder_config: &FrameBuilderConfig,
        interners: &mut Interners,
        stats: &SceneStats,
    ) -> BuiltScene {
        profile_scope!("build_scene");

        // We checked that the root pipeline is available on the render backend.
        let root_pipeline_id = scene.root_pipeline_id.unwrap();
        let root_pipeline = scene.pipelines.get(&root_pipeline_id).unwrap();

        let background_color = root_pipeline
            .background_color
            .and_then(|color| if color.a > 0.0 { Some(color) } else { None });

        let mut builder = SceneBuilder {
            scene,
            spatial_tree: SpatialTree::new(),
            font_instances,
            config: *frame_builder_config,
            output_pipelines,
            id_to_index_mapper: NodeIdToIndexMapper::default(),
            hit_testing_scene: HitTestingScene::new(&stats.hit_test_stats),
            pending_shadow_items: VecDeque::new(),
            sc_stack: Vec::new(),
            prim_store: PrimitiveStore::new(&stats.prim_store_stats),
            clip_store: ClipStore::new(),
            interners,
            root_pic_index: PictureIndex(0),
            rf_mapper: ReferenceFrameMapper::new(),
            external_scroll_mapper: ScrollOffsetMapper::new(),
            picture_caching_initialized: false,
            iframe_depth: 0,
            content_slice_count: 0,
            picture_cache_spatial_nodes: FastHashSet::default(),
            quality_settings: view.quality_settings,
        };

        let device_pixel_scale = view.accumulated_scale_factor_for_snapping();

        builder.clip_store.register_clip_template(
            ClipId::root(root_pipeline_id),
            ClipId::root(root_pipeline_id),
            &[],
        );

        builder.clip_store.push_clip_root(
            Some(ClipId::root(root_pipeline_id)),
            false,
        );

        builder.push_root(
            root_pipeline_id,
            &root_pipeline.viewport_size,
            &root_pipeline.content_size,
            device_pixel_scale,
        );

        // In order to ensure we have a single root stacking context for the
        // entire display list, we push one here. Gecko _almost_ wraps its
        // entire display list within a single stacking context, but sometimes
        // appends a few extra items in AddWindowOverlayWebRenderCommands. We
        // could fix it there, but it's easier and more robust for WebRender
        // to just ensure there's a context on the stack whenever we append
        // primitives (since otherwise we'd panic).
        //
        // Note that we don't do this for iframes, even if they're pipeline
        // roots, because they should be entirely contained within a stacking
        // context, and we probably wouldn't crash if they weren't.
        builder.push_stacking_context(
            root_pipeline.pipeline_id,
            CompositeOps::default(),
            TransformStyle::Flat,
            /* prim_flags = */ PrimitiveFlags::IS_BACKFACE_VISIBLE,
            ROOT_SPATIAL_NODE_INDEX,
            None,
            RasterSpace::Screen,
            StackingContextFlags::IS_BACKDROP_ROOT,
            device_pixel_scale,
        );

        builder.build_items(
            &mut root_pipeline.display_list.iter(),
            root_pipeline.pipeline_id,
        );

        builder.pop_stacking_context();
        builder.clip_store.pop_clip_root();

        debug_assert!(builder.sc_stack.is_empty());

        BuiltScene {
            has_root_pipeline: scene.has_root_pipeline(),
            pipeline_epochs: scene.pipeline_epochs.clone(),
            output_rect: view.device_rect.size.into(),
            background_color,
            hit_testing_scene: Arc::new(builder.hit_testing_scene),
            spatial_tree: builder.spatial_tree,
            prim_store: builder.prim_store,
            clip_store: builder.clip_store,
            root_pic_index: builder.root_pic_index,
            config: builder.config,
            content_slice_count: builder.content_slice_count,
            picture_cache_spatial_nodes: builder.picture_cache_spatial_nodes,
        }
    }

    /// Retrieve the current offset to allow converting a stacking context
    /// relative coordinate to be relative to the owing reference frame,
    /// also considering any external scroll offset on the provided
    /// spatial node.
    fn current_offset(
        &mut self,
        spatial_node_index: SpatialNodeIndex,
    ) -> LayoutVector2D {
        // Get the current offset from stacking context <-> reference frame space.
        let rf_offset = self.rf_mapper.current_offset();

        // Get the external scroll offset, if applicable.
        let scroll_offset = self
            .external_scroll_mapper
            .external_scroll_offset(
                spatial_node_index,
                &self.spatial_tree,
            );

        rf_offset + scroll_offset
    }

    /// Figure out the shape of the display list, and wrap various primitive clusters
    /// into tile cache primitive instances.
    fn setup_picture_caching(
        &mut self,
        main_prim_list: &mut PrimitiveList,
    ) {
        if !self.config.global_enable_picture_caching {
            return;
        }

        // Ensure that setup_picture_caching has executed
        debug_assert!(self.picture_caching_initialized);

        // Unconditionally insert a marker to create a picture cache slice on the
        // first cluster. This handles implicit picture caches, and also the common
        // case, by allowing the root / background primitives to be cached in a slice.
        if let Some(cluster) = main_prim_list.clusters.first_mut() {
            cluster.flags.insert(ClusterFlags::CREATE_PICTURE_CACHE_PRE);
        }

        // List of slices that have been found
        let mut slices: Vec<Slice> = Vec::new();
        // Tracker for whether a new slice should be created
        let mut create_slice = true;
        // The clips found the last time we traversed a set of clip chains. Stored and cleared
        // here to avoid constant allocations.
        let mut prim_clips = Vec::new();
        // If true, the cache is out of date and needs to be rebuilt.
        let mut update_shared_clips = true;
        // The last prim clip chain we build prim_clips for.
        let mut last_prim_clip_chain_id = ClipChainId::NONE;

        // Walk the supplied top level of clusters, slicing into slices as appropriate
        for cluster in main_prim_list.clusters.drain(..) {
            // Check if this cluster requires a new slice
            create_slice |= cluster.flags.intersects(
                ClusterFlags::CREATE_PICTURE_CACHE_PRE | ClusterFlags::IS_CLEAR_PRIMITIVE
            );

            if create_slice {
                let slice_flags = if cluster.flags.contains(ClusterFlags::SCROLLBAR_CONTAINER) {
                    SliceFlags::IS_SCROLLBAR
                } else {
                    SliceFlags::empty()
                };
                let slice = Slice {
                    cache_scroll_root: cluster.cache_scroll_root,
                    prim_list: PrimitiveList::empty(),
                    shared_clips: None,
                    flags: slice_flags
                };

                // Open up clip chains on the stack on the new slice
                slices.push(slice);
                create_slice = false;
            }

            // Step through each prim instance, in order to collect shared clips for the slice.
            for instance in &cluster.prim_instances {
                // If the primitive clip chain is different, then we need to rebuild prim_clips.
                update_shared_clips |= last_prim_clip_chain_id != instance.clip_chain_id;
                last_prim_clip_chain_id = instance.clip_chain_id;

                if update_shared_clips {
                    prim_clips.clear();
                    // Update the list of clips that apply to this primitive instance
                    add_clips(
                        instance.clip_chain_id,
                        &mut prim_clips,
                        &self.clip_store,
                        &self.interners,
                    );
                }

                // If there are no shared clips set for this slice, the shared clips are just
                // the current clips set. Otherwise, the shared clips are those that are
                // in both the current shared list and the clips list for this primitive.
                match slices.last_mut().unwrap().shared_clips {
                    Some(ref mut shared_clips) => {
                        if update_shared_clips {
                            shared_clips.retain(|h1: &ClipInstance| {
                                let uid = h1.handle.uid();
                                prim_clips.iter().any(|h2| {
                                    uid == h2.handle.uid() &&
                                    h1.spatial_node_index == h2.spatial_node_index
                                })
                            });
                        }
                    }
                    ref mut shared_clips @ None => {
                        *shared_clips = Some(prim_clips.clone());
                    }
                }

                update_shared_clips = false;
            }

            // If this cluster creates a slice after, then note that for next cluster
            create_slice |= cluster.flags.intersects(
                ClusterFlags::CREATE_PICTURE_CACHE_POST | ClusterFlags::IS_CLEAR_PRIMITIVE
            );

            // Finally, add this cluster to the current slice
            slices.last_mut().unwrap().prim_list.add_cluster(cluster);
        }

        // Step through the slices, creating picture cache wrapper instances.
        for (slice_index, slice) in slices.drain(..).enumerate() {
            let background_color = if slice_index == 0 {
                self.config.background_color
            } else {
                None
            };

            // If the cluster specifies a scroll root, use it. Otherwise,
            // just cache assuming no scrolling takes place. Even if that's
            // not true, we still get caching benefits for any changes that
            // occur while not scrolling (such as animation, video etc);
            let scroll_root = slice.cache_scroll_root.unwrap_or(ROOT_SPATIAL_NODE_INDEX);

            let instance = create_tile_cache(
                slice_index,
                slice.flags,
                scroll_root,
                slice.prim_list,
                background_color,
                slice.shared_clips.unwrap_or_else(Vec::new),
                &mut self.interners,
                &mut self.prim_store,
                &mut self.clip_store,
                &mut self.picture_cache_spatial_nodes,
                &self.config,
            );

            main_prim_list.add_prim(
                instance,
                LayoutRect::zero(),
                scroll_root,
                PrimitiveFlags::IS_BACKFACE_VISIBLE,
            );
        }
    }

    fn build_items(
        &mut self,
        traversal: &mut BuiltDisplayListIter<'a>,
        pipeline_id: PipelineId,
    ) {
        loop {
            let item = match traversal.next() {
                Some(item) => item,
                None => break,
            };

            let subtraversal = match item.item() {
                DisplayItem::PushStackingContext(ref info) => {
                    let space = self.get_space(info.spatial_id);
                    let mut subtraversal = item.sub_iter();
                    self.build_stacking_context(
                        &mut subtraversal,
                        pipeline_id,
                        &info.stacking_context,
                        space,
                        info.origin,
                        item.filters(),
                        &item.filter_datas(),
                        item.filter_primitives(),
                        info.prim_flags,
                    );
                    Some(subtraversal)
                }
                DisplayItem::PushReferenceFrame(ref info) => {
                    let parent_space = self.get_space(info.parent_spatial_id);
                    let mut subtraversal = item.sub_iter();
                    self.build_reference_frame(
                        &mut subtraversal,
                        pipeline_id,
                        parent_space,
                        info.origin,
                        &info.reference_frame,
                    );
                    Some(subtraversal)
                }
                DisplayItem::PopReferenceFrame |
                DisplayItem::PopStackingContext => return,
                _ => None,
            };

            // If build_item created a sub-traversal, we need `traversal` to have the
            // same state as the completed subtraversal, so we reinitialize it here.
            if let Some(mut subtraversal) = subtraversal {
                subtraversal.merge_debug_stats_from(traversal);
                *traversal = subtraversal;
            } else {
                self.build_item(item, pipeline_id);
            }
        }

        // TODO: factor this out to be part of capture
        if cfg!(feature = "display_list_stats") {
            let stats = traversal.debug_stats();
            let total_bytes: usize = stats.iter().map(|(_, stats)| stats.num_bytes).sum();
            println!("item, total count, total bytes, % of DL bytes, bytes per item");
            for (label, stats) in stats {
                println!("{}, {}, {}kb, {}%, {}",
                    label,
                    stats.total_count,
                    stats.num_bytes / 1000,
                    ((stats.num_bytes as f32 / total_bytes.max(1) as f32) * 100.0) as usize,
                    stats.num_bytes / stats.total_count.max(1));
            }
            println!();
        }
    }

    fn build_sticky_frame(
        &mut self,
        info: &StickyFrameDisplayItem,
        parent_node_index: SpatialNodeIndex,
    ) {
        let current_offset = self.current_offset(parent_node_index);
        let frame_rect = info.bounds.translate(current_offset);
        let sticky_frame_info = StickyFrameInfo::new(
            frame_rect,
            info.margins,
            info.vertical_offset_bounds,
            info.horizontal_offset_bounds,
            info.previously_applied_offset,
        );

        let index = self.spatial_tree.add_sticky_frame(
            parent_node_index,
            sticky_frame_info,
            info.id.pipeline_id(),
        );
        self.id_to_index_mapper.add_spatial_node(info.id, index);
    }

    fn build_scroll_frame(
        &mut self,
        info: &ScrollFrameDisplayItem,
        parent_node_index: SpatialNodeIndex,
        pipeline_id: PipelineId,
    ) {
        let current_offset = self.current_offset(parent_node_index);
        let clip_region = ClipRegion::create_for_clip_node_with_local_clip(
            &info.clip_rect,
            &current_offset,
        );
        // Just use clip rectangle as the frame rect for this scroll frame.
        // This is useful when calculating scroll extents for the
        // SpatialNode::scroll(..) API as well as for properly setting sticky
        // positioning offsets.
        let frame_rect = clip_region.main;
        let content_size = info.content_rect.size;

        self.add_clip_node(info.clip_id, &info.parent_space_and_clip, clip_region);

        self.add_scroll_frame(
            info.scroll_frame_id,
            parent_node_index,
            info.external_id,
            pipeline_id,
            &frame_rect,
            &content_size,
            info.scroll_sensitivity,
            ScrollFrameKind::Explicit,
            info.external_scroll_offset,
        );
    }

    fn build_reference_frame(
        &mut self,
        traversal: &mut BuiltDisplayListIter<'a>,
        pipeline_id: PipelineId,
        parent_spatial_node: SpatialNodeIndex,
        origin: LayoutPoint,
        reference_frame: &ReferenceFrame,
    ) {
        profile_scope!("build_reference_frame");
        let current_offset = self.current_offset(parent_spatial_node);
        self.push_reference_frame(
            reference_frame.id,
            Some(parent_spatial_node),
            pipeline_id,
            reference_frame.transform_style,
            reference_frame.transform,
            reference_frame.kind,
            current_offset + origin.to_vector(),
        );

        self.rf_mapper.push_scope();
        self.build_items(
            traversal,
            pipeline_id,
        );
        self.rf_mapper.pop_scope();
    }


    fn build_stacking_context(
        &mut self,
        traversal: &mut BuiltDisplayListIter<'a>,
        pipeline_id: PipelineId,
        stacking_context: &StackingContext,
        spatial_node_index: SpatialNodeIndex,
        origin: LayoutPoint,
        filters: ItemRange<FilterOp>,
        filter_datas: &[TempFilterData],
        filter_primitives: ItemRange<FilterPrimitive>,
        prim_flags: PrimitiveFlags,
    ) {
        profile_scope!("build_stacking_context");
        // Avoid doing unnecessary work for empty stacking contexts.
        if traversal.current_stacking_context_empty() {
            traversal.skip_current_stacking_context();
            return;
        }

        let composition_operations = {
            CompositeOps::new(
                filter_ops_for_compositing(filters),
                filter_datas_for_compositing(filter_datas),
                filter_primitives_for_compositing(filter_primitives),
                stacking_context.mix_blend_mode_for_compositing(),
            )
        };

        self.push_stacking_context(
            pipeline_id,
            composition_operations,
            stacking_context.transform_style,
            prim_flags,
            spatial_node_index,
            stacking_context.clip_id,
            stacking_context.raster_space,
            stacking_context.flags,
            self.sc_stack.last().unwrap().snap_to_device.device_pixel_scale,
        );

        self.rf_mapper.push_offset(origin.to_vector());
        self.build_items(
            traversal,
            pipeline_id,
        );
        self.rf_mapper.pop_offset();

        self.pop_stacking_context();
    }

    fn build_iframe(
        &mut self,
        info: &IframeDisplayItem,
        spatial_node_index: SpatialNodeIndex,
    ) {
        let iframe_pipeline_id = info.pipeline_id;
        let pipeline = match self.scene.pipelines.get(&iframe_pipeline_id) {
            Some(pipeline) => pipeline,
            None => {
                debug_assert!(info.ignore_missing_pipeline);
                return
            },
        };

        let current_offset = self.current_offset(spatial_node_index);
        self.add_clip_node(
            ClipId::root(iframe_pipeline_id),
            &info.space_and_clip,
            ClipRegion::create_for_clip_node_with_local_clip(
                &info.clip_rect,
                &current_offset,
            ),
        );

        self.clip_store.push_clip_root(
            Some(ClipId::root(iframe_pipeline_id)),
            true,
        );

        let snap_to_device = &mut self.sc_stack.last_mut().unwrap().snap_to_device;
        snap_to_device.set_target_spatial_node(
            spatial_node_index,
            &self.spatial_tree,
        );

        let bounds = snap_to_device.snap_rect(
            &info.bounds.translate(current_offset),
        );

        let content_size = snap_to_device.snap_size(&pipeline.content_size);

        let spatial_node_index = self.push_reference_frame(
            SpatialId::root_reference_frame(iframe_pipeline_id),
            Some(spatial_node_index),
            iframe_pipeline_id,
            TransformStyle::Flat,
            PropertyBinding::Value(LayoutTransform::identity()),
            ReferenceFrameKind::Transform,
            bounds.origin.to_vector(),
        );

        let iframe_rect = LayoutRect::new(LayoutPoint::zero(), bounds.size);
        self.add_scroll_frame(
            SpatialId::root_scroll_node(iframe_pipeline_id),
            spatial_node_index,
            Some(ExternalScrollId(0, iframe_pipeline_id)),
            iframe_pipeline_id,
            &iframe_rect,
            &content_size,
            ScrollSensitivity::ScriptAndInputEvents,
            ScrollFrameKind::PipelineRoot,
            LayoutVector2D::zero(),
        );

        self.rf_mapper.push_scope();
        self.iframe_depth += 1;

        self.build_items(
            &mut pipeline.display_list.iter(),
            pipeline.pipeline_id,
        );
        self.iframe_depth -= 1;
        self.rf_mapper.pop_scope();

        self.clip_store.pop_clip_root();
    }

    fn get_space(
        &self,
        spatial_id: SpatialId,
    ) -> SpatialNodeIndex {
        self.id_to_index_mapper.get_spatial_node_index(spatial_id)
    }

    fn get_clip_chain(
        &mut self,
        clip_id: ClipId,
    ) -> ClipChainId {
        self.clip_store.get_or_build_clip_chain_id(clip_id)
    }

    fn process_common_properties(
        &mut self,
        common: &CommonItemProperties,
        bounds: Option<&LayoutRect>,
    ) -> (LayoutPrimitiveInfo, LayoutRect, SpatialNodeIndex, ClipChainId) {
        let spatial_node_index = self.get_space(common.spatial_id);
        let clip_chain_id = self.get_clip_chain(common.clip_id);

        let current_offset = self.current_offset(spatial_node_index);

        let snap_to_device = &mut self.sc_stack.last_mut().unwrap().snap_to_device;
        snap_to_device.set_target_spatial_node(
            spatial_node_index,
            &self.spatial_tree
        );

        let unsnapped_clip_rect = common.clip_rect.translate(current_offset);
        let clip_rect = snap_to_device.snap_rect(&unsnapped_clip_rect);

        let unsnapped_rect = bounds.map(|bounds| {
            bounds.translate(current_offset)
        });

        // If no bounds rect is given, default to clip rect.
        let rect = unsnapped_rect.map_or(clip_rect, |bounds| {
            snap_to_device.snap_rect(&bounds)
        });

        let layout = LayoutPrimitiveInfo {
            rect,
            clip_rect,
            flags: common.flags,
            hit_info: common.hit_info,
        };

        (layout, unsnapped_rect.unwrap_or(unsnapped_clip_rect), spatial_node_index, clip_chain_id)
    }

    fn process_common_properties_with_bounds(
        &mut self,
        common: &CommonItemProperties,
        bounds: &LayoutRect,
    ) -> (LayoutPrimitiveInfo, LayoutRect, SpatialNodeIndex, ClipChainId) {
        self.process_common_properties(
            common,
            Some(bounds),
        )
    }

    pub fn snap_rect(
        &mut self,
        rect: &LayoutRect,
        target_spatial_node: SpatialNodeIndex,
    ) -> LayoutRect {
        let snap_to_device = &mut self.sc_stack.last_mut().unwrap().snap_to_device;
        snap_to_device.set_target_spatial_node(
            target_spatial_node,
            &self.spatial_tree
        );
        snap_to_device.snap_rect(rect)
    }

    fn build_item<'b>(
        &'b mut self,
        item: DisplayItemRef,
        pipeline_id: PipelineId,
    ) {
        match *item.item() {
            DisplayItem::Image(ref info) => {
                let (layout, _, spatial_node_index, clip_chain_id) = self.process_common_properties_with_bounds(
                    &info.common,
                    &info.bounds,
                );

                self.add_image(
                    spatial_node_index,
                    clip_chain_id,
                    &layout,
                    layout.rect.size,
                    LayoutSize::zero(),
                    info.image_key,
                    info.image_rendering,
                    info.alpha_type,
                    info.color,
                );
            }
            DisplayItem::RepeatingImage(ref info) => {
                let (layout, unsnapped_rect, spatial_node_index, clip_chain_id) = self.process_common_properties_with_bounds(
                    &info.common,
                    &info.bounds,
                );

                let stretch_size = process_repeat_size(
                    &layout.rect,
                    &unsnapped_rect,
                    info.stretch_size,
                );

                self.add_image(
                    spatial_node_index,
                    clip_chain_id,
                    &layout,
                    stretch_size,
                    info.tile_spacing,
                    info.image_key,
                    info.image_rendering,
                    info.alpha_type,
                    info.color,
                );
            }
            DisplayItem::YuvImage(ref info) => {
                let (layout, _, spatial_node_index, clip_chain_id) = self.process_common_properties_with_bounds(
                    &info.common,
                    &info.bounds,
                );

                self.add_yuv_image(
                    spatial_node_index,
                    clip_chain_id,
                    &layout,
                    info.yuv_data,
                    info.color_depth,
                    info.color_space,
                    info.color_range,
                    info.image_rendering,
                );
            }
            DisplayItem::Text(ref info) => {
                // TODO(aosmond): Snapping text primitives does not make much sense, given the
                // primitive bounds and clip are supposed to be conservative, not definitive.
                // E.g. they should be able to grow and not impact the output. However there
                // are subtle interactions between the primitive origin and the glyph offset
                // which appear to be significant (presumably due to some sort of accumulated
                // error throughout the layers). We should fix this at some point.
                let (layout, _, spatial_node_index, clip_chain_id) = self.process_common_properties_with_bounds(
                    &info.common,
                    &info.bounds,
                );

                self.add_text(
                    spatial_node_index,
                    clip_chain_id,
                    &layout,
                    &info.font_key,
                    &info.color,
                    item.glyphs(),
                    info.glyph_options,
                );
            }
            DisplayItem::Rectangle(ref info) => {
                let (layout, _, spatial_node_index, clip_chain_id) = self.process_common_properties_with_bounds(
                    &info.common,
                    &info.bounds,
                );

                self.add_solid_rectangle(
                    spatial_node_index,
                    clip_chain_id,
                    &layout,
                    info.color,
                );
            }
            DisplayItem::HitTest(ref info) => {
                let (layout, _, spatial_node_index, clip_chain_id) = self.process_common_properties(
                    &info.common,
                    None,
                );

                self.add_solid_rectangle(
                    spatial_node_index,
                    clip_chain_id,
                    &layout,
                    PropertyBinding::Value(ColorF::TRANSPARENT),
                );
            }
            DisplayItem::ClearRectangle(ref info) => {
                let (layout, _, spatial_node_index, clip_chain_id) = self.process_common_properties_with_bounds(
                    &info.common,
                    &info.bounds,
                );

                self.add_clear_rectangle(
                    spatial_node_index,
                    clip_chain_id,
                    &layout,
                );
            }
            DisplayItem::Line(ref info) => {
                let (layout, _, spatial_node_index, clip_chain_id) = self.process_common_properties_with_bounds(
                    &info.common,
                    &info.area,
                );

                self.add_line(
                    spatial_node_index,
                    clip_chain_id,
                    &layout,
                    info.wavy_line_thickness,
                    info.orientation,
                    info.color,
                    info.style,
                );
            }
            DisplayItem::Gradient(ref info) => {
                let (layout, unsnapped_rect, spatial_node_index, clip_chain_id) = self.process_common_properties_with_bounds(
                    &info.common,
                    &info.bounds,
                );

                let tile_size = process_repeat_size(
                    &layout.rect,
                    &unsnapped_rect,
                    info.tile_size,
                );

                if let Some(prim_key_kind) = self.create_linear_gradient_prim(
                    &layout,
                    info.gradient.start_point,
                    info.gradient.end_point,
                    item.gradient_stops(),
                    info.gradient.extend_mode,
                    tile_size,
                    info.tile_spacing,
                    None,
                ) {
                    self.add_nonshadowable_primitive(
                        spatial_node_index,
                        clip_chain_id,
                        &layout,
                        Vec::new(),
                        prim_key_kind,
                    );
                }
            }
            DisplayItem::RadialGradient(ref info) => {
                let (layout, unsnapped_rect, spatial_node_index, clip_chain_id) = self.process_common_properties_with_bounds(
                    &info.common,
                    &info.bounds,
                );

                let tile_size = process_repeat_size(
                    &layout.rect,
                    &unsnapped_rect,
                    info.tile_size,
                );

                let prim_key_kind = self.create_radial_gradient_prim(
                    &layout,
                    info.gradient.center,
                    info.gradient.start_offset * info.gradient.radius.width,
                    info.gradient.end_offset * info.gradient.radius.width,
                    info.gradient.radius.width / info.gradient.radius.height,
                    item.gradient_stops(),
                    info.gradient.extend_mode,
                    tile_size,
                    info.tile_spacing,
                    None,
                );

                self.add_nonshadowable_primitive(
                    spatial_node_index,
                    clip_chain_id,
                    &layout,
                    Vec::new(),
                    prim_key_kind,
                );
            }
            DisplayItem::ConicGradient(ref info) => {
                let (layout, unsnapped_rect, spatial_node_index, clip_chain_id) = self.process_common_properties_with_bounds(
                    &info.common,
                    &info.bounds,
                );

                let tile_size = process_repeat_size(
                    &layout.rect,
                    &unsnapped_rect,
                    info.tile_size,
                );

                let prim_key_kind = self.create_conic_gradient_prim(
                    &layout,
                    info.gradient.center,
                    info.gradient.angle,
                    info.gradient.start_offset,
                    info.gradient.end_offset,
                    item.gradient_stops(),
                    info.gradient.extend_mode,
                    tile_size,
                    info.tile_spacing,
                    None,
                );

                self.add_nonshadowable_primitive(
                    spatial_node_index,
                    clip_chain_id,
                    &layout,
                    Vec::new(),
                    prim_key_kind,
                );
            }
            DisplayItem::BoxShadow(ref info) => {
                let (layout, _, spatial_node_index, clip_chain_id) = self.process_common_properties_with_bounds(
                    &info.common,
                    &info.box_bounds,
                );

                self.add_box_shadow(
                    spatial_node_index,
                    clip_chain_id,
                    &layout,
                    &info.offset,
                    info.color,
                    info.blur_radius,
                    info.spread_radius,
                    info.border_radius,
                    info.clip_mode,
                );
            }
            DisplayItem::Border(ref info) => {
                let (layout, _, spatial_node_index, clip_chain_id) = self.process_common_properties_with_bounds(
                    &info.common,
                    &info.bounds,
                );

                self.add_border(
                    spatial_node_index,
                    clip_chain_id,
                    &layout,
                    info,
                    item.gradient_stops(),
                );
            }
            DisplayItem::Iframe(ref info) => {
                let space = self.get_space(info.space_and_clip.spatial_id);
                self.build_iframe(
                    info,
                    space,
                );
            }
            DisplayItem::ImageMaskClip(ref info) => {
                let parent_space = self.get_space(info.parent_space_and_clip.spatial_id);
                let current_offset = self.current_offset(parent_space);

                let image_mask = ImageMask {
                    rect: info.image_mask.rect.translate(current_offset),
                    ..info.image_mask
                };

                self.add_image_mask_clip_node(
                    info.id,
                    &info.parent_space_and_clip,
                    &image_mask,
                );
            }
            DisplayItem::RoundedRectClip(ref info) => {
                let parent_space = self.get_space(info.parent_space_and_clip.spatial_id);
                let current_offset = self.current_offset(parent_space);

                self.add_rounded_rect_clip_node(
                    info.id,
                    &info.parent_space_and_clip,
                    &info.clip,
                    current_offset,
                );
            }
            DisplayItem::RectClip(ref info) => {
                let parent_space = self.get_space(info.parent_space_and_clip.spatial_id);
                let current_offset = self.current_offset(parent_space);
                let clip_rect = info.clip_rect.translate(current_offset);

                self.add_rect_clip_node(
                    info.id,
                    &info.parent_space_and_clip,
                    &clip_rect,
                );
            }
            DisplayItem::Clip(ref info) => {
                let parent_space = self.get_space(info.parent_space_and_clip.spatial_id);
                let current_offset = self.current_offset(parent_space);
                let clip_region = ClipRegion::create_for_clip_node(
                    info.clip_rect,
                    item.complex_clip().iter(),
                    &current_offset,
                );
                self.add_clip_node(info.id, &info.parent_space_and_clip, clip_region);
            }
            DisplayItem::ClipChain(ref info) => {
                let parent = info.parent.map_or(ClipId::root(pipeline_id), |id| ClipId::ClipChain(id));
                let mut instances: SmallVec<[ClipInstance; 4]> = SmallVec::new();

                for clip_item in item.clip_chain_items() {
                    let template = self.clip_store.get_template(clip_item);
                    instances.extend_from_slice(&template.instances);
                }

                self.clip_store.register_clip_template(
                    ClipId::ClipChain(info.id),
                    parent,
                    &instances,
                );
            },
            DisplayItem::ScrollFrame(ref info) => {
                let parent_space = self.get_space(info.parent_space_and_clip.spatial_id);
                self.build_scroll_frame(
                    info,
                    parent_space,
                    pipeline_id,
                );
            }
            DisplayItem::StickyFrame(ref info) => {
                let parent_space = self.get_space(info.parent_spatial_id);
                self.build_sticky_frame(
                    info,
                    parent_space,
                );
            }
            DisplayItem::BackdropFilter(ref info) => {
                let (layout, _, spatial_node_index, clip_chain_id) = self.process_common_properties(
                    &info.common,
                    None,
                );

                let filters = filter_ops_for_compositing(item.filters());
                let filter_datas = filter_datas_for_compositing(item.filter_datas());
                let filter_primitives = filter_primitives_for_compositing(item.filter_primitives());

                self.add_backdrop_filter(
                    spatial_node_index,
                    clip_chain_id,
                    &layout,
                    filters,
                    filter_datas,
                    filter_primitives,
                );
            }

            // Do nothing; these are dummy items for the display list parser
            DisplayItem::SetGradientStops |
            DisplayItem::SetFilterOps |
            DisplayItem::SetFilterData |
            DisplayItem::SetFilterPrimitives => {}

            // Special items that are handled in the parent method
            DisplayItem::PushStackingContext(..) |
            DisplayItem::PushReferenceFrame(..) |
            DisplayItem::PopReferenceFrame |
            DisplayItem::PopStackingContext => {
                unreachable!("Should have returned in parent method.")
            }

            DisplayItem::ReuseItems(key) |
            DisplayItem::RetainedItems(key) => {
                unreachable!("Iterator logic error: {:?}", key);
            }

            DisplayItem::PushShadow(info) => {
                let spatial_node_index = self.get_space(info.space_and_clip.spatial_id);
                let clip_chain_id = self.get_clip_chain(
                    info.space_and_clip.clip_id,
                );

                self.push_shadow(
                    info.shadow,
                    spatial_node_index,
                    clip_chain_id,
                    info.should_inflate,
                );
            }
            DisplayItem::PopAllShadows => {
                self.pop_all_shadows();
            }
        }
    }

    // Given a list of clip sources, a positioning node and
    // a parent clip chain, return a new clip chain entry.
    // If the supplied list of clip sources is empty, then
    // just return the parent clip chain id directly.
    fn build_clip_chain(
        &mut self,
        clip_items: Vec<ClipItemKey>,
        spatial_node_index: SpatialNodeIndex,
        parent_clip_chain_id: ClipChainId,
    ) -> ClipChainId {
        if clip_items.is_empty() {
            parent_clip_chain_id
        } else {
            let mut clip_chain_id = parent_clip_chain_id;

            for item in clip_items {
                // Intern this clip item, and store the handle
                // in the clip chain node.
                let handle = self.interners
                    .clip
                    .intern(&item, || {
                        ClipInternData {
                            clip_node_kind: item.kind.node_kind(),
                        }
                    });

                clip_chain_id = self.clip_store.add_clip_chain_node(
                    handle,
                    spatial_node_index,
                    clip_chain_id,
                );
            }

            clip_chain_id
        }
    }

    /// Create a primitive and add it to the prim store. This method doesn't
    /// add the primitive to the draw list, so can be used for creating
    /// sub-primitives.
    ///
    /// TODO(djg): Can this inline into `add_interned_prim_to_draw_list`
    fn create_primitive<P>(
        &mut self,
        info: &LayoutPrimitiveInfo,
        spatial_node_index: SpatialNodeIndex,
        clip_chain_id: ClipChainId,
        prim: P,
    ) -> PrimitiveInstance
    where
        P: InternablePrimitive,
        Interners: AsMut<Interner<P>>,
    {
        // Build a primitive key.
        let prim_key = prim.into_key(info);

        let current_offset = self.current_offset(spatial_node_index);
        let interner = self.interners.as_mut();
        let prim_data_handle = interner
            .intern(&prim_key, || ());

        let instance_kind = P::make_instance_kind(
            prim_key,
            prim_data_handle,
            &mut self.prim_store,
            current_offset,
        );

        PrimitiveInstance::new(
            info.clip_rect,
            instance_kind,
            clip_chain_id,
        )
    }

    pub fn add_primitive_to_hit_testing_list(
        &mut self,
        info: &LayoutPrimitiveInfo,
        spatial_node_index: SpatialNodeIndex,
        clip_chain_id: ClipChainId,
    ) {
        let tag = match info.hit_info {
            Some(tag) => tag,
            None => return,
        };

        // We want to get a range of clip chain roots that apply to this
        // hit testing primitive.

        // Get the start index for the clip chain root range for this primitive.
        let start = self.hit_testing_scene.next_clip_chain_index();

        // Add the clip chain root for the primitive itself.
        self.hit_testing_scene.add_clip_chain(clip_chain_id);

        // Append any clip chain roots from enclosing stacking contexts.
        for sc in &self.sc_stack {
            self.hit_testing_scene.add_clip_chain(sc.clip_chain_id);
        }

        // Construct a clip chain roots range to be stored with the item.
        let clip_chain_range = ops::Range {
            start,
            end: self.hit_testing_scene.next_clip_chain_index(),
        };

        // Create and store the hit testing primitive itself.
        let new_item = HitTestingItem::new(
            tag,
            info,
            spatial_node_index,
            clip_chain_range,
        );
        self.hit_testing_scene.add_item(new_item);
    }

    /// Add an already created primitive to the draw lists.
    pub fn add_primitive_to_draw_list(
        &mut self,
        prim_instance: PrimitiveInstance,
        prim_rect: LayoutRect,
        spatial_node_index: SpatialNodeIndex,
        flags: PrimitiveFlags,
    ) {
        // Add primitive to the top-most stacking context on the stack.
        if prim_instance.is_chased() {
            println!("\tadded to stacking context at {}", self.sc_stack.len());
        }

        let stacking_context = self.sc_stack.last_mut().unwrap();
        stacking_context.prim_list.add_prim(
            prim_instance,
            prim_rect,
            spatial_node_index,
            flags,
        );
    }

    /// Convenience interface that creates a primitive entry and adds it
    /// to the draw list.
    fn add_nonshadowable_primitive<P>(
        &mut self,
        spatial_node_index: SpatialNodeIndex,
        clip_chain_id: ClipChainId,
        info: &LayoutPrimitiveInfo,
        clip_items: Vec<ClipItemKey>,
        prim: P,
    )
    where
        P: InternablePrimitive + IsVisible,
        Interners: AsMut<Interner<P>>,
    {
        if prim.is_visible() {
            let clip_chain_id = self.build_clip_chain(
                clip_items,
                spatial_node_index,
                clip_chain_id,
            );
            self.add_prim_to_draw_list(
                info,
                spatial_node_index,
                clip_chain_id,
                prim,
            );
        }
    }

    pub fn add_primitive<P>(
        &mut self,
        spatial_node_index: SpatialNodeIndex,
        clip_chain_id: ClipChainId,
        info: &LayoutPrimitiveInfo,
        clip_items: Vec<ClipItemKey>,
        prim: P,
    )
    where
        P: InternablePrimitive + IsVisible,
        Interners: AsMut<Interner<P>>,
        ShadowItem: From<PendingPrimitive<P>>
    {
        // If a shadow context is not active, then add the primitive
        // directly to the parent picture.
        if self.pending_shadow_items.is_empty() {
            self.add_nonshadowable_primitive(
                spatial_node_index,
                clip_chain_id,
                info,
                clip_items,
                prim,
            );
        } else {
            debug_assert!(clip_items.is_empty(), "No per-prim clips expected for shadowed primitives");

            // There is an active shadow context. Store as a pending primitive
            // for processing during pop_all_shadows.
            self.pending_shadow_items.push_back(PendingPrimitive {
                spatial_node_index,
                clip_chain_id,
                info: *info,
                prim,
            }.into());
        }
    }

    fn add_prim_to_draw_list<P>(
        &mut self,
        info: &LayoutPrimitiveInfo,
        spatial_node_index: SpatialNodeIndex,
        clip_chain_id: ClipChainId,
        prim: P,
    )
    where
        P: InternablePrimitive,
        Interners: AsMut<Interner<P>>,
    {
        let prim_instance = self.create_primitive(
            info,
            spatial_node_index,
            clip_chain_id,
            prim,
        );
        self.register_chase_primitive_by_rect(
            &info.rect,
            &prim_instance,
        );
        self.add_primitive_to_hit_testing_list(
            info,
            spatial_node_index,
            clip_chain_id,
        );
        self.add_primitive_to_draw_list(
            prim_instance,
            info.rect,
            spatial_node_index,
            info.flags,
        );
    }

    pub fn push_stacking_context(
        &mut self,
        pipeline_id: PipelineId,
        composite_ops: CompositeOps,
        transform_style: TransformStyle,
        prim_flags: PrimitiveFlags,
        spatial_node_index: SpatialNodeIndex,
        clip_id: Option<ClipId>,
        requested_raster_space: RasterSpace,
        flags: StackingContextFlags,
        device_pixel_scale: DevicePixelScale,
    ) {
        // Check if this stacking context is the root of a pipeline, and the caller
        // has requested it as an output frame.
        let is_pipeline_root =
            self.sc_stack.last().map_or(true, |sc| sc.pipeline_id != pipeline_id);
        let frame_output_pipeline_id = if is_pipeline_root && self.output_pipelines.contains(&pipeline_id) {
            Some(pipeline_id)
        } else {
            None
        };

        let clip_chain_id = match clip_id {
            Some(clip_id) => self.clip_store.get_or_build_clip_chain_id(clip_id),
            None => ClipChainId::NONE,
        };

        // Get the transform-style of the parent stacking context,
        // which determines if we *might* need to draw this on
        // an intermediate surface for plane splitting purposes.
        let (parent_is_3d, extra_3d_instance) = match self.sc_stack.last_mut() {
            Some(ref mut sc) if sc.is_3d() => {
                let flat_items_context_3d = match sc.context_3d {
                    Picture3DContext::In { ancestor_index, .. } => Picture3DContext::In {
                        root_data: None,
                        ancestor_index,
                    },
                    Picture3DContext::Out => panic!("Unexpected out of 3D context"),
                };
                // Cut the sequence of flat children before starting a child stacking context,
                // so that the relative order between them and our current SC is preserved.
                let extra_instance = sc.cut_item_sequence(
                    &mut self.prim_store,
                    &mut self.interners,
                    Some(PictureCompositeMode::Blit(BlitReason::PRESERVE3D)),
                    flat_items_context_3d,
                );
                let extra_instance = extra_instance.map(|(_, instance)| {
                    ExtendedPrimitiveInstance {
                        instance,
                        spatial_node_index: sc.spatial_node_index,
                        flags: sc.prim_flags,
                    }
                });
                (true, extra_instance)
            },
            _ => (false, None),
        };

        if let Some(instance) = extra_3d_instance {
            self.add_primitive_instance_to_3d_root(instance);
        }

        // If this is preserve-3d *or* the parent is, then this stacking
        // context is participating in the 3d rendering context. In that
        // case, hoist the picture up to the 3d rendering context
        // container, so that it's rendered as a sibling with other
        // elements in this context.
        let participating_in_3d_context =
            composite_ops.is_empty() &&
            (parent_is_3d || transform_style == TransformStyle::Preserve3D);

        let context_3d = if participating_in_3d_context {
            // Find the spatial node index of the containing block, which
            // defines the context of backface-visibility.
            let ancestor_context = self.sc_stack
                .iter()
                .rfind(|sc| !sc.is_3d());
            Picture3DContext::In {
                root_data: if parent_is_3d {
                    None
                } else {
                    Some(Vec::new())
                },
                ancestor_index: match ancestor_context {
                    Some(sc) => sc.spatial_node_index,
                    None => ROOT_SPATIAL_NODE_INDEX,
                },
            }
        } else {
            Picture3DContext::Out
        };

        // Force an intermediate surface if the stacking context has a
        // complex clip node. In the future, we may decide during
        // prepare step to skip the intermediate surface if the
        // clip node doesn't affect the stacking context rect.
        let mut blit_reason = BlitReason::empty();
        let mut current_clip_chain_id = clip_chain_id;

        if flags.contains(StackingContextFlags::IS_BLEND_CONTAINER) {
            blit_reason |= BlitReason::ISOLATE;
        }

        // Walk each clip in this chain, to see whether any of the clips
        // require that we draw this to an intermediate surface.
        while current_clip_chain_id != ClipChainId::NONE {
            let clip_chain_node = &self
                .clip_store
                .clip_chain_nodes[current_clip_chain_id.0 as usize];

            let clip_node_data = &self.interners.clip[clip_chain_node.handle];

            if let ClipNodeKind::Complex = clip_node_data.clip_node_kind {
                blit_reason = BlitReason::CLIP;
                break;
            }

            current_clip_chain_id = clip_chain_node.parent_clip_chain_id;
        }

        let snap_to_device = self.sc_stack.last().map_or(
            SpaceSnapper::new(
                ROOT_SPATIAL_NODE_INDEX,
                device_pixel_scale,
            ),
            |sc| sc.snap_to_device.clone(),
        );

        let is_redundant = match self.sc_stack.last() {
            Some(parent) => {
                FlattenedStackingContext::is_redundant(
                    &context_3d,
                    &composite_ops,
                    prim_flags,
                    blit_reason,
                    requested_raster_space,
                    parent,
                )
            }
            None => {
                false
            }
        };

        if let Some(clip_id) = clip_id {
            // If this stacking context is redundant (prims will be pushed into
            // the parent during pop) but it has a valid clip, then we need to
            // add that clip to the current clip chain builder, so it's correctly
            // applied to any primitives within this redundant stacking context.
            // For the normal case, we start a new clip root, knowing that the
            // clip on this stacking context will be pushed onto the stack during
            // frame building.
            if is_redundant {
                self.clip_store.push_clip_root(Some(clip_id), true);
            } else {
                self.clip_store.push_clip_root(None, false);
            }
        }

        // Push the SC onto the stack, so we know how to handle things in
        // pop_stacking_context.
        self.sc_stack.push(FlattenedStackingContext {
            prim_list: PrimitiveList::empty(),
            pipeline_id,
            prim_flags,
            requested_raster_space,
            spatial_node_index,
            clip_id,
            clip_chain_id,
            frame_output_pipeline_id,
            composite_ops,
            blit_reason,
            transform_style,
            context_3d,
            is_redundant,
            is_backdrop_root: flags.contains(StackingContextFlags::IS_BACKDROP_ROOT),
            snap_to_device,
        });
    }

    pub fn pop_stacking_context(&mut self) {
        let mut stacking_context = self.sc_stack.pop().unwrap();

        if stacking_context.clip_id.is_some() {
            self.clip_store.pop_clip_root();
        }

        // If we encounter a stacking context that is effectively a no-op, then instead
        // of creating a picture, just append the primitive list to the parent stacking
        // context as a short cut. This serves two purposes:
        // (a) It's an optimization to reduce picture count and allocations, as display lists
        //     often contain a lot of these stacking contexts that don't require pictures or
        //     off-screen surfaces.
        // (b) It's useful for the initial version of picture caching in gecko, by enabling
        //     is to just look for interesting scroll roots on the root stacking context,
        //     without having to consider cuts at stacking context boundaries.
        let parent_is_empty = match self.sc_stack.last_mut() {
            Some(parent_sc) => {
                if stacking_context.is_redundant {
                    if !stacking_context.prim_list.is_empty() {
                        // If popping a redundant stacking context that is from a different pipeline,
                        // we want to insert flags where the picture cache slices should be created
                        // for this iframe. For now, we want to match existing behavior, that is:
                        // - Only cache content that is within the main scroll root, and:
                        // - Skip caching fixed position content before / after the scroll root.
                        // This means that we don't add scrollbars, which cause lots of extra
                        // invalidations. There is ongoing work to add tags to primitives that
                        // are scrollbars. Once this lands, we can simplify this logic considerably
                        // (and add a separate picture cache slice / OS layer for scroll bars).
                        if parent_sc.pipeline_id != stacking_context.pipeline_id && self.iframe_depth == 1 {
                            self.content_slice_count = stacking_context.init_picture_caching(
                                &self.spatial_tree,
                                &self.clip_store,
                                &self.quality_settings,
                            );

                            // Mark that a user supplied tile cache was specified.
                            self.picture_caching_initialized = true;
                        }

                        // If the parent context primitives list is empty, it's faster
                        // to assign the storage of the popped context instead of paying
                        // the copying cost for extend.
                        if parent_sc.prim_list.is_empty() {
                            parent_sc.prim_list = stacking_context.prim_list;
                        } else {
                            parent_sc.prim_list.extend(stacking_context.prim_list);
                        }
                    }

                    return;
                }
                parent_sc.prim_list.is_empty()
            },
            None => true,
        };

        if self.sc_stack.is_empty() {
            // If we didn't encounter a content iframe, then set up picture caching slice markers
            // on the root stacking context. This can happen in Gecko when the parent process
            // provides the content display list (e.g. about:support, about:config etc).
            if !self.picture_caching_initialized {
                self.content_slice_count = stacking_context.init_picture_caching(
                    &self.spatial_tree,
                    &self.clip_store,
                    &self.quality_settings,
                );
                self.picture_caching_initialized = true;
            }

            self.setup_picture_caching(
                &mut stacking_context.prim_list,
            );
        }

        let (leaf_context_3d, leaf_composite_mode, leaf_output_pipeline_id) = match stacking_context.context_3d {
            // TODO(gw): For now, as soon as this picture is in
            //           a 3D context, we draw it to an intermediate
            //           surface and apply plane splitting. However,
            //           there is a large optimization opportunity here.
            //           During culling, we can check if there is actually
            //           perspective present, and skip the plane splitting
            //           completely when that is not the case.
            Picture3DContext::In { ancestor_index, .. } => (
                Picture3DContext::In { root_data: None, ancestor_index },
                Some(PictureCompositeMode::Blit(BlitReason::PRESERVE3D | stacking_context.blit_reason)),
                None,
            ),
            Picture3DContext::Out => (
                Picture3DContext::Out,
                if stacking_context.blit_reason.is_empty() {
                    // By default, this picture will be collapsed into
                    // the owning target.
                    None
                } else {
                    // Add a dummy composite filter if the SC has to be isolated.
                    Some(PictureCompositeMode::Blit(stacking_context.blit_reason))
                },
                stacking_context.frame_output_pipeline_id
            ),
        };

        // Add picture for this actual stacking context contents to render into.
        let leaf_pic_index = PictureIndex(self.prim_store.pictures
            .alloc()
            .init(PicturePrimitive::new_image(
                leaf_composite_mode.clone(),
                leaf_context_3d,
                leaf_output_pipeline_id,
                true,
                stacking_context.prim_flags,
                stacking_context.requested_raster_space,
                stacking_context.prim_list,
                stacking_context.spatial_node_index,
                None,
                PictureOptions::default(),
            ))
        );

        // Create a chain of pictures based on presence of filters,
        // mix-blend-mode and/or 3d rendering context containers.

        let mut current_pic_index = leaf_pic_index;
        let mut cur_instance = create_prim_instance(
            leaf_pic_index,
            leaf_composite_mode.into(),
            ClipChainId::NONE,
            &mut self.interners,
        );

        if cur_instance.is_chased() {
            println!("\tis a leaf primitive for a stacking context");
        }

        // If establishing a 3d context, the `cur_instance` represents
        // a picture with all the *trailing* immediate children elements.
        // We append this to the preserve-3D picture set and make a container picture of them.
        if let Picture3DContext::In { root_data: Some(mut prims), ancestor_index } = stacking_context.context_3d {
            prims.push(ExtendedPrimitiveInstance {
                instance: cur_instance,
                spatial_node_index: stacking_context.spatial_node_index,
                flags: stacking_context.prim_flags,
            });

            let mut prim_list = PrimitiveList::empty();
            for ext_prim in prims.drain(..) {
                prim_list.add_prim(
                    ext_prim.instance,
                    LayoutRect::zero(),
                    ext_prim.spatial_node_index,
                    ext_prim.flags,
                );
            }

            // This is the acttual picture representing our 3D hierarchy root.
            current_pic_index = PictureIndex(self.prim_store.pictures
                .alloc()
                .init(PicturePrimitive::new_image(
                    None,
                    Picture3DContext::In {
                        root_data: Some(Vec::new()),
                        ancestor_index,
                    },
                    stacking_context.frame_output_pipeline_id,
                    true,
                    stacking_context.prim_flags,
                    stacking_context.requested_raster_space,
                    prim_list,
                    stacking_context.spatial_node_index,
                    None,
                    PictureOptions::default(),
                ))
            );

            cur_instance = create_prim_instance(
                current_pic_index,
                PictureCompositeKey::Identity,
                ClipChainId::NONE,
                &mut self.interners,
            );
        }

        let (filtered_pic_index, filtered_instance) = self.wrap_prim_with_filters(
            cur_instance,
            current_pic_index,
            stacking_context.composite_ops.filters,
            stacking_context.composite_ops.filter_primitives,
            stacking_context.composite_ops.filter_datas,
            stacking_context.prim_flags,
            stacking_context.requested_raster_space,
            stacking_context.spatial_node_index,
            true,
        );

        let has_filters = current_pic_index != filtered_pic_index;

        current_pic_index = filtered_pic_index;
        cur_instance = filtered_instance;

        // Same for mix-blend-mode, except we can skip if this primitive is the first in the parent
        // stacking context.
        // From https://drafts.fxtf.org/compositing-1/#generalformula, the formula for blending is:
        // Cs = (1 - ab) x Cs + ab x Blend(Cb, Cs)
        // where
        // Cs = Source color
        // ab = Backdrop alpha
        // Cb = Backdrop color
        //
        // If we're the first primitive within a stacking context, then we can guarantee that the
        // backdrop alpha will be 0, and then the blend equation collapses to just
        // Cs = Cs, and the blend mode isn't taken into account at all.
        if let (Some(mix_blend_mode), false) = (stacking_context.composite_ops.mix_blend_mode, parent_is_empty) {
            if self.sc_stack.last().unwrap().blit_reason.contains(BlitReason::ISOLATE) {
                let composite_mode = Some(PictureCompositeMode::MixBlend(mix_blend_mode));

                let mut prim_list = PrimitiveList::empty();
                prim_list.add_prim(
                    cur_instance.clone(),
                    LayoutRect::zero(),
                    stacking_context.spatial_node_index,
                    stacking_context.prim_flags,
                );

                let blend_pic_index = PictureIndex(self.prim_store.pictures
                    .alloc()
                    .init(PicturePrimitive::new_image(
                        composite_mode.clone(),
                        Picture3DContext::Out,
                        None,
                        true,
                        stacking_context.prim_flags,
                        stacking_context.requested_raster_space,
                        prim_list,
                        stacking_context.spatial_node_index,
                        None,
                        PictureOptions::default(),
                    ))
                );

                current_pic_index = blend_pic_index;
                cur_instance = create_prim_instance(
                    blend_pic_index,
                    composite_mode.into(),
                    ClipChainId::NONE,
                    &mut self.interners,
                );

                if cur_instance.is_chased() {
                    println!("\tis a mix-blend picture for a stacking context with {:?}", mix_blend_mode);
                }
            } else {
                // If we have a mix-blend-mode, the stacking context needs to be isolated
                // to blend correctly as per the CSS spec.
                // If not already isolated, we can't correctly blend.
                warn!("found a mix-blend-mode outside a blend container, ignoring");
            }
        }

        // Set the stacking context clip on the outermost picture in the chain,
        // unless we already set it on the leaf picture.
        cur_instance.clip_chain_id = stacking_context.clip_chain_id;

        // The primitive instance for the remainder of flat children of this SC
        // if it's a part of 3D hierarchy but not the root of it.
        let trailing_children_instance = match self.sc_stack.last_mut() {
            // Preserve3D path (only relevant if there are no filters/mix-blend modes)
            Some(ref parent_sc) if !has_filters && parent_sc.is_3d() => {
                Some(cur_instance)
            }
            // Regular parenting path
            Some(ref mut parent_sc) => {
                parent_sc.prim_list.add_prim(
                    cur_instance,
                    LayoutRect::zero(),
                    stacking_context.spatial_node_index,
                    stacking_context.prim_flags,
                );
                None
            }
            // This must be the root stacking context
            None => {
                self.root_pic_index = current_pic_index;
                None
            }
        };

        // finally, if there any outstanding 3D primitive instances,
        // find the 3D hierarchy root and add them there.
        if let Some(instance) = trailing_children_instance {
            self.add_primitive_instance_to_3d_root(ExtendedPrimitiveInstance {
                instance,
                spatial_node_index: stacking_context.spatial_node_index,
                flags: stacking_context.prim_flags,
            });
        }

        assert!(
            self.pending_shadow_items.is_empty(),
            "Found unpopped shadows when popping stacking context!"
        );
    }

    pub fn push_reference_frame(
        &mut self,
        reference_frame_id: SpatialId,
        parent_index: Option<SpatialNodeIndex>,
        pipeline_id: PipelineId,
        transform_style: TransformStyle,
        source_transform: PropertyBinding<LayoutTransform>,
        kind: ReferenceFrameKind,
        origin_in_parent_reference_frame: LayoutVector2D,
    ) -> SpatialNodeIndex {
        let index = self.spatial_tree.add_reference_frame(
            parent_index,
            transform_style,
            source_transform,
            kind,
            origin_in_parent_reference_frame,
            pipeline_id,
        );
        self.id_to_index_mapper.add_spatial_node(reference_frame_id, index);

        index
    }

    pub fn push_root(
        &mut self,
        pipeline_id: PipelineId,
        viewport_size: &LayoutSize,
        content_size: &LayoutSize,
        device_pixel_scale: DevicePixelScale,
    ) {
        if let ChasePrimitive::Id(id) = self.config.chase_primitive {
            println!("Chasing {:?} by index", id);
            register_prim_chase_id(id);
        }

        let spatial_node_index = self.push_reference_frame(
            SpatialId::root_reference_frame(pipeline_id),
            None,
            pipeline_id,
            TransformStyle::Flat,
            PropertyBinding::Value(LayoutTransform::identity()),
            ReferenceFrameKind::Transform,
            LayoutVector2D::zero(),
        );

        // We can't use this with the stacking context because it does not exist
        // yet. Just create a dedicated snapper for the root.
        let snap_to_device = SpaceSnapper::new_with_target(
            spatial_node_index,
            ROOT_SPATIAL_NODE_INDEX,
            device_pixel_scale,
            &self.spatial_tree,
        );

        let content_size = snap_to_device.snap_size(content_size);
        let viewport_rect = snap_to_device.snap_rect(
            &LayoutRect::new(LayoutPoint::zero(), *viewport_size),
        );

        self.add_scroll_frame(
            SpatialId::root_scroll_node(pipeline_id),
            spatial_node_index,
            Some(ExternalScrollId(0, pipeline_id)),
            pipeline_id,
            &viewport_rect,
            &content_size,
            ScrollSensitivity::ScriptAndInputEvents,
            ScrollFrameKind::PipelineRoot,
            LayoutVector2D::zero(),
        );
    }

    fn add_image_mask_clip_node(
        &mut self,
        new_node_id: ClipId,
        space_and_clip: &SpaceAndClipInfo,
        image_mask: &ImageMask,
    ) {
        let spatial_node_index = self.id_to_index_mapper.get_spatial_node_index(space_and_clip.spatial_id);

        let snap_to_device = &mut self.sc_stack.last_mut().unwrap().snap_to_device;
        snap_to_device.set_target_spatial_node(
            spatial_node_index,
            &self.spatial_tree,
        );

        let snapped_mask_rect = snap_to_device.snap_rect(&image_mask.rect);
        let item = ClipItemKey {
            kind: ClipItemKeyKind::image_mask(image_mask, snapped_mask_rect),
        };

        let handle = self
            .interners
            .clip
            .intern(&item, || {
                ClipInternData {
                    clip_node_kind: ClipNodeKind::Complex,
                }
            });

        let instance = ClipInstance::new(handle, spatial_node_index);

        self.clip_store.register_clip_template(
            new_node_id,
            space_and_clip.clip_id,
            &[instance],
        );
    }

    /// Add a new rectangle clip, positioned by the spatial node in the `space_and_clip`.
    pub fn add_rect_clip_node(
        &mut self,
        new_node_id: ClipId,
        space_and_clip: &SpaceAndClipInfo,
        clip_rect: &LayoutRect,
    ) {
        let spatial_node_index = self.id_to_index_mapper.get_spatial_node_index(space_and_clip.spatial_id);

        let snap_to_device = &mut self.sc_stack.last_mut().unwrap().snap_to_device;
        snap_to_device.set_target_spatial_node(
            spatial_node_index,
            &self.spatial_tree,
        );

        let snapped_clip_rect = snap_to_device.snap_rect(clip_rect);

        let item = ClipItemKey {
            kind: ClipItemKeyKind::rectangle(snapped_clip_rect, ClipMode::Clip),
        };
        let handle = self
            .interners
            .clip
            .intern(&item, || {
                ClipInternData {
                    clip_node_kind: ClipNodeKind::Rectangle,
                }
            });

        let instance = ClipInstance::new(handle, spatial_node_index);

        self.clip_store.register_clip_template(
            new_node_id,
            space_and_clip.clip_id,
            &[instance],
        );
    }

    pub fn add_rounded_rect_clip_node(
        &mut self,
        new_node_id: ClipId,
        space_and_clip: &SpaceAndClipInfo,
        clip: &ComplexClipRegion,
        current_offset: LayoutVector2D,
    ) {
        let spatial_node_index = self.id_to_index_mapper.get_spatial_node_index(space_and_clip.spatial_id);

        let snap_to_device = &mut self.sc_stack.last_mut().unwrap().snap_to_device;
        snap_to_device.set_target_spatial_node(
            spatial_node_index,
            &self.spatial_tree,
        );

        let snapped_region_rect = snap_to_device.snap_rect(&clip.rect.translate(current_offset));
        let item = ClipItemKey {
            kind: ClipItemKeyKind::rounded_rect(
                snapped_region_rect,
                clip.radii,
                clip.mode,
            ),
        };

        let handle = self
            .interners
            .clip
            .intern(&item, || {
                ClipInternData {
                    clip_node_kind: ClipNodeKind::Complex,
                }
            });

        let instance = ClipInstance::new(handle, spatial_node_index);

        self.clip_store.register_clip_template(
            new_node_id,
            space_and_clip.clip_id,
            &[instance],
        );
    }

    pub fn add_clip_node<I>(
        &mut self,
        new_node_id: ClipId,
        space_and_clip: &SpaceAndClipInfo,
        clip_region: ClipRegion<I>,
    )
    where
        I: IntoIterator<Item = ComplexClipRegion>
    {
        // Map the ClipId for the positioning node to a spatial node index.
        let spatial_node_index = self.id_to_index_mapper.get_spatial_node_index(space_and_clip.spatial_id);

        let snap_to_device = &mut self.sc_stack.last_mut().unwrap().snap_to_device;
        snap_to_device.set_target_spatial_node(
            spatial_node_index,
            &self.spatial_tree,
        );

        let snapped_clip_rect = snap_to_device.snap_rect(&clip_region.main);
        let mut instances: SmallVec<[ClipInstance; 4]> = SmallVec::new();

        // Intern each clip item in this clip node, and add the interned
        // handle to a clip chain node, parented to form a chain.
        // TODO(gw): We could re-structure this to share some of the
        //           interning and chaining code.

        // Build the clip sources from the supplied region.
        let item = ClipItemKey {
            kind: ClipItemKeyKind::rectangle(snapped_clip_rect, ClipMode::Clip),
        };
        let handle = self
            .interners
            .clip
            .intern(&item, || {
                ClipInternData {
                    clip_node_kind: ClipNodeKind::Rectangle,
                }
            });
        instances.push(ClipInstance::new(handle, spatial_node_index));

        for region in clip_region.complex_clips {
            let snapped_region_rect = snap_to_device.snap_rect(&region.rect);
            let item = ClipItemKey {
                kind: ClipItemKeyKind::rounded_rect(
                    snapped_region_rect,
                    region.radii,
                    region.mode,
                ),
            };

            let handle = self
                .interners
                .clip
                .intern(&item, || {
                    ClipInternData {
                        clip_node_kind: ClipNodeKind::Complex,
                    }
                });

            instances.push(ClipInstance::new(handle, spatial_node_index));
        }

        self.clip_store.register_clip_template(
            new_node_id,
            space_and_clip.clip_id,
            &instances,
        );
    }

    pub fn add_scroll_frame(
        &mut self,
        new_node_id: SpatialId,
        parent_node_index: SpatialNodeIndex,
        external_id: Option<ExternalScrollId>,
        pipeline_id: PipelineId,
        frame_rect: &LayoutRect,
        content_size: &LayoutSize,
        scroll_sensitivity: ScrollSensitivity,
        frame_kind: ScrollFrameKind,
        external_scroll_offset: LayoutVector2D,
    ) -> SpatialNodeIndex {
        let node_index = self.spatial_tree.add_scroll_frame(
            parent_node_index,
            external_id,
            pipeline_id,
            frame_rect,
            content_size,
            scroll_sensitivity,
            frame_kind,
            external_scroll_offset,
        );
        self.id_to_index_mapper.add_spatial_node(new_node_id, node_index);
        node_index
    }

    pub fn push_shadow(
        &mut self,
        shadow: Shadow,
        spatial_node_index: SpatialNodeIndex,
        clip_chain_id: ClipChainId,
        should_inflate: bool,
    ) {
        // Store this shadow in the pending list, for processing
        // during pop_all_shadows.
        self.pending_shadow_items.push_back(ShadowItem::Shadow(PendingShadow {
            shadow,
            spatial_node_index,
            clip_chain_id,
            should_inflate,
        }));
    }

    pub fn pop_all_shadows(
        &mut self,
    ) {
        assert!(!self.pending_shadow_items.is_empty(), "popped shadows, but none were present");

        let mut items = mem::replace(&mut self.pending_shadow_items, VecDeque::new());

        //
        // The pending_shadow_items queue contains a list of shadows and primitives
        // that were pushed during the active shadow context. To process these, we:
        //
        // Iterate the list, popping an item from the front each iteration.
        //
        // If the item is a shadow:
        //      - Create a shadow picture primitive.
        //      - Add *any* primitives that remain in the item list to this shadow.
        // If the item is a primitive:
        //      - Add that primitive as a normal item (if alpha > 0)
        //

        while let Some(item) = items.pop_front() {
            match item {
                ShadowItem::Shadow(pending_shadow) => {
                    // Quote from https://drafts.csswg.org/css-backgrounds-3/#shadow-blur
                    // "the image that would be generated by applying to the shadow a
                    // Gaussian blur with a standard deviation equal to half the blur radius."
                    let std_deviation = pending_shadow.shadow.blur_radius * 0.5;

                    // If the shadow has no blur, any elements will get directly rendered
                    // into the parent picture surface, instead of allocating and drawing
                    // into an intermediate surface. In this case, we will need to apply
                    // the local clip rect to primitives.
                    let is_passthrough = pending_shadow.shadow.blur_radius == 0.0;

                    // shadows always rasterize in local space.
                    // TODO(gw): expose API for clients to specify a raster scale
                    let raster_space = if is_passthrough {
                        self.sc_stack.last().unwrap().requested_raster_space
                    } else {
                        RasterSpace::Local(1.0)
                    };

                    // Add any primitives that come after this shadow in the item
                    // list to this shadow.
                    let mut prim_list = PrimitiveList::empty();

                    for item in &items {
                        match item {
                            ShadowItem::Image(ref pending_image) => {
                                self.add_shadow_prim(
                                    &pending_shadow,
                                    pending_image,
                                    &mut prim_list,
                                )
                            }
                            ShadowItem::LineDecoration(ref pending_line_dec) => {
                                self.add_shadow_prim(
                                    &pending_shadow,
                                    pending_line_dec,
                                    &mut prim_list,
                                )
                            }
                            ShadowItem::NormalBorder(ref pending_border) => {
                                self.add_shadow_prim(
                                    &pending_shadow,
                                    pending_border,
                                    &mut prim_list,
                                )
                            }
                            ShadowItem::Primitive(ref pending_primitive) => {
                                self.add_shadow_prim(
                                    &pending_shadow,
                                    pending_primitive,
                                    &mut prim_list,
                                )
                            }
                            ShadowItem::TextRun(ref pending_text_run) => {
                                self.add_shadow_prim(
                                    &pending_shadow,
                                    pending_text_run,
                                    &mut prim_list,
                                )
                            }
                            _ => {}
                        }
                    }

                    // No point in adding a shadow here if there were no primitives
                    // added to the shadow.
                    if !prim_list.is_empty() {
                        // Create a picture that the shadow primitives will be added to. If the
                        // blur radius is 0, the code in Picture::prepare_for_render will
                        // detect this and mark the picture to be drawn directly into the
                        // parent picture, which avoids an intermediate surface and blur.
                        let blur_filter = Filter::Blur(std_deviation);
                        let composite_mode = PictureCompositeMode::Filter(blur_filter);
                        let composite_mode_key = Some(composite_mode.clone()).into();

                        // Pass through configuration information about whether WR should
                        // do the bounding rect inflation for text shadows.
                        let options = PictureOptions {
                            inflate_if_required: pending_shadow.should_inflate,
                        };

                        // Create the primitive to draw the shadow picture into the scene.
                        let shadow_pic_index = PictureIndex(self.prim_store.pictures
                            .alloc()
                            .init(PicturePrimitive::new_image(
                                Some(composite_mode),
                                Picture3DContext::Out,
                                None,
                                is_passthrough,
                                PrimitiveFlags::IS_BACKFACE_VISIBLE,
                                raster_space,
                                prim_list,
                                pending_shadow.spatial_node_index,
                                None,
                                options,
                            ))
                        );

                        let shadow_pic_key = PictureKey::new(
                            Picture { composite_mode_key },
                        );

                        let shadow_prim_data_handle = self.interners
                            .picture
                            .intern(&shadow_pic_key, || ());

                        let shadow_prim_instance = PrimitiveInstance::new(
                            LayoutRect::max_rect(),
                            PrimitiveInstanceKind::Picture {
                                data_handle: shadow_prim_data_handle,
                                pic_index: shadow_pic_index,
                                segment_instance_index: SegmentInstanceIndex::INVALID,
                            },
                            pending_shadow.clip_chain_id,
                        );

                        // Add the shadow primitive. This must be done before pushing this
                        // picture on to the shadow stack, to avoid infinite recursion!
                        self.add_primitive_to_draw_list(
                            shadow_prim_instance,
                            LayoutRect::zero(),
                            pending_shadow.spatial_node_index,
                            PrimitiveFlags::IS_BACKFACE_VISIBLE,
                        );
                    }
                }
                ShadowItem::Image(pending_image) => {
                    self.add_shadow_prim_to_draw_list(
                        pending_image,
                    )
                },
                ShadowItem::LineDecoration(pending_line_dec) => {
                    self.add_shadow_prim_to_draw_list(
                        pending_line_dec,
                    )
                },
                ShadowItem::NormalBorder(pending_border) => {
                    self.add_shadow_prim_to_draw_list(
                        pending_border,
                    )
                },
                ShadowItem::Primitive(pending_primitive) => {
                    self.add_shadow_prim_to_draw_list(
                        pending_primitive,
                    )
                },
                ShadowItem::TextRun(pending_text_run) => {
                    self.add_shadow_prim_to_draw_list(
                        pending_text_run,
                    )
                },
            }
        }

        debug_assert!(items.is_empty());
        self.pending_shadow_items = items;
    }

    fn add_shadow_prim<P>(
        &mut self,
        pending_shadow: &PendingShadow,
        pending_primitive: &PendingPrimitive<P>,
        prim_list: &mut PrimitiveList,
    )
    where
        P: InternablePrimitive + CreateShadow,
        Interners: AsMut<Interner<P>>,
    {
        let snap_to_device = &mut self.sc_stack.last_mut().unwrap().snap_to_device;
        snap_to_device.set_target_spatial_node(
            pending_primitive.spatial_node_index,
            &self.spatial_tree,
        );

        // Offset the local rect and clip rect by the shadow offset. The pending
        // primitive has already been snapped, but we will need to snap the
        // shadow after translation. We don't need to worry about the size
        // changing because the shadow has the same raster space as the
        // primitive, and thus we know the size is already rounded.
        let mut info = pending_primitive.info.clone();
        info.rect = snap_to_device.snap_rect(
            &info.rect.translate(pending_shadow.shadow.offset),
        );
        info.clip_rect = snap_to_device.snap_rect(
            &info.clip_rect.translate(pending_shadow.shadow.offset),
        );

        // Construct and add a primitive for the given shadow.
        let shadow_prim_instance = self.create_primitive(
            &info,
            pending_primitive.spatial_node_index,
            pending_primitive.clip_chain_id,
            pending_primitive.prim.create_shadow(&pending_shadow.shadow),
        );

        // Add the new primitive to the shadow picture.
        prim_list.add_prim(
            shadow_prim_instance,
            info.rect,
            pending_primitive.spatial_node_index,
            info.flags,
        );
    }

    fn add_shadow_prim_to_draw_list<P>(
        &mut self,
        pending_primitive: PendingPrimitive<P>,
    ) where
        P: InternablePrimitive + IsVisible,
        Interners: AsMut<Interner<P>>,
    {
        // For a normal primitive, if it has alpha > 0, then we add this
        // as a normal primitive to the parent picture.
        if pending_primitive.prim.is_visible() {
            self.add_prim_to_draw_list(
                &pending_primitive.info,
                pending_primitive.spatial_node_index,
                pending_primitive.clip_chain_id,
                pending_primitive.prim,
            );
        }
    }

    #[cfg(debug_assertions)]
    fn register_chase_primitive_by_rect(
        &mut self,
        rect: &LayoutRect,
        prim_instance: &PrimitiveInstance,
    ) {
        if ChasePrimitive::LocalRect(*rect) == self.config.chase_primitive {
            println!("Chasing {:?} by local rect", prim_instance.id);
            register_prim_chase_id(prim_instance.id);
        }
    }

    #[cfg(not(debug_assertions))]
    fn register_chase_primitive_by_rect(
        &mut self,
        _rect: &LayoutRect,
        _prim_instance: &PrimitiveInstance,
    ) {
    }

    pub fn add_solid_rectangle(
        &mut self,
        spatial_node_index: SpatialNodeIndex,
        clip_chain_id: ClipChainId,
        info: &LayoutPrimitiveInfo,
        color: PropertyBinding<ColorF>,
    ) {
        match color {
            PropertyBinding::Value(value) => {
                if value.a == 0.0 {
                    // Don't add transparent rectangles to the draw list,
                    // but do consider them for hit testing. This allows
                    // specifying invisible hit testing areas.
                    self.add_primitive_to_hit_testing_list(
                        info,
                        spatial_node_index,
                        clip_chain_id,
                    );
                    return;
                }
            },
            PropertyBinding::Binding(..) => {},
        }

        self.add_primitive(
            spatial_node_index,
            clip_chain_id,
            info,
            Vec::new(),
            PrimitiveKeyKind::Rectangle {
                color: color.into(),
            },
        );
    }

    pub fn add_clear_rectangle(
        &mut self,
        spatial_node_index: SpatialNodeIndex,
        clip_chain_id: ClipChainId,
        info: &LayoutPrimitiveInfo,
    ) {
        self.add_primitive(
            spatial_node_index,
            clip_chain_id,
            info,
            Vec::new(),
            PrimitiveKeyKind::Clear,
        );
    }

    pub fn add_line(
        &mut self,
        spatial_node_index: SpatialNodeIndex,
        clip_chain_id: ClipChainId,
        info: &LayoutPrimitiveInfo,
        wavy_line_thickness: f32,
        orientation: LineOrientation,
        color: ColorF,
        style: LineStyle,
    ) {
        // For line decorations, we can construct the render task cache key
        // here during scene building, since it doesn't depend on device
        // pixel ratio or transform.
        let mut info = info.clone();

        let size = get_line_decoration_size(
            &info.rect.size,
            orientation,
            style,
            wavy_line_thickness,
        );

        let cache_key = size.map(|size| {
            // If dotted, adjust the clip rect to ensure we don't draw a final
            // partial dot.
            if style == LineStyle::Dotted {
                let clip_size = match orientation {
                    LineOrientation::Horizontal => {
                        LayoutSize::new(
                            size.width * (info.rect.size.width / size.width).floor(),
                            info.rect.size.height,
                        )
                    }
                    LineOrientation::Vertical => {
                        LayoutSize::new(
                            info.rect.size.width,
                            size.height * (info.rect.size.height / size.height).floor(),
                        )
                    }
                };
                let clip_rect = LayoutRect::new(
                    info.rect.origin,
                    clip_size,
                );
                info.clip_rect = clip_rect
                    .intersection(&info.clip_rect)
                    .unwrap_or_else(LayoutRect::zero);
            }

            LineDecorationCacheKey {
                style,
                orientation,
                wavy_line_thickness: Au::from_f32_px(wavy_line_thickness),
                size: size.to_au(),
            }
        });

        self.add_primitive(
            spatial_node_index,
            clip_chain_id,
            &info,
            Vec::new(),
            LineDecoration {
                cache_key,
                color: color.into(),
            },
        );
    }

    pub fn add_border(
        &mut self,
        spatial_node_index: SpatialNodeIndex,
        clip_chain_id: ClipChainId,
        info: &LayoutPrimitiveInfo,
        border_item: &BorderDisplayItem,
        gradient_stops: ItemRange<GradientStop>,
    ) {
        match border_item.details {
            BorderDetails::NinePatch(ref border) => {
                let nine_patch = NinePatchDescriptor {
                    width: border.width,
                    height: border.height,
                    slice: border.slice,
                    fill: border.fill,
                    repeat_horizontal: border.repeat_horizontal,
                    repeat_vertical: border.repeat_vertical,
                    outset: border.outset.into(),
                    widths: border_item.widths.into(),
                };

                match border.source {
                    NinePatchBorderSource::Image(image_key) => {
                        let prim = ImageBorder {
                            request: ImageRequest {
                                key: image_key,
                                rendering: ImageRendering::Auto,
                                tile: None,
                            },
                            nine_patch,
                        };

                        self.add_nonshadowable_primitive(
                            spatial_node_index,
                            clip_chain_id,
                            info,
                            Vec::new(),
                            prim,
                        );
                    }
                    NinePatchBorderSource::Gradient(gradient) => {
                        let prim = match self.create_linear_gradient_prim(
                            &info,
                            gradient.start_point,
                            gradient.end_point,
                            gradient_stops,
                            gradient.extend_mode,
                            LayoutSize::new(border.height as f32, border.width as f32),
                            LayoutSize::zero(),
                            Some(Box::new(nine_patch)),
                        ) {
                            Some(prim) => prim,
                            None => return,
                        };

                        self.add_nonshadowable_primitive(
                            spatial_node_index,
                            clip_chain_id,
                            info,
                            Vec::new(),
                            prim,
                        );
                    }
                    NinePatchBorderSource::RadialGradient(gradient) => {
                        let prim = self.create_radial_gradient_prim(
                            &info,
                            gradient.center,
                            gradient.start_offset * gradient.radius.width,
                            gradient.end_offset * gradient.radius.width,
                            gradient.radius.width / gradient.radius.height,
                            gradient_stops,
                            gradient.extend_mode,
                            LayoutSize::new(border.height as f32, border.width as f32),
                            LayoutSize::zero(),
                            Some(Box::new(nine_patch)),
                        );

                        self.add_nonshadowable_primitive(
                            spatial_node_index,
                            clip_chain_id,
                            info,
                            Vec::new(),
                            prim,
                        );
                    }
                    NinePatchBorderSource::ConicGradient(gradient) => {
                        let prim = self.create_conic_gradient_prim(
                            &info,
                            gradient.center,
                            gradient.angle,
                            gradient.start_offset,
                            gradient.end_offset,
                            gradient_stops,
                            gradient.extend_mode,
                            LayoutSize::new(border.height as f32, border.width as f32),
                            LayoutSize::zero(),
                            Some(Box::new(nine_patch)),
                        );

                        self.add_nonshadowable_primitive(
                            spatial_node_index,
                            clip_chain_id,
                            info,
                            Vec::new(),
                            prim,
                        );
                    }
                };
            }
            BorderDetails::Normal(ref border) => {
                self.add_normal_border(
                    info,
                    border,
                    border_item.widths,
                    spatial_node_index,
                    clip_chain_id,
                );
            }
        }
    }

    pub fn create_linear_gradient_prim(
        &mut self,
        info: &LayoutPrimitiveInfo,
        start_point: LayoutPoint,
        end_point: LayoutPoint,
        stops: ItemRange<GradientStop>,
        extend_mode: ExtendMode,
        stretch_size: LayoutSize,
        mut tile_spacing: LayoutSize,
        nine_patch: Option<Box<NinePatchDescriptor>>,
    ) -> Option<LinearGradient> {
        let mut prim_rect = info.rect;
        simplify_repeated_primitive(&stretch_size, &mut tile_spacing, &mut prim_rect);

        let mut max_alpha: f32 = 0.0;

        let stops = stops.iter().map(|stop| {
            max_alpha = max_alpha.max(stop.color.a);
            GradientStopKey {
                offset: stop.offset,
                color: stop.color.into(),
            }
        }).collect();

        // If all the stops have no alpha, then this
        // gradient can't contribute to the scene.
        if max_alpha <= 0.0 {
            return None;
        }

        // Try to ensure that if the gradient is specified in reverse, then so long as the stops
        // are also supplied in reverse that the rendered result will be equivalent. To do this,
        // a reference orientation for the gradient line must be chosen, somewhat arbitrarily, so
        // just designate the reference orientation as start < end. Aligned gradient rendering
        // manages to produce the same result regardless of orientation, so don't worry about
        // reversing in that case.
        let reverse_stops = start_point.x > end_point.x ||
            (start_point.x == end_point.x && start_point.y > end_point.y);

        // To get reftests exactly matching with reverse start/end
        // points, it's necessary to reverse the gradient
        // line in some cases.
        let (sp, ep) = if reverse_stops {
            (end_point, start_point)
        } else {
            (start_point, end_point)
        };

        Some(LinearGradient {
            extend_mode,
            start_point: sp.into(),
            end_point: ep.into(),
            stretch_size: stretch_size.into(),
            tile_spacing: tile_spacing.into(),
            stops,
            reverse_stops,
            nine_patch,
        })
    }

    pub fn create_radial_gradient_prim(
        &mut self,
        info: &LayoutPrimitiveInfo,
        center: LayoutPoint,
        start_radius: f32,
        end_radius: f32,
        ratio_xy: f32,
        stops: ItemRange<GradientStop>,
        extend_mode: ExtendMode,
        stretch_size: LayoutSize,
        mut tile_spacing: LayoutSize,
        nine_patch: Option<Box<NinePatchDescriptor>>,
    ) -> RadialGradient {
        let mut prim_rect = info.rect;
        simplify_repeated_primitive(&stretch_size, &mut tile_spacing, &mut prim_rect);

        let params = RadialGradientParams {
            start_radius,
            end_radius,
            ratio_xy,
        };

        let stops = stops.iter().map(|stop| {
            GradientStopKey {
                offset: stop.offset,
                color: stop.color.into(),
            }
        }).collect();

        RadialGradient {
            extend_mode,
            center: center.into(),
            params,
            stretch_size: stretch_size.into(),
            tile_spacing: tile_spacing.into(),
            nine_patch,
            stops,
        }
    }

    pub fn create_conic_gradient_prim(
        &mut self,
        info: &LayoutPrimitiveInfo,
        center: LayoutPoint,
        angle: f32,
        start_offset: f32,
        end_offset: f32,
        stops: ItemRange<GradientStop>,
        extend_mode: ExtendMode,
        stretch_size: LayoutSize,
        mut tile_spacing: LayoutSize,
        nine_patch: Option<Box<NinePatchDescriptor>>,
    ) -> ConicGradient {
        let mut prim_rect = info.rect;
        simplify_repeated_primitive(&stretch_size, &mut tile_spacing, &mut prim_rect);

        let stops = stops.iter().map(|stop| {
            GradientStopKey {
                offset: stop.offset,
                color: stop.color.into(),
            }
        }).collect();

        ConicGradient {
            extend_mode,
            center: center.into(),
            params: ConicGradientParams { angle, start_offset, end_offset },
            stretch_size: stretch_size.into(),
            tile_spacing: tile_spacing.into(),
            nine_patch,
            stops,
        }
    }

    pub fn add_text(
        &mut self,
        spatial_node_index: SpatialNodeIndex,
        clip_chain_id: ClipChainId,
        prim_info: &LayoutPrimitiveInfo,
        font_instance_key: &FontInstanceKey,
        text_color: &ColorF,
        glyph_range: ItemRange<GlyphInstance>,
        glyph_options: Option<GlyphOptions>,
    ) {
        let offset = self.current_offset(spatial_node_index);

        let text_run = {
            let instance_map = self.font_instances.lock().unwrap();
            let font_instance = match instance_map.get(font_instance_key) {
                Some(instance) => instance,
                None => {
                    warn!("Unknown font instance key");
                    debug!("key={:?}", font_instance_key);
                    return;
                }
            };

            // Trivial early out checks
            if font_instance.size <= FontSize::zero() {
                return;
            }

            // TODO(gw): Use a proper algorithm to select
            // whether this item should be rendered with
            // subpixel AA!
            let mut render_mode = self.config
                .default_font_render_mode
                .limit_by(font_instance.render_mode);
            let mut flags = font_instance.flags;
            if let Some(options) = glyph_options {
                render_mode = render_mode.limit_by(options.render_mode);
                flags |= options.flags;
            }

            let font = FontInstance::new(
                Arc::clone(font_instance),
                (*text_color).into(),
                render_mode,
                flags,
            );

            // TODO(gw): It'd be nice not to have to allocate here for creating
            //           the primitive key, when the common case is that the
            //           hash will match and we won't end up creating a new
            //           primitive template.
            let prim_offset = prim_info.rect.origin.to_vector() - offset;
            let glyphs = glyph_range
                .iter()
                .map(|glyph| {
                    GlyphInstance {
                        index: glyph.index,
                        point: glyph.point - prim_offset,
                    }
                })
                .collect();

            TextRun {
                glyphs: Arc::new(glyphs),
                font,
                shadow: false,
            }
        };

        self.add_primitive(
            spatial_node_index,
            clip_chain_id,
            prim_info,
            Vec::new(),
            text_run,
        );
    }

    pub fn add_image(
        &mut self,
        spatial_node_index: SpatialNodeIndex,
        clip_chain_id: ClipChainId,
        info: &LayoutPrimitiveInfo,
        stretch_size: LayoutSize,
        mut tile_spacing: LayoutSize,
        image_key: ImageKey,
        image_rendering: ImageRendering,
        alpha_type: AlphaType,
        color: ColorF,
    ) {
        let mut prim_rect = info.rect;
        simplify_repeated_primitive(&stretch_size, &mut tile_spacing, &mut prim_rect);
        let info = LayoutPrimitiveInfo {
            rect: prim_rect,
            .. *info
        };

        self.add_primitive(
            spatial_node_index,
            clip_chain_id,
            &info,
            Vec::new(),
            Image {
                key: image_key,
                tile_spacing: tile_spacing.into(),
                stretch_size: stretch_size.into(),
                color: color.into(),
                image_rendering,
                alpha_type,
            },
        );
    }

    pub fn add_yuv_image(
        &mut self,
        spatial_node_index: SpatialNodeIndex,
        clip_chain_id: ClipChainId,
        info: &LayoutPrimitiveInfo,
        yuv_data: YuvData,
        color_depth: ColorDepth,
        color_space: YuvColorSpace,
        color_range: ColorRange,
        image_rendering: ImageRendering,
    ) {
        let format = yuv_data.get_format();
        let yuv_key = match yuv_data {
            YuvData::NV12(plane_0, plane_1) => [plane_0, plane_1, ImageKey::DUMMY],
            YuvData::PlanarYCbCr(plane_0, plane_1, plane_2) => [plane_0, plane_1, plane_2],
            YuvData::InterleavedYCbCr(plane_0) => [plane_0, ImageKey::DUMMY, ImageKey::DUMMY],
        };

        self.add_nonshadowable_primitive(
            spatial_node_index,
            clip_chain_id,
            info,
            Vec::new(),
            YuvImage {
                color_depth,
                yuv_key,
                format,
                color_space,
                color_range,
                image_rendering,
            },
        );
    }

    fn add_primitive_instance_to_3d_root(
        &mut self,
        prim: ExtendedPrimitiveInstance,
    ) {
        // find the 3D root and append to the children list
        for sc in self.sc_stack.iter_mut().rev() {
            match sc.context_3d {
                Picture3DContext::In { root_data: Some(ref mut prims), .. } => {
                    prims.push(prim);
                    break;
                }
                Picture3DContext::In { .. } => {}
                Picture3DContext::Out => panic!("Unable to find 3D root"),
            }
        }
    }

    pub fn add_backdrop_filter(
        &mut self,
        spatial_node_index: SpatialNodeIndex,
        clip_chain_id: ClipChainId,
        info: &LayoutPrimitiveInfo,
        filters: Vec<Filter>,
        filter_datas: Vec<FilterData>,
        filter_primitives: Vec<FilterPrimitive>,
    ) {
        let mut backdrop_pic_index = match self.cut_backdrop_picture() {
            // Backdrop contains no content, so no need to add backdrop-filter
            None => return,
            Some(backdrop_pic_index) => backdrop_pic_index,
        };

        let backdrop_spatial_node_index = self.prim_store.pictures[backdrop_pic_index.0].spatial_node_index;
        let requested_raster_space = self.sc_stack.last().expect("no active stacking context").requested_raster_space;

        let mut instance = self.create_primitive(
            info,
            // TODO(cbrewster): This is a bit of a hack to help figure out the correct sizing of the backdrop
            // region. By makings sure to include this, the clip chain instance computes the correct clip rect,
            // but we don't actually apply the filtered backdrop clip yet (this is done to the last instance in
            // the filter chain below).
            backdrop_spatial_node_index,
            clip_chain_id,
            Backdrop {
                pic_index: backdrop_pic_index,
                spatial_node_index,
                border_rect: info.rect.into(),
            },
        );

        // We will append the filtered backdrop to the backdrop root, but we need to
        // make sure all clips between the current stacking context and backdrop root
        // are taken into account. So we wrap the backdrop filter instance with a picture with
        // a clip for each stacking context.
        for stacking_context in self.sc_stack.iter().rev().take_while(|sc| !sc.is_backdrop_root) {
            let clip_chain_id = stacking_context.clip_chain_id;
            let prim_flags = stacking_context.prim_flags;
            let composite_mode = None;

            let mut prim_list = PrimitiveList::empty();
            prim_list.add_prim(
                instance,
                LayoutRect::zero(),
                backdrop_spatial_node_index,
                prim_flags,
            );

            backdrop_pic_index = PictureIndex(self.prim_store.pictures
                .alloc()
                .init(PicturePrimitive::new_image(
                    composite_mode.clone(),
                    Picture3DContext::Out,
                    None,
                    true,
                    prim_flags,
                    requested_raster_space,
                    prim_list,
                    backdrop_spatial_node_index,
                    None,
                    PictureOptions {
                       inflate_if_required: false,
                    },
                ))
            );

            instance = create_prim_instance(
                backdrop_pic_index,
                composite_mode.into(),
                clip_chain_id,
                &mut self.interners,
            );
        }

        let (mut filtered_pic_index, mut filtered_instance) = self.wrap_prim_with_filters(
            instance,
            backdrop_pic_index,
            filters,
            filter_primitives,
            filter_datas,
            info.flags,
            requested_raster_space,
            backdrop_spatial_node_index,
            false,
        );

        // Apply filters from all stacking contexts up to, but not including the backdrop root.
        // Gecko pushes separate stacking contexts for filters and opacity,
        // so we must iterate through multiple stacking contexts to find all effects
        // that need to be applied to the filtered backdrop.
        let backdrop_root_pos = self.sc_stack.iter().rposition(|sc| sc.is_backdrop_root).expect("no backdrop root?");
        for i in ((backdrop_root_pos + 1)..self.sc_stack.len()).rev() {
            let stacking_context = &self.sc_stack[i];
            let filters = stacking_context.composite_ops.filters.clone();
            let filter_primitives = stacking_context.composite_ops.filter_primitives.clone();
            let filter_datas = stacking_context.composite_ops.filter_datas.clone();

            let (pic_index, instance) = self.wrap_prim_with_filters(
                filtered_instance,
                filtered_pic_index,
                filters,
                filter_primitives,
                filter_datas,
                info.flags,
                requested_raster_space,
                backdrop_spatial_node_index,
                false,
            );

            filtered_instance = instance;
            filtered_pic_index = pic_index;
        }

        filtered_instance.clip_chain_id = clip_chain_id;

        self.sc_stack
            .iter_mut()
            .rev()
            .find(|sc| sc.is_backdrop_root)
            .unwrap()
            .prim_list
            .add_prim(
                filtered_instance,
                LayoutRect::zero(),
                backdrop_spatial_node_index,
                info.flags,
            );
    }

    pub fn cut_backdrop_picture(&mut self) -> Option<PictureIndex> {
        let mut flattened_items = None;
        let mut backdrop_root =  None;
        let mut spatial_node_index = SpatialNodeIndex::INVALID;
        let mut prim_flags = PrimitiveFlags::default();
        for sc in self.sc_stack.iter_mut().rev() {
            // Add child contents to parent stacking context
            if let Some((_, flattened_instance)) = flattened_items.take() {
                sc.prim_list.add_prim(
                    flattened_instance,
                    LayoutRect::zero(),
                    spatial_node_index,
                    prim_flags,
                );
            }
            flattened_items = sc.cut_item_sequence(
                &mut self.prim_store,
                &mut self.interners,
                None,
                Picture3DContext::Out,
            );
            spatial_node_index = sc.spatial_node_index;
            prim_flags = sc.prim_flags;
            if sc.is_backdrop_root {
                backdrop_root = Some(sc);
                break;
            }
        }

        let (pic_index, instance) = flattened_items?;
        self.prim_store.pictures[pic_index.0].requested_composite_mode = Some(PictureCompositeMode::Blit(BlitReason::BACKDROP));
        backdrop_root.expect("no backdrop root found")
            .prim_list
            .add_prim(
                instance,
                LayoutRect::zero(),
                spatial_node_index,
                prim_flags,
            );

        Some(pic_index)
    }

    fn wrap_prim_with_filters(
        &mut self,
        mut cur_instance: PrimitiveInstance,
        mut current_pic_index: PictureIndex,
        mut filter_ops: Vec<Filter>,
        mut filter_primitives: Vec<FilterPrimitive>,
        filter_datas: Vec<FilterData>,
        flags: PrimitiveFlags,
        requested_raster_space: RasterSpace,
        spatial_node_index: SpatialNodeIndex,
        inflate_if_required: bool,
    ) -> (PictureIndex, PrimitiveInstance) {
        // TODO(cbrewster): Currently CSS and SVG filters live side by side in WebRender, but unexpected results will
        // happen if they are used simulataneously. Gecko only provides either filter ops or filter primitives.
        // At some point, these two should be combined and CSS filters should be expressed in terms of SVG filters.
        assert!(filter_ops.is_empty() || filter_primitives.is_empty(),
            "Filter ops and filter primitives are not allowed on the same stacking context.");

        // For each filter, create a new image with that composite mode.
        let mut current_filter_data_index = 0;
        for filter in &mut filter_ops {
            let composite_mode = Some(match *filter {
                Filter::ComponentTransfer => {
                    let filter_data =
                        &filter_datas[current_filter_data_index];
                    let filter_data = filter_data.sanitize();
                    current_filter_data_index = current_filter_data_index + 1;
                    if filter_data.is_identity() {
                        continue
                    } else {
                        let filter_data_key = SFilterDataKey {
                            data:
                                SFilterData {
                                    r_func: SFilterDataComponent::from_functype_values(
                                        filter_data.func_r_type, &filter_data.r_values),
                                    g_func: SFilterDataComponent::from_functype_values(
                                        filter_data.func_g_type, &filter_data.g_values),
                                    b_func: SFilterDataComponent::from_functype_values(
                                        filter_data.func_b_type, &filter_data.b_values),
                                    a_func: SFilterDataComponent::from_functype_values(
                                        filter_data.func_a_type, &filter_data.a_values),
                                },
                        };

                        let handle = self.interners
                            .filter_data
                            .intern(&filter_data_key, || ());
                        PictureCompositeMode::ComponentTransferFilter(handle)
                    }
                }
                _ => PictureCompositeMode::Filter(filter.clone()),
            });

            let mut prim_list = PrimitiveList::empty();
            prim_list.add_prim(
                cur_instance.clone(),
                LayoutRect::zero(),
                spatial_node_index,
                flags,
            );

            let filter_pic_index = PictureIndex(self.prim_store.pictures
                .alloc()
                .init(PicturePrimitive::new_image(
                    composite_mode.clone(),
                    Picture3DContext::Out,
                    None,
                    true,
                    flags,
                    requested_raster_space,
                    prim_list,
                    spatial_node_index,
                    None,
                    PictureOptions {
                       inflate_if_required,
                    },
                ))
            );

            current_pic_index = filter_pic_index;
            cur_instance = create_prim_instance(
                current_pic_index,
                composite_mode.into(),
                ClipChainId::NONE,
                &mut self.interners,
            );

            if cur_instance.is_chased() {
                println!("\tis a composite picture for a stacking context with {:?}", filter);
            }

            // Run the optimize pass on this picture, to see if we can
            // collapse opacity and avoid drawing to an off-screen surface.
            self.prim_store.optimize_picture_if_possible(current_pic_index);
        }

        if !filter_primitives.is_empty() {
            let filter_datas = filter_datas.iter()
                .map(|filter_data| filter_data.sanitize())
                .map(|filter_data| {
                    SFilterData {
                        r_func: SFilterDataComponent::from_functype_values(
                            filter_data.func_r_type, &filter_data.r_values),
                        g_func: SFilterDataComponent::from_functype_values(
                            filter_data.func_g_type, &filter_data.g_values),
                        b_func: SFilterDataComponent::from_functype_values(
                            filter_data.func_b_type, &filter_data.b_values),
                        a_func: SFilterDataComponent::from_functype_values(
                            filter_data.func_a_type, &filter_data.a_values),
                    }
                })
                .collect();

            // Sanitize filter inputs
            for primitive in &mut filter_primitives {
                primitive.sanitize();
            }

            let composite_mode = PictureCompositeMode::SvgFilter(
                filter_primitives,
                filter_datas,
            );

            let mut prim_list = PrimitiveList::empty();
            prim_list.add_prim(
                cur_instance.clone(),
                LayoutRect::zero(),
                spatial_node_index,
                flags,
            );

            let filter_pic_index = PictureIndex(self.prim_store.pictures
                .alloc()
                .init(PicturePrimitive::new_image(
                    Some(composite_mode.clone()),
                    Picture3DContext::Out,
                    None,
                    true,
                    flags,
                    requested_raster_space,
                    prim_list,
                    spatial_node_index,
                    None,
                    PictureOptions {
                        inflate_if_required,
                    },
                ))
            );

            current_pic_index = filter_pic_index;
            cur_instance = create_prim_instance(
                current_pic_index,
                Some(composite_mode).into(),
                ClipChainId::NONE,
                &mut self.interners,
            );

            if cur_instance.is_chased() {
                println!("\tis a composite picture for a stacking context with an SVG filter");
            }

            // Run the optimize pass on this picture, to see if we can
            // collapse opacity and avoid drawing to an off-screen surface.
            self.prim_store.optimize_picture_if_possible(current_pic_index);
        }
        (current_pic_index, cur_instance)
    }
}


pub trait CreateShadow {
    fn create_shadow(&self, shadow: &Shadow) -> Self;
}

pub trait IsVisible {
    fn is_visible(&self) -> bool;
}

/// A primitive instance + some extra information about the primitive. This is
/// stored when constructing 3d rendering contexts, which involve cutting
/// primitive lists.
struct ExtendedPrimitiveInstance {
    instance: PrimitiveInstance,
    spatial_node_index: SpatialNodeIndex,
    flags: PrimitiveFlags,
}

/// Properties of a stacking context that are maintained
/// during creation of the scene. These structures are
/// not persisted after the initial scene build.
struct FlattenedStackingContext {
    /// The list of primitive instances added to this stacking context.
    prim_list: PrimitiveList,

    /// Primitive instance flags for compositing this stacking context
    prim_flags: PrimitiveFlags,

    /// Whether or not the caller wants this drawn in
    /// screen space (quality) or local space (performance)
    requested_raster_space: RasterSpace,

    /// The positioning node for this stacking context
    spatial_node_index: SpatialNodeIndex,

    /// The clip chain for this stacking context
    clip_chain_id: ClipChainId,
    clip_id: Option<ClipId>,

    /// If set, this should be provided to caller
    /// as an output texture.
    frame_output_pipeline_id: Option<PipelineId>,

    /// The list of filters / mix-blend-mode for this
    /// stacking context.
    composite_ops: CompositeOps,

    /// Bitfield of reasons this stacking context needs to
    /// be an offscreen surface.
    blit_reason: BlitReason,

    /// Pipeline this stacking context belongs to.
    pipeline_id: PipelineId,

    /// CSS transform-style property.
    transform_style: TransformStyle,

    /// Defines the relationship to a preserve-3D hiearachy.
    context_3d: Picture3DContext<ExtendedPrimitiveInstance>,

    /// True if this stacking context is a backdrop root.
    is_backdrop_root: bool,

    /// True if this stacking context is redundant (i.e. doesn't require a surface)
    is_redundant: bool,

    /// A helper struct to snap local rects in device space. During frame
    /// building we may establish new raster roots, however typically that is in
    /// cases where we won't be applying snapping (e.g. has perspective), or in
    /// edge cases (e.g. SVG filter) where we can accept slightly incorrect
    /// behaviour in favour of getting the common case right.
    snap_to_device: SpaceSnapper,
}

impl FlattenedStackingContext {
    /// Return true if the stacking context has a valid preserve-3d property
    pub fn is_3d(&self) -> bool {
        self.transform_style == TransformStyle::Preserve3D && self.composite_ops.is_empty()
    }

    /// Set up appropriate cluster flags for picture caching on this stacking context.
    fn init_picture_caching(
        &mut self,
        spatial_tree: &SpatialTree,
        clip_store: &ClipStore,
        quality_settings: &QualitySettings,
    ) -> usize {
        struct SliceInfo {
            cluster_index: usize,
            scroll_root: SpatialNodeIndex,
            cluster_flags: ClusterFlags,
        }

        let mut content_slice_count = 0;
        let mut slices: Vec<SliceInfo> = Vec::new();

        // Step through each cluster, and work out where the slice boundaries should be.
        for (cluster_index, cluster) in self.prim_list.clusters.iter().enumerate() {
            let scroll_root = spatial_tree.find_scroll_root(
                cluster.spatial_node_index,
            );

            // We want to create a slice in the following conditions:
            // (1) This cluster is a scrollbar
            // (2) Certain conditions when the scroll root changes (see below)
            // (3) No slice exists yet
            let mut cluster_flags = ClusterFlags::empty();

            if cluster.flags.contains(ClusterFlags::SCROLLBAR_CONTAINER) {
                // Scrollbar containers need to ensure that a new slice is
                // created both before and after the scrollbar, so that no
                // other prims with the same scroll root sneak into this slice.
                cluster_flags.insert(
                    ClusterFlags::CREATE_PICTURE_CACHE_PRE |
                    ClusterFlags::CREATE_PICTURE_CACHE_POST
                );
            }

            let create_new_slice_for_scroll_root =
                slices.last().map(|slice| {
                    match (slice.scroll_root, scroll_root) {
                        (ROOT_SPATIAL_NODE_INDEX, ROOT_SPATIAL_NODE_INDEX) => {
                            // Both current slice and this cluster are fixed position, no need to cut
                            false
                        }
                        (ROOT_SPATIAL_NODE_INDEX, _) => {
                            // A real scroll root is being established, so create a cache slice
                            true
                        }
                        (_, ROOT_SPATIAL_NODE_INDEX) => {
                            // If quality settings force subpixel AA over performance, skip creating
                            // a slice for the fixed position element(s) here.
                            if quality_settings.force_subpixel_aa_where_possible {
                                return false;
                            }

                            // A fixed position slice is encountered within a scroll root. Only create
                            // a slice in this case if all the clips referenced by this cluster are also
                            // fixed position. There's no real point in creating slices for these cases,
                            // since we'll have to rasterize them as the scrolling clip moves anyway. It
                            // also allows us to retain subpixel AA in these cases. For these types of
                            // slices, the intra-slice dirty rect handling typically works quite well
                            // (a common case is parallax scrolling effects).
                            for prim_instance in &cluster.prim_instances {
                                let mut current_clip_chain_id = prim_instance.clip_chain_id;

                                while current_clip_chain_id != ClipChainId::NONE {
                                    let clip_chain_node = &clip_store
                                        .clip_chain_nodes[current_clip_chain_id.0 as usize];
                                    let spatial_root = spatial_tree.find_scroll_root(clip_chain_node.spatial_node_index);
                                    if spatial_root != ROOT_SPATIAL_NODE_INDEX {
                                        return false;
                                    }
                                    current_clip_chain_id = clip_chain_node.parent_clip_chain_id;
                                }
                            }

                            true
                        }
                        (curr_scroll_root, scroll_root) => {
                            // Two scrolling roots - only need a new slice if they differ
                            curr_scroll_root != scroll_root
                        }
                    }
                }).unwrap_or(true);

            if create_new_slice_for_scroll_root {
                cluster_flags.insert(ClusterFlags::CREATE_PICTURE_CACHE_PRE);
            }

            // Create a new slice if required
            if !cluster_flags.is_empty() {
                slices.push(SliceInfo {
                    cluster_index,
                    scroll_root,
                    cluster_flags,
                });
            }
        }

        // If the page would create too many slices (an arbitrary definition where
        // it's assumed the GPU memory + compositing overhead would be too high)
        // then just create a single picture cache for the entire content. This at
        // least means that we can cache small content changes efficiently when
        // scrolling isn't occurring. Scrolling regions will be handled reasonably
        // efficiently by the dirty rect tracking (since it's likely that if the
        // page has so many slices there isn't a single major scroll region).
        const MAX_CONTENT_SLICES: usize = 8;

        if slices.len() > MAX_CONTENT_SLICES {
            if let Some(cluster) = self.prim_list.clusters.first_mut() {
                content_slice_count = 1;
                cluster.flags.insert(ClusterFlags::CREATE_PICTURE_CACHE_PRE);
                cluster.cache_scroll_root = None;
            }
        } else {
            // Walk the list of slices, setting appropriate flags on the clusters which are
            // later used during setup_picture_caching.
            for slice in slices.drain(..) {
                content_slice_count += 1;
                let cluster = &mut self.prim_list.clusters[slice.cluster_index];
                // Mark that this cluster creates a picture cache slice
                cluster.flags.insert(slice.cluster_flags);
                cluster.cache_scroll_root = Some(slice.scroll_root);
            }
        }

        // Always end the cache at the end of the stacking context, so that we don't
        // cache anything from primitives outside this pipeline in the same slice.
        if let Some(cluster) = self.prim_list.clusters.last_mut() {
            cluster.flags.insert(ClusterFlags::CREATE_PICTURE_CACHE_POST);
        }

        content_slice_count
    }

    /// Return true if the stacking context isn't needed.
    pub fn is_redundant(
        context_3d: &Picture3DContext<ExtendedPrimitiveInstance>,
        composite_ops: &CompositeOps,
        prim_flags: PrimitiveFlags,
        blit_reason: BlitReason,
        requested_raster_space: RasterSpace,
        parent: &FlattenedStackingContext,
    ) -> bool {
        // Any 3d context is required
        if let Picture3DContext::In { .. } = context_3d {
            return false;
        }

        // If there are filters / mix-blend-mode
        if !composite_ops.filters.is_empty() {
            return false;
        }

        // If there are svg filters
        if !composite_ops.filter_primitives.is_empty() {
            return false;
        }

        // We can skip mix-blend modes if they are the first primitive in a stacking context,
        // see pop_stacking_context for a full explanation.
        if composite_ops.mix_blend_mode.is_some() &&
            !parent.prim_list.is_empty() {
            return false;
        }

        // If backface visibility is explicitly set.
        if !prim_flags.contains(PrimitiveFlags::IS_BACKFACE_VISIBLE) {
            return false;
        }

        // If rasterization space is different
        if requested_raster_space != parent.requested_raster_space {
            return false;
        }

        // If need to isolate in surface due to clipping / mix-blend-mode
        if !blit_reason.is_empty() {
            return false;
        }

        // If this stacking context is a scrollbar, retain it so it can form a picture cache slice
        if prim_flags.contains(PrimitiveFlags::IS_SCROLLBAR_CONTAINER) {
            return false;
        }

        // It is redundant!
        true
    }

    /// Cut the sequence of the immediate children recorded so far and generate a picture from them.
    pub fn cut_item_sequence(
        &mut self,
        prim_store: &mut PrimitiveStore,
        interners: &mut Interners,
        composite_mode: Option<PictureCompositeMode>,
        flat_items_context_3d: Picture3DContext<OrderedPictureChild>,
    ) -> Option<(PictureIndex, PrimitiveInstance)> {
        if self.prim_list.is_empty() {
            return None
        }

        let pic_index = PictureIndex(prim_store.pictures
            .alloc()
            .init(PicturePrimitive::new_image(
                composite_mode.clone(),
                flat_items_context_3d,
                None,
                true,
                self.prim_flags,
                self.requested_raster_space,
                mem::replace(&mut self.prim_list, PrimitiveList::empty()),
                self.spatial_node_index,
                None,
                PictureOptions::default(),
            ))
        );

        let prim_instance = create_prim_instance(
            pic_index,
            composite_mode.into(),
            self.clip_chain_id,
            interners,
        );

        Some((pic_index, prim_instance))
    }
}

/// A primitive that is added while a shadow context is
/// active is stored as a pending primitive and only
/// added to pictures during pop_all_shadows.
pub struct PendingPrimitive<T> {
    spatial_node_index: SpatialNodeIndex,
    clip_chain_id: ClipChainId,
    info: LayoutPrimitiveInfo,
    prim: T,
}

/// As shadows are pushed, they are stored as pending
/// shadows, and handled at once during pop_all_shadows.
pub struct PendingShadow {
    shadow: Shadow,
    should_inflate: bool,
    spatial_node_index: SpatialNodeIndex,
    clip_chain_id: ClipChainId,
}

pub enum ShadowItem {
    Shadow(PendingShadow),
    Image(PendingPrimitive<Image>),
    LineDecoration(PendingPrimitive<LineDecoration>),
    NormalBorder(PendingPrimitive<NormalBorderPrim>),
    Primitive(PendingPrimitive<PrimitiveKeyKind>),
    TextRun(PendingPrimitive<TextRun>),
}

impl From<PendingPrimitive<Image>> for ShadowItem {
    fn from(image: PendingPrimitive<Image>) -> Self {
        ShadowItem::Image(image)
    }
}

impl From<PendingPrimitive<LineDecoration>> for ShadowItem {
    fn from(line_dec: PendingPrimitive<LineDecoration>) -> Self {
        ShadowItem::LineDecoration(line_dec)
    }
}

impl From<PendingPrimitive<NormalBorderPrim>> for ShadowItem {
    fn from(border: PendingPrimitive<NormalBorderPrim>) -> Self {
        ShadowItem::NormalBorder(border)
    }
}

impl From<PendingPrimitive<PrimitiveKeyKind>> for ShadowItem {
    fn from(container: PendingPrimitive<PrimitiveKeyKind>) -> Self {
        ShadowItem::Primitive(container)
    }
}

impl From<PendingPrimitive<TextRun>> for ShadowItem {
    fn from(text_run: PendingPrimitive<TextRun>) -> Self {
        ShadowItem::TextRun(text_run)
    }
}

fn create_prim_instance(
    pic_index: PictureIndex,
    composite_mode_key: PictureCompositeKey,
    clip_chain_id: ClipChainId,
    interners: &mut Interners,
) -> PrimitiveInstance {
    let pic_key = PictureKey::new(
        Picture { composite_mode_key },
    );

    let data_handle = interners
        .picture
        .intern(&pic_key, || ());

    PrimitiveInstance::new(
        LayoutRect::max_rect(),
        PrimitiveInstanceKind::Picture {
            data_handle,
            pic_index,
            segment_instance_index: SegmentInstanceIndex::INVALID,
        },
        clip_chain_id,
    )
}

fn filter_ops_for_compositing(
    input_filters: ItemRange<FilterOp>,
) -> Vec<Filter> {
    // TODO(gw): Now that we resolve these later on,
    //           we could probably make it a bit
    //           more efficient than cloning these here.
    input_filters.iter().map(|filter| filter.into()).collect()
}

fn filter_datas_for_compositing(
    input_filter_datas: &[TempFilterData],
) -> Vec<FilterData> {
    // TODO(gw): Now that we resolve these later on,
    //           we could probably make it a bit
    //           more efficient than cloning these here.
    let mut filter_datas = vec![];
    for temp_filter_data in input_filter_datas {
        let func_types : Vec<ComponentTransferFuncType> = temp_filter_data.func_types.iter().collect();
        debug_assert!(func_types.len() == 4);
        filter_datas.push( FilterData {
            func_r_type: func_types[0],
            r_values: temp_filter_data.r_values.iter().collect(),
            func_g_type: func_types[1],
            g_values: temp_filter_data.g_values.iter().collect(),
            func_b_type: func_types[2],
            b_values: temp_filter_data.b_values.iter().collect(),
            func_a_type: func_types[3],
            a_values: temp_filter_data.a_values.iter().collect(),
        });
    }
    filter_datas
}

fn filter_primitives_for_compositing(
    input_filter_primitives: ItemRange<FilterPrimitive>,
) -> Vec<FilterPrimitive> {
    // Resolve these in the flattener?
    // TODO(gw): Now that we resolve these later on,
    //           we could probably make it a bit
    //           more efficient than cloning these here.
    input_filter_primitives.iter().map(|primitive| primitive).collect()
}

fn process_repeat_size(
    snapped_rect: &LayoutRect,
    unsnapped_rect: &LayoutRect,
    repeat_size: LayoutSize,
) -> LayoutSize {
    // FIXME(aosmond): The tile size is calculated based on several parameters
    // during display list building. It may produce a slightly different result
    // than the bounds due to floating point error accumulation, even though in
    // theory they should be the same. We do a fuzzy check here to paper over
    // that. It may make more sense to push the original parameters into scene
    // building and let it do a saner calculation with more information (e.g.
    // the snapped values).
    const EPSILON: f32 = 0.001;
    LayoutSize::new(
        if repeat_size.width.approx_eq_eps(&unsnapped_rect.size.width, &EPSILON) {
            snapped_rect.size.width
        } else {
            repeat_size.width
        },
        if repeat_size.height.approx_eq_eps(&unsnapped_rect.size.height, &EPSILON) {
            snapped_rect.size.height
        } else {
            repeat_size.height
        },
    )
}

/// Given a PrimitiveList and scroll root, construct a tile cache primitive instance
/// that wraps the primitive list.
fn create_tile_cache(
    slice: usize,
    slice_flags: SliceFlags,
    scroll_root: SpatialNodeIndex,
    prim_list: PrimitiveList,
    background_color: Option<ColorF>,
    shared_clips: Vec<ClipInstance>,
    interners: &mut Interners,
    prim_store: &mut PrimitiveStore,
    clip_store: &mut ClipStore,
    picture_cache_spatial_nodes: &mut FastHashSet<SpatialNodeIndex>,
    frame_builder_config: &FrameBuilderConfig,
) -> PrimitiveInstance {
    // Add this spatial node to the list to check for complex transforms
    // at the start of a frame build.
    picture_cache_spatial_nodes.insert(scroll_root);

    // Now, create a picture with tile caching enabled that will hold all
    // of the primitives selected as belonging to the main scroll root.
    let pic_key = PictureKey::new(
        Picture {
            composite_mode_key: PictureCompositeKey::Identity,
        },
    );

    let pic_data_handle = interners
        .picture
        .intern(&pic_key, || ());

    // Build a clip-chain for the tile cache, that contains any of the shared clips
    // we will apply when drawing the tiles. In all cases provided by Gecko, these
    // are rectangle clips with a scale/offset transform only, and get handled as
    // a simple local clip rect in the vertex shader. However, this should in theory
    // also work with any complex clips, such as rounded rects and image masks, by
    // producing a clip mask that is applied to the picture cache tiles.
    let mut parent_clip_chain_id = ClipChainId::NONE;
    for clip_instance in &shared_clips {
        // Add this spatial node to the list to check for complex transforms
        // at the start of a frame build.
        picture_cache_spatial_nodes.insert(clip_instance.spatial_node_index);

        parent_clip_chain_id = clip_store.add_clip_chain_node(
            clip_instance.handle,
            clip_instance.spatial_node_index,
            parent_clip_chain_id,
        );
    }

    let tile_cache = Box::new(TileCacheInstance::new(
        slice,
        slice_flags,
        scroll_root,
        background_color,
        shared_clips,
        parent_clip_chain_id,
        frame_builder_config,
    ));

    let pic_index = prim_store.pictures.alloc().init(PicturePrimitive::new_image(
        Some(PictureCompositeMode::TileCache { }),
        Picture3DContext::Out,
        None,
        true,
        PrimitiveFlags::IS_BACKFACE_VISIBLE,
        RasterSpace::Screen,
        prim_list,
        scroll_root,
        Some(tile_cache),
        PictureOptions::default(),
    ));

    PrimitiveInstance::new(
        LayoutRect::max_rect(),
        PrimitiveInstanceKind::Picture {
            data_handle: pic_data_handle,
            pic_index: PictureIndex(pic_index),
            segment_instance_index: SegmentInstanceIndex::INVALID,
        },
        parent_clip_chain_id,
    )
}

// Helper fn to collect clip handles from a given clip chain.
fn add_clips(
    clip_chain_id: ClipChainId,
    prim_clips: &mut Vec<ClipInstance>,
    clip_store: &ClipStore,
    interners: &Interners,
) {
    let mut current_clip_chain_id = clip_chain_id;

    while current_clip_chain_id != ClipChainId::NONE {
        let clip_chain_node = &clip_store
            .clip_chain_nodes[current_clip_chain_id.0 as usize];

        let clip_node_data = &interners.clip[clip_chain_node.handle];
        if let ClipNodeKind::Rectangle = clip_node_data.clip_node_kind {
            prim_clips.push(ClipInstance::new(clip_chain_node.handle, clip_chain_node.spatial_node_index));
        }

        current_clip_chain_id = clip_chain_node.parent_clip_chain_id;
    }
}
