/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Defines data structures which are consumed by `Paint`.

use std::cell::Cell;
use std::collections::HashMap;

use base::Epoch;
use base::id::ScrollTreeNodeId;
use base::print_tree::PrintTree;
use bitflags::bitflags;
use embedder_traits::ViewportDetails;
use euclid::SideOffsets2D;
use malloc_size_of_derive::MallocSizeOf;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use servo_geometry::FastLayoutTransform;
use style::values::specified::Overflow;
use webrender_api::units::{LayoutPixel, LayoutPoint, LayoutRect, LayoutSize, LayoutVector2D};
use webrender_api::{
    ExternalScrollId, PipelineId, ReferenceFrameKind, ScrollLocation, SpatialId,
    StickyOffsetBounds, TransformStyle,
};

/// A scroll type, describing whether what kind of action originated this scroll request.
/// This is a bitflag as it is also used to track what kinds of [`ScrollType`]s scroll
/// nodes are sensitive to.
#[derive(Clone, Copy, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub struct ScrollType(u8);

bitflags! {
    impl ScrollType: u8 {
        /// This node can be scrolled by input events or an input event originated this
        /// scroll.
        const InputEvents = 1 << 0;
        /// This node can be scrolled by script events or script originated this scroll.
        const Script = 1 << 1;
    }
}

/// Convert [Overflow] to [ScrollType].
impl From<Overflow> for ScrollType {
    fn from(overflow: Overflow) -> Self {
        match overflow {
            Overflow::Hidden => ScrollType::Script,
            Overflow::Scroll | Overflow::Auto => ScrollType::Script | ScrollType::InputEvents,
            Overflow::Visible | Overflow::Clip => ScrollType::empty(),
        }
    }
}

/// The [ScrollType] of particular node in the vertical and horizontal axes.
#[derive(Clone, Copy, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub struct AxesScrollSensitivity {
    pub x: ScrollType,
    pub y: ScrollType,
}

#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
pub enum SpatialTreeNodeInfo {
    ReferenceFrame(ReferenceFrameNodeInfo),
    Scroll(ScrollableNodeInfo),
    Sticky(StickyNodeInfo),
}

#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct StickyNodeInfo {
    pub frame_rect: LayoutRect,
    pub margins: SideOffsets2D<Option<f32>, LayoutPixel>,
    pub vertical_offset_bounds: StickyOffsetBounds,
    pub horizontal_offset_bounds: StickyOffsetBounds,
}

