/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// TODO(gw): This contains helper traits and implementations for converting Servo display lists
//           into WebRender display lists. In the future, this step should be completely removed.
//           This might be achieved by sharing types between WR and Servo display lists, or
//           completely converting layout to directly generate WebRender display lists, for example.

use crate::display_list::items::{ClipScrollNode, ClipScrollNodeType};
use crate::display_list::items::{DisplayItem, DisplayList, StackingContextType};
use msg::constellation_msg::PipelineId;
use webrender_api::{self, ClipId, DisplayListBuilder, RasterSpace, SpaceAndClipInfo, SpatialId};
use webrender_api::{LayoutPoint, SpecificDisplayItem};

pub trait WebRenderDisplayListConverter {
    fn convert_to_webrender(&self, pipeline_id: PipelineId) -> DisplayListBuilder;
}

struct ClipScrollState {
    clip_ids: Vec<Option<ClipId>>,
    spatial_ids: Vec<Option<SpatialId>>,
    active_clip_id: ClipId,
    active_spatial_id: SpatialId,
}

trait WebRenderDisplayItemConverter {
    fn prim_info(&self) -> webrender_api::LayoutPrimitiveInfo;
    fn convert_to_webrender(
        &self,
        clip_scroll_nodes: &[ClipScrollNode],
        state: &mut ClipScrollState,
        builder: &mut DisplayListBuilder,
    );
}

