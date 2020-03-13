/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::context::LayoutContext;
use crate::dom_traversal::{Contents, NodeExt};
use crate::formatting_contexts::IndependentFormattingContext;
use crate::fragments::{BoxFragment, CollapsedBlockMargins, Fragment};
use crate::geom::flow_relative::{Rect, Sides, Vec2};
use crate::sizing::ContentSizesRequest;
use crate::style_ext::{ComputedValuesExt, DisplayInside};
use crate::{ContainingBlock, DefiniteContainingBlock};
use rayon::iter::{IntoParallelRefIterator, ParallelExtend};
use rayon_croissant::ParallelIteratorExt;
use servo_arc::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use style::computed_values::position::T as Position;
use style::properties::ComputedValues;
use style::values::computed::{Length, LengthOrAuto, LengthPercentage, LengthPercentageOrAuto};
use style::Zero;

static HOISTED_FRAGMENT_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize)]
pub(crate) struct HoistedFragmentId(u16);

impl HoistedFragmentId {
    pub fn new() -> HoistedFragmentId {
        let new_id = HOISTED_FRAGMENT_ID_COUNTER.fetch_add(1, Ordering::SeqCst) as u16;
        HoistedFragmentId(new_id)
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct AbsolutelyPositionedBox {
    pub contents: IndependentFormattingContext,
}

pub(crate) struct PositioningContext {
    for_nearest_positioned_ancestor: Option<Vec<HoistedAbsolutelyPositionedBox>>,

    // For nearest `containing block for all descendants` as defined by the CSS transforms
    // spec.
    // https://www.w3.org/TR/css-transforms-1/#containing-block-for-all-descendants
    for_nearest_containing_block_for_all_descendants: Vec<HoistedAbsolutelyPositionedBox>,
}

#[derive(Debug)]
pub(crate) struct HoistedAbsolutelyPositionedBox {
    absolutely_positioned_box: Arc<AbsolutelyPositionedBox>,

    /// The rank of the child from which this absolutely positioned fragment
    /// came from, when doing the layout of a block container. Used to compute
    /// static positions when going up the tree.
    pub(crate) tree_rank: usize,

    box_offsets: Vec2<AbsoluteBoxOffsets>,

    /// The id which is shared between this HoistedAbsolutelyPositionedBox and its
    /// placeholder AbsoluteOrFixedPositionedFragment in its original tree position.
    /// This will be used later in order to paint this hoisted box in tree order.
    pub fragment_id: HoistedFragmentId,
}

#[derive(Clone, Debug)]
pub(crate) enum AbsoluteBoxOffsets {
    StaticStart {
        start: Length,
    },
    Start {
        start: LengthPercentage,
    },
    End {
        end: LengthPercentage,
    },
    Both {
        start: LengthPercentage,
        end: LengthPercentage,
    },
}

impl AbsolutelyPositionedBox {
    pub fn construct<'dom>(
        context: &LayoutContext,
        node: impl NodeExt<'dom>,
        style: Arc<ComputedValues>,
        display_inside: DisplayInside,
        contents: Contents,
    ) -> Self {
        // "Shrink-to-fit" in https://drafts.csswg.org/css2/visudet.html#abs-non-replaced-width
        let content_sizes = ContentSizesRequest::inline_if(
            // If inline-size is non-auto, that value is used without shrink-to-fit
            !style.inline_size_is_length() &&
            // If it is, then the only case where shrink-to-fit is *not* used is
            // if both offsets are non-auto, leaving inline-size as the only variable
            // in the constraint equation.
            !style.inline_box_offsets_are_both_non_auto(),
        );
        Self {
            contents: IndependentFormattingContext::construct(
                context,
                node,
                style,
                display_inside,
                contents,
                content_sizes,
            ),
        }
    }

    pub(crate) fn to_hoisted(
        self: Arc<Self>,
        initial_start_corner: Vec2<Length>,
        tree_rank: usize,
    ) -> HoistedAbsolutelyPositionedBox {
        fn absolute_box_offsets(
            initial_static_start: Length,
            start: LengthPercentageOrAuto,
            end: LengthPercentageOrAuto,
        ) -> AbsoluteBoxOffsets {
            match (start.non_auto(), end.non_auto()) {
                (None, None) => AbsoluteBoxOffsets::StaticStart {
                    start: initial_static_start,
                },
                (Some(start), Some(end)) => AbsoluteBoxOffsets::Both { start, end },
                (None, Some(end)) => AbsoluteBoxOffsets::End { end },
                (Some(start), None) => AbsoluteBoxOffsets::Start { start },
            }
        }

        let box_offsets = self.contents.style.box_offsets();
        HoistedAbsolutelyPositionedBox {
            absolutely_positioned_box: self,
            tree_rank,
            box_offsets: Vec2 {
                inline: absolute_box_offsets(
                    initial_start_corner.inline,
                    box_offsets.inline_start.clone(),
                    box_offsets.inline_end.clone(),
                ),
                block: absolute_box_offsets(
                    initial_start_corner.block,
                    box_offsets.block_start.clone(),
                    box_offsets.block_end.clone(),
                ),
            },
            fragment_id: HoistedFragmentId::new(),
        }
    }
}

impl PositioningContext {
    pub(crate) fn new_for_containing_block_for_all_descendants() -> Self {
        Self {
            for_nearest_positioned_ancestor: None,
            for_nearest_containing_block_for_all_descendants: Vec::new(),
        }
    }