impl StickyNodeInfo {
    /// Calculate the sticky offset for this [`StickyNodeInfo`] given information about
    /// sticky positioning from its ancestors.
    ///
    /// This is originally taken from WebRender `SpatialTree` implementation.
    fn calculate_sticky_offset(
        &self,
        viewport_scroll_offset: &LayoutVector2D,
        viewport_rect: &LayoutRect,
    ) -> LayoutVector2D {
        if self.margins.top.is_none() &&
            self.margins.bottom.is_none() &&
            self.margins.left.is_none() &&
            self.margins.right.is_none()
        {
            return LayoutVector2D::zero();
        }

        // The viewport and margins of the item establishes the maximum amount that it can
        // be offset in order to keep it on screen. Since we care about the relationship
        // between the scrolled content and unscrolled viewport we adjust the viewport's
        // position by the scroll offset in order to work with their relative positions on the
        // page.
        let mut sticky_rect = self.frame_rect.translate(*viewport_scroll_offset);

        let mut sticky_offset = LayoutVector2D::zero();
        if let Some(margin) = self.margins.top {
            let top_viewport_edge = viewport_rect.min.y + margin;
            if sticky_rect.min.y < top_viewport_edge {
                // If the sticky rect is positioned above the top edge of the viewport (plus margin)
                // we move it down so that it is fully inside the viewport.
                sticky_offset.y = top_viewport_edge - sticky_rect.min.y;
            }
        }

        // If we don't have a sticky-top offset (sticky_offset.y == 0) then we check for
        // handling the bottom margin case. Note that the "don't have a sticky-top offset"
        // case includes the case where we *had* a sticky-top offset but we reduced it to
        // zero in the above block.
        if sticky_offset.y <= 0.0 {
            if let Some(margin) = self.margins.bottom {
                // If sticky_offset.y is nonzero that means we must have set it
                // in the sticky-top handling code above, so this item must have
                // both top and bottom sticky margins. We adjust the item's rect
                // by the top-sticky offset, and then combine any offset from
                // the bottom-sticky calculation into sticky_offset below.
                sticky_rect.min.y += sticky_offset.y;
                sticky_rect.max.y += sticky_offset.y;

                // Same as the above case, but inverted for bottom-sticky items. Here
                // we adjust items upwards, resulting in a negative sticky_offset.y,
                // or reduce the already-present upward adjustment, resulting in a positive
                // sticky_offset.y.
                let bottom_viewport_edge = viewport_rect.max.y - margin;
                if sticky_rect.max.y > bottom_viewport_edge {
                    sticky_offset.y += bottom_viewport_edge - sticky_rect.max.y;
                }
            }
        }

        // Same as above, but for the x-axis.
        if let Some(margin) = self.margins.left {
            let left_viewport_edge = viewport_rect.min.x + margin;
            if sticky_rect.min.x < left_viewport_edge {
                sticky_offset.x = left_viewport_edge - sticky_rect.min.x;
            }
        }

        if sticky_offset.x <= 0.0 {
            if let Some(margin) = self.margins.right {
                sticky_rect.min.x += sticky_offset.x;
                sticky_rect.max.x += sticky_offset.x;
                let right_viewport_edge = viewport_rect.max.x - margin;
                if sticky_rect.max.x > right_viewport_edge {
                    sticky_offset.x += right_viewport_edge - sticky_rect.max.x;
                }
            }
        }

        // The total "sticky offset" and the extra amount we computed as a result of
        // scrolling, stored in sticky_offset needs to be clamped to the provided bounds.
        let clamp =
            |value: f32, bounds: &StickyOffsetBounds| (value).max(bounds.min).min(bounds.max);
        sticky_offset.y = clamp(sticky_offset.y, &self.vertical_offset_bounds);
        sticky_offset.x = clamp(sticky_offset.x, &self.horizontal_offset_bounds);

        sticky_offset
    }
}

#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct ReferenceFrameNodeInfo {
    pub origin: LayoutPoint,
    /// Origin of this frame relative to the document for bounding box queries.
    pub frame_origin_for_query: LayoutPoint,
    pub transform_style: TransformStyle,
    pub transform: FastLayoutTransform,
    pub kind: ReferenceFrameKind,
}

/// Data stored for nodes in the [ScrollTree] that actually scroll,
/// as opposed to reference frames and sticky nodes which do not.
#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct ScrollableNodeInfo {
    /// The external scroll id of this node, used to track
    /// it between successive re-layouts.
    pub external_id: ExternalScrollId,

    /// The content rectangle for this scroll node;
    pub content_rect: LayoutRect,

    /// The clip rectange for this scroll node.
    pub clip_rect: LayoutRect,

    /// Whether this `ScrollableNode` is sensitive to input events.
    pub scroll_sensitivity: AxesScrollSensitivity,

    /// The current offset of this scroll node.
    pub offset: LayoutVector2D,

    /// Whether or not the scroll offset of this node has changed and it needs it's
    /// cached transformations invalidated.
    pub offset_changed: Cell<bool>,
}

impl ScrollableNodeInfo {
    fn scroll_to_offset(
        &mut self,
        new_offset: LayoutVector2D,
        context: ScrollType,
    ) -> Option<LayoutVector2D> {
        if !self.scroll_sensitivity.x.contains(context) &&
            !self.scroll_sensitivity.y.contains(context)
        {
            return None;
        }

        let scrollable_size = self.scrollable_size();
        let original_layer_scroll_offset = self.offset;

        if scrollable_size.width > 0. && self.scroll_sensitivity.x.contains(context) {
            self.offset.x = new_offset.x.clamp(0.0, scrollable_size.width);
        }

        if scrollable_size.height > 0. && self.scroll_sensitivity.y.contains(context) {
            self.offset.y = new_offset.y.clamp(0.0, scrollable_size.height);
        }

        if self.offset != original_layer_scroll_offset {
            self.offset_changed.set(true);
            Some(self.offset)
        } else {
            None
        }
    }

