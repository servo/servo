/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
#![allow(rustdoc::private_intra_doc_links)]

//! Same-formatting context blocks. This represents a block in block flow that does not
//! establish a new formatting context.

use std::sync::Arc;

use app_units::Au;
use malloc_size_of_derive::MallocSizeOf;
use script::layout_dom::ServoLayoutNode;
use servo_arc::Arc as ServoArc;
use style::Zero;
use style::context::SharedStyleContext;
use style::logical_geometry::Direction;
use style::properties::ComputedValues;
use style::servo::selector_parser::PseudoElement;

use crate::context::LayoutContext;
use crate::flow::float::{Clear, ContainingBlockPositionInfo, SequentialLayoutState};
use crate::flow::{
    BlockContainer, CollapsibleWithParentStartMargin, ContainingBlockPaddingAndBorder,
    ResolvedMargins, solve_containing_block_padding_and_border_for_in_flow_box, solve_margins,
};
use crate::fragment_tree::{BoxFragment, CollapsedBlockMargins, CollapsedMargin, FragmentFlags};
use crate::geom::{LogicalRect, LogicalSides1D, LogicalVec2};
use crate::layout_box_base::LayoutBoxBase;
use crate::positioned::PositioningContext;
use crate::sizing::{InlineContentSizesResult, Size};
use crate::style_ext::LayoutStyle;
use crate::{ConstraintSpace, ContainingBlock};

/// A block in block flow that does not establish a new formatting context.
#[derive(Debug, MallocSizeOf)]
pub(crate) struct SameFormattingContextBlock {
    pub base: LayoutBoxBase,
    pub contents: BlockContainer,
    pub contains_floats: bool,
}

impl SameFormattingContextBlock {
    pub(crate) fn new(
        base: LayoutBoxBase,
        contents: BlockContainer,
        contains_floats: bool,
    ) -> Self {
        Self {
            base,
            contents,
            contains_floats,
        }
    }

