/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::mem;

use app_units::Au;
use malloc_size_of_derive::MallocSizeOf;
use rayon::iter::IntoParallelRefMutIterator;
use rayon::prelude::{IndexedParallelIterator, ParallelIterator};
use style::Zero;
use style::computed_values::position::T as Position;
use style::logical_geometry::{Direction, WritingMode};
use style::properties::ComputedValues;
use style::values::specified::align::AlignFlags;

use crate::cell::ArcRefCell;
use crate::context::LayoutContext;
use crate::dom_traversal::{Contents, NodeAndStyleInfo};
use crate::formatting_contexts::{
    IndependentFormattingContext, IndependentFormattingContextContents,
};
use crate::fragment_tree::{
    BoxFragment, Fragment, FragmentFlags, HoistedSharedFragment, SpecificLayoutInfo,
};
use crate::geom::{
    AuOrAuto, LengthPercentageOrAuto, LogicalRect, LogicalSides, LogicalSides1D, LogicalVec2,
    PhysicalPoint, PhysicalRect, PhysicalSides, PhysicalSize, PhysicalVec, Size, Sizes, ToLogical,
    ToLogicalWithContainingBlock,
};
use crate::layout_box_base::LayoutBoxBase;
use crate::sizing::ContentSizes;
use crate::style_ext::{Clamp, ComputedValuesExt, ContentBoxSizesAndPBM, DisplayInside};
use crate::{
    ConstraintSpace, ContainingBlock, ContainingBlockSize, DefiniteContainingBlock,
    PropagatedBoxTreeData, SizeConstraint,
};

#[derive(Debug, MallocSizeOf)]
pub(crate) struct AbsolutelyPositionedBox {
    pub context: IndependentFormattingContext,
}

#[derive(Clone, MallocSizeOf)]
pub(crate) struct HoistedAbsolutelyPositionedBox {
    absolutely_positioned_box: ArcRefCell<AbsolutelyPositionedBox>,

    /// A reference to a Fragment which is shared between this `HoistedAbsolutelyPositionedBox`
    /// and its placeholder `AbsoluteOrFixedPositionedFragment` in the original tree position.
    /// This will be used later in order to paint this hoisted box in tree order.
    pub fragment: ArcRefCell<HoistedSharedFragment>,
}

impl AbsolutelyPositionedBox {
    pub fn new(context: IndependentFormattingContext) -> Self {
        Self { context }
    }

    pub fn construct(
        context: &LayoutContext,
        node_info: &NodeAndStyleInfo,
        display_inside: DisplayInside,
        contents: Contents,
    ) -> Self {
        Self {
            context: IndependentFormattingContext::construct(
                context,
                node_info,
                display_inside,
                contents,
                // Text decorations are not propagated to any out-of-flow descendants. In addition,
                // absolutes don't affect the size of ancestors so it is fine to allow descendent
                // tables to resolve percentage columns.
                PropagatedBoxTreeData::default(),
            ),
        }
    }

    pub(crate) fn to_hoisted(
        absolutely_positioned_box: ArcRefCell<Self>,
        static_position_rectangle: PhysicalRect<Au>,
        resolved_alignment: LogicalVec2<AlignFlags>,
        original_parent_writing_mode: WritingMode,
    ) -> HoistedAbsolutelyPositionedBox {
        HoistedAbsolutelyPositionedBox {
            fragment: ArcRefCell::new(HoistedSharedFragment::new(
                static_position_rectangle,
                resolved_alignment,
                original_parent_writing_mode,
            )),
            absolutely_positioned_box,
        }
    }
}

#[derive(Clone, Default, MallocSizeOf)]
pub(crate) struct PositioningContext {
    absolutes: Vec<HoistedAbsolutelyPositionedBox>,
}

impl PositioningContext {
    #[inline]
    pub(crate) fn new_for_layout_box_base(layout_box_base: &LayoutBoxBase) -> Option<Self> {
        Self::new_for_style_and_fragment_flags(
            &layout_box_base.style,
            &layout_box_base.base_fragment_info.flags,
        )
    }

    fn new_for_style_and_fragment_flags(
        style: &ComputedValues,
        flags: &FragmentFlags,
    ) -> Option<Self> {
        if style.establishes_containing_block_for_absolute_descendants(*flags) {
            Some(Self::default())
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
            Fragment::Box(fragment) | Fragment::Float(fragment) => {
                fragment.borrow().content_rect.origin
            },
            Fragment::AbsoluteOrFixedPositioned(_) => return,
            Fragment::Positioning(fragment) => fragment.borrow().rect.origin,
            _ => unreachable!(),
        };
        self.adjust_static_position_of_hoisted_fragments_with_offset(
            &start_offset.to_vector(),
            index,
        );
    }

