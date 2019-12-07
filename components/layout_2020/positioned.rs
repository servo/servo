/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::context::LayoutContext;
use crate::dom_traversal::{Contents, NodeExt};
use crate::formatting_contexts::IndependentFormattingContext;
use crate::fragments::{AnonymousFragment, BoxFragment, CollapsedBlockMargins, Fragment};
use crate::geom::flow_relative::{Rect, Sides, Vec2};
use crate::sizing::ContentSizesRequest;
use crate::style_ext::{ComputedValuesExt, DisplayInside};
use crate::{ContainingBlock, DefiniteContainingBlock};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use servo_arc::Arc;
use style::properties::ComputedValues;
use style::values::computed::{Length, LengthOrAuto, LengthPercentage, LengthPercentageOrAuto};
use style::Zero;

#[derive(Debug)]
pub(crate) struct AbsolutelyPositionedBox {
    pub contents: IndependentFormattingContext,
}

#[derive(Debug)]
pub(crate) struct AbsolutelyPositionedFragment<'box_> {
    absolutely_positioned_box: &'box_ AbsolutelyPositionedBox,

    /// The rank of the child from which this absolutely positioned fragment
    /// came from, when doing the layout of a block container. Used to compute
    /// static positions when going up the tree.
    pub(crate) tree_rank: usize,

    pub(crate) inline_start: AbsoluteBoxOffsets<LengthPercentage>,
    pub(crate) block_start: AbsoluteBoxOffsets<LengthPercentage>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum AbsoluteBoxOffsets<NonStatic> {
    StaticStart { start: Length },
    Start { start: NonStatic },
    End { end: NonStatic },
    Both { start: NonStatic, end: NonStatic },
}

impl AbsolutelyPositionedBox {
    pub fn construct<'dom>(
        context: &LayoutContext,
        style: Arc<ComputedValues>,
        display_inside: DisplayInside,
        contents: Contents<impl NodeExt<'dom>>,
    ) -> Self {
        // "Shrink-to-fit" in https://drafts.csswg.org/css2/visudet.html#abs-non-replaced-width
        let content_sizes = ContentSizesRequest::inline_if(
            // If inline-size is non-auto, that value is used without shrink-to-fit
            style.inline_size_is_auto() &&
            // If it is, then the only case where shrink-to-fit is *not* used is
            // if both offsets are non-auto, leaving inline-size as the only variable
            // in the constraint equation.
            !style.inline_box_offsets_are_both_non_auto(),
        );
        Self {
            contents: IndependentFormattingContext::construct(
                context,
                style,
                display_inside,
                contents,
                content_sizes,
            ),
        }
    }

    pub(crate) fn layout<'a>(
        &'a self,
        initial_start_corner: Vec2<Length>,
        tree_rank: usize,
    ) -> AbsolutelyPositionedFragment {
        let style = &self.contents.style;
        let box_offsets = style.box_offsets();

        fn absolute_box_offsets(
            initial_static_start: Length,
            start: LengthPercentageOrAuto,
            end: LengthPercentageOrAuto,
        ) -> AbsoluteBoxOffsets<LengthPercentage> {
            match (start.non_auto(), end.non_auto()) {
                (None, None) => AbsoluteBoxOffsets::StaticStart {
                    start: initial_static_start,
                },
                (Some(start), Some(end)) => AbsoluteBoxOffsets::Both { start, end },
                (None, Some(end)) => AbsoluteBoxOffsets::End { end },
                (Some(start), None) => AbsoluteBoxOffsets::Start { start },
            }
        }

        let inline_start = absolute_box_offsets(
            initial_start_corner.inline,
            box_offsets.inline_start,
            box_offsets.inline_end,
        );
        let block_start = absolute_box_offsets(
            initial_start_corner.block,
            box_offsets.block_start,
            box_offsets.block_end,
        );

        AbsolutelyPositionedFragment {
            absolutely_positioned_box: self,
            tree_rank,
            inline_start,
            block_start,
        }
    }
}

