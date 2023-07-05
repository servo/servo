/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use api::{BorderRadius, ClipMode, HitTestFlags, HitTestItem, HitTestResult, ItemTag, PrimitiveFlags};
use api::{PipelineId, ApiHitTester};
use api::units::*;
use crate::clip::{ClipChainId, ClipDataStore, ClipNode, ClipItemKind, ClipStore};
use crate::clip::{rounded_rectangle_contains_point};
use crate::spatial_tree::{SpatialNodeIndex, SpatialTree};
use crate::internal_types::{FastHashMap, LayoutPrimitiveInfo};
use std::{ops, u32};
use std::sync::{Arc, Mutex};
use crate::util::LayoutToWorldFastTransform;

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
        flags: HitTestFlags
    ) -> HitTestResult {
        self.get_ref().hit_test(HitTest::new(pipeline_id, point, flags))
    }
}

/// A copy of important spatial node data to use during hit testing. This a copy of
/// data from the SpatialTree that will persist as a new frame is under construction,
/// allowing hit tests consistent with the currently rendered frame.
#[derive(MallocSizeOf)]
pub struct HitTestSpatialNode {
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
pub struct HitTestClipNode {
    /// A particular point must be inside all of these regions to be considered clipped in
    /// for the purposes of a hit test.
    region: HitTestRegion,
}

impl HitTestClipNode {
    fn new(node: &ClipNode) -> Self {
        let region = match node.item.kind {
            ClipItemKind::Rectangle { rect, mode } => {
                HitTestRegion::Rectangle(rect, mode)
            }
            ClipItemKind::RoundedRectangle { rect, radius, mode } => {
                HitTestRegion::RoundedRectangle(rect, radius, mode)
            }
            ClipItemKind::Image { rect, .. } => {
                HitTestRegion::Rectangle(rect, ClipMode::Clip)
            }
            ClipItemKind::BoxShadow { .. } => HitTestRegion::Invalid,
        };

        HitTestClipNode {
            region,
        }
    }
}

#[derive(Debug, Copy, Clone, MallocSizeOf, PartialEq, Eq, Hash)]
pub struct HitTestClipChainId(u32);

impl HitTestClipChainId {
    pub const NONE: Self = HitTestClipChainId(u32::MAX);
}

/// A hit testing clip chain node is the same as a
/// normal clip chain node, except that the clip
/// node is embedded inside the clip chain, rather
/// than referenced. This means we don't need to
/// copy the complete interned clip data store for
/// hit testing.
#[derive(MallocSizeOf)]
pub struct HitTestClipChainNode {
    pub region: HitTestClipNode,
    pub spatial_node_index: SpatialNodeIndex,
    pub parent_clip_chain_id: HitTestClipChainId,
}

#[derive(Copy, Clone, Debug, MallocSizeOf)]
pub struct HitTestingClipChainIndex(u32);

#[derive(Clone, MallocSizeOf)]
pub struct HitTestingItem {
    rect: LayoutRect,
    clip_rect: LayoutRect,
    tag: ItemTag,
    is_backface_visible: bool,
    #[ignore_malloc_size_of = "simple"]
    clip_chain_range: ops::Range<HitTestingClipChainIndex>,
    spatial_node_index: SpatialNodeIndex,
}

impl HitTestingItem {
    pub fn new(
        tag: ItemTag,
        info: &LayoutPrimitiveInfo,
        spatial_node_index: SpatialNodeIndex,
        clip_chain_range: ops::Range<HitTestingClipChainIndex>,
    ) -> HitTestingItem {
        HitTestingItem {
            rect: info.rect,
            clip_rect: info.clip_rect,
            tag,
            is_backface_visible: info.flags.contains(PrimitiveFlags::IS_BACKFACE_VISIBLE),
            spatial_node_index,
            clip_chain_range,
        }
    }
}

/// Statistics about allocation sizes of current hit tester,
/// used to pre-allocate size of the next hit tester.
pub struct HitTestingSceneStats {
    pub clip_chain_roots_count: usize,
    pub items_count: usize,
}

impl HitTestingSceneStats {
    pub fn empty() -> Self {
        HitTestingSceneStats {
            clip_chain_roots_count: 0,
            items_count: 0,
        }
    }
}

/// Defines the immutable part of a hit tester for a given scene.
/// The hit tester is recreated each time a frame is built, since
/// it relies on the current values of the spatial tree.
/// However, the clip chain and item definitions don't change,
/// so they are created once per scene, and shared between
/// hit tester instances via Arc.
#[derive(MallocSizeOf)]
pub struct HitTestingScene {
    /// The list of variable clip chain roots referenced by the items.
    pub clip_chain_roots: Vec<HitTestClipChainId>,