    pub(crate) fn layout_style(&self) -> LayoutStyle<'_> {
        self.contents.layout_style(&self.base)
    }

    pub(crate) fn repair_style(
        &mut self,
        context: &SharedStyleContext,
        node: &ServoLayoutNode,
        new_style: &ServoArc<ComputedValues>,
    ) {
        self.base.repair_style(new_style);
        self.contents.repair_style(context, node, new_style);
    }

    pub(crate) fn inline_content_sizes(
        &self,
        layout_context: &LayoutContext,
        constraint_space: &ConstraintSpace,
    ) -> InlineContentSizesResult {
        self.base
            .inline_content_sizes(layout_context, constraint_space, &self.contents)
    }

    /// Lay out a normal flow non-replaced [`SameFormattingContextBlock`], properly taking
    /// into account relative positioning. This version also handles caching the layout
    /// results and fetching the results from the cache, if they are still valid.
    ///
    /// - <https://drafts.csswg.org/css2/visudet.html#blockwidth>
    /// - <https://drafts.csswg.org/css2/visudet.html#normal-block>
    #[expect(clippy::too_many_arguments)]
    pub(crate) fn layout_in_flow_non_replaced_block_level_cached(
        &self,
        layout_context: &LayoutContext<'_>,
        positioning_context: &mut PositioningContext,
        containing_block: &ContainingBlock<'_>,
        sequential_layout_state: Option<&mut SequentialLayoutState>,
        collapsible_with_parent_start_margin: Option<CollapsibleWithParentStartMargin>,
        ignore_block_margins_for_stretch: LogicalSides1D<bool>,
        has_inline_parent: bool,
    ) -> Arc<BoxFragment> {
        let mut allows_caching = sequential_layout_state.is_none();

        if allows_caching &&
            let Some(cached_result) = self
                .base
                .cached_same_formatting_context_block_if_applicable(
                    containing_block,
                    collapsible_with_parent_start_margin,
                    ignore_block_margins_for_stretch,
                    has_inline_parent,
                )
        {
            return cached_result;
        };

        let positioning_context_length = positioning_context.len();
        let fragment = Arc::new(positioning_context.layout_maybe_position_relative_fragment(
            layout_context,
            containing_block,
            &self.base,
            |positioning_context| {
                self.layout_in_flow_non_replaced_block_level(
                    layout_context,
                    positioning_context,
                    containing_block,
                    sequential_layout_state,
                    collapsible_with_parent_start_margin,
                    ignore_block_margins_for_stretch,
                    has_inline_parent,
                )
            },
        ));

        // We currently do not allow caching `SameFormattingContextBlock` box layout results if they
        // contain absolutely positioned children.
        //
        // TODO: It would be good to find a way to allow this, without having to create and store a
        // PositioningContext for every single SameFormattingContextBlock.
        allows_caching = allows_caching && positioning_context_length == positioning_context.len();

        if !allows_caching {
            self.base.clear_fragments_and_dirty_fragment_cache();
        } else {
            self.base.cache_same_formatting_context_block_layout(
                containing_block,
                collapsible_with_parent_start_margin,
                ignore_block_margins_for_stretch,
                has_inline_parent,
                fragment.clone(),
            );
        }

        fragment
    }

    /// Lay out a normal flow non-replaced [`SameFormattingContextBlock`].
    ///
    /// - <https://drafts.csswg.org/css2/visudet.html#blockwidth>
    /// - <https://drafts.csswg.org/css2/visudet.html#normal-block>
    #[expect(clippy::too_many_arguments)]
    fn layout_in_flow_non_replaced_block_level(
        &self,
        layout_context: &LayoutContext,
        positioning_context: &mut PositioningContext,
        containing_block: &ContainingBlock,
        mut sequential_layout_state: Option<&mut SequentialLayoutState>,
        collapsible_with_parent_start_margin: Option<CollapsibleWithParentStartMargin>,
        ignore_block_margins_for_stretch: LogicalSides1D<bool>,
        has_inline_parent: bool,
    ) -> BoxFragment {
        let style = &self.base.style;
        let layout_style = self.contents.layout_style(&self.base);
        let containing_block_writing_mode = containing_block.style.writing_mode;
        let get_inline_content_sizes = |constraint_space: &ConstraintSpace| {
            self.base
                .inline_content_sizes(layout_context, constraint_space, &self.contents)
                .sizes
        };
        let ContainingBlockPaddingAndBorder {
            containing_block: containing_block_for_children,
            pbm,
            block_sizes,
            depends_on_block_constraints,
            available_block_size,
            justify_self,
            ..
        } = solve_containing_block_padding_and_border_for_in_flow_box(
            containing_block,
            &layout_style,
            get_inline_content_sizes,
            ignore_block_margins_for_stretch,
            None,
            has_inline_parent,
        );
        let ResolvedMargins {
            margin,
            effective_margin_inline_start,
        } = solve_margins(
            containing_block,
            &pbm,
            containing_block_for_children.size.inline,
            justify_self,
        );

        let start_margin_can_collapse_with_children =
            pbm.padding.block_start.is_zero() && pbm.border.block_start.is_zero();

        let mut clearance = None;
        let parent_containing_block_position_info;
        match sequential_layout_state {
            None => parent_containing_block_position_info = None,
            Some(ref mut sequential_layout_state) => {
                let clear = Clear::from_style_and_container_writing_mode(
                    style,
                    containing_block_writing_mode,
                );
                let mut block_start_margin = CollapsedMargin::new(margin.block_start);

                // The block start margin may collapse with content margins,
                // compute the resulting one in order to place floats correctly.
                // Only need to do this if the element isn't also collapsing with its parent,
                // otherwise we should have already included the margin in an ancestor.
                // Note this lookahead stops when finding a descendant whose `clear` isn't `none`
                // (since clearance prevents collapsing margins with the parent).
                // But then we have to decide whether to actually add clearance or not,
                // so look forward again regardless of `collapsible_with_parent_start_margin`.
                // TODO: This isn't completely right: if we don't add actual clearance,
                // the margin should have been included in the parent (or some ancestor).
                // The lookahead should stop for actual clearance, not just for `clear`.
                let collapsible_with_parent_start_margin = collapsible_with_parent_start_margin.expect(
                    "We should know whether we are collapsing the block start margin with the parent \
                    when laying out sequentially",
                ).0 && clear == Clear::None;
                if !collapsible_with_parent_start_margin && start_margin_can_collapse_with_children
                {
                    self.contents.find_block_margin_collapsing_with_parent(
                        layout_context,
                        &mut block_start_margin,
                        &containing_block_for_children,
                    );
                }

                // Introduce clearance if necessary.
                clearance = sequential_layout_state.calculate_clearance(clear, &block_start_margin);
                if clearance.is_some() {
                    sequential_layout_state.commit_margin();
                }
                sequential_layout_state.adjoin_assign(&block_start_margin);
                if !start_margin_can_collapse_with_children {
                    sequential_layout_state.commit_margin();
                }

                // NB: This will be a no-op if we're collapsing margins with our children since that
                // can only happen if we have no block-start padding and border.
                sequential_layout_state.advance_block_position(
                    pbm.padding.block_start +
                        pbm.border.block_start +
                        clearance.unwrap_or_else(Au::zero),
                );

                // We are about to lay out children. Update the offset between the block formatting
                // context and the containing block that we create for them. This offset is used to
                // ajust BFC relative coordinates to coordinates that are relative to our content box.
                // Our content box establishes the containing block for non-abspos children, including
                // floats.
                let inline_start = sequential_layout_state
                    .floats
                    .containing_block_info
                    .inline_start +
                    pbm.padding.inline_start +
                    pbm.border.inline_start +
                    effective_margin_inline_start;
                let new_cb_offsets = ContainingBlockPositionInfo {
                    block_start: sequential_layout_state.bfc_relative_block_position,
                    block_start_margins_not_collapsed: sequential_layout_state.current_margin,
                    inline_start,
                    inline_end: inline_start + containing_block_for_children.size.inline,
                };
                parent_containing_block_position_info = Some(
                    sequential_layout_state.replace_containing_block_position_info(new_cb_offsets),
                );
            },
        };

        // https://drafts.csswg.org/css-sizing-4/#stretch-fit-sizing
        // > If this is a block axis size, and the element is in a Block Layout formatting context,
        // > and the parent element does not have a block-start border or padding and is not an
        // > independent formatting context, treat the element’s block-start margin as zero
        // > for the purpose of calculating this size. Do the same for the block-end margin.
        let ignore_block_margins_for_stretch = LogicalSides1D::new(
            pbm.border.block_start.is_zero() && pbm.padding.block_start.is_zero(),
            pbm.border.block_end.is_zero() && pbm.padding.block_end.is_zero(),
        );

        let flow_layout = self.contents.layout(
            layout_context,
            positioning_context,
            &containing_block_for_children,
            sequential_layout_state.as_deref_mut(),
            CollapsibleWithParentStartMargin(start_margin_can_collapse_with_children),
            ignore_block_margins_for_stretch,
        );
        let mut content_block_size = flow_layout.content_block_size;

        // Update margins.
        let mut block_margins_collapsed_with_children = CollapsedBlockMargins::from_margin(&margin);
        let mut collapsible_margins_in_children = flow_layout.collapsible_margins_in_children;
        if start_margin_can_collapse_with_children {
            block_margins_collapsed_with_children
                .start
                .adjoin_assign(&collapsible_margins_in_children.start);
            if collapsible_margins_in_children.collapsed_through {
                block_margins_collapsed_with_children
                    .start
                    .adjoin_assign(&std::mem::replace(
                        &mut collapsible_margins_in_children.end,
                        CollapsedMargin::zero(),
                    ));
            }
        }

        let is_anonymous = matches!(
            self.base.style.pseudo(),
            Some(PseudoElement::ServoAnonymousBox)
        );
        let tentative_block_size = if is_anonymous {
            // Anonymous blocks do not establish a containing block for their children,
            // so we can't use that. However, they always have their sizing properties
            // set to their initial values, so it's fine to use the default.
            &Default::default()
        } else {
            &containing_block_for_children.size.block
        };
        let collapsed_through = collapsible_margins_in_children.collapsed_through &&
            pbm.padding_border_sums.block.is_zero() &&
            tentative_block_size.definite_or_min().is_zero();
        block_margins_collapsed_with_children.collapsed_through = collapsed_through;

        let end_margin_can_collapse_with_children =
            pbm.padding.block_end.is_zero() && pbm.border.block_end.is_zero();
        if !end_margin_can_collapse_with_children {
            content_block_size += collapsible_margins_in_children.end.solve();
        }

        let block_size = block_sizes.resolve(
            Direction::Block,
            Size::FitContent,
            Au::zero,
            available_block_size,
            || content_block_size.into(),
            false, /* is_table */
        );

        // If the final block size is different than the intrinsic size of the contents,
        // then we can't actually collapse the end margins. This can happen due to min
        // or max block sizes, or due to `calc-size()` once we implement it.
        //
        // We also require `block-size` to have an intrinsic value, by checking whether
        // the containing block established for the contents has an indefinite block size.
        // However, even if `block-size: 0px` is extrinsic (so it would normally prevent
        // collapsing the end margin with children), it doesn't prevent the top and end
        // margins from collapsing through. If that happens, allow collapsing end margins.
        //
        // This is being discussed in https://github.com/w3c/csswg-drafts/issues/12218.
        // It would probably make more sense to check the definiteness of the containing
        // block in the logic above (when we check if there is some block-end padding or
        // border), or maybe drop the condition altogether. But for now, we match Blink.
        let end_margin_can_collapse_with_children = end_margin_can_collapse_with_children &&
            block_size == content_block_size &&
            (collapsed_through || !tentative_block_size.is_definite());
        if end_margin_can_collapse_with_children {
            block_margins_collapsed_with_children
                .end
                .adjoin_assign(&collapsible_margins_in_children.end);
        }

        if let Some(ref mut sequential_layout_state) = sequential_layout_state {
            // Now that we're done laying out our children, we can restore the
            // parent's containing block position information.
            sequential_layout_state.replace_containing_block_position_info(
                parent_containing_block_position_info.unwrap(),
            );

            // Account for padding and border. We also might have to readjust the
            // `bfc_relative_block_position` if it was different from the content size (i.e. was
            // non-`auto` and/or was affected by min/max block size).
            //
            // If this adjustment is positive, that means that a block size was specified, but
            // the content inside had a smaller block size. If this adjustment is negative, a
            // block size was specified, but the content inside overflowed this container in
            // the block direction. In that case, the ceiling for floats is effectively raised
            // as long as no floats in the overflowing content lowered it.
            sequential_layout_state.advance_block_position(
                block_size - content_block_size + pbm.padding.block_end + pbm.border.block_end,
            );

            if !end_margin_can_collapse_with_children {
                sequential_layout_state.commit_margin();
            }
            sequential_layout_state.adjoin_assign(&CollapsedMargin::new(margin.block_end));
        }

        let content_rect = LogicalRect {
            start_corner: LogicalVec2 {
                block: (pbm.padding.block_start +
                    pbm.border.block_start +
                    clearance.unwrap_or_else(Au::zero)),
                inline: pbm.padding.inline_start +
                    pbm.border.inline_start +
                    effective_margin_inline_start,
            },
            size: LogicalVec2 {
                block: block_size,
                inline: containing_block_for_children.size.inline,
            },
        };

        let mut base_fragment_info = self.base.base_fragment_info;

        // An anonymous block doesn't establish a containing block for its contents. Therefore,
        // if its contents depend on block constraints, its block size (which is intrinsic) also
        // depends on block constraints.
        if depends_on_block_constraints ||
            (is_anonymous && flow_layout.depends_on_block_constraints)
        {
            base_fragment_info.flags.insert(
                FragmentFlags::SIZE_DEPENDS_ON_BLOCK_CONSTRAINTS_AND_CAN_BE_CHILD_OF_FLEX_ITEM,
            );
        }

        BoxFragment::new(
            base_fragment_info,
            style.clone(),
            flow_layout.fragments,
            content_rect.as_physical(Some(containing_block)),
            pbm.padding.to_physical(containing_block_writing_mode),
            pbm.border.to_physical(containing_block_writing_mode),
            margin.to_physical(containing_block_writing_mode),
            flow_layout.specific_layout_info,
        )
        .with_baselines(flow_layout.baselines)
        .with_block_level_layout_info(block_margins_collapsed_with_children, clearance)
    }
}
