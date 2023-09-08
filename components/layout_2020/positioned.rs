/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use rayon::iter::IntoParallelRefMutIterator;
use rayon::prelude::{IndexedParallelIterator, ParallelIterator};
use serde::Serialize;
use style::computed_values::position::T as Position;
use style::properties::ComputedValues;
use style::values::computed::{CSSPixelLength, Length};
use style::values::specified::text::TextDecorationLine;
use style::Zero;

use crate::cell::ArcRefCell;
use crate::context::LayoutContext;
use crate::dom::NodeExt;
use crate::dom_traversal::{Contents, NodeAndStyleInfo};
use crate::formatting_contexts::IndependentFormattingContext;
use crate::fragment_tree::{
    AbsoluteBoxOffsets, BoxFragment, CollapsedBlockMargins, Fragment, HoistedSharedFragment,
};
use crate::geom::{LengthOrAuto, LengthPercentageOrAuto, LogicalRect, LogicalSides, LogicalVec2};
use crate::style_ext::{ComputedValuesExt, DisplayInside};
use crate::{ContainingBlock, DefiniteContainingBlock};

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

    /// A reference to a Fragment which is shared between this `HoistedAbsolutelyPositionedBox`
    /// and its placeholder `AbsoluteOrFixedPositionedFragment` in the original tree position.
    /// This will be used later in order to paint this hoisted box in tree order.
    pub fragment: ArcRefCell<HoistedSharedFragment>,
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
        initial_start_corner: LogicalVec2<Length>,
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
            LogicalVec2 {
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

    /// Create a [PositioninContext] to use for laying out a subtree. The idea is that
    /// when subtree layout is finished, the newly hoisted boxes can be processed
    /// (normally adjusting their static insets) and then appended to the parent
    /// [PositioningContext].
    pub(crate) fn new_for_subtree(collects_for_nearest_positioned_ancestor: bool) -> Self {
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
        } else if style.establishes_containing_block_for_absolute_descendants() {
            Some(Self {
                for_nearest_positioned_ancestor: Some(Vec::new()),
                for_nearest_containing_block_for_all_descendants: Vec::new(),
            })
        } else {
            None
        }
    }

    /// Absolute and fixed position fragments are hoisted up to their containing blocks
    /// from their tree position. When these fragments have static inset start positions,
    /// that position (relative to the ancestor containing block) needs to be included
    /// with the hoisted fragment so that it can be laid out properly at the containing
    /// block.
    ///
    /// This function is used to update the static position of hoisted boxes added after
    /// the given index at every level of the fragment tree as the hoisted fragments move
    /// up to their containing blocks. Once an ancestor fragment is laid out, this
    /// function can be used to aggregate its offset to any descendent boxes that are
    /// being hoisted. In this case, the appropriate index to use is the result of
    /// [`PositioningContext::len()`] cached before laying out the [`Fragment`].
    pub(crate) fn adjust_static_position_of_hoisted_fragments(
        &mut self,
        parent_fragment: &Fragment,
        index: PositioningContextLength,
    ) {
        let start_offset = match &parent_fragment {
            Fragment::Box(b) | Fragment::Float(b) => &b.content_rect.start_corner,
            Fragment::AbsoluteOrFixedPositioned(_) => return,
            Fragment::Anonymous(a) => &a.rect.start_corner,
            _ => unreachable!(),
        };
        self.adjust_static_position_of_hoisted_fragments_with_offset(start_offset, index);
    }

    /// See documentation for [adjust_static_position_of_hoisted_fragments].
    pub(crate) fn adjust_static_position_of_hoisted_fragments_with_offset(
        &mut self,
        start_offset: &LogicalVec2<CSSPixelLength>,
        index: PositioningContextLength,
    ) {
        let update_fragment_if_needed = |hoisted_fragment: &mut HoistedAbsolutelyPositionedBox| {
            let mut fragment = hoisted_fragment.fragment.borrow_mut();
            if let AbsoluteBoxOffsets::StaticStart { start } = &mut fragment.box_offsets.inline {
                *start += start_offset.inline;
            }
            if let AbsoluteBoxOffsets::StaticStart { start } = &mut fragment.box_offsets.block {
                *start += start_offset.block;
            }
        };

        self.for_nearest_positioned_ancestor
            .as_mut()
            .map(|hoisted_boxes| {
                hoisted_boxes
                    .iter_mut()
                    .skip(index.for_nearest_positioned_ancestor)
                    .for_each(update_fragment_if_needed);
            });
        self.for_nearest_containing_block_for_all_descendants
            .iter_mut()
            .skip(index.for_nearest_containing_block_for_all_descendants)
            .for_each(update_fragment_if_needed);
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
        // pass them up the tree.
        self.append(new_context);

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
        let padding_rect = LogicalRect {
            size: new_fragment.content_rect.size.clone(),
            // Ignore the content rect’s position in its own containing block:
            start_corner: LogicalVec2::zero(),
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
                Position::Static | Position::Relative | Position::Sticky => unreachable!(),
            }
        }
        self.for_nearest_containing_block_for_all_descendants
            .push(box_)
    }

    fn is_empty(&self) -> bool {
        self.for_nearest_containing_block_for_all_descendants
            .is_empty() &&
            self.for_nearest_positioned_ancestor
                .as_ref()
                .map_or(true, |vector| vector.is_empty())
    }

    pub(crate) fn append(&mut self, other: Self) {
        if other.is_empty() {
            return;
        }

        vec_append_owned(
            &mut self.for_nearest_containing_block_for_all_descendants,
            other.for_nearest_containing_block_for_all_descendants,
        );

        match (
            self.for_nearest_positioned_ancestor.as_mut(),
            other.for_nearest_positioned_ancestor,
        ) {
            (Some(us), Some(them)) => vec_append_owned(us, them),
            (None, Some(them)) => {
                // This is the case where we have laid out the absolute children in a containing
                // block for absolutes and we then are passing up the fixed-position descendants
                // to the containing block for all descendants.
                vec_append_owned(
                    &mut self.for_nearest_containing_block_for_all_descendants,
                    them,
                );
            },
            (None, None) => {},
            _ => unreachable!(),
        }
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

    pub(crate) fn clear(&mut self) {
        self.for_nearest_containing_block_for_all_descendants
            .clear();
        self.for_nearest_positioned_ancestor
            .as_mut()
            .map(|v| v.clear());
    }

    /// Get the length of this [PositioningContext].
    pub(crate) fn len(&self) -> PositioningContextLength {
        PositioningContextLength {
            for_nearest_positioned_ancestor: self
                .for_nearest_positioned_ancestor
                .as_ref()
                .map_or(0, |vec| vec.len()),
            for_nearest_containing_block_for_all_descendants: self
                .for_nearest_containing_block_for_all_descendants
                .len(),
        }
    }

    /// Truncate this [PositioningContext] to the given [PositioningContextLength].  This
    /// is useful for "unhoisting" boxes in this context and returning it to the state at
    /// the time that [`len()`] was called.
    pub(crate) fn truncate(&mut self, length: &PositioningContextLength) {
        if let Some(vec) = self.for_nearest_positioned_ancestor.as_mut() {
            vec.truncate(length.for_nearest_positioned_ancestor);
        }
        self.for_nearest_containing_block_for_all_descendants
            .truncate(length.for_nearest_containing_block_for_all_descendants);
    }
}