    /// List of hit testing primitives.
    pub items: Vec<HitTestingItem>,
}

impl HitTestingScene {
    /// Construct a new hit testing scene, pre-allocating to size
    /// provided by previous scene stats.
    pub fn new(stats: &HitTestingSceneStats) -> Self {
        HitTestingScene {
            clip_chain_roots: Vec::with_capacity(stats.clip_chain_roots_count),
            items: Vec::with_capacity(stats.items_count),
        }
    }

    /// Get stats about the current scene allocation sizes.
    pub fn get_stats(&self) -> HitTestingSceneStats {
        HitTestingSceneStats {
            clip_chain_roots_count: self.clip_chain_roots.len(),
            items_count: self.items.len(),
        }
    }

    /// Add a hit testing primitive.
    pub fn add_item(&mut self, item: HitTestingItem) {
        self.items.push(item);
    }

    /// Add a clip chain to the clip chain roots list.
    pub fn add_clip_chain(&mut self, clip_chain_id: ClipChainId) {
        if clip_chain_id != ClipChainId::INVALID {
            self.clip_chain_roots.push(HitTestClipChainId(clip_chain_id.0));
        }
    }

    /// Get the slice of clip chain roots for a given hit test primitive.
    fn get_clip_chains_for_item(&self, item: &HitTestingItem) -> &[HitTestClipChainId] {
        &self.clip_chain_roots[item.clip_chain_range.start.0 as usize .. item.clip_chain_range.end.0 as usize]
    }

    /// Get the next index of the clip chain roots list.
    pub fn next_clip_chain_index(&self) -> HitTestingClipChainIndex {
        HitTestingClipChainIndex(self.clip_chain_roots.len() as u32)
    }
}

#[derive(MallocSizeOf)]
enum HitTestRegion {
    Invalid,
    Rectangle(LayoutRect, ClipMode),
    RoundedRectangle(LayoutRect, BorderRadius, ClipMode),
}

impl HitTestRegion {
    pub fn contains(&self, point: &LayoutPoint) -> bool {
        match *self {
            HitTestRegion::Rectangle(ref rectangle, ClipMode::Clip) =>
                rectangle.contains(*point),
            HitTestRegion::Rectangle(ref rectangle, ClipMode::ClipOut) =>
                !rectangle.contains(*point),
            HitTestRegion::RoundedRectangle(rect, radii, ClipMode::Clip) =>
                rounded_rectangle_contains_point(point, &rect, &radii),
            HitTestRegion::RoundedRectangle(rect, radii, ClipMode::ClipOut) =>
                !rounded_rectangle_contains_point(point, &rect, &radii),
            HitTestRegion::Invalid => true,
        }
    }
}

#[derive(MallocSizeOf)]
pub struct HitTester {
    #[ignore_malloc_size_of = "Arc"]
    scene: Arc<HitTestingScene>,
    spatial_nodes: Vec<HitTestSpatialNode>,
    clip_chains: Vec<HitTestClipChainNode>,
    pipeline_root_nodes: FastHashMap<PipelineId, SpatialNodeIndex>,
}

impl HitTester {
    pub fn empty() -> Self {
        HitTester {
            scene: Arc::new(HitTestingScene::new(&HitTestingSceneStats::empty())),
            spatial_nodes: Vec::new(),
            clip_chains: Vec::new(),
            pipeline_root_nodes: FastHashMap::default(),
        }
    }

