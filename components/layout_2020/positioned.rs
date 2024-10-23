/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::LazyCell;
use std::mem;

use app_units::Au;
use rayon::iter::IntoParallelRefMutIterator;
use rayon::prelude::{IndexedParallelIterator, ParallelIterator};
use serde::Serialize;
use style::computed_values::position::T as Position;
use style::logical_geometry::WritingMode;
use style::properties::ComputedValues;
use style::values::specified::align::{AlignFlags, AxisDirection};
use style::values::specified::text::TextDecorationLine;
use style::Zero;

use crate::cell::ArcRefCell;
use crate::context::LayoutContext;
use crate::dom::NodeExt;
use crate::dom_traversal::{Contents, NodeAndStyleInfo};
use crate::formatting_contexts::IndependentFormattingContext;
use crate::fragment_tree::{
    BoxFragment, CollapsedBlockMargins, Fragment, FragmentFlags, HoistedSharedFragment,
};
use crate::geom::{
    AuOrAuto, LengthPercentageOrAuto, LogicalRect, LogicalSides, LogicalVec2, PhysicalPoint,
    PhysicalRect, PhysicalVec, Size, ToLogical, ToLogicalWithContainingBlock,
};
use crate::sizing::ContentSizes;
use crate::style_ext::{Clamp, ComputedValuesExt, DisplayInside};
use crate::{ContainingBlock, DefiniteContainingBlock, IndefiniteContainingBlock};

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
            Fragment::Box(fragment) | Fragment::Float(fragment) => &fragment.content_rect.origin,
            Fragment::AbsoluteOrFixedPositioned(_) => return,
            Fragment::Positioning(fragment) => &fragment.rect.origin,
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
        let containing_block_writing_mode = containing_block.style.writing_mode;
        let mut absolutely_positioned_box = self.absolutely_positioned_box.borrow_mut();
        let context = &mut absolutely_positioned_box.context;
        let style = context.style().clone();
        let containing_block = &containing_block.into();
        let pbm = style.padding_border_margin(containing_block);

        let (computed_size, computed_min_size, computed_max_size) = match context {
            IndependentFormattingContext::Replaced(replaced) => {
                // https://drafts.csswg.org/css2/visudet.html#abs-replaced-width
                // https://drafts.csswg.org/css2/visudet.html#abs-replaced-height
                let used_size = replaced
                    .contents
                    .used_size_as_if_inline_element(containing_block, &style, &pbm)
                    .map(|size| Size::Numeric(*size));
                (used_size, Default::default(), Default::default())
            },
            IndependentFormattingContext::NonReplaced(_) => (
                style.content_box_size(containing_block, &pbm),
                style.content_min_box_size(containing_block, &pbm),
                style.content_max_box_size(containing_block, &pbm),
            ),
        };

        let shared_fragment = self.fragment.borrow();
        let static_position_rect = shared_fragment
            .static_position_rect
            .to_logical(containing_block);

        let box_offset = style.box_offsets(containing_block);

        // When the "static-position rect" doesn't come into play, we do not do any alignment
        // in the inline axis.
        let inline_box_offsets = AbsoluteBoxOffsets {
            start: box_offset.inline_start,
            end: box_offset.inline_end,
        };
        let inline_alignment = match inline_box_offsets.either_specified() {
            true => AlignFlags::START | AlignFlags::SAFE,
            false => shared_fragment.resolved_alignment.inline,
        };

        let mut inline_axis_solver = AbsoluteAxisSolver {
            axis: AxisDirection::Inline,
            containing_size: cbis,
            padding_border_sum: pbm.padding_border_sums.inline,
            computed_margin_start: pbm.margin.inline_start,
            computed_margin_end: pbm.margin.inline_end,
            computed_size: computed_size.inline,
            computed_min_size: computed_min_size.inline,
            computed_max_size: computed_max_size.inline,
            avoid_negative_margin_start: true,
            box_offsets: inline_box_offsets,
            static_position_rect_axis: static_position_rect.get_axis(AxisDirection::Inline),
            alignment: inline_alignment,
            flip_anchor: shared_fragment.original_parent_writing_mode.is_bidi_ltr() !=
                containing_block_writing_mode.is_bidi_ltr(),
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
            axis: AxisDirection::Block,
            containing_size: cbbs,
            padding_border_sum: pbm.padding_border_sums.block,
            computed_margin_start: pbm.margin.block_start,
            computed_margin_end: pbm.margin.block_end,
            computed_size: computed_size.block,
            computed_min_size: computed_min_size.block,
            computed_max_size: computed_max_size.block,
            avoid_negative_margin_start: false,
            box_offsets: block_box_offsets,
            static_position_rect_axis: static_position_rect.get_axis(AxisDirection::Block),
            alignment: block_alignment,
            flip_anchor: false,
        };

        // The block axis can depend on layout results, so we only solve it tentatively,
        // we may have to resolve it properly later on.
        let mut block_axis = block_axis_solver.solve_tentatively();

        // The inline axis can be fully resolved, computing intrinsic sizes using the
        // tentative block size.
        let mut inline_axis = inline_axis_solver.solve(Some(|| {
            let containing_block_for_children =
                IndefiniteContainingBlock::new_for_style_and_block_size(&style, block_axis.size);
            context
                .inline_content_sizes(
                    layout_context,
                    &containing_block_for_children,
                    &containing_block.into(),
                )
                .sizes
        }));

        let mut positioning_context = PositioningContext::new_for_style(&style).unwrap();
        let mut new_fragment = {
            let content_size: LogicalVec2<Au>;
            let fragments;
            match context {
                IndependentFormattingContext::Replaced(replaced) => {
                    // https://drafts.csswg.org/css2/visudet.html#abs-replaced-width
                    // https://drafts.csswg.org/css2/visudet.html#abs-replaced-height
                    content_size = computed_size.map(|size| size.to_numeric().unwrap());
                    fragments = replaced.contents.make_fragments(
                        &style,
                        containing_block,
                        content_size.to_physical_size(containing_block_writing_mode),
                    );
                },
                IndependentFormattingContext::NonReplaced(non_replaced) => {
                    // https://drafts.csswg.org/css2/visudet.html#abs-non-replaced-width
                    // https://drafts.csswg.org/css2/visudet.html#abs-non-replaced-height
                    let inline_size = inline_axis.size.non_auto().unwrap();
                    let containing_block_for_children = ContainingBlock {
                        inline_size,
                        block_size: block_axis.size,
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

                    let (block_size, inline_size) = match independent_layout
                        .content_inline_size_for_table
                    {
                        Some(table_inline_size) => {
                            // Tables can override their sizes regardless of the sizing properties,
                            // so we may need to solve again to update margins.
                            if inline_size != table_inline_size {
                                inline_axis = inline_axis_solver.solve_with_size(table_inline_size);
                            }
                            let table_block_size = independent_layout.content_block_size;
                            if block_axis.size != AuOrAuto::LengthPercentage(table_block_size) {
                                block_axis = block_axis_solver.solve_with_size(table_block_size);
                            }
                            (table_block_size, table_inline_size)
                        },
                        None => {
                            // Now we can properly solve the block size.
                            block_axis = block_axis_solver
                                .solve(Some(|| independent_layout.content_block_size.into()));
                            (block_axis.size.non_auto().unwrap(), inline_size)
                        },
                    };

                    content_size = LogicalVec2 {
                        inline: inline_size,
                        block: block_size,
                    };
                    fragments = independent_layout.fragments;
                },
            };

            let margin = LogicalSides {
                inline_start: inline_axis.margin_start,
                inline_end: inline_axis.margin_end,
                block_start: block_axis.margin_start,
                block_end: block_axis.margin_end,
            };

            let pb = pbm.padding + pbm.border;
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

            let mut content_rect = LogicalRect {
                start_corner: LogicalVec2 {
                    inline: inline_start,
                    block: block_start,
                },
                size: content_size,
            };

            let margin_box_rect = content_rect
                .inflate(&pbm.padding)
                .inflate(&pbm.border)
                .inflate(&margin);
            block_axis_solver.solve_alignment(margin_box_rect, &mut content_rect);
            inline_axis_solver.solve_alignment(margin_box_rect, &mut content_rect);

            BoxFragment::new(
                context.base_fragment_info(),
                style,
                fragments,
                content_rect.to_physical(Some(containing_block)),
                pbm.padding.to_physical(containing_block_writing_mode),
                pbm.border.to_physical(containing_block_writing_mode),
                margin.to_physical(containing_block_writing_mode),
                None, /* clearance */
                // We do not set the baseline offset, because absolutely positioned
                // elements are not inflow.
                CollapsedBlockMargins::zero(),
            )
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

        new_fragment
    }
}

