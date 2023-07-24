/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use api::{ExternalScrollId, PropertyBinding, ReferenceFrameKind, TransformStyle};
use api::{PipelineId, ScrollClamping, ScrollNodeState, ScrollLocation, ScrollSensitivity};
use api::units::*;
use euclid::Transform3D;
use crate::gpu_types::TransformPalette;
use crate::internal_types::{FastHashMap, FastHashSet};
use crate::print_tree::{PrintableTree, PrintTree, PrintTreePrinter};
use crate::scene::SceneProperties;
use crate::spatial_node::{ScrollFrameInfo, SpatialNode, SpatialNodeType, StickyFrameInfo, ScrollFrameKind};
use std::{ops, u32};
use crate::util::{FastTransform, LayoutToWorldFastTransform, MatrixHelpers, ScaleOffset, scale_factors};

pub type ScrollStates = FastHashMap<ExternalScrollId, ScrollFrameInfo>;

/// An id that identifies coordinate systems in the SpatialTree. Each
/// coordinate system has an id and those ids will be shared when the coordinates
/// system are the same or are in the same axis-aligned space. This allows
/// for optimizing mask generation.
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct CoordinateSystemId(pub u32);

/// A node in the hierarchy of coordinate system
/// transforms.
#[derive(Debug)]
pub struct CoordinateSystem {
    pub transform: LayoutTransform,
    pub world_transform: LayoutToWorldTransform,
    pub should_flatten: bool,
    pub parent: Option<CoordinateSystemId>,
}

