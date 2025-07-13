/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base::id::ScrollTreeNodeId;
use compositing_traits::display_list::{ScrollTree, ScrollTreeNode, SpatialTreeNodeInfo};
use compositing_traits::largest_contentful_paint_candidate::{LCPCandidate, LCPCandidateID};
use webrender_api::Epoch;
use webrender_api::units::{LayoutRect, LayoutSize};

use crate::display_list::clip::{ClipId, StackingContextTreeClipStore};

pub(crate) struct LargestContentfulPaintCandidateCollector<'a> {
    /// Used to track CSS like overflow from current element to root element.
    pub clip_tree: &'a StackingContextTreeClipStore,
    /// The scroll node id, used to get [`ScrollTreeNode`] from scroll tree and
    /// resolve transform CSS
    pub current_scroll_node_id: ScrollTreeNodeId,
    /// The ClipId, used to get [`Clip`] from ClipTree.
    pub current_clip_id: ClipId,
    /// The LCP candidate, it may be a image or text.
    pub lcp_candidate: Option<LCPCandidate>,
    /// Whether is recording LCP.
    /// TODO(boluochoufeng): Set it to false for now, and add the corresponding handling later.
    pub is_recording_lcp: bool,
    /// The rect of viewport.
    pub viewport_rect: LayoutRect,
    /// The current [`Epoch`]
    pub cur_epoch: Epoch,
}

impl<'a> LargestContentfulPaintCandidateCollector<'a> {
    pub fn new(
        clip_tree: &'a StackingContextTreeClipStore,
        current_scroll_node_id: ScrollTreeNodeId,
        is_recording_lcp: bool,
        viewport_size: LayoutSize,
        cur_epoch: Epoch,
    ) -> Self {
        Self {
            clip_tree,
            current_scroll_node_id,
            current_clip_id: ClipId::INVALID,
            lcp_candidate: None,
            is_recording_lcp,
            viewport_rect: LayoutRect::from_size(viewport_size),
            cur_epoch,
        }
    }

    pub fn update_current_node_id(
        &mut self,
        next_scroll_node_id: ScrollTreeNodeId,
        next_clip_id: ClipId,
    ) {
        self.current_scroll_node_id = next_scroll_node_id;
        self.current_clip_id = next_clip_id;
    }

    pub fn update_image_candidate(
        &mut self,
        id: LCPCandidateID,
        rect: LayoutRect,
        scroll_tree: &ScrollTree,
    ) {
        if !self.is_recording_lcp {
            return;
        }

        let visual_rect = self.compute_visual_rect(rect, scroll_tree);
        let area = visual_rect.area() as usize;
        if area == 0 {
            return;
        }

        // TODO(boluochoufeng): need to filter low-content image

        self.update_candidate(LCPCandidate::new(id, area, self.cur_epoch));
    }

    fn update_candidate(&mut self, candidate: LCPCandidate) {
        if let Some(ref mut latest_candidate) = self.lcp_candidate {
            if candidate.area > latest_candidate.area {
                *latest_candidate = candidate;
            }
        } else {
            self.lcp_candidate = Some(candidate);
        }
    }

    /// Calculate the size of visual area in viewport. Because the parent elements will affect children,
    /// We need track these css up to the root node.
    fn compute_visual_rect(&self, rect: LayoutRect, scroll_tree: &ScrollTree) -> LayoutRect {
        let visual_rect = self.compute_visual_rect_with_scroll_clip(
            rect,
            Some(self.current_scroll_node_id),
            self.current_clip_id,
            scroll_tree,
        );

        visual_rect
            .intersection(&self.viewport_rect)
            .unwrap_or(LayoutRect::zero())
    }

    fn compute_visual_rect_with_scroll_clip(
        &self,
        rect: LayoutRect,
        scroll_id: Option<ScrollTreeNodeId>,
        clip_id: ClipId,
        scroll_tree: &ScrollTree,
    ) -> LayoutRect {
        let mut visual_rect = rect;
        let mut parent_clip_id = clip_id;
        let mut parent_scroll_id = scroll_id;

        while let Some(scroll_id) = parent_scroll_id {
            // 1.If there is clip CSS in the stacking context created by transform, apply it.
            while let Some(clip_node) = &self
                .clip_tree
                .get_node(&parent_clip_id)
                .filter(|node| node.parent_scroll_node_id == scroll_id)
            {
                if clip_node.id == ClipId::INVALID {
                    parent_clip_id = clip_node.id;
                } else {
                    visual_rect = clip_node
                        .rect
                        .intersection(&rect)
                        .unwrap_or(LayoutRect::zero());
                    parent_clip_id = clip_node.parent_clip_id;
                }
            }

            // 2.Apply transform CSS.
            let scroll_node = scroll_tree.get_node(&scroll_id);
            visual_rect = self.apply_transform_to_visual_rect(visual_rect, scroll_node);
            parent_scroll_id = scroll_node.parent;
        }

        visual_rect
    }

    fn apply_transform_to_visual_rect(
        &self,
        rect: LayoutRect,
        scroll_node: &ScrollTreeNode,
    ) -> LayoutRect {
        match &scroll_node.info {
            SpatialTreeNodeInfo::ReferenceFrame(frame_node_info) => {
                let origin = frame_node_info.origin;
                let transform = &frame_node_info.transform;
                let outer_rect = transform.outer_transformed_box2d(&rect);
                if let Some(rect) = outer_rect {
                    rect.translate(origin.to_vector())
                } else {
                    LayoutRect::zero()
                }
            },
            _ => rect,
        }
    }
}
