/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Flow layout, also known as block-and-inline layout.

use crate::cell::ArcRefCell;
use crate::context::LayoutContext;
use crate::dom_traversal::NodeFlags;
use crate::flow::float::{FloatBox, FloatContext};
use crate::flow::inline::InlineFormattingContext;
use crate::formatting_contexts::{IndependentFormattingContext, IndependentLayout, NonReplacedIFC};
use crate::fragments::{AbsoluteOrFixedPositionedFragment, AnonymousFragment, BoxFragment};
use crate::fragments::{CollapsedBlockMargins, CollapsedMargin, Fragment};
use crate::geom::flow_relative::{Rect, Sides, Vec2};
use crate::positioned::{AbsolutelyPositionedBox, PositioningContext};
use crate::replaced::ReplacedContent;
use crate::style_ext::{ComputedValuesExt, PaddingBorderMargin};
use crate::ContainingBlock;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use rayon_croissant::ParallelIteratorExt;
use servo_arc::Arc;
use style::dom::OpaqueNode;
use style::properties::ComputedValues;
use style::values::computed::{Length, LengthOrAuto};
use style::Zero;

mod construct;
mod float;
pub mod inline;
mod root;

pub use root::{BoxTreeRoot, FragmentTreeRoot};

#[derive(Debug, Serialize)]
pub(crate) struct BlockFormattingContext {
    pub contents: BlockContainer,
    pub contains_floats: bool,
}

#[derive(Debug, Serialize)]
pub(crate) enum BlockContainer {
    BlockLevelBoxes(Vec<ArcRefCell<BlockLevelBox>>),
    InlineFormattingContext(InlineFormattingContext),
}

#[derive(Debug, Serialize)]
pub(crate) enum BlockLevelBox {
    SameFormattingContextBlock {
        tag: OpaqueNode,
        #[serde(skip_serializing)]
        style: Arc<ComputedValues>,
        contents: BlockContainer,
        flags: NodeFlags,
    },
    OutOfFlowAbsolutelyPositionedBox(Arc<AbsolutelyPositionedBox>),
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
    pub(super) fn layout(
        &self,
        layout_context: &LayoutContext,
        positioning_context: &mut PositioningContext,
        containing_block: &ContainingBlock,
        tree_rank: usize,
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
            positioning_context,
            containing_block,
            tree_rank,
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
    fn layout(
        &self,
        layout_context: &LayoutContext,
        positioning_context: &mut PositioningContext,
        containing_block: &ContainingBlock,
        tree_rank: usize,
        float_context: Option<&mut FloatContext>,
        collapsible_with_parent_start_margin: CollapsibleWithParentStartMargin,
    ) -> FlowLayout {
        match self {
            BlockContainer::BlockLevelBoxes(child_boxes) => layout_block_level_children(
                layout_context,
                positioning_context,
                child_boxes,
                containing_block,
                tree_rank,
                float_context,
                collapsible_with_parent_start_margin,
            ),
            BlockContainer::InlineFormattingContext(ifc) => ifc.layout(
                layout_context,
                positioning_context,
                containing_block,
                tree_rank,
            ),
        }
    }
}

fn layout_block_level_children(
    layout_context: &LayoutContext,
    positioning_context: &mut PositioningContext,
    child_boxes: &[ArcRefCell<BlockLevelBox>],
    containing_block: &ContainingBlock,
    tree_rank: usize,
    mut float_context: Option<&mut FloatContext>,
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
            Fragment::Anonymous(_) | Fragment::AbsoluteOrFixedPositioned(_) => {},
            _ => unreachable!(),
        }
    }

    struct PlacementState {
        next_in_flow_margin_collapses_with_parent_start_margin: bool,
        start_margin: CollapsedMargin,
        current_margin: CollapsedMargin,
        current_block_direction_position: Length,
    }

    let mut placement_state = PlacementState {
        next_in_flow_margin_collapses_with_parent_start_margin:
            collapsible_with_parent_start_margin.0,
        start_margin: CollapsedMargin::zero(),
        current_margin: CollapsedMargin::zero(),
        current_block_direction_position: Length::zero(),
    };
    let fragments = positioning_context.adjust_static_positions(tree_rank, |positioning_context| {
        if float_context.is_some() || !layout_context.use_rayon {
            // Because floats are involved, we do layout for this block formatting context
            // in tree order without parallelism. This enables mutable access
            // to a `FloatContext` that tracks every float encountered so far (again in tree order).
            child_boxes
                .iter()
                .enumerate()
                .map(|(tree_rank, box_)| {
                    let mut fragment = box_.borrow().layout(
                        layout_context,
                        positioning_context,
                        containing_block,
                        tree_rank,
                        float_context.as_mut().map(|c| &mut **c),
                    );
                    place_block_level_fragment(&mut fragment, &mut placement_state);
                    fragment
                })
                .collect()
        } else {
            let collects_for_nearest_positioned_ancestor =
                positioning_context.collects_for_nearest_positioned_ancestor();
            let mut fragments = child_boxes
                .par_iter()
                .enumerate()
                .mapfold_reduce_into(
                    positioning_context,
                    |positioning_context, (tree_rank, box_)| {
                        box_.borrow().layout(
                            layout_context,
                            positioning_context,
                            containing_block,
                            tree_rank,
                            /* float_context = */ None,
                        )
                    },
                    || PositioningContext::new_for_rayon(collects_for_nearest_positioned_ancestor),
                    PositioningContext::append,
                )
                .collect();
            for fragment in &mut fragments {
                place_block_level_fragment(fragment, &mut placement_state)
            }
            fragments
        }
    });

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
    fn layout(
        &self,
        layout_context: &LayoutContext,
        positioning_context: &mut PositioningContext,
        containing_block: &ContainingBlock,
        tree_rank: usize,
        float_context: Option<&mut FloatContext>,
    ) -> Fragment {
        match self {
            BlockLevelBox::SameFormattingContextBlock {
                tag,
                style,
                contents,
                flags,
            } => Fragment::Box(positioning_context.layout_maybe_position_relative_fragment(
                layout_context,
                containing_block,
                style,
                |positioning_context| {
                    layout_in_flow_non_replaced_block_level(
                        layout_context,
                        positioning_context,
                        containing_block,
                        *tag,
                        style,
                        NonReplacedContents::SameFormattingContextBlock(contents),
                        tree_rank,
                        float_context,
                        *flags,
                    )
                },
            )),
            BlockLevelBox::Independent(contents) => {
                Fragment::Box(positioning_context.layout_maybe_position_relative_fragment(
                    layout_context,
                    containing_block,
                    &contents.style,
                    |positioning_context| match contents.as_replaced() {
                        Ok(replaced) => layout_in_flow_replaced_block_level(
                            containing_block,
                            contents.tag,
                            &contents.style,
                            replaced,
                            contents.flags,
                        ),
                        Err(non_replaced) => layout_in_flow_non_replaced_block_level(
                            layout_context,
                            positioning_context,
                            containing_block,
                            contents.tag,
                            &contents.style,
                            NonReplacedContents::EstablishesAnIndependentFormattingContext(
                                non_replaced,
                            ),
                            tree_rank,
                            float_context,
                            contents.flags,
                        ),
                    },
                ))
            },
            BlockLevelBox::OutOfFlowAbsolutelyPositionedBox(box_) => {
                let hoisted_box = box_.clone().to_hoisted(Vec2::zero(), tree_rank);
                let hoisted_fragment = hoisted_box.fragment.clone();
                positioning_context.push(hoisted_box);
                Fragment::AbsoluteOrFixedPositioned(AbsoluteOrFixedPositionedFragment {
                    hoisted_fragment,
                    position: box_.contents.style.clone_position(),
                })
            },
            BlockLevelBox::OutOfFlowFloatBox(_box_) => {
                // FIXME: call layout_maybe_position_relative_fragment here
                Fragment::Anonymous(AnonymousFragment::no_op(
                    containing_block.style.writing_mode,
                ))
            },
        }
    }
}

