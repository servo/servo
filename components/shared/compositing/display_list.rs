/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Defines data structures which are consumed by the Compositor.

use std::collections::HashMap;

use base::id::ScrollTreeNodeId;
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

/// The scroll sensitivity of a scroll node in a particular axis ie whether it can be scrolled due to
/// input events and script events or only script events.
#[derive(Clone, Copy, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub enum ScrollSensitivity {
    /// This node can be scrolled by input and script events.
    ScriptAndInputEvents,
    /// This node can only be scrolled by script events.
    Script,
    /// This node cannot be scrolled.
    None,
}

/// Convert [Overflow] to [ScrollSensitivity].
impl From<Overflow> for ScrollSensitivity {
    fn from(overflow: Overflow) -> Self {
        match overflow {
            Overflow::Hidden => ScrollSensitivity::Script,
            Overflow::Scroll | Overflow::Auto => ScrollSensitivity::ScriptAndInputEvents,
            Overflow::Visible | Overflow::Clip => ScrollSensitivity::None,
        }
    }
}

/// The [ScrollSensitivity] of particular node in the vertical and horizontal axes.
#[derive(Clone, Copy, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub struct AxesScrollSensitivity {
    pub x: ScrollSensitivity,
    pub y: ScrollSensitivity,
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
    /// Get the WebRender [`SpatialId`] for the given [`ScrollTreeNodeId`]. This will
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
    pub fn set_offset(&mut self, new_offset: LayoutVector2D) -> bool {
        match self.info {
            SpatialTreeNodeInfo::Scroll(ref mut info) => {
                let scrollable_size = info.scrollable_size();
                if scrollable_size.width > 0. {
                    info.offset.x = (new_offset.x).min(0.0).max(-scrollable_size.width);
                }

                if scrollable_size.height > 0. {
                    info.offset.y = (new_offset.y).min(0.0).max(-scrollable_size.height);
                }
                true
            },
            _ => false,
        }
    }

    /// Scroll this node given a WebRender ScrollLocation. Returns a tuple that can
    /// be used to scroll an individual WebRender scroll frame if the operation
    /// actually changed an offset.
    pub fn scroll(
        &mut self,
        scroll_location: ScrollLocation,
    ) -> Option<(ExternalScrollId, LayoutVector2D)> {
        let info = match self.info {
            SpatialTreeNodeInfo::Scroll(ref mut info) => info,
            _ => return None,
        };

        if info.scroll_sensitivity.x != ScrollSensitivity::ScriptAndInputEvents &&
            info.scroll_sensitivity.y != ScrollSensitivity::ScriptAndInputEvents
        {
            return None;
        }

        let delta = match scroll_location {
            ScrollLocation::Delta(delta) => delta,
            ScrollLocation::Start => {
                if info.offset.y.round() >= 0.0 {
                    // Nothing to do on this layer.
                    return None;
                }

                info.offset.y = 0.0;
                return Some((info.external_id, info.offset));
            },
            ScrollLocation::End => {
                let end_pos = -info.scrollable_size().height;
                if info.offset.y.round() <= end_pos {
                    // Nothing to do on this layer.
                    return None;
                }

                info.offset.y = end_pos;
                return Some((info.external_id, info.offset));
            },
        };

        let scrollable_size = info.scrollable_size();
        let original_layer_scroll_offset = info.offset;

        if scrollable_size.width > 0. &&
            info.scroll_sensitivity.x == ScrollSensitivity::ScriptAndInputEvents
        {
            info.offset.x = (info.offset.x + delta.x)
                .min(0.0)
                .max(-scrollable_size.width);
        }

        if scrollable_size.height > 0. &&
            info.scroll_sensitivity.y == ScrollSensitivity::ScriptAndInputEvents
        {
            info.offset.y = (info.offset.y + delta.y)
                .min(0.0)
                .max(-scrollable_size.height);
        }

        if info.offset != original_layer_scroll_offset {
            Some((info.external_id, info.offset))
        } else {
            None
        }
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

    /// Mapping of [ExternalScrollId] to [ScrollTreeNodeId], this is used for quick
    /// access of scrolling nodes operation and querying.
    pub external_scroll_id_to_node_id: HashMap<ExternalScrollId, ScrollTreeNodeId>,
}

impl ScrollTree {
    /// Add a scroll node to this ScrollTree returning the id of the new node.
    pub fn add_scroll_tree_node(
        &mut self,
        parent: Option<&ScrollTreeNodeId>,
        info: SpatialTreeNodeInfo,
    ) -> ScrollTreeNodeId {
        let new_scroll_id =         ScrollTreeNodeId {
            index: self.nodes.len(),
        };
        if let SpatialTreeNodeInfo::Scroll(spatial_scroll_node) = &info {
            self.external_scroll_id_to_node_id.insert(spatial_scroll_node.external_id, new_scroll_id);
        }

        self.nodes.push(ScrollTreeNode {
            parent: parent.cloned(),
            webrender_id: None,
            info,
        });
        new_scroll_id
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

    /// Get a mutable reference to the node with the given external scroll id, returning [None] if an
    /// [ExternalScrollId] does not have a respective [ScrollTreeNode] (e.g. a fragment doesn't have a scroll node).
    /// Same as [Self::get_node_mut], it will panic if the given id is not registered.
    pub fn get_node_by_external_scroll_id_mut(
        &mut self,
        external_scroll_id: &ExternalScrollId,
    ) -> Option<&mut ScrollTreeNode> {
        self.external_scroll_id_to_node_id
            .get(external_scroll_id)
            .cloned()
            .map(|scroll_id| self.get_node_mut(&scroll_id))
    }

    /// Get an immutable reference to the node with the given external scroll id, returning [None] if an
    /// [ExternalScrollId] does not have a respective [ScrollTreeNode] (e.g. a fragment doesn't have a scroll node).
    /// Same as [Self::get_node_mut], it will panic if the given id is not registered.
    pub fn get_node_by_external_scroll_id(
        &self,
        external_scroll_id: &ExternalScrollId,
    ) -> Option<&ScrollTreeNode> {
        self.external_scroll_id_to_node_id
            .get(external_scroll_id)
            .cloned()
            .map(|scroll_id| self.get_node(&scroll_id))
    }

    /// Get the WebRender [`SpatialId`] for the given [`ScrollTreeNodeId`]. This will
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
    ) -> Option<(ExternalScrollId, LayoutVector2D)> {
        let parent = {
            let node = &mut self.get_node_mut(scroll_node_id);
            let result = node.scroll(scroll_location);
            if result.is_some() {
                return result;
            }
            node.parent
        };

        parent.and_then(|parent| self.scroll_node_or_ancestor(&parent, scroll_location))
    }

    /// Given an [`ExternalScrollId`] and an offset, update the scroll offset of the scroll node
    /// with the given id.
    pub fn set_scroll_offsets_for_node_with_external_scroll_id(
        &mut self,
        external_scroll_id: &ExternalScrollId,
        offset: LayoutVector2D,
    ) -> bool {
        if let Some(node) = self.get_node_by_external_scroll_id_mut(external_scroll_id) {
            match node.info {
                SpatialTreeNodeInfo::Scroll(ref mut scroll_info)
                    if &scroll_info.external_id == external_scroll_id =>
                {
                    scroll_info.offset = offset;
                    return true;
                },
                _ => {},
            }
        }
        false
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
