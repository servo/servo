/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// TODO(gw): This contains helper traits and implementations for converting Servo display lists
//           into WebRender display lists. In the future, this step should be completely removed.
//           This might be achieved by sharing types between WR and Servo display lists, or
//           completely converting layout to directly generate WebRender display lists, for example.

use display_list::items::{ClipScrollNode, ClipScrollNodeIndex, ClipScrollNodeType};
use display_list::items::{DisplayItem, DisplayList, StackingContextType};
use msg::constellation_msg::PipelineId;
use webrender_api::{self, ClipAndScrollInfo, ClipId, DisplayListBuilder, RasterSpace};
use webrender_api::{LayoutPoint, SpecificDisplayItem};

pub trait WebRenderDisplayListConverter {
    fn convert_to_webrender(&self, pipeline_id: PipelineId) -> DisplayListBuilder;
}

trait WebRenderDisplayItemConverter {
    fn prim_info(&self) -> webrender_api::LayoutPrimitiveInfo;
    fn convert_to_webrender(
        &self,
        builder: &mut DisplayListBuilder,
        clip_scroll_nodes: &[ClipScrollNode],
        clip_ids: &mut Vec<Option<ClipId>>,
        current_clip_and_scroll_info: &mut ClipAndScrollInfo,
    );
}

impl WebRenderDisplayListConverter for DisplayList {
    fn convert_to_webrender(&self, pipeline_id: PipelineId) -> DisplayListBuilder {
        let mut builder = DisplayListBuilder::with_capacity(
            pipeline_id.to_webrender(),
            self.bounds().size,
            1024 * 1024,
        ); // 1 MB of space

        let mut current_clip_and_scroll_info = pipeline_id.root_clip_and_scroll_info();
        builder.push_clip_and_scroll_info(current_clip_and_scroll_info);

        let mut clip_ids = Vec::with_capacity(self.clip_scroll_nodes.len());
        clip_ids.resize(self.clip_scroll_nodes.len(), None);

        // We need to add the WebRender root reference frame and root scroll node ids
        // here manually, because WebRender creates these automatically.
        let webrender_pipeline = pipeline_id.to_webrender();
        clip_ids[0] = Some(ClipId::root_reference_frame(webrender_pipeline));
        clip_ids[1] = Some(ClipId::root_scroll_node(webrender_pipeline));

        for item in &self.list {
            item.convert_to_webrender(
                &mut builder,
                &self.clip_scroll_nodes,
                &mut clip_ids,
                &mut current_clip_and_scroll_info,
            );
        }
        builder
    }
}

impl WebRenderDisplayItemConverter for DisplayItem {
    fn prim_info(&self) -> webrender_api::LayoutPrimitiveInfo {
        let tag = match self.base().metadata.pointing {
            Some(cursor) => Some((self.base().metadata.node.0 as u64, cursor)),
            None => None,
        };
        webrender_api::LayoutPrimitiveInfo {
            rect: self.base().bounds,
            clip_rect: self.base().clip_rect,
            // TODO(gw): Make use of the WR backface visibility functionality.
            is_backface_visible: true,
            tag,
        }
    }