#[derive(Clone, Copy)]
struct RectAxis {
    origin: Au,
    length: Au,
}

impl LogicalRect<Au> {
    fn get_axis(&self, axis: AxisDirection) -> RectAxis {
        match axis {
            AxisDirection::Block => RectAxis {
                origin: self.start_corner.block,
                length: self.size.block,
            },
            AxisDirection::Inline => RectAxis {
                origin: self.start_corner.inline,
                length: self.size.inline,
            },
        }
    }
}

#[derive(Debug)]
struct AbsoluteBoxOffsets<'a> {
    start: LengthPercentageOrAuto<'a>,
    end: LengthPercentageOrAuto<'a>,
}

impl AbsoluteBoxOffsets<'_> {
    pub(crate) fn either_specified(&self) -> bool {
        !self.start.is_auto() || !self.end.is_auto()
    }
}

enum Anchor {
    Start(Au),
    End(Au),
}

impl Anchor {
    fn inset(&self) -> Au {
        match self {
            Self::Start(start) => *start,
            Self::End(end) => *end,
        }
    }
}

struct AxisResult {
    anchor: Anchor,
    size: AuOrAuto,
    margin_start: Au,
    margin_end: Au,
}

struct AbsoluteAxisSolver<'a> {
    axis: AxisDirection,
    containing_size: Au,
    padding_border_sum: Au,
    computed_margin_start: AuOrAuto,
    computed_margin_end: AuOrAuto,
    computed_size: Size<Au>,
    computed_min_size: Size<Au>,
    computed_max_size: Size<Au>,
    avoid_negative_margin_start: bool,
    box_offsets: AbsoluteBoxOffsets<'a>,
    static_position_rect_axis: RectAxis,
    alignment: AlignFlags,
    flip_anchor: bool,
}