    pub(crate) fn new_for_rayon(collects_for_nearest_positioned_ancestor: bool) -> Self {
        Self {
            for_nearest_positioned_ancestor: if collects_for_nearest_positioned_ancestor {
                Some(Vec::new())
            } else {
                None
            },
            for_nearest_containing_block_for_all_descendants: Vec::new(),
        }
    }

    pub(crate) fn collects_for_nearest_positioned_ancestor(&self) -> bool {
        self.for_nearest_positioned_ancestor.is_some()
    }

    /// Given `fragment_layout_fn`, a closure which lays out a fragment in a provided
    /// `PositioningContext`, create a new positioning context if necessary for the fragment and
    /// lay out the fragment and all its children. Returns the newly created `BoxFragment`.
    pub(crate) fn layout_maybe_position_relative_fragment(
        &mut self,
        layout_context: &LayoutContext,
        containing_block: &ContainingBlock,
        style: &ComputedValues,
        fragment_layout_fn: impl FnOnce(&mut Self) -> BoxFragment,
    ) -> BoxFragment {
        debug_assert!(style.clone_position() != Position::Fixed);
        debug_assert!(style.clone_position() != Position::Absolute);

        if style.establishes_containing_block_for_all_descendants() {
            let mut fragment = Self::layout_containing_block_for_all_descendants(
                layout_context,
                fragment_layout_fn,
            );
            if style.clone_position() == Position::Relative {
                fragment.content_rect.start_corner +=
                    &relative_adjustement(style, containing_block);
            }
            return fragment;
        }

        if style.clone_position() == Position::Relative {
            let mut fragment = Self::create_and_layout_positioned(
                layout_context,
                style,
                &mut self.for_nearest_containing_block_for_all_descendants,
                fragment_layout_fn,
            );
            fragment.content_rect.start_corner += &relative_adjustement(style, containing_block);
            return fragment;
        }

        // We don't need to create a new PositioningContext for this Fragment, so
        // we pass in the current one to the fragment layout closure.
        fragment_layout_fn(self)
    }

    /// Given `fragment_layout_fn`, a closure which lays out a fragment in a provided
    /// `PositioningContext`, create a positioning context a positioned fragment and lay out the
    /// fragment and all its children. Returns the resulting `BoxFragment`.
    fn create_and_layout_positioned(
        layout_context: &LayoutContext,
        style: &ComputedValues,
        for_nearest_containing_block_for_all_descendants: &mut Vec<HoistedAbsolutelyPositionedBox>,
        fragment_layout_fn: impl FnOnce(&mut Self) -> BoxFragment,
    ) -> BoxFragment {
        if style.establishes_containing_block_for_all_descendants() {
            return Self::layout_containing_block_for_all_descendants(
                layout_context,
                fragment_layout_fn,
            );
        }

        let mut new = Self {
            for_nearest_positioned_ancestor: Some(Vec::new()),
            for_nearest_containing_block_for_all_descendants: std::mem::take(
                for_nearest_containing_block_for_all_descendants,
            ),
        };
        let mut positioned_box_fragment = fragment_layout_fn(&mut new);
        new.layout_positioned_fragment_children(layout_context, &mut positioned_box_fragment);
        *for_nearest_containing_block_for_all_descendants =
            new.for_nearest_containing_block_for_all_descendants;
        positioned_box_fragment
    }

