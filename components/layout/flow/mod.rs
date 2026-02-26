/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
#![allow(rustdoc::private_intra_doc_links)]

//! Flow layout, also known as block-and-inline layout.

use app_units::{Au, MAX_AU};
use inline::InlineFormattingContext;
use layout_api::wrapper_traits::ThreadSafeLayoutNode;
use malloc_size_of_derive::MallocSizeOf;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use script::layout_dom::ServoThreadSafeLayoutNode;
use servo_arc::Arc;
use style::Zero;
use style::computed_values::clear::T as StyleClear;
use style::context::SharedStyleContext;
use style::logical_geometry::Direction;
use style::properties::ComputedValues;
use style::servo::selector_parser::PseudoElement;
use style::values::specified::align::AlignFlags;
use style::values::specified::{Display, TextAlignKeyword};

use crate::cell::ArcRefCell;
use crate::context::LayoutContext;
use crate::dom::WeakLayoutBox;
use crate::flow::float::{
    Clear, ContainingBlockPositionInfo, FloatBox, FloatSide, PlacementAmongFloats,
    SequentialLayoutState,
};
use crate::formatting_contexts::{Baselines, IndependentFormattingContext};
use crate::fragment_tree::{
    BaseFragmentInfo, BlockLevelLayoutInfo, BoxFragment, CollapsedBlockMargins, CollapsedMargin,
    Fragment, FragmentFlags,
};
use crate::geom::{
    AuOrAuto, LogicalRect, LogicalSides, LogicalSides1D, LogicalVec2, PhysicalPoint, PhysicalRect,
    PhysicalSides, ToLogical, ToLogicalWithContainingBlock,
};
use crate::layout_box_base::{CacheableLayoutResult, LayoutBoxBase};
use crate::positioned::{AbsolutelyPositionedBox, PositioningContext, PositioningContextLength};
use crate::sizing::{
    self, ComputeInlineContentSizes, ContentSizes, InlineContentSizesResult, LazySize, Size,
    SizeConstraint, Sizes,
};
use crate::style_ext::{AspectRatio, ContentBoxSizesAndPBM, LayoutStyle, PaddingBorderMargin};
use crate::{ConstraintSpace, ContainingBlock, ContainingBlockSize, IndefiniteContainingBlock};

mod construct;
pub mod float;
pub mod inline;
mod root;

pub(crate) use construct::{BlockContainerBuilder, BlockLevelCreator};
pub(crate) use root::BoxTree;

#[derive(Debug, MallocSizeOf)]
pub(crate) struct BlockFormattingContext {
    pub contents: BlockContainer,
    pub contains_floats: bool,
}

#[derive(Debug, MallocSizeOf)]
pub(crate) enum BlockContainer {
    BlockLevelBoxes(Vec<ArcRefCell<BlockLevelBox>>),
    InlineFormattingContext(InlineFormattingContext),
}

impl BlockContainer {
    fn contains_floats(&self) -> bool {
        match self {
            BlockContainer::BlockLevelBoxes(boxes) => boxes
                .iter()
                .any(|block_level_box| block_level_box.borrow().contains_floats()),
            BlockContainer::InlineFormattingContext(context) => context.contains_floats,
        }
    }

    pub(crate) fn repair_style(
        &mut self,
        context: &SharedStyleContext,
        node: &ServoThreadSafeLayoutNode,
        new_style: &Arc<ComputedValues>,
    ) {
        match self {
            BlockContainer::BlockLevelBoxes(..) => {},
            BlockContainer::InlineFormattingContext(inline_formatting_context) => {
                inline_formatting_context.repair_style(context, node, new_style)
            },
        }
    }
}

#[derive(Debug, MallocSizeOf)]
pub(crate) enum BlockLevelBox {
    Independent(IndependentFormattingContext),
    OutOfFlowAbsolutelyPositionedBox(ArcRefCell<AbsolutelyPositionedBox>),
    OutOfFlowFloatBox(FloatBox),
    OutsideMarker(OutsideMarker),
    SameFormattingContextBlock {
        base: LayoutBoxBase,
        contents: BlockContainer,
        contains_floats: bool,
    },
}

impl BlockLevelBox {
    pub(crate) fn repair_style(
        &mut self,
        context: &SharedStyleContext,
        node: &ServoThreadSafeLayoutNode,
        new_style: &Arc<ComputedValues>,
    ) {
        match self {
            BlockLevelBox::Independent(independent_formatting_context) => {
                independent_formatting_context.repair_style(context, node, new_style)
            },
            BlockLevelBox::OutOfFlowAbsolutelyPositionedBox(positioned_box) => positioned_box
                .borrow_mut()
                .context
                .repair_style(context, node, new_style),
            BlockLevelBox::OutOfFlowFloatBox(float_box) => {
                float_box.contents.repair_style(context, node, new_style)
            },
            BlockLevelBox::OutsideMarker(outside_marker) => {
                outside_marker.repair_style(context, node, new_style)
            },
            BlockLevelBox::SameFormattingContextBlock { base, contents, .. } => {
                base.repair_style(new_style);
                contents.repair_style(context, node, new_style);
            },
        }
    }

    pub(crate) fn with_base<T>(&self, callback: impl FnOnce(&LayoutBoxBase) -> T) -> T {
        match self {
            BlockLevelBox::Independent(independent_formatting_context) => {
                callback(&independent_formatting_context.base)
            },
            BlockLevelBox::OutOfFlowAbsolutelyPositionedBox(positioned_box) => {
                callback(&positioned_box.borrow().context.base)
            },
            BlockLevelBox::OutOfFlowFloatBox(float_box) => callback(&float_box.contents.base),
            BlockLevelBox::OutsideMarker(outside_marker) => callback(&outside_marker.context.base),
            BlockLevelBox::SameFormattingContextBlock { base, .. } => callback(base),
        }
    }

    pub(crate) fn with_base_mut<T>(&mut self, callback: impl FnOnce(&mut LayoutBoxBase) -> T) -> T {
        match self {
            BlockLevelBox::Independent(independent_formatting_context) => {
                callback(&mut independent_formatting_context.base)
            },
            BlockLevelBox::OutOfFlowAbsolutelyPositionedBox(positioned_box) => {
                callback(&mut positioned_box.borrow_mut().context.base)
            },
            BlockLevelBox::OutOfFlowFloatBox(float_box) => callback(&mut float_box.contents.base),
            BlockLevelBox::OutsideMarker(outside_marker) => {
                callback(&mut outside_marker.context.base)
            },
            BlockLevelBox::SameFormattingContextBlock { base, .. } => callback(base),
        }
    }

    pub(crate) fn attached_to_tree(&self, layout_box: WeakLayoutBox) {
        match self {
            Self::Independent(independent_formatting_context) => {
                independent_formatting_context.attached_to_tree(layout_box)
            },
            Self::OutOfFlowAbsolutelyPositionedBox(positioned_box) => {
                positioned_box.borrow().context.attached_to_tree(layout_box)
            },
            Self::OutOfFlowFloatBox(float_box) => float_box.contents.attached_to_tree(layout_box),
            Self::OutsideMarker(outside_marker) => {
                outside_marker.context.attached_to_tree(layout_box)
            },
            Self::SameFormattingContextBlock { contents, .. } => {
                contents.attached_to_tree(layout_box)
            },
        }
    }

    fn contains_floats(&self) -> bool {
        match self {
            BlockLevelBox::SameFormattingContextBlock {
                contains_floats, ..
            } => *contains_floats,
            BlockLevelBox::OutOfFlowFloatBox { .. } => true,
            _ => false,
        }
    }

    fn find_block_margin_collapsing_with_parent(
        &self,
        layout_context: &LayoutContext,
        collected_margin: &mut CollapsedMargin,
        containing_block: &ContainingBlock,
    ) -> bool {
        let layout_style = match self {
            BlockLevelBox::SameFormattingContextBlock { base, contents, .. } => {
                contents.layout_style(base)
            },
            BlockLevelBox::OutOfFlowAbsolutelyPositionedBox(_) |
            BlockLevelBox::OutOfFlowFloatBox(_) => return true,
            BlockLevelBox::OutsideMarker(_) => return false,
            BlockLevelBox::Independent(context) => {
                // FIXME: If the element doesn't fit next to floats, it will get clearance.
                // In that case this should be returning false.
                context.layout_style()
            },
        };

        // FIXME: This should only return false when 'clear' causes clearance.
        let style = layout_style.style();
        if style.get_box().clear != StyleClear::None {
            return false;
        }

        let ContentBoxSizesAndPBM {
            content_box_sizes,
            pbm,
            ..
        } = layout_style.content_box_sizes_and_padding_border_margin(&containing_block.into());
        let margin = pbm.margin.auto_is(Au::zero);
        collected_margin.adjoin_assign(&CollapsedMargin::new(margin.block_start));

        let BlockLevelBox::SameFormattingContextBlock { contents, .. } = self else {
            return false;
        };

        if !pbm.padding.block_start.is_zero() || !pbm.border.block_start.is_zero() {
            return false;
        }

        let available_inline_size =
            containing_block.size.inline - pbm.padding_border_sums.inline - margin.inline_sum();
        let available_block_size = containing_block.size.block.to_definite().map(|block_size| {
            Au::zero().max(block_size - pbm.padding_border_sums.block - margin.block_sum())
        });

        let tentative_block_size = content_box_sizes.block.resolve_extrinsic(
            Size::FitContent,
            Au::zero(),
            available_block_size,
        );

        let get_inline_content_sizes = || {
            let constraint_space = ConstraintSpace::new(
                tentative_block_size,
                style,
                None, /* TODO: support preferred aspect ratios on non-replaced boxes */
            );
            self.inline_content_sizes(layout_context, &constraint_space)
                .sizes
        };
        let inline_size = content_box_sizes.inline.resolve(
            Direction::Inline,
            Size::Stretch,
            Au::zero,
            Some(available_inline_size),
            get_inline_content_sizes,
            false, /* is_table */
        );

        let containing_block_for_children = ContainingBlock {
            size: ContainingBlockSize {
                inline: inline_size,
                block: tentative_block_size,
            },
            style,
        };

        if !contents.find_block_margin_collapsing_with_parent(
            layout_context,
            collected_margin,
            &containing_block_for_children,
        ) {
            return false;
        }

        if !tentative_block_size.definite_or_min().is_zero() ||
            !pbm.padding_border_sums.block.is_zero()
        {
            return false;
        }

        collected_margin.adjoin_assign(&CollapsedMargin::new(margin.block_end));

        true
    }
}

#[derive(Clone, Copy)]
pub(crate) struct CollapsibleWithParentStartMargin(bool);

/// The contentes of a BlockContainer created to render a list marker
/// for a list that has `list-style-position: outside`.
#[derive(Debug, MallocSizeOf)]
pub(crate) struct OutsideMarker {
    pub list_item_style: Arc<ComputedValues>,
    pub context: IndependentFormattingContext,
}