impl<'a> AbsolutelyPositionedFragment<'a> {
    pub(crate) fn in_positioned_containing_block(
        layout_context: &LayoutContext,
        absolute: &[Self],
        fragments: &mut Vec<Fragment>,
        content_rect_size: &Vec2<Length>,
        padding: &Sides<Length>,
        style: &ComputedValues,
    ) {
        if absolute.is_empty() {
            return;
        }
        let padding_rect = Rect {
            size: content_rect_size.clone(),
            // Ignore the content rect’s position in its own containing block:
            start_corner: Vec2::zero(),
        }
        .inflate(&padding);
        let containing_block = DefiniteContainingBlock {
            size: padding_rect.size.clone(),
            style,
        };
        fragments.push(Fragment::Anonymous(AnonymousFragment {
            children: absolute
                .par_iter()
                .map(|a| a.layout(layout_context, &containing_block))
                .collect(),
            rect: padding_rect,
            mode: style.writing_mode,
        }))
    }

    pub(crate) fn layout(
        &self,
        layout_context: &LayoutContext,
        containing_block: &DefiniteContainingBlock,
    ) -> Fragment {
        let style = &self.absolutely_positioned_box.contents.style;
        let cbis = containing_block.size.inline;
        let cbbs = containing_block.size.block;

        let box_size = style.box_size();
        let padding = style.padding().percentages_relative_to(cbis);
        let border = style.border_width();
        let computed_margin = style.margin().percentages_relative_to(cbis);
        let pb = &padding + &border;

        enum Anchor {
            Start(Length),
            End(Length),
        }

        fn solve_axis(
            containing_size: Length,
            padding_border_sum: Length,
            computed_margin_start: LengthOrAuto,
            computed_margin_end: LengthOrAuto,
            solve_margins: impl FnOnce(Length) -> (Length, Length),
            box_offsets: AbsoluteBoxOffsets<LengthPercentage>,
            size: LengthPercentageOrAuto,
        ) -> (Anchor, LengthOrAuto, Length, Length) {
            let size = size.percentage_relative_to(containing_size);
            match box_offsets {
                AbsoluteBoxOffsets::StaticStart { start } => (
                    Anchor::Start(start),
                    size,
                    computed_margin_start.auto_is(Length::zero),
                    computed_margin_end.auto_is(Length::zero),
                ),
                AbsoluteBoxOffsets::Start { start } => (
                    Anchor::Start(start.percentage_relative_to(containing_size)),
                    size,
                    computed_margin_start.auto_is(Length::zero),
                    computed_margin_end.auto_is(Length::zero),
                ),
                AbsoluteBoxOffsets::End { end } => (
                    Anchor::End(end.percentage_relative_to(containing_size)),
                    size,
                    computed_margin_start.auto_is(Length::zero),
                    computed_margin_end.auto_is(Length::zero),
                ),
                AbsoluteBoxOffsets::Both { start, end } => {
                    let start = start.percentage_relative_to(containing_size);
                    let end = end.percentage_relative_to(containing_size);

                    let mut margin_start = computed_margin_start.auto_is(Length::zero);
                    let mut margin_end = computed_margin_end.auto_is(Length::zero);

                    let size = if let LengthOrAuto::LengthPercentage(size) = size {
                        let margins = containing_size - start - end - padding_border_sum - size;
                        match (computed_margin_start, computed_margin_end) {
                            (LengthOrAuto::Auto, LengthOrAuto::Auto) => {
                                let (s, e) = solve_margins(margins);
                                margin_start = s;
                                margin_end = e;
                            },
                            (LengthOrAuto::Auto, LengthOrAuto::LengthPercentage(end)) => {
                                margin_start = margins - end;
                            },
                            (LengthOrAuto::LengthPercentage(start), LengthOrAuto::Auto) => {
                                margin_end = margins - start;
                            },
                            (
                                LengthOrAuto::LengthPercentage(_),
                                LengthOrAuto::LengthPercentage(_),
                            ) => {},
                        }
                        size
                    } else {
                        // FIXME(nox): What happens if that is negative?
                        containing_size -
                            start -
                            end -
                            padding_border_sum -
                            margin_start -
                            margin_end
                    };
                    (
                        Anchor::Start(start),
                        LengthOrAuto::LengthPercentage(size),
                        margin_start,
                        margin_end,
                    )
                },
            }
        }

        let (inline_anchor, inline_size, margin_inline_start, margin_inline_end) = solve_axis(
            cbis,
            pb.inline_sum(),
            computed_margin.inline_start,
            computed_margin.inline_end,
            |margins| {
                if margins.px() >= 0. {
                    (margins / 2., margins / 2.)
                } else {
                    (Length::zero(), margins)
                }
            },
            self.inline_start,
            box_size.inline,
        );

        let (block_anchor, block_size, margin_block_start, margin_block_end) = solve_axis(
            cbis,
            pb.block_sum(),
            computed_margin.block_start,
            computed_margin.block_end,
            |margins| (margins / 2., margins / 2.),
            self.block_start,
            box_size.block,
        );

        let margin = Sides {
            inline_start: margin_inline_start,
            inline_end: margin_inline_end,
            block_start: margin_block_start,
            block_end: margin_block_end,
        };

        let inline_size = inline_size.auto_is(|| {
            let available_size = match inline_anchor {
                Anchor::Start(start) => cbis - start - pb.inline_sum() - margin.inline_sum(),
                Anchor::End(end) => cbis - end - pb.inline_sum() - margin.inline_sum(),
            };

            if self
                .absolutely_positioned_box
                .contents
                .as_replaced()
                .is_ok()
            {
                // FIXME: implement https://drafts.csswg.org/css2/visudet.html#abs-replaced-width
                available_size
            } else {
                self.absolutely_positioned_box
                    .contents
                    .content_sizes
                    .shrink_to_fit(available_size)
            }
        });

        let mut absolutely_positioned_fragments = Vec::new();
        let mut independent_layout = match self.absolutely_positioned_box.contents.as_replaced() {
            Ok(replaced) => {
                // FIXME: implement https://drafts.csswg.org/css2/visudet.html#abs-replaced-width
                // and https://drafts.csswg.org/css2/visudet.html#abs-replaced-height
                let block_size = block_size.auto_is(Length::zero);
                let fragments = replaced.make_fragments(
                    &self.absolutely_positioned_box.contents.style,
                    Vec2 {
                        inline: inline_size,
                        block: block_size,
                    },
                );
                crate::formatting_contexts::IndependentLayout {
                    fragments,
                    content_block_size: block_size,
                }
            },
            Err(non_replaced) => {
                let containing_block_for_children = ContainingBlock {
                    inline_size,
                    block_size,
                    style,
                };
                // https://drafts.csswg.org/css-writing-modes/#orthogonal-flows
                assert_eq!(
                    containing_block.style.writing_mode,
                    containing_block_for_children.style.writing_mode,
                    "Mixed writing modes are not supported yet"
                );
                let dummy_tree_rank = 0;
                non_replaced.layout(
                    layout_context,
                    &containing_block_for_children,
                    dummy_tree_rank,
                    &mut absolutely_positioned_fragments,
                )
            },
        };

        let inline_start = match inline_anchor {
            Anchor::Start(start) => start + pb.inline_start + margin.inline_start,
            Anchor::End(end) => cbbs - end - pb.inline_end - margin.inline_end - inline_size,
        };

        let block_size = block_size.auto_is(|| independent_layout.content_block_size);
        let block_start = match block_anchor {
            Anchor::Start(start) => start + pb.block_start + margin.block_start,
            Anchor::End(end) => cbbs - end - pb.block_end - margin.block_end - block_size,
        };

        let content_rect = Rect {
            start_corner: Vec2 {
                inline: inline_start,
                block: block_start,
            },
            size: Vec2 {
                inline: inline_size,
                block: block_size,
            },
        };

        AbsolutelyPositionedFragment::in_positioned_containing_block(
            layout_context,
            &absolutely_positioned_fragments,
            &mut independent_layout.fragments,
            &content_rect.size,
            &padding,
            style,
        );

        Fragment::Box(BoxFragment {
            style: style.clone(),
            children: independent_layout.fragments,
            content_rect,
            padding,
            border,
            margin,
            block_margins_collapsed_with_children: CollapsedBlockMargins::zero(),
        })
    }
}

pub(crate) fn adjust_static_positions(
    absolutely_positioned_fragments: &mut [AbsolutelyPositionedFragment],
    child_fragments: &mut [Fragment],
    tree_rank_in_parent: usize,
) {
    for abspos_fragment in absolutely_positioned_fragments {
        let child_fragment_rect = match &child_fragments[abspos_fragment.tree_rank] {
            Fragment::Box(b) => &b.content_rect,
            Fragment::Anonymous(a) => &a.rect,
            _ => unreachable!(),
        };

        abspos_fragment.tree_rank = tree_rank_in_parent;

        if let AbsoluteBoxOffsets::StaticStart { start } = &mut abspos_fragment.inline_start {
            *start += child_fragment_rect.start_corner.inline;
        }

        if let AbsoluteBoxOffsets::StaticStart { start } = &mut abspos_fragment.block_start {
            *start += child_fragment_rect.start_corner.block;
        }
    }
}
