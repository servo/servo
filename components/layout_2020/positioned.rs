/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::cell::ArcRefCell;
use crate::context::LayoutContext;
use crate::dom_traversal::{Contents, NodeAndStyleInfo, NodeExt};
use crate::formatting_contexts::IndependentFormattingContext;
use crate::fragments::{BoxFragment, CollapsedBlockMargins, Fragment};
use crate::geom::flow_relative::{Rect, Sides, Vec2};
use crate::geom::{LengthOrAuto, LengthPercentageOrAuto};
use crate::style_ext::{ComputedValuesExt, DisplayInside};
use crate::{ContainingBlock, DefiniteContainingBlock};
use rayon::iter::{IntoParallelRefMutIterator, ParallelExtend};
use rayon_croissant::ParallelIteratorExt;
use style::computed_values::position::T as Position;
use style::properties::ComputedValues;
use style::values::computed::{Length, LengthPercentage};
use style::values::specified::text::TextDecorationLine;
use style::Zero;

#[derive(Debug, Serialize)]
pub(crate) struct AbsolutelyPositionedBox {
    pub context: IndependentFormattingContext,
}

pub(crate) struct PositioningContext {
    for_nearest_positioned_ancestor: Option<Vec<HoistedAbsolutelyPositionedBox>>,

    // For nearest `containing block for all descendants` as defined by the CSS transforms
    // spec.
    // https://www.w3.org/TR/css-transforms-1/#containing-block-for-all-descendants
    for_nearest_containing_block_for_all_descendants: Vec<HoistedAbsolutelyPositionedBox>,
}

pub(crate) struct HoistedAbsolutelyPositionedBox {
    absolutely_positioned_box: ArcRefCell<AbsolutelyPositionedBox>,

    /// The rank of the child from which this absolutely positioned fragment
    /// came from, when doing the layout of a block container. Used to compute
    /// static positions when going up the tree.
    pub(crate) tree_rank: usize,

    /// A reference to a Fragment which is shared between this `HoistedAbsolutelyPositionedBox`
    /// and its placeholder `AbsoluteOrFixedPositionedFragment` in the original tree position.
    /// This will be used later in order to paint this hoisted box in tree order.
    pub fragment: ArcRefCell<HoistedSharedFragment>,
}

/// A reference to a Fragment which is shared between `HoistedAbsolutelyPositionedBox`
/// and its placeholder `AbsoluteOrFixedPositionedFragment` in the original tree position.
/// This will be used later in order to paint this hoisted box in tree order.
#[derive(Serialize)]
pub(crate) struct HoistedSharedFragment {
    pub(crate) fragment: Option<ArcRefCell<Fragment>>,
    pub(crate) box_offsets: Vec2<AbsoluteBoxOffsets>,
}

impl HoistedSharedFragment {
    pub(crate) fn new(box_offsets: Vec2<AbsoluteBoxOffsets>) -> Self {
        HoistedSharedFragment {
            fragment: None,
            box_offsets,
        }
    }
}

impl HoistedSharedFragment {
    /// In some cases `inset: auto`-positioned elements do not know their precise
    /// position until after they're hoisted. This lets us adjust auto values
    /// after the fact.
    pub(crate) fn adjust_offsets(&mut self, offsets: Vec2<Length>) {
        self.box_offsets.inline.adjust_offset(offsets.inline);
        self.box_offsets.block.adjust_offset(offsets.block);
    }
}

#[derive(Clone, Debug, Serialize)]
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

impl AbsoluteBoxOffsets {
    fn adjust_offset(&mut self, new_offset: Length) {
        match *self {
            AbsoluteBoxOffsets::StaticStart { ref mut start } => *start = new_offset,
            _ => (),
        }
    }
}

