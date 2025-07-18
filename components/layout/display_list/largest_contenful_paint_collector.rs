/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base::id::ScrollTreeNodeId;
use compositing_traits::display_list::{ScrollTree, SpatialTreeNodeInfo};
use compositing_traits::largest_contentful_paint_candidate::{ImageCandidate, LCPCandidates};
use log::debug;
use webrender_api::Epoch;
use webrender_api::units::{LayoutRect, LayoutSize};

use crate::display_list::clip::{Clip, ClipId, StackingContextTreeClipStore};

pub(crate) struct LargestContentfulPaintCollector<'a> {
    pub clip_tree: &'a StackingContextTreeClipStore,

    pub current_scroll_node_id: ScrollTreeNodeId,

    pub current_clip_id: ClipId,

    pub largerst_image: Option<ImageCandidate>,

    pub lcp_candidates: &'a mut LCPCandidates,
}

impl LargestContentfulPaintCollector<'_> {
    pub fn update_current_node_id(
        &mut self,
        next_scroll_node_id: ScrollTreeNodeId,
        next_clip_id: ClipId,
    ) {
        self.current_scroll_node_id = next_scroll_node_id;
        self.current_clip_id = next_clip_id;
    }

    pub fn record_image_candidate(
        &mut self,
        tag: usize,
        rect: LayoutRect,
        viewport_size: LayoutSize,
        scroll_tree: &ScrollTree,
        epoch: Epoch,
    ) {
        let visual_rect = self.compute_visual_rect(rect, scroll_tree, viewport_size);
        let size = visual_rect.area() as usize;
        if size == 0 {
            return;
        }

        // TODO. need to filter low-content image

        if let Some(largest_image) = self.largerst_image.as_mut() {
            if largest_image.size() < size {
                *largest_image = ImageCandidate::new(tag, size, epoch);
            }
        } else {
            self.largerst_image = Some(ImageCandidate::new(tag, size, epoch));
        }
    }

    pub fn update_largest_contentful_paint_candiate(&mut self) {
        self.lcp_candidates.image_candidate = self.largerst_image.take();
    }

    /// Calculate the size of visual area in viewport. Because the parent elements will affect children,
    /// We need track these css up to the root node.
    fn compute_visual_rect(
        &self,
        rect: LayoutRect,
        scroll_tree: &ScrollTree,
        viewport_size: LayoutSize,
    ) -> LayoutRect {
        let visual_rect = self.compute_visual_rect_with_scroll_clip(
            rect,
            Some(self.current_scroll_node_id),
            self.current_clip_id,
            scroll_tree,
        );
        debug!("visual_rect: {:?}", visual_rect);
        let viewport = LayoutRect::from_size(viewport_size);
        visual_rect
            .intersection(&viewport)
            .unwrap_or(LayoutRect::zero())
    }

    fn compute_visual_rect_with_scroll_clip(
        &self,
        rect: LayoutRect,
        scroll_id: Option<ScrollTreeNodeId>,
        clip_id: ClipId,
        scroll_tree: &ScrollTree,
    ) -> LayoutRect {
        let Some(scroll_id) = scroll_id else {
            return rect;
        };

        // Start calculate the size of visual area is affected by transform, filter and clip.
        let mut visual_rect = rect;
        let mut parent_clip_id = clip_id;

        // 1.If there is clip CSS in the stacking context created by transform, apply it.
        if let Some(clip_node) = &self
            .clip_tree
            .get_node(&parent_clip_id)
            .filter(|node| node.parent_scroll_node_id == scroll_id)
        {
            visual_rect =
                self.apply_clip_to_visual_rect(visual_rect, clip_node, &mut parent_clip_id);
        }

        // 2.Apply transform CSS.
        let mut parent_scroll_id = None;
        visual_rect = self.apply_transform_to_visual_rect(
            visual_rect,
            scroll_id,
            &mut parent_scroll_id,
            scroll_tree,
        );

        if visual_rect.is_empty() {
            visual_rect
        } else {
            self.compute_visual_rect_with_scroll_clip(
                visual_rect,
                parent_scroll_id,
                parent_clip_id,
                scroll_tree,
            )
        }
    }

    fn apply_clip_to_visual_rect(
        &self,
        rect: LayoutRect,
        clip_node: &Clip,
        parent_clip_id: &mut ClipId,
    ) -> LayoutRect {
        if clip_node.id == ClipId::INVALID {
            *parent_clip_id = clip_node.id;
            return rect;
        }
        *parent_clip_id = clip_node.parent_clip_id;
        clip_node
            .rect
            .intersection(&rect)
            .unwrap_or(LayoutRect::zero())
    }

    fn apply_transform_to_visual_rect(
        &self,
        rect: LayoutRect,
        scroll_id: ScrollTreeNodeId,
        parent_scroll_id: &mut Option<ScrollTreeNodeId>,
        scroll_tree: &ScrollTree,
    ) -> LayoutRect {
        let scroll_node = scroll_tree.get_node(&scroll_id);
        let visual_rect = match &scroll_node.info {
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
        };
        *parent_scroll_id = scroll_node.parent;

        visual_rect
    }
}