    /// Given `fragment_layout_fn`, a closure which lays out a fragment in a provided
    /// `PositioningContext`, create a positioning context for a fragment that establishes a
    /// containing block for all descendants and lay out the fragment and all its children using
    /// the new positioning context. Returns the resulting `BoxFragment`.
    fn layout_containing_block_for_all_descendants(
        layout_context: &LayoutContext,
        fragment_layout_fn: impl FnOnce(&mut Self) -> BoxFragment,
    ) -> BoxFragment {
        let mut containing_block_for_all_descendants =
            Self::new_for_containing_block_for_all_descendants();
        debug_assert!(containing_block_for_all_descendants
            .for_nearest_positioned_ancestor
            .is_none());

        let mut new_fragment = fragment_layout_fn(&mut containing_block_for_all_descendants);

        let padding_rect = Rect {
            size: new_fragment.content_rect.size.clone(),
            // Ignore the content rect’s position in its own containing block:
            start_corner: Vec2::zero(),
        }
        .inflate(&new_fragment.padding);
        let containing_block = DefiniteContainingBlock {
            size: padding_rect.size.clone(),
            style: &new_fragment.style,
        };

        // Loop because it’s possible that we discover (the static position of)
        // more absolutely-positioned boxes while doing layout for others.
        let mut new_child_fragments = Vec::new();
        while !containing_block_for_all_descendants
            .for_nearest_containing_block_for_all_descendants
            .is_empty()
        {
            HoistedAbsolutelyPositionedBox::layout_many(
                layout_context,
                &std::mem::take(
                    &mut containing_block_for_all_descendants
                        .for_nearest_containing_block_for_all_descendants,
                ),
                &mut new_child_fragments,
                &mut containing_block_for_all_descendants
                    .for_nearest_containing_block_for_all_descendants,
                &containing_block,
            )
        }

        new_fragment.children.extend(new_child_fragments);
        new_fragment
    }

    pub(crate) fn push(&mut self, box_: HoistedAbsolutelyPositionedBox) {
        if let Some(nearest) = &mut self.for_nearest_positioned_ancestor {
            match box_
                .absolutely_positioned_box
                .contents
                .style
                .clone_position()
            {
                Position::Fixed => {}, // fall through
                Position::Absolute => return nearest.push(box_),
                Position::Static | Position::Relative => unreachable!(),
            }
        }
        self.for_nearest_containing_block_for_all_descendants
            .push(box_)
    }

    pub(crate) fn append(&mut self, other: Self) {
        vec_append_owned(
            &mut self.for_nearest_containing_block_for_all_descendants,
            other.for_nearest_containing_block_for_all_descendants,
        );
        match (
            self.for_nearest_positioned_ancestor.as_mut(),
            other.for_nearest_positioned_ancestor,
        ) {
            (Some(a), Some(b)) => vec_append_owned(a, b),
            (None, None) => {},
            _ => unreachable!(),
        }
    }

    pub(crate) fn adjust_static_positions(
        &mut self,
        tree_rank_in_parent: usize,
        f: impl FnOnce(&mut Self) -> Vec<Fragment>,
    ) -> Vec<Fragment> {
        let for_containing_block_for_all_descendants =
            self.for_nearest_containing_block_for_all_descendants.len();
        let for_nearest_so_far = self
            .for_nearest_positioned_ancestor
            .as_ref()
            .map(|v| v.len());

        let fragments = f(self);

        adjust_static_positions(
            &mut self.for_nearest_containing_block_for_all_descendants
                [for_containing_block_for_all_descendants..],
            &fragments,
            tree_rank_in_parent,
        );
        if let Some(nearest) = &mut self.for_nearest_positioned_ancestor {
            adjust_static_positions(
                &mut nearest[for_nearest_so_far.unwrap()..],
                &fragments,
                tree_rank_in_parent,
            );
        }
        fragments
    }

    pub(crate) fn layout_initial_containing_block_children(
        &mut self,
        layout_context: &LayoutContext,
        initial_containing_block: &DefiniteContainingBlock,
        fragments: &mut Vec<Fragment>,
    ) {
        debug_assert!(self.for_nearest_positioned_ancestor.is_none());

        // Loop because it’s possible that we discover (the static position of)
        // more absolutely-positioned boxes while doing layout for others.
        while !self
            .for_nearest_containing_block_for_all_descendants
            .is_empty()
        {
            HoistedAbsolutelyPositionedBox::layout_many(
                layout_context,
                &std::mem::take(&mut self.for_nearest_containing_block_for_all_descendants),
                fragments,
                &mut self.for_nearest_containing_block_for_all_descendants,
                initial_containing_block,
            )
        }
    }