impl OutsideMarker {
    fn layout(
        &self,
        layout_context: &LayoutContext<'_>,
        containing_block: &ContainingBlock<'_>,
        positioning_context: &mut PositioningContext,
    ) -> Fragment {
        let style = &self.context.base.style;
        let preferred_aspect_ratio = self.context.preferred_aspect_ratio(&LogicalVec2::zero());
        let constraint_space =
            ConstraintSpace::new(SizeConstraint::default(), style, preferred_aspect_ratio);
        let content_sizes = self
            .context
            .inline_content_sizes(layout_context, &constraint_space);
        let containing_block_for_children = ContainingBlock {
            size: ContainingBlockSize {
                inline: content_sizes.sizes.max_content,
                block: SizeConstraint::default(),
            },
            style,
        };

        let layout = self.context.layout(
            layout_context,
            positioning_context,
            &containing_block_for_children,
            containing_block,
            preferred_aspect_ratio,
            &LazySize::intrinsic(),
        );

        let max_inline_size = layout
            .fragments
            .iter()
            .map(|fragment| {
                fragment
                    .base()
                    .map(|base| base.rect)
                    .unwrap_or_default()
                    .to_logical(&containing_block_for_children)
                    .max_inline_position()
            })
            .max()
            .unwrap_or_default();

        // Position the marker beyond the inline start of the border box list item. This needs to
        // take into account the border and padding of the item.
        //
        // TODO: This is the wrong containing block, as it should be the containing block of the
        // parent of this list item. What this means in practice is that the writing mode could be
        // wrong and padding defined as a percentage will be resolved incorrectly.
        //
        // TODO: This should use the LayoutStyle of the list item, not the default one. Currently
        // they are the same, but this could change in the future.
        let pbm_of_list_item =
            LayoutStyle::Default(&self.list_item_style).padding_border_margin(containing_block);
        let content_rect = LogicalRect {
            start_corner: LogicalVec2 {
                inline: -max_inline_size -
                    (pbm_of_list_item.border.inline_start +
                        pbm_of_list_item.padding.inline_start),
                block: Zero::zero(),
            },
            size: LogicalVec2 {
                inline: max_inline_size,
                block: layout.content_block_size,
            },
        };

        let mut base_fragment_info = BaseFragmentInfo::anonymous();
        base_fragment_info.flags |= FragmentFlags::IS_OUTSIDE_LIST_ITEM_MARKER;

        Fragment::Box(ArcRefCell::new(BoxFragment::new(
            base_fragment_info,
            style.clone(),
            layout.fragments,
            content_rect.as_physical(Some(containing_block)),
            PhysicalSides::zero(),
            PhysicalSides::zero(),
            PhysicalSides::zero(),
            layout.specific_layout_info,
        )))
    }

    fn repair_style(
        &mut self,
        context: &SharedStyleContext,
        node: &ServoThreadSafeLayoutNode,
        new_style: &Arc<ComputedValues>,
    ) {
        self.list_item_style = node.parent_style(context);
        self.context.repair_style(context, node, new_style);
    }
}

impl BlockFormattingContext {
    pub(super) fn layout(
        &self,
        layout_context: &LayoutContext,
        positioning_context: &mut PositioningContext,
        containing_block: &ContainingBlock,
    ) -> CacheableLayoutResult {
        let mut sequential_layout_state = if self.contains_floats || !layout_context.use_rayon {
            Some(SequentialLayoutState::new(containing_block.size.inline))
        } else {
            None
        };

        // Since this is an independent formatting context, we don't ignore block margins when
        // resolving a stretch block size of the children.
        // https://drafts.csswg.org/css-sizing-4/#stretch-fit-sizing
        let ignore_block_margins_for_stretch = LogicalSides1D::new(false, false);

        let flow_layout = self.contents.layout(
            layout_context,
            positioning_context,
            containing_block,
            sequential_layout_state.as_mut(),
            CollapsibleWithParentStartMargin(false),
            ignore_block_margins_for_stretch,
        );
        debug_assert!(
            !flow_layout
                .collapsible_margins_in_children
                .collapsed_through
        );

        // The content height of a BFC root should include any float participating in that BFC
        // (https://drafts.csswg.org/css2/#root-height), we implement this by imagining there is
        // an element with `clear: both` after the actual contents.
        let clearance = sequential_layout_state.and_then(|sequential_layout_state| {
            sequential_layout_state.calculate_clearance(Clear::Both, &CollapsedMargin::zero())
        });

        CacheableLayoutResult {
            fragments: flow_layout.fragments,
            content_block_size: flow_layout.content_block_size +
                flow_layout.collapsible_margins_in_children.end.solve() +
                clearance.unwrap_or_default(),
            content_inline_size_for_table: None,
            baselines: flow_layout.baselines,
            depends_on_block_constraints: flow_layout.depends_on_block_constraints,
            specific_layout_info: None,
            collapsible_margins_in_children: CollapsedBlockMargins::zero(),
        }
    }

    #[inline]
    pub(crate) fn layout_style<'a>(&self, base: &'a LayoutBoxBase) -> LayoutStyle<'a> {
        LayoutStyle::Default(&base.style)
    }

    pub(crate) fn repair_style(
        &mut self,
        context: &SharedStyleContext,
        node: &ServoThreadSafeLayoutNode,
        new_style: &Arc<ComputedValues>,
    ) {
        self.contents.repair_style(context, node, new_style);
    }

    pub(crate) fn attached_to_tree(&self, layout_box: WeakLayoutBox) {
        self.contents.attached_to_tree(layout_box);
    }
}

/// Finds the min/max-content inline size of the block-level children of a block container.
/// The in-flow boxes will stack vertically, so we only need to consider the maximum size.
/// But floats can flow horizontally depending on 'clear', so we may need to sum their sizes.
/// CSS 2 does not define the exact algorithm, this logic is based on the behavior observed
/// on Gecko and Blink.
fn compute_inline_content_sizes_for_block_level_boxes(
    boxes: &[ArcRefCell<BlockLevelBox>],
    layout_context: &LayoutContext,
    containing_block: &IndefiniteContainingBlock,
) -> InlineContentSizesResult {
    let get_box_info = |box_: &ArcRefCell<BlockLevelBox>| {
        match &*box_.borrow() {
            BlockLevelBox::OutOfFlowAbsolutelyPositionedBox(_) |
            BlockLevelBox::OutsideMarker { .. } => None,
            BlockLevelBox::OutOfFlowFloatBox(float_box) => {
                let inline_content_sizes_result = float_box.contents.outer_inline_content_sizes(
                    layout_context,
                    containing_block,
                    &LogicalVec2::zero(),
                    false, /* auto_block_size_stretches_to_containing_block */
                );
                let style = &float_box.contents.style();
                let container_writing_mode = containing_block.style.writing_mode;
                Some((
                    inline_content_sizes_result,
                    FloatSide::from_style_and_container_writing_mode(style, container_writing_mode),
                    Clear::from_style_and_container_writing_mode(style, container_writing_mode),
                ))
            },
            BlockLevelBox::SameFormattingContextBlock { base, contents, .. } => {
                let is_anonymous_block =
                    matches!(base.style.pseudo(), Some(PseudoElement::ServoAnonymousBox));
                let inline_content_sizes_result = sizing::outer_inline(
                    base,
                    &contents.layout_style(base),
                    containing_block,
                    &LogicalVec2::zero(),
                    false,               /* auto_block_size_stretches_to_containing_block */
                    false,               /* is_replaced */
                    !is_anonymous_block, /* establishes_containing_block */
                    |_| None, /* TODO: support preferred aspect ratios on non-replaced boxes */
                    |constraint_space| {
                        base.inline_content_sizes(layout_context, constraint_space, contents)
                    },
                    |_aspect_ratio| None,
                );
                // A block in the same BFC can overlap floats, it's not moved next to them,
                // so we shouldn't add its size to the size of the floats.
                // Instead, we treat it like an independent block with 'clear: both',
                // except if it's an anonymous block.
                // Presumably, the exception is because an anonymous block will always have
                // inline-level contents, which don't overlap floats. However, the same might
                // also happen with a non-anonymous block, so the logic is a bit arbitrary,
                // but matches other browsers (see #41280).
                let clear = if is_anonymous_block {
                    Clear::None
                } else {
                    Clear::Both
                };
                Some((inline_content_sizes_result, None, clear))
            },
            BlockLevelBox::Independent(independent) => {
                let inline_content_sizes_result = independent.outer_inline_content_sizes(
                    layout_context,
                    containing_block,
                    &LogicalVec2::zero(),
                    false, /* auto_block_size_stretches_to_containing_block */
                );
                Some((
                    inline_content_sizes_result,
                    None,
                    Clear::from_style_and_container_writing_mode(
                        independent.style(),
                        containing_block.style.writing_mode,
                    ),
                ))
            },
        }
    };

    /// When iterating the block-level boxes to compute the inline content sizes,
    /// this struct contains the data accumulated up to the current box.
    #[derive(Default)]
    struct AccumulatedData {
        /// Whether the inline size depends on the block one.
        depends_on_block_constraints: bool,
        /// The maximum size seen so far, not including trailing uncleared floats.
        max_size: ContentSizes,
        /// The size of the trailing uncleared floats on the inline-start and
        /// inline-end sides of the containing block.
        floats: LogicalSides1D<ContentSizes>,
    }

    impl AccumulatedData {
        fn max_size_including_uncleared_floats(&self) -> ContentSizes {
            self.max_size.max(self.floats.start.union(&self.floats.end))
        }
        fn clear_floats(&mut self, clear: Clear) {
            match clear {
                Clear::InlineStart => {
                    self.max_size = self.max_size_including_uncleared_floats();
                    self.floats.start = ContentSizes::default();
                },
                Clear::InlineEnd => {
                    self.max_size = self.max_size_including_uncleared_floats();
                    self.floats.end = ContentSizes::default();
                },
                Clear::Both => {
                    self.max_size = self.max_size_including_uncleared_floats();
                    self.floats = LogicalSides1D::default();
                },
                Clear::None => {},
            };
        }
    }

    let accumulate =
        |mut data: AccumulatedData,
         (inline_content_sizes_result, float, clear): (InlineContentSizesResult, _, _)| {
            let size = inline_content_sizes_result.sizes.max(ContentSizes::zero());
            let depends_on_block_constraints =
                inline_content_sizes_result.depends_on_block_constraints;
            data.depends_on_block_constraints |= depends_on_block_constraints;
            data.clear_floats(clear);
            match float {
                Some(FloatSide::InlineStart) => data.floats.start.union_assign(&size),
                Some(FloatSide::InlineEnd) => data.floats.end.union_assign(&size),
                None => {
                    data.max_size
                        .max_assign(data.floats.start.union(&data.floats.end).union(&size));
                    data.floats = LogicalSides1D::default();
                },
            }
            data
        };
    let data = if layout_context.use_rayon {
        boxes
            .par_iter()
            .filter_map(get_box_info)
            .collect::<Vec<_>>()
            .into_iter()
            .fold(AccumulatedData::default(), accumulate)
    } else {
        boxes
            .iter()
            .filter_map(get_box_info)
            .fold(AccumulatedData::default(), accumulate)
    };
    InlineContentSizesResult {
        depends_on_block_constraints: data.depends_on_block_constraints,
        sizes: data.max_size_including_uncleared_floats(),
    }
}

