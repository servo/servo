/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// TODO(gw): This contains helper traits and implementations for converting Servo display lists
//           into WebRender display lists. In the future, this step should be completely removed.
//           This might be achieved by sharing types between WR and Servo display lists, or
//           completely converting layout to directly generate WebRender display lists, for example.

use crate::display_list::items::{BaseDisplayItem, ClipScrollNode, ClipScrollNodeType, ClipType};
use crate::display_list::items::{DisplayItem, DisplayList, StackingContextType};
use msg::constellation_msg::PipelineId;
use script_traits::compositor::CompositorDisplayListInfo;
use webrender_api::units::{LayoutPoint, LayoutVector2D};
use webrender_api::{
    self, ClipId, CommonItemProperties, DisplayItem as WrDisplayItem, DisplayListBuilder,
    PrimitiveFlags, PropertyBinding, PushStackingContextDisplayItem, RasterSpace,
    ReferenceFrameKind, SpaceAndClipInfo, SpatialId, StackingContext,
};

struct ClipScrollState {
    clip_ids: Vec<Option<ClipId>>,
    spatial_ids: Vec<Option<SpatialId>>,
    active_clip_id: ClipId,
    active_spatial_id: SpatialId,
    compositor_info: CompositorDisplayListInfo,
}

impl ClipScrollState {
    fn new(size: usize, pipeline_id: webrender_api::PipelineId) -> Self {
        let root_clip_id = ClipId::root(pipeline_id);
        let root_scroll_node_id = SpatialId::root_scroll_node(pipeline_id);
        let root_reference_frame_id = SpatialId::root_reference_frame(pipeline_id);
        let mut state = ClipScrollState {
            clip_ids: vec![None; size],
            spatial_ids: vec![None; size],
            active_clip_id: root_clip_id,
            active_spatial_id: root_scroll_node_id,
            compositor_info: CompositorDisplayListInfo::default(),
        };

        // We need to register the WebRender root reference frame and root scroll node ids
        // here manually, because WebRender and the CompositorDisplayListInfo create them
        // automatically. We also follow the "old" WebRender API for clip/scroll for now,
        // hence both arrays are initialized based on FIRST_SPATIAL_NODE_INDEX, while
        // FIRST_CLIP_NODE_INDEX is not taken into account.
        state.spatial_ids[0] = Some(root_reference_frame_id);
        state.spatial_ids[1] = Some(root_scroll_node_id);

        state.add_clip_node_mapping(0, root_clip_id);
        state.add_clip_node_mapping(1, root_clip_id);

        state
    }

    fn webrender_clip_id_for_index(&mut self, index: usize) -> ClipId {
        self.clip_ids[index].expect("Tried to use WebRender parent ClipId before it was defined.")
    }

    fn webrender_spatial_id_for_index(&mut self, index: usize) -> SpatialId {
        self.spatial_ids[index]
            .expect("Tried to use WebRender parent SpatialId before it was defined.")
    }

    fn add_clip_node_mapping(&mut self, index: usize, webrender_id: ClipId) {
        self.clip_ids[index] = Some(webrender_id);
    }

    fn register_spatial_node(&mut self, index: usize, webrender_id: SpatialId) {
        self.spatial_ids[index] = Some(webrender_id);
    }

    fn add_spatial_node_mapping_to_parent_index(&mut self, index: usize, parent_index: usize) {
        self.spatial_ids[index] = self.spatial_ids[parent_index];
    }
}

/// Contentful paint, for the purpose of
/// https://w3c.github.io/paint-timing/#first-contentful-paint
/// (i.e. the display list contains items of type text,
/// image, non-white canvas or SVG). Used by metrics.
pub struct IsContentful(pub bool);

impl DisplayList {
    pub fn convert_to_webrender(
        &mut self,
        pipeline_id: PipelineId,
    ) -> (DisplayListBuilder, CompositorDisplayListInfo, IsContentful) {
        let webrender_pipeline = pipeline_id.to_webrender();
        let mut state = ClipScrollState::new(self.clip_scroll_nodes.len(), webrender_pipeline);

        let mut builder = DisplayListBuilder::with_capacity(
            webrender_pipeline,
            self.bounds().size,
            1024 * 1024, // 1 MB of space
        );

        let mut is_contentful = IsContentful(false);
        for item in &mut self.list {
            is_contentful.0 |= item
                .convert_to_webrender(&self.clip_scroll_nodes, &mut state, &mut builder)
                .0;
        }

        (builder, state.compositor_info, is_contentful)
    }
}