    /// See documentation for [PositioningContext::adjust_static_position_of_hoisted_fragments].
    pub(crate) fn adjust_static_position_of_hoisted_fragments_with_offset(
        &mut self,
        offset: &PhysicalVec<Au>,
        index: PositioningContextLength,
    ) {
        self.absolutes
            .iter_mut()
            .skip(index.0)
            .for_each(|hoisted_fragment| {
                hoisted_fragment
                    .fragment
                    .borrow_mut()
                    .adjust_offsets(offset)
            })
    }

    /// Given `fragment_layout_fn`, a closure which lays out a fragment in a provided
    /// `PositioningContext`, create a new positioning context if necessary for the fragment and
    /// lay out the fragment and all its children. Returns the newly created `BoxFragment`.
    pub(crate) fn layout_maybe_position_relative_fragment(
        &mut self,
        layout_context: &LayoutContext,
        containing_block: &ContainingBlock,
        base: &LayoutBoxBase,
        fragment_layout_fn: impl FnOnce(&mut Self) -> BoxFragment,
    ) -> BoxFragment {
        // If a new `PositioningContext` isn't necessary, simply create the fragment using
        // the given closure and the current `PositioningContext`.
        let establishes_containing_block_for_absolutes = base
            .style
            .establishes_containing_block_for_absolute_descendants(base.base_fragment_info.flags);
        if !establishes_containing_block_for_absolutes {
            return fragment_layout_fn(self);
        }

        let mut new_context = PositioningContext::default();
        let mut new_fragment = fragment_layout_fn(&mut new_context);

        // Lay out all of the absolutely positioned children for this fragment, and, if it
        // isn't a containing block for fixed elements, then pass those up to the parent.
        new_context.layout_collected_children(layout_context, &mut new_fragment);
        self.append(new_context);

        if base.style.clone_position() == Position::Relative {
            new_fragment.content_rect.origin += relative_adjustement(&base.style, containing_block)
                .to_physical_vector(containing_block.style.writing_mode)
        }

        new_fragment
    }

    fn take_boxes_for_fragment(
        &mut self,
        new_fragment: &BoxFragment,
        boxes_to_layout_out: &mut Vec<HoistedAbsolutelyPositionedBox>,
        boxes_to_continue_hoisting_out: &mut Vec<HoistedAbsolutelyPositionedBox>,
    ) {
        debug_assert!(
            new_fragment
                .style
                .establishes_containing_block_for_absolute_descendants(new_fragment.base.flags)
        );

        if new_fragment
            .style
            .establishes_containing_block_for_all_descendants(new_fragment.base.flags)
        {
            boxes_to_layout_out.append(&mut self.absolutes);
            return;
        }

        // TODO: This could potentially use `extract_if` when that is stabilized.
        let (mut boxes_to_layout, mut boxes_to_continue_hoisting) = self
            .absolutes
            .drain(..)
            .partition(|hoisted_box| hoisted_box.position() != Position::Fixed);
        boxes_to_layout_out.append(&mut boxes_to_layout);
        boxes_to_continue_hoisting_out.append(&mut boxes_to_continue_hoisting);
    }