impl BlockContainer {
    fn layout(
        &self,
        layout_context: &LayoutContext,
        positioning_context: &mut PositioningContext,
        containing_block: &ContainingBlock,
        sequential_layout_state: Option<&mut SequentialLayoutState>,
        collapsible_with_parent_start_margin: CollapsibleWithParentStartMargin,
        ignore_block_margins_for_stretch: LogicalSides1D<bool>,
    ) -> CacheableLayoutResult {
        match self {
            BlockContainer::BlockLevelBoxes(child_boxes) => layout_block_level_children(
                layout_context,
                positioning_context,
                child_boxes,
                containing_block,
                sequential_layout_state,
                collapsible_with_parent_start_margin,
                ignore_block_margins_for_stretch,
            ),
            BlockContainer::InlineFormattingContext(ifc) => ifc.layout(
                layout_context,
                positioning_context,
                containing_block,
                sequential_layout_state,
                collapsible_with_parent_start_margin,
            ),
        }
    }

    #[inline]
    pub(crate) fn layout_style<'a>(&self, base: &'a LayoutBoxBase) -> LayoutStyle<'a> {
        LayoutStyle::Default(&base.style)
    }

    pub(crate) fn attached_to_tree(&self, layout_box: WeakLayoutBox) {
        match self {
            Self::BlockLevelBoxes(child_boxes) => {
                for child_box in child_boxes {
                    child_box.borrow_mut().with_base_mut(|base| {
                        base.parent_box.replace(layout_box.clone());
                    });
                }
            },
            Self::InlineFormattingContext(ifc) => ifc.attached_to_tree(layout_box),
        }
    }

    fn find_block_margin_collapsing_with_parent(
        &self,
        layout_context: &LayoutContext,
        collected_margin: &mut CollapsedMargin,
        containing_block_for_children: &ContainingBlock,
    ) -> bool {
        match self {
            BlockContainer::BlockLevelBoxes(boxes) => boxes.iter().all(|block_level_box| {
                block_level_box
                    .borrow()
                    .find_block_margin_collapsing_with_parent(
                        layout_context,
                        collected_margin,
                        containing_block_for_children,
                    )
            }),
            BlockContainer::InlineFormattingContext(context) => context
                .find_block_margin_collapsing_with_parent(
                    layout_context,
                    collected_margin,
                    containing_block_for_children,
                ),
        }
    }
}

impl ComputeInlineContentSizes for BlockContainer {
    fn compute_inline_content_sizes(
        &self,
        layout_context: &LayoutContext,
        constraint_space: &ConstraintSpace,
    ) -> InlineContentSizesResult {
        match &self {
            Self::BlockLevelBoxes(boxes) => compute_inline_content_sizes_for_block_level_boxes(
                boxes,
                layout_context,
                &constraint_space.into(),
            ),
            Self::InlineFormattingContext(context) => {
                context.compute_inline_content_sizes(layout_context, constraint_space)
            },
        }
    }
}

fn layout_block_level_children(
    layout_context: &LayoutContext,
    positioning_context: &mut PositioningContext,
    child_boxes: &[ArcRefCell<BlockLevelBox>],
    containing_block: &ContainingBlock,
    mut sequential_layout_state: Option<&mut SequentialLayoutState>,
    collapsible_with_parent_start_margin: CollapsibleWithParentStartMargin,
    ignore_block_margins_for_stretch: LogicalSides1D<bool>,
) -> CacheableLayoutResult {
    let mut placement_state =
        PlacementState::new(collapsible_with_parent_start_margin, containing_block);

    let fragments = match sequential_layout_state {
        Some(ref mut sequential_layout_state) => layout_block_level_children_sequentially(
            layout_context,
            positioning_context,
            child_boxes,
            containing_block,
            sequential_layout_state,
            &mut placement_state,
            ignore_block_margins_for_stretch,
        ),
        None => layout_block_level_children_in_parallel(
            layout_context,
            positioning_context,
            child_boxes,
            containing_block,
            &mut placement_state,
            ignore_block_margins_for_stretch,
        ),
    };

    let depends_on_block_constraints = fragments.iter().any(|fragment| {
        fragment.base().is_some_and(|base| {
            base.flags.contains(
                FragmentFlags::SIZE_DEPENDS_ON_BLOCK_CONSTRAINTS_AND_CAN_BE_CHILD_OF_FLEX_ITEM,
            )
        })
    });

    let (content_block_size, collapsible_margins_in_children, baselines) = placement_state.finish();
    CacheableLayoutResult {
        fragments,
        content_block_size,
        collapsible_margins_in_children,
        baselines,
        depends_on_block_constraints,
        content_inline_size_for_table: None,
        specific_layout_info: None,
    }
}

fn layout_block_level_children_in_parallel(
    layout_context: &LayoutContext,
    positioning_context: &mut PositioningContext,
    child_boxes: &[ArcRefCell<BlockLevelBox>],
    containing_block: &ContainingBlock,
    placement_state: &mut PlacementState,
    ignore_block_margins_for_stretch: LogicalSides1D<bool>,
) -> Vec<Fragment> {
    let mut layout_results: Vec<(Fragment, PositioningContext)> =
        Vec::with_capacity(child_boxes.len());

    child_boxes
        .par_iter()
        .map(|child_box| {
            let mut child_positioning_context = PositioningContext::default();
            let fragment = child_box.borrow().layout(
                layout_context,
                &mut child_positioning_context,
                containing_block,
                /* sequential_layout_state = */ None,
                /* collapsible_with_parent_start_margin = */ None,
                ignore_block_margins_for_stretch,
            );
            (fragment, child_positioning_context)
        })
        .collect_into_vec(&mut layout_results);

    layout_results
        .into_iter()
        .map(|(mut fragment, mut child_positioning_context)| {
            placement_state.place_fragment_and_update_baseline(&mut fragment, None);
            child_positioning_context.adjust_static_position_of_hoisted_fragments(
                &fragment,
                PositioningContextLength::zero(),
            );
            positioning_context.append(child_positioning_context);
            fragment
        })
        .collect()
}

fn layout_block_level_children_sequentially(
    layout_context: &LayoutContext,
    positioning_context: &mut PositioningContext,
    child_boxes: &[ArcRefCell<BlockLevelBox>],
    containing_block: &ContainingBlock,
    sequential_layout_state: &mut SequentialLayoutState,
    placement_state: &mut PlacementState,
    ignore_block_margins_for_stretch: LogicalSides1D<bool>,
) -> Vec<Fragment> {
    // Because floats are involved, we do layout for this block formatting context in tree
    // order without parallelism. This enables mutable access to a `SequentialLayoutState` that
    // tracks every float encountered so far (again in tree order).
    child_boxes
        .iter()
        .map(|child_box| {
            let positioning_context_length_before_layout = positioning_context.len();
            let mut fragment = child_box.borrow().layout(
                layout_context,
                positioning_context,
                containing_block,
                Some(&mut *sequential_layout_state),
                Some(CollapsibleWithParentStartMargin(
                    placement_state.next_in_flow_margin_collapses_with_parent_start_margin,
                )),
                ignore_block_margins_for_stretch,
            );

            placement_state
                .place_fragment_and_update_baseline(&mut fragment, Some(sequential_layout_state));
            positioning_context.adjust_static_position_of_hoisted_fragments(
                &fragment,
                positioning_context_length_before_layout,
            );

            fragment
        })
        .collect()
}

impl BlockLevelBox {
    fn layout(
        &self,
        layout_context: &LayoutContext,
        positioning_context: &mut PositioningContext,
        containing_block: &ContainingBlock,
        sequential_layout_state: Option<&mut SequentialLayoutState>,
        collapsible_with_parent_start_margin: Option<CollapsibleWithParentStartMargin>,
        ignore_block_margins_for_stretch: LogicalSides1D<bool>,
    ) -> Fragment {
        let fragment = match self {
            BlockLevelBox::SameFormattingContextBlock { base, contents, .. } => Fragment::Box(
                ArcRefCell::new(positioning_context.layout_maybe_position_relative_fragment(
                    layout_context,
                    containing_block,
                    base,
                    |positioning_context| {
                        layout_in_flow_non_replaced_block_level_same_formatting_context(
                            layout_context,
                            positioning_context,
                            containing_block,
                            base,
                            contents,
                            sequential_layout_state,
                            collapsible_with_parent_start_margin,
                            ignore_block_margins_for_stretch,
                        )
                    },
                )),
            ),
            BlockLevelBox::Independent(independent) => Fragment::Box(ArcRefCell::new(
                positioning_context.layout_maybe_position_relative_fragment(
                    layout_context,
                    containing_block,
                    &independent.base,
                    |positioning_context| {
                        independent.layout_in_flow_block_level(
                            layout_context,
                            positioning_context,
                            containing_block,
                            sequential_layout_state,
                            ignore_block_margins_for_stretch,
                        )
                    },
                ),
            )),
            BlockLevelBox::OutOfFlowAbsolutelyPositionedBox(box_) => {
                // The static position of zero here is incorrect, however we do not know
                // the correct positioning until later, in place_block_level_fragment, and
                // this value will be adjusted there.
                let hoisted_box = AbsolutelyPositionedBox::to_hoisted(
                    box_.clone(),
                    // This is incorrect, however we do not know the correct positioning
                    // until later, in PlacementState::place_fragment, and this value will be
                    // adjusted there
                    PhysicalRect::zero(),
                    LogicalVec2 {
                        inline: AlignFlags::START,
                        block: AlignFlags::START,
                    },
                    containing_block.style.writing_mode,
                );
                let hoisted_fragment = hoisted_box.fragment.clone();
                positioning_context.push(hoisted_box);
                Fragment::AbsoluteOrFixedPositioned(hoisted_fragment)
            },
            BlockLevelBox::OutOfFlowFloatBox(float_box) => Fragment::Float(ArcRefCell::new(
                float_box.layout(layout_context, positioning_context, containing_block),
            )),
            BlockLevelBox::OutsideMarker(outside_marker) => {
                outside_marker.layout(layout_context, containing_block, positioning_context)
            },
        };

        self.with_base(|base| base.set_fragment(fragment.clone()));

        fragment
    }

    fn inline_content_sizes(
        &self,
        layout_context: &LayoutContext,
        constraint_space: &ConstraintSpace,
    ) -> InlineContentSizesResult {
        let independent_formatting_context = match self {
            BlockLevelBox::Independent(independent) => independent,
            BlockLevelBox::OutOfFlowAbsolutelyPositionedBox(box_) => &box_.borrow().context,
            BlockLevelBox::OutOfFlowFloatBox(float_box) => &float_box.contents,
            BlockLevelBox::OutsideMarker(outside_marker) => &outside_marker.context,
            BlockLevelBox::SameFormattingContextBlock { base, contents, .. } => {
                return base.inline_content_sizes(layout_context, constraint_space, contents);
            },
        };
        independent_formatting_context.inline_content_sizes(layout_context, constraint_space)
    }
}