    fn scroll_to_webrender_location(
        &mut self,
        scroll_location: ScrollLocation,
        context: ScrollType,
    ) -> Option<LayoutVector2D> {
        if !self.scroll_sensitivity.x.contains(context) &&
            !self.scroll_sensitivity.y.contains(context)
        {
            return None;
        }

        let delta = match scroll_location {
            ScrollLocation::Delta(delta) => delta,
            ScrollLocation::Start => {
                if self.offset.y.round() <= 0.0 {
                    // Nothing to do on this layer.
                    return None;
                }

                self.offset.y = 0.0;
                self.offset_changed.set(true);
                return Some(self.offset);
            },
            ScrollLocation::End => {
                let end_pos = self.scrollable_size().height;
                if self.offset.y.round() >= end_pos {
                    // Nothing to do on this layer.
                    return None;
                }

                self.offset.y = end_pos;
                self.offset_changed.set(true);
                return Some(self.offset);
            },
        };

        self.scroll_to_offset(self.offset + delta, context)
    }
}

impl ScrollableNodeInfo {
    fn scrollable_size(&self) -> LayoutSize {
        self.content_rect.size() - self.clip_rect.size()
    }
}

/// A cached of transforms of a particular [`ScrollTree`] node in both directions:
/// mapping from node-relative points to root-relative points and vice-versa.
///
/// Potential ideas for improvement:
///  - Test optimizing simple translations to avoid having to do full matrix
///    multiplication when transforms are not involved.
#[derive(Clone, Copy, Debug, Default, Deserialize, MallocSizeOf, Serialize)]
pub struct ScrollTreeNodeTransformationCache {
    node_to_root_transform: FastLayoutTransform,
    root_to_node_transform: Option<FastLayoutTransform>,
    nearest_scrolling_ancestor_offset: LayoutVector2D,
    nearest_scrolling_ancestor_viewport: LayoutRect,
    cumulative_sticky_offsets: LayoutVector2D,
}

#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
/// A node in a tree of scroll nodes. This may either be a scrollable
/// node which responds to scroll events or a non-scrollable one.
pub struct ScrollTreeNode {
    /// The index of the parent of this node in the tree. If this is
    /// None then this is the root node.
    pub parent: Option<ScrollTreeNodeId>,

    /// The children of this [`ScrollTreeNode`].
    pub children: Vec<ScrollTreeNodeId>,

    /// The WebRender id, which is filled in when this tree is serialiezd
    /// into a WebRender display list.
    pub webrender_id: Option<SpatialId>,

    /// Specific information about this node, depending on whether it is a scroll node
    /// or a reference frame.
    pub info: SpatialTreeNodeInfo,

    /// Cached transformation information that's used to do things like hit testing
    /// and viewport bounding box calculation.
    transformation_cache: Cell<Option<ScrollTreeNodeTransformationCache>>,
}

impl ScrollTreeNode {
    /// Get the WebRender [`SpatialId`] for the given [`ScrollNodeId`]. This will
    /// panic if [`ScrollTree::build_display_list`] has not been called yet.
    pub fn webrender_id(&self) -> SpatialId {
        self.webrender_id
            .expect("Should have called ScrollTree::build_display_list before querying SpatialId")
    }

    /// Get the external id of this node.
    pub fn external_id(&self) -> Option<ExternalScrollId> {
        match self.info {
            SpatialTreeNodeInfo::Scroll(ref info) => Some(info.external_id),
            _ => None,
        }
    }

    /// Get the offset id of this node if it applies.
    pub fn offset(&self) -> Option<LayoutVector2D> {
        match self.info {
            SpatialTreeNodeInfo::Scroll(ref info) => Some(info.offset),
            _ => None,
        }
    }

    /// Scroll this node given a WebRender ScrollLocation. Returns a tuple that can
    /// be used to scroll an individual WebRender scroll frame if the operation
    /// actually changed an offset.
    fn scroll(
        &mut self,
        scroll_location: ScrollLocation,
        context: ScrollType,
    ) -> Option<(ExternalScrollId, LayoutVector2D)> {
        let SpatialTreeNodeInfo::Scroll(ref mut info) = self.info else {
            return None;
        };

        info.scroll_to_webrender_location(scroll_location, context)
            .map(|location| (info.external_id, location))
    }