    // Lay out the hoisted boxes collected into this `PositioningContext` and add them
    // to the given `BoxFragment`.
    pub(crate) fn layout_collected_children(
        &mut self,
        layout_context: &LayoutContext,
        new_fragment: &mut BoxFragment,
    ) {
        if self.absolutes.is_empty() {
            return;
        }

        // Sometimes we create temporary PositioningContexts just to collect hoisted absolutes and
        // then these are processed later. In that case and if this fragment doesn't establish a
        // containing block for absolutes at all, we just do nothing. All hoisted fragments will
        // later be passed up to a parent PositioningContext.
        //
        // Handling this case here, when the PositioningContext is completely ineffectual other than
        // as a temporary container for hoisted boxes, means that callers can execute less conditional
        // code.
        if !new_fragment
            .style
            .establishes_containing_block_for_absolute_descendants(new_fragment.base.flags)
        {
            return;
        }

        let padding_rect = PhysicalRect::new(
            // Ignore the content rect’s position in its own containing block:
            PhysicalPoint::origin(),
            new_fragment.content_rect.size,
        )
        .outer_rect(new_fragment.padding);
        let containing_block = DefiniteContainingBlock {
            size: padding_rect
                .size
                .to_logical(new_fragment.style.writing_mode),
            style: &new_fragment.style,
        };

        let mut fixed_position_boxes_to_hoist = Vec::new();
        let mut boxes_to_layout = Vec::new();
        self.take_boxes_for_fragment(
            new_fragment,
            &mut boxes_to_layout,
            &mut fixed_position_boxes_to_hoist,
        );

        // Laying out a `position: absolute` child (which only establishes a containing block for
        // `position: absolute` descendants) can result in more `position: fixed` descendants
        // collecting in `self.absolutes`. We need to loop here in order to keep either laying them
        // out or putting them into `fixed_position_boxes_to_hoist`. We know there aren't any more
        // when `self.absolutes` is empty.
        while !boxes_to_layout.is_empty() {
            HoistedAbsolutelyPositionedBox::layout_many(
                layout_context,
                std::mem::take(&mut boxes_to_layout),
                &mut new_fragment.children,
                &mut self.absolutes,
                &containing_block,
                new_fragment.padding,
            );

            self.take_boxes_for_fragment(
                new_fragment,
                &mut boxes_to_layout,
                &mut fixed_position_boxes_to_hoist,
            );
        }

        // We replace here instead of simply preserving these in `take_boxes_for_fragment`
        // so that we don't have to continually re-iterate over them when laying out in the
        // loop above.
        self.absolutes = fixed_position_boxes_to_hoist;
    }

    pub(crate) fn push(&mut self, hoisted_box: HoistedAbsolutelyPositionedBox) {
        debug_assert!(matches!(
            hoisted_box.position(),
            Position::Absolute | Position::Fixed
        ));
        self.absolutes.push(hoisted_box);
    }

    pub(crate) fn append(&mut self, mut other: Self) {
        if other.absolutes.is_empty() {
            return;
        }
        if self.absolutes.is_empty() {
            self.absolutes = other.absolutes;
        } else {
            self.absolutes.append(&mut other.absolutes)
        }
    }

    pub(crate) fn layout_initial_containing_block_children(
        &mut self,
        layout_context: &LayoutContext,
        initial_containing_block: &DefiniteContainingBlock,
        fragments: &mut Vec<Fragment>,
    ) {
        // Laying out a `position: absolute` child (which only establishes a containing block for
        // `position: absolute` descendants) can result in more `position: fixed` descendants
        // collecting in `self.absolutes`. We need to loop here in order to keep laying them out. We
        // know there aren't any more when `self.absolutes` is empty.
        while !self.absolutes.is_empty() {
            HoistedAbsolutelyPositionedBox::layout_many(
                layout_context,
                mem::take(&mut self.absolutes),
                fragments,
                &mut self.absolutes,
                initial_containing_block,
                Default::default(),
            )
        }
    }

    /// Get the length of this [PositioningContext].
    pub(crate) fn len(&self) -> PositioningContextLength {
        PositioningContextLength(self.absolutes.len())
    }

    /// Truncate this [PositioningContext] to the given [PositioningContextLength].  This
    /// is useful for "unhoisting" boxes in this context and returning it to the state at
    /// the time that [`PositioningContext::len()`] was called.
    pub(crate) fn truncate(&mut self, length: &PositioningContextLength) {
        self.absolutes.truncate(length.0)
    }
}

/// A data structure which stores the size of a positioning context.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PositioningContextLength(usize);

impl Zero for PositioningContextLength {
    fn zero() -> Self {
        Self(0)
    }

    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}

impl HoistedAbsolutelyPositionedBox {
    fn position(&self) -> Position {
        let position = self
            .absolutely_positioned_box
            .borrow()
            .context
            .style()
            .clone_position();
        assert!(position == Position::Fixed || position == Position::Absolute);
        position
    }

