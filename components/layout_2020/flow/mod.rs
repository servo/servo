/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Flow layout, also known as block-and-inline layout.

use crate::flow::float::{FloatBox, FloatContext};
use crate::flow::inline::InlineFormattingContext;
use crate::fragments::{
    AnonymousFragment, BoxFragment, CollapsedBlockMargins, CollapsedMargin, Fragment,
};
use crate::geom::flow_relative::{Rect, Vec2};
use crate::positioned::{
    adjust_static_positions, AbsolutelyPositionedBox, AbsolutelyPositionedFragment,
};
use crate::style_ext::{ComputedValuesExt, Position};
use crate::{relative_adjustement, ContainingBlock, IndependentFormattingContext};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use rayon_croissant::ParallelIteratorExt;
use servo_arc::Arc;
use style::properties::ComputedValues;
use style::values::computed::{Length, LengthOrAuto};
use style::Zero;

mod construct;
mod float;
pub mod inline;
mod root;

#[derive(Debug)]
pub(crate) struct BlockFormattingContext {
    pub contents: BlockContainer,
    pub contains_floats: bool,
}

#[derive(Debug)]
pub(crate) enum BlockContainer {
    BlockLevelBoxes(Vec<Arc<BlockLevelBox>>),
    InlineFormattingContext(InlineFormattingContext),
}

#[derive(Debug)]
pub(crate) enum BlockLevelBox {
    SameFormattingContextBlock {
        style: Arc<ComputedValues>,
        contents: BlockContainer,
    },
    OutOfFlowAbsolutelyPositionedBox(AbsolutelyPositionedBox),
    OutOfFlowFloatBox(FloatBox),
    Independent {
        style: Arc<ComputedValues>,
        contents: IndependentFormattingContext,
    },
}

pub(super) struct FlowChildren {
    pub fragments: Vec<Fragment>,
    pub block_size: Length,
    pub collapsible_margins_in_children: CollapsedBlockMargins,
}

#[derive(Clone, Copy)]
struct CollapsibleWithParentStartMargin(bool);

impl BlockFormattingContext {
    pub(super) fn layout<'a>(
        &'a self,
        containing_block: &ContainingBlock,
        tree_rank: usize,
        absolutely_positioned_fragments: &mut Vec<AbsolutelyPositionedFragment<'a>>,
    ) -> FlowChildren {
        let mut float_context;
        let float_context = if self.contains_floats {
            float_context = FloatContext::new();
            Some(&mut float_context)
        } else {
            None
        };
        let mut flow_children = self.contents.layout(
            containing_block,
            tree_rank,
            absolutely_positioned_fragments,
            float_context,
            CollapsibleWithParentStartMargin(false),
        );
        flow_children.block_size += flow_children.collapsible_margins_in_children.end.solve();
        flow_children
            .collapsible_margins_in_children
            .collapsed_through = false;
        flow_children.collapsible_margins_in_children.end = CollapsedMargin::zero();
        flow_children
    }
}

impl BlockContainer {
    fn layout<'a>(
        &'a self,
        containing_block: &ContainingBlock,
        tree_rank: usize,
        absolutely_positioned_fragments: &mut Vec<AbsolutelyPositionedFragment<'a>>,
        float_context: Option<&mut FloatContext>,
        collapsible_with_parent_start_margin: CollapsibleWithParentStartMargin,
    ) -> FlowChildren {
        match self {
            BlockContainer::BlockLevelBoxes(child_boxes) => layout_block_level_children(
                child_boxes,
                containing_block,
                tree_rank,
                absolutely_positioned_fragments,
                float_context,
                collapsible_with_parent_start_margin,
            ),
            BlockContainer::InlineFormattingContext(ifc) => {
                ifc.layout(containing_block, tree_rank, absolutely_positioned_fragments)
            },
        }
    }
}