/// A data structure which stores the size of a positioning context.
#[derive(PartialEq)]
pub(crate) struct PositioningContextLength {
    /// The number of boxes that will be hoisted the the nearest positioned ancestor for
    /// layout.
    for_nearest_positioned_ancestor: usize,
    /// The number of boxes that will be hoisted the the nearest ancestor which
    /// establishes a containing block for all descendants for layout.
    for_nearest_containing_block_for_all_descendants: usize,
}

impl Zero for PositioningContextLength {
    fn zero() -> Self {
        PositioningContextLength {
            for_nearest_positioned_ancestor: 0,
            for_nearest_containing_block_for_all_descendants: 0,
        }
    }

    fn is_zero(&self) -> bool {
        self.for_nearest_positioned_ancestor == 0 &&
            self.for_nearest_containing_block_for_all_descendants == 0
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
            let mut new_fragments = Vec::new();
            let mut new_hoisted_boxes = Vec::new();

            boxes
                .par_iter_mut()
                .map(|hoisted_box| {
                    let mut new_hoisted_boxes: Vec<HoistedAbsolutelyPositionedBox> = Vec::new();
                    let new_fragment = ArcRefCell::new(Fragment::Box(hoisted_box.layout(
                        layout_context,
                        &mut new_hoisted_boxes,
                        containing_block,
                    )));

                    hoisted_box.fragment.borrow_mut().fragment = Some(new_fragment.clone());
                    (new_fragment, new_hoisted_boxes)
                })
                .unzip_into_vecs(&mut new_fragments, &mut new_hoisted_boxes);

            fragments.extend(new_fragments);
            for_nearest_containing_block_for_all_descendants
                .extend(new_hoisted_boxes.into_iter().flatten());
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

        let computed_size = match &absolutely_positioned_box.context {
            IndependentFormattingContext::Replaced(replaced) => {
                // https://drafts.csswg.org/css2/visudet.html#abs-replaced-width
                // https://drafts.csswg.org/css2/visudet.html#abs-replaced-height
                let used_size = replaced.contents.used_size_as_if_inline_element(
                    &containing_block.into(),
                    &replaced.style,
                    None,
                    &pbm,
                );
                LogicalVec2 {
                    inline: LengthOrAuto::LengthPercentage(used_size.inline),
                    block: LengthOrAuto::LengthPercentage(used_size.block),
                }
            },
            IndependentFormattingContext::NonReplaced(non_replaced) => non_replaced
                .style
                .content_box_size(&containing_block.into(), &pbm),
        };

        let shared_fragment = self.fragment.borrow();
        let inline_axis_solver = AbsoluteAxisSolver {
            containing_size: cbis,
            padding_border_sum: pbm.padding_border_sums.inline,
            computed_margin_start: pbm.margin.inline_start,
            computed_margin_end: pbm.margin.inline_end,
            avoid_negative_margin_start: true,
            box_offsets: &shared_fragment.box_offsets.inline,
        };

        let block_axis_solver = AbsoluteAxisSolver {
            containing_size: cbbs,
            padding_border_sum: pbm.padding_border_sums.block,
            computed_margin_start: pbm.margin.block_start,
            computed_margin_end: pbm.margin.block_end,
            avoid_negative_margin_start: false,
            box_offsets: &shared_fragment.box_offsets.block,
        };
        let overconstrained = LogicalVec2 {
            inline: inline_axis_solver.is_overconstrained_for_size(computed_size.inline),
            block: block_axis_solver.is_overconstrained_for_size(computed_size.block),
        };

        let mut inline_axis = inline_axis_solver.solve_for_size(computed_size.inline);
        let mut block_axis = block_axis_solver.solve_for_size(computed_size.block);

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
                    content_size = computed_size.auto_is(|| unreachable!());
                    fragments = replaced
                        .contents
                        .make_fragments(style, content_size.clone());
                },
                IndependentFormattingContext::NonReplaced(non_replaced) => {
                    // https://drafts.csswg.org/css2/#min-max-widths
                    // https://drafts.csswg.org/css2/#min-max-heights
                    let min_size = non_replaced
                        .style
                        .content_min_box_size(&containing_block.into(), &pbm)
                        .auto_is(|| Length::zero());
                    let max_size = non_replaced
                        .style
                        .content_max_box_size(&containing_block.into(), &pbm);

                    // https://drafts.csswg.org/css2/visudet.html#abs-non-replaced-width
                    // https://drafts.csswg.org/css2/visudet.html#abs-non-replaced-height
                    let mut inline_size = inline_axis.size.auto_is(|| {
                        let anchor = match inline_axis.anchor {
                            Anchor::Start(start) => start,
                            Anchor::End(end) => end,
                        };
                        let margin_sum = inline_axis.margin_start + inline_axis.margin_end;
                        let available_size =
                            cbis - anchor - pbm.padding_border_sums.inline - margin_sum;
                        non_replaced
                            .inline_content_sizes(layout_context)
                            .shrink_to_fit(available_size)
                    });

                    // If the tentative used inline size is greater than ‘max-inline-size’,
                    // recalculate the inline size and margins with ‘max-inline-size’ as the
                    // computed ‘inline-size’. We can assume the new inline size won’t be ‘auto’,
                    // because a non-‘auto’ computed ‘inline-size’ always becomes the used value.
                    // https://drafts.csswg.org/css2/#min-max-widths (step 2)
                    if let Some(max) = max_size.inline {
                        if inline_size > max {
                            inline_axis = inline_axis_solver
                                .solve_for_size(LengthOrAuto::LengthPercentage(max));
                            inline_size = inline_axis.size.auto_is(|| unreachable!());
                        }
                    }

                    // If the tentative used inline size is less than ‘min-inline-size’,
                    // recalculate the inline size and margins with ‘min-inline-size’ as the
                    // computed ‘inline-size’. We can assume the new inline size won’t be ‘auto’,
                    // because a non-‘auto’ computed ‘inline-size’ always becomes the used value.
                    // https://drafts.csswg.org/css2/#min-max-widths (step 3)
                    if inline_size < min_size.inline {
                        inline_axis = inline_axis_solver
                            .solve_for_size(LengthOrAuto::LengthPercentage(min_size.inline));
                        inline_size = inline_axis.size.auto_is(|| unreachable!());
                    }

                    struct Result {
                        content_size: LogicalVec2<CSSPixelLength>,
                        fragments: Vec<Fragment>,
                    }

                    // If we end up recalculating the block size and margins below, we also need
                    // to relayout the children with a containing block of that size, otherwise
                    // percentages may be resolved incorrectly.
                    let mut try_layout = |size| {
                        let containing_block_for_children = ContainingBlock {
                            inline_size,
                            block_size: size,
                            style: &non_replaced.style,
                        };
                        // https://drafts.csswg.org/css-writing-modes/#orthogonal-flows
                        assert_eq!(
                            containing_block.style.writing_mode,
                            containing_block_for_children.style.writing_mode,
                            "Mixed writing modes are not supported yet"
                        );

                        // Clear the context since we will lay out the same descendants
                        // more than once. Otherwise, absolute descendants will create
                        // multiple fragments which could later lead to double-borrow
                        // errors.
                        positioning_context.clear();

                        let independent_layout = non_replaced.layout(
                            layout_context,
                            &mut positioning_context,
                            &containing_block_for_children,
                        );
                        let block_size = size.auto_is(|| independent_layout.content_block_size);
                        Result {
                            content_size: LogicalVec2 {
                                inline: inline_size,
                                block: block_size,
                            },
                            fragments: independent_layout.fragments,
                        }
                    };

                    let mut result = try_layout(block_axis.size);

                    // If the tentative used block size is greater than ‘max-block-size’,
                    // recalculate the block size and margins with ‘max-block-size’ as the
                    // computed ‘block-size’. We can assume the new block size won’t be ‘auto’,
                    // because a non-‘auto’ computed ‘block-size’ always becomes the used value.
                    // https://drafts.csswg.org/css2/#min-max-heights (step 2)
                    if let Some(max) = max_size.block {
                        if result.content_size.block > max {
                            block_axis = block_axis_solver
                                .solve_for_size(LengthOrAuto::LengthPercentage(max));
                            result = try_layout(LengthOrAuto::LengthPercentage(max));
                        }
                    }

                    // If the tentative used block size is less than ‘min-block-size’,
                    // recalculate the block size and margins with ‘min-block-size’ as the
                    // computed ‘block-size’. We can assume the new block size won’t be ‘auto’,
                    // because a non-‘auto’ computed ‘block-size’ always becomes the used value.
                    // https://drafts.csswg.org/css2/#min-max-heights (step 3)
                    if result.content_size.block < min_size.block {
                        block_axis = block_axis_solver
                            .solve_for_size(LengthOrAuto::LengthPercentage(min_size.block));
                        result = try_layout(LengthOrAuto::LengthPercentage(min_size.block));
                    }

                    content_size = result.content_size;
                    fragments = result.fragments;
                },
            };

            let margin = LogicalSides {
                inline_start: inline_axis.margin_start,
                inline_end: inline_axis.margin_end,
                block_start: block_axis.margin_start,
                block_end: block_axis.margin_end,
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

            let content_rect = LogicalRect {
                start_corner: LogicalVec2 {
                    inline: inline_start,
                    block: block_start,
                },
                size: content_size,
            };

            let physical_overconstrained =
                overconstrained.to_physical(containing_block.style.writing_mode);

            BoxFragment::new_with_overconstrained(
                absolutely_positioned_box.context.base_fragment_info(),
                absolutely_positioned_box.context.style().clone(),
                fragments,
                content_rect,
                pbm.padding,
                pbm.border,
                margin,
                None,
                CollapsedBlockMargins::zero(),
                physical_overconstrained,
            )
        };
        positioning_context.layout_collected_children(layout_context, &mut new_fragment);

        // Any hoisted boxes that remain in this positioning context are going to be hoisted
        // up above this absolutely positioned box. These will necessarily be fixed position
        // elements, because absolutely positioned elements form containing blocks for all
        // other elements. If any of them have a static start position though, we need to
        // adjust it to account for the start corner of this absolute.
        positioning_context.adjust_static_position_of_hoisted_fragments_with_offset(
            &new_fragment.content_rect.start_corner,
            PositioningContextLength::zero(),
        );

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

