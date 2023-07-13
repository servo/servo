/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use api::{BorderRadius, ClipMode, HitTestItem, HitTestResult, ItemTag, PrimitiveFlags};
use api::{PipelineId, ApiHitTester, ClipId};
use api::units::*;
use crate::clip::{ClipItemKind, ClipStore, ClipNode, rounded_rectangle_contains_point};
use crate::clip::{polygon_contains_point};
use crate::prim_store::PolygonKey;
use crate::scene_builder_thread::Interners;
use crate::spatial_tree::{SpatialNodeIndex, SpatialTree};
use crate::internal_types::{FastHashMap, FastHashSet, LayoutPrimitiveInfo};
use std::ops;
use std::sync::{Arc, Mutex};
use crate::util::{LayoutToWorldFastTransform, VecHelper};

pub struct SharedHitTester {
    // We don't really need a mutex here. We could do with some sort of
    // atomic-atomic-ref-counted pointer (an Arc which would let the pointer
    // be swapped atomically like an AtomicPtr).
    // In practive this shouldn't cause performance issues, though.
    hit_tester: Mutex<Arc<HitTester>>,
}

impl SharedHitTester {
    pub fn new() -> Self {
        SharedHitTester {
            hit_tester: Mutex::new(Arc::new(HitTester::empty())),
        }
    }

    pub fn get_ref(&self) -> Arc<HitTester> {
        let guard = self.hit_tester.lock().unwrap();
        Arc::clone(&*guard)
    }

    pub(crate) fn update(&self, new_hit_tester: Arc<HitTester>) {
        let mut guard = self.hit_tester.lock().unwrap();
        *guard = new_hit_tester;
    }
}

impl ApiHitTester for SharedHitTester {
    fn hit_test(&self,
        pipeline_id: Option<PipelineId>,
        point: WorldPoint,
    ) -> HitTestResult {
        self.get_ref().hit_test(HitTest::new(pipeline_id, point))
    }
}

/// A copy of important spatial node data to use during hit testing. This a copy of
/// data from the SpatialTree that will persist as a new frame is under construction,
/// allowing hit tests consistent with the currently rendered frame.
#[derive(MallocSizeOf)]
struct HitTestSpatialNode {
    /// The pipeline id of this node.
    pipeline_id: PipelineId,

    /// World transform for content transformed by this node.
    world_content_transform: LayoutToWorldFastTransform,

    /// World viewport transform for content transformed by this node.
    world_viewport_transform: LayoutToWorldFastTransform,

    /// The accumulated external scroll offset for this spatial node.
    external_scroll_offset: LayoutVector2D,
}

#[derive(MallocSizeOf)]
struct HitTestClipNode {
    /// A particular point must be inside all of these regions to be considered clipped in
    /// for the purposes of a hit test.
    region: HitTestRegion,
    /// The positioning node for this clip
    spatial_node_index: SpatialNodeIndex,
}

impl HitTestClipNode {
    fn new(
        node: ClipNode,
        spatial_node_index: SpatialNodeIndex,
        interners: &Interners,
    ) -> Self {
        let region = match node.item.kind {
            ClipItemKind::Rectangle { rect, mode } => {
                HitTestRegion::Rectangle(rect, mode)
            }
            ClipItemKind::RoundedRectangle { rect, radius, mode } => {
                HitTestRegion::RoundedRectangle(rect, radius, mode)
            }
            ClipItemKind::Image { rect, polygon_handle, .. } => {
                if let Some(handle) = polygon_handle {
                    // Retrieve the polygon data from the interner.
                    let polygon = &interners.polygon[handle];
                    HitTestRegion::Polygon(rect, *polygon)
                } else {
                    HitTestRegion::Rectangle(rect, ClipMode::Clip)
                }
            }
            ClipItemKind::BoxShadow { .. } => HitTestRegion::Invalid,
        };

        HitTestClipNode {
            region,
            spatial_node_index,
        }
    }
}

#[derive(Clone, MallocSizeOf)]
struct HitTestingItem {
    rect: LayoutRect,
    clip_rect: LayoutRect,
    tag: ItemTag,
    is_backface_visible: bool,
    spatial_node_index: SpatialNodeIndex,
    #[ignore_malloc_size_of = "Range"]
    clip_nodes_range: ops::Range<ClipNodeIndex>,
}