    pub(crate) fn layout_many(
        layout_context: &LayoutContext,
        mut boxes: Vec<Self>,
        fragments: &mut Vec<Fragment>,
        for_nearest_containing_block_for_all_descendants: &mut Vec<HoistedAbsolutelyPositionedBox>,
        containing_block: &DefiniteContainingBlock,
        containing_block_padding: PhysicalSides<Au>,
    ) {
        if layout_context.use_rayon {
            let mut new_fragments = Vec::new();
            let mut new_hoisted_boxes = Vec::new();

            boxes
                .par_iter_mut()
                .map(|hoisted_box| {
                    let mut new_hoisted_boxes: Vec<HoistedAbsolutelyPositionedBox> = Vec::new();
                    let new_fragment = hoisted_box.layout(
                        layout_context,
                        &mut new_hoisted_boxes,
                        containing_block,
                        containing_block_padding,
                    );

                    hoisted_box.fragment.borrow_mut().fragment = Some(new_fragment.clone());
                    (new_fragment, new_hoisted_boxes)
                })
                .unzip_into_vecs(&mut new_fragments, &mut new_hoisted_boxes);

            fragments.extend(new_fragments);
            for_nearest_containing_block_for_all_descendants
                .extend(new_hoisted_boxes.into_iter().flatten());
        } else {
            fragments.extend(boxes.iter_mut().map(|box_| {
                let new_fragment = box_.layout(
                    layout_context,
                    for_nearest_containing_block_for_all_descendants,
                    containing_block,
                    containing_block_padding,
                );

                box_.fragment.borrow_mut().fragment = Some(new_fragment.clone());
                new_fragment
            }))
        }
    }

