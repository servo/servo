/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// TODO(gw): This contains helper traits and implementations for converting Servo display lists
//           into WebRender display lists. In the future, this step should be completely removed.
//           This might be achieved by sharing types between WR and Servo display lists, or
//           completely converting layout to directly generate WebRender display lists, for example.

use base::id::PipelineId;
use base::WebRenderEpochToU16;
use log::trace;
use webrender_api::units::{LayoutPoint, LayoutSize, LayoutVector2D};
use webrender_api::{
    self, ClipChainId, ClipId, CommonItemProperties, DisplayItem as WrDisplayItem,
    DisplayListBuilder, Epoch, HasScrollLinkedEffect, PrimitiveFlags, PropertyBinding, RasterSpace,
    ReferenceFrameKind, SpaceAndClipInfo, SpatialId, SpatialTreeItemKey,
};
use webrender_traits::display_list::{
    CompositorDisplayListInfo, ScrollSensitivity, ScrollTreeNodeId, ScrollableNodeInfo,
};

use crate::display_list::items::{
    BaseDisplayItem, ClipScrollNode, ClipScrollNodeType, ClipType, DisplayItem, DisplayList,
    StackingContextType,
};

struct ClipScrollState<'a> {
    clip_scroll_nodes: &'a mut Vec<ClipScrollNode>,
    compositor_info: CompositorDisplayListInfo,
}

impl<'a> ClipScrollState<'a> {
    fn new(
        clip_scroll_nodes: &'a mut Vec<ClipScrollNode>,
        compositor_info: CompositorDisplayListInfo,
    ) -> Self {
        let mut state = ClipScrollState {
            clip_scroll_nodes,
            compositor_info,
        };

        // We need to register the WebRender root reference frame and root scroll node ids
        // here manually, because WebRender and the CompositorDisplayListInfo create them
        // automatically. We also follow the "old" WebRender API for clip/scroll for now,
        // hence both arrays are initialized based on FIRST_SPATIAL_NODE_INDEX, while
        // FIRST_CLIP_NODE_INDEX is not taken into account.
        state.clip_scroll_nodes[0].scroll_node_id =
            Some(state.compositor_info.root_reference_frame_id);
        state.clip_scroll_nodes[1].scroll_node_id = Some(state.compositor_info.root_scroll_node_id);

        let root_clip_chain = ClipChainId::INVALID;
        state.add_clip_node_mapping(0, root_clip_chain);
        state.add_clip_node_mapping(1, root_clip_chain);

        state
    }

    fn webrender_clip_id_for_index(&mut self, index: usize) -> ClipChainId {
        self.clip_scroll_nodes[index]
            .clip_chain_id
            .expect("Tried to access WebRender ClipId before definining it.")
    }

    fn webrender_spatial_id_for_index(&mut self, index: usize) -> SpatialId {
        self.clip_scroll_nodes[index]
            .scroll_node_id
            .expect("Tried to use WebRender parent SpatialId before it was defined.")
            .spatial_id
    }

    fn add_clip_node_mapping(&mut self, index: usize, webrender_id: ClipChainId) {
        self.clip_scroll_nodes[index].clip_chain_id = Some(webrender_id);
    }

    fn scroll_node_id_from_index(&self, index: usize) -> ScrollTreeNodeId {
        self.clip_scroll_nodes[index]
            .scroll_node_id
            .expect("Tried to use WebRender parent SpatialId before it was defined.")
    }

    fn register_spatial_node(
        &mut self,
        index: usize,
        spatial_id: SpatialId,
        parent_index: Option<usize>,
        scroll_info: Option<ScrollableNodeInfo>,
    ) {
        let parent_scroll_node_id = parent_index.map(|index| self.scroll_node_id_from_index(index));
        self.clip_scroll_nodes[index].scroll_node_id =
            Some(self.compositor_info.scroll_tree.add_scroll_tree_node(
                parent_scroll_node_id.as_ref(),
                spatial_id,
                scroll_info,
            ));
    }