/// Lay out a normal flow non-replaced block that does not establish a new formatting
/// context.
///
/// - <https://drafts.csswg.org/css2/visudet.html#blockwidth>
/// - <https://drafts.csswg.org/css2/visudet.html#normal-block>
#[allow(clippy::too_many_arguments)]
pub(crate) fn layout_in_flow_non_replaced_block_level_same_formatting_context(
    layout_context: &LayoutContext,
    positioning_context: &mut PositioningContext,
    containing_block: &ContainingBlock,
    base: &LayoutBoxBase,
    contents: &BlockContainer,
    mut sequential_layout_state: Option<&mut SequentialLayoutState>,
    collapsible_with_parent_start_margin: Option<CollapsibleWithParentStartMargin>,
    ignore_block_margins_for_stretch: LogicalSides1D<bool>,
) -> BoxFragment {
    let style = &base.style;
    let layout_style = contents.layout_style(base);
    let containing_block_writing_mode = containing_block.style.writing_mode;
    let get_inline_content_sizes = |constraint_space: &ConstraintSpace| {
        base.inline_content_sizes(layout_context, constraint_space, contents)
            .sizes
    };
    let ContainingBlockPaddingAndBorder {
        containing_block: containing_block_for_children,
        pbm,
        block_sizes,
        depends_on_block_constraints,
        available_block_size,
        justify_self,
        ..
    } = solve_containing_block_padding_and_border_for_in_flow_box(
        containing_block,
        &layout_style,
        get_inline_content_sizes,
        ignore_block_margins_for_stretch,
        None,
    );
    let ResolvedMargins {
        margin,
        effective_margin_inline_start,
    } = solve_margins(
        containing_block,
        &pbm,
        containing_block_for_children.size.inline,
        justify_self,
    );

    let start_margin_can_collapse_with_children =
        pbm.padding.block_start.is_zero() && pbm.border.block_start.is_zero();

    let mut clearance = None;
    let parent_containing_block_position_info;
    match sequential_layout_state {
        None => parent_containing_block_position_info = None,
        Some(ref mut sequential_layout_state) => {
            let clear =
                Clear::from_style_and_container_writing_mode(style, containing_block_writing_mode);
            let mut block_start_margin = CollapsedMargin::new(margin.block_start);

            // The block start margin may collapse with content margins,
            // compute the resulting one in order to place floats correctly.
            // Only need to do this if the element isn't also collapsing with its parent,
            // otherwise we should have already included the margin in an ancestor.
            // Note this lookahead stops when finding a descendant whose `clear` isn't `none`
            // (since clearance prevents collapsing margins with the parent).
            // But then we have to decide whether to actually add clearance or not,
            // so look forward again regardless of `collapsible_with_parent_start_margin`.
            // TODO: This isn't completely right: if we don't add actual clearance,
            // the margin should have been included in the parent (or some ancestor).
            // The lookahead should stop for actual clearance, not just for `clear`.
            let collapsible_with_parent_start_margin = collapsible_with_parent_start_margin.expect(
                "We should know whether we are collapsing the block start margin with the parent \
                when laying out sequentially",
            ).0 && clear == Clear::None;
            if !collapsible_with_parent_start_margin && start_margin_can_collapse_with_children {
                contents.find_block_margin_collapsing_with_parent(
                    layout_context,
                    &mut block_start_margin,
                    &containing_block_for_children,
                );
            }

            // Introduce clearance if necessary.
            clearance = sequential_layout_state.calculate_clearance(clear, &block_start_margin);
            if clearance.is_some() {
                sequential_layout_state.collapse_margins();
            }
            sequential_layout_state.adjoin_assign(&block_start_margin);
            if !start_margin_can_collapse_with_children {
                sequential_layout_state.collapse_margins();
            }

            // NB: This will be a no-op if we're collapsing margins with our children since that
            // can only happen if we have no block-start padding and border.
            sequential_layout_state.advance_block_position(
                pbm.padding.block_start +
                    pbm.border.block_start +
                    clearance.unwrap_or_else(Au::zero),
            );

            // We are about to lay out children. Update the offset between the block formatting
            // context and the containing block that we create for them. This offset is used to
            // ajust BFC relative coordinates to coordinates that are relative to our content box.
            // Our content box establishes the containing block for non-abspos children, including
            // floats.
            let inline_start = sequential_layout_state
                .floats
                .containing_block_info
                .inline_start +
                pbm.padding.inline_start +
                pbm.border.inline_start +
                effective_margin_inline_start;
            let new_cb_offsets = ContainingBlockPositionInfo {
                block_start: sequential_layout_state.bfc_relative_block_position,
                block_start_margins_not_collapsed: sequential_layout_state.current_margin,
                inline_start,
                inline_end: inline_start + containing_block_for_children.size.inline,
            };
            parent_containing_block_position_info = Some(
                sequential_layout_state.replace_containing_block_position_info(new_cb_offsets),
            );
        },
    };

    // https://drafts.csswg.org/css-sizing-4/#stretch-fit-sizing
    // > If this is a block axis size, and the element is in a Block Layout formatting context,
    // > and the parent element does not have a block-start border or padding and is not an
    // > independent formatting context, treat the element’s block-start margin as zero
    // > for the purpose of calculating this size. Do the same for the block-end margin.
    let ignore_block_margins_for_stretch = LogicalSides1D::new(
        pbm.border.block_start.is_zero() && pbm.padding.block_start.is_zero(),
        pbm.border.block_end.is_zero() && pbm.padding.block_end.is_zero(),
    );

    let flow_layout = contents.layout(
        layout_context,
        positioning_context,
        &containing_block_for_children,
        sequential_layout_state.as_deref_mut(),
        CollapsibleWithParentStartMargin(start_margin_can_collapse_with_children),
        ignore_block_margins_for_stretch,
    );
    let mut content_block_size = flow_layout.content_block_size;

    // Update margins.
    let mut block_margins_collapsed_with_children = CollapsedBlockMargins::from_margin(&margin);
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

    let is_anonymous = matches!(base.style.pseudo(), Some(PseudoElement::ServoAnonymousBox));
    let tentative_block_size = if is_anonymous {
        // Anonymous blocks do not establish a containing block for their children,
        // so we can't use that. However, they always have their sizing properties
        // set to their initial values, so it's fine to use the default.
        &Default::default()
    } else {
        &containing_block_for_children.size.block
    };
    let collapsed_through = collapsible_margins_in_children.collapsed_through &&
        pbm.padding_border_sums.block.is_zero() &&
        tentative_block_size.definite_or_min().is_zero();
    block_margins_collapsed_with_children.collapsed_through = collapsed_through;

    let end_margin_can_collapse_with_children =
        pbm.padding.block_end.is_zero() && pbm.border.block_end.is_zero();
    if !end_margin_can_collapse_with_children {
        content_block_size += collapsible_margins_in_children.end.solve();
    }

    let block_size = block_sizes.resolve(
        Direction::Block,
        Size::FitContent,
        Au::zero,
        available_block_size,
        || content_block_size.into(),
        false, /* is_table */
    );

    // If the final block size is different than the intrinsic size of the contents,
    // then we can't actually collapse the end margins. This can happen due to min
    // or max block sizes, or due to `calc-size()` once we implement it.
    //
    // We also require `block-size` to have an intrinsic value, by checking whether
    // the containing block established for the contents has an indefinite block size.
    // However, even if `block-size: 0px` is extrinsic (so it would normally prevent
    // collapsing the end margin with children), it doesn't prevent the top and end
    // margins from collapsing through. If that happens, allow collapsing end margins.
    //
    // This is being discussed in https://github.com/w3c/csswg-drafts/issues/12218.
    // It would probably make more sense to check the definiteness of the containing
    // block in the logic above (when we check if there is some block-end padding or
    // border), or maybe drop the condition altogether. But for now, we match Blink.
    let end_margin_can_collapse_with_children = end_margin_can_collapse_with_children &&
        block_size == content_block_size &&
        (collapsed_through || !tentative_block_size.is_definite());
    if end_margin_can_collapse_with_children {
        block_margins_collapsed_with_children
            .end
            .adjoin_assign(&collapsible_margins_in_children.end);
    }

    if let Some(ref mut sequential_layout_state) = sequential_layout_state {
        // Now that we're done laying out our children, we can restore the
        // parent's containing block position information.
        sequential_layout_state
            .replace_containing_block_position_info(parent_containing_block_position_info.unwrap());

        // Account for padding and border. We also might have to readjust the
        // `bfc_relative_block_position` if it was different from the content size (i.e. was
        // non-`auto` and/or was affected by min/max block size).
        //
        // If this adjustment is positive, that means that a block size was specified, but
        // the content inside had a smaller block size. If this adjustment is negative, a
        // block size was specified, but the content inside overflowed this container in
        // the block direction. In that case, the ceiling for floats is effectively raised
        // as long as no floats in the overflowing content lowered it.
        sequential_layout_state.advance_block_position(
            block_size - content_block_size + pbm.padding.block_end + pbm.border.block_end,
        );

        if !end_margin_can_collapse_with_children {
            sequential_layout_state.collapse_margins();
        }
        sequential_layout_state.adjoin_assign(&CollapsedMargin::new(margin.block_end));
    }

    let content_rect = LogicalRect {
        start_corner: LogicalVec2 {
            block: (pbm.padding.block_start +
                pbm.border.block_start +
                clearance.unwrap_or_else(Au::zero)),
            inline: pbm.padding.inline_start +
                pbm.border.inline_start +
                effective_margin_inline_start,
        },
        size: LogicalVec2 {
            block: block_size,
            inline: containing_block_for_children.size.inline,
        },
    };

    let mut base_fragment_info = base.base_fragment_info;

    // An anonymous block doesn't establish a containing block for its contents. Therefore,
    // if its contents depend on block constraints, its block size (which is intrinsic) also
    // depends on block constraints.
    if depends_on_block_constraints || (is_anonymous && flow_layout.depends_on_block_constraints) {
        base_fragment_info
            .flags
            .insert(FragmentFlags::SIZE_DEPENDS_ON_BLOCK_CONSTRAINTS_AND_CAN_BE_CHILD_OF_FLEX_ITEM);
    }

    BoxFragment::new(
        base_fragment_info,
        style.clone(),
        flow_layout.fragments,
        content_rect.as_physical(Some(containing_block)),
        pbm.padding.to_physical(containing_block_writing_mode),
        pbm.border.to_physical(containing_block_writing_mode),
        margin.to_physical(containing_block_writing_mode),
        flow_layout.specific_layout_info,
    )
    .with_baselines(flow_layout.baselines)
    .with_block_level_layout_info(block_margins_collapsed_with_children, clearance)
}