enum NonReplacedContents<'a> {
    SameFormattingContextBlock(&'a BlockContainer),
    EstablishesAnIndependentFormattingContext(NonReplacedIFC<'a>),
}

/// https://drafts.csswg.org/css2/visudet.html#blockwidth
/// https://drafts.csswg.org/css2/visudet.html#normal-block
fn layout_in_flow_non_replaced_block_level(
    layout_context: &LayoutContext,
    positioning_context: &mut PositioningContext,
    containing_block: &ContainingBlock,
    tag: OpaqueNode,
    style: &Arc<ComputedValues>,
    block_level_kind: NonReplacedContents,
    tree_rank: usize,
    float_context: Option<&mut FloatContext>,
    flags: NodeFlags,
) -> BoxFragment {
    let pbm = style.padding_border_margin(containing_block);
    let box_size = style.content_box_size(containing_block, &pbm);
    let max_box_size = style.content_max_box_size(containing_block, &pbm);
    let min_box_size = style
        .content_min_box_size(containing_block, &pbm)
        .auto_is(Length::zero);

    // https://drafts.csswg.org/css2/visudet.html#min-max-widths
    let solve_inline_margins = |inline_size| {
        solve_inline_margins_for_in_flow_block_level(containing_block, &pbm, inline_size)
    };
    let (mut inline_size, mut inline_margins) =
        if let Some(inline_size) = box_size.inline.non_auto() {
            (inline_size, solve_inline_margins(inline_size))
        } else {
            let margin_inline_start = pbm.margin.inline_start.auto_is(Length::zero);
            let margin_inline_end = pbm.margin.inline_end.auto_is(Length::zero);
            let inline_size = containing_block.inline_size -
                pbm.padding_border_sums.inline -
                margin_inline_start -
                margin_inline_end;
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
        block_start: pbm.margin.block_start.auto_is(Length::zero),
        block_end: pbm.margin.block_end.auto_is(Length::zero),
    };

    // https://drafts.csswg.org/css2/visudet.html#min-max-heights
    let mut block_size = box_size.block;
    if let LengthOrAuto::LengthPercentage(ref mut block_size) = block_size {
        *block_size = block_size.clamp_between_extremums(min_box_size.block, max_box_size.block);
    }

    let containing_block_for_children = ContainingBlock {
        inline_size,
        block_size,
        style,
    };
    // https://drafts.csswg.org/css-writing-modes/#orthogonal-flows
    assert_eq!(
        containing_block.style.writing_mode, containing_block_for_children.style.writing_mode,
        "Mixed writing modes are not supported yet"
    );

    let mut block_margins_collapsed_with_children = CollapsedBlockMargins::from_margin(&margin);

    let fragments;
    let mut content_block_size;
    match block_level_kind {
        NonReplacedContents::SameFormattingContextBlock(contents) => {
            let start_margin_can_collapse_with_children = pbm.padding.block_start == Length::zero() &&
                pbm.border.block_start == Length::zero();
            let end_margin_can_collapse_with_children = pbm.padding.block_end == Length::zero() &&
                pbm.border.block_end == Length::zero() &&
                block_size == LengthOrAuto::Auto &&
                min_box_size.block == Length::zero();

            let flow_layout = contents.layout(
                layout_context,
                positioning_context,
                &containing_block_for_children,
                tree_rank,
                float_context,
                CollapsibleWithParentStartMargin(start_margin_can_collapse_with_children),
            );
            fragments = flow_layout.fragments;
            content_block_size = flow_layout.content_block_size;
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
            if end_margin_can_collapse_with_children {
                block_margins_collapsed_with_children
                    .end
                    .adjoin_assign(&collapsible_margins_in_children.end);
            } else {
                content_block_size += collapsible_margins_in_children.end.solve();
            }
            block_margins_collapsed_with_children.collapsed_through =
                start_margin_can_collapse_with_children &&
                    end_margin_can_collapse_with_children &&
                    collapsible_margins_in_children.collapsed_through;
        },
        NonReplacedContents::EstablishesAnIndependentFormattingContext(non_replaced) => {
            let independent_layout = non_replaced.layout(
                layout_context,
                positioning_context,
                &containing_block_for_children,
                tree_rank,
            );
            fragments = independent_layout.fragments;
            content_block_size = independent_layout.content_block_size;
        },
    };
    let block_size = block_size.auto_is(|| {
        content_block_size.clamp_between_extremums(min_box_size.block, max_box_size.block)
    });
    let content_rect = Rect {
        start_corner: Vec2 {
            block: pbm.padding.block_start + pbm.border.block_start,
            inline: pbm.padding.inline_start + pbm.border.inline_start + margin.inline_start,
        },
        size: Vec2 {
            block: block_size,
            inline: inline_size,
        },
    };
    BoxFragment::new(
        tag,
        style.clone(),
        fragments,
        content_rect,
        pbm.padding,
        pbm.border,
        margin,
        block_margins_collapsed_with_children,
        flags,
    )
}

/// https://drafts.csswg.org/css2/visudet.html#block-replaced-width
/// https://drafts.csswg.org/css2/visudet.html#inline-replaced-width
/// https://drafts.csswg.org/css2/visudet.html#inline-replaced-height
fn layout_in_flow_replaced_block_level<'a>(
    containing_block: &ContainingBlock,
    tag: OpaqueNode,
    style: &Arc<ComputedValues>,
    replaced: &ReplacedContent,
    flags: NodeFlags,
) -> BoxFragment {
    let pbm = style.padding_border_margin(containing_block);
    let size = replaced.used_size_as_if_inline_element(containing_block, style, &pbm);

    let (margin_inline_start, margin_inline_end) =
        solve_inline_margins_for_in_flow_block_level(containing_block, &pbm, size.inline);
    let margin = Sides {
        inline_start: margin_inline_start,
        inline_end: margin_inline_end,
        block_start: pbm.margin.block_start.auto_is(Length::zero),
        block_end: pbm.margin.block_end.auto_is(Length::zero),
    };
    let fragments = replaced.make_fragments(style, size.clone());
    let content_rect = Rect {
        start_corner: Vec2 {
            block: pbm.padding.block_start + pbm.border.block_start,
            inline: pbm.padding.inline_start + pbm.border.inline_start + margin.inline_start,
        },
        size,
    };
    let block_margins_collapsed_with_children = CollapsedBlockMargins::from_margin(&margin);
    BoxFragment::new(
        tag,
        style.clone(),
        fragments,
        content_rect,
        pbm.padding,
        pbm.border,
        margin,
        block_margins_collapsed_with_children,
        flags,
    )
}

fn solve_inline_margins_for_in_flow_block_level(
    containing_block: &ContainingBlock,
    pbm: &PaddingBorderMargin,
    inline_size: Length,
) -> (Length, Length) {
    let available = containing_block.inline_size - pbm.padding_border_sums.inline - inline_size;
    match (pbm.margin.inline_start, pbm.margin.inline_end) {
        (LengthOrAuto::Auto, LengthOrAuto::Auto) => (available / 2., available / 2.),
        (LengthOrAuto::Auto, LengthOrAuto::LengthPercentage(end)) => (available - end, end),
        (LengthOrAuto::LengthPercentage(start), _) => (start, available - start),
    }
}