    pub fn debug_print(&self, print_tree: &mut PrintTree, node_index: usize) {
        match &self.info {
            SpatialTreeNodeInfo::ReferenceFrame(info) => {
                print_tree.new_level(format!(
                    "Reference Frame({node_index}): webrender_id={:?}\
                        \norigin: {:?}\
                        \ntransform_style: {:?}\
                        \ntransform: {:?}\
                        \nkind: {:?}",
                    self.webrender_id, info.origin, info.transform_style, info.transform, info.kind,
                ));
            },
            SpatialTreeNodeInfo::Scroll(info) => {
                print_tree.new_level(format!(
                    "Scroll Frame({node_index}): webrender_id={:?}\
                        \nexternal_id: {:?}\
                        \ncontent_rect: {:?}\
                        \nclip_rect: {:?}\
                        \nscroll_sensitivity: {:?}\
                        \noffset: {:?}",
                    self.webrender_id,
                    info.external_id,
                    info.content_rect,
                    info.clip_rect,
                    info.scroll_sensitivity,
                    info.offset,
                ));
            },
            SpatialTreeNodeInfo::Sticky(info) => {
                print_tree.new_level(format!(
                    "Sticky Frame({node_index}): webrender_id={:?}\
                        \nframe_rect: {:?}\
                        \nmargins: {:?}\
                        \nhorizontal_offset_bounds: {:?}\
                        \nvertical_offset_bounds: {:?}",
                    self.webrender_id,
                    info.frame_rect,
                    info.margins,
                    info.horizontal_offset_bounds,
                    info.vertical_offset_bounds,
                ));
            },
        };
    }

    fn invalidate_cached_transforms(&self, scroll_tree: &ScrollTree, ancestors_invalid: bool) {
        let node_invalid = match &self.info {
            SpatialTreeNodeInfo::Scroll(info) => info.offset_changed.take(),
            _ => false,
        };

        let invalid = node_invalid || ancestors_invalid;
        if invalid {
            self.transformation_cache.set(None);
        }

        for child_id in &self.children {
            scroll_tree
                .get_node(*child_id)
                .invalidate_cached_transforms(scroll_tree, invalid);
        }
    }
}

/// A tree of spatial nodes, which mirrors the spatial nodes in the WebRender
/// display list, except these are used for scrolling in `Paint` so that
/// new offsets can be sent to WebRender.
#[derive(Clone, Debug, Default, Deserialize, MallocSizeOf, Serialize)]
pub struct ScrollTree {
    /// A list of `Paint`-side scroll nodes that describe the tree
    /// of WebRender spatial nodes, used by `Paint` to scroll the
    /// contents of the display list.
    pub nodes: Vec<ScrollTreeNode>,
}

impl ScrollTree {
    /// Add a scroll node to this ScrollTree returning the id of the new node.
    pub fn add_scroll_tree_node(
        &mut self,
        parent: Option<ScrollTreeNodeId>,
        info: SpatialTreeNodeInfo,
    ) -> ScrollTreeNodeId {
        self.nodes.push(ScrollTreeNode {
            parent,
            children: Vec::new(),
            webrender_id: None,
            info,
            transformation_cache: Cell::default(),
        });

        let new_node_id = ScrollTreeNodeId {
            index: self.nodes.len() - 1,
        };

        if let Some(parent_id) = parent {
            self.get_node_mut(parent_id).children.push(new_node_id);
        }

        new_node_id
    }

    /// Once WebRender display list construction is complete for this [`ScrollTree`], update
    /// the mapping of nodes to WebRender [`SpatialId`]s.
    pub fn update_mapping(&mut self, mapping: Vec<SpatialId>) {
        for (spatial_id, node) in mapping.into_iter().zip(self.nodes.iter_mut()) {
            node.webrender_id = Some(spatial_id);
        }
    }

    /// Get a mutable reference to the node with the given index.
    pub fn get_node_mut(&mut self, id: ScrollTreeNodeId) -> &mut ScrollTreeNode {
        &mut self.nodes[id.index]
    }

