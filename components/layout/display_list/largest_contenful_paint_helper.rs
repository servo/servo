/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base::id::ScrollTreeNodeId;
use compositing_traits::display_list::{ScrollTree, SpatialTreeNodeInfo};
use compositing_traits::largest_contentful_paint_record::LCPCandidateRecord;
use style::values::computed::Filter;
use webrender_api::units::{
    LayoutPoint, LayoutRect, LayoutSideOffsets, LayoutSize, LayoutVector2D,
};
use webrender_api::{Epoch, ImageKey};

use crate::display_list::clip::{ClipId, StackingContextTreeClipStore};
use crate::display_list::effect::{EffectNodeId, StackingContextTreeEffectStore};

pub(crate) struct LargestContentfulPaintHelper<'a> {
    pub clip_tree: &'a StackingContextTreeClipStore,

    pub effect_tree: &'a StackingContextTreeEffectStore,

    pub current_scroll_node_id: ScrollTreeNodeId,

    pub current_clip_id: ClipId,

    pub current_effect_id: EffectNodeId,

    pub lcp_record: &'a mut LCPCandidateRecord,
}

impl<'a> LargestContentfulPaintHelper<'a> {
    pub fn update_current_node_id(
        &mut self,
        next_effect_id: EffectNodeId,
        next_scroll_node_id: ScrollTreeNodeId,
        next_clip_id: ClipId,
    ) {
        self.current_effect_id = next_effect_id;
        self.current_scroll_node_id = next_scroll_node_id;
        self.current_clip_id = next_clip_id;
    }

    pub fn record_image(
        &mut self,
        tag: usize,
        visual_rect: &LayoutRect,
        image_key: &ImageKey,
        epoch: Epoch,
    ) {
        self.lcp_record
            .record_image(tag, visual_rect, image_key, epoch);
    }

    pub fn record_text(
        &mut self,
        tag: usize,
        text_block_parent: usize,
        visual_rect: &LayoutRect,
        epoch: Epoch,
    ) {
        self.lcp_record
            .record_text(tag, text_block_parent, visual_rect, epoch);
    }

    /// Calculate the size of visual area in viewport. Because the parent elements will affect children,
    /// We need track these css up to the root node.
    pub fn compute_visual_rect(
        &self,
        rect: LayoutRect,
        scroll_tree: &ScrollTree,
        viewport_size: LayoutSize,
    ) -> LayoutRect {
        let visual_rect = self.compute_visual_rect_with_effect_scroll_clip(
            rect,
            self.current_effect_id,
            Some(self.current_scroll_node_id),
            self.current_clip_id,
            scroll_tree,
        );
        let viewport = LayoutRect::from_size(viewport_size);

        visual_rect
            .intersection(&viewport)
            .unwrap_or(LayoutRect::zero())
    }
    fn compute_visual_rect_with_effect_scroll_clip(
        &self,
        rect: LayoutRect,
        effect_id: EffectNodeId,
        scroll_id: Option<ScrollTreeNodeId>,
        clip_id: ClipId,
        scroll_tree: &ScrollTree,
    ) -> LayoutRect {
        if scroll_id.is_none() {
            return rect;
        }

        let scroll_id = scroll_id.unwrap();
        let mut next_effect_id = EffectNodeId::INVALID;
        let mut next_scroll_id = None;
        let mut next_clip_id = ClipId::INVALID;

        // The size of visual area is affected by transform, filter and clip.
        let visual_rect =
            self.apply_effect_to_visual_rect(rect, effect_id, scroll_id, &mut next_effect_id);
        let (visual_rect, origin) = self.apply_transform_to_visual_rect(
            visual_rect,
            scroll_id,
            &mut next_scroll_id,
            scroll_tree,
        );
        let visual_rect = self.apply_clip_to_visual_rect(
            visual_rect,
            origin,
            clip_id,
            scroll_id,
            &mut next_clip_id,
        );

        if visual_rect.is_empty() {
            visual_rect
        } else {
            self.compute_visual_rect_with_effect_scroll_clip(
                rect,
                next_effect_id,
                next_scroll_id,
                next_clip_id,
                scroll_tree,
            )
        }
    }

