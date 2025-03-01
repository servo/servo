/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::mem;

use app_units::Au;
use rayon::iter::IntoParallelRefMutIterator;
use rayon::prelude::{IndexedParallelIterator, ParallelIterator};
use style::computed_values::position::T as Position;
use style::logical_geometry::{Direction, WritingMode};
use style::properties::ComputedValues;
use style::values::specified::align::AlignFlags;
use style::Zero;

use crate::cell::ArcRefCell;
use crate::context::LayoutContext;
use crate::dom::NodeExt;
use crate::dom_traversal::{Contents, NodeAndStyleInfo};
use crate::formatting_contexts::{
    IndependentFormattingContext, IndependentFormattingContextContents,
};
use crate::fragment_tree::{
    BoxFragment, CollapsedBlockMargins, Fragment, FragmentFlags, HoistedSharedFragment,
    SpecificLayoutInfo,
};
use crate::geom::{
    AuOrAuto, LengthPercentageOrAuto, LogicalRect, LogicalSides, LogicalVec2, PhysicalPoint,
    PhysicalRect, PhysicalVec, Size, Sizes, ToLogical, ToLogicalWithContainingBlock,
};
use crate::sizing::ContentSizes;
use crate::style_ext::{Clamp, ComputedValuesExt, ContentBoxSizesAndPBM, DisplayInside};
use crate::{
    ConstraintSpace, ContainingBlock, ContainingBlockSize, DefiniteContainingBlock,
    PropagatedBoxTreeData, SizeConstraint,
};

#[derive(Debug)]
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
    pub fn new(context: IndependentFormattingContext) -> Self {
        Self { context }
    }

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

impl PositioningContext {
    pub(crate) fn new_for_containing_block_for_all_descendants() -> Self {
        Self {
            for_nearest_positioned_ancestor: None,
            for_nearest_containing_block_for_all_descendants: Vec::new(),
        }
    }

    /// Create a [PositioningContext] to use for laying out a subtree. The idea is that
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
        // NB: We never make PositioningContexts for replaced elements, which is why we always
        // pass false here.
        if style.establishes_containing_block_for_all_descendants(FragmentFlags::empty()) {
            Some(Self::new_for_containing_block_for_all_descendants())
        } else if style
            .establishes_containing_block_for_absolute_descendants(FragmentFlags::empty())
        {
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
        if let Some(hoisted_boxes) = self.for_nearest_positioned_ancestor.as_mut() {
            hoisted_boxes
                .iter_mut()
                .skip(index.for_nearest_positioned_ancestor)
                .for_each(|hoisted_fragment| {
                    hoisted_fragment
                        .fragment
                        .borrow_mut()
                        .adjust_offsets(offset)
                })
        }
        self.for_nearest_containing_block_for_all_descendants
            .iter_mut()
            .skip(index.for_nearest_containing_block_for_all_descendants)
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
            new_fragment.content_rect.origin += relative_adjustement(style, containing_block)
                .to_physical_vector(containing_block.style.writing_mode)
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

        let take_hoisted_boxes_pending_layout =
            |context: &mut Self| match context.for_nearest_positioned_ancestor.as_mut() {
                Some(fragments) => mem::take(fragments),
                None => mem::take(&mut context.for_nearest_containing_block_for_all_descendants),
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
                .is_none_or(|vector| vector.is_empty())
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
                &mut mem::take(&mut self.for_nearest_containing_block_for_all_descendants),
                fragments,
                &mut self.for_nearest_containing_block_for_all_descendants,
                initial_containing_block,
            )
        }
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
    /// the time that [`PositioningContext::len()`] was called.
    pub(crate) fn truncate(&mut self, length: &PositioningContextLength) {
        if let Some(vec) = self.for_nearest_positioned_ancestor.as_mut() {
            vec.truncate(length.for_nearest_positioned_ancestor);
        }
        self.for_nearest_containing_block_for_all_descendants
            .truncate(length.for_nearest_containing_block_for_all_descendants);
    }
}