    fn convert_to_webrender(
        &self,
        builder: &mut DisplayListBuilder,
        clip_scroll_nodes: &[ClipScrollNode],
        clip_ids: &mut Vec<Option<ClipId>>,
        current_clip_and_scroll_info: &mut ClipAndScrollInfo,
    ) {
        let get_id = |clip_ids: &[Option<ClipId>], index: ClipScrollNodeIndex| -> ClipId {
            match clip_ids[index.to_index()] {
                Some(id) => id,
                None => unreachable!("Tried to use WebRender ClipId before it was defined."),
            }
        };

        let clip_and_scroll_indices = self.base().clipping_and_scrolling;
        let scrolling_id = get_id(clip_ids, clip_and_scroll_indices.scrolling);
        let clip_and_scroll_info = match clip_and_scroll_indices.clipping {
            None => ClipAndScrollInfo::simple(scrolling_id),
            Some(index) => ClipAndScrollInfo::new(scrolling_id, get_id(clip_ids, index)),
        };

        if clip_and_scroll_info != *current_clip_and_scroll_info {
            builder.pop_clip_id();
            builder.push_clip_and_scroll_info(clip_and_scroll_info);
            *current_clip_and_scroll_info = clip_and_scroll_info;
        }

        match *self {
            DisplayItem::Rectangle(ref item) => {
                builder.push_item(SpecificDisplayItem::Rectangle(item.item), &self.prim_info());
            },
            DisplayItem::Text(ref item) => {
                builder.push_item(SpecificDisplayItem::Text(item.item), &self.prim_info());
                builder.push_iter(item.data.iter());
            },
            DisplayItem::Image(ref item) => {
                builder.push_item(SpecificDisplayItem::Image(item.item), &self.prim_info());
            },
            DisplayItem::Border(ref item) => {
                if !item.data.is_empty() {
                    builder.push_stops(item.data.as_ref());
                }
                builder.push_item(SpecificDisplayItem::Border(item.item), &self.prim_info());
            },
            DisplayItem::Gradient(ref item) => {
                builder.push_stops(item.data.as_ref());
                builder.push_item(SpecificDisplayItem::Gradient(item.item), &self.prim_info());
            },
            DisplayItem::RadialGradient(ref item) => {
                builder.push_stops(item.data.as_ref());
                builder.push_item(
                    SpecificDisplayItem::RadialGradient(item.item),
                    &self.prim_info(),
                );
            },
            DisplayItem::Line(ref item) => {
                builder.push_item(SpecificDisplayItem::Line(item.item), &self.prim_info());
            },
            DisplayItem::BoxShadow(ref item) => {
                builder.push_item(SpecificDisplayItem::BoxShadow(item.item), &self.prim_info());
            },
            DisplayItem::PushTextShadow(ref item) => {
                builder.push_shadow(&self.prim_info(), item.shadow);
            },
            DisplayItem::PopAllTextShadows(_) => {
                builder.pop_all_shadows();
            },
            DisplayItem::Iframe(ref item) => {
                builder.push_iframe(&self.prim_info(), item.iframe.to_webrender(), true);
            },
            DisplayItem::PushStackingContext(ref item) => {
                let stacking_context = &item.stacking_context;
                debug_assert_eq!(stacking_context.context_type, StackingContextType::Real);

                let mut info = webrender_api::LayoutPrimitiveInfo::new(stacking_context.bounds);
                if let Some(frame_index) = stacking_context.established_reference_frame {
                    debug_assert!(
                        stacking_context.transform.is_some() ||
                            stacking_context.perspective.is_some()
                    );

                    let clip_id = builder.push_reference_frame(
                        &info.clone(),
                        stacking_context.transform.map(Into::into),
                        stacking_context.perspective,
                    );
                    clip_ids[frame_index.to_index()] = Some(clip_id);

                    info.rect.origin = LayoutPoint::zero();
                    info.clip_rect.origin = LayoutPoint::zero();
                    builder.push_clip_id(clip_id);
                }

                builder.push_stacking_context(
                    &info,
                    None,
                    stacking_context.transform_style,
                    stacking_context.mix_blend_mode,
                    stacking_context.filters.clone(),
                    RasterSpace::Screen,
                );

                if stacking_context.established_reference_frame.is_some() {
                    builder.pop_clip_id();
                }
            },
            DisplayItem::PopStackingContext(_) => builder.pop_stacking_context(),
            DisplayItem::DefineClipScrollNode(ref item) => {
                let node = &clip_scroll_nodes[item.node_index.to_index()];
                let parent_id = get_id(clip_ids, node.parent_index);
                let item_rect = node.clip.main;

                let webrender_id = match node.node_type {
                    ClipScrollNodeType::Clip => builder.define_clip_with_parent(
                        parent_id,
                        item_rect,
                        node.clip.complex.clone(),
                        None,
                    ),
                    ClipScrollNodeType::ScrollFrame(scroll_sensitivity, external_id) => builder
                        .define_scroll_frame_with_parent(
                            parent_id,
                            Some(external_id),
                            node.content_rect,
                            node.clip.main,
                            node.clip.complex.clone(),
                            None,
                            scroll_sensitivity,
                        ),
                    ClipScrollNodeType::StickyFrame(ref sticky_data) => {
                        // TODO: Add define_sticky_frame_with_parent to WebRender.
                        builder.push_clip_id(parent_id);
                        let id = builder.define_sticky_frame(
                            item_rect,
                            sticky_data.margins,
                            sticky_data.vertical_offset_bounds,
                            sticky_data.horizontal_offset_bounds,
                            webrender_api::LayoutVector2D::zero(),
                        );
                        builder.pop_clip_id();
                        id
                    },
                    ClipScrollNodeType::Placeholder => {
                        unreachable!("Found DefineClipScrollNode for Placeholder type node.");
                    },
                };

                clip_ids[item.node_index.to_index()] = Some(webrender_id);
            },
        }
    }
}