impl DisplayItem {
    fn convert_to_webrender(
        &mut self,
        clip_scroll_nodes: &[ClipScrollNode],
        state: &mut ClipScrollState,
        builder: &mut DisplayListBuilder,
    ) -> IsContentful {
        // Note: for each time of a display item, if we register one of `clip_ids` or `spatial_ids`,
        // we also register the other one as inherited from the current state or the stack.
        // This is not an ideal behavior, but it is compatible with the old WebRender model
        // of the clip-scroll tree.

        let clip_and_scroll_indices = self.base().clipping_and_scrolling;
        trace!("converting {:?}", clip_and_scroll_indices);

        let current_scrolling_index = clip_and_scroll_indices.scrolling.to_index();
        let cur_spatial_id = state.webrender_spatial_id_for_index(current_scrolling_index);
        if cur_spatial_id != state.active_spatial_id {
            state.active_spatial_id = cur_spatial_id;
        }

        let internal_clip_id = clip_and_scroll_indices
            .clipping
            .unwrap_or(clip_and_scroll_indices.scrolling);
        let cur_clip_id = state.webrender_clip_id_for_index(internal_clip_id.to_index());
        if cur_clip_id != state.active_clip_id {
            state.active_clip_id = cur_clip_id;
        }

        let mut build_common_item_properties = |base: &BaseDisplayItem| {
            let tag = match base.metadata.cursor {
                Some(cursor) => {
                    let hit_test_index = state
                        .compositor_info
                        .add_hit_test_info(base.metadata.node.0 as u64, Some(cursor));
                    Some((hit_test_index as u64, 0u16))
                },
                None => None,
            };
            CommonItemProperties {
                clip_rect: base.clip_rect,
                spatial_id: state.active_spatial_id,
                clip_id: state.active_clip_id,
                // TODO(gw): Make use of the WR backface visibility functionality.
                flags: PrimitiveFlags::default(),
                hit_info: tag,
            }
        };

        match *self {
            DisplayItem::Rectangle(ref mut item) => {
                item.item.common = build_common_item_properties(&item.base);
                builder.push_item(&WrDisplayItem::Rectangle(item.item));
                IsContentful(false)
            },
            DisplayItem::Text(ref mut item) => {
                item.item.common = build_common_item_properties(&item.base);
                builder.push_item(&WrDisplayItem::Text(item.item));
                builder.push_iter(item.data.iter());
                IsContentful(true)
            },
            DisplayItem::Image(ref mut item) => {
                item.item.common = build_common_item_properties(&item.base);
                builder.push_item(&WrDisplayItem::Image(item.item));
                IsContentful(true)
            },
            DisplayItem::RepeatingImage(ref mut item) => {
                item.item.common = build_common_item_properties(&item.base);
                builder.push_item(&WrDisplayItem::RepeatingImage(item.item));
                IsContentful(true)
            },
            DisplayItem::Border(ref mut item) => {
                item.item.common = build_common_item_properties(&item.base);
                if !item.data.is_empty() {
                    builder.push_stops(item.data.as_ref());
                }
                builder.push_item(&WrDisplayItem::Border(item.item));
                IsContentful(false)
            },
            DisplayItem::Gradient(ref mut item) => {
                item.item.common = build_common_item_properties(&item.base);
                builder.push_stops(item.data.as_ref());
                builder.push_item(&WrDisplayItem::Gradient(item.item));
                IsContentful(false)
            },
            DisplayItem::RadialGradient(ref mut item) => {
                item.item.common = build_common_item_properties(&item.base);
                builder.push_stops(item.data.as_ref());
                builder.push_item(&WrDisplayItem::RadialGradient(item.item));
                IsContentful(false)
            },
            DisplayItem::Line(ref mut item) => {
                item.item.common = build_common_item_properties(&item.base);
                builder.push_item(&WrDisplayItem::Line(item.item));
                IsContentful(false)
            },
            DisplayItem::BoxShadow(ref mut item) => {
                item.item.common = build_common_item_properties(&item.base);
                builder.push_item(&WrDisplayItem::BoxShadow(item.item));
                IsContentful(false)
            },
            DisplayItem::PushTextShadow(ref mut item) => {
                let common = build_common_item_properties(&item.base);
                builder.push_shadow(
                    &SpaceAndClipInfo {
                        spatial_id: common.spatial_id,
                        clip_id: common.clip_id,
                    },
                    item.shadow,
                    true,
                );
                IsContentful(false)
            },
            DisplayItem::PopAllTextShadows(_) => {
                builder.push_item(&WrDisplayItem::PopAllShadows);
                IsContentful(false)
            },
            DisplayItem::Iframe(ref mut item) => {
                let common = build_common_item_properties(&item.base);
                builder.push_iframe(
                    item.bounds,
                    common.clip_rect,
                    &SpaceAndClipInfo {
                        spatial_id: common.spatial_id,
                        clip_id: common.clip_id,
                    },
                    item.iframe.to_webrender(),
                    true,
                );
                IsContentful(false)
            },
            DisplayItem::PushStackingContext(ref item) => {
                let stacking_context = &item.stacking_context;
                debug_assert_eq!(stacking_context.context_type, StackingContextType::Real);

                //let mut info = webrender_api::LayoutPrimitiveInfo::new(stacking_context.bounds);
                let mut bounds = stacking_context.bounds;
                let spatial_id =
                    if let Some(frame_index) = stacking_context.established_reference_frame {
                        let (transform, ref_frame) =
                            match (stacking_context.transform, stacking_context.perspective) {
                                (None, Some(p)) => (
                                    p,
                                    ReferenceFrameKind::Perspective {
                                        scrolling_relative_to: None,
                                    },
                                ),
                                (Some(t), None) => (t, ReferenceFrameKind::Transform),
                                (Some(t), Some(p)) => (
                                    p.then(&t),
                                    ReferenceFrameKind::Perspective {
                                        scrolling_relative_to: None,
                                    },
                                ),
                                (None, None) => unreachable!(),
                            };

                        let new_spatial_id = builder.push_reference_frame(
                            stacking_context.bounds.origin,
                            state.active_spatial_id,
                            stacking_context.transform_style,
                            PropertyBinding::Value(transform),
                            ref_frame,
                        );

                        let index = frame_index.to_index();
                        state.add_clip_node_mapping(index, cur_clip_id);
                        state.register_spatial_node(index, new_spatial_id);

                        bounds.origin = LayoutPoint::zero();
                        new_spatial_id
                    } else {
                        state.active_spatial_id
                    };

                if !stacking_context.filters.is_empty() {
                    builder.push_item(&WrDisplayItem::SetFilterOps);
                    builder.push_iter(&stacking_context.filters);
                }

                // TODO(jdm): WebRender now requires us to create stacking context items
                //            with the IS_BLEND_CONTAINER flag enabled if any children
                //            of the stacking context have a blend mode applied.
                //            This will require additional tracking during layout
                //            before we start collecting stacking contexts so that
                //            information will be available when we reach this point.
                let wr_item = PushStackingContextDisplayItem {
                    origin: bounds.origin,
                    spatial_id,
                    prim_flags: PrimitiveFlags::default(),
                    stacking_context: StackingContext {
                        transform_style: stacking_context.transform_style,
                        mix_blend_mode: stacking_context.mix_blend_mode,
                        clip_id: None,
                        raster_space: RasterSpace::Screen,
                        flags: Default::default(),
                    },
                };

                builder.push_item(&WrDisplayItem::PushStackingContext(wr_item));
                IsContentful(false)
            },
            DisplayItem::PopStackingContext(ref item) => {
                builder.pop_stacking_context();
                if item.established_reference_frame {
                    builder.pop_reference_frame();
                }
                IsContentful(false)
            },
            DisplayItem::DefineClipScrollNode(ref mut item) => {
                let index = item.node_index.to_index();
                let node = &clip_scroll_nodes[index];
                let item_rect = node.clip.main;

                let parent_index = node.parent_index.to_index();
                let parent_spatial_id = state.webrender_spatial_id_for_index(parent_index);
                let parent_clip_id = state.webrender_clip_id_for_index(parent_index);

                match node.node_type {
                    ClipScrollNodeType::Clip(clip_type) => {
                        let space_and_clip_info = SpaceAndClipInfo {
                            clip_id: parent_clip_id,
                            spatial_id: parent_spatial_id,
                        };
                        let clip_id = match clip_type {
                            ClipType::Rect => {
                                builder.define_clip_rect(&space_and_clip_info, item_rect)
                            },
                            ClipType::Rounded(complex) => {
                                builder.define_clip_rounded_rect(&space_and_clip_info, complex)
                            },
                        };

                        state.add_clip_node_mapping(index, clip_id);
                        state.add_spatial_node_mapping_to_parent_index(index, parent_index);
                    },
                    ClipScrollNodeType::ScrollFrame(scroll_sensitivity, external_id) => {
                        let space_clip_info = builder.define_scroll_frame(
                            &SpaceAndClipInfo {
                                clip_id: parent_clip_id,
                                spatial_id: parent_spatial_id,
                            },
                            Some(external_id),
                            node.content_rect,
                            item_rect,
                            scroll_sensitivity,
                            LayoutVector2D::zero(),
                        );

                        state.register_spatial_node(index, space_clip_info.spatial_id);
                        state.add_clip_node_mapping(index, space_clip_info.clip_id);
                    },
                    ClipScrollNodeType::StickyFrame(ref sticky_data) => {
                        // TODO: Add define_sticky_frame_with_parent to WebRender.
                        let id = builder.define_sticky_frame(
                            parent_spatial_id,
                            item_rect,
                            sticky_data.margins,
                            sticky_data.vertical_offset_bounds,
                            sticky_data.horizontal_offset_bounds,
                            LayoutVector2D::zero(),
                        );

                        state.add_clip_node_mapping(index, parent_clip_id);
                        state.register_spatial_node(index, id);
                    },
                    ClipScrollNodeType::Placeholder => {
                        unreachable!("Found DefineClipScrollNode for Placeholder type node.");
                    },
                };
                IsContentful(false)
            },
        }
    }
}