struct AbsoluteAxisSolver<'a> {
    containing_size: Length,
    padding_border_sum: Length,
    computed_margin_start: LengthOrAuto,
    computed_margin_end: LengthOrAuto,
    avoid_negative_margin_start: bool,
    box_offsets: &'a AbsoluteBoxOffsets,
}

impl<'a> AbsoluteAxisSolver<'a> {
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
    fn solve_for_size(&self, computed_size: LengthOrAuto) -> AxisResult {
        match self.box_offsets {
            AbsoluteBoxOffsets::StaticStart { start } => AxisResult {
                anchor: Anchor::Start(*start),
                size: computed_size,
                margin_start: self.computed_margin_start.auto_is(Length::zero),
                margin_end: self.computed_margin_end.auto_is(Length::zero),
            },
            AbsoluteBoxOffsets::Start { start } => AxisResult {
                anchor: Anchor::Start(start.percentage_relative_to(self.containing_size)),
                size: computed_size,
                margin_start: self.computed_margin_start.auto_is(Length::zero),
                margin_end: self.computed_margin_end.auto_is(Length::zero),
            },
            AbsoluteBoxOffsets::End { end } => AxisResult {
                anchor: Anchor::End(end.percentage_relative_to(self.containing_size)),
                size: computed_size,
                margin_start: self.computed_margin_start.auto_is(Length::zero),
                margin_end: self.computed_margin_end.auto_is(Length::zero),
            },
            AbsoluteBoxOffsets::Both { start, end } => {
                let start = start.percentage_relative_to(self.containing_size);
                let end = end.percentage_relative_to(self.containing_size);

                let margin_start;
                let margin_end;
                let used_size;
                if let LengthOrAuto::LengthPercentage(s) = computed_size {
                    used_size = s;
                    let margins = self.containing_size - start - end - self.padding_border_sum - s;
                    match (self.computed_margin_start, self.computed_margin_end) {
                        (LengthOrAuto::Auto, LengthOrAuto::Auto) => {
                            if self.avoid_negative_margin_start && margins < Length::zero() {
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
                    margin_start = self.computed_margin_start.auto_is(Length::zero);
                    margin_end = self.computed_margin_end.auto_is(Length::zero);

                    // This may be negative, but the caller will later effectively
                    // clamp it to ‘min-inline-size’ or ‘min-block-size’.
                    used_size = self.containing_size -
                        start -
                        end -
                        self.padding_border_sum -
                        margin_start -
                        margin_end;
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

    fn is_overconstrained_for_size(&self, computed_size: LengthOrAuto) -> bool {
        !computed_size.is_auto() &&
            self.box_offsets.both_specified() &&
            !self.computed_margin_start.is_auto() &&
            !self.computed_margin_end.is_auto()
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
) -> LogicalVec2<Length> {
    // "If the height of the containing block is not specified explicitly (i.e.,
    // it depends on content height), and this element is not absolutely
    // positioned, the value computes to 'auto'.""
    // https://www.w3.org/TR/CSS2/visudet.html#the-height-property
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
    LogicalVec2 {
        inline: adjust(box_offsets.inline_start, box_offsets.inline_end),
        block: adjust(box_offsets.block_start, box_offsets.block_end),
    }
}