fn layout_block_level_children<'a>(
    child_boxes: &'a [Arc<BlockLevelBox>],
    containing_block: &ContainingBlock,
    tree_rank: usize,
    absolutely_positioned_fragments: &mut Vec<AbsolutelyPositionedFragment<'a>>,
    float_context: Option<&mut FloatContext>,
    collapsible_with_parent_start_margin: CollapsibleWithParentStartMargin,
) -> FlowChildren {
    fn place_block_level_fragment(fragment: &mut Fragment, placement_state: &mut PlacementState) {
        match fragment {
            Fragment::Box(fragment) => {
                let fragment_block_margins = &fragment.block_margins_collapsed_with_children;
                let fragment_block_size = fragment.padding.block_sum() +
                    fragment.border.block_sum() +
                    fragment.content_rect.size.block;

                if placement_state.next_in_flow_margin_collapses_with_parent_start_margin {
                    assert_eq!(placement_state.current_margin.solve(), Length::zero());
                    placement_state
                        .start_margin
                        .adjoin_assign(&fragment_block_margins.start);
                    if fragment_block_margins.collapsed_through {
                        placement_state
                            .start_margin
                            .adjoin_assign(&fragment_block_margins.end);
                        return;
                    }
                    placement_state.next_in_flow_margin_collapses_with_parent_start_margin = false;
                } else {
                    placement_state
                        .current_margin
                        .adjoin_assign(&fragment_block_margins.start);
                }
                fragment.content_rect.start_corner.block += placement_state.current_margin.solve() +
                    placement_state.current_block_direction_position;
                if fragment_block_margins.collapsed_through {
                    placement_state
                        .current_margin
                        .adjoin_assign(&fragment_block_margins.end);
                    return;
                }
                placement_state.current_block_direction_position +=
                    placement_state.current_margin.solve() + fragment_block_size;
                placement_state.current_margin = fragment_block_margins.end;
            },
            Fragment::Anonymous(fragment) => {
                // FIXME(nox): Margin collapsing for hypothetical boxes of
                // abspos elements is probably wrong.
                assert!(fragment.children.is_empty());
                assert_eq!(fragment.rect.size.block, Length::zero());
                fragment.rect.start_corner.block +=
                    placement_state.current_block_direction_position;
            },
            _ => unreachable!(),
        }
    }

    struct PlacementState {
        next_in_flow_margin_collapses_with_parent_start_margin: bool,
        start_margin: CollapsedMargin,
        current_margin: CollapsedMargin,
        current_block_direction_position: Length,
    }

    let abspos_so_far = absolutely_positioned_fragments.len();
    let mut placement_state = PlacementState {
        next_in_flow_margin_collapses_with_parent_start_margin:
            collapsible_with_parent_start_margin.0,
        start_margin: CollapsedMargin::zero(),
        current_margin: CollapsedMargin::zero(),
        current_block_direction_position: Length::zero(),
    };
    let mut fragments: Vec<_>;
    if let Some(float_context) = float_context {
        // Because floats are involved, we do layout for this block formatting context
        // in tree order without parallelism. This enables mutable access
        // to a `FloatContext` that tracks every float encountered so far (again in tree order).
        fragments = child_boxes
            .iter()
            .enumerate()
            .map(|(tree_rank, box_)| {
                let mut fragment = box_.layout(
                    containing_block,
                    tree_rank,
                    absolutely_positioned_fragments,
                    Some(float_context),
                );
                place_block_level_fragment(&mut fragment, &mut placement_state);
                fragment
            })
            .collect()
    } else {
        fragments = child_boxes
            .par_iter()
            .enumerate()
            .mapfold_reduce_into(
                absolutely_positioned_fragments,
                |abspos_fragments, (tree_rank, box_)| {
                    box_.layout(
                        containing_block,
                        tree_rank,
                        abspos_fragments,
                        /* float_context = */ None,
                    )
                },
                |left_abspos_fragments, mut right_abspos_fragments| {
                    left_abspos_fragments.append(&mut right_abspos_fragments);
                },
            )
            .collect();
        for fragment in &mut fragments {
            place_block_level_fragment(fragment, &mut placement_state)
        }
    }

    adjust_static_positions(
        &mut absolutely_positioned_fragments[abspos_so_far..],
        &mut fragments,
        tree_rank,
    );

    FlowChildren {
        fragments,
        block_size: placement_state.current_block_direction_position,
        collapsible_margins_in_children: CollapsedBlockMargins {
            collapsed_through: placement_state
                .next_in_flow_margin_collapses_with_parent_start_margin,
            start: placement_state.start_margin,
            end: placement_state.current_margin,
        },
    }
}