impl WebRenderDisplayListConverter for DisplayList {
    fn convert_to_webrender(&self, pipeline_id: PipelineId) -> DisplayListBuilder {
        let mut clip_ids = vec![None; self.clip_scroll_nodes.len()];
        let mut spatial_ids = vec![None; self.clip_scroll_nodes.len()];

        // We need to add the WebRender root reference frame and root scroll node ids
        // here manually, because WebRender creates these automatically.
        // We also follow the "old" WebRender API for clip/scroll for now,
        // hence both arrays are initialized based on FIRST_SPATIAL_NODE_INDEX,
        // while FIRST_CLIP_NODE_INDEX is not taken into account.

        let webrender_pipeline = pipeline_id.to_webrender();
        clip_ids[0] = Some(ClipId::root(webrender_pipeline));
        clip_ids[1] = Some(ClipId::root(webrender_pipeline));
        spatial_ids[0] = Some(SpatialId::root_reference_frame(webrender_pipeline));
        spatial_ids[1] = Some(SpatialId::root_scroll_node(webrender_pipeline));

        let mut state = ClipScrollState {
            clip_ids,
            spatial_ids,
            active_clip_id: ClipId::root(webrender_pipeline),
            active_spatial_id: SpatialId::root_scroll_node(webrender_pipeline),
        };

        let mut builder = DisplayListBuilder::with_capacity(
            webrender_pipeline,
            self.bounds().size,
            1024 * 1024, // 1 MB of space
        );

        for item in &self.list {
            item.convert_to_webrender(&self.clip_scroll_nodes, &mut state, &mut builder);
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
        clip_scroll_nodes: &[ClipScrollNode],
        state: &mut ClipScrollState,
        builder: &mut DisplayListBuilder,
    ) {
        // Note: for each time of a display item, if we register one of `clip_ids` or `spatial_ids`,
        // we also register the other one as inherited from the current state or the stack.
        // This is not an ideal behavior, but it is compatible with the old WebRender model
        // of the clip-scroll tree.

        let clip_and_scroll_indices = self.base().clipping_and_scrolling;
        trace!("converting {:?}", clip_and_scroll_indices);

        let cur_spatial_id = state.spatial_ids[clip_and_scroll_indices.scrolling.to_index()]
            .expect("Tried to use WebRender SpatialId before it was defined.");
        if cur_spatial_id != state.active_spatial_id {
            state.active_spatial_id = cur_spatial_id;
        }

        let internal_clip_id = clip_and_scroll_indices
            .clipping
            .unwrap_or(clip_and_scroll_indices.scrolling);
        let cur_clip_id = state.clip_ids[internal_clip_id.to_index()]
            .expect("Tried to use WebRender ClipId before it was defined.");
        if cur_clip_id != state.active_clip_id {
            state.active_clip_id = cur_clip_id;
        }

        let space_clip_info = SpaceAndClipInfo {
            spatial_id: state.active_spatial_id,
            clip_id: state.active_clip_id,
        };
        match *self {
            DisplayItem::Rectangle(ref item) => {
                builder.push_item(
                    &SpecificDisplayItem::Rectangle(item.item),
                    &self.prim_info(),
                    &space_clip_info,
                );
            },
            DisplayItem::Text(ref item) => {
                builder.push_item(
                    &SpecificDisplayItem::Text(item.item),
                    &self.prim_info(),
                    &space_clip_info,
                );
                builder.push_iter(item.data.iter());
            },
            DisplayItem::Image(ref item) => {
                builder.push_item(
                    &SpecificDisplayItem::Image(item.item),
                    &self.prim_info(),
                    &space_clip_info,
                );
            },
            DisplayItem::Border(ref item) => {
                if !item.data.is_empty() {
                    builder.push_stops(item.data.as_ref());
                }
                builder.push_item(
                    &SpecificDisplayItem::Border(item.item),
                    &self.prim_info(),
                    &space_clip_info,
                );
            },
            DisplayItem::Gradient(ref item) => {
                builder.push_stops(item.data.as_ref());
                builder.push_item(
                    &SpecificDisplayItem::Gradient(item.item),
                    &self.prim_info(),
                    &space_clip_info,
                );
            },
            DisplayItem::RadialGradient(ref item) => {
                builder.push_stops(item.data.as_ref());
                builder.push_item(
                    &SpecificDisplayItem::RadialGradient(item.item),
                    &self.prim_info(),
                    &space_clip_info,
                );
            },
            DisplayItem::Line(ref item) => {
                builder.push_item(
                    &SpecificDisplayItem::Line(item.item),
                    &self.prim_info(),
                    &space_clip_info,
                );
            },
            DisplayItem::BoxShadow(ref item) => {
                builder.push_item(
                    &SpecificDisplayItem::BoxShadow(item.item),
                    &self.prim_info(),
                    &space_clip_info,
                );
            },
            DisplayItem::PushTextShadow(ref item) => {
                builder.push_shadow(&self.prim_info(), &space_clip_info, item.shadow);
            },
            DisplayItem::PopAllTextShadows(_) => {
                builder.pop_all_shadows();
            },
            DisplayItem::Iframe(ref item) => {
                builder.push_iframe(
                    &self.prim_info(),
                    &space_clip_info,
                    item.iframe.to_webrender(),
                    true,
                );
            },
            DisplayItem::PushStackingContext(ref item) => {
                let stacking_context = &item.stacking_context;
                debug_assert_eq!(stacking_context.context_type, StackingContextType::Real);

                let mut info = webrender_api::LayoutPrimitiveInfo::new(stacking_context.bounds);
                let spatial_id =
                    if let Some(frame_index) = stacking_context.established_reference_frame {
                        debug_assert!(
                            stacking_context.transform.is_some() ||
                                stacking_context.perspective.is_some()
                        );

                        let spatial_id = builder.push_reference_frame(
                            &stacking_context.bounds,
                            state.active_spatial_id,
                            stacking_context.transform_style,
                            stacking_context.transform.map(Into::into),
                            stacking_context.perspective,
                        );
                        state.spatial_ids[frame_index.to_index()] = Some(spatial_id);
                        state.clip_ids[frame_index.to_index()] = Some(cur_clip_id);

                        info.rect.origin = LayoutPoint::zero();
                        info.clip_rect.origin = LayoutPoint::zero();
                        spatial_id
                    } else {
                        state.active_spatial_id
                    };

                builder.push_stacking_context(
                    &info,
                    spatial_id,
                    None,
                    stacking_context.transform_style,
                    stacking_context.mix_blend_mode,
                    &stacking_context.filters,
                    RasterSpace::Screen,
                );
            },
            DisplayItem::PopStackingContext(_) => builder.pop_stacking_context(),
            DisplayItem::DefineClipScrollNode(ref item) => {
                let node = &clip_scroll_nodes[item.node_index.to_index()];
                let item_rect = node.clip.main;

                let parent_spatial_id = state.spatial_ids[node.parent_index.to_index()]
                    .expect("Tried to use WebRender parent SpatialId before it was defined.");
                let parent_clip_id = state.clip_ids[node.parent_index.to_index()]
                    .expect("Tried to use WebRender parent ClipId before it was defined.");

                match node.node_type {
                    ClipScrollNodeType::Clip => {
                        let id = builder.define_clip(
                            &SpaceAndClipInfo {
                                clip_id: parent_clip_id,
                                spatial_id: parent_spatial_id,
                            },
                            item_rect,
                            node.clip.complex.clone(),
                            None,
                        );

                        state.spatial_ids[item.node_index.to_index()] = Some(parent_spatial_id);
                        state.clip_ids[item.node_index.to_index()] = Some(id);
                    },
                    ClipScrollNodeType::ScrollFrame(scroll_sensitivity, external_id) => {
                        let space_clip_info = builder.define_scroll_frame(
                            &SpaceAndClipInfo {
                                clip_id: parent_clip_id,
                                spatial_id: parent_spatial_id,
                            },
                            Some(external_id),
                            node.content_rect,
                            node.clip.main,
                            node.clip.complex.clone(),
                            None,
                            scroll_sensitivity,
                        );

                        state.clip_ids[item.node_index.to_index()] = Some(space_clip_info.clip_id);
                        state.spatial_ids[item.node_index.to_index()] =
                            Some(space_clip_info.spatial_id);
                    },
                    ClipScrollNodeType::StickyFrame(ref sticky_data) => {
                        // TODO: Add define_sticky_frame_with_parent to WebRender.
                        let id = builder.define_sticky_frame(
                            parent_spatial_id,
                            item_rect,
                            sticky_data.margins,
                            sticky_data.vertical_offset_bounds,
                            sticky_data.horizontal_offset_bounds,
                            webrender_api::LayoutVector2D::zero(),
                        );

                        state.spatial_ids[item.node_index.to_index()] = Some(id);
                        state.clip_ids[item.node_index.to_index()] = Some(parent_clip_id);
                    },
                    ClipScrollNodeType::Placeholder => {
                        unreachable!("Found DefineClipScrollNode for Placeholder type node.");
                    },
                };
            },
        }
    }
}