    fn add_spatial_node_mapping_to_parent_index(&mut self, index: usize, parent_index: usize) {
        self.clip_scroll_nodes[index].scroll_node_id =
            self.clip_scroll_nodes[parent_index].scroll_node_id
    }

    pub fn define_clip_chain<I>(
        &self,
        builder: &mut DisplayListBuilder,
        parent: ClipChainId,
        clips: I,
    ) -> ClipChainId
    where
        I: IntoIterator<Item = ClipId>,
        I::IntoIter: ExactSizeIterator + Clone,
    {
        // We use INVALID to mean "no clipping", but that cannot be passed as an argument
        // to `define_clip_chain()`, so it must be converted into `None`.
        let parent = match parent {
            ClipChainId::INVALID => None,
            parent => Some(parent),
        };
        builder.define_clip_chain(parent, clips)
    }
}

/// Contentful paint, for the purpose of
/// <https://w3c.github.io/paint-timing/#first-contentful-paint>
/// (i.e. the display list contains items of type text,
/// image, non-white canvas or SVG). Used by metrics.
pub struct IsContentful(pub bool);

impl DisplayList {
    pub fn convert_to_webrender(
        &mut self,
        pipeline_id: PipelineId,
        viewport_size: LayoutSize,
        epoch: Epoch,
    ) -> (DisplayListBuilder, CompositorDisplayListInfo, IsContentful) {
        let webrender_pipeline = pipeline_id.into();
        let mut builder = DisplayListBuilder::new(webrender_pipeline);
        builder.begin();

        let content_size = self.bounds().size();
        let mut state = ClipScrollState::new(
            &mut self.clip_scroll_nodes,
            CompositorDisplayListInfo::new(
                viewport_size,
                content_size,
                webrender_pipeline,
                epoch,
                ScrollSensitivity::ScriptAndInputEvents,
            ),
        );

        let mut is_contentful = IsContentful(false);
        for item in &mut self.list {
            is_contentful.0 |= item.convert_to_webrender(&mut state, &mut builder).0;
        }

        (builder, state.compositor_info, is_contentful)
    }
}

impl DisplayItem {
    fn get_spatial_tree_item_key(
        &self,
        builder: &DisplayListBuilder,
        node_index: usize,
    ) -> SpatialTreeItemKey {
        let pipeline_tag = (builder.pipeline_id.0 as u64) << 32 | builder.pipeline_id.1 as u64;
        SpatialTreeItemKey::new(pipeline_tag, node_index as u64)
    }