    pub fn new(
        scene: Arc<HitTestingScene>,
        spatial_tree: &SpatialTree,
        clip_store: &ClipStore,
        clip_data_store: &ClipDataStore,
    ) -> HitTester {
        let mut hit_tester = HitTester {
            scene,
            spatial_nodes: Vec::new(),
            clip_chains: Vec::new(),
            pipeline_root_nodes: FastHashMap::default(),
        };
        hit_tester.read_spatial_tree(
            spatial_tree,
            clip_store,
            clip_data_store,
        );
        hit_tester
    }

    fn read_spatial_tree(
        &mut self,
        spatial_tree: &SpatialTree,
        clip_store: &ClipStore,
        clip_data_store: &ClipDataStore,
    ) {
        self.spatial_nodes.clear();
        self.clip_chains.clear();

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

        // For each clip chain node, extract the clip node from the clip
        // data store, and store it inline with the clip chain node.
        self.clip_chains.reserve(clip_store.clip_chain_nodes.len());
        for node in &clip_store.clip_chain_nodes {
            let clip_node = &clip_data_store[node.handle];
            self.clip_chains.push(HitTestClipChainNode {
                region: HitTestClipNode::new(clip_node),
                spatial_node_index: node.spatial_node_index,
                parent_clip_chain_id: HitTestClipChainId(node.parent_clip_chain_id.0),
            });
        }
    }

    fn is_point_clipped_in_for_clip_chain(
        &self,
        point: WorldPoint,
        clip_chain_id: HitTestClipChainId,
        test: &mut HitTest
    ) -> bool {
        if clip_chain_id == HitTestClipChainId::NONE {
            return true;
        }

        if let Some(result) = test.get_from_clip_chain_cache(clip_chain_id) {
            return result == ClippedIn::ClippedIn;
        }

        let descriptor = &self.clip_chains[clip_chain_id.0 as usize];
        let parent_clipped_in = self.is_point_clipped_in_for_clip_chain(
            point,
            descriptor.parent_clip_chain_id,
            test,
        );

        if !parent_clipped_in {
            test.set_in_clip_chain_cache(clip_chain_id, ClippedIn::NotClippedIn);
            return false;
        }

        if !self.is_point_clipped_in_for_clip_node(
            point,
            clip_chain_id,
            descriptor.spatial_node_index,
            test,
        ) {
            test.set_in_clip_chain_cache(clip_chain_id, ClippedIn::NotClippedIn);
            return false;
        }

        test.set_in_clip_chain_cache(clip_chain_id, ClippedIn::ClippedIn);
        true
    }

    fn is_point_clipped_in_for_clip_node(
        &self,
        point: WorldPoint,
        clip_chain_node_id: HitTestClipChainId,
        spatial_node_index: SpatialNodeIndex,
        test: &mut HitTest
    ) -> bool {
        if let Some(clipped_in) = test.node_cache.get(&clip_chain_node_id) {
            return *clipped_in == ClippedIn::ClippedIn;
        }

        let node = &self.clip_chains[clip_chain_node_id.0 as usize].region;
        let transform = self
            .spatial_nodes[spatial_node_index.0 as usize]
            .world_content_transform;
        let transformed_point = match transform
            .inverse()
            .and_then(|inverted| inverted.transform_point2d(point))
        {
            Some(point) => point,
            None => {
                test.node_cache.insert(clip_chain_node_id, ClippedIn::NotClippedIn);
                return false;
            }
        };

        if !node.region.contains(&transformed_point) {
            test.node_cache.insert(clip_chain_node_id, ClippedIn::NotClippedIn);
            return false;
        }

        test.node_cache.insert(clip_chain_node_id, ClippedIn::ClippedIn);
        true
    }