    fn layout_positioned_fragment_children(
        &mut self,
        layout_context: &LayoutContext,
        positioned_box_fragment: &mut BoxFragment,
    ) {
        let for_here = self.for_nearest_positioned_ancestor.take().unwrap();
        if !for_here.is_empty() {
            let padding_rect = Rect {
                size: positioned_box_fragment.content_rect.size.clone(),
                // Ignore the content rect’s position in its own containing block:
                start_corner: Vec2::zero(),
            }
            .inflate(&positioned_box_fragment.padding);
            let containing_block = DefiniteContainingBlock {
                size: padding_rect.size.clone(),
                style: &positioned_box_fragment.style,
            };
            let mut children = Vec::new();
            HoistedAbsolutelyPositionedBox::layout_many(
                layout_context,
                &for_here,
                &mut children,
                &mut self.for_nearest_containing_block_for_all_descendants,
                &containing_block,
            );
            positioned_box_fragment.children.extend(children);
        }
    }
}

impl HoistedAbsolutelyPositionedBox {
    pub(crate) fn layout_many(
        layout_context: &LayoutContext,
        boxes: &[Self],
        fragments: &mut Vec<Fragment>,
        for_nearest_containing_block_for_all_descendants: &mut Vec<HoistedAbsolutelyPositionedBox>,
        containing_block: &DefiniteContainingBlock,
    ) {
        if layout_context.use_rayon {
            fragments.par_extend(boxes.par_iter().mapfold_reduce_into(
                for_nearest_containing_block_for_all_descendants,
                |for_nearest_containing_block_for_all_descendants, box_| {
                    Fragment::Box(box_.layout(
                        layout_context,
                        for_nearest_containing_block_for_all_descendants,
                        containing_block,
                    ))
                },
                Vec::new,
                vec_append_owned,
            ))
        } else {
            fragments.extend(boxes.iter().map(|box_| {
                Fragment::Box(box_.layout(
                    layout_context,
                    for_nearest_containing_block_for_all_descendants,
                    containing_block,
                ))
            }))
        }
    }