    pub(crate) fn layout(
        &mut self,
        layout_context: &LayoutContext,
        hoisted_absolutes_from_children: &mut Vec<HoistedAbsolutelyPositionedBox>,
        containing_block: &DefiniteContainingBlock,
        containing_block_padding: PhysicalSides<Au>,
    ) -> Fragment {
        let cbis = containing_block.size.inline;
        let cbbs = containing_block.size.block;
        let containing_block_writing_mode = containing_block.style.writing_mode;
        let absolutely_positioned_box = self.absolutely_positioned_box.borrow();
        let context = &absolutely_positioned_box.context;
        let style = context.style().clone();
        let layout_style = context.layout_style();
        let ContentBoxSizesAndPBM {
            content_box_sizes,
            pbm,
            ..
        } = layout_style.content_box_sizes_and_padding_border_margin(&containing_block.into());
        let containing_block = &containing_block.into();
        let is_table = layout_style.is_table();
        let shared_fragment = self.fragment.borrow();

        // The static position rect was calculated assuming that the containing block would be
        // established by the content box of some ancestor, but the actual containing block is
        // established by the padding box. So we need to add the padding of that ancestor.
        let mut static_position_rect = shared_fragment
            .static_position_rect
            .outer_rect(-containing_block_padding);
        static_position_rect.size = static_position_rect.size.max(PhysicalSize::zero());
        let static_position_rect = static_position_rect.to_logical(containing_block);

        let box_offset = style.box_offsets(containing_block.style.writing_mode);

        // When the "static-position rect" doesn't come into play, we do not do any alignment
        // in the inline axis.
        let inline_box_offsets = box_offset.inline_sides();
        let inline_alignment = match inline_box_offsets.either_specified() {
            true => style.clone_justify_self().0.0,
            false => shared_fragment.resolved_alignment.inline,
        };

        let inline_axis_solver = AbsoluteAxisSolver {
            axis: Direction::Inline,
            containing_size: cbis,
            padding_border_sum: pbm.padding_border_sums.inline,
            computed_margin_start: pbm.margin.inline_start,
            computed_margin_end: pbm.margin.inline_end,
            computed_sizes: content_box_sizes.inline,
            avoid_negative_margin_start: true,
            box_offsets: inline_box_offsets,
            static_position_rect_axis: static_position_rect.get_axis(Direction::Inline),
            alignment: inline_alignment,
            flip_anchor: shared_fragment.original_parent_writing_mode.is_bidi_ltr() !=
                containing_block_writing_mode.is_bidi_ltr(),
            is_table,
        };

        // When the "static-position rect" doesn't come into play, we re-resolve "align-self"
        // against this containing block.
        let block_box_offsets = box_offset.block_sides();
        let block_alignment = match block_box_offsets.either_specified() {
            true => style.clone_align_self().0.0,
            false => shared_fragment.resolved_alignment.block,
        };
        let block_axis_solver = AbsoluteAxisSolver {
            axis: Direction::Block,
            containing_size: cbbs,
            padding_border_sum: pbm.padding_border_sums.block,
            computed_margin_start: pbm.margin.block_start,
            computed_margin_end: pbm.margin.block_end,
            computed_sizes: content_box_sizes.block,
            avoid_negative_margin_start: false,
            box_offsets: block_box_offsets,
            static_position_rect_axis: static_position_rect.get_axis(Direction::Block),
            alignment: block_alignment,
            flip_anchor: false,
            is_table,
        };

        let mut positioning_context = PositioningContext::default();
        let mut new_fragment = {
            let content_size: LogicalVec2<Au>;
            let fragments;
            let mut specific_layout_info: Option<SpecificLayoutInfo> = None;
            match &context.contents {
                IndependentFormattingContextContents::Replaced(replaced) => {
                    // https://drafts.csswg.org/css2/visudet.html#abs-replaced-width
                    // https://drafts.csswg.org/css2/visudet.html#abs-replaced-height
                    let inset_sums = LogicalVec2 {
                        inline: inline_axis_solver.inset_sum(),
                        block: block_axis_solver.inset_sum(),
                    };
                    let automatic_size = |alignment: AlignFlags, offsets: &LogicalSides1D<_>| {
                        if alignment.value() == AlignFlags::STRETCH && !offsets.either_auto() {
                            Size::Stretch
                        } else {
                            Size::FitContent
                        }
                    };
                    content_size = replaced.used_size_as_if_inline_element_from_content_box_sizes(
                        containing_block,
                        &style,
                        context.preferred_aspect_ratio(&pbm.padding_border_sums),
                        LogicalVec2 {
                            inline: &inline_axis_solver.computed_sizes,
                            block: &block_axis_solver.computed_sizes,
                        },
                        LogicalVec2 {
                            inline: automatic_size(
                                inline_alignment,
                                &inline_axis_solver.box_offsets,
                            ),
                            block: automatic_size(block_alignment, &block_axis_solver.box_offsets),
                        },
                        pbm.padding_border_sums + pbm.margin.auto_is(Au::zero).sum() + inset_sums,
                    );
                    fragments = replaced.make_fragments(
                        layout_context,
                        &style,
                        content_size.to_physical_size(containing_block_writing_mode),
                    );
                },
                IndependentFormattingContextContents::NonReplaced(non_replaced) => {
                    // https://drafts.csswg.org/css2/visudet.html#abs-non-replaced-width
                    // https://drafts.csswg.org/css2/visudet.html#abs-non-replaced-height

                    // The block size can depend on layout results, so we only solve it extrinsically,
                    // we may have to resolve it properly later on.
                    let extrinsic_block_size = block_axis_solver.solve_size_extrinsically();

                    // The inline axis can be fully resolved, computing intrinsic sizes using the
                    // extrinsic block size.
                    let inline_size = inline_axis_solver.solve_size(|| {
                        let ratio = context.preferred_aspect_ratio(&pbm.padding_border_sums);
                        let constraint_space =
                            ConstraintSpace::new(extrinsic_block_size, style.writing_mode, ratio);
                        context
                            .inline_content_sizes(layout_context, &constraint_space)
                            .sizes
                    });

                    let containing_block_for_children = ContainingBlock {
                        size: ContainingBlockSize {
                            inline: inline_size,
                            block: extrinsic_block_size,
                        },
                        style: &style,
                    };
                    // https://drafts.csswg.org/css-writing-modes/#orthogonal-flows
                    assert_eq!(
                        containing_block_writing_mode.is_horizontal(),
                        style.writing_mode.is_horizontal(),
                        "Mixed horizontal and vertical writing modes are not supported yet"
                    );

                    let independent_layout = non_replaced.layout(
                        layout_context,
                        &mut positioning_context,
                        &containing_block_for_children,
                        containing_block,
                        &context.base,
                        false, /* depends_on_block_constraints */
                    );

                    // Tables can become narrower than predicted due to collapsed columns
                    let inline_size = independent_layout
                        .content_inline_size_for_table
                        .unwrap_or(inline_size);

                    // Now we can properly solve the block size.
                    let block_size = block_axis_solver
                        .solve_size(|| independent_layout.content_block_size.into());

                    content_size = LogicalVec2 {
                        inline: inline_size,
                        block: block_size,
                    };
                    fragments = independent_layout.fragments;
                    specific_layout_info = independent_layout.specific_layout_info;
                },
            };

            let inline_margins = inline_axis_solver.solve_margins(content_size.inline);
            let block_margins = block_axis_solver.solve_margins(content_size.block);
            let margin = LogicalSides {
                inline_start: inline_margins.start,
                inline_end: inline_margins.end,
                block_start: block_margins.start,
                block_end: block_margins.end,
            };

            let pb = pbm.padding + pbm.border;
            let margin_rect_size = content_size + pbm.padding_border_sums + margin.sum();
            let inline_origin = inline_axis_solver.origin_for_margin_box(
                margin_rect_size.inline,
                style.writing_mode,
                shared_fragment.original_parent_writing_mode,
                containing_block_writing_mode,
            );
            let block_origin = block_axis_solver.origin_for_margin_box(
                margin_rect_size.block,
                style.writing_mode,
                shared_fragment.original_parent_writing_mode,
                containing_block_writing_mode,
            );

            let content_rect = LogicalRect {
                start_corner: LogicalVec2 {
                    inline: inline_origin + margin.inline_start + pb.inline_start,
                    block: block_origin + margin.block_start + pb.block_start,
                },
                size: content_size,
            };
            BoxFragment::new(
                context.base_fragment_info(),
                style,
                fragments,
                content_rect.as_physical(Some(containing_block)),
                pbm.padding.to_physical(containing_block_writing_mode),
                pbm.border.to_physical(containing_block_writing_mode),
                margin.to_physical(containing_block_writing_mode),
                None, /* clearance */
            )
            .with_specific_layout_info(specific_layout_info)
        };

        // This is an absolutely positioned element, which means it also establishes a
        // containing block for absolutes. We lay out any absolutely positioned children
        // here and pass the rest to `hoisted_absolutes_from_children.`
        positioning_context.layout_collected_children(layout_context, &mut new_fragment);

        // Any hoisted boxes that remain in this positioning context are going to be hoisted
        // up above this absolutely positioned box. These will necessarily be fixed position
        // elements, because absolutely positioned elements form containing blocks for all
        // other elements. If any of them have a static start position though, we need to
        // adjust it to account for the start corner of this absolute.
        positioning_context.adjust_static_position_of_hoisted_fragments_with_offset(
            &new_fragment.content_rect.origin.to_vector(),
            PositioningContextLength::zero(),
        );

        hoisted_absolutes_from_children.extend(positioning_context.absolutes);

        let fragment = Fragment::Box(ArcRefCell::new(new_fragment));
        context.base.set_fragment(fragment.clone());
        fragment
    }
}