impl AbsolutelyPositionedBox {
    pub fn construct<'dom>(
        context: &LayoutContext,
        node_info: &NodeAndStyleInfo<impl NodeExt<'dom>>,
        display_inside: DisplayInside,
        contents: Contents,
    ) -> Self {
        Self {
            context: IndependentFormattingContext::construct(
                context,
                node_info,
                display_inside,
                contents,
                // Text decorations are not propagated to any out-of-flow descendants.
                TextDecorationLine::NONE,
            ),
        }
    }

    pub(crate) fn to_hoisted(
        self_: ArcRefCell<Self>,
        initial_start_corner: Vec2<Length>,
        tree_rank: usize,
        containing_block: &ContainingBlock,
    ) -> HoistedAbsolutelyPositionedBox {
        fn absolute_box_offsets(
            initial_static_start: Length,
            start: LengthPercentageOrAuto<'_>,
            end: LengthPercentageOrAuto<'_>,
        ) -> AbsoluteBoxOffsets {
            match (start.non_auto(), end.non_auto()) {
                (None, None) => AbsoluteBoxOffsets::StaticStart {
                    start: initial_static_start,
                },
                (Some(start), Some(end)) => AbsoluteBoxOffsets::Both {
                    start: start.clone(),
                    end: end.clone(),
                },
                (None, Some(end)) => AbsoluteBoxOffsets::End { end: end.clone() },
                (Some(start), None) => AbsoluteBoxOffsets::Start {
                    start: start.clone(),
                },
            }
        }

        let box_offsets = {
            let box_ = self_.borrow();
            let box_offsets = box_.context.style().box_offsets(containing_block);
            Vec2 {
                inline: absolute_box_offsets(
                    initial_start_corner.inline,
                    box_offsets.inline_start,
                    box_offsets.inline_end,
                ),
                block: absolute_box_offsets(
                    initial_start_corner.block,
                    box_offsets.block_start,
                    box_offsets.block_end,
                ),
            }
        };
        HoistedAbsolutelyPositionedBox {
            tree_rank,
            fragment: ArcRefCell::new(HoistedSharedFragment::new(box_offsets)),
            absolutely_positioned_box: self_,
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

    pub(crate) fn new_for_style(style: &ComputedValues) -> Option<Self> {
        if style.establishes_containing_block_for_all_descendants() {
            Some(Self::new_for_containing_block_for_all_descendants())
        } else if style.establishes_containing_block() {
            Some(Self {
                for_nearest_positioned_ancestor: Some(Vec::new()),
                for_nearest_containing_block_for_all_descendants: Vec::new(),
            })
        } else {
            None
        }
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
        // Try to create a context, but if one isn't necessary, simply create the fragment
        // using the given closure and the current `PositioningContext`.
        let mut new_context = match Self::new_for_style(style) {
            Some(new_context) => new_context,
            None => return fragment_layout_fn(self),
        };

        let mut new_fragment = fragment_layout_fn(&mut new_context);
        new_context.layout_collected_children(layout_context, &mut new_fragment);

        // If the new context has any hoisted boxes for the nearest containing block for
        // all descendants than collect them and pass them up the tree.
        vec_append_owned(
            &mut self.for_nearest_containing_block_for_all_descendants,
            new_context.for_nearest_containing_block_for_all_descendants,
        );

        if style.clone_position() == Position::Relative {
            new_fragment.content_rect.start_corner +=
                &relative_adjustement(style, containing_block);
        }

        new_fragment
    }

    // Lay out the hoisted boxes collected into this `PositioningContext` and add them
    // to the given `BoxFragment`.
    pub fn layout_collected_children(
        &mut self,
        layout_context: &LayoutContext,
        new_fragment: &mut BoxFragment,
    ) {
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

        let take_hoisted_boxes_pending_layout = |context: &mut Self| match context
            .for_nearest_positioned_ancestor
            .as_mut()
        {
            Some(fragments) => std::mem::take(fragments),
            None => std::mem::take(&mut context.for_nearest_containing_block_for_all_descendants),
        };

        // Loop because it’s possible that we discover (the static position of)
        // more absolutely-positioned boxes while doing layout for others.
        let mut hoisted_boxes = take_hoisted_boxes_pending_layout(self);
        let mut laid_out_child_fragments = Vec::new();
        while !hoisted_boxes.is_empty() {
            HoistedAbsolutelyPositionedBox::layout_many(
                layout_context,
                &mut hoisted_boxes,
                &mut laid_out_child_fragments,
                &mut self.for_nearest_containing_block_for_all_descendants,
                &containing_block,
            );
            hoisted_boxes = take_hoisted_boxes_pending_layout(self);
        }

        new_fragment.children.extend(laid_out_child_fragments);
    }

    pub(crate) fn push(&mut self, box_: HoistedAbsolutelyPositionedBox) {
        if let Some(nearest) = &mut self.for_nearest_positioned_ancestor {
            let position = box_
                .absolutely_positioned_box
                .borrow()
                .context
                .style()
                .clone_position();
            match position {
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
        fragments: &mut Vec<ArcRefCell<Fragment>>,
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
                &mut std::mem::take(&mut self.for_nearest_containing_block_for_all_descendants),
                fragments,
                &mut self.for_nearest_containing_block_for_all_descendants,
                initial_containing_block,
            )
        }
    }
}

impl HoistedAbsolutelyPositionedBox {
    pub(crate) fn layout_many(
        layout_context: &LayoutContext,
        boxes: &mut [Self],
        fragments: &mut Vec<ArcRefCell<Fragment>>,
        for_nearest_containing_block_for_all_descendants: &mut Vec<HoistedAbsolutelyPositionedBox>,
        containing_block: &DefiniteContainingBlock,
    ) {
        if layout_context.use_rayon {
            fragments.par_extend(boxes.par_iter_mut().mapfold_reduce_into(
                for_nearest_containing_block_for_all_descendants,
                |for_nearest_containing_block_for_all_descendants, box_| {
                    let new_fragment = ArcRefCell::new(Fragment::Box(box_.layout(
                        layout_context,
                        for_nearest_containing_block_for_all_descendants,
                        containing_block,
                    )));

                    box_.fragment.borrow_mut().fragment = Some(new_fragment.clone());
                    new_fragment
                },
                Vec::new,
                vec_append_owned,
            ))
        } else {
            fragments.extend(boxes.iter_mut().map(|box_| {
                let new_fragment = ArcRefCell::new(Fragment::Box(box_.layout(
                    layout_context,
                    for_nearest_containing_block_for_all_descendants,
                    containing_block,
                )));
                box_.fragment.borrow_mut().fragment = Some(new_fragment.clone());
                new_fragment
            }))
        }
    }

    pub(crate) fn layout(
        &mut self,
        layout_context: &LayoutContext,
        for_nearest_containing_block_for_all_descendants: &mut Vec<HoistedAbsolutelyPositionedBox>,
        containing_block: &DefiniteContainingBlock,
    ) -> BoxFragment {
        let cbis = containing_block.size.inline;
        let cbbs = containing_block.size.block;
        let mut absolutely_positioned_box = self.absolutely_positioned_box.borrow_mut();
        let pbm = absolutely_positioned_box
            .context
            .style()
            .padding_border_margin(&containing_block.into());

        let size = match &absolutely_positioned_box.context {
            IndependentFormattingContext::Replaced(replaced) => {
                // https://drafts.csswg.org/css2/visudet.html#abs-replaced-width
                // https://drafts.csswg.org/css2/visudet.html#abs-replaced-height
                let used_size = replaced.contents.used_size_as_if_inline_element(
                    &containing_block.into(),
                    &replaced.style,
                    &pbm,
                );
                Vec2 {
                    inline: LengthOrAuto::LengthPercentage(used_size.inline),
                    block: LengthOrAuto::LengthPercentage(used_size.block),
                }
            },
            IndependentFormattingContext::NonReplaced(non_replaced) => non_replaced
                .style
                .content_box_size(&containing_block.into(), &pbm),
        };

        let shared_fragment = self.fragment.borrow();

        let inline_axis = solve_axis(
            cbis,
            pbm.padding_border_sums.inline,
            pbm.margin.inline_start,
            pbm.margin.inline_end,
            /* avoid_negative_margin_start */ true,
            &shared_fragment.box_offsets.inline,
            size.inline,
        );

        let block_axis = solve_axis(
            cbis,
            pbm.padding_border_sums.block,
            pbm.margin.block_start,
            pbm.margin.block_end,
            /* avoid_negative_margin_start */ false,
            &shared_fragment.box_offsets.block,
            size.block,
        );

        let margin = Sides {
            inline_start: inline_axis.margin_start,
            inline_end: inline_axis.margin_end,
            block_start: block_axis.margin_start,
            block_end: block_axis.margin_end,
        };

        let mut positioning_context =
            PositioningContext::new_for_style(absolutely_positioned_box.context.style()).unwrap();
        let mut new_fragment = {
            let content_size;
            let fragments;
            match &mut absolutely_positioned_box.context {
                IndependentFormattingContext::Replaced(replaced) => {
                    // https://drafts.csswg.org/css2/visudet.html#abs-replaced-width
                    // https://drafts.csswg.org/css2/visudet.html#abs-replaced-height
                    let style = &replaced.style;
                    content_size = size.auto_is(|| unreachable!());
                    fragments = replaced
                        .contents
                        .make_fragments(style, content_size.clone());
                },
                IndependentFormattingContext::NonReplaced(non_replaced) => {
                    // https://drafts.csswg.org/css2/visudet.html#abs-non-replaced-width
                    // https://drafts.csswg.org/css2/visudet.html#abs-non-replaced-height
                    let inline_size = inline_axis.size.auto_is(|| {
                        let anchor = match inline_axis.anchor {
                            Anchor::Start(start) => start,
                            Anchor::End(end) => end,
                        };
                        let available_size =
                            cbis - anchor - pbm.padding_border_sums.inline - margin.inline_sum();
                        non_replaced
                            .inline_content_sizes(layout_context)
                            .shrink_to_fit(available_size)
                    });

                    let containing_block_for_children = ContainingBlock {
                        inline_size,
                        block_size: block_axis.size,
                        style: &non_replaced.style,
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
                        &mut positioning_context,
                        &containing_block_for_children,
                        dummy_tree_rank,
                    );

                    content_size = Vec2 {
                        inline: inline_size,
                        block: block_axis
                            .size
                            .auto_is(|| independent_layout.content_block_size),
                    };
                    fragments = independent_layout.fragments
                },
            };

            let pb = &pbm.padding + &pbm.border;
            let inline_start = match inline_axis.anchor {
                Anchor::Start(start) => start + pb.inline_start + margin.inline_start,
                Anchor::End(end) => {
                    cbis - end - pb.inline_end - margin.inline_end - content_size.inline
                },
            };
            let block_start = match block_axis.anchor {
                Anchor::Start(start) => start + pb.block_start + margin.block_start,
                Anchor::End(end) => {
                    cbbs - end - pb.block_end - margin.block_end - content_size.block
                },
            };

            let content_rect = Rect {
                start_corner: Vec2 {
                    inline: inline_start,
                    block: block_start,
                },
                size: content_size,
            };

            BoxFragment::new(
                absolutely_positioned_box.context.tag(),
                absolutely_positioned_box.context.style().clone(),
                fragments,
                content_rect,
                pbm.padding,
                pbm.border,
                margin,
                Length::zero(),
                CollapsedBlockMargins::zero(),
            )
        };
        positioning_context.layout_collected_children(layout_context, &mut new_fragment);
        for_nearest_containing_block_for_all_descendants
            .extend(positioning_context.for_nearest_containing_block_for_all_descendants);
        new_fragment
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
    box_offsets: &AbsoluteBoxOffsets,
    size: LengthOrAuto,
) -> AxisResult {
    match box_offsets {
        AbsoluteBoxOffsets::StaticStart { start } => AxisResult {
            anchor: Anchor::Start(*start),
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
            Fragment::AbsoluteOrFixedPositioned(_) | Fragment::Float => continue,
            Fragment::Anonymous(a) => &a.rect,
            _ => unreachable!(),
        };

        let mut shared_fragment = abspos_fragment.fragment.borrow_mut();

        if let AbsoluteBoxOffsets::StaticStart { start } = &mut shared_fragment.box_offsets.inline {
            *start += child_fragment_rect.start_corner.inline;
        }

        if let AbsoluteBoxOffsets::StaticStart { start } = &mut shared_fragment.box_offsets.block {
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
    let box_offsets = style
        .box_offsets(containing_block)
        .map_inline_and_block_axes(
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