impl HitTestingItem {
    fn new(
        tag: ItemTag,
        info: &LayoutPrimitiveInfo,
        spatial_node_index: SpatialNodeIndex,
        clip_nodes_range: ops::Range<ClipNodeIndex>,
    ) -> HitTestingItem {
        HitTestingItem {
            rect: info.rect,
            clip_rect: info.clip_rect,
            tag,
            is_backface_visible: info.flags.contains(PrimitiveFlags::IS_BACKFACE_VISIBLE),
            spatial_node_index,
            clip_nodes_range,
        }
    }
}

/// Statistics about allocation sizes of current hit tester,
/// used to pre-allocate size of the next hit tester.
pub struct HitTestingSceneStats {
    pub clip_nodes_count: usize,
    pub items_count: usize,
}

impl HitTestingSceneStats {
    pub fn empty() -> Self {
        HitTestingSceneStats {
            clip_nodes_count: 0,
            items_count: 0,
        }
    }
}

#[derive(MallocSizeOf, Debug, Copy, Clone)]
pub struct ClipNodeIndex(u32);

/// Defines the immutable part of a hit tester for a given scene.
/// The hit tester is recreated each time a frame is built, since
/// it relies on the current values of the spatial tree.
/// However, the clip chain and item definitions don't change,
/// so they are created once per scene, and shared between
/// hit tester instances via Arc.
#[derive(MallocSizeOf)]
pub struct HitTestingScene {
    /// Packed array of all hit test clip nodes
    clip_nodes: Vec<HitTestClipNode>,

    /// List of hit testing primitives.
    items: Vec<HitTestingItem>,

    /// Current stack of clip ids from stacking context
    #[ignore_malloc_size_of = "ClipId"]
    clip_id_stack: Vec<ClipId>,

    /// Last cached clip id, useful for scenes with a lot
    /// of hit-test items that reference the same clip
    #[ignore_malloc_size_of = "simple"]
    cached_clip_id: Option<(ClipId, ops::Range<ClipNodeIndex>)>,

    /// Temporary buffer used to de-duplicate clip ids when creating hit
    /// test clip nodes.
    #[ignore_malloc_size_of = "ClipId"]
    seen_clips: FastHashSet<ClipId>,
}

impl HitTestingScene {
    /// Construct a new hit testing scene, pre-allocating to size
    /// provided by previous scene stats.
    pub fn new(stats: &HitTestingSceneStats) -> Self {
        HitTestingScene {
            clip_nodes: Vec::with_capacity(stats.clip_nodes_count),
            items: Vec::with_capacity(stats.items_count),
            clip_id_stack: Vec::with_capacity(8),
            cached_clip_id: None,
            seen_clips: FastHashSet::default(),
        }
    }

    /// Get stats about the current scene allocation sizes.
    pub fn get_stats(&self) -> HitTestingSceneStats {
        HitTestingSceneStats {
            clip_nodes_count: self.clip_nodes.len(),
            items_count: self.items.len(),
        }
    }

    /// Add a hit testing primitive.
    pub fn add_item(
        &mut self,
        tag: ItemTag,
        info: &LayoutPrimitiveInfo,
        spatial_node_index: SpatialNodeIndex,
        clip_id: ClipId,
        clip_store: &ClipStore,
        interners: &Interners,
    ) {
        let clip_range = match self.cached_clip_id {
            Some((cached_clip_id, ref range)) if cached_clip_id == clip_id => {
                range.clone()
            }
            Some(_) | None => {
                let start = ClipNodeIndex(self.clip_nodes.len() as u32);

                // Clear the set of which clip ids have been encountered for this item
                self.seen_clips.clear();

                // Flatten all clips from the stacking context hierarchy
                for clip_id in &self.clip_id_stack {
                    add_clips(
                        *clip_id,
                        clip_store,
                        &mut self.clip_nodes,
                        &mut self.seen_clips,
                        interners,
                    );
                }

                // Add the primitive clip
                add_clips(
                    clip_id,
                    clip_store,
                    &mut self.clip_nodes,
                    &mut self.seen_clips,
                    interners,
                );

                let end = ClipNodeIndex(self.clip_nodes.len() as u32);

                let range = ops::Range {
                    start,
                    end,
                };

                self.cached_clip_id = Some((clip_id, range.clone()));

                range
            }
        };

        let item = HitTestingItem::new(
            tag,
            info,
            spatial_node_index,
            clip_range,
        );

        self.items.push(item);
    }

