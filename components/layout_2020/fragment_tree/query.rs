/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::hash::RandomState;

use app_units::Au;
use base::id::PipelineId;
use euclid::Vector2D;
use webrender_api::ExternalScrollId;
use webrender_api::units::LayoutPixel;

use super::{ContainingBlockManager, Fragment, FragmentTree};
use crate::geom::{PhysicalRect, PhysicalVec};
use crate::style_ext::ComputedValuesExt;

/// Informations that could be stored and mutated between iteration of [Fragment::find_v2]
pub struct FragmentTreeQueryContext<'a, T, D: 'a + Clone> {
    manager: ContainingBlockManager<'a, T>,
    level: usize,
    additional_data: D,
}

/// Additional information to compute containing block for `getClientRect()`. In this case,
/// we would store scroll offsets from LayoutThread for computation as well.
///
/// FIXME: Ideally, we should consider the Webrender display list.
pub struct ContainingBlockInfoData<'a> {
    /// Reference to scroll offsets from LayoutThread
    pub(crate) scroll_offsets:
        &'a HashMap<ExternalScrollId, Vector2D<f32, LayoutPixel>, RandomState>,
    /// Pipeline information from LayoutThread
    pub(crate) pipeline_id: PipelineId,
}

impl Clone for ContainingBlockInfoData<'_> {
    fn clone(&self) -> Self {
        Self {
            scroll_offsets: self.scroll_offsets,
            pipeline_id: self.pipeline_id,
        }
    }
}

impl<'a, T, D: Clone> FragmentTreeQueryContext<'a, T, D> {
    fn new_for_next_level(&self, manager: ContainingBlockManager<'a, T>) -> Self {
        Self {
            manager,
            level: self.level + 1,
            additional_data: self.additional_data.clone(),
        }
    }
}

pub type ContainingBlockInfoContext<'a> =
    FragmentTreeQueryContext<'a, ContainingBlockQueryInfo, ContainingBlockInfoData<'a>>;