impl IndependentFormattingContext {
    /// Lay out an in-flow block-level box that establishes an independent
    /// formatting context in its containing formatting context.
    ///
    /// - <https://drafts.csswg.org/css2/visudet.html#blockwidth>
    /// - <https://drafts.csswg.org/css2/visudet.html#block-replaced-width>
    /// - <https://drafts.csswg.org/css2/visudet.html#normal-block>
    /// - <https://drafts.csswg.org/css2/visudet.html#inline-replaced-height>
    pub(crate) fn layout_in_flow_block_level(
        &self,
        layout_context: &LayoutContext,
        positioning_context: &mut PositioningContext,
        containing_block: &ContainingBlock,
        sequential_layout_state: Option<&mut SequentialLayoutState>,
        ignore_block_margins_for_stretch: LogicalSides1D<bool>,
    ) -> BoxFragment {
        if let Some(sequential_layout_state) = sequential_layout_state {
            return self.layout_in_flow_block_level_sequentially(
                layout_context,
                positioning_context,
                containing_block,
                sequential_layout_state,
                ignore_block_margins_for_stretch,
            );
        }

        let get_inline_content_sizes = |constraint_space: &ConstraintSpace| {
            self.inline_content_sizes(layout_context, constraint_space)
                .sizes
        };
        let layout_style = self.layout_style();
        let ContainingBlockPaddingAndBorder {
            containing_block: containing_block_for_children,
            pbm,
            block_sizes,
            depends_on_block_constraints,
            available_block_size,
            justify_self,
            preferred_aspect_ratio,
        } = solve_containing_block_padding_and_border_for_in_flow_box(
            containing_block,
            &layout_style,
            get_inline_content_sizes,
            ignore_block_margins_for_stretch,
            Some(self),
        );

        let lazy_block_size = LazySize::new(
            &block_sizes,
            Direction::Block,
            Size::FitContent,
            Au::zero,
            available_block_size,
            layout_style.is_table(),
        );

        let layout = self.layout(
            layout_context,
            positioning_context,
            &containing_block_for_children,
            containing_block,
            preferred_aspect_ratio,
            &lazy_block_size,
        );

        let inline_size = layout
            .content_inline_size_for_table
            .unwrap_or(containing_block_for_children.size.inline);
        let block_size = lazy_block_size.resolve(|| layout.content_block_size);

        let ResolvedMargins {
            margin,
            effective_margin_inline_start,
        } = solve_margins(containing_block, &pbm, inline_size, justify_self);

        let content_rect = LogicalRect {
            start_corner: LogicalVec2 {
                block: pbm.padding.block_start + pbm.border.block_start,
                inline: pbm.padding.inline_start +
                    pbm.border.inline_start +
                    effective_margin_inline_start,
            },
            size: LogicalVec2 {
                block: block_size,
                inline: inline_size,
            },
        };

        let block_margins_collapsed_with_children = CollapsedBlockMargins::from_margin(&margin);
        let containing_block_writing_mode = containing_block.style.writing_mode;

        let mut base_fragment_info = self.base.base_fragment_info;
        if depends_on_block_constraints {
            base_fragment_info.flags.insert(
                FragmentFlags::SIZE_DEPENDS_ON_BLOCK_CONSTRAINTS_AND_CAN_BE_CHILD_OF_FLEX_ITEM,
            );
        }
        BoxFragment::new(
            base_fragment_info,
            self.base.style.clone(),
            layout.fragments,
            content_rect.as_physical(Some(containing_block)),
            pbm.padding.to_physical(containing_block_writing_mode),
            pbm.border.to_physical(containing_block_writing_mode),
            margin.to_physical(containing_block_writing_mode),
            layout.specific_layout_info,
        )
        .with_baselines(layout.baselines)
        .with_block_level_layout_info(block_margins_collapsed_with_children, None)
    }

    /// Lay out a normal in flow non-replaced block that establishes an independent
    /// formatting context in its containing formatting context but handling sequential
    /// layout concerns, such clearing and placing the content next to floats.
    fn layout_in_flow_block_level_sequentially(
        &self,
        layout_context: &LayoutContext<'_>,
        positioning_context: &mut PositioningContext,
        containing_block: &ContainingBlock<'_>,
        sequential_layout_state: &mut SequentialLayoutState,
        ignore_block_margins_for_stretch: LogicalSides1D<bool>,
    ) -> BoxFragment {
        let style = &self.base.style;
        let containing_block_writing_mode = containing_block.style.writing_mode;
        let ContentBoxSizesAndPBM {
            content_box_sizes,
            pbm,
            depends_on_block_constraints,
            ..
        } = self
            .layout_style()
            .content_box_sizes_and_padding_border_margin(&containing_block.into());

        let (margin_block_start, margin_block_end) =
            solve_block_margins_for_in_flow_block_level(&pbm);
        let collapsed_margin_block_start = CollapsedMargin::new(margin_block_start);

        // From https://drafts.csswg.org/css2/#floats:
        // "The border box of a table, a block-level replaced element, or an element in
        //  the normal flow that establishes a new block formatting context (such as an
        //  element with overflow other than visible) must not overlap the margin box of
        //  any floats in the same block formatting context as the element itself. If
        //  necessary, implementations should clear the said element by placing it below
        //  any preceding floats, but may place it adjacent to such floats if there is
        //  sufficient space. They may even make the border box of said element narrower
        //  than defined by section 10.3.3. CSS 2 does not define when a UA may put said
        //  element next to the float or by how much said element may become narrower."
        let mut content_size;
        let mut layout;
        let mut placement_rect;

        // First compute the clear position required by the 'clear' property.
        // The code below may then add extra clearance when the element can't fit
        // next to floats not covered by 'clear'.
        let clear_position = sequential_layout_state.calculate_clear_position(
            Clear::from_style_and_container_writing_mode(style, containing_block_writing_mode),
            &collapsed_margin_block_start,
        );
        let ceiling = clear_position.unwrap_or_else(|| {
            sequential_layout_state.position_without_clearance(&collapsed_margin_block_start)
        });

        // Then compute a tentative block size.
        let pbm_sums = pbm.sums_auto_is_zero(ignore_block_margins_for_stretch);
        let available_block_size = containing_block
            .size
            .block
            .to_definite()
            .map(|block_size| Au::zero().max(block_size - pbm_sums.block));
        let is_table = self.is_table();
        let preferred_aspect_ratio = self.preferred_aspect_ratio(&pbm.padding_border_sums);
        let tentative_block_content_size =
            self.tentative_block_content_size(preferred_aspect_ratio);
        let (preferred_block_size, min_block_size, max_block_size) =
            if let Some(block_content_size) = tentative_block_content_size {
                let (preferred, min, max) = content_box_sizes.block.resolve_each(
                    Size::FitContent,
                    Au::zero,
                    available_block_size,
                    || block_content_size,
                    is_table,
                );
                (Some(preferred), min, max)
            } else {
                content_box_sizes.block.resolve_each_extrinsic(
                    Size::FitContent,
                    Au::zero(),
                    available_block_size,
                )
            };
        let tentative_block_size =
            SizeConstraint::new(preferred_block_size, min_block_size, max_block_size);

        // With the tentative block size we can compute the inline min/max-content sizes.
        let get_inline_content_sizes = || {
            let constraint_space =
                ConstraintSpace::new(tentative_block_size, style, preferred_aspect_ratio);
            self.inline_content_sizes(layout_context, &constraint_space)
                .sizes
        };

        let justify_self = resolve_justify_self(style, containing_block.style);
        let automatic_inline_size = automatic_inline_size(justify_self, Some(self));
        let compute_inline_size = |stretch_size| {
            content_box_sizes.inline.resolve(
                Direction::Inline,
                automatic_inline_size,
                Au::zero,
                Some(stretch_size),
                get_inline_content_sizes,
                is_table,
            )
        };

        let get_lazy_block_size = || {
            LazySize::new(
                &content_box_sizes.block,
                Direction::Block,
                Size::FitContent,
                Au::zero,
                available_block_size,
                is_table,
            )
        };

        // The final inline size can depend on the available space, which depends on where
        // we are placing the box, since floats reduce the available space.
        // Here we assume that `compute_inline_size()` is a monotonically increasing function
        // with respect to the available space. Therefore, if we get the same result for 0
        // and for MAX_AU, it means that the function is constant.
        // TODO: `compute_inline_size()` may not be monotonic with `calc-size()`. For example,
        // `calc-size(stretch, (1px / (size + 1px) + sign(size)) * 1px)` would result in 1px
        // both when the available space is zero and infinity, but it's not constant.
        let inline_size_with_no_available_space = compute_inline_size(Au::zero());
        if inline_size_with_no_available_space == compute_inline_size(MAX_AU) {
            // If the inline size doesn't depend on the available inline space, we can just
            // compute it with an available inline space of zero. Then, after layout we can
            // compute the block size, and finally place among floats.
            let inline_size = inline_size_with_no_available_space;
            let lazy_block_size = get_lazy_block_size();
            layout = self.layout(
                layout_context,
                positioning_context,
                &ContainingBlock {
                    size: ContainingBlockSize {
                        inline: inline_size,
                        block: tentative_block_size,
                    },
                    style,
                },
                containing_block,
                preferred_aspect_ratio,
                &lazy_block_size,
            );

            content_size = LogicalVec2 {
                block: lazy_block_size.resolve(|| layout.content_block_size),
                inline: layout.content_inline_size_for_table.unwrap_or(inline_size),
            };

            let mut placement = PlacementAmongFloats::new(
                &sequential_layout_state.floats,
                ceiling,
                content_size + pbm.padding_border_sums,
                &pbm,
            );
            placement_rect = placement.place();
        } else {
            // If the inline size depends on the available space, then we need to iterate
            // the various placement candidates, resolve both the inline and block sizes
            // on each one placement area, and then check if the box actually fits it.
            // As an optimization, we first compute a lower bound of the final box size,
            // and skip placement candidates where not even the lower bound would fit.
            let minimum_size_of_block = LogicalVec2 {
                // For the lower bound of the inline size, simply assume no available space.
                // TODO: this won't work for things like `calc-size(stretch, 100px - size)`,
                // which should result in a bigger size when the available space gets smaller.
                inline: inline_size_with_no_available_space,
                block: match tentative_block_size {
                    // If we were able to resolve the preferred and maximum block sizes,
                    // use the tentative block size (it takes the 3 sizes into account).
                    SizeConstraint::Definite(size) if max_block_size.is_some() => size,
                    // Oherwise the preferred or maximum block size might end up being zero,
                    // so can only rely on the minimum block size.
                    _ => min_block_size,
                },
            } + pbm.padding_border_sums;
            let mut placement = PlacementAmongFloats::new(
                &sequential_layout_state.floats,
                ceiling,
                minimum_size_of_block,
                &pbm,
            );

            loop {
                // First try to place the block using the minimum size as the object size.
                placement_rect = placement.place();
                let available_inline_size =
                    placement_rect.size.inline - pbm.padding_border_sums.inline;
                let proposed_inline_size = compute_inline_size(available_inline_size);

                // Now lay out the block using the inline size we calculated from the placement.
                // Later we'll check to see if the resulting block size is compatible with the
                // placement.
                let positioning_context_length = positioning_context.len();
                let lazy_block_size = get_lazy_block_size();
                layout = self.layout(
                    layout_context,
                    positioning_context,
                    &ContainingBlock {
                        size: ContainingBlockSize {
                            inline: proposed_inline_size,
                            block: tentative_block_size,
                        },
                        style,
                    },
                    containing_block,
                    preferred_aspect_ratio,
                    &lazy_block_size,
                );

                let inline_size = if let Some(inline_size) = layout.content_inline_size_for_table {
                    // This is a table that ended up being smaller than predicted because of
                    // collapsed columns. Note we don't backtrack to consider areas that we
                    // previously thought weren't big enough.
                    // TODO: Should `minimum_size_of_block.inline` be zero for tables?
                    debug_assert!(inline_size < proposed_inline_size);
                    inline_size
                } else {
                    proposed_inline_size
                };
                content_size = LogicalVec2 {
                    block: lazy_block_size.resolve(|| layout.content_block_size),
                    inline: inline_size,
                };

                // Now we know the block size of this attempted layout of a box with block
                // size of auto. Try to fit it into our precalculated placement among the
                // floats. If it fits, then we can stop trying layout candidates.
                if placement.try_to_expand_for_auto_block_size(
                    content_size.block + pbm.padding_border_sums.block,
                    &placement_rect.size,
                ) {
                    break;
                }

                // The previous attempt to lay out this independent formatting context
                // among the floats did not work, so we must unhoist any boxes from that
                // attempt.
                positioning_context.truncate(&positioning_context_length);
            }
        }

        // Only set clearance if we would have cleared or the placement among floats moves
        // the block further in the block direction. These two situations are the ones that
        // prevent margin collapse.
        let has_clearance = clear_position.is_some() || placement_rect.start_corner.block > ceiling;
        let clearance = has_clearance.then(|| {
            placement_rect.start_corner.block -
                sequential_layout_state
                    .position_with_zero_clearance(&collapsed_margin_block_start)
        });

        let ((margin_inline_start, margin_inline_end), effective_margin_inline_start) =
            solve_inline_margins_avoiding_floats(
                sequential_layout_state,
                containing_block,
                &pbm,
                content_size.inline + pbm.padding_border_sums.inline,
                placement_rect,
                justify_self,
            );

        let margin = LogicalSides {
            inline_start: margin_inline_start,
            inline_end: margin_inline_end,
            block_start: margin_block_start,
            block_end: margin_block_end,
        };

        // Clearance prevents margin collapse between this block and previous ones,
        // so in that case collapse margins before adjoining them below.
        if clearance.is_some() {
            sequential_layout_state.collapse_margins();
        }
        sequential_layout_state.adjoin_assign(&collapsed_margin_block_start);

        // Margins can never collapse into independent formatting contexts.
        sequential_layout_state.collapse_margins();
        sequential_layout_state.advance_block_position(
            pbm.padding_border_sums.block + content_size.block + clearance.unwrap_or_else(Au::zero),
        );
        sequential_layout_state.adjoin_assign(&CollapsedMargin::new(margin.block_end));

        let content_rect = LogicalRect {
            start_corner: LogicalVec2 {
                block: pbm.padding.block_start +
                    pbm.border.block_start +
                    clearance.unwrap_or_else(Au::zero),
                inline: pbm.padding.inline_start +
                    pbm.border.inline_start +
                    effective_margin_inline_start,
            },
            size: content_size,
        };

        let mut base_fragment_info = self.base.base_fragment_info;
        if depends_on_block_constraints {
            base_fragment_info.flags.insert(
                FragmentFlags::SIZE_DEPENDS_ON_BLOCK_CONSTRAINTS_AND_CAN_BE_CHILD_OF_FLEX_ITEM,
            );
        }

        BoxFragment::new(
            base_fragment_info,
            style.clone(),
            layout.fragments,
            content_rect.as_physical(Some(containing_block)),
            pbm.padding.to_physical(containing_block_writing_mode),
            pbm.border.to_physical(containing_block_writing_mode),
            margin.to_physical(containing_block_writing_mode),
            layout.specific_layout_info,
        )
        .with_baselines(layout.baselines)
        .with_block_level_layout_info(CollapsedBlockMargins::from_margin(&margin), clearance)
    }
}