    /// Push a clip onto the current stack
    pub fn push_clip(
        &mut self,
        clip_id: ClipId,
    ) {
        // Invalidate the cache since the stack may affect the produced hit test clip struct
        self.cached_clip_id = None;

        self.clip_id_stack.push(clip_id);
    }

    /// Pop a clip from the current stack
    pub fn pop_clip(
        &mut self,
    ) {
        // Invalidate the cache since the stack may affect the produced hit test clip struct
        self.cached_clip_id = None;

        self.clip_id_stack.pop().unwrap();
    }
}

#[derive(MallocSizeOf)]
enum HitTestRegion {
    Invalid,
    Rectangle(LayoutRect, ClipMode),
    RoundedRectangle(LayoutRect, BorderRadius, ClipMode),
    Polygon(LayoutRect, PolygonKey),
}

impl HitTestRegion {
    fn contains(&self, point: &LayoutPoint) -> bool {
        match *self {
            HitTestRegion::Rectangle(ref rectangle, ClipMode::Clip) =>
                rectangle.contains(*point),
            HitTestRegion::Rectangle(ref rectangle, ClipMode::ClipOut) =>
                !rectangle.contains(*point),
            HitTestRegion::RoundedRectangle(rect, radii, ClipMode::Clip) =>
                rounded_rectangle_contains_point(point, &rect, &radii),
            HitTestRegion::RoundedRectangle(rect, radii, ClipMode::ClipOut) =>
                !rounded_rectangle_contains_point(point, &rect, &radii),
            HitTestRegion::Polygon(rect, polygon) =>
                polygon_contains_point(point, &rect, &polygon),
            HitTestRegion::Invalid => true,
        }
    }
}

#[derive(MallocSizeOf)]
pub struct HitTester {
    #[ignore_malloc_size_of = "Arc"]
    scene: Arc<HitTestingScene>,
    spatial_nodes: Vec<HitTestSpatialNode>,
    pipeline_root_nodes: FastHashMap<PipelineId, SpatialNodeIndex>,
}

impl HitTester {
    pub fn empty() -> Self {
        HitTester {
            scene: Arc::new(HitTestingScene::new(&HitTestingSceneStats::empty())),
            spatial_nodes: Vec::new(),
            pipeline_root_nodes: FastHashMap::default(),
        }
    }

    pub fn new(
        scene: Arc<HitTestingScene>,
        spatial_tree: &SpatialTree,
    ) -> HitTester {
        let mut hit_tester = HitTester {
            scene,
            spatial_nodes: Vec::new(),
            pipeline_root_nodes: FastHashMap::default(),
        };
        hit_tester.read_spatial_tree(spatial_tree);
        hit_tester
    }

    fn read_spatial_tree(
        &mut self,
        spatial_tree: &SpatialTree,
    ) {
        self.spatial_nodes.clear();

        self.spatial_nodes.reserve(spatial_tree.spatial_nodes.len());
        for (index, node) in spatial_tree.spatial_nodes.iter().enumerate() {
            let index = SpatialNodeIndex::new(index);

            // If we haven't already seen a node for this pipeline, record this one as the root
            // node.
            self.pipeline_root_nodes.entry(node.pipeline_id).or_insert(index);

            //TODO: avoid inverting more than necessary:
            //  - if the coordinate system is non-invertible, no need to try any of these concrete transforms
            //  - if there are other places where inversion is needed, let's not repeat the step

            self.spatial_nodes.push(HitTestSpatialNode {
                pipeline_id: node.pipeline_id,
                world_content_transform: spatial_tree
                    .get_world_transform(index)
                    .into_fast_transform(),
                world_viewport_transform: spatial_tree
                    .get_world_viewport_transform(index)
                    .into_fast_transform(),
                external_scroll_offset: spatial_tree.external_scroll_offset(index),
            });
        }
    }