    pub fn find_node_under_point(&self, mut test: HitTest) -> Option<SpatialNodeIndex> {
        let point = test.get_absolute_point(self);
        let mut current_spatial_node_index = SpatialNodeIndex::INVALID;
        let mut point_in_layer = None;

        // For each hit test primitive
        for item in self.scene.items.iter().rev() {
            let scroll_node = &self.spatial_nodes[item.spatial_node_index.0 as usize];

            // Update the cached point in layer space, if the spatial node
            // changed since last primitive.
            if item.spatial_node_index != current_spatial_node_index {
                point_in_layer = scroll_node
                    .world_content_transform
                    .inverse()
                    .and_then(|inverted| inverted.transform_point2d(point));

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

                // See if any of the clip chain roots for this primitive
                // cull out the item.
                let clip_chains = self.scene.get_clip_chains_for_item(item);
                let mut is_valid = true;
                for clip_chain_id in clip_chains {
                    if !self.is_point_clipped_in_for_clip_chain(point, *clip_chain_id, &mut test) {
                        is_valid = false;
                        break;
                    }
                }

                // Found a valid hit test result!
                if is_valid {
                    return Some(item.spatial_node_index);
                }
            }
        }

        None
    }

    pub fn hit_test(&self, mut test: HitTest) -> HitTestResult {
        let point = test.get_absolute_point(self);

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
                    .and_then(|inverted| inverted.transform_point2d(point));
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

                // See if any of the clip chain roots for this primitive
                // cull out the item.
                let clip_chains = self.scene.get_clip_chains_for_item(item);
                let mut is_valid = true;
                for clip_chain_id in clip_chains {
                    if !self.is_point_clipped_in_for_clip_chain(point, *clip_chain_id, &mut test) {
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
                        .and_then(|inverted| inverted.transform_point2d(point))
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

                    if !test.flags.contains(HitTestFlags::FIND_ALL) {
                        return result;
                    }
                }
            }
        }

        result.items.dedup();
        result
    }

    pub fn get_pipeline_root(&self, pipeline_id: PipelineId) -> &HitTestSpatialNode {
        &self.spatial_nodes[self.pipeline_root_nodes[&pipeline_id].0 as usize]
    }
}

#[derive(Clone, Copy, MallocSizeOf, PartialEq)]
enum ClippedIn {
    ClippedIn,
    NotClippedIn,
}

#[derive(MallocSizeOf)]
pub struct HitTest {
    pipeline_id: Option<PipelineId>,
    point: WorldPoint,
    flags: HitTestFlags,
    node_cache: FastHashMap<HitTestClipChainId, ClippedIn>,
    clip_chain_cache: Vec<Option<ClippedIn>>,
}

impl HitTest {
    pub fn new(
        pipeline_id: Option<PipelineId>,
        point: WorldPoint,
        flags: HitTestFlags,
    ) -> HitTest {
        HitTest {
            pipeline_id,
            point,
            flags,
            node_cache: FastHashMap::default(),
            clip_chain_cache: Vec::new(),
        }
    }

    fn get_from_clip_chain_cache(&mut self, index: HitTestClipChainId) -> Option<ClippedIn> {
        let index = index.0 as usize;
        if index >= self.clip_chain_cache.len() {
            None
        } else {
            self.clip_chain_cache[index]
        }
    }

    fn set_in_clip_chain_cache(&mut self, index: HitTestClipChainId, value: ClippedIn) {
        let index = index.0 as usize;
        if index >= self.clip_chain_cache.len() {
            self.clip_chain_cache.resize(index + 1, None);
        }
        self.clip_chain_cache[index] = Some(value);
    }

    fn get_absolute_point(&self, hit_tester: &HitTester) -> WorldPoint {
        if !self.flags.contains(HitTestFlags::POINT_RELATIVE_TO_PIPELINE_VIEWPORT) {
            return self.point;
        }

        let point = LayoutPoint::new(self.point.x, self.point.y);
        self.pipeline_id
            .and_then(|id|
                hit_tester
                    .get_pipeline_root(id)
                    .world_viewport_transform
                    .transform_point2d(point)
            )
            .unwrap_or_else(|| {
                WorldPoint::new(self.point.x, self.point.y)
            })
    }
}
