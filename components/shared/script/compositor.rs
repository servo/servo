/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Defines data structures which are consumed by the Compositor.

use embedder_traits::Cursor;
use serde::{Deserialize, Serialize};
use webrender_api::units::{LayoutSize, LayoutVector2D};
use webrender_api::{Epoch, ExternalScrollId, PipelineId, ScrollLocation, SpatialId};

/// The scroll sensitivity of a scroll node ie whether it can be scrolled due to input event and
/// script events or only script events.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum ScrollSensitivity {
    /// This node can be scrolled by input and script events.
    ScriptAndInputEvents,
    /// This node can only be scrolled by script events.
    Script,
}

/// Information that Servo keeps alongside WebRender display items
/// in order to add more context to hit test results.
#[derive(Debug, Deserialize, Serialize)]
pub struct HitTestInfo {
    /// The id of the node of this hit test item.
    pub node: u64,

    /// The cursor of this node's hit test item.
    pub cursor: Option<Cursor>,

    /// The id of the [ScrollTree] associated with this hit test item.
    pub scroll_tree_node: ScrollTreeNodeId,
}

/// An id for a ScrollTreeNode in the ScrollTree. This contains both the index
/// to the node in the tree's array of nodes as well as the corresponding SpatialId
/// for the SpatialNode in the WebRender display list.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct ScrollTreeNodeId {
    /// The index of this scroll tree node in the tree's array of nodes.
    pub index: usize,

    /// The WebRender spatial id of this scroll tree node.
    pub spatial_id: SpatialId,
}

/// Data stored for nodes in the [ScrollTree] that actually scroll,
/// as opposed to reference frames and sticky nodes which do not.
#[derive(Debug, Deserialize, Serialize)]
pub struct ScrollableNodeInfo {
    /// The external scroll id of this node, used to track
    /// it between successive re-layouts.
    pub external_id: ExternalScrollId,

    /// Amount that this `ScrollableNode` can scroll in both directions.
    pub scrollable_size: LayoutSize,

    /// Whether this `ScrollableNode` is sensitive to input events.
    pub scroll_sensitivity: ScrollSensitivity,

    /// The current offset of this scroll node.
    pub offset: LayoutVector2D,
}

#[derive(Debug, Deserialize, Serialize)]
/// A node in a tree of scroll nodes. This may either be a scrollable
/// node which responds to scroll events or a non-scrollable one.
pub struct ScrollTreeNode {
    /// The index of the parent of this node in the tree. If this is
    /// None then this is the root node.
    pub parent: Option<ScrollTreeNodeId>,

    /// Scrolling data which will not be None if this is a scrolling node.
    pub scroll_info: Option<ScrollableNodeInfo>,
}

impl ScrollTreeNode {
    /// Get the external id of this node.
    pub fn external_id(&self) -> Option<ExternalScrollId> {
        self.scroll_info.as_ref().map(|info| info.external_id)
    }

    /// Get the offset id of this node if it applies.
    pub fn offset(&self) -> Option<LayoutVector2D> {
        self.scroll_info.as_ref().map(|info| info.offset)
    }

    /// Set the offset for this node, returns false if this was a
    /// non-scrolling node for which you cannot set the offset.
    pub fn set_offset(&mut self, new_offset: LayoutVector2D) -> bool {
        match self.scroll_info {
            Some(ref mut info) => {
                info.offset = new_offset;
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
        let info = match self.scroll_info {
            Some(ref mut data) => data,
            None => return None,
        };

        if info.scroll_sensitivity != ScrollSensitivity::ScriptAndInputEvents {
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
                let end_pos = -info.scrollable_size.height;
                if info.offset.y.round() <= end_pos {
                    // Nothing to do on this layer.
                    return None;
                }

                info.offset.y = end_pos;
                return Some((info.external_id, info.offset));
            },
        };

        let scrollable_width = info.scrollable_size.width;
        let scrollable_height = info.scrollable_size.height;
        let original_layer_scroll_offset = info.offset;

        if scrollable_width > 0. {
            info.offset.x = (info.offset.x + delta.x).min(0.0).max(-scrollable_width);
        }

        if scrollable_height > 0. {
            info.offset.y = (info.offset.y + delta.y).min(0.0).max(-scrollable_height);
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
}

impl ScrollTree {
    /// Add a scroll node to this ScrollTree returning the id of the new node.
    pub fn add_scroll_tree_node(
        &mut self,
        parent: Option<&ScrollTreeNodeId>,
        spatial_id: SpatialId,
        scroll_info: Option<ScrollableNodeInfo>,
    ) -> ScrollTreeNodeId {
        self.nodes.push(ScrollTreeNode {
            parent: parent.cloned(),
            scroll_info,
        });
        ScrollTreeNodeId {
            index: self.nodes.len() - 1,
            spatial_id,
        }
    }

    /// Get a mutable reference to the node with the given index.
    pub fn get_node_mut(&mut self, id: &ScrollTreeNodeId) -> &mut ScrollTreeNode {
        &mut self.nodes[id.index]
    }

    /// Get an immutable reference to the node with the given index.
    pub fn get_node(&mut self, id: &ScrollTreeNodeId) -> &ScrollTreeNode {
        &self.nodes[id.index]
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
        external_scroll_id: ExternalScrollId,
        offset: LayoutVector2D,
    ) -> bool {
        for node in self.nodes.iter_mut() {
            match node.scroll_info {
                Some(ref mut scroll_info) if scroll_info.external_id == external_scroll_id => {
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
}

impl CompositorDisplayListInfo {
    /// Create a new CompositorDisplayListInfo with the root reference frame
    /// and scroll frame already added to the scroll tree.
    pub fn new(
        viewport_size: LayoutSize,
        content_size: LayoutSize,
        pipeline_id: PipelineId,
        epoch: Epoch,
        root_scroll_sensitivity: ScrollSensitivity,
    ) -> Self {
        let mut scroll_tree = ScrollTree::default();
        let root_reference_frame_id = scroll_tree.add_scroll_tree_node(
            None,
            SpatialId::root_reference_frame(pipeline_id),
            None,
        );
        let root_scroll_node_id = scroll_tree.add_scroll_tree_node(
            Some(&root_reference_frame_id),
            SpatialId::root_scroll_node(pipeline_id),
            Some(ScrollableNodeInfo {
                external_id: ExternalScrollId(0, pipeline_id),
                scrollable_size: content_size - viewport_size,
                scroll_sensitivity: root_scroll_sensitivity,
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
        if let Some(last) = self.hit_test_info.last() {
            if node == last.node && cursor == last.cursor {
                return self.hit_test_info.len() - 1;
            }
        }

        self.hit_test_info.push(HitTestInfo {
            node,
            cursor,
            scroll_tree_node,
        });
        self.hit_test_info.len() - 1
    }
}
