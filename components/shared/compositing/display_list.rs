/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Defines data structures which are consumed by the Compositor.

use std::collections::HashMap;

use base::id::ScrollTreeNodeId;
use base::print_tree::PrintTree;
use bitflags::bitflags;
use embedder_traits::Cursor;
use euclid::SideOffsets2D;
use malloc_size_of_derive::MallocSizeOf;
use serde::{Deserialize, Serialize};
use style::values::specified::Overflow;
use webrender_api::units::{
    LayoutPixel, LayoutPoint, LayoutRect, LayoutSize, LayoutTransform, LayoutVector2D,
};
use webrender_api::{
    Epoch, ExternalScrollId, PipelineId, ReferenceFrameKind, ScrollLocation, SpatialId,
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

/// Convert [Overflow] to [ScrollSensitivity].
impl From<Overflow> for ScrollType {
    fn from(overflow: Overflow) -> Self {
        match overflow {
            Overflow::Hidden => ScrollType::Script,
            Overflow::Scroll | Overflow::Auto => ScrollType::Script | ScrollType::InputEvents,
            Overflow::Visible | Overflow::Clip => ScrollType::empty(),
        }
    }
}

/// The [ScrollSensitivity] of particular node in the vertical and horizontal axes.
#[derive(Clone, Copy, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub struct AxesScrollSensitivity {
    pub x: ScrollType,
    pub y: ScrollType,
}