    /// Get an immutable reference to the node with the given index.
    pub fn get_node(&self, id: ScrollTreeNodeId) -> &ScrollTreeNode {
        &self.nodes[id.index]
    }

    /// Get the WebRender [`SpatialId`] for the given [`ScrollNodeId`]. This will
    /// panic if [`ScrollTree::build_display_list`] has not been called yet.
    pub fn webrender_id(&self, id: ScrollTreeNodeId) -> SpatialId {
        self.get_node(id).webrender_id()
    }

    pub fn scroll_node_or_ancestor_inner(
        &mut self,
        scroll_node_id: ScrollTreeNodeId,
        scroll_location: ScrollLocation,
        context: ScrollType,
    ) -> Option<(ExternalScrollId, LayoutVector2D)> {
        let parent = {
            let node = &mut self.get_node_mut(scroll_node_id);
            let result = node.scroll(scroll_location, context);
            if result.is_some() {
                return result;
            }
            node.parent
        };

        parent
            .and_then(|parent| self.scroll_node_or_ancestor_inner(parent, scroll_location, context))
    }

    fn node_with_external_scroll_node_id(
        &self,
        external_id: ExternalScrollId,
    ) -> Option<ScrollTreeNodeId> {
        self.nodes
            .iter()
            .enumerate()
            .find_map(|(index, node)| match &node.info {
                SpatialTreeNodeInfo::Scroll(info) if info.external_id == external_id => {
                    Some(ScrollTreeNodeId { index })
                },
                _ => None,
            })
    }

    /// Scroll the scroll node with the given [`ExternalScrollId`] on this scroll tree. If
    /// the node cannot be scrolled, because it's already scrolled to the maximum scroll
    /// extent, try to scroll an ancestor of this node. Returns the node scrolled and the
    /// new offset if a scroll was performed, otherwise returns None.
    pub fn scroll_node_or_ancestor(
        &mut self,
        external_id: ExternalScrollId,
        scroll_location: ScrollLocation,
        context: ScrollType,
    ) -> Option<(ExternalScrollId, LayoutVector2D)> {
        let scroll_node_id = self.node_with_external_scroll_node_id(external_id)?;
        let result = self.scroll_node_or_ancestor_inner(scroll_node_id, scroll_location, context);
        if result.is_some() {
            self.invalidate_cached_transforms();
        }
        result
    }

    /// Given an [`ExternalScrollId`] and an offset, update the scroll offset of the scroll node
    /// with the given id.
    pub fn set_scroll_offset_for_node_with_external_scroll_id(
        &mut self,
        external_scroll_id: ExternalScrollId,
        offset: LayoutVector2D,
        context: ScrollType,
    ) -> Option<LayoutVector2D> {
        let result = self.nodes.iter_mut().find_map(|node| match node.info {
            SpatialTreeNodeInfo::Scroll(ref mut scroll_info)
                if scroll_info.external_id == external_scroll_id =>
            {
                scroll_info.scroll_to_offset(offset, context)
            },
            _ => None,
        });

        if result.is_some() {
            self.invalidate_cached_transforms();
        }

        result
    }

    /// Given a set of all scroll offsets coming from the Servo renderer, update all of the offsets
    /// for nodes that actually exist in this tree.
    pub fn set_all_scroll_offsets(
        &mut self,
        offsets: &FxHashMap<ExternalScrollId, LayoutVector2D>,
    ) {
        for node in self.nodes.iter_mut() {
            if let SpatialTreeNodeInfo::Scroll(ref mut scroll_info) = node.info {
                if let Some(offset) = offsets.get(&scroll_info.external_id) {
                    scroll_info.scroll_to_offset(*offset, ScrollType::Script);
                }
            }
        }

        self.invalidate_cached_transforms();
    }

    /// Set the offsets of all scrolling nodes in this tree to 0.
    pub fn reset_all_scroll_offsets(&mut self) {
        for node in self.nodes.iter_mut() {
            if let SpatialTreeNodeInfo::Scroll(ref mut scroll_info) = node.info {
                scroll_info.scroll_to_offset(LayoutVector2D::zero(), ScrollType::Script);
            }
        }

        self.invalidate_cached_transforms();
    }