struct ContainingBlockPaddingAndBorder<'a> {
    containing_block: ContainingBlock<'a>,
    pbm: PaddingBorderMargin,
    block_sizes: Sizes,
    depends_on_block_constraints: bool,
    available_block_size: Option<Au>,
    justify_self: AlignFlags,
    preferred_aspect_ratio: Option<AspectRatio>,
}

struct ResolvedMargins {
    /// Used value for the margin properties, as exposed in getComputedStyle().
    pub margin: LogicalSides<Au>,

    /// Distance between the border box and the containing block on the inline-start side.
    /// This is typically the same as the inline-start margin, but can be greater when
    /// the box is justified within the free space in the containing block.
    /// The reason we aren't just adjusting the used margin-inline-start is that
    /// this shouldn't be observable via getComputedStyle().
    /// <https://drafts.csswg.org/css-align/#justify-self-property>
    pub effective_margin_inline_start: Au,
}

/// Given the style for an in-flow box and its containing block, determine the containing
/// block for its children.
/// Note that in the presence of floats, this shouldn't be used for a block-level box
/// that establishes an independent formatting context (or is replaced), since the
/// inline size could then be incorrect.
fn solve_containing_block_padding_and_border_for_in_flow_box<'a>(
    containing_block: &ContainingBlock<'_>,
    layout_style: &'a LayoutStyle,
    get_inline_content_sizes: impl FnOnce(&ConstraintSpace) -> ContentSizes,
    ignore_block_margins_for_stretch: LogicalSides1D<bool>,
    context: Option<&IndependentFormattingContext>,
) -> ContainingBlockPaddingAndBorder<'a> {
    let style = layout_style.style();
    if matches!(style.pseudo(), Some(PseudoElement::ServoAnonymousBox)) {
        // <https://drafts.csswg.org/css2/#anonymous-block-level>
        // > Anonymous block boxes are ignored when resolving percentage values that would
        // > refer to it: the closest non-anonymous ancestor box is used instead.
        let containing_block_for_children = ContainingBlock {
            size: ContainingBlockSize {
                inline: containing_block.size.inline,
                block: containing_block.size.block,
            },
            style,
        };
        // <https://drafts.csswg.org/css2/#anonymous-block-level>
        // > Non-inherited properties have their initial value.
        return ContainingBlockPaddingAndBorder {
            containing_block: containing_block_for_children,
            pbm: PaddingBorderMargin::zero(),
            block_sizes: Sizes::default(),
            depends_on_block_constraints: false,
            // The available block size may actually be definite, but it should be irrelevant
            // since the sizing properties are set to their initial value.
            available_block_size: None,
            // The initial `justify-self` is `auto`, but use `normal` (behaving as `stretch`).
            // This is being discussed in <https://github.com/w3c/csswg-drafts/issues/11461>.
            justify_self: AlignFlags::NORMAL,
            preferred_aspect_ratio: None,
        };
    }

    let ContentBoxSizesAndPBM {
        content_box_sizes,
        pbm,
        depends_on_block_constraints,
        ..
    } = layout_style.content_box_sizes_and_padding_border_margin(&containing_block.into());

    let pbm_sums = pbm.sums_auto_is_zero(ignore_block_margins_for_stretch);
    let available_inline_size = Au::zero().max(containing_block.size.inline - pbm_sums.inline);
    let available_block_size = containing_block
        .size
        .block
        .to_definite()
        .map(|block_size| Au::zero().max(block_size - pbm_sums.block));

    // TODO: support preferred aspect ratios on boxes that don't establish an independent
    // formatting context.
    let preferred_aspect_ratio =
        context.and_then(|context| context.preferred_aspect_ratio(&pbm.padding_border_sums));
    let is_table = layout_style.is_table();

    // https://drafts.csswg.org/css2/#the-height-property
    // https://drafts.csswg.org/css2/visudet.html#min-max-heights
    let tentative_block_content_size =
        context.and_then(|context| context.tentative_block_content_size(preferred_aspect_ratio));
    let tentative_block_size = if let Some(block_content_size) = tentative_block_content_size {
        SizeConstraint::Definite(content_box_sizes.block.resolve(
            Direction::Block,
            Size::FitContent,
            Au::zero,
            available_block_size,
            || block_content_size,
            is_table,
        ))
    } else {
        content_box_sizes.block.resolve_extrinsic(
            Size::FitContent,
            Au::zero(),
            available_block_size,
        )
    };

    // https://drafts.csswg.org/css2/#the-width-property
    // https://drafts.csswg.org/css2/visudet.html#min-max-widths
    let get_inline_content_sizes = || {
        get_inline_content_sizes(&ConstraintSpace::new(
            tentative_block_size,
            style,
            preferred_aspect_ratio,
        ))
    };
    let justify_self = resolve_justify_self(style, containing_block.style);
    let inline_size = content_box_sizes.inline.resolve(
        Direction::Inline,
        automatic_inline_size(justify_self, context),
        Au::zero,
        Some(available_inline_size),
        get_inline_content_sizes,
        is_table,
    );

    let containing_block_for_children = ContainingBlock {
        size: ContainingBlockSize {
            inline: inline_size,
            block: tentative_block_size,
        },
        style,
    };
    // https://drafts.csswg.org/css-writing-modes/#orthogonal-flows
    assert_eq!(
        containing_block.style.writing_mode.is_horizontal(),
        containing_block_for_children
            .style
            .writing_mode
            .is_horizontal(),
        "Vertical writing modes are not supported yet"
    );
    ContainingBlockPaddingAndBorder {
        containing_block: containing_block_for_children,
        pbm,
        block_sizes: content_box_sizes.block,
        depends_on_block_constraints,
        available_block_size,
        justify_self,
        preferred_aspect_ratio,
    }
}

/// Given the containing block and size of an in-flow box, determine the margins.
/// Note that in the presence of floats, this shouldn't be used for a block-level box
/// that establishes an independent formatting context (or is replaced), since the
/// margins could then be incorrect.
fn solve_margins(
    containing_block: &ContainingBlock<'_>,
    pbm: &PaddingBorderMargin,
    inline_size: Au,
    justify_self: AlignFlags,
) -> ResolvedMargins {
    let (inline_margins, effective_margin_inline_start) =
        solve_inline_margins_for_in_flow_block_level(
            containing_block,
            pbm,
            inline_size,
            justify_self,
        );
    let block_margins = solve_block_margins_for_in_flow_block_level(pbm);
    ResolvedMargins {
        margin: LogicalSides {
            inline_start: inline_margins.0,
            inline_end: inline_margins.1,
            block_start: block_margins.0,
            block_end: block_margins.1,
        },
        effective_margin_inline_start,
    }
}