/// Information that Servo keeps alongside WebRender display items
/// in order to add more context to hit test results.
#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct HitTestInfo {
    /// The id of the node of this hit test item.
    pub node: u64,

    /// The cursor of this node's hit test item.
    pub cursor: Option<Cursor>,

    /// The id of the [ScrollTree] associated with this hit test item.
    pub scroll_tree_node: ScrollTreeNodeId,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum SpatialTreeNodeInfo {
    ReferenceFrame(ReferenceFrameNodeInfo),
    Scroll(ScrollableNodeInfo),
    Sticky(StickyNodeInfo),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StickyNodeInfo {
    pub frame_rect: LayoutRect,
    pub margins: SideOffsets2D<Option<f32>, LayoutPixel>,
    pub vertical_offset_bounds: StickyOffsetBounds,
    pub horizontal_offset_bounds: StickyOffsetBounds,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ReferenceFrameNodeInfo {
    pub origin: LayoutPoint,
    pub transform_style: TransformStyle,
    pub transform: LayoutTransform,
    pub kind: ReferenceFrameKind,
}

/// Data stored for nodes in the [ScrollTree] that actually scroll,
/// as opposed to reference frames and sticky nodes which do not.
#[derive(Debug, Deserialize, Serialize)]
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
                return Some(self.offset);
            },
            ScrollLocation::End => {
                let end_pos = self.scrollable_size().height;
                if self.offset.y.round() >= end_pos {
                    // Nothing to do on this layer.
                    return None;
                }

                self.offset.y = end_pos;
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

#[derive(Debug, Deserialize, Serialize)]
/// A node in a tree of scroll nodes. This may either be a scrollable
/// node which responds to scroll events or a non-scrollable one.
pub struct ScrollTreeNode {
    /// The index of the parent of this node in the tree. If this is
    /// None then this is the root node.
    pub parent: Option<ScrollTreeNodeId>,

    /// The WebRender id, which is filled in when this tree is serialiezd
    /// into a WebRender display list.
    pub webrender_id: Option<SpatialId>,

    /// Specific information about this node, depending on whether it is a scroll node
    /// or a reference frame.
    pub info: SpatialTreeNodeInfo,
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

    /// Set the offset for this node, returns false if this was a
    /// non-scrolling node for which you cannot set the offset.
    pub fn set_offset(&mut self, new_offset: LayoutVector2D) {
        if let SpatialTreeNodeInfo::Scroll(ref mut info) = self.info {
            info.scroll_to_offset(new_offset, ScrollType::Script);
        }
    }

    /// Scroll this node given a WebRender ScrollLocation. Returns a tuple that can
    /// be used to scroll an individual WebRender scroll frame if the operation
    /// actually changed an offset.
    pub fn scroll(
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
}

/// A tree of spatial nodes, which mirrors the spatial nodes in the WebRender
/// display list, except these are used to scrolling in the compositor so that
/// new offsets can be sent to WebRender.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ScrollTree {
    /// A list of compositor-side scroll nodes that describe the tree
    /// of WebRender spatial nodes, used by the compositor to scroll the
    /// contents of the display list.
    pub nodes: Vec<ScrollTreeNode>,
}

impl ScrollTree {
    /// Add a scroll node to this ScrollTree returning the id of the new node.
    pub fn add_scroll_tree_node(
        &mut self,
        parent: Option<&ScrollTreeNodeId>,
        info: SpatialTreeNodeInfo,
    ) -> ScrollTreeNodeId {
        self.nodes.push(ScrollTreeNode {
            parent: parent.cloned(),
            webrender_id: None,
            info,
        });
        ScrollTreeNodeId {
            index: self.nodes.len() - 1,
        }
    }

    /// Once WebRender display list construction is complete for this [`ScrollTree`], update
    /// the mapping of nodes to WebRender [`SpatialId`]s.
    pub fn update_mapping(&mut self, mapping: Vec<SpatialId>) {
        for (spatial_id, node) in mapping.into_iter().zip(self.nodes.iter_mut()) {
            node.webrender_id = Some(spatial_id);
        }
    }

    /// Get a mutable reference to the node with the given index.
    pub fn get_node_mut(&mut self, id: &ScrollTreeNodeId) -> &mut ScrollTreeNode {
        &mut self.nodes[id.index]
    }

    /// Get an immutable reference to the node with the given index.
    pub fn get_node(&self, id: &ScrollTreeNodeId) -> &ScrollTreeNode {
        &self.nodes[id.index]
    }

    /// Get the WebRender [`SpatialId`] for the given [`ScrollNodeId`]. This will
    /// panic if [`ScrollTree::build_display_list`] has not been called yet.
    pub fn webrender_id(&self, id: &ScrollTreeNodeId) -> SpatialId {
        self.get_node(id).webrender_id()
    }

    /// Scroll the given scroll node on this scroll tree. If the node cannot be scrolled,
    /// because it isn't a scrollable node or it's already scrolled to the maximum scroll
    /// extent, try to scroll an ancestor of this node. Returns the node scrolled and the
    /// new offset if a scroll was performed, otherwise returns None.
    pub fn scroll_node_or_ancestor(
        &mut self,
        scroll_node_id: &ScrollTreeNodeId,
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

        parent.and_then(|parent| self.scroll_node_or_ancestor(&parent, scroll_location, context))
    }

    /// Given an [`ExternalScrollId`] and an offset, update the scroll offset of the scroll node
    /// with the given id.
    pub fn set_scroll_offset_for_node_with_external_scroll_id(
        &mut self,
        external_scroll_id: ExternalScrollId,
        offset: LayoutVector2D,
        context: ScrollType,
    ) -> Option<LayoutVector2D> {
        self.nodes.iter_mut().find_map(|node| match node.info {
            SpatialTreeNodeInfo::Scroll(ref mut scroll_info)
                if scroll_info.external_id == external_scroll_id =>
            {
                scroll_info.scroll_to_offset(offset, context)
            },
            _ => None,
        })
    }

    /// Given a set of all scroll offsets coming from the Servo renderer, update all of the offsets
    /// for nodes that actually exist in this tree.
    pub fn set_all_scroll_offsets(&mut self, offsets: &HashMap<ExternalScrollId, LayoutVector2D>) {
        for node in self.nodes.iter_mut() {
            if let SpatialTreeNodeInfo::Scroll(ref mut scroll_info) = node.info {
                if let Some(offset) = offsets.get(&scroll_info.external_id) {
                    scroll_info.scroll_to_offset(*offset, ScrollType::Script);
                }
            }
        }
    }

    /// Collect all of the scroll offsets of the scrolling nodes of this tree into a
    /// [`HashMap`] which can be applied to another tree.
    pub fn scroll_offsets(&self) -> HashMap<ExternalScrollId, LayoutVector2D> {
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
        current_id: &ScrollTreeNodeId,
        adjacency_list: &[Vec<ScrollTreeNodeId>],
    ) {
        for node_id in &adjacency_list[current_id.index] {
            self.nodes[node_id.index].debug_print(print_tree, node_id.index);
            self.debug_print_traversal(print_tree, node_id, adjacency_list);
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
        self.debug_print_traversal(&mut print_tree, &root_id, &adj_list);
        print_tree.end_level();
    }
}

/// A data structure which stores compositor-side information about
/// display lists sent to the compositor.
#[derive(Debug, Deserialize, Serialize)]
pub struct CompositorDisplayListInfo {
    /// The WebRender [PipelineId] of this display list.
    pub pipeline_id: PipelineId,

    /// The size of the viewport that this display list renders into.
    pub viewport_size: LayoutSize,

    /// The size of this display list's content.
    pub content_size: LayoutSize,

    /// The epoch of the display list.
    pub epoch: Epoch,

    /// An array of `HitTestInfo` which is used to store information
    /// to assist the compositor to take various actions (set the cursor,
    /// scroll without layout) using a WebRender hit test result.
    pub hit_test_info: Vec<HitTestInfo>,

    /// A ScrollTree used by the compositor to scroll the contents of the
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

impl CompositorDisplayListInfo {
    /// Create a new CompositorDisplayListInfo with the root reference frame
    /// and scroll frame already added to the scroll tree.
    pub fn new(
        viewport_size: LayoutSize,
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
                transform_style: TransformStyle::Flat,
                transform: LayoutTransform::identity(),
                kind: ReferenceFrameKind::default(),
            }),
        );
        let root_scroll_node_id = scroll_tree.add_scroll_tree_node(
            Some(&root_reference_frame_id),
            SpatialTreeNodeInfo::Scroll(ScrollableNodeInfo {
                external_id: ExternalScrollId(0, pipeline_id),
                content_rect: LayoutRect::from_origin_and_size(LayoutPoint::zero(), content_size),
                clip_rect: LayoutRect::from_origin_and_size(LayoutPoint::zero(), viewport_size),
                scroll_sensitivity: viewport_scroll_sensitivity,
                offset: LayoutVector2D::zero(),
            }),
        );

        CompositorDisplayListInfo {
            pipeline_id,
            viewport_size,
            content_size,
            epoch,
            hit_test_info: Default::default(),
            scroll_tree,
            root_reference_frame_id,
            root_scroll_node_id,
            is_contentful: false,
            first_reflow,
        }
    }

    /// Add or re-use a duplicate HitTestInfo entry in this `CompositorHitTestInfo`
    /// and return the index.
    pub fn add_hit_test_info(
        &mut self,
        node: u64,
        cursor: Option<Cursor>,
        scroll_tree_node: ScrollTreeNodeId,
    ) -> usize {
        let hit_test_info = HitTestInfo {
            node,
            cursor,
            scroll_tree_node,
        };

        if let Some(last) = self.hit_test_info.last() {
            if hit_test_info == *last {
                return self.hit_test_info.len() - 1;
            }
        }

        self.hit_test_info.push(hit_test_info);
        self.hit_test_info.len() - 1
    }
}