impl BlockLevelBox {
    fn layout<'a>(
        &'a self,
        containing_block: &ContainingBlock,
        tree_rank: usize,
        absolutely_positioned_fragments: &mut Vec<AbsolutelyPositionedFragment<'a>>,
        float_context: Option<&mut FloatContext>,
    ) -> Fragment {
        match self {
            BlockLevelBox::SameFormattingContextBlock { style, contents } => {
                Fragment::Box(layout_in_flow_non_replaced_block_level(
                    containing_block,
                    absolutely_positioned_fragments,
                    style,
                    BlockLevelKind::SameFormattingContextBlock,
                    |containing_block, nested_abspos, collapsible_with_parent_start_margin| {
                        contents.layout(
                            containing_block,
                            tree_rank,
                            nested_abspos,
                            float_context,
                            collapsible_with_parent_start_margin,
                        )
                    },
                ))
            },
            BlockLevelBox::Independent { style, contents } => match contents.as_replaced() {
                Ok(replaced) => {
                    // FIXME
                    match *replaced {}
                },
                Err(contents) => Fragment::Box(layout_in_flow_non_replaced_block_level(
                    containing_block,
                    absolutely_positioned_fragments,
                    style,
                    BlockLevelKind::EstablishesAnIndependentFormattingContext,
                    |containing_block, nested_abspos, _| {
                        contents.layout(containing_block, tree_rank, nested_abspos)
                    },
                )),
            },
            BlockLevelBox::OutOfFlowAbsolutelyPositionedBox(box_) => {
                absolutely_positioned_fragments.push(box_.layout(Vec2::zero(), tree_rank));
                Fragment::Anonymous(AnonymousFragment::no_op(containing_block.mode))
            },
            BlockLevelBox::OutOfFlowFloatBox(_box_) => {
                // TODO
                Fragment::Anonymous(AnonymousFragment::no_op(containing_block.mode))
            },
        }
    }
}

#[derive(PartialEq)]
enum BlockLevelKind {
    SameFormattingContextBlock,
    EstablishesAnIndependentFormattingContext,
}