impl CoordinateSystem {
    fn root() -> Self {
        CoordinateSystem {
            transform: LayoutTransform::identity(),
            world_transform: LayoutToWorldTransform::identity(),
            should_flatten: false,
            parent: None,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, Hash, MallocSizeOf, PartialEq, PartialOrd, Ord)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct SpatialNodeIndex(pub u32);

impl SpatialNodeIndex {
    pub const INVALID: SpatialNodeIndex = SpatialNodeIndex(u32::MAX);
}

//Note: these have to match ROOT_REFERENCE_FRAME_SPATIAL_ID and ROOT_SCROLL_NODE_SPATIAL_ID
pub const ROOT_SPATIAL_NODE_INDEX: SpatialNodeIndex = SpatialNodeIndex(0);
const TOPMOST_SCROLL_NODE_INDEX: SpatialNodeIndex = SpatialNodeIndex(1);

impl SpatialNodeIndex {
    pub fn new(index: usize) -> Self {
        debug_assert!(index < ::std::u32::MAX as usize);
        SpatialNodeIndex(index as u32)
    }
}

impl CoordinateSystemId {
    pub fn root() -> Self {
        CoordinateSystemId(0)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum VisibleFace {
    Front,
    Back,
}

impl Default for VisibleFace {
    fn default() -> Self {
        VisibleFace::Front
    }
}

impl ops::Not for VisibleFace {
    type Output = Self;
    fn not(self) -> Self {
        match self {
            VisibleFace::Front => VisibleFace::Back,
            VisibleFace::Back => VisibleFace::Front,
        }
    }
}

pub struct SpatialTree {
    /// Nodes which determine the positions (offsets and transforms) for primitives
    /// and clips.
    pub spatial_nodes: Vec<SpatialNode>,

    /// A list of transforms that establish new coordinate systems.
    /// Spatial nodes only establish a new coordinate system when
    /// they have a transform that is not a simple 2d translation.
    coord_systems: Vec<CoordinateSystem>,

    pub pending_scroll_offsets: FastHashMap<ExternalScrollId, (LayoutPoint, ScrollClamping)>,

    /// A set of pipelines which should be discarded the next time this
    /// tree is drained.
    pub pipelines_to_discard: FastHashSet<PipelineId>,

    /// Temporary stack of nodes to update when traversing the tree.
    nodes_to_update: Vec<(SpatialNodeIndex, TransformUpdateState)>,
}

#[derive(Clone)]
pub struct TransformUpdateState {
    pub parent_reference_frame_transform: LayoutToWorldFastTransform,
    pub parent_accumulated_scroll_offset: LayoutVector2D,
    pub nearest_scrolling_ancestor_offset: LayoutVector2D,
    pub nearest_scrolling_ancestor_viewport: LayoutRect,

    /// An id for keeping track of the axis-aligned space of this node. This is used in
    /// order to to track what kinds of clip optimizations can be done for a particular
    /// display list item, since optimizations can usually only be done among
    /// coordinate systems which are relatively axis aligned.
    pub current_coordinate_system_id: CoordinateSystemId,

    /// Scale and offset from the coordinate system that started this compatible coordinate system.
    pub coordinate_system_relative_scale_offset: ScaleOffset,

    /// True if this node is transformed by an invertible transform.  If not, display items
    /// transformed by this node will not be displayed and display items not transformed by this
    /// node will not be clipped by clips that are transformed by this node.
    pub invertible: bool,

    /// True if this node is a part of Preserve3D hierarchy.
    pub preserves_3d: bool,
}


/// Transformation between two nodes in the spatial tree that can sometimes be
/// encoded more efficiently than with a full matrix.
#[derive(Debug, Clone)]
pub enum CoordinateSpaceMapping<Src, Dst> {
    Local,
    ScaleOffset(ScaleOffset),
    Transform(Transform3D<f32, Src, Dst>),
}

impl<Src, Dst> CoordinateSpaceMapping<Src, Dst> {
    pub fn into_transform(self) -> Transform3D<f32, Src, Dst> {
        match self {
            CoordinateSpaceMapping::Local => Transform3D::identity(),
            CoordinateSpaceMapping::ScaleOffset(scale_offset) => scale_offset.to_transform(),
            CoordinateSpaceMapping::Transform(transform) => transform,
        }
    }

    pub fn into_fast_transform(self) -> FastTransform<Src, Dst> {
        match self {
            CoordinateSpaceMapping::Local => FastTransform::identity(),
            CoordinateSpaceMapping::ScaleOffset(scale_offset) => FastTransform::with_scale_offset(scale_offset),
            CoordinateSpaceMapping::Transform(transform) => FastTransform::with_transform(transform),
        }
    }

    pub fn visible_face(&self) -> VisibleFace {
        match *self {
            CoordinateSpaceMapping::Transform(ref transform) if transform.is_backface_visible() => VisibleFace::Back,
            CoordinateSpaceMapping::Local |
            CoordinateSpaceMapping::Transform(_) |
            CoordinateSpaceMapping::ScaleOffset(_) => VisibleFace::Front,

        }
    }

    pub fn is_perspective(&self) -> bool {
        match *self {
            CoordinateSpaceMapping::Local |
            CoordinateSpaceMapping::ScaleOffset(_) => false,
            CoordinateSpaceMapping::Transform(ref transform) => transform.has_perspective_component(),
        }
    }

    pub fn is_2d_axis_aligned(&self) -> bool {
        match *self {
            CoordinateSpaceMapping::Local |
            CoordinateSpaceMapping::ScaleOffset(_) => true,
            CoordinateSpaceMapping::Transform(ref transform) => transform.preserves_2d_axis_alignment(),
        }
    }

    pub fn scale_factors(&self) -> (f32, f32) {
        match *self {
            CoordinateSpaceMapping::Local => (1.0, 1.0),
            CoordinateSpaceMapping::ScaleOffset(ref scale_offset) => (scale_offset.scale.x, scale_offset.scale.y),
            CoordinateSpaceMapping::Transform(ref transform) => scale_factors(transform),
        }
    }

    pub fn inverse(&self) -> Option<CoordinateSpaceMapping<Dst, Src>> {
        match *self {
            CoordinateSpaceMapping::Local => Some(CoordinateSpaceMapping::Local),
            CoordinateSpaceMapping::ScaleOffset(ref scale_offset) => {
                Some(CoordinateSpaceMapping::ScaleOffset(scale_offset.inverse()))
            }
            CoordinateSpaceMapping::Transform(ref transform) => {
                transform.inverse().map(CoordinateSpaceMapping::Transform)
            }
        }
    }
}

enum TransformScroll {
    Scrolled,
    Unscrolled,
}

impl SpatialTree {
    pub fn new() -> Self {
        SpatialTree {
            spatial_nodes: Vec::new(),
            coord_systems: Vec::new(),
            pending_scroll_offsets: FastHashMap::default(),
            pipelines_to_discard: FastHashSet::default(),
            nodes_to_update: Vec::new(),
        }
    }

    /// Calculate the accumulated external scroll offset for
    /// a given spatial node.
    pub fn external_scroll_offset(&self, node_index: SpatialNodeIndex) -> LayoutVector2D {
        let mut offset = LayoutVector2D::zero();
        let mut current_node = Some(node_index);

        while let Some(node_index) = current_node {
            let node = &self.spatial_nodes[node_index.0 as usize];

            match node.node_type {
                SpatialNodeType::ScrollFrame(ref scrolling) => {
                    offset += scrolling.external_scroll_offset;
                }
                SpatialNodeType::StickyFrame(..) => {
                    // Doesn't provide any external scroll offset
                }
                SpatialNodeType::ReferenceFrame(..) => {
                    // External scroll offsets are not propagated across
                    // reference frames.
                    break;
                }
            }

            current_node = node.parent;
        }

        offset
    }

    /// Calculate the relative transform from `child_index` to `parent_index`.
    /// This method will panic if the nodes are not connected!
    pub fn get_relative_transform(
        &self,
        child_index: SpatialNodeIndex,
        parent_index: SpatialNodeIndex,
    ) -> CoordinateSpaceMapping<LayoutPixel, LayoutPixel> {
        if child_index == parent_index {
            return CoordinateSpaceMapping::Local;
        }

        let child = &self.spatial_nodes[child_index.0 as usize];
        let parent = &self.spatial_nodes[parent_index.0 as usize];

        if child.coordinate_system_id == parent.coordinate_system_id {
            let scale_offset = parent.content_transform
                .inverse()
                .accumulate(&child.content_transform);
            return CoordinateSpaceMapping::ScaleOffset(scale_offset);
        }

        if child_index.0 < parent_index.0 {
            warn!("Unexpected transform queried from {:?} to {:?}, please call the graphics team!", child_index, parent_index);
            let child_cs = &self.coord_systems[child.coordinate_system_id.0 as usize];
            let child_transform = child.content_transform
                .to_transform::<LayoutPixel, LayoutPixel>()
                .then(&child_cs.world_transform);
            let parent_cs = &self.coord_systems[parent.coordinate_system_id.0 as usize];
            let parent_transform = parent.content_transform
                .to_transform()
                .then(&parent_cs.world_transform);

            let result = parent_transform
                .inverse()
                .unwrap_or_default()
                .then(&child_transform)
                .with_source::<LayoutPixel>()
                .with_destination::<LayoutPixel>();
            return CoordinateSpaceMapping::Transform(result);
        }

        let mut coordinate_system_id = child.coordinate_system_id;
        let mut transform = child.content_transform.to_transform();

        // we need to update the associated parameters of a transform in two cases:
        // 1) when the flattening happens, so that we don't lose that original 3D aspects
        // 2) when we reach the end of iteration, so that our result is up to date

        while coordinate_system_id != parent.coordinate_system_id {
            let coord_system = &self.coord_systems[coordinate_system_id.0 as usize];

            if coord_system.should_flatten {
                transform.flatten_z_output();
            }

            coordinate_system_id = coord_system.parent.expect("invalid parent!");
            transform = transform.then(&coord_system.transform);
        }

        transform = transform.then(
            &parent.content_transform
                .inverse()
                .to_transform(),
        );

        CoordinateSpaceMapping::Transform(transform)
    }

    fn get_world_transform_impl(
        &self,
        index: SpatialNodeIndex,
        scroll: TransformScroll,
    ) -> CoordinateSpaceMapping<LayoutPixel, WorldPixel> {
        let child = &self.spatial_nodes[index.0 as usize];

        if child.coordinate_system_id.0 == 0 {
            if index == ROOT_SPATIAL_NODE_INDEX {
                CoordinateSpaceMapping::Local
            } else {
                CoordinateSpaceMapping::ScaleOffset(child.content_transform)
            }
        } else {
            let system = &self.coord_systems[child.coordinate_system_id.0 as usize];
            let scale_offset = match scroll {
                TransformScroll::Scrolled => &child.content_transform,
                TransformScroll::Unscrolled => &child.viewport_transform,
            };
            let transform = scale_offset
                .to_transform()
                .then(&system.world_transform);

            CoordinateSpaceMapping::Transform(transform)
        }
    }

    /// Calculate the relative transform from `index` to the root.
    pub fn get_world_transform(
        &self,
        index: SpatialNodeIndex,
    ) -> CoordinateSpaceMapping<LayoutPixel, WorldPixel> {
        self.get_world_transform_impl(index, TransformScroll::Scrolled)
    }

    /// Calculate the relative transform from `index` to the root.
    /// Unlike `get_world_transform`, this variant doesn't account for the local scroll offset.
    pub fn get_world_viewport_transform(
        &self,
        index: SpatialNodeIndex,
    ) -> CoordinateSpaceMapping<LayoutPixel, WorldPixel> {
        self.get_world_transform_impl(index, TransformScroll::Unscrolled)
    }

    /// The root reference frame, which is the true root of the SpatialTree. Initially
    /// this ID is not valid, which is indicated by ```spatial_nodes``` being empty.
    pub fn root_reference_frame_index(&self) -> SpatialNodeIndex {
        // TODO(mrobinson): We should eventually make this impossible to misuse.
        debug_assert!(!self.spatial_nodes.is_empty());
        ROOT_SPATIAL_NODE_INDEX
    }

    /// The root scroll node which is the first child of the root reference frame.
    /// Initially this ID is not valid, which is indicated by ```spatial_nodes``` being empty.
    pub fn topmost_scroll_node_index(&self) -> SpatialNodeIndex {
        // TODO(mrobinson): We should eventually make this impossible to misuse.
        debug_assert!(self.spatial_nodes.len() >= 1);
        TOPMOST_SCROLL_NODE_INDEX
    }

    pub fn get_scroll_node_state(&self) -> Vec<ScrollNodeState> {
        let mut result = vec![];
        for node in &self.spatial_nodes {
            if let SpatialNodeType::ScrollFrame(info) = node.node_type {
                if let Some(id) = info.external_id {
                    result.push(ScrollNodeState {
                        id,
                        scroll_offset: info.offset - info.external_scroll_offset,
                    })
                }
            }
        }
        result
    }

    pub fn drain(&mut self) -> ScrollStates {
        let mut scroll_states = FastHashMap::default();
        for old_node in &mut self.spatial_nodes.drain(..) {
            if self.pipelines_to_discard.contains(&old_node.pipeline_id) {
                continue;
            }

            match old_node.node_type {
                SpatialNodeType::ScrollFrame(info) if info.external_id.is_some() => {
                    scroll_states.insert(info.external_id.unwrap(), info);
                }
                _ => {}
            }
        }

        self.coord_systems.clear();
        self.pipelines_to_discard.clear();
        scroll_states
    }

    pub fn scroll_node(
        &mut self,
        origin: LayoutPoint,
        id: ExternalScrollId,
        clamp: ScrollClamping
    ) -> bool {
        for node in &mut self.spatial_nodes {
            if node.matches_external_id(id) {
                return node.set_scroll_origin(&origin, clamp);
            }
        }

        self.pending_scroll_offsets.insert(id, (origin, clamp));
        false
    }

    fn find_nearest_scrolling_ancestor(
        &self,
        index: Option<SpatialNodeIndex>
    ) -> SpatialNodeIndex {
        let index = match index {
            Some(index) => index,
            None => return self.topmost_scroll_node_index(),
        };

        let node = &self.spatial_nodes[index.0 as usize];
        match node.node_type {
            SpatialNodeType::ScrollFrame(state) if state.sensitive_to_input_events() => index,
            _ => self.find_nearest_scrolling_ancestor(node.parent)
        }
    }

    pub fn scroll_nearest_scrolling_ancestor(
        &mut self,
        scroll_location: ScrollLocation,
        node_index: Option<SpatialNodeIndex>,
    ) -> bool {
        if self.spatial_nodes.is_empty() {
            return false;
        }
        let node_index = self.find_nearest_scrolling_ancestor(node_index);
        self.spatial_nodes[node_index.0 as usize].scroll(scroll_location)
    }

    pub fn update_tree(
        &mut self,
        pan: WorldPoint,
        global_device_pixel_scale: DevicePixelScale,
        scene_properties: &SceneProperties,
    ) {
        if self.spatial_nodes.is_empty() {
            return;
        }

        profile_scope!("update_tree");
        self.coord_systems.clear();
        self.coord_systems.push(CoordinateSystem::root());

        let root_node_index = self.root_reference_frame_index();
        let state = TransformUpdateState {
            parent_reference_frame_transform: LayoutVector2D::new(pan.x, pan.y).into(),
            parent_accumulated_scroll_offset: LayoutVector2D::zero(),
            nearest_scrolling_ancestor_offset: LayoutVector2D::zero(),
            nearest_scrolling_ancestor_viewport: LayoutRect::zero(),
            current_coordinate_system_id: CoordinateSystemId::root(),
            coordinate_system_relative_scale_offset: ScaleOffset::identity(),
            invertible: true,
            preserves_3d: false,
        };
        debug_assert!(self.nodes_to_update.is_empty());
        self.nodes_to_update.push((root_node_index, state));

        while let Some((node_index, mut state)) = self.nodes_to_update.pop() {
            let (previous, following) = self.spatial_nodes.split_at_mut(node_index.0 as usize);
            let node = match following.get_mut(0) {
                Some(node) => node,
                None => continue,
            };

            node.update(&mut state, &mut self.coord_systems, global_device_pixel_scale, scene_properties, &*previous);

            if !node.children.is_empty() {
                node.prepare_state_for_children(&mut state);
                self.nodes_to_update.extend(node.children
                    .iter()
                    .rev()
                    .map(|child_index| (*child_index, state.clone()))
                );
            }
        }
    }

    pub fn build_transform_palette(&self) -> TransformPalette {
        profile_scope!("build_transform_palette");
        let mut palette = TransformPalette::new(self.spatial_nodes.len());
        //Note: getting the world transform of a node is O(1) operation
        for i in 0 .. self.spatial_nodes.len() {
            let index = SpatialNodeIndex(i as u32);
            let world_transform = self.get_world_transform(index).into_transform();
            palette.set_world_transform(index, world_transform);
        }
        palette
    }

    pub fn finalize_and_apply_pending_scroll_offsets(&mut self, old_states: ScrollStates) {
        for node in &mut self.spatial_nodes {
            let external_id = match node.node_type {
                SpatialNodeType::ScrollFrame(ScrollFrameInfo { external_id: Some(id), ..} ) => id,
                _ => continue,
            };

            if let Some(scrolling_state) = old_states.get(&external_id) {
                node.apply_old_scrolling_state(scrolling_state);
            }

            if let Some((offset, clamping)) = self.pending_scroll_offsets.remove(&external_id) {
                node.set_scroll_origin(&offset, clamping);
            }
        }
    }

    pub fn add_scroll_frame(
        &mut self,
        parent_index: SpatialNodeIndex,
        external_id: Option<ExternalScrollId>,
        pipeline_id: PipelineId,
        frame_rect: &LayoutRect,
        content_size: &LayoutSize,
        scroll_sensitivity: ScrollSensitivity,
        frame_kind: ScrollFrameKind,
        external_scroll_offset: LayoutVector2D,
    ) -> SpatialNodeIndex {
        let node = SpatialNode::new_scroll_frame(
            pipeline_id,
            parent_index,
            external_id,
            frame_rect,
            content_size,
            scroll_sensitivity,
            frame_kind,
            external_scroll_offset,
        );
        self.add_spatial_node(node)
    }

    pub fn add_reference_frame(
        &mut self,
        parent_index: Option<SpatialNodeIndex>,
        transform_style: TransformStyle,
        source_transform: PropertyBinding<LayoutTransform>,
        kind: ReferenceFrameKind,
        origin_in_parent_reference_frame: LayoutVector2D,
        pipeline_id: PipelineId,
    ) -> SpatialNodeIndex {
        let node = SpatialNode::new_reference_frame(
            parent_index,
            transform_style,
            source_transform,
            kind,
            origin_in_parent_reference_frame,
            pipeline_id,
        );
        self.add_spatial_node(node)
    }

    pub fn add_sticky_frame(
        &mut self,
        parent_index: SpatialNodeIndex,
        sticky_frame_info: StickyFrameInfo,
        pipeline_id: PipelineId,
    ) -> SpatialNodeIndex {
        let node = SpatialNode::new_sticky_frame(
            parent_index,
            sticky_frame_info,
            pipeline_id,
        );
        self.add_spatial_node(node)
    }

    pub fn add_spatial_node(&mut self, mut node: SpatialNode) -> SpatialNodeIndex {
        let index = SpatialNodeIndex::new(self.spatial_nodes.len());

        // When the parent node is None this means we are adding the root.
        if let Some(parent_index) = node.parent {
            let parent_node = &mut self.spatial_nodes[parent_index.0 as usize];
            parent_node.add_child(index);
            node.update_snapping(Some(parent_node));
        } else {
            node.update_snapping(None);
        }

        self.spatial_nodes.push(node);
        index
    }

    pub fn discard_frame_state_for_pipeline(&mut self, pipeline_id: PipelineId) {
        self.pipelines_to_discard.insert(pipeline_id);
    }

    /// Find the spatial node that is the scroll root for a given spatial node.
    /// A scroll root is the first spatial node when found travelling up the
    /// spatial node tree that is an explicit scroll frame.
    pub fn find_scroll_root(
        &self,
        spatial_node_index: SpatialNodeIndex,
    ) -> SpatialNodeIndex {
        let mut scroll_root = ROOT_SPATIAL_NODE_INDEX;
        let mut node_index = spatial_node_index;

        while node_index != ROOT_SPATIAL_NODE_INDEX {
            let node = &self.spatial_nodes[node_index.0 as usize];
            match node.node_type {
                SpatialNodeType::ReferenceFrame(ref info) => {
                    match info.kind {
                        ReferenceFrameKind::Zoom => {
                            // We can handle scroll nodes that pass through a zoom node
                        }
                        ReferenceFrameKind::Transform |
                        ReferenceFrameKind::Perspective { .. } => {
                            // When a reference frame is encountered, forget any scroll roots
                            // we have encountered, as they may end up with a non-axis-aligned transform.
                            scroll_root = ROOT_SPATIAL_NODE_INDEX;
                        }
                    }
                }
                SpatialNodeType::StickyFrame(..) => {}
                SpatialNodeType::ScrollFrame(ref info) => {
                    match info.frame_kind {
                        ScrollFrameKind::PipelineRoot => {
                            // Once we encounter a pipeline root, there is no need to look further
                            break;
                        }
                        ScrollFrameKind::Explicit => {
                            // If the scroll root has no scrollable area, we don't want to
                            // consider it. This helps pages that have a nested scroll root
                            // within a redundant scroll root to avoid selecting the wrong
                            // reference spatial node for a picture cache.
                            if info.scrollable_size.width > 0.0 ||
                               info.scrollable_size.height > 0.0 {
                                // Since we are skipping redundant scroll roots, we may end up
                                // selecting inner scroll roots that are very small. There is
                                // no performance benefit to creating a slice for these roots,
                                // as they are cheap to rasterize. The size comparison is in
                                // local-space, but makes for a reasonable estimate. The value
                                // is arbitrary, but is generally small enough to ignore things
                                // like scroll roots around text input elements.
                                if info.viewport_rect.size.width > 128.0 &&
                                   info.viewport_rect.size.height > 128.0 {
                                    // If we've found a root that is scrollable, and a reasonable
                                    // size, select that as the current root for this node
                                    scroll_root = node_index;
                                }
                            }
                        }
                    }
                }
            }
            node_index = node.parent.expect("unable to find parent node");
        }

        scroll_root
    }

    fn print_node<T: PrintTreePrinter>(
        &self,
        index: SpatialNodeIndex,
        pt: &mut T,
    ) {
        let node = &self.spatial_nodes[index.0 as usize];
        match node.node_type {
            SpatialNodeType::StickyFrame(ref sticky_frame_info) => {
                pt.new_level(format!("StickyFrame"));
                pt.add_item(format!("sticky info: {:?}", sticky_frame_info));
            }
            SpatialNodeType::ScrollFrame(scrolling_info) => {
                pt.new_level(format!("ScrollFrame"));
                pt.add_item(format!("viewport: {:?}", scrolling_info.viewport_rect));
                pt.add_item(format!("scrollable_size: {:?}", scrolling_info.scrollable_size));
                pt.add_item(format!("scroll offset: {:?}", scrolling_info.offset));
                pt.add_item(format!("external_scroll_offset: {:?}", scrolling_info.external_scroll_offset));
                pt.add_item(format!("kind: {:?}", scrolling_info.frame_kind));
            }
            SpatialNodeType::ReferenceFrame(ref info) => {
                pt.new_level(format!("ReferenceFrame"));
                pt.add_item(format!("kind: {:?}", info.kind));
                pt.add_item(format!("transform_style: {:?}", info.transform_style));
                pt.add_item(format!("source_transform: {:?}", info.source_transform));
                pt.add_item(format!("origin_in_parent_reference_frame: {:?}", info.origin_in_parent_reference_frame));
            }
        }

        pt.add_item(format!("index: {:?}", index));
        pt.add_item(format!("content_transform: {:?}", node.content_transform));
        pt.add_item(format!("viewport_transform: {:?}", node.viewport_transform));
        pt.add_item(format!("snapping_transform: {:?}", node.snapping_transform));
        pt.add_item(format!("coordinate_system_id: {:?}", node.coordinate_system_id));

        for child_index in &node.children {
            self.print_node(*child_index, pt);
        }

        pt.end_level();
    }

    /// Get the visible face of the transfrom from the specified node to its parent.
    pub fn get_local_visible_face(&self, node_index: SpatialNodeIndex) -> VisibleFace {
        let node = &self.spatial_nodes[node_index.0 as usize];
        let parent_index = match node.parent {
            Some(index) => index,
            None => return VisibleFace::Front
        };
        self.get_relative_transform(node_index, parent_index)
            .visible_face()
    }

    #[allow(dead_code)]
    pub fn print(&self) {
        if !self.spatial_nodes.is_empty() {
            let mut buf = Vec::<u8>::new();
            {
                let mut pt = PrintTree::new_with_sink("spatial tree", &mut buf);
                self.print_with(&mut pt);
            }
            // If running in Gecko, set RUST_LOG=webrender::spatial_tree=debug
            // to get this logging to be emitted to stderr/logcat.
            debug!("{}", std::str::from_utf8(&buf).unwrap_or("(Tree printer emitted non-utf8)"));
        }
    }
}

impl PrintableTree for SpatialTree {
    fn print_with<T: PrintTreePrinter>(&self, pt: &mut T) {
        if !self.spatial_nodes.is_empty() {
            self.print_node(self.root_reference_frame_index(), pt);
        }
    }
}

#[cfg(test)]
fn add_reference_frame(
    cst: &mut SpatialTree,
    parent: Option<SpatialNodeIndex>,
    transform: LayoutTransform,
    origin_in_parent_reference_frame: LayoutVector2D,
) -> SpatialNodeIndex {
    cst.add_reference_frame(
        parent,
        TransformStyle::Preserve3D,
        PropertyBinding::Value(transform),
        ReferenceFrameKind::Transform,
        origin_in_parent_reference_frame,
        PipelineId::dummy(),
    )
}

#[cfg(test)]
fn test_pt(
    px: f32,
    py: f32,
    cst: &SpatialTree,
    child: SpatialNodeIndex,
    parent: SpatialNodeIndex,
    expected_x: f32,
    expected_y: f32,
) {
    use euclid::approxeq::ApproxEq;
    const EPSILON: f32 = 0.0001;

    let p = LayoutPoint::new(px, py);
    let m = cst.get_relative_transform(child, parent).into_transform();
    let pt = m.transform_point2d(p).unwrap();
    assert!(pt.x.approx_eq_eps(&expected_x, &EPSILON) &&
            pt.y.approx_eq_eps(&expected_y, &EPSILON),
            "p: {:?} -> {:?}\nm={:?}",
            p, pt, m,
            );
}

#[test]
fn test_cst_simple_translation() {
    // Basic translations only

    let mut cst = SpatialTree::new();

    let root = add_reference_frame(
        &mut cst,
        None,
        LayoutTransform::identity(),
        LayoutVector2D::zero(),
    );

    let child1 = add_reference_frame(
        &mut cst,
        Some(root),
        LayoutTransform::translation(100.0, 0.0, 0.0),
        LayoutVector2D::zero(),
    );

    let child2 = add_reference_frame(
        &mut cst,
        Some(child1),
        LayoutTransform::translation(0.0, 50.0, 0.0),
        LayoutVector2D::zero(),
    );

    let child3 = add_reference_frame(
        &mut cst,
        Some(child2),
        LayoutTransform::translation(200.0, 200.0, 0.0),
        LayoutVector2D::zero(),
    );

    cst.update_tree(WorldPoint::zero(), DevicePixelScale::new(1.0), &SceneProperties::new());

    test_pt(100.0, 100.0, &cst, child1, root, 200.0, 100.0);
    test_pt(100.0, 100.0, &cst, child2, root, 200.0, 150.0);
    test_pt(100.0, 100.0, &cst, child2, child1, 100.0, 150.0);
    test_pt(100.0, 100.0, &cst, child3, root, 400.0, 350.0);
}

#[test]
fn test_cst_simple_scale() {
    // Basic scale only

    let mut cst = SpatialTree::new();

    let root = add_reference_frame(
        &mut cst,
        None,
        LayoutTransform::identity(),
        LayoutVector2D::zero(),
    );

    let child1 = add_reference_frame(
        &mut cst,
        Some(root),
        LayoutTransform::scale(4.0, 1.0, 1.0),
        LayoutVector2D::zero(),
    );

    let child2 = add_reference_frame(
        &mut cst,
        Some(child1),
        LayoutTransform::scale(1.0, 2.0, 1.0),
        LayoutVector2D::zero(),
    );

    let child3 = add_reference_frame(
        &mut cst,
        Some(child2),
        LayoutTransform::scale(2.0, 2.0, 1.0),
        LayoutVector2D::zero(),
    );

    cst.update_tree(WorldPoint::zero(), DevicePixelScale::new(1.0), &SceneProperties::new());

    test_pt(100.0, 100.0, &cst, child1, root, 400.0, 100.0);
    test_pt(100.0, 100.0, &cst, child2, root, 400.0, 200.0);
    test_pt(100.0, 100.0, &cst, child3, root, 800.0, 400.0);
    test_pt(100.0, 100.0, &cst, child2, child1, 100.0, 200.0);
    test_pt(100.0, 100.0, &cst, child3, child1, 200.0, 400.0);
}

#[test]
fn test_cst_scale_translation() {
    // Scale + translation

    let mut cst = SpatialTree::new();

    let root = add_reference_frame(
        &mut cst,
        None,
        LayoutTransform::identity(),
        LayoutVector2D::zero(),
    );

    let child1 = add_reference_frame(
        &mut cst,
        Some(root),
        LayoutTransform::translation(100.0, 50.0, 0.0),
        LayoutVector2D::zero(),
    );

    let child2 = add_reference_frame(
        &mut cst,
        Some(child1),
        LayoutTransform::scale(2.0, 4.0, 1.0),
        LayoutVector2D::zero(),
    );

    let child3 = add_reference_frame(
        &mut cst,
        Some(child2),
        LayoutTransform::translation(200.0, -100.0, 0.0),
        LayoutVector2D::zero(),
    );

    let child4 = add_reference_frame(
        &mut cst,
        Some(child3),
        LayoutTransform::scale(3.0, 2.0, 1.0),
        LayoutVector2D::zero(),
    );

    cst.update_tree(WorldPoint::zero(), DevicePixelScale::new(1.0), &SceneProperties::new());

    test_pt(100.0, 100.0, &cst, child1, root, 200.0, 150.0);
    test_pt(100.0, 100.0, &cst, child2, root, 300.0, 450.0);
    test_pt(100.0, 100.0, &cst, child4, root, 1100.0, 450.0);

    test_pt(0.0, 0.0, &cst, child4, child1, 400.0, -400.0);
    test_pt(100.0, 100.0, &cst, child4, child1, 1000.0, 400.0);
    test_pt(100.0, 100.0, &cst, child2, child1, 200.0, 400.0);

    test_pt(100.0, 100.0, &cst, child3, child1, 600.0, 0.0);
}

#[test]
fn test_cst_translation_rotate() {
    // Rotation + translation
    use euclid::Angle;

    let mut cst = SpatialTree::new();

    let root = add_reference_frame(
        &mut cst,
        None,
        LayoutTransform::identity(),
        LayoutVector2D::zero(),
    );

    let child1 = add_reference_frame(
        &mut cst,
        Some(root),
        LayoutTransform::rotation(0.0, 0.0, 1.0, Angle::degrees(-90.0)),
        LayoutVector2D::zero(),
    );

    cst.update_tree(WorldPoint::zero(), DevicePixelScale::new(1.0), &SceneProperties::new());

    test_pt(100.0, 0.0, &cst, child1, root, 0.0, -100.0);
}
