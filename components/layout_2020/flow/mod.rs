/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Flow layout, also known as block-and-inline layout.

use crate::context::LayoutContext;
use crate::flow::float::{FloatBox, FloatContext};
use crate::flow::inline::InlineFormattingContext;
use crate::formatting_contexts::{IndependentFormattingContext, IndependentLayout};
use crate::fragments::{AnonymousFragment, BoxFragment, Fragment};
use crate::fragments::{CollapsedBlockMargins, CollapsedMargin};
use crate::geom::flow_relative::{Rect, Sides, Vec2};
use crate::positioned::adjust_static_positions;
use crate::positioned::{AbsolutelyPositionedBox, AbsolutelyPositionedFragment};
use crate::replaced::ReplacedContent;
use crate::style_ext::ComputedValuesExt;
use crate::{relative_adjustement, ContainingBlock};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use rayon_croissant::ParallelIteratorExt;
use servo_arc::Arc;
use style::computed_values::position::T as Position;
use style::properties::ComputedValues;
use style::values::computed::{Length, LengthOrAuto, LengthPercentage, LengthPercentageOrAuto};
use style::values::generics::length::MaxSize;
use style::Zero;

mod construct;
mod float;
pub mod inline;
mod root;

pub use root::{BoxTreeRoot, FragmentTreeRoot};

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
    Independent(IndependentFormattingContext),
}

struct FlowLayout {
    pub fragments: Vec<Fragment>,
    pub content_block_size: Length,
    pub collapsible_margins_in_children: CollapsedBlockMargins,
}

#[derive(Clone, Copy)]
struct CollapsibleWithParentStartMargin(bool);

impl BlockFormattingContext {
    pub(super) fn layout<'a>(
        &'a self,
        layout_context: &LayoutContext,
        containing_block: &ContainingBlock,
        tree_rank: usize,
        absolutely_positioned_fragments: &mut Vec<AbsolutelyPositionedFragment<'a>>,
    ) -> IndependentLayout {
        let mut float_context;
        let float_context = if self.contains_floats {
            float_context = FloatContext::new();
            Some(&mut float_context)
        } else {
            None
        };
        let flow_layout = self.contents.layout(
            layout_context,
            containing_block,
            tree_rank,
            absolutely_positioned_fragments,
            float_context,
            CollapsibleWithParentStartMargin(false),
        );
        assert!(
            !flow_layout
                .collapsible_margins_in_children
                .collapsed_through
        );
        IndependentLayout {
            fragments: flow_layout.fragments,
            content_block_size: flow_layout.content_block_size +
                flow_layout.collapsible_margins_in_children.end.solve(),
        }
    }
}

impl BlockContainer {
    fn layout<'a>(
        &'a self,
        layout_context: &LayoutContext,
        containing_block: &ContainingBlock,
        tree_rank: usize,
        absolutely_positioned_fragments: &mut Vec<AbsolutelyPositionedFragment<'a>>,
        float_context: Option<&mut FloatContext>,
        collapsible_with_parent_start_margin: CollapsibleWithParentStartMargin,
    ) -> FlowLayout {
        match self {
            BlockContainer::BlockLevelBoxes(child_boxes) => layout_block_level_children(
                layout_context,
                child_boxes,
                containing_block,
                tree_rank,
                absolutely_positioned_fragments,
                float_context,
                collapsible_with_parent_start_margin,
            ),
            BlockContainer::InlineFormattingContext(ifc) => ifc.layout(
                layout_context,
                containing_block,
                tree_rank,
                absolutely_positioned_fragments,
            ),
        }
    }
}