#[derive(Clone, Copy, Debug)]
struct RectAxis {
    origin: Au,
    length: Au,
}

impl LogicalRect<Au> {
    fn get_axis(&self, axis: Direction) -> RectAxis {
        match axis {
            Direction::Block => RectAxis {
                origin: self.start_corner.block,
                length: self.size.block,
            },
            Direction::Inline => RectAxis {
                origin: self.start_corner.inline,
                length: self.size.inline,
            },
        }
    }
}

struct AbsoluteAxisSolver<'a> {
    axis: Direction,
    containing_size: Au,
    padding_border_sum: Au,
    computed_margin_start: AuOrAuto,
    computed_margin_end: AuOrAuto,
    computed_sizes: Sizes,
    avoid_negative_margin_start: bool,
    box_offsets: LogicalSides1D<LengthPercentageOrAuto<'a>>,
    static_position_rect_axis: RectAxis,
    alignment: AlignFlags,
    flip_anchor: bool,
    is_table: bool,
}

impl AbsoluteAxisSolver<'_> {
    /// Returns the amount that we need to subtract from the containing block size in order to
    /// obtain the inset-modified containing block that we will use for sizing purposes.
    /// (Note that for alignment purposes, we may re-resolve auto insets to a different value.)
    /// <https://drafts.csswg.org/css-position/#resolving-insets>
    fn inset_sum(&self) -> Au {
        match (
            self.box_offsets.start.non_auto(),
            self.box_offsets.end.non_auto(),
        ) {
            (None, None) => {
                if self.flip_anchor {
                    self.containing_size -
                        self.static_position_rect_axis.origin -
                        self.static_position_rect_axis.length
                } else {
                    self.static_position_rect_axis.origin
                }
            },
            (Some(start), None) => start.to_used_value(self.containing_size),
            (None, Some(end)) => end.to_used_value(self.containing_size),
            (Some(start), Some(end)) => {
                start.to_used_value(self.containing_size) + end.to_used_value(self.containing_size)
            },
        }
    }

    #[inline]
    fn automatic_size(&self) -> Size<Au> {
        match self.alignment.value() {
            _ if self.box_offsets.either_auto() => Size::FitContent,
            AlignFlags::NORMAL | AlignFlags::AUTO if !self.is_table => Size::Stretch,
            AlignFlags::STRETCH => Size::Stretch,
            _ => Size::FitContent,
        }
    }

    #[inline]
    fn stretch_size(&self) -> Au {
        Au::zero().max(
            self.containing_size -
                self.inset_sum() -
                self.padding_border_sum -
                self.computed_margin_start.auto_is(Au::zero) -
                self.computed_margin_end.auto_is(Au::zero),
        )
    }

    #[inline]
    fn solve_size_extrinsically(&self) -> SizeConstraint {
        self.computed_sizes.resolve_extrinsic(
            self.automatic_size(),
            Au::zero(),
            Some(self.stretch_size()),
        )
    }

    #[inline]
    fn solve_size(&self, get_content_size: impl FnOnce() -> ContentSizes) -> Au {
        self.computed_sizes.resolve(
            self.axis,
            self.automatic_size(),
            Au::zero,
            Some(self.stretch_size()),
            get_content_size,
            self.is_table,
        )
    }

    fn solve_margins(&self, size: Au) -> LogicalSides1D<Au> {
        if self.box_offsets.either_auto() {
            LogicalSides1D::new(
                self.computed_margin_start.auto_is(Au::zero),
                self.computed_margin_end.auto_is(Au::zero),
            )
        } else {
            let free_space =
                self.containing_size - self.inset_sum() - self.padding_border_sum - size;
            match (self.computed_margin_start, self.computed_margin_end) {
                (AuOrAuto::Auto, AuOrAuto::Auto) => {
                    if self.avoid_negative_margin_start && free_space < Au::zero() {
                        LogicalSides1D::new(Au::zero(), free_space)
                    } else {
                        let margin_start = free_space / 2;
                        LogicalSides1D::new(margin_start, free_space - margin_start)
                    }
                },
                (AuOrAuto::Auto, AuOrAuto::LengthPercentage(end)) => {
                    LogicalSides1D::new(free_space - end, end)
                },
                (AuOrAuto::LengthPercentage(start), AuOrAuto::Auto) => {
                    LogicalSides1D::new(start, free_space - start)
                },
                (AuOrAuto::LengthPercentage(start), AuOrAuto::LengthPercentage(end)) => {
                    LogicalSides1D::new(start, end)
                },
            }
        }
    }

    fn origin_for_margin_box(
        &self,
        size: Au,
        self_writing_mode: WritingMode,
        original_parent_writing_mode: WritingMode,
        containing_block_writing_mode: WritingMode,
    ) -> Au {
        let (alignment_container, alignment_container_writing_mode, flip_anchor, offsets) = match (
            self.box_offsets.start.non_auto(),
            self.box_offsets.end.non_auto(),
        ) {
            (None, None) => (
                self.static_position_rect_axis,
                original_parent_writing_mode,
                self.flip_anchor,
                None,
            ),
            (Some(start), Some(end)) => {
                let offsets = LogicalSides1D {
                    start: start.to_used_value(self.containing_size),
                    end: end.to_used_value(self.containing_size),
                };
                let alignment_container = RectAxis {
                    origin: offsets.start,
                    length: self.containing_size - offsets.sum(),
                };
                (
                    alignment_container,
                    containing_block_writing_mode,
                    false,
                    Some(offsets),
                )
            },
            // If a single offset is auto, for alignment purposes it resolves to the amount
            // that makes the inset-modified containing block be exactly as big as the abspos.
            // Therefore the free space is zero and the alignment value is irrelevant.
            (Some(start), None) => return start.to_used_value(self.containing_size),
            (None, Some(end)) => {
                return self.containing_size - size - end.to_used_value(self.containing_size);
            },
        };

        assert_eq!(
            self_writing_mode.is_horizontal(),
            original_parent_writing_mode.is_horizontal(),
            "Mixed horizontal and vertical writing modes are not supported yet"
        );
        assert_eq!(
            self_writing_mode.is_horizontal(),
            containing_block_writing_mode.is_horizontal(),
            "Mixed horizontal and vertical writing modes are not supported yet"
        );
        let self_value_matches_container = || {
            self.axis == Direction::Block ||
                self_writing_mode.is_bidi_ltr() == alignment_container_writing_mode.is_bidi_ltr()
        };

        // Here we resolve the alignment to either start, center, or end.
        // Note we need to handle both self-alignment values (when some inset isn't auto)
        // and distributed alignment values (when both insets are auto).
        // The latter are treated as their fallback alignment.
        let alignment = match self.alignment.value() {
            // https://drafts.csswg.org/css-align/#valdef-self-position-center
            // https://drafts.csswg.org/css-align/#valdef-align-content-space-around
            // https://drafts.csswg.org/css-align/#valdef-align-content-space-evenly
            AlignFlags::CENTER | AlignFlags::SPACE_AROUND | AlignFlags::SPACE_EVENLY => {
                AlignFlags::CENTER
            },
            // https://drafts.csswg.org/css-align/#valdef-self-position-self-start
            AlignFlags::SELF_START if self_value_matches_container() => AlignFlags::START,
            AlignFlags::SELF_START => AlignFlags::END,
            // https://drafts.csswg.org/css-align/#valdef-self-position-self-end
            AlignFlags::SELF_END if self_value_matches_container() => AlignFlags::END,
            AlignFlags::SELF_END => AlignFlags::START,
            // https://drafts.csswg.org/css-align/#valdef-justify-content-left
            AlignFlags::LEFT if alignment_container_writing_mode.is_bidi_ltr() => AlignFlags::START,
            AlignFlags::LEFT => AlignFlags::END,
            // https://drafts.csswg.org/css-align/#valdef-justify-content-right
            AlignFlags::RIGHT if alignment_container_writing_mode.is_bidi_ltr() => AlignFlags::END,
            AlignFlags::RIGHT => AlignFlags::START,
            // https://drafts.csswg.org/css-align/#valdef-self-position-end
            // https://drafts.csswg.org/css-align/#valdef-self-position-flex-end
            AlignFlags::END | AlignFlags::FLEX_END => AlignFlags::END,
            // https://drafts.csswg.org/css-align/#valdef-self-position-start
            // https://drafts.csswg.org/css-align/#valdef-self-position-flex-start
            _ => AlignFlags::START,
        };

        let alignment = match alignment {
            AlignFlags::START if flip_anchor => AlignFlags::END,
            AlignFlags::END if flip_anchor => AlignFlags::START,
            alignment => alignment,
        };

        let free_space = alignment_container.length - size;
        let flags = self.alignment.flags();
        let alignment = if flags == AlignFlags::SAFE && free_space < Au::zero() {
            AlignFlags::START
        } else {
            alignment
        };

        let origin = match alignment {
            AlignFlags::START => alignment_container.origin,
            AlignFlags::CENTER => alignment_container.origin + free_space / 2,
            AlignFlags::END => alignment_container.origin + free_space,
            _ => unreachable!(),
        };
        if matches!(flags, AlignFlags::SAFE | AlignFlags::UNSAFE) ||
            matches!(
                self.alignment,
                AlignFlags::NORMAL | AlignFlags::AUTO | AlignFlags::STRETCH
            )
        {
            return origin;
        }
        let Some(offsets) = offsets else {
            return origin;
        };

        // Handle default overflow alignment.
        // https://drafts.csswg.org/css-align/#auto-safety-position
        let min = Au::zero().min(offsets.start);
        let max = self.containing_size - Au::zero().min(offsets.end) - size;
        origin.clamp_between_extremums(min, Some(max))
    }
}