impl<'a> AbsoluteAxisSolver<'a> {
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
        let mut get_content_size = get_content_size.map(|get_content_size| {
            // The provided `get_content_size` is a FnOnce but we may need its result multiple times.
            // A LazyCell will only invoke it once if needed, and then reuse the result.
            let content_size = LazyCell::new(get_content_size);
            move || *content_size
        });
        let mut solve_size = |initial_behavior, stretch_size: Au| {
            let initial_is_stretch = initial_behavior == Size::Stretch;
            let stretch_size = stretch_size.max(Au::zero());
            get_content_size
                .as_mut()
                .map(|mut get_content_size| {
                    let min_size = self
                        .computed_min_size
                        .resolve_non_initial(stretch_size, &mut get_content_size)
                        .unwrap_or_default();
                    let max_size = self
                        .computed_max_size
                        .resolve_non_initial(stretch_size, &mut get_content_size);
                    self.computed_size
                        .resolve(initial_behavior, stretch_size, &mut get_content_size)
                        .clamp_between_extremums(min_size, max_size)
                })
                .or_else(|| {
                    self.computed_size
                        .maybe_resolve_extrinsic(Some(stretch_size))
                        .or(initial_is_stretch.then_some(stretch_size))
                        .map(|size| {
                            let min_size = self
                                .computed_min_size
                                .maybe_resolve_extrinsic(Some(stretch_size))
                                .unwrap_or_default();
                            let max_size = self
                                .computed_max_size
                                .maybe_resolve_extrinsic(Some(stretch_size));
                            size.clamp_between_extremums(min_size, max_size)
                        })
                })
        };
        let mut solve_for_anchor = |anchor: Anchor| {
            let margin_start = self.computed_margin_start.auto_is(Au::zero);
            let margin_end = self.computed_margin_end.auto_is(Au::zero);
            let stretch_size = self.containing_size -
                anchor.inset() -
                self.padding_border_sum -
                margin_start -
                margin_end;
            let size = solve_size(Size::FitContent, stretch_size)
                .map_or(AuOrAuto::Auto, AuOrAuto::LengthPercentage);
            AxisResult {
                anchor,
                size,
                margin_start,
                margin_end,
            }
        };
        match (
            self.box_offsets.start.non_auto(),
            self.box_offsets.end.non_auto(),
        ) {
            (None, None) => solve_for_anchor(if self.flip_anchor {
                Anchor::End(self.containing_size - self.static_position_rect_axis.origin)
            } else {
                Anchor::Start(self.static_position_rect_axis.origin)
            }),
            (Some(start), None) => {
                solve_for_anchor(Anchor::Start(start.to_used_value(self.containing_size)))
            },
            (None, Some(end)) => {
                solve_for_anchor(Anchor::End(end.to_used_value(self.containing_size)))
            },
            (Some(start), Some(end)) => {
                let start = start.to_used_value(self.containing_size);
                let end = end.to_used_value(self.containing_size);
                let mut free_space = self.containing_size - start - end - self.padding_border_sum;
                let stretch_size = free_space -
                    self.computed_margin_start.auto_is(Au::zero) -
                    self.computed_margin_end.auto_is(Au::zero);
                let used_size = solve_size(Size::Stretch, stretch_size).unwrap();
                free_space -= used_size;
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
                        (AuOrAuto::Auto, AuOrAuto::LengthPercentage(end)) => {
                            (free_space - end, end)
                        },
                        (AuOrAuto::LengthPercentage(start), AuOrAuto::Auto) => {
                            (start, free_space - start)
                        },
                        (AuOrAuto::LengthPercentage(start), AuOrAuto::LengthPercentage(end)) => {
                            (start, end)
                        },
                    };
                AxisResult {
                    anchor: Anchor::Start(start),
                    size: AuOrAuto::LengthPercentage(used_size),
                    margin_start,
                    margin_end,
                }
            },
        }
    }

    fn solve_tentatively(&mut self) -> AxisResult {
        self.solve(None::<fn() -> ContentSizes>)
    }

    fn solve_with_size(&mut self, size: Au) -> AxisResult {
        // Override sizes
        let old_size = mem::replace(&mut self.computed_size, Size::Numeric(size));
        let old_min_size = mem::take(&mut self.computed_min_size);
        let old_max_size = mem::take(&mut self.computed_max_size);

        let result = self.solve_tentatively();

        // Restore original sizes
        self.computed_size = old_size;
        self.computed_min_size = old_min_size;
        self.computed_max_size = old_max_size;

        result
    }

    fn origin_for_alignment_or_justification(&self, margin_box_axis: RectAxis) -> Option<Au> {
        let alignment_container = match (
            self.box_offsets.start.non_auto(),
            self.box_offsets.end.non_auto(),
        ) {
            (None, None) => self.static_position_rect_axis,
            (Some(start), Some(end)) => {
                let start = start.to_used_value(self.containing_size);
                let end = end.to_used_value(self.containing_size);

                RectAxis {
                    origin: start,
                    length: self.containing_size - (end + start),
                }
            },
            _ => return None,
        };

        let mut value_after_safety = self.alignment.value();
        if self.alignment.flags() == AlignFlags::SAFE &&
            margin_box_axis.length > alignment_container.length
        {
            value_after_safety = AlignFlags::START;
        }

        match value_after_safety {
            AlignFlags::CENTER | AlignFlags::SPACE_AROUND | AlignFlags::SPACE_EVENLY => Some(
                alignment_container.origin +
                    ((alignment_container.length - margin_box_axis.length) / 2),
            ),
            AlignFlags::FLEX_END | AlignFlags::END => Some(
                alignment_container.origin + alignment_container.length - margin_box_axis.length,
            ),
            _ => None,
        }
    }

    fn solve_alignment(
        &self,
        margin_box_rect: LogicalRect<Au>,
        content_box_rect: &mut LogicalRect<Au>,
    ) {
        let Some(new_origin) =
            self.origin_for_alignment_or_justification(margin_box_rect.get_axis(self.axis))
        else {
            return;
        };

        match self.axis {
            AxisDirection::Block => {
                content_box_rect.start_corner.block +=
                    new_origin - margin_box_rect.start_corner.block
            },
            AxisDirection::Inline => {
                content_box_rect.start_corner.inline +=
                    new_origin - margin_box_rect.start_corner.inline
            },
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

/// <https://drafts.csswg.org/css2/visuren.html#relative-positioning>
pub(crate) fn relative_adjustement(
    style: &ComputedValues,
    containing_block: &ContainingBlock,
) -> LogicalVec2<Au> {
    // It's not completely clear what to do with indefinite percentages
    // (https://github.com/w3c/csswg-drafts/issues/9353), so we match
    // other browsers and treat them as 'auto' offsets.
    let cbis = containing_block.inline_size;
    let cbbs = containing_block.block_size;
    let box_offsets = style
        .box_offsets(containing_block)
        .map_inline_and_block_axes(
            |value| value.map(|value| value.to_used_value(cbis)),
            |value| match cbbs.non_auto() {
                Some(cbbs) => value.map(|value| value.to_used_value(cbbs)),
                None => match value.non_auto().and_then(|value| value.to_length()) {
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