/// A data structure which stores the size of a positioning context.
#[derive(Clone, Copy, Debug, PartialEq)]
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
        fragments: &mut Vec<Fragment>,
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
                    let new_fragment = hoisted_box.layout(
                        layout_context,
                        &mut new_hoisted_boxes,
                        containing_block,
                    );

                    hoisted_box.fragment.borrow_mut().fragment =
                        Some(Fragment::Box(new_fragment.clone()));
                    (Fragment::Box(new_fragment), new_hoisted_boxes)
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
                );

                box_.fragment.borrow_mut().fragment = Some(Fragment::Box(new_fragment.clone()));
                Fragment::Box(new_fragment)
            }))
        }
    }

    pub(crate) fn layout(
        &mut self,
        layout_context: &LayoutContext,
        for_nearest_containing_block_for_all_descendants: &mut Vec<HoistedAbsolutelyPositionedBox>,
        containing_block: &DefiniteContainingBlock,
    ) -> ArcRefCell<BoxFragment> {
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
        let static_position_rect = shared_fragment
            .static_position_rect
            .to_logical(containing_block);

        let box_offset = style.box_offsets(containing_block.style.writing_mode);

        // When the "static-position rect" doesn't come into play, we do not do any alignment
        // in the inline axis.
        let inline_box_offsets = AbsoluteBoxOffsets {
            start: box_offset.inline_start,
            end: box_offset.inline_end,
        };
        let inline_alignment = match inline_box_offsets.either_specified() {
            true => style.clone_justify_self().0 .0,
            false => shared_fragment.resolved_alignment.inline,
        };

        let mut inline_axis_solver = AbsoluteAxisSolver {
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
        let block_box_offsets = AbsoluteBoxOffsets {
            start: box_offset.block_start,
            end: box_offset.block_end,
        };
        let block_alignment = match block_box_offsets.either_specified() {
            true => style.clone_align_self().0 .0,
            false => shared_fragment.resolved_alignment.block,
        };
        let mut block_axis_solver = AbsoluteAxisSolver {
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

        if let IndependentFormattingContextContents::Replaced(replaced) = &context.contents {
            // https://drafts.csswg.org/css2/visudet.html#abs-replaced-width
            // https://drafts.csswg.org/css2/visudet.html#abs-replaced-height
            let inset_sums = LogicalVec2 {
                inline: inline_axis_solver.inset_sum(),
                block: block_axis_solver.inset_sum(),
            };
            let automatic_size = |alignment: AlignFlags, offsets: &AbsoluteBoxOffsets<_>| {
                if alignment.value() == AlignFlags::STRETCH && !offsets.either_auto() {
                    Size::Stretch
                } else {
                    Size::FitContent
                }
            };
            let used_size = replaced.used_size_as_if_inline_element_from_content_box_sizes(
                containing_block,
                &style,
                context.preferred_aspect_ratio(&pbm.padding_border_sums),
                LogicalVec2 {
                    inline: &inline_axis_solver.computed_sizes,
                    block: &block_axis_solver.computed_sizes,
                },
                LogicalVec2 {
                    inline: automatic_size(inline_alignment, &inline_axis_solver.box_offsets),
                    block: automatic_size(block_alignment, &block_axis_solver.box_offsets),
                },
                pbm.padding_border_sums + pbm.margin.auto_is(Au::zero).sum() + inset_sums,
            );
            inline_axis_solver.override_size(used_size.inline);
            block_axis_solver.override_size(used_size.block);
        }

        // The block axis can depend on layout results, so we only solve it tentatively,
        // we may have to resolve it properly later on.
        let mut block_axis = block_axis_solver.solve_tentatively();

        // The inline axis can be fully resolved, computing intrinsic sizes using the
        // tentative block size.
        let mut inline_axis = inline_axis_solver.solve(Some(|| {
            let ratio = context.preferred_aspect_ratio(&pbm.padding_border_sums);
            let constraint_space = ConstraintSpace::new(block_axis.size, style.writing_mode, ratio);
            context
                .inline_content_sizes(layout_context, &constraint_space)
                .sizes
        }));

        let mut positioning_context = PositioningContext::new_for_style(&style).unwrap();
        let mut new_fragment = {
            let content_size: LogicalVec2<Au>;
            let fragments;
            let mut specific_layout_info: Option<SpecificLayoutInfo> = None;
            match &context.contents {
                IndependentFormattingContextContents::Replaced(replaced) => {
                    // https://drafts.csswg.org/css2/visudet.html#abs-replaced-width
                    // https://drafts.csswg.org/css2/visudet.html#abs-replaced-height
                    content_size = LogicalVec2 {
                        inline: inline_axis.size.to_definite().unwrap(),
                        block: block_axis.size.to_definite().unwrap(),
                    };
                    fragments = replaced.make_fragments(
                        layout_context,
                        &style,
                        content_size.to_physical_size(containing_block_writing_mode),
                    );
                },
                IndependentFormattingContextContents::NonReplaced(non_replaced) => {
                    // https://drafts.csswg.org/css2/visudet.html#abs-non-replaced-width
                    // https://drafts.csswg.org/css2/visudet.html#abs-non-replaced-height
                    let inline_size = inline_axis.size.to_definite().unwrap();
                    let containing_block_for_children = ContainingBlock {
                        size: ContainingBlockSize {
                            inline: inline_size,
                            block: block_axis.size,
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
                    );

                    let inline_size = if let Some(inline_size) =
                        independent_layout.content_inline_size_for_table
                    {
                        // Tables can become narrower than predicted due to collapsed columns,
                        // so we need to solve again to update margins.
                        inline_axis_solver.override_size(inline_size);
                        inline_axis = inline_axis_solver.solve_tentatively();
                        inline_size
                    } else {
                        inline_size
                    };

                    // Now we can properly solve the block size.
                    block_axis = block_axis_solver
                        .solve(Some(|| independent_layout.content_block_size.into()));
                    let block_size = block_axis.size.to_definite().unwrap();

                    content_size = LogicalVec2 {
                        inline: inline_size,
                        block: block_size,
                    };
                    fragments = independent_layout.fragments;
                    specific_layout_info = independent_layout.specific_layout_info;
                },
            };

            let margin = LogicalSides {
                inline_start: inline_axis.margin_start,
                inline_end: inline_axis.margin_end,
                block_start: block_axis.margin_start,
                block_end: block_axis.margin_end,
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
                // We do not set the baseline offset, because absolutely positioned
                // elements are not inflow.
                CollapsedBlockMargins::zero(),
            )
            .with_specific_layout_info(specific_layout_info)
        };
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

        for_nearest_containing_block_for_all_descendants
            .extend(positioning_context.for_nearest_containing_block_for_all_descendants);

        ArcRefCell::new(new_fragment)
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

#[derive(Debug)]
struct AbsoluteBoxOffsets<T> {
    start: T,
    end: T,
}

impl AbsoluteBoxOffsets<LengthPercentageOrAuto<'_>> {
    pub(crate) fn either_specified(&self) -> bool {
        !self.start.is_auto() || !self.end.is_auto()
    }

    pub(crate) fn either_auto(&self) -> bool {
        self.start.is_auto() || self.end.is_auto()
    }
}

impl AbsoluteBoxOffsets<Au> {
    pub(crate) fn sum(&self) -> Au {
        self.start + self.end
    }
}

struct AxisResult {
    size: SizeConstraint,
    margin_start: Au,
    margin_end: Au,
}

struct AbsoluteAxisSolver<'a> {
    axis: Direction,
    containing_size: Au,
    padding_border_sum: Au,
    computed_margin_start: AuOrAuto,
    computed_margin_end: AuOrAuto,
    computed_sizes: Sizes,
    avoid_negative_margin_start: bool,
    box_offsets: AbsoluteBoxOffsets<LengthPercentageOrAuto<'a>>,
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

    /// This unifies some of the parts in common in:
    ///
    /// * <https://drafts.csswg.org/css2/visudet.html#abs-non-replaced-width>
    /// * <https://drafts.csswg.org/css2/visudet.html#abs-non-replaced-height>
    ///
    /// … and:
    ///
    /// * <https://drafts.csswg.org/css2/visudet.html#abs-replaced-width>
    /// * <https://drafts.csswg.org/css2/visudet.html#abs-replaced-height>
    ///
    /// In the replaced case, `size` is never `Auto`.
    fn solve(&self, get_content_size: Option<impl FnOnce() -> ContentSizes>) -> AxisResult {
        let solve_size = |initial_behavior, stretch_size: Au| -> SizeConstraint {
            let stretch_size = stretch_size.max(Au::zero());
            if let Some(get_content_size) = get_content_size {
                SizeConstraint::Definite(self.computed_sizes.resolve(
                    self.axis,
                    initial_behavior,
                    Au::zero(),
                    Some(stretch_size),
                    get_content_size,
                    self.is_table,
                ))
            } else {
                self.computed_sizes.resolve_extrinsic(
                    initial_behavior,
                    Au::zero(),
                    Some(stretch_size),
                )
            }
        };
        if self.box_offsets.either_auto() {
            let margin_start = self.computed_margin_start.auto_is(Au::zero);
            let margin_end = self.computed_margin_end.auto_is(Au::zero);
            let stretch_size = self.containing_size -
                self.inset_sum() -
                self.padding_border_sum -
                margin_start -
                margin_end;
            let size = solve_size(Size::FitContent, stretch_size);
            AxisResult {
                size,
                margin_start,
                margin_end,
            }
        } else {
            let mut free_space = self.containing_size - self.inset_sum() - self.padding_border_sum;
            let stretch_size = free_space -
                self.computed_margin_start.auto_is(Au::zero) -
                self.computed_margin_end.auto_is(Au::zero);
            let initial_behavior = match self.alignment.value() {
                AlignFlags::NORMAL | AlignFlags::AUTO if !self.is_table => Size::Stretch,
                AlignFlags::STRETCH => Size::Stretch,
                _ => Size::FitContent,
            };
            let size = solve_size(initial_behavior, stretch_size);
            if let Some(used_size) = size.to_definite() {
                free_space -= used_size;
            } else {
                free_space = Au::zero();
            }
            let (margin_start, margin_end) =
                match (self.computed_margin_start, self.computed_margin_end) {
                    (AuOrAuto::Auto, AuOrAuto::Auto) => {
                        if self.avoid_negative_margin_start && free_space < Au::zero() {
                            (Au::zero(), free_space)
                        } else {
                            let margin_start = free_space / 2;
                            (margin_start, free_space - margin_start)
                        }
                    },
                    (AuOrAuto::Auto, AuOrAuto::LengthPercentage(end)) => (free_space - end, end),
                    (AuOrAuto::LengthPercentage(start), AuOrAuto::Auto) => {
                        (start, free_space - start)
                    },
                    (AuOrAuto::LengthPercentage(start), AuOrAuto::LengthPercentage(end)) => {
                        (start, end)
                    },
                };
            AxisResult {
                size,
                margin_start,
                margin_end,
            }
        }
    }

    fn solve_tentatively(&mut self) -> AxisResult {
        self.solve(None::<fn() -> ContentSizes>)
    }

    fn override_size(&mut self, size: Au) {
        self.computed_sizes.preferred = Size::Numeric(size);
        self.computed_sizes.min = Size::default();
        self.computed_sizes.max = Size::default();
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
                let offsets = AbsoluteBoxOffsets {
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
                return self.containing_size - size - end.to_used_value(self.containing_size)
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

fn vec_append_owned<T>(a: &mut Vec<T>, mut b: Vec<T>) {
    if a.is_empty() {
        *a = b
    } else {
        a.append(&mut b)
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