/// <https://drafts.csswg.org/css2/visuren.html#relative-positioning>
pub(crate) fn relative_adjustement(
    style: &ComputedValues,
    containing_block: &ContainingBlock,
) -> LogicalVec2<Au> {
    // It's not completely clear what to do with indefinite percentages
    // (https://github.com/w3c/csswg-drafts/issues/9353), so we match
    // other browsers and treat them as 'auto' offsets.
    let cbis = containing_block.size.inline;
    let cbbs = containing_block.size.block;
    let box_offsets = style
        .box_offsets(containing_block.style.writing_mode)
        .map_inline_and_block_axes(
            |value| value.map(|value| value.to_used_value(cbis)),
            |value| match cbbs {
                SizeConstraint::Definite(cbbs) => value.map(|value| value.to_used_value(cbbs)),
                _ => match value.non_auto().and_then(|value| value.to_length()) {
                    Some(value) => AuOrAuto::LengthPercentage(value.into()),
                    None => AuOrAuto::Auto,
                },
            },
        );
    fn adjust(start: AuOrAuto, end: AuOrAuto) -> Au {
        match (start, end) {
            (AuOrAuto::Auto, AuOrAuto::Auto) => Au::zero(),
            (AuOrAuto::Auto, AuOrAuto::LengthPercentage(end)) => -end,
            (AuOrAuto::LengthPercentage(start), _) => start,
        }
    }
    LogicalVec2 {
        inline: adjust(box_offsets.inline_start, box_offsets.inline_end),
        block: adjust(box_offsets.block_start, box_offsets.block_end),
    }
}