/// Resolves 'auto' margins of an in-flow block-level box in the block axis.
/// <https://drafts.csswg.org/css2/#normal-block>
/// <https://drafts.csswg.org/css2/#block-root-margin>
fn solve_block_margins_for_in_flow_block_level(pbm: &PaddingBorderMargin) -> (Au, Au) {
    (
        pbm.margin.block_start.auto_is(Au::zero),
        pbm.margin.block_end.auto_is(Au::zero),
    )
}

/// Resolves the `justify-self` value, preserving flags.
fn resolve_justify_self(style: &ComputedValues, parent_style: &ComputedValues) -> AlignFlags {
    let is_ltr = |style: &ComputedValues| style.writing_mode.line_left_is_inline_start();
    let alignment = match style.clone_justify_self().0 {
        AlignFlags::AUTO => parent_style.clone_justify_items().computed.0.0,
        alignment => alignment,
    };
    let alignment_value = match alignment.value() {
        AlignFlags::LEFT if is_ltr(parent_style) => AlignFlags::START,
        AlignFlags::LEFT => AlignFlags::END,
        AlignFlags::RIGHT if is_ltr(parent_style) => AlignFlags::END,
        AlignFlags::RIGHT => AlignFlags::START,
        AlignFlags::SELF_START if is_ltr(parent_style) == is_ltr(style) => AlignFlags::START,
        AlignFlags::SELF_START => AlignFlags::END,
        AlignFlags::SELF_END if is_ltr(parent_style) == is_ltr(style) => AlignFlags::END,
        AlignFlags::SELF_END => AlignFlags::START,
        alignment_value => alignment_value,
    };
    alignment.flags() | alignment_value
}

/// Determines the automatic size for the inline axis of a block-level box.
/// <https://drafts.csswg.org/css-sizing-3/#automatic-size>
#[inline]
fn automatic_inline_size<T>(
    justify_self: AlignFlags,
    context: Option<&IndependentFormattingContext>,
) -> Size<T> {
    let normal_stretches = || {
        !context.is_some_and(|context| {
            context
                .base
                .base_fragment_info
                .flags
                .intersects(FragmentFlags::IS_REPLACED | FragmentFlags::IS_WIDGET) ||
                context.is_table()
        })
    };
    match justify_self {
        AlignFlags::STRETCH => Size::Stretch,
        AlignFlags::NORMAL if normal_stretches() => Size::Stretch,
        _ => Size::FitContent,
    }
}

/// Justifies a block-level box, distributing the free space according to `justify-self`.
/// Note `<center>` and `<div align>` are implemented via internal 'text-align' values,
/// which are also handled here.
/// The provided free space should already take margins into account. In particular,
/// it should be zero if there is an auto margin.
/// <https://drafts.csswg.org/css-align/#justify-block>
fn justify_self_alignment(
    containing_block: &ContainingBlock,
    free_space: Au,
    justify_self: AlignFlags,
) -> Au {
    let mut alignment = justify_self.value();
    let is_safe = justify_self.flags() == AlignFlags::SAFE || alignment == AlignFlags::NORMAL;
    if is_safe && free_space <= Au::zero() {
        alignment = AlignFlags::START
    }
    match alignment {
        AlignFlags::NORMAL => {},
        AlignFlags::CENTER => return free_space / 2,
        AlignFlags::END => return free_space,
        _ => return Au::zero(),
    }

    // For `justify-self: normal`, fall back to the special 'text-align' values.
    let style = containing_block.style;
    match style.clone_text_align() {
        TextAlignKeyword::MozCenter => free_space / 2,
        TextAlignKeyword::MozLeft if !style.writing_mode.line_left_is_inline_start() => free_space,
        TextAlignKeyword::MozRight if style.writing_mode.line_left_is_inline_start() => free_space,
        _ => Au::zero(),
    }
}

/// Resolves 'auto' margins of an in-flow block-level box in the inline axis,
/// distributing the free space in the containing block.
///
/// This is based on CSS2.1 § 10.3.3 <https://drafts.csswg.org/css2/#blockwidth>
/// but without adjusting the margins in "over-contrained" cases, as mandated by
/// <https://drafts.csswg.org/css-align/#justify-block>.
///
/// Note that in the presence of floats, this shouldn't be used for a block-level box
/// that establishes an independent formatting context (or is replaced).
///
/// In addition to the used margins, it also returns the effective margin-inline-start
/// (see ContainingBlockPaddingAndBorder).
fn solve_inline_margins_for_in_flow_block_level(
    containing_block: &ContainingBlock,
    pbm: &PaddingBorderMargin,
    inline_size: Au,
    justify_self: AlignFlags,
) -> ((Au, Au), Au) {
    let free_space = containing_block.size.inline - pbm.padding_border_sums.inline - inline_size;
    let mut justification = Au::zero();
    let inline_margins = match (pbm.margin.inline_start, pbm.margin.inline_end) {
        (AuOrAuto::Auto, AuOrAuto::Auto) => {
            let start = Au::zero().max(free_space / 2);
            (start, free_space - start)
        },
        (AuOrAuto::Auto, AuOrAuto::LengthPercentage(end)) => {
            (Au::zero().max(free_space - end), end)
        },
        (AuOrAuto::LengthPercentage(start), AuOrAuto::Auto) => (start, free_space - start),
        (AuOrAuto::LengthPercentage(start), AuOrAuto::LengthPercentage(end)) => {
            // In the cases above, the free space is zero after taking 'auto' margins into account.
            // But here we may still have some free space to perform 'justify-self' alignment.
            // This aligns the margin box within the containing block, or in other words,
            // aligns the border box within the margin-shrunken containing block.
            justification =
                justify_self_alignment(containing_block, free_space - start - end, justify_self);
            (start, end)
        },
    };
    let effective_margin_inline_start = inline_margins.0 + justification;
    (inline_margins, effective_margin_inline_start)
}

/// Resolves 'auto' margins of an in-flow block-level box in the inline axis
/// similarly to |solve_inline_margins_for_in_flow_block_level|. However,
/// they align within the provided rect (instead of the containing block),
/// to avoid overlapping floats.
/// In addition to the used margins, it also returns the effective
/// margin-inline-start (see ContainingBlockPaddingAndBorder).
/// It may differ from the used inline-start margin if the computed value
/// wasn't 'auto' and there are floats to avoid or the box is justified.
/// See <https://github.com/w3c/csswg-drafts/issues/9174>
fn solve_inline_margins_avoiding_floats(
    sequential_layout_state: &SequentialLayoutState,
    containing_block: &ContainingBlock,
    pbm: &PaddingBorderMargin,
    inline_size: Au,
    placement_rect: LogicalRect<Au>,
    justify_self: AlignFlags,
) -> ((Au, Au), Au) {
    // PlacementAmongFloats should guarantee that the inline size of the placement rect
    // is at least as big as `inline_size`. However, that may fail when dealing with
    // huge sizes that need to be saturated to MAX_AU, so floor by zero. See #37312.
    let free_space = Au::zero().max(placement_rect.size.inline - inline_size);
    let cb_info = &sequential_layout_state.floats.containing_block_info;
    let start_adjustment = placement_rect.start_corner.inline - cb_info.inline_start;
    let end_adjustment = cb_info.inline_end - placement_rect.max_inline_position();
    let mut justification = Au::zero();
    let inline_margins = match (pbm.margin.inline_start, pbm.margin.inline_end) {
        (AuOrAuto::Auto, AuOrAuto::Auto) => {
            let half = free_space / 2;
            (start_adjustment + half, end_adjustment + free_space - half)
        },
        (AuOrAuto::Auto, AuOrAuto::LengthPercentage(end)) => (start_adjustment + free_space, end),
        (AuOrAuto::LengthPercentage(start), AuOrAuto::Auto) => (start, end_adjustment + free_space),
        (AuOrAuto::LengthPercentage(start), AuOrAuto::LengthPercentage(end)) => {
            // The spec says 'justify-self' aligns the margin box within the float-shrunken
            // containing block. That's wrong (https://github.com/w3c/csswg-drafts/issues/9963),
            // and Blink and WebKit are broken anyways. So we match Gecko instead: this aligns
            // the border box within the instersection of the float-shrunken containing-block
            // and the margin-shrunken containing-block.
            justification = justify_self_alignment(containing_block, free_space, justify_self);
            (start, end)
        },
    };
    let effective_margin_inline_start = inline_margins.0.max(start_adjustment) + justification;
    (inline_margins, effective_margin_inline_start)
}

/// State that we maintain when placing blocks.
///
/// In parallel mode, this placement is done after all child blocks are laid out. In
/// sequential mode, this is done right after each block is laid out.
struct PlacementState<'container> {
    next_in_flow_margin_collapses_with_parent_start_margin: bool,
    last_in_flow_margin_collapses_with_parent_end_margin: bool,
    start_margin: CollapsedMargin,
    current_margin: CollapsedMargin,
    current_block_direction_position: Au,
    inflow_baselines: Baselines,
    is_inline_block_context: bool,

    /// If this [`PlacementState`] is laying out a list item with an outside marker. Record the
    /// block size of that marker, because the content block size of the list item needs to be at
    /// least as tall as the marker size -- even though the marker doesn't advance the block
    /// position of the placement.
    marker_block_size: Option<Au>,

    /// The [`ContainingBlock`] of the container into which this [`PlacementState`] is laying out
    /// fragments. This is used to convert between physical and logical geometry.
    containing_block: &'container ContainingBlock<'container>,
}