fn layout_block_level_children<'a>(
    layout_context: &LayoutContext,
    child_boxes: &'a [Arc<BlockLevelBox>],
    containing_block: &ContainingBlock,
    tree_rank: usize,
    absolutely_positioned_fragments: &mut Vec<AbsolutelyPositionedFragment<'a>>,
    float_context: Option<&mut FloatContext>,
    collapsible_with_parent_start_margin: CollapsibleWithParentStartMargin,
) -> FlowLayout {
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
                    layout_context,
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
                        layout_context,
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

    FlowLayout {
        fragments,
        content_block_size: placement_state.current_block_direction_position,
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
        layout_context: &LayoutContext,
        containing_block: &ContainingBlock,
        tree_rank: usize,
        absolutely_positioned_fragments: &mut Vec<AbsolutelyPositionedFragment<'a>>,
        float_context: Option<&mut FloatContext>,
    ) -> Fragment {
        match self {
            BlockLevelBox::SameFormattingContextBlock { style, contents } => {
                Fragment::Box(layout_in_flow_non_replaced_block_level(
                    layout_context,
                    containing_block,
                    absolutely_positioned_fragments,
                    style,
                    BlockLevelKind::SameFormattingContextBlock,
                    |containing_block, nested_abspos, collapsible_with_parent_start_margin| {
                        contents.layout(
                            layout_context,
                            containing_block,
                            tree_rank,
                            nested_abspos,
                            float_context,
                            collapsible_with_parent_start_margin,
                        )
                    },
                ))
            },
            BlockLevelBox::Independent(contents) => match contents.as_replaced() {
                Ok(replaced) => Fragment::Box(layout_in_flow_replaced_block_level(
                    containing_block,
                    &contents.style,
                    replaced,
                )),
                Err(non_replaced) => Fragment::Box(layout_in_flow_non_replaced_block_level(
                    layout_context,
                    containing_block,
                    absolutely_positioned_fragments,
                    &contents.style,
                    BlockLevelKind::EstablishesAnIndependentFormattingContext,
                    |containing_block, nested_abspos, _| {
                        let independent_layout = non_replaced.layout(
                            layout_context,
                            containing_block,
                            tree_rank,
                            nested_abspos,
                        );
                        FlowLayout {
                            fragments: independent_layout.fragments,
                            content_block_size: independent_layout.content_block_size,
                            collapsible_margins_in_children: CollapsedBlockMargins::zero(),
                        }
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
    layout_context: &LayoutContext,
    containing_block: &ContainingBlock,
    absolutely_positioned_fragments: &mut Vec<AbsolutelyPositionedFragment<'a>>,
    style: &Arc<ComputedValues>,
    block_level_kind: BlockLevelKind,
    layout_contents: impl FnOnce(
        &ContainingBlock,
        &mut Vec<AbsolutelyPositionedFragment<'a>>,
        CollapsibleWithParentStartMargin,
    ) -> FlowLayout,
) -> BoxFragment {
    let cbis = containing_block.inline_size;
    let padding = style.padding().percentages_relative_to(cbis);
    let border = style.border_width();
    let margin = style.margin().percentages_relative_to(cbis);
    let pb = &padding + &border;
    let pb_inline_sum = pb.inline_sum();

    let box_size = percent_resolved_box_size(style.box_size(), containing_block);
    let max_box_size = percent_resolved_max_box_size(style.max_box_size(), containing_block);
    let min_box_size =
        percent_resolved_box_size(style.min_box_size(), containing_block).auto_is(Length::zero);

    // https://drafts.csswg.org/css2/visudet.html#min-max-widths
    let solve_inline_margins = |inline_size| {
        solve_inline_margins_for_in_flow_block_level(
            containing_block,
            pb_inline_sum,
            margin.inline_start,
            margin.inline_end,
            inline_size,
        )
    };
    let (mut inline_size, mut inline_margins) =
        if let Some(inline_size) = box_size.inline.non_auto() {
            (inline_size, solve_inline_margins(inline_size))
        } else {
            let margin_inline_start = margin.inline_start.auto_is(Length::zero);
            let margin_inline_end = margin.inline_end.auto_is(Length::zero);
            let margin_inline_sum = margin_inline_start + margin_inline_end;
            let inline_size = cbis - pb_inline_sum - margin_inline_sum;
            (inline_size, (margin_inline_start, margin_inline_end))
        };
    if let Some(max_inline_size) = max_box_size.inline {
        if inline_size > max_inline_size {
            inline_size = max_inline_size;
            inline_margins = solve_inline_margins(inline_size);
        }
    }
    if inline_size < min_box_size.inline {
        inline_size = min_box_size.inline;
        inline_margins = solve_inline_margins(inline_size);
    }

    let margin = Sides {
        inline_start: inline_margins.0,
        inline_end: inline_margins.1,
        block_start: margin.block_start.auto_is(Length::zero),
        block_end: margin.block_end.auto_is(Length::zero),
    };

    // https://drafts.csswg.org/css2/visudet.html#min-max-heights
    let mut block_size = box_size.block;
    if let LengthOrAuto::LengthPercentage(ref mut block_size) = block_size {
        *block_size = clamp_between_extremums(*block_size, min_box_size.block, max_box_size.block);
    }

    let containing_block_for_children = ContainingBlock {
        inline_size,
        block_size,
        mode: style.writing_mode,
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
    let this_end_margin_can_collapse_with_children = block_size == LengthOrAuto::Auto &&
        min_box_size.block == Length::zero() &&
        pb.block_end == Length::zero() &&
        block_level_kind == BlockLevelKind::SameFormattingContextBlock;
    let mut nested_abspos = vec![];
    let mut flow_layout = layout_contents(
        &containing_block_for_children,
        if style.get_box().position == Position::Relative {
            &mut nested_abspos
        } else {
            absolutely_positioned_fragments
        },
        this_start_margin_can_collapse_with_children,
    );
    let mut block_margins_collapsed_with_children = CollapsedBlockMargins::from_margin(&margin);
    if this_start_margin_can_collapse_with_children.0 {
        block_margins_collapsed_with_children
            .start
            .adjoin_assign(&flow_layout.collapsible_margins_in_children.start);
        if flow_layout
            .collapsible_margins_in_children
            .collapsed_through
        {
            block_margins_collapsed_with_children
                .start
                .adjoin_assign(&std::mem::replace(
                    &mut flow_layout.collapsible_margins_in_children.end,
                    CollapsedMargin::zero(),
                ));
        }
    }
    if this_end_margin_can_collapse_with_children {
        block_margins_collapsed_with_children
            .end
            .adjoin_assign(&flow_layout.collapsible_margins_in_children.end);
    } else {
        flow_layout.content_block_size += flow_layout.collapsible_margins_in_children.end.solve();
    }
    block_margins_collapsed_with_children.collapsed_through =
        this_start_margin_can_collapse_with_children.0 &&
            this_end_margin_can_collapse_with_children &&
            flow_layout
                .collapsible_margins_in_children
                .collapsed_through;
    let relative_adjustement = relative_adjustement(style, inline_size, block_size);
    let block_size = block_size.auto_is(|| {
        clamp_between_extremums(
            flow_layout.content_block_size,
            min_box_size.block,
            max_box_size.block,
        )
    });
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
            layout_context,
            &nested_abspos,
            &mut flow_layout.fragments,
            &content_rect.size,
            &padding,
            containing_block_for_children.mode,
        )
    }
    BoxFragment {
        style: style.clone(),
        children: flow_layout.fragments,
        content_rect,
        padding,
        border,
        margin,
        block_margins_collapsed_with_children,
    }
}

/// https://drafts.csswg.org/css2/visudet.html#block-replaced-width
/// https://drafts.csswg.org/css2/visudet.html#inline-replaced-width
/// https://drafts.csswg.org/css2/visudet.html#inline-replaced-height
fn layout_in_flow_replaced_block_level<'a>(
    containing_block: &ContainingBlock,
    style: &Arc<ComputedValues>,
    replaced: &ReplacedContent,
) -> BoxFragment {
    let cbis = containing_block.inline_size;
    let padding = style.padding().percentages_relative_to(cbis);
    let border = style.border_width();
    let computed_margin = style.margin().percentages_relative_to(cbis);
    let pb = &padding + &border;
    let mode = style.writing_mode;
    // FIXME(nox): We shouldn't pretend we always have a fully known intrinsic size.
    let intrinsic_size = replaced.intrinsic_size.size_to_flow_relative(mode);
    // FIXME(nox): This can divide by zero.
    let intrinsic_ratio = intrinsic_size.inline.px() / intrinsic_size.block.px();

    let box_size = percent_resolved_box_size(style.box_size(), containing_block);
    let min_box_size =
        percent_resolved_box_size(style.min_box_size(), containing_block).auto_is(Length::zero);
    let max_box_size = percent_resolved_max_box_size(style.max_box_size(), containing_block);

    let clamp = |inline_size, block_size| {
        (
            clamp_between_extremums(inline_size, min_box_size.inline, max_box_size.inline),
            clamp_between_extremums(block_size, min_box_size.block, max_box_size.block),
        )
    };
    // https://drafts.csswg.org/css2/visudet.html#min-max-widths
    // https://drafts.csswg.org/css2/visudet.html#min-max-heights
    let (inline_size, block_size) = match (box_size.inline, box_size.block) {
        (LengthOrAuto::LengthPercentage(inline), LengthOrAuto::LengthPercentage(block)) => {
            clamp(inline, block)
        },
        (LengthOrAuto::LengthPercentage(inline), LengthOrAuto::Auto) => {
            clamp(inline, inline / intrinsic_ratio)
        },
        (LengthOrAuto::Auto, LengthOrAuto::LengthPercentage(block)) => {
            clamp(block * intrinsic_ratio, block)
        },
        (LengthOrAuto::Auto, LengthOrAuto::Auto) => {
            enum Violation {
                None,
                Below(Length),
                Above(Length),
            }
            let violation = |size, min_size, mut max_size: Option<Length>| {
                if let Some(max) = max_size.as_mut() {
                    max.max_assign(min_size);
                }
                if size < min_size {
                    return Violation::Below(min_size);
                }
                match max_size {
                    Some(max_size) if size > max_size => Violation::Above(max_size),
                    _ => Violation::None,
                }
            };
            match (
                violation(
                    intrinsic_size.inline,
                    min_box_size.inline,
                    max_box_size.inline,
                ),
                violation(intrinsic_size.block, min_box_size.block, max_box_size.block),
            ) {
                // Row 1.
                (Violation::None, Violation::None) => (intrinsic_size.inline, intrinsic_size.block),
                // Row 2.
                (Violation::Above(max_inline_size), Violation::None) => {
                    let block_size = (max_inline_size / intrinsic_ratio).max(min_box_size.block);
                    (max_inline_size, block_size)
                },
                // Row 3.
                (Violation::Below(min_inline_size), Violation::None) => {
                    let block_size =
                        clamp_below_max(min_inline_size / intrinsic_ratio, max_box_size.block);
                    (min_inline_size, block_size)
                },
                // Row 4.
                (Violation::None, Violation::Above(max_block_size)) => {
                    let inline_size = (max_block_size * intrinsic_ratio).max(min_box_size.inline);
                    (inline_size, max_block_size)
                },
                // Row 5.
                (Violation::None, Violation::Below(min_block_size)) => {
                    let inline_size =
                        clamp_below_max(min_block_size * intrinsic_ratio, max_box_size.inline);
                    (inline_size, min_block_size)
                },
                // Rows 6-7.
                (Violation::Above(max_inline_size), Violation::Above(max_block_size)) => {
                    if max_inline_size.px() / intrinsic_size.inline.px() <=
                        max_block_size.px() / intrinsic_size.block.px()
                    {
                        // Row 6.
                        let block_size =
                            (max_inline_size / intrinsic_ratio).max(min_box_size.block);
                        (max_inline_size, block_size)
                    } else {
                        // Row 7.
                        let inline_size =
                            (max_block_size * intrinsic_ratio).max(min_box_size.inline);
                        (inline_size, max_block_size)
                    }
                },
                // Rows 8-9.
                (Violation::Below(min_inline_size), Violation::Below(min_block_size)) => {
                    if min_inline_size.px() / intrinsic_size.inline.px() <=
                        min_block_size.px() / intrinsic_size.block.px()
                    {
                        // Row 8.
                        let inline_size =
                            clamp_below_max(min_block_size * intrinsic_ratio, max_box_size.inline);
                        (inline_size, min_block_size)
                    } else {
                        // Row 9.
                        let block_size =
                            clamp_below_max(min_inline_size / intrinsic_ratio, max_box_size.block);
                        (min_inline_size, block_size)
                    }
                },
                // Row 10.
                (Violation::Below(min_inline_size), Violation::Above(max_block_size)) => {
                    (min_inline_size, max_block_size)
                },
                // Row 11.
                (Violation::Above(max_inline_size), Violation::Below(min_block_size)) => {
                    (max_inline_size, min_block_size)
                },
            }
        },
    };

    let (margin_inline_start, margin_inline_end) = solve_inline_margins_for_in_flow_block_level(
        containing_block,
        pb.inline_sum(),
        computed_margin.inline_start,
        computed_margin.inline_end,
        inline_size,
    );
    let margin = Sides {
        inline_start: margin_inline_start,
        inline_end: margin_inline_end,
        block_start: computed_margin.block_start.auto_is(Length::zero),
        block_end: computed_margin.block_end.auto_is(Length::zero),
    };
    let size = Vec2 {
        block: block_size,
        inline: inline_size,
    };
    let fragments = replaced.make_fragments(style, size.clone());
    let relative_adjustement = relative_adjustement(
        style,
        inline_size,
        LengthOrAuto::LengthPercentage(block_size),
    );
    let content_rect = Rect {
        start_corner: Vec2 {
            block: pb.block_start + relative_adjustement.block,
            inline: pb.inline_start + relative_adjustement.inline + margin.inline_start,
        },
        size,
    };
    BoxFragment {
        style: style.clone(),
        children: fragments,
        content_rect,
        padding,
        border,
        block_margins_collapsed_with_children: CollapsedBlockMargins::from_margin(&margin),
        margin,
    }
}

fn solve_inline_margins_for_in_flow_block_level(
    containing_block: &ContainingBlock,
    padding_border_inline_sum: Length,
    computed_margin_inline_start: LengthOrAuto,
    computed_margin_inline_end: LengthOrAuto,
    inline_size: Length,
) -> (Length, Length) {
    let inline_margins = containing_block.inline_size - padding_border_inline_sum - inline_size;
    match (computed_margin_inline_start, computed_margin_inline_end) {
        (LengthOrAuto::Auto, LengthOrAuto::Auto) => (inline_margins / 2., inline_margins / 2.),
        (LengthOrAuto::Auto, LengthOrAuto::LengthPercentage(end)) => (inline_margins - end, end),
        (LengthOrAuto::LengthPercentage(start), _) => (start, inline_margins - start),
    }
}

fn clamp_between_extremums(size: Length, min_size: Length, max_size: Option<Length>) -> Length {
    clamp_below_max(size, max_size).max(min_size)
}

fn clamp_below_max(size: Length, max_size: Option<Length>) -> Length {
    max_size.map_or(size, |max_size| size.min(max_size))
}

fn percent_resolved_box_size(
    box_size: Vec2<LengthPercentageOrAuto>,
    containing_block: &ContainingBlock,
) -> Vec2<LengthOrAuto> {
    Vec2 {
        inline: box_size
            .inline
            .percentage_relative_to(containing_block.inline_size),
        block: box_size
            .block
            .maybe_percentage_relative_to(containing_block.block_size.non_auto()),
    }
}

fn percent_resolved_max_box_size(
    max_box_size: Vec2<MaxSize<LengthPercentage>>,
    containing_block: &ContainingBlock,
) -> Vec2<Option<Length>> {
    Vec2 {
        inline: match max_box_size.inline {
            MaxSize::LengthPercentage(max_inline_size) => {
                Some(max_inline_size.percentage_relative_to(containing_block.inline_size))
            },
            MaxSize::None => None,
        },
        block: match max_box_size.block {
            MaxSize::LengthPercentage(max_block_size) => {
                max_block_size.maybe_percentage_relative_to(containing_block.block_size.non_auto())
            },
            MaxSize::None => None,
        },
    }
}