    /// Collect all of the scroll offsets of the scrolling nodes of this tree into a
    /// [`HashMap`] which can be applied to another tree.
    pub fn scroll_offsets(&self) -> FxHashMap<ExternalScrollId, LayoutVector2D> {
        HashMap::from_iter(self.nodes.iter().filter_map(|node| match node.info {
            SpatialTreeNodeInfo::Scroll(ref scroll_info) => {
                Some((scroll_info.external_id, scroll_info.offset))
            },
            _ => None,
        }))
    }

    /// Get the scroll offset for the given [`ExternalScrollId`] or `None` if that node cannot
    /// be found in the tree.
    pub fn scroll_offset(&self, id: ExternalScrollId) -> Option<LayoutVector2D> {
        self.nodes.iter().find_map(|node| match node.info {
            SpatialTreeNodeInfo::Scroll(ref info) if info.external_id == id => Some(info.offset),
            _ => None,
        })
    }

    /// Find a transformation that can convert a point in the node coordinate system to a
    /// point in the root coordinate system.
    pub fn cumulative_node_to_root_transform(
        &self,
        node_id: ScrollTreeNodeId,
    ) -> FastLayoutTransform {
        self.cumulative_node_transform(node_id)
            .node_to_root_transform
    }

    /// Find a transformation that can convert a point in the root coordinate system to a
    /// point in the coordinate system of the given node. This may be `None` if the cumulative
    /// transform is uninvertible.
    pub fn cumulative_root_to_node_transform(
        &self,
        node_id: ScrollTreeNodeId,
    ) -> Option<FastLayoutTransform> {
        self.cumulative_node_transform(node_id)
            .root_to_node_transform
    }

    /// Find the cumulative offsets of sticky positioned boxes from the given node up to
    /// the root.
    pub fn cumulative_sticky_offsets(&self, node_id: ScrollTreeNodeId) -> LayoutVector2D {
        self.cumulative_node_transform(node_id)
            .cumulative_sticky_offsets
    }

    fn cumulative_node_transform(
        &self,
        node_id: ScrollTreeNodeId,
    ) -> ScrollTreeNodeTransformationCache {
        let node = self.get_node(node_id);
        if let Some(cached_transforms) = node.transformation_cache.get() {
            return cached_transforms;
        }

        let transforms = self.cumulative_node_transform_inner(node);
        node.transformation_cache.set(Some(transforms));
        transforms
    }