    fn apply_clip_to_visual_rect(
        &self,
        rect: LayoutRect,
        origin: LayoutPoint,
        clip_id: ClipId,
        scroll_id: ScrollTreeNodeId,
        next_clip_id: &mut ClipId,
    ) -> LayoutRect {
        if clip_id == ClipId::INVALID {
            *next_clip_id = clip_id;
            return rect;
        }

        let clip_node = &self.clip_tree.0[clip_id.0];
        if clip_node.parent_scroll_node_id == scroll_id {
            *next_clip_id = clip_node.parent_clip_id;
            let clip_rect = clip_node.rect.translate(origin.to_vector());
            clip_rect.intersection(&rect).unwrap_or(LayoutRect::zero())
        } else {
            *next_clip_id = clip_id;
            rect
        }
    }

    fn apply_transform_to_visual_rect(
        &self,
        rect: LayoutRect,
        scroll_id: ScrollTreeNodeId,
        next_scroll_id: &mut Option<ScrollTreeNodeId>,
        scroll_tree: &ScrollTree,
    ) -> (LayoutRect, LayoutPoint) {
        let scroll_node = scroll_tree.get_node(&scroll_id);
        let mut origin = LayoutPoint::zero();
        let visual_rect = match &scroll_node.info {
            SpatialTreeNodeInfo::ReferenceFrame(frame_node_info) => {
                origin = frame_node_info.origin;
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
        *next_scroll_id = scroll_node.parent;

        (visual_rect, origin)
    }

    fn apply_effect_to_visual_rect(
        &self,
        rect: LayoutRect,
        effect_id: EffectNodeId,
        scroll_id: ScrollTreeNodeId,
        next_effect_id: &mut EffectNodeId,
    ) -> LayoutRect {
        if effect_id == EffectNodeId::INVALID {
            return rect;
        }

        let effect_node = self.effect_tree.get(effect_id).unwrap();
        // The place where the effect is actually applied.
        if effect_node.scroll_tree_node_id != scroll_id {
            return rect;
        }

        let effects = effect_node.style.get_effects();
        // If the opacity of the element is zero, it will be seen in viewport.
        if effects.opacity == 0.0 {
            return LayoutRect::zero();
        }

        let mut current_rect = rect;
        for filter in effects.filter.0.iter() {
            current_rect = match filter {
                Filter::Blur(radius) => {
                    let (spread_x, spread_y) = compute_spread(radius.px());
                    let offsets = LayoutSideOffsets::new(spread_y, spread_x, spread_y, spread_x);
                    current_rect.outer_box(offsets)
                },
                Filter::DropShadow(shadow) => {
                    let (spread_x, spread_y) = compute_spread(shadow.blur.px());
                    let offsets = LayoutSideOffsets::new(spread_y, spread_x, spread_y, spread_x);
                    let result = rect.inner_box(offsets).translate(LayoutVector2D::new(
                        shadow.horizontal.px(),
                        shadow.vertical.px(),
                    ));
                    result.union(&current_rect)
                },
                Filter::Opacity(opacity) if opacity.0 == 0.0 => {
                    return LayoutRect::zero();
                },
                _ => current_rect,
            };
        }
        *next_effect_id = effect_node.parent_id;

        current_rect
    }
}

fn map_std_deviation(std_deviation: f32) -> LayoutVector2D {
    let sigma = LayoutVector2D::new(std_deviation, std_deviation);
    sigma * 3.0
}

fn compute_spread(std_deviation: f32) -> (f32, f32) {
    let spread = map_std_deviation(std_deviation);
    (spread.x.abs(), spread.y.abs())
}