    pub(crate) fn layout(
        &self,
        layout_context: &LayoutContext,
        for_nearest_containing_block_for_all_descendants: &mut Vec<HoistedAbsolutelyPositionedBox>,
        containing_block: &DefiniteContainingBlock,
    ) -> BoxFragment {
        let style = &self.absolutely_positioned_box.contents.style;
        let cbis = containing_block.size.inline;
        let cbbs = containing_block.size.block;

        let size;
        let replaced_used_size;
        match self.absolutely_positioned_box.contents.as_replaced() {
            Ok(replaced) => {
                // https://drafts.csswg.org/css2/visudet.html#abs-replaced-width
                // https://drafts.csswg.org/css2/visudet.html#abs-replaced-height
                let u = replaced.used_size_as_if_inline_element(&containing_block.into(), style);
                size = Vec2 {
                    inline: LengthOrAuto::LengthPercentage(u.inline),
                    block: LengthOrAuto::LengthPercentage(u.block),
                };
                replaced_used_size = Some(u);
            },
            Err(_non_replaced) => {
                let box_size = style.box_size();
                size = Vec2 {
                    inline: box_size.inline.percentage_relative_to(cbis),
                    block: box_size.block.percentage_relative_to(cbbs),
                };
                replaced_used_size = None;
            },
        }

        let padding = style.padding().percentages_relative_to(cbis);
        let border = style.border_width();
        let computed_margin = style.margin().percentages_relative_to(cbis);
        let pb = &padding + &border;

        let inline_axis = solve_axis(
            cbis,
            pb.inline_sum(),
            computed_margin.inline_start.clone(),
            computed_margin.inline_end.clone(),
            /* avoid_negative_margin_start */ true,
            self.box_offsets.inline.clone(),
            size.inline,
        );

        let block_axis = solve_axis(
            cbis,
            pb.block_sum(),
            computed_margin.block_start.clone(),
            computed_margin.block_end.clone(),
            /* avoid_negative_margin_start */ false,
            self.box_offsets.block.clone(),
            size.block,
        );

        let margin = Sides {
            inline_start: inline_axis.margin_start,
            inline_end: inline_axis.margin_end,
            block_start: block_axis.margin_start,
            block_end: block_axis.margin_end,
        };

        let for_containing_block_for_all_descendants =
            for_nearest_containing_block_for_all_descendants;
        PositioningContext::create_and_layout_positioned(
            layout_context,
            style,
            for_containing_block_for_all_descendants,
            |positioning_context| {
                let size;
                let fragments;
                match self.absolutely_positioned_box.contents.as_replaced() {
                    Ok(replaced) => {
                        // https://drafts.csswg.org/css2/visudet.html#abs-replaced-width
                        // https://drafts.csswg.org/css2/visudet.html#abs-replaced-height
                        let style = &self.absolutely_positioned_box.contents.style;
                        size = replaced_used_size.unwrap();
                        fragments = replaced.make_fragments(style, size.clone());
                    },
                    Err(non_replaced) => {
                        // https://drafts.csswg.org/css2/visudet.html#abs-non-replaced-width
                        // https://drafts.csswg.org/css2/visudet.html#abs-non-replaced-height
                        let inline_size = inline_axis.size.auto_is(|| {
                            let available_size = match inline_axis.anchor {
                                Anchor::Start(start) => {
                                    cbis - start - pb.inline_sum() - margin.inline_sum()
                                },
                                Anchor::End(end) => {
                                    cbis - end - pb.inline_sum() - margin.inline_sum()
                                },
                            };
                            self.absolutely_positioned_box
                                .contents
                                .content_sizes
                                .shrink_to_fit(available_size)
                        });

                        let containing_block_for_children = ContainingBlock {
                            inline_size,
                            block_size: block_axis.size,
                            style,
                        };
                        // https://drafts.csswg.org/css-writing-modes/#orthogonal-flows
                        assert_eq!(
                            containing_block.style.writing_mode,
                            containing_block_for_children.style.writing_mode,
                            "Mixed writing modes are not supported yet"
                        );
                        let dummy_tree_rank = 0;
                        let independent_layout = non_replaced.layout(
                            layout_context,
                            positioning_context,
                            &containing_block_for_children,
                            dummy_tree_rank,
                        );

                        size = Vec2 {
                            inline: inline_size,
                            block: block_axis
                                .size
                                .auto_is(|| independent_layout.content_block_size),
                        };
                        fragments = independent_layout.fragments
                    },
                };

                let inline_start = match inline_axis.anchor {
                    Anchor::Start(start) => start + pb.inline_start + margin.inline_start,
                    Anchor::End(end) => {
                        cbis - end - pb.inline_end - margin.inline_end - size.inline
                    },
                };
                let block_start = match block_axis.anchor {
                    Anchor::Start(start) => start + pb.block_start + margin.block_start,
                    Anchor::End(end) => cbbs - end - pb.block_end - margin.block_end - size.block,
                };

                let content_rect = Rect {
                    start_corner: Vec2 {
                        inline: inline_start,
                        block: block_start,
                    },
                    size,
                };

                BoxFragment::new(
                    self.absolutely_positioned_box.contents.tag,
                    style.clone(),
                    fragments,
                    content_rect,
                    padding,
                    border,
                    margin,
                    CollapsedBlockMargins::zero(),
                    Some(self.fragment_id),
                )
            },
        )
    }
}

enum Anchor {
    Start(Length),
    End(Length),
}

struct AxisResult {
    anchor: Anchor,
    size: LengthOrAuto,
    margin_start: Length,
    margin_end: Length,
}

/// This unifies some of the parts in common in:
///
/// * https://drafts.csswg.org/css2/visudet.html#abs-non-replaced-width
/// * https://drafts.csswg.org/css2/visudet.html#abs-non-replaced-height
///
/// … and:
///
/// * https://drafts.csswg.org/css2/visudet.html#abs-replaced-width
/// * https://drafts.csswg.org/css2/visudet.html#abs-replaced-height
///
/// In the replaced case, `size` is never `Auto`.
fn solve_axis(
    containing_size: Length,
    padding_border_sum: Length,
    computed_margin_start: LengthOrAuto,
    computed_margin_end: LengthOrAuto,
    avoid_negative_margin_start: bool,
    box_offsets: AbsoluteBoxOffsets,
    size: LengthOrAuto,
) -> AxisResult {
    match box_offsets {
        AbsoluteBoxOffsets::StaticStart { start } => AxisResult {
            anchor: Anchor::Start(start),
            size,
            margin_start: computed_margin_start.auto_is(Length::zero),
            margin_end: computed_margin_end.auto_is(Length::zero),
        },
        AbsoluteBoxOffsets::Start { start } => AxisResult {
            anchor: Anchor::Start(start.percentage_relative_to(containing_size)),
            size,
            margin_start: computed_margin_start.auto_is(Length::zero),
            margin_end: computed_margin_end.auto_is(Length::zero),
        },
        AbsoluteBoxOffsets::End { end } => AxisResult {
            anchor: Anchor::End(end.percentage_relative_to(containing_size)),
            size,
            margin_start: computed_margin_start.auto_is(Length::zero),
            margin_end: computed_margin_end.auto_is(Length::zero),
        },
        AbsoluteBoxOffsets::Both { start, end } => {
            let start = start.percentage_relative_to(containing_size);
            let end = end.percentage_relative_to(containing_size);

            let margin_start;
            let margin_end;
            let used_size;
            if let LengthOrAuto::LengthPercentage(s) = size {
                used_size = s;
                let margins = containing_size - start - end - padding_border_sum - s;
                match (computed_margin_start, computed_margin_end) {
                    (LengthOrAuto::Auto, LengthOrAuto::Auto) => {
                        if avoid_negative_margin_start && margins < Length::zero() {
                            margin_start = Length::zero();
                            margin_end = margins;
                        } else {
                            margin_start = margins / 2.;
                            margin_end = margins / 2.;
                        }
                    },
                    (LengthOrAuto::Auto, LengthOrAuto::LengthPercentage(end)) => {
                        margin_start = margins - end;
                        margin_end = end;
                    },
                    (LengthOrAuto::LengthPercentage(start), LengthOrAuto::Auto) => {
                        margin_start = start;
                        margin_end = margins - start;
                    },
                    (
                        LengthOrAuto::LengthPercentage(start),
                        LengthOrAuto::LengthPercentage(end),
                    ) => {
                        margin_start = start;
                        margin_end = end;
                    },
                }
            } else {
                margin_start = computed_margin_start.auto_is(Length::zero);
                margin_end = computed_margin_end.auto_is(Length::zero);
                // FIXME(nox): What happens if that is negative?
                used_size =
                    containing_size - start - end - padding_border_sum - margin_start - margin_end
            };
            AxisResult {
                anchor: Anchor::Start(start),
                size: LengthOrAuto::LengthPercentage(used_size),
                margin_start,
                margin_end,
            }
        },
    }
}

fn adjust_static_positions(
    absolutely_positioned_fragments: &mut [HoistedAbsolutelyPositionedBox],
    child_fragments: &[Fragment],
    tree_rank_in_parent: usize,
) {
    for abspos_fragment in absolutely_positioned_fragments {
        let original_tree_rank = abspos_fragment.tree_rank;
        abspos_fragment.tree_rank = tree_rank_in_parent;

        let child_fragment_rect = match &child_fragments[original_tree_rank] {
            Fragment::Box(b) => &b.content_rect,
            Fragment::AbsoluteOrFixedPositioned(_) => continue,
            Fragment::Anonymous(a) => &a.rect,
            _ => unreachable!(),
        };

        if let AbsoluteBoxOffsets::StaticStart { start } = &mut abspos_fragment.box_offsets.inline {
            *start += child_fragment_rect.start_corner.inline;
        }

        if let AbsoluteBoxOffsets::StaticStart { start } = &mut abspos_fragment.box_offsets.block {
            *start += child_fragment_rect.start_corner.block;
        }
    }
}

fn vec_append_owned<T>(a: &mut Vec<T>, mut b: Vec<T>) {
    if a.is_empty() {
        *a = b
    } else {
        a.append(&mut b)
    }
}

/// https://drafts.csswg.org/css2/visuren.html#relative-positioning
pub(crate) fn relative_adjustement(
    style: &ComputedValues,
    containing_block: &ContainingBlock,
) -> Vec2<Length> {
    let cbis = containing_block.inline_size;
    let cbbs = containing_block.block_size.auto_is(Length::zero);
    let box_offsets = style.box_offsets().map_inline_and_block_axes(
        |v| v.percentage_relative_to(cbis),
        |v| v.percentage_relative_to(cbbs),
    );
    fn adjust(start: LengthOrAuto, end: LengthOrAuto) -> Length {
        match (start, end) {
            (LengthOrAuto::Auto, LengthOrAuto::Auto) => Length::zero(),
            (LengthOrAuto::Auto, LengthOrAuto::LengthPercentage(end)) => -end,
            (LengthOrAuto::LengthPercentage(start), _) => start,
        }
    }
    Vec2 {
        inline: adjust(box_offsets.inline_start, box_offsets.inline_end),
        block: adjust(box_offsets.block_start, box_offsets.block_end),
    }
}