    /// Traverse a scroll node to its root to calculate the transform.
    fn cumulative_node_transform_inner(
        &self,
        node: &ScrollTreeNode,
    ) -> ScrollTreeNodeTransformationCache {
        let parent_transforms = node
            .parent
            .map(|parent_id| self.cumulative_node_transform(parent_id))
            .unwrap_or_default();

        let node_to_root_transform = |node_to_parent_transform: FastLayoutTransform| {
            node_to_parent_transform.then(&parent_transforms.node_to_root_transform)
        };
        let root_to_node_transform = |parent_to_node_transform: FastLayoutTransform| {
            parent_transforms
                .root_to_node_transform
                .map_or(parent_to_node_transform, |parent_transform| {
                    parent_transform.then(&parent_to_node_transform)
                })
        };

        match &node.info {
            SpatialTreeNodeInfo::ReferenceFrame(info) => {
                // To apply a transformation we need to make sure the rectangle's
                // coordinate space is the same as reference frame's coordinate space.
                let offset = info.frame_origin_for_query.to_vector();
                let node_to_parent_transform =
                    info.transform.pre_translate(-offset).then_translate(offset);
                let parent_to_node_transform = info.transform.inverse().map(|inverse_transform| {
                    FastLayoutTransform::Offset(-info.origin.to_vector()).then(&inverse_transform)
                });
                ScrollTreeNodeTransformationCache {
                    node_to_root_transform: node_to_root_transform(node_to_parent_transform),
                    root_to_node_transform: parent_to_node_transform.map(root_to_node_transform),
                    nearest_scrolling_ancestor_viewport: parent_transforms
                        .nearest_scrolling_ancestor_viewport
                        .translate(-info.origin.to_vector()),
                    nearest_scrolling_ancestor_offset: parent_transforms
                        .nearest_scrolling_ancestor_offset,
                    cumulative_sticky_offsets: parent_transforms.cumulative_sticky_offsets,
                }
            },
            SpatialTreeNodeInfo::Scroll(info) => {
                let node_to_parent_transform = FastLayoutTransform::Offset(-info.offset);
                let parent_to_node_transform = node_to_parent_transform.inverse();
                ScrollTreeNodeTransformationCache {
                    node_to_root_transform: node_to_root_transform(node_to_parent_transform),
                    root_to_node_transform: parent_to_node_transform.map(root_to_node_transform),
                    nearest_scrolling_ancestor_viewport: info.clip_rect,
                    nearest_scrolling_ancestor_offset: -info.offset,
                    cumulative_sticky_offsets: parent_transforms.cumulative_sticky_offsets,
                }
            },

            SpatialTreeNodeInfo::Sticky(info) => {
                let offset = info.calculate_sticky_offset(
                    &parent_transforms.nearest_scrolling_ancestor_offset,
                    &parent_transforms.nearest_scrolling_ancestor_viewport,
                );
                let node_to_parent_transform = FastLayoutTransform::Offset(offset);
                let parent_to_node_transform = node_to_parent_transform.inverse();
                ScrollTreeNodeTransformationCache {
                    node_to_root_transform: node_to_root_transform(node_to_parent_transform),
                    root_to_node_transform: parent_to_node_transform.map(root_to_node_transform),
                    nearest_scrolling_ancestor_viewport: parent_transforms
                        .nearest_scrolling_ancestor_viewport,
                    nearest_scrolling_ancestor_offset: parent_transforms
                        .nearest_scrolling_ancestor_offset +
                        offset,
                    cumulative_sticky_offsets: parent_transforms.cumulative_sticky_offsets + offset,
                }
            },
        }
    }

    fn invalidate_cached_transforms(&self) {
        let Some(root_node) = self.nodes.first() else {
            return;
        };
        root_node.invalidate_cached_transforms(self, false /* ancestors_invalid */);
    }

    fn external_scroll_id_for_scroll_tree_node(
        &self,
        id: ScrollTreeNodeId,
    ) -> Option<ExternalScrollId> {
        let mut maybe_node = Some(self.get_node(id));

        while let Some(node) = maybe_node {
            if let Some(external_scroll_id) = node.external_id() {
                return Some(external_scroll_id);
            }
            maybe_node = node.parent.map(|id| self.get_node(id));
        }

        None
    }
}

/// In order to pretty print the [ScrollTree] structure, we are converting
/// the node list inside the tree to be a adjacency list. The adjacency list
/// then is used for the [ScrollTree::debug_print_traversal] of the tree.
///
/// This preprocessing helps decouples print logic a lot from its construction.
type AdjacencyListForPrint = Vec<Vec<ScrollTreeNodeId>>;

/// Implementation of [ScrollTree] that is related to debugging.
// FIXME: probably we could have a universal trait for this. Especially for
//        structures that utilizes PrintTree.
impl ScrollTree {
    fn nodes_in_adjacency_list(&self) -> AdjacencyListForPrint {
        let mut adjacency_list: AdjacencyListForPrint = vec![Default::default(); self.nodes.len()];

        for (node_index, node) in self.nodes.iter().enumerate() {
            let current_id = ScrollTreeNodeId { index: node_index };
            if let Some(parent_id) = node.parent {
                adjacency_list[parent_id.index].push(current_id);
            }
        }

        adjacency_list
    }

    fn debug_print_traversal(
        &self,
        print_tree: &mut PrintTree,
        current_id: ScrollTreeNodeId,
        adjacency_list: &[Vec<ScrollTreeNodeId>],
    ) {
        for node_id in &adjacency_list[current_id.index] {
            self.nodes[node_id.index].debug_print(print_tree, node_id.index);
            self.debug_print_traversal(print_tree, *node_id, adjacency_list);
        }
        print_tree.end_level();
    }