    fn convert_to_webrender(
        &mut self,
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
        let current_scroll_node_id = state.scroll_node_id_from_index(current_scrolling_index);

        let internal_clip_id = clip_and_scroll_indices
            .clipping
            .unwrap_or(clip_and_scroll_indices.scrolling);
        let current_clip_chain_id = state.webrender_clip_id_for_index(internal_clip_id.to_index());
        let hit_test_bounds = self.bounds().intersection(&self.base().clip_rect);

        let build_common_item_properties = |base: &BaseDisplayItem| {
            CommonItemProperties {
                clip_rect: base.clip_rect,
                spatial_id: current_scroll_node_id.spatial_id,
                clip_chain_id: current_clip_chain_id,
                // TODO(gw): Make use of the WR backface visibility functionality.
                flags: PrimitiveFlags::default(),
            }
        };

        let mut push_hit_test = |base: &BaseDisplayItem| {
            let bounds = match hit_test_bounds {
                Some(bounds) => bounds,
                None => return,
            };

            let cursor = match base.metadata.cursor {
                Some(cursor) => cursor,
                None => return,
            };

            let hit_test_index = state.compositor_info.add_hit_test_info(
                base.metadata.node.0 as u64,
                Some(cursor),
                current_scroll_node_id,
            );

            builder.push_hit_test(
                bounds,
                current_clip_chain_id,
                current_scroll_node_id.spatial_id,
                PrimitiveFlags::default(),
                (hit_test_index as u64, state.compositor_info.epoch.as_u16()),
            );
        };

        match *self {
            DisplayItem::Rectangle(ref mut item) => {
                item.item.common = build_common_item_properties(&item.base);
                push_hit_test(&item.base);
                builder.push_item(&WrDisplayItem::Rectangle(item.item));
                IsContentful(false)
            },
            DisplayItem::Text(ref mut item) => {
                item.item.common = build_common_item_properties(&item.base);
                push_hit_test(&item.base);
                builder.push_item(&WrDisplayItem::Text(item.item));
                builder.push_iter(item.data.iter());
                IsContentful(true)
            },
            DisplayItem::Image(ref mut item) => {
                item.item.common = build_common_item_properties(&item.base);
                push_hit_test(&item.base);
                builder.push_item(&WrDisplayItem::Image(item.item));
                IsContentful(true)
            },
            DisplayItem::RepeatingImage(ref mut item) => {
                item.item.common = build_common_item_properties(&item.base);
                push_hit_test(&item.base);
                builder.push_item(&WrDisplayItem::RepeatingImage(item.item));
                IsContentful(true)
            },
            DisplayItem::Border(ref mut item) => {
                item.item.common = build_common_item_properties(&item.base);
                push_hit_test(&item.base);
                if !item.data.is_empty() {
                    builder.push_stops(item.data.as_ref());
                }
                builder.push_item(&WrDisplayItem::Border(item.item));
                IsContentful(false)
            },
            DisplayItem::Gradient(ref mut item) => {
                item.item.common = build_common_item_properties(&item.base);
                push_hit_test(&item.base);
                builder.push_stops(item.data.as_ref());
                builder.push_item(&WrDisplayItem::Gradient(item.item));
                IsContentful(false)
            },
            DisplayItem::RadialGradient(ref mut item) => {
                item.item.common = build_common_item_properties(&item.base);
                push_hit_test(&item.base);
                builder.push_stops(item.data.as_ref());
                builder.push_item(&WrDisplayItem::RadialGradient(item.item));
                IsContentful(false)
            },
            DisplayItem::Line(ref mut item) => {
                item.item.common = build_common_item_properties(&item.base);
                push_hit_test(&item.base);
                builder.push_item(&WrDisplayItem::Line(item.item));
                IsContentful(false)
            },
            DisplayItem::BoxShadow(ref mut item) => {
                item.item.common = build_common_item_properties(&item.base);
                push_hit_test(&item.base);
                builder.push_item(&WrDisplayItem::BoxShadow(item.item));
                IsContentful(false)
            },
            DisplayItem::PushTextShadow(ref mut item) => {
                let common = build_common_item_properties(&item.base);
                push_hit_test(&item.base);
                builder.push_shadow(
                    &SpaceAndClipInfo {
                        spatial_id: common.spatial_id,
                        clip_chain_id: common.clip_chain_id,
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
                push_hit_test(&item.base);
                builder.push_iframe(
                    item.bounds,
                    common.clip_rect,
                    &SpaceAndClipInfo {
                        spatial_id: common.spatial_id,
                        clip_chain_id: common.clip_chain_id,
                    },
                    item.iframe.into(),
                    true,
                );
                IsContentful(false)
            },
            DisplayItem::PushStackingContext(ref item) => {
                let stacking_context = &item.stacking_context;
                debug_assert_eq!(stacking_context.context_type, StackingContextType::Real);

                //let mut info = LayoutPrimitiveInfo::new(stacking_context.bounds);
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
                                (Some(t), None) => (
                                    t,
                                    ReferenceFrameKind::Transform {
                                        is_2d_scale_translation: false,
                                        should_snap: false,
                                        paired_with_perspective: false,
                                    },
                                ),
                                (Some(t), Some(p)) => (
                                    p.then(&t),
                                    ReferenceFrameKind::Perspective {
                                        scrolling_relative_to: None,
                                    },
                                ),
                                (None, None) => unreachable!(),
                            };

                        let index = frame_index.to_index();
                        let new_spatial_id = builder.push_reference_frame(
                            stacking_context.bounds.min,
                            current_scroll_node_id.spatial_id,
                            stacking_context.transform_style,
                            PropertyBinding::Value(transform),
                            ref_frame,
                            self.get_spatial_tree_item_key(builder, index),
                        );

                        state.add_clip_node_mapping(index, current_clip_chain_id);
                        state.register_spatial_node(
                            index,
                            new_spatial_id,
                            Some(current_scrolling_index),
                            None,
                        );

                        bounds.min = LayoutPoint::zero();
                        new_spatial_id
                    } else {
                        current_scroll_node_id.spatial_id
                    };

                // TODO(jdm): WebRender now requires us to create stacking context items
                //            with the IS_BLEND_CONTAINER flag enabled if any children
                //            of the stacking context have a blend mode applied.
                //            This will require additional tracking during layout
                //            before we start collecting stacking contexts so that
                //            information will be available when we reach this point.
                builder.push_stacking_context(
                    bounds.min,
                    spatial_id,
                    PrimitiveFlags::default(),
                    None,
                    stacking_context.transform_style,
                    stacking_context.mix_blend_mode,
                    &stacking_context.filters,
                    &[],
                    &[],
                    RasterSpace::Screen,
                    Default::default(),
                );

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
                let node = state.clip_scroll_nodes[index].clone();
                let item_rect = node.clip.main;

                let parent_index = node.parent_index.to_index();
                let parent_spatial_id = state.webrender_spatial_id_for_index(parent_index);
                let parent_clip_chain_id = state.webrender_clip_id_for_index(parent_index);

                match node.node_type {
                    ClipScrollNodeType::Clip(clip_type) => {
                        let clip_id = match clip_type {
                            ClipType::Rect => {
                                builder.define_clip_rect(parent_spatial_id, item_rect)
                            },
                            ClipType::Rounded(complex) => {
                                builder.define_clip_rounded_rect(parent_spatial_id, complex)
                            },
                        };

                        let clip_chain_id =
                            state.define_clip_chain(builder, parent_clip_chain_id, [clip_id]);
                        state.add_clip_node_mapping(index, clip_chain_id);
                        state.add_spatial_node_mapping_to_parent_index(index, parent_index);
                    },
                    ClipScrollNodeType::ScrollFrame(scroll_sensitivity, external_id) => {
                        let clip_id = builder.define_clip_rect(parent_spatial_id, item_rect);
                        let clip_chain_id =
                            state.define_clip_chain(builder, parent_clip_chain_id, [clip_id]);
                        state.add_clip_node_mapping(index, clip_chain_id);

                        let spatial_id = builder.define_scroll_frame(
                            parent_spatial_id,
                            external_id,
                            node.content_rect,
                            item_rect,
                            LayoutVector2D::zero(), /* external_scroll_offset */
                            0,                      /* scroll_offset_generation */
                            HasScrollLinkedEffect::No,
                            self.get_spatial_tree_item_key(builder, index),
                        );

                        state.register_spatial_node(
                            index,
                            spatial_id,
                            Some(parent_index),
                            Some(ScrollableNodeInfo {
                                external_id,
                                scrollable_size: node.content_rect.size() - item_rect.size(),
                                scroll_sensitivity,
                                offset: LayoutVector2D::zero(),
                            }),
                        );
                    },
                    ClipScrollNodeType::StickyFrame(ref sticky_data) => {
                        // TODO: Add define_sticky_frame_with_parent to WebRender.
                        let id = builder.define_sticky_frame(
                            parent_spatial_id,
                            item_rect,
                            sticky_data.margins,
                            sticky_data.vertical_offset_bounds,
                            sticky_data.horizontal_offset_bounds,
                            LayoutVector2D::zero(), /* previously_applied_offset */
                            self.get_spatial_tree_item_key(builder, index),
                        );

                        state.add_clip_node_mapping(index, parent_clip_chain_id);
                        state.register_spatial_node(index, id, Some(current_scrolling_index), None);
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