    pub fn hit_test(&self, test: HitTest) -> HitTestResult {
        let mut result = HitTestResult::default();

        let mut current_spatial_node_index = SpatialNodeIndex::INVALID;
        let mut point_in_layer = None;
        let mut current_root_spatial_node_index = SpatialNodeIndex::INVALID;
        let mut point_in_viewport = None;

        // For each hit test primitive
        for item in self.scene.items.iter().rev() {
            let scroll_node = &self.spatial_nodes[item.spatial_node_index.0 as usize];
            let pipeline_id = scroll_node.pipeline_id;
            match (test.pipeline_id, pipeline_id) {
                (Some(id), node_id) if node_id != id => continue,
                _ => {},
            }

            // Update the cached point in layer space, if the spatial node
            // changed since last primitive.
            if item.spatial_node_index != current_spatial_node_index {
                point_in_layer = scroll_node
                    .world_content_transform
                    .inverse()
                    .and_then(|inverted| inverted.transform_point2d(test.point));
                current_spatial_node_index = item.spatial_node_index;
            }

            // Only consider hit tests on transformable layers.
            if let Some(point_in_layer) = point_in_layer {
                // If the item's rect or clip rect don't contain this point,
                // it's not a valid hit.
                if !item.rect.contains(point_in_layer) {
                    continue;
                }
                if !item.clip_rect.contains(point_in_layer) {
                    continue;
                }

                // See if any of the clips for this primitive cull out the item.
                let mut is_valid = true;
                let clip_nodes = &self.scene.clip_nodes[item.clip_nodes_range.start.0 as usize .. item.clip_nodes_range.end.0 as usize];
                for clip_node in clip_nodes {
                    let transform = self
                        .spatial_nodes[clip_node.spatial_node_index.0 as usize]
                        .world_content_transform;
                    let transformed_point = match transform
                        .inverse()
                        .and_then(|inverted| inverted.transform_point2d(test.point))
                    {
                        Some(point) => point,
                        None => {
                            continue;
                        }
                    };
                    if !clip_node.region.contains(&transformed_point) {
                        is_valid = false;
                        break;
                    }
                }
                if !is_valid {
                    continue;
                }

                // Don't hit items with backface-visibility:hidden if they are facing the back.
                if !item.is_backface_visible && scroll_node.world_content_transform.is_backface_visible() {
                    continue;
                }

                // We need to calculate the position of the test point relative to the origin of
                // the pipeline of the hit item. If we cannot get a transformed point, we are
                // in a situation with an uninvertible transformation so we should just skip this
                // result.
                let root_spatial_node_index = self.pipeline_root_nodes[&pipeline_id];
                if root_spatial_node_index != current_root_spatial_node_index {
                    let root_node = &self.spatial_nodes[root_spatial_node_index.0 as usize];
                    point_in_viewport = root_node
                        .world_viewport_transform
                        .inverse()
                        .and_then(|inverted| inverted.transform_point2d(test.point))
                        .map(|pt| pt - scroll_node.external_scroll_offset);

                    current_root_spatial_node_index = root_spatial_node_index;
                }

                if let Some(point_in_viewport) = point_in_viewport {
                    result.items.push(HitTestItem {
                        pipeline: pipeline_id,
                        tag: item.tag,
                        point_in_viewport,
                        point_relative_to_item: point_in_layer - item.rect.origin.to_vector(),
                    });
                }
            }
        }

        result.items.dedup();
        result
    }
}

#[derive(MallocSizeOf)]
pub struct HitTest {
    pipeline_id: Option<PipelineId>,
    point: WorldPoint,
}

impl HitTest {
    pub fn new(
        pipeline_id: Option<PipelineId>,
        point: WorldPoint,
    ) -> HitTest {
        HitTest {
            pipeline_id,
            point,
        }
    }
}

/// Collect clips for a given ClipId, convert and add them to the hit testing
/// scene, if not already present.
fn add_clips(
    clip_id: ClipId,
    clip_store: &ClipStore,
    clip_nodes: &mut Vec<HitTestClipNode>,
    seen_clips: &mut FastHashSet<ClipId>,
    interners: &Interners,
) {
    // If this clip-id has already been added to this hit-test item, skip it
    if seen_clips.contains(&clip_id) {
        return;
    }
    seen_clips.insert(clip_id);

    let template = &clip_store.templates[&clip_id];
    let instances = &clip_store.instances[template.clips.start as usize .. template.clips.end as usize];

    for clip in instances {
        clip_nodes.alloc().init(
            HitTestClipNode::new(
                clip.key.into(),
                clip.clip.spatial_node_index,
                interners,
            )
        );
    }

    // The ClipId parenting is terminated when we reach the root ClipId
    if clip_id != template.parent {
        add_clips(
            template.parent,
            clip_store,
            clip_nodes,
            seen_clips,
            interners,
        );
    }
}