    /// Print the [ScrollTree]. Particularly, we are printing the node in
    /// preorder traversal. The order of the nodes will depends of the
    /// index of a node in the [ScrollTree] which corresponds to the
    /// declarations of the nodes.
    // TODO(stevennovaryo): add information about which fragment that
    //                      defines this node.
    pub fn debug_print(&self) {
        let mut print_tree = PrintTree::new("Scroll Tree".to_owned());

        let adj_list = self.nodes_in_adjacency_list();
        let root_id = ScrollTreeNodeId { index: 0 };

        self.nodes[root_id.index].debug_print(&mut print_tree, root_id.index);
        self.debug_print_traversal(&mut print_tree, root_id, &adj_list);
        print_tree.end_level();
    }
}

/// A data structure which stores `Paint`-side information about
/// display lists sent to `Paint`.
#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct PaintDisplayListInfo {
    /// The WebRender [PipelineId] of this display list.
    pub pipeline_id: PipelineId,

    /// The [`ViewportDetails`] that describe the viewport in the script/layout thread at
    /// the time this display list was created.
    pub viewport_details: ViewportDetails,

    /// The size of this display list's content.
    pub content_size: LayoutSize,

    /// The epoch of the display list.
    pub epoch: Epoch,

    /// A ScrollTree used by `Paint` to scroll the contents of the
    /// display list.
    pub scroll_tree: ScrollTree,

    /// The `ScrollTreeNodeId` of the root reference frame of this info's scroll
    /// tree.
    pub root_reference_frame_id: ScrollTreeNodeId,

    /// The `ScrollTreeNodeId` of the topmost scrolling frame of this info's scroll
    /// tree.
    pub root_scroll_node_id: ScrollTreeNodeId,

    /// Contentful paint i.e. whether the display list contains items of type
    /// text, image, non-white canvas or SVG). Used by metrics.
    /// See <https://w3c.github.io/paint-timing/#first-contentful-paint>.
    pub is_contentful: bool,

    /// Whether the first layout or a subsequent (incremental) layout triggered this
    /// display list creation.
    pub first_reflow: bool,
}

impl PaintDisplayListInfo {
    /// Create a new PaintDisplayListInfo with the root reference frame
    /// and scroll frame already added to the scroll tree.
    pub fn new(
        viewport_details: ViewportDetails,
        content_size: LayoutSize,
        pipeline_id: PipelineId,
        epoch: Epoch,
        viewport_scroll_sensitivity: AxesScrollSensitivity,
        first_reflow: bool,
    ) -> Self {
        let mut scroll_tree = ScrollTree::default();
        let root_reference_frame_id = scroll_tree.add_scroll_tree_node(
            None,
            SpatialTreeNodeInfo::ReferenceFrame(ReferenceFrameNodeInfo {
                origin: Default::default(),
                frame_origin_for_query: Default::default(),
                transform_style: TransformStyle::Flat,
                transform: FastLayoutTransform::identity(),
                kind: ReferenceFrameKind::default(),
            }),
        );
        let root_scroll_node_id = scroll_tree.add_scroll_tree_node(
            Some(root_reference_frame_id),
            SpatialTreeNodeInfo::Scroll(ScrollableNodeInfo {
                external_id: ExternalScrollId(0, pipeline_id),
                content_rect: LayoutRect::from_origin_and_size(LayoutPoint::zero(), content_size),
                clip_rect: LayoutRect::from_origin_and_size(
                    LayoutPoint::zero(),
                    viewport_details.layout_size(),
                ),
                scroll_sensitivity: viewport_scroll_sensitivity,
                offset: LayoutVector2D::zero(),
                offset_changed: Cell::new(false),
            }),
        );

        PaintDisplayListInfo {
            pipeline_id,
            viewport_details,
            content_size,
            epoch,
            scroll_tree,
            root_reference_frame_id,
            root_scroll_node_id,
            is_contentful: false,
            first_reflow,
        }
    }

    pub fn external_scroll_id_for_scroll_tree_node(
        &self,
        id: ScrollTreeNodeId,
    ) -> ExternalScrollId {
        self.scroll_tree
            .external_scroll_id_for_scroll_tree_node(id)
            .unwrap_or(ExternalScrollId(0, self.pipeline_id))
    }
}