impl<'container> PlacementState<'container> {
    fn new(
        collapsible_with_parent_start_margin: CollapsibleWithParentStartMargin,
        containing_block: &'container ContainingBlock<'container>,
    ) -> PlacementState<'container> {
        let is_inline_block_context =
            containing_block.style.get_box().clone_display() == Display::InlineBlock;
        PlacementState {
            next_in_flow_margin_collapses_with_parent_start_margin:
                collapsible_with_parent_start_margin.0,
            last_in_flow_margin_collapses_with_parent_end_margin: true,
            start_margin: CollapsedMargin::zero(),
            current_margin: CollapsedMargin::zero(),
            current_block_direction_position: Au::zero(),
            inflow_baselines: Baselines::default(),
            is_inline_block_context,
            marker_block_size: None,
            containing_block,
        }
    }

    fn place_fragment_and_update_baseline(
        &mut self,
        fragment: &mut Fragment,
        sequential_layout_state: Option<&mut SequentialLayoutState>,
    ) {
        self.place_fragment(fragment, sequential_layout_state);

        let box_fragment = match fragment {
            Fragment::Box(box_fragment) => box_fragment,
            _ => return,
        };
        let box_fragment = box_fragment.borrow();

        // From <https://drafts.csswg.org/css-align-3/#baseline-export>:
        // > When finding the first/last baseline set of an inline-block, any baselines
        // > contributed by table boxes must be skipped. (This quirk is a legacy behavior from
        // > [CSS2].)
        if self.is_inline_block_context && box_fragment.is_table_wrapper() {
            return;
        }

        let box_block_offset = box_fragment
            .content_rect()
            .origin
            .to_logical(self.containing_block)
            .block;
        let box_fragment_baselines =
            box_fragment.baselines(self.containing_block.style.writing_mode);
        if let (None, Some(first)) = (self.inflow_baselines.first, box_fragment_baselines.first) {
            self.inflow_baselines.first = Some(first + box_block_offset);
        }
        if let Some(last) = box_fragment_baselines.last {
            self.inflow_baselines.last = Some(last + box_block_offset);
        }
    }

    /// Place a single [Fragment] in a block level context using the state so far and
    /// information gathered from the [Fragment] itself.
    fn place_fragment(
        &mut self,
        fragment: &mut Fragment,
        sequential_layout_state: Option<&mut SequentialLayoutState>,
    ) {
        match fragment {
            Fragment::Box(fragment) => {
                // If this child is a marker positioned outside of a list item, then record its
                // size, but also ensure that it doesn't advance the block position of the placment.
                // This ensures item content is placed next to the marker.
                //
                // This is a pretty big hack because it doesn't properly handle all interactions
                // between the marker and the item. For instance the marker should be positioned at
                // the baseline of list item content and the first line of the item content should
                // be at least as tall as the marker -- not the entire list item itself.
                let fragment = &mut *fragment.borrow_mut();
                let is_outside_marker = fragment
                    .base
                    .flags
                    .contains(FragmentFlags::IS_OUTSIDE_LIST_ITEM_MARKER);
                if is_outside_marker {
                    assert!(self.marker_block_size.is_none());
                    self.marker_block_size = Some(
                        fragment
                            .content_rect()
                            .size
                            .to_logical(self.containing_block.style.writing_mode)
                            .block,
                    );
                    return;
                }

                let BlockLevelLayoutInfo {
                    clearance,
                    block_margins_collapsed_with_children: fragment_block_margins,
                } = &**fragment
                    .block_level_layout_info
                    .as_ref()
                    .expect("A block-level fragment should have a BlockLevelLayoutInfo.");
                let mut fragment_block_size = fragment
                    .border_rect()
                    .size
                    .to_logical(self.containing_block.style.writing_mode)
                    .block;

                // We use `last_in_flow_margin_collapses_with_parent_end_margin` to implement
                // this quote from https://drafts.csswg.org/css2/#collapsing-margins
                // > If the top and bottom margins of an element with clearance are adjoining,
                // > its margins collapse with the adjoining margins of following siblings but that
                // > resulting margin does not collapse with the bottom margin of the parent block.
                if let Some(clearance) = *clearance {
                    fragment_block_size += clearance;
                    // Margins can't be adjoining if they are separated by clearance.
                    // Setting `next_in_flow_margin_collapses_with_parent_start_margin` to false
                    // prevents collapsing with the start margin of the parent, and will set
                    // `collapsed_through` to false, preventing the parent from collapsing through.
                    self.current_block_direction_position += self.current_margin.solve();
                    self.current_margin = CollapsedMargin::zero();
                    self.next_in_flow_margin_collapses_with_parent_start_margin = false;
                    if fragment_block_margins.collapsed_through {
                        self.last_in_flow_margin_collapses_with_parent_end_margin = false;
                    }
                } else if !fragment_block_margins.collapsed_through {
                    self.last_in_flow_margin_collapses_with_parent_end_margin = true;
                }

                if self.next_in_flow_margin_collapses_with_parent_start_margin {
                    debug_assert!(self.current_margin.solve().is_zero());
                    self.start_margin
                        .adjoin_assign(&fragment_block_margins.start);
                    if fragment_block_margins.collapsed_through {
                        self.start_margin.adjoin_assign(&fragment_block_margins.end);
                        return;
                    }
                    self.next_in_flow_margin_collapses_with_parent_start_margin = false;
                } else {
                    self.current_margin
                        .adjoin_assign(&fragment_block_margins.start);
                }

                fragment.base.rect.origin += LogicalVec2 {
                    inline: Au::zero(),
                    block: self.current_margin.solve() + self.current_block_direction_position,
                }
                .to_physical_size(self.containing_block.style.writing_mode);

                if fragment_block_margins.collapsed_through {
                    // `fragment_block_size` is typically zero when collapsing through,
                    // but we still need to consider it in case there is clearance.
                    self.current_block_direction_position += fragment_block_size;
                    self.current_margin
                        .adjoin_assign(&fragment_block_margins.end);
                } else {
                    self.current_block_direction_position +=
                        self.current_margin.solve() + fragment_block_size;
                    self.current_margin = fragment_block_margins.end;
                }
            },
            Fragment::AbsoluteOrFixedPositioned(fragment) => {
                // The alignment of absolutes in block flow layout is always "start", so the size of
                // the static position rectangle does not matter.
                fragment.borrow_mut().original_static_position_rect = LogicalRect {
                    start_corner: LogicalVec2 {
                        block: (self.current_margin.solve() +
                            self.current_block_direction_position),
                        inline: Au::zero(),
                    },
                    size: LogicalVec2::zero(),
                }
                .as_physical(Some(self.containing_block));
            },
            Fragment::Float(box_fragment) => {
                let sequential_layout_state = sequential_layout_state
                    .expect("Found float fragment without SequentialLayoutState");
                let block_offset_from_containing_block_top =
                    self.current_block_direction_position + self.current_margin.solve();
                let box_fragment = &mut *box_fragment.borrow_mut();
                sequential_layout_state.place_float_fragment(
                    box_fragment,
                    self.containing_block,
                    self.start_margin,
                    block_offset_from_containing_block_top,
                );
            },
            Fragment::Positioning(_) => {},
            _ => unreachable!(),
        }
    }

    fn finish(mut self) -> (Au, CollapsedBlockMargins, Baselines) {
        if !self.last_in_flow_margin_collapses_with_parent_end_margin {
            self.current_block_direction_position += self.current_margin.solve();
            self.current_margin = CollapsedMargin::zero();
        }
        let (total_block_size, collapsed_through) = match self.marker_block_size {
            Some(marker_block_size) => (
                self.current_block_direction_position.max(marker_block_size),
                // If this is a list item (even empty) with an outside marker, then it
                // should not collapse through.
                false,
            ),
            None => (
                self.current_block_direction_position,
                self.next_in_flow_margin_collapses_with_parent_start_margin,
            ),
        };

        (
            total_block_size,
            CollapsedBlockMargins {
                collapsed_through,
                start: self.start_margin,
                end: self.current_margin,
            },
            self.inflow_baselines,
        )
    }
}

pub(crate) struct IndependentFloatOrAtomicLayoutResult {
    pub fragment: BoxFragment,
    pub baselines: Baselines,
    pub pbm_sums: LogicalSides<Au>,
}

impl IndependentFormattingContext {
    pub(crate) fn layout_float_or_atomic_inline(
        &self,
        layout_context: &LayoutContext,
        child_positioning_context: &mut PositioningContext,
        containing_block: &ContainingBlock,
    ) -> IndependentFloatOrAtomicLayoutResult {
        let style = self.style();
        let container_writing_mode = containing_block.style.writing_mode;
        let layout_style = self.layout_style();
        let content_box_sizes_and_pbm =
            layout_style.content_box_sizes_and_padding_border_margin(&containing_block.into());
        let pbm = &content_box_sizes_and_pbm.pbm;
        let margin = pbm.margin.auto_is(Au::zero);
        let pbm_sums = pbm.padding + pbm.border + margin;
        let preferred_aspect_ratio = self.preferred_aspect_ratio(&pbm.padding_border_sums);
        let is_table = self.is_table();

        let available_inline_size =
            Au::zero().max(containing_block.size.inline - pbm_sums.inline_sum());
        let available_block_size = containing_block
            .size
            .block
            .to_definite()
            .map(|block_size| Au::zero().max(block_size - pbm_sums.block_sum()));

        let tentative_block_content_size =
            self.tentative_block_content_size(preferred_aspect_ratio);
        let tentative_block_size = if let Some(block_content_size) = tentative_block_content_size {
            SizeConstraint::Definite(content_box_sizes_and_pbm.content_box_sizes.block.resolve(
                Direction::Block,
                Size::FitContent,
                Au::zero,
                available_block_size,
                || block_content_size,
                is_table,
            ))
        } else {
            content_box_sizes_and_pbm
                .content_box_sizes
                .block
                .resolve_extrinsic(Size::FitContent, Au::zero(), available_block_size)
        };

        let get_content_size = || {
            let constraint_space =
                ConstraintSpace::new(tentative_block_size, style, preferred_aspect_ratio);
            self.inline_content_sizes(layout_context, &constraint_space)
                .sizes
        };

        let inline_size = content_box_sizes_and_pbm.content_box_sizes.inline.resolve(
            Direction::Inline,
            Size::FitContent,
            Au::zero,
            Some(available_inline_size),
            get_content_size,
            is_table,
        );

        let containing_block_for_children = ContainingBlock {
            size: ContainingBlockSize {
                inline: inline_size,
                block: tentative_block_size,
            },
            style,
        };
        assert_eq!(
            container_writing_mode.is_horizontal(),
            style.writing_mode.is_horizontal(),
            "Mixed horizontal and vertical writing modes are not supported yet"
        );

        let lazy_block_size = LazySize::new(
            &content_box_sizes_and_pbm.content_box_sizes.block,
            Direction::Block,
            Size::FitContent,
            Au::zero,
            available_block_size,
            is_table,
        );

        let CacheableLayoutResult {
            content_inline_size_for_table,
            content_block_size,
            fragments,
            baselines,
            specific_layout_info,
            ..
        } = self.layout(
            layout_context,
            child_positioning_context,
            &containing_block_for_children,
            containing_block,
            preferred_aspect_ratio,
            &lazy_block_size,
        );

        let content_size = LogicalVec2 {
            inline: content_inline_size_for_table.unwrap_or(inline_size),
            block: lazy_block_size.resolve(|| content_block_size),
        }
        .to_physical_size(container_writing_mode);
        let content_rect = PhysicalRect::new(PhysicalPoint::zero(), content_size);

        let mut base_fragment_info = self.base_fragment_info();
        if content_box_sizes_and_pbm.depends_on_block_constraints {
            base_fragment_info.flags.insert(
                FragmentFlags::SIZE_DEPENDS_ON_BLOCK_CONSTRAINTS_AND_CAN_BE_CHILD_OF_FLEX_ITEM,
            );
        }

        // Floats can have clearance, but it's handled internally by the float placement logic,
        // so there's no need to store it explicitly in the fragment.
        // And atomic inlines don't have clearance.
        let fragment = BoxFragment::new(
            base_fragment_info,
            style.clone(),
            fragments,
            content_rect,
            pbm.padding.to_physical(container_writing_mode),
            pbm.border.to_physical(container_writing_mode),
            margin.to_physical(container_writing_mode),
            specific_layout_info,
        );

        IndependentFloatOrAtomicLayoutResult {
            fragment,
            baselines,
            pbm_sums,
        }
    }
}