impl ContainingBlockInfoContext<'_> {
    /// Create an [FragmentTreeQueryContext] context and run the predicate with it as the argument.
    /// We run it inside the predicate to maintain the [ContainingBlockManager]'s borrow structure.
    pub(crate) fn for_fragment_tree_and_then<
        P: FnMut(&ContainingBlockInfoContext<'_>) -> Option<ContainingBlockQueryInfo>,
    >(
        fragment_tree: &FragmentTree,
        additional_data: ContainingBlockInfoData,
        mut predicate: P,
    ) -> Option<ContainingBlockQueryInfo> {
        let scroll_offset = additional_data
            .scroll_offsets
            .get(&additional_data.pipeline_id.root_scroll_id())
            .map(|offset| PhysicalVec::new(Au::from_f32_px(offset.x), Au::from_f32_px(offset.y)))
            .unwrap_or_default();

        let initial_containing_block_info = ContainingBlockQueryInfo {
            rect: fragment_tree.initial_containing_block,
            scroll_offset,
        };
        let fixed_containing_block_info = ContainingBlockQueryInfo {
            rect: fragment_tree.initial_containing_block,
            scroll_offset: Vector2D::zero(),
        };

        let info: ContainingBlockManager<'_, ContainingBlockQueryInfo> = ContainingBlockManager {
            for_non_absolute_descendants: &initial_containing_block_info,
            for_absolute_descendants: Some(&initial_containing_block_info),
            for_absolute_and_fixed_descendants: &fixed_containing_block_info,
        };

        let initial_context = FragmentTreeQueryContext {
            manager: info,
            level: 0,
            additional_data,
        };

        predicate(&initial_context)
    }

    /// Create/mutate [FragmentTreeQueryContext] context and run the predicate with it as the argument.
    /// We run it inside the predicate to maintain the [ContainingBlockManager]'s borrow structure.
    pub(crate) fn precompute_state_and_then<
        P: FnMut(&ContainingBlockInfoContext<'_>) -> Option<ContainingBlockQueryInfo>,
    >(
        &self,
        parent: &Fragment,
        mut predicate: P,
    ) -> Option<ContainingBlockQueryInfo> {
        let containing_block = self.manager.get_containing_block_for_fragment(parent);
        let additional_data = &self.additional_data;

        match parent {
            Fragment::Box(fragment) | Fragment::Float(fragment) => {
                let fragment = fragment.borrow();
                let scroll_id = fragment.base.tag.map(|tag| {
                    ExternalScrollId(
                        tag.to_display_list_fragment_id(),
                        additional_data.pipeline_id.into(),
                    )
                });
                let scroll_offset = scroll_id
                    .and_then(|id| additional_data.scroll_offsets.get(&id))
                    .map(|offset| {
                        PhysicalVec::new(Au::from_f32_px(offset.x), Au::from_f32_px(offset.y))
                    })
                    .unwrap_or_default();

                let content_rect_info = containing_block
                    .new_relative_transformed_child(fragment.content_rect, scroll_offset);
                let padding_rect_info = containing_block
                    .new_relative_transformed_child(fragment.padding_rect(), scroll_offset);

                let new_manager = if fragment
                    .style
                    .establishes_containing_block_for_all_descendants(fragment.base.flags)
                {
                    self.manager.new_for_absolute_and_fixed_descendants(
                        &content_rect_info,
                        &padding_rect_info,
                    )
                } else if fragment
                    .style
                    .establishes_containing_block_for_absolute_descendants(fragment.base.flags)
                {
                    self.manager
                        .new_for_absolute_descendants(&content_rect_info, &padding_rect_info)
                } else {
                    self.manager
                        .new_for_non_absolute_descendants(&content_rect_info)
                };

                predicate(&self.new_for_next_level(new_manager))
            },
            Fragment::Positioning(fragment) => {
                let fragment = fragment.borrow();
                let scroll_id = fragment.base.tag.map(|tag| {
                    ExternalScrollId(
                        tag.to_display_list_fragment_id(),
                        additional_data.pipeline_id.into(),
                    )
                });
                let scroll_offset = scroll_id
                    .and_then(|id| additional_data.scroll_offsets.get(&id))
                    .map(|offset| {
                        PhysicalVec::new(Au::from_f32_px(offset.x), Au::from_f32_px(offset.y))
                    })
                    .unwrap_or_default();

                let content_rect_info =
                    containing_block.new_relative_transformed_child(fragment.rect, scroll_offset);

                let new_manager = self
                    .manager
                    .new_for_non_absolute_descendants(&content_rect_info);

                predicate(&self.new_for_next_level(new_manager))
            },
            _ => None,
        }
    }

    pub(crate) fn get_payload(&self, fragment: &Fragment) -> &ContainingBlockQueryInfo {
        self.manager.get_containing_block_for_fragment(fragment)
    }
}

/// Containing block rect with additional information required for a query.
pub(crate) struct ContainingBlockQueryInfo {
    /// Containing block rect, that bounds the children.
    pub(crate) rect: PhysicalRect<Au>,

    /// The scroll offset of the containing block has.
    pub(crate) scroll_offset: PhysicalVec<Au>,
}

impl ContainingBlockQueryInfo {
    /// Transform child's rectangle according to this containing block transformation.
    /// TODO: this is supposed to handle CSS transform but it is not happening.
    pub(crate) fn transform_rect_relative_to_self(
        &self,
        rect: PhysicalRect<Au>,
    ) -> PhysicalRect<Au> {
        rect.translate(self.rect.origin.to_vector() + self.scroll_offset)
    }

    /// New containing block that is a child of this containing block with
    /// ancestor's transformation applied.
    pub(crate) fn new_relative_transformed_child(
        &self,
        rect: PhysicalRect<Au>,
        scroll_offset: PhysicalVec<Au>,
    ) -> ContainingBlockQueryInfo {
        ContainingBlockQueryInfo {
            rect: self.transform_rect_relative_to_self(rect),
            scroll_offset,
        }
    }
}