/// https://drafts.csswg.org/css2/visudet.html#blockwidth
/// https://drafts.csswg.org/css2/visudet.html#normal-block
fn layout_in_flow_non_replaced_block_level<'a>(
    containing_block: &ContainingBlock,
    absolutely_positioned_fragments: &mut Vec<AbsolutelyPositionedFragment<'a>>,
    style: &Arc<ComputedValues>,
    block_level_kind: BlockLevelKind,
    layout_contents: impl FnOnce(
        &ContainingBlock,
        &mut Vec<AbsolutelyPositionedFragment<'a>>,
        CollapsibleWithParentStartMargin,
    ) -> FlowChildren,
) -> BoxFragment {
    let cbis = containing_block.inline_size;
    let padding = style.padding().percentages_relative_to(cbis);
    let border = style.border_width();
    let mut computed_margin = style.margin().percentages_relative_to(cbis);
    let pb = &padding + &border;
    let box_size = style.box_size();
    let inline_size = box_size.inline.percentage_relative_to(cbis);
    if let LengthOrAuto::LengthPercentage(is) = inline_size {
        let inline_margins = cbis - is - pb.inline_sum();
        match (
            &mut computed_margin.inline_start,
            &mut computed_margin.inline_end,
        ) {
            (s @ &mut LengthOrAuto::Auto, e @ &mut LengthOrAuto::Auto) => {
                *s = LengthOrAuto::LengthPercentage(inline_margins / 2.);
                *e = LengthOrAuto::LengthPercentage(inline_margins / 2.);
            },
            (s @ &mut LengthOrAuto::Auto, _) => {
                *s = LengthOrAuto::LengthPercentage(inline_margins);
            },
            (_, e @ &mut LengthOrAuto::Auto) => {
                *e = LengthOrAuto::LengthPercentage(inline_margins);
            },
            (_, e @ _) => {
                // Either the inline-end margin is auto,
                // or we’re over-constrained and we do as if it were.
                *e = LengthOrAuto::LengthPercentage(inline_margins);
            },
        }
    }
    let margin = computed_margin.auto_is(Length::zero);
    let mut block_margins_collapsed_with_children = CollapsedBlockMargins::from_margin(&margin);
    let inline_size = inline_size.auto_is(|| cbis - pb.inline_sum() - margin.inline_sum());
    let block_size = box_size
        .block
        .maybe_percentage_relative_to(containing_block.block_size.non_auto());
    let containing_block_for_children = ContainingBlock {
        inline_size,
        block_size,
        mode: style.writing_mode(),
    };
    // https://drafts.csswg.org/css-writing-modes/#orthogonal-flows
    assert_eq!(
        containing_block.mode, containing_block_for_children.mode,
        "Mixed writing modes are not supported yet"
    );
    let this_start_margin_can_collapse_with_children = CollapsibleWithParentStartMargin(
        block_level_kind == BlockLevelKind::SameFormattingContextBlock &&
            pb.block_start == Length::zero(),
    );
    let this_end_margin_can_collapse_with_children = (block_level_kind, pb.block_end, block_size) ==
        (
            BlockLevelKind::SameFormattingContextBlock,
            Length::zero(),
            LengthOrAuto::Auto,
        );
    let mut nested_abspos = vec![];
    let mut flow_children = layout_contents(
        &containing_block_for_children,
        if style.get_box().position == Position::Relative {
            &mut nested_abspos
        } else {
            absolutely_positioned_fragments
        },
        this_start_margin_can_collapse_with_children,
    );
    if this_start_margin_can_collapse_with_children.0 {
        block_margins_collapsed_with_children
            .start
            .adjoin_assign(&flow_children.collapsible_margins_in_children.start);
        if flow_children
            .collapsible_margins_in_children
            .collapsed_through
        {
            block_margins_collapsed_with_children
                .start
                .adjoin_assign(&std::mem::replace(
                    &mut flow_children.collapsible_margins_in_children.end,
                    CollapsedMargin::zero(),
                ));
        }
    }
    if this_end_margin_can_collapse_with_children {
        block_margins_collapsed_with_children
            .end
            .adjoin_assign(&flow_children.collapsible_margins_in_children.end);
    } else {
        flow_children.block_size += flow_children.collapsible_margins_in_children.end.solve();
    }
    block_margins_collapsed_with_children.collapsed_through =
        this_start_margin_can_collapse_with_children.0 &&
            this_end_margin_can_collapse_with_children &&
            flow_children
                .collapsible_margins_in_children
                .collapsed_through;
    let relative_adjustement = relative_adjustement(style, inline_size, block_size);
    let block_size = block_size.auto_is(|| flow_children.block_size);
    let content_rect = Rect {
        start_corner: Vec2 {
            block: pb.block_start + relative_adjustement.block,
            inline: pb.inline_start + relative_adjustement.inline + margin.inline_start,
        },
        size: Vec2 {
            block: block_size,
            inline: inline_size,
        },
    };
    if style.get_box().position == Position::Relative {
        AbsolutelyPositionedFragment::in_positioned_containing_block(
            &nested_abspos,
            &mut flow_children.fragments,
            &content_rect.size,
            &padding,
            containing_block_for_children.mode,
        )
    }
    BoxFragment {
        style: style.clone(),
        children: flow_children.fragments,
        content_rect,
        padding,
        border,
        margin,
        block_margins_collapsed_with_children,
    }
}
