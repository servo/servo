use super::*;
use rayon::prelude::*;

#[derive(Debug)]
pub(super) struct AbsolutelyPositionedBox {
    pub style: Arc<ComputedValues>,
    pub contents: IndependentFormattingContext,
}

#[derive(Debug)]
pub(super) struct AbsolutelyPositionedFragment<'box_> {
    absolutely_positioned_box: &'box_ AbsolutelyPositionedBox,

    /// The rank of the child from which this absolutely positioned fragment
    /// came from, when doing the layout of a block container. Used to compute
    /// static positions when going up the tree.
    pub(super) tree_rank: usize,

    pub(super) inline_start: AbsoluteBoxOffsets<LengthOrPercentage>,
    inline_size: LengthOrPercentageOrAuto,

    pub(super) block_start: AbsoluteBoxOffsets<LengthOrPercentage>,
    block_size: LengthOrPercentageOrAuto,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum AbsoluteBoxOffsets<NonStatic> {
    StaticStart { start: Length },
    Start { start: NonStatic },
    End { end: NonStatic },
    Both { start: NonStatic, end: NonStatic },
}

impl AbsolutelyPositionedBox {
    pub(super) fn layout<'a>(
        &'a self,
        initial_start_corner: Vec2<Length>,
        tree_rank: usize,
    ) -> AbsolutelyPositionedFragment {
        let style = &self.style;
        let box_offsets = style.box_offsets();
        let box_size = style.box_size();

        let inline_size = box_size.inline;
        let block_size = box_size.block;

        fn absolute_box_offsets(
            initial_static_start: Length,
            start: LengthOrPercentageOrAuto,
            end: LengthOrPercentageOrAuto,
        ) -> AbsoluteBoxOffsets<LengthOrPercentage> {
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
            inline_size,
            block_start,
            block_size,
        }
    }
}

impl<'a> AbsolutelyPositionedFragment<'a> {
    pub(super) fn in_positioned_containing_block(
        absolute: &[Self],
        fragments: &mut Vec<Fragment>,
        content_rect_size: &Vec2<Length>,
        padding: &Sides<Length>,
        mode: (WritingMode, Direction),
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
            mode,
        };
        fragments.push(Fragment::Anonymous(AnonymousFragment {
            children: absolute
                .par_iter()
                .map(|a| a.layout(&containing_block))
                .collect(),
            rect: padding_rect,
            mode,
        }))
    }

    pub(super) fn layout(&self, containing_block: &DefiniteContainingBlock) -> Fragment {
        let style = &self.absolutely_positioned_box.style;
        let cbis = containing_block.size.inline;
        let cbbs = containing_block.size.block;

        let padding = style.padding().percentages_relative_to(cbis);
        let border = style.border_width().percentages_relative_to(cbis);
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
            box_offsets: AbsoluteBoxOffsets<LengthOrPercentage>,
            size: LengthOrPercentageOrAuto,
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

                    let size = if let LengthOrAuto::Length(size) = size {
                        use LengthOrAuto::Auto;
                        let margins = containing_size - start - end - padding_border_sum - size;
                        match (computed_margin_start, computed_margin_end) {
                            (Auto, Auto) => {
                                let (s, e) = solve_margins(margins);
                                margin_start = s;
                                margin_end = e;
                            }
                            (Auto, LengthOrAuto::Length(end)) => {
                                margin_start = margins - end;
                            }
                            (LengthOrAuto::Length(start), Auto) => {
                                margin_end = margins - start;
                            }
                            (LengthOrAuto::Length(_), LengthOrAuto::Length(_)) => {}
                        }
                        size
                    } else {
                        // FIXME(nox): What happens if that is negative?
                        containing_size
                            - start
                            - end
                            - padding_border_sum
                            - margin_start
                            - margin_end
                    };
                    (
                        Anchor::Start(start),
                        LengthOrAuto::Length(size),
                        margin_start,
                        margin_end,
                    )
                }
            }
        }

        let (inline_anchor, inline_size, margin_inline_start, margin_inline_end) = solve_axis(
            cbis,
            pb.inline_sum(),
            computed_margin.inline_start,
            computed_margin.inline_end,
            |margins| {
                if margins.px >= 0. {
                    (margins / 2., margins / 2.)
                } else {
                    (Length::zero(), margins)
                }
            },
            self.inline_start,
            self.inline_size,
        );

        let (block_anchor, block_size, margin_block_start, margin_block_end) = solve_axis(
            cbis,
            pb.block_sum(),
            computed_margin.block_start,
            computed_margin.block_end,
            |margins| (margins / 2., margins / 2.),
            self.block_start,
            self.block_size,
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

            // FIXME(nox): shrink-to-fit.
            available_size
        });

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
        let dummy_tree_rank = 0;
        let mut absolutely_positioned_fragments = vec![];
        let mut flow_children = self.absolutely_positioned_box.contents.layout(
            &containing_block_for_children,
            dummy_tree_rank,
            &mut absolutely_positioned_fragments,
        );

        let inline_start = match inline_anchor {
            Anchor::Start(start) => start + pb.inline_start + margin.inline_start,
            Anchor::End(end) => cbbs - end - pb.inline_end - margin.inline_end - inline_size,
        };

        let block_size = block_size.auto_is(|| flow_children.block_size);
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
            &absolutely_positioned_fragments,
            &mut flow_children.fragments,
            &content_rect.size,
            &padding,
            containing_block_for_children.mode,
        );

        Fragment::Box(BoxFragment {
            style: style.clone(),
            children: flow_children.fragments,
            content_rect,
            padding,
            border,
            margin,
            block_margins_collapsed_with_children: CollapsedBlockMargins::zero(),
        })
    }
}

pub(super) fn adjust_static_positions(
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
