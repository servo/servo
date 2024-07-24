/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
#![allow(rustdoc::private_intra_doc_links)]

//! Flow layout, also known as block-and-inline layout.

use app_units::Au;
use inline::InlineFormattingContext;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::Serialize;
use servo_arc::Arc;
use style::computed_values::clear::T as Clear;
use style::computed_values::float::T as Float;
use style::logical_geometry::WritingMode;
use style::properties::ComputedValues;
use style::values::computed::{Length, LengthOrAuto, Size};
use style::values::specified::{Display, TextAlignKeyword};
use style::Zero;

use crate::cell::ArcRefCell;
use crate::context::LayoutContext;
use crate::flow::float::{
    ContainingBlockPositionInfo, FloatBox, PlacementAmongFloats, SequentialLayoutState,
};
use crate::formatting_contexts::{
    Baselines, IndependentFormattingContext, IndependentLayout, NonReplacedFormattingContext,
};
use crate::fragment_tree::{
    BaseFragmentInfo, BoxFragment, CollapsedBlockMargins, CollapsedMargin, Fragment, FragmentFlags,
};
use crate::geom::{AuOrAuto, LogicalRect, LogicalSides, LogicalVec2};
use crate::positioned::{AbsolutelyPositionedBox, PositioningContext, PositioningContextLength};
use crate::replaced::ReplacedContent;
use crate::sizing::{self, ContentSizes};
use crate::style_ext::{Clamp, ComputedValuesExt, PaddingBorderMargin};
use crate::ContainingBlock;

mod construct;
pub mod float;
pub mod inline;
mod root;

pub(crate) use construct::BlockContainerBuilder;
pub use root::{BoxTree, CanvasBackground};

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

impl BlockContainer {
    fn contains_floats(&self) -> bool {
        match self {
            BlockContainer::BlockLevelBoxes(boxes) => boxes
                .iter()
                .any(|block_level_box| block_level_box.borrow().contains_floats()),
            BlockContainer::InlineFormattingContext(context) => context.contains_floats,
        }
    }
}

#[derive(Debug, Serialize)]
pub(crate) enum BlockLevelBox {
    Independent(IndependentFormattingContext),
    OutOfFlowAbsolutelyPositionedBox(ArcRefCell<AbsolutelyPositionedBox>),
    OutOfFlowFloatBox(FloatBox),
    OutsideMarker(OutsideMarker),
    SameFormattingContextBlock {
        base_fragment_info: BaseFragmentInfo,
        #[serde(skip_serializing)]
        style: Arc<ComputedValues>,
        contents: BlockContainer,
        contains_floats: bool,
    },
}

impl BlockLevelBox {
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
        collected_margin: &mut CollapsedMargin,
        containing_block: &ContainingBlock,
    ) -> bool {
        let style = match self {
            BlockLevelBox::SameFormattingContextBlock { ref style, .. } => style,
            BlockLevelBox::OutOfFlowAbsolutelyPositionedBox(_) |
            BlockLevelBox::OutOfFlowFloatBox(_) => return true,
            BlockLevelBox::OutsideMarker(_) => return false,
            BlockLevelBox::Independent(ref context) => {
                // FIXME: If the element doesn't fit next to floats, it will get clearance.
                // In that case this should be returning false.
                context.style()
            },
        };

        // FIXME: This should only return false when 'clear' causes clearance.
        if style.get_box().clear != Clear::None {
            return false;
        }

        let pbm = style.padding_border_margin(containing_block);
        let start_margin = pbm.margin.block_start.auto_is(Au::zero);
        collected_margin.adjoin_assign(&CollapsedMargin::new(start_margin));

        let child_boxes = match self {
            BlockLevelBox::SameFormattingContextBlock { ref contents, .. } => match contents {
                BlockContainer::BlockLevelBoxes(boxes) => boxes,
                BlockContainer::InlineFormattingContext(_) => return false,
            },
            _ => return false,
        };

        if pbm.padding.block_start != Au::zero() || pbm.border.block_start != Au::zero() {
            return false;
        }

        let min_size = style
            .content_min_box_size(containing_block, &pbm)
            .auto_is(Length::zero);
        let max_size = style.content_max_box_size(containing_block, &pbm);
        let prefered_size = style.content_box_size(containing_block, &pbm);
        let inline_size = prefered_size
            .inline
            .auto_is(|| {
                let margin_inline_start = pbm.margin.inline_start.auto_is(Au::zero);
                let margin_inline_end = pbm.margin.inline_end.auto_is(Au::zero);
                (containing_block.inline_size -
                    pbm.padding_border_sums.inline -
                    margin_inline_start -
                    margin_inline_end)
                    .into()
            })
            .clamp_between_extremums(min_size.inline, max_size.inline);
        let block_size = prefered_size
            .block
            .map(|size| Au::from(size.clamp_between_extremums(min_size.block, max_size.block)));

        let containing_block_for_children = ContainingBlock {
            inline_size: inline_size.into(),
            block_size,
            style,
        };

        if !Self::find_block_margin_collapsing_with_parent_from_slice(
            child_boxes,
            collected_margin,
            &containing_block_for_children,
        ) {
            return false;
        }

        if !block_size_is_zero_or_auto(style.content_block_size(), containing_block) ||
            !block_size_is_zero_or_auto(style.min_block_size(), containing_block) ||
            pbm.padding_border_sums.block != Au::zero()
        {
            return false;
        }

        let end_margin = pbm.margin.block_end.auto_is(Au::zero);
        collected_margin.adjoin_assign(&CollapsedMargin::new(end_margin));

        true
    }

    fn find_block_margin_collapsing_with_parent_from_slice(
        boxes: &[ArcRefCell<BlockLevelBox>],
        margin: &mut CollapsedMargin,
        containing_block: &ContainingBlock,
    ) -> bool {
        boxes.iter().all(|block_level_box| {
            block_level_box
                .borrow()
                .find_block_margin_collapsing_with_parent(margin, containing_block)
        })
    }
}

pub(crate) struct FlowLayout {
    pub fragments: Vec<Fragment>,
    pub content_block_size: Length,
    pub collapsible_margins_in_children: CollapsedBlockMargins,
    /// The offset of the baselines in this layout in the content area, if there were some. This is
    /// used to propagate inflow baselines to the ancestors of `display: inline-block` elements
    /// and table content.
    pub baselines: Baselines,
}

#[derive(Clone, Copy)]
pub(crate) struct CollapsibleWithParentStartMargin(bool);

/// The contentes of a BlockContainer created to render a list marker
/// for a list that has `list-style-position: outside`.
#[derive(Debug, Serialize)]
pub(crate) struct OutsideMarker {
    #[serde(skip_serializing)]
    pub marker_style: Arc<ComputedValues>,
    #[serde(skip_serializing)]
    pub list_item_style: Arc<ComputedValues>,
    pub block_container: BlockContainer,
}

impl OutsideMarker {
    fn layout(
        &self,
        layout_context: &LayoutContext<'_>,
        containing_block: &ContainingBlock<'_>,
        positioning_context: &mut PositioningContext,
        sequential_layout_state: Option<&mut SequentialLayoutState>,
        collapsible_with_parent_start_margin: Option<CollapsibleWithParentStartMargin>,
    ) -> Fragment {
        let content_sizes = self
            .block_container
            .inline_content_sizes(layout_context, containing_block.style.writing_mode);
        let containing_block_for_children = ContainingBlock {
            inline_size: content_sizes.max_content,
            block_size: AuOrAuto::auto(),
            style: &self.marker_style,
        };

        let flow_layout = self.block_container.layout(
            layout_context,
            positioning_context,
            &containing_block_for_children,
            sequential_layout_state,
            collapsible_with_parent_start_margin.unwrap_or(CollapsibleWithParentStartMargin(false)),
        );
        let max_inline_size = flow_layout.fragments.iter().fold(
            Length::zero(),
            |current_max, fragment| match fragment {
                Fragment::Text(text) => current_max.max(text.rect.max_inline_position().into()),
                Fragment::Image(image) => current_max.max(image.rect.max_inline_position().into()),
                Fragment::Positioning(positioning) => {
                    current_max.max(positioning.rect.max_inline_position().into())
                },
                Fragment::Box(_) |
                Fragment::Float(_) |
                Fragment::AbsoluteOrFixedPositioned(_) |
                Fragment::IFrame(_) => {
                    unreachable!("Found unexpected fragment type in outside list marker!");
                },
            },
        );

        // Position the marker beyond the inline start of the border box list item. This needs to
        // take into account the border and padding of the item.
        //
        // TODO: This is the wrong containing block, as it should be the containing block of the
        // parent of this list item. What this means in practice is that the writing mode could be
        // wrong and padding defined as a percentage will be resolved incorrectly.
        let pbm_of_list_item = self.list_item_style.padding_border_margin(containing_block);
        let content_rect = LogicalRect {
            start_corner: LogicalVec2 {
                inline: -max_inline_size -
                    (pbm_of_list_item.border.inline_start +
                        pbm_of_list_item.padding.inline_start)
                        .into(),
                block: Zero::zero(),
            },
            size: LogicalVec2 {
                inline: max_inline_size,
                block: flow_layout.content_block_size,
            },
        };

        let mut base_fragment_info = BaseFragmentInfo::anonymous();
        base_fragment_info.flags |= FragmentFlags::IS_OUTSIDE_LIST_ITEM_MARKER;

        Fragment::Box(BoxFragment::new(
            base_fragment_info,
            self.marker_style.clone(),
            flow_layout.fragments,
            content_rect.into(),
            LogicalSides::zero(),
            LogicalSides::zero(),
            LogicalSides::zero(),
            None,
            CollapsedBlockMargins::zero(),
        ))
    }
}

impl BlockFormattingContext {
    pub(super) fn layout(
        &self,
        layout_context: &LayoutContext,
        positioning_context: &mut PositioningContext,
        containing_block: &ContainingBlock,
    ) -> IndependentLayout {
        let mut sequential_layout_state = if self.contains_floats || !layout_context.use_rayon {
            Some(SequentialLayoutState::new(containing_block.inline_size))
        } else {
            None
        };

        let flow_layout = self.contents.layout(
            layout_context,
            positioning_context,
            containing_block,
            sequential_layout_state.as_mut(),
            CollapsibleWithParentStartMargin(false),
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

        IndependentLayout {
            fragments: flow_layout.fragments,
            content_block_size: Au::from(flow_layout.content_block_size) +
                flow_layout.collapsible_margins_in_children.end.solve() +
                clearance.unwrap_or_default(),
            content_inline_size_for_table: None,
            baselines: flow_layout.baselines,
        }
    }
}

/// Finds the min/max-content inline size of the block-level children of a block container.
/// The in-flow boxes will stack vertically, so we only need to consider the maximum size.
/// But floats can flow horizontally depending on 'clear', so we may need to sum their sizes.
/// CSS 2 does not define the exact algorithm, this logic is based on the behavior observed
/// on Gecko and Blink.
fn calculate_inline_content_size_for_block_level_boxes(
    boxes: &[ArcRefCell<BlockLevelBox>],
    layout_context: &LayoutContext,
    writing_mode: WritingMode,
) -> ContentSizes {
    let get_box_info = |box_: &ArcRefCell<BlockLevelBox>| {
        match &mut *box_.borrow_mut() {
            BlockLevelBox::OutOfFlowAbsolutelyPositionedBox(_) |
            BlockLevelBox::OutsideMarker { .. } => None,
            BlockLevelBox::OutOfFlowFloatBox(ref mut float_box) => {
                let size = float_box
                    .contents
                    .outer_inline_content_sizes(layout_context, writing_mode)
                    .max(ContentSizes::zero());
                let style_box = &float_box.contents.style().get_box();
                Some((size, style_box.float, style_box.clear))
            },
            BlockLevelBox::SameFormattingContextBlock {
                style, contents, ..
            } => {
                let size = sizing::outer_inline(style, writing_mode, || {
                    contents.inline_content_sizes(layout_context, style.writing_mode)
                })
                .max(ContentSizes::zero());
                // A block in the same BFC can overlap floats, it's not moved next to them,
                // so we shouldn't add its size to the size of the floats.
                // Instead, we treat it like an independent block with 'clear: both'.
                Some((size, Float::None, Clear::Both))
            },
            BlockLevelBox::Independent(ref mut independent) => {
                let size = independent
                    .outer_inline_content_sizes(layout_context, writing_mode)
                    .max(ContentSizes::zero());
                Some((size, Float::None, independent.style().get_box().clear))
            },
        }
    };

    /// When iterating the block-level boxes to compute the inline content sizes,
    /// this struct contains the data accumulated up to the current box.
    struct AccumulatedData {
        /// The maximum size seen so far, not including trailing uncleared floats.
        max_size: ContentSizes,
        /// The size of the trailing uncleared floats with 'float: left'.
        left_floats: ContentSizes,
        /// The size of the trailing uncleared floats with 'float: right'.
        right_floats: ContentSizes,
    }

    impl AccumulatedData {
        fn max_size_including_uncleared_floats(&self) -> ContentSizes {
            self.max_size
                .max(self.left_floats.union(&self.right_floats))
        }
        fn clear_floats(&mut self, clear: Clear) {
            match clear {
                Clear::Left => {
                    self.max_size = self.max_size_including_uncleared_floats();
                    self.left_floats = ContentSizes::zero();
                },
                Clear::Right => {
                    self.max_size = self.max_size_including_uncleared_floats();
                    self.right_floats = ContentSizes::zero();
                },
                Clear::Both => {
                    self.max_size = self.max_size_including_uncleared_floats();
                    self.left_floats = ContentSizes::zero();
                    self.right_floats = ContentSizes::zero();
                },
                Clear::None => {},
            };
        }
    }

    let accumulate = |mut data: AccumulatedData, (size, float, clear)| {
        data.clear_floats(clear);
        match float {
            Float::Left => data.left_floats = data.left_floats.union(&size),
            Float::Right => data.right_floats = data.right_floats.union(&size),
            Float::None => {
                data.max_size = data
                    .max_size
                    .max(data.left_floats.union(&data.right_floats).union(&size));
                data.left_floats = ContentSizes::zero();
                data.right_floats = ContentSizes::zero();
            },
        }
        data
    };
    let zero = AccumulatedData {
        max_size: ContentSizes::zero(),
        left_floats: ContentSizes::zero(),
        right_floats: ContentSizes::zero(),
    };
    let data = if layout_context.use_rayon {
        boxes
            .par_iter()
            .filter_map(get_box_info)
            .collect::<Vec<_>>()
            .into_iter()
            .fold(zero, accumulate)
    } else {
        boxes.iter().filter_map(get_box_info).fold(zero, accumulate)
    };
    data.max_size_including_uncleared_floats()
}

impl BlockContainer {
    fn layout(
        &self,
        layout_context: &LayoutContext,
        positioning_context: &mut PositioningContext,
        containing_block: &ContainingBlock,
        sequential_layout_state: Option<&mut SequentialLayoutState>,
        collapsible_with_parent_start_margin: CollapsibleWithParentStartMargin,
    ) -> FlowLayout {
        match self {
            BlockContainer::BlockLevelBoxes(child_boxes) => layout_block_level_children(
                layout_context,
                positioning_context,
                child_boxes,
                containing_block,
                sequential_layout_state,
                collapsible_with_parent_start_margin,
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

    pub(super) fn inline_content_sizes(
        &self,
        layout_context: &LayoutContext,
        writing_mode: WritingMode,
    ) -> ContentSizes {
        match &self {
            Self::BlockLevelBoxes(boxes) => calculate_inline_content_size_for_block_level_boxes(
                boxes,
                layout_context,
                writing_mode,
            ),
            Self::InlineFormattingContext(context) => {
                context.inline_content_sizes(layout_context, writing_mode)
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
) -> FlowLayout {
    let mut placement_state =
        PlacementState::new(collapsible_with_parent_start_margin, containing_block.style);

    let fragments = match sequential_layout_state {
        Some(ref mut sequential_layout_state) => layout_block_level_children_sequentially(
            layout_context,
            positioning_context,
            child_boxes,
            containing_block,
            sequential_layout_state,
            &mut placement_state,
        ),
        None => layout_block_level_children_in_parallel(
            layout_context,
            positioning_context,
            child_boxes,
            containing_block,
            &mut placement_state,
        ),
    };

    let (content_block_size, collapsible_margins_in_children, baselines) = placement_state.finish();
    FlowLayout {
        fragments,
        content_block_size,
        collapsible_margins_in_children,
        baselines,
    }
}

fn layout_block_level_children_in_parallel(
    layout_context: &LayoutContext,
    positioning_context: &mut PositioningContext,
    child_boxes: &[ArcRefCell<BlockLevelBox>],
    containing_block: &ContainingBlock,
    placement_state: &mut PlacementState,
) -> Vec<Fragment> {
    let collects_for_nearest_positioned_ancestor =
        positioning_context.collects_for_nearest_positioned_ancestor();
    let layout_results: Vec<(Fragment, PositioningContext)> = child_boxes
        .par_iter()
        .map(|child_box| {
            let mut child_positioning_context =
                PositioningContext::new_for_subtree(collects_for_nearest_positioned_ancestor);
            let fragment = child_box.borrow_mut().layout(
                layout_context,
                &mut child_positioning_context,
                containing_block,
                /* sequential_layout_state = */ None,
                /* collapsible_with_parent_start_margin = */ None,
            );
            (fragment, child_positioning_context)
        })
        .collect();

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
) -> Vec<Fragment> {
    // Because floats are involved, we do layout for this block formatting context in tree
    // order without parallelism. This enables mutable access to a `SequentialLayoutState` that
    // tracks every float encountered so far (again in tree order).
    child_boxes
        .iter()
        .map(|child_box| {
            let positioning_context_length_before_layout = positioning_context.len();
            let mut fragment = child_box.borrow_mut().layout(
                layout_context,
                positioning_context,
                containing_block,
                Some(&mut *sequential_layout_state),
                Some(CollapsibleWithParentStartMargin(
                    placement_state.next_in_flow_margin_collapses_with_parent_start_margin,
                )),
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
        &mut self,
        layout_context: &LayoutContext,
        positioning_context: &mut PositioningContext,
        containing_block: &ContainingBlock,
        sequential_layout_state: Option<&mut SequentialLayoutState>,
        collapsible_with_parent_start_margin: Option<CollapsibleWithParentStartMargin>,
    ) -> Fragment {
        match self {
            BlockLevelBox::SameFormattingContextBlock {
                base_fragment_info: tag,
                style,
                contents,
                ..
            } => Fragment::Box(positioning_context.layout_maybe_position_relative_fragment(
                layout_context,
                containing_block,
                style,
                |positioning_context| {
                    layout_in_flow_non_replaced_block_level_same_formatting_context(
                        layout_context,
                        positioning_context,
                        containing_block,
                        *tag,
                        style,
                        contents,
                        sequential_layout_state,
                        collapsible_with_parent_start_margin,
                    )
                },
            )),
            BlockLevelBox::Independent(independent) => match independent {
                IndependentFormattingContext::Replaced(replaced) => {
                    Fragment::Box(positioning_context.layout_maybe_position_relative_fragment(
                        layout_context,
                        containing_block,
                        &replaced.style,
                        |_positioning_context| {
                            layout_in_flow_replaced_block_level(
                                containing_block,
                                replaced.base_fragment_info,
                                &replaced.style,
                                &replaced.contents,
                                sequential_layout_state,
                            )
                        },
                    ))
                },
                IndependentFormattingContext::NonReplaced(non_replaced) => {
                    Fragment::Box(positioning_context.layout_maybe_position_relative_fragment(
                        layout_context,
                        containing_block,
                        &non_replaced.style,
                        |positioning_context| {
                            non_replaced.layout_in_flow_block_level(
                                layout_context,
                                positioning_context,
                                containing_block,
                                sequential_layout_state,
                            )
                        },
                    ))
                },
            },
            BlockLevelBox::OutOfFlowAbsolutelyPositionedBox(box_) => {
                let hoisted_box = AbsolutelyPositionedBox::to_hoisted(
                    box_.clone(),
                    // This is incorrect, however we do not know the
                    // correct positioning until later, in place_block_level_fragment,
                    // and this value will be adjusted there
                    LogicalVec2::zero(),
                    containing_block,
                );
                let hoisted_fragment = hoisted_box.fragment.clone();
                positioning_context.push(hoisted_box);
                Fragment::AbsoluteOrFixedPositioned(hoisted_fragment)
            },
            BlockLevelBox::OutOfFlowFloatBox(float_box) => Fragment::Float(float_box.layout(
                layout_context,
                positioning_context,
                containing_block,
            )),
            BlockLevelBox::OutsideMarker(outside_marker) => outside_marker.layout(
                layout_context,
                containing_block,
                positioning_context,
                sequential_layout_state,
                collapsible_with_parent_start_margin,
            ),
        }
    }
}

/// Lay out a normal flow non-replaced block that does not establish a new formatting
/// context.
///
/// - <https://drafts.csswg.org/css2/visudet.html#blockwidth>
/// - <https://drafts.csswg.org/css2/visudet.html#normal-block>
#[allow(clippy::too_many_arguments)]
fn layout_in_flow_non_replaced_block_level_same_formatting_context(
    layout_context: &LayoutContext,
    positioning_context: &mut PositioningContext,
    containing_block: &ContainingBlock,
    base_fragment_info: BaseFragmentInfo,
    style: &Arc<ComputedValues>,
    contents: &BlockContainer,
    mut sequential_layout_state: Option<&mut SequentialLayoutState>,
    collapsible_with_parent_start_margin: Option<CollapsibleWithParentStartMargin>,
) -> BoxFragment {
    let ContainingBlockPaddingAndBorder {
        containing_block: containing_block_for_children,
        pbm,
        min_box_size,
        max_box_size,
    } = solve_containing_block_padding_and_border_for_in_flow_box(containing_block, style);
    let ResolvedMargins {
        margin,
        effective_margin_inline_start,
    } = solve_margins(
        containing_block,
        &pbm,
        containing_block_for_children.inline_size,
    );

    let computed_block_size = style.content_block_size();
    let start_margin_can_collapse_with_children =
        pbm.padding.block_start == Au::zero() && pbm.border.block_start == Au::zero();

    let mut clearance = None;
    let parent_containing_block_position_info;
    match sequential_layout_state {
        None => parent_containing_block_position_info = None,
        Some(ref mut sequential_layout_state) => {
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
            ).0 && style.get_box().clear == Clear::None;
            if !collapsible_with_parent_start_margin && start_margin_can_collapse_with_children {
                if let BlockContainer::BlockLevelBoxes(child_boxes) = contents {
                    BlockLevelBox::find_block_margin_collapsing_with_parent_from_slice(
                        child_boxes,
                        &mut block_start_margin,
                        containing_block,
                    );
                }
            }

            // Introduce clearance if necessary.
            clearance = sequential_layout_state
                .calculate_clearance(style.get_box().clear, &block_start_margin);
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
                inline_end: inline_start + containing_block_for_children.inline_size,
            };
            parent_containing_block_position_info = Some(
                sequential_layout_state.replace_containing_block_position_info(new_cb_offsets),
            );
        },
    };

    let flow_layout = contents.layout(
        layout_context,
        positioning_context,
        &containing_block_for_children,
        sequential_layout_state.as_deref_mut(),
        CollapsibleWithParentStartMargin(start_margin_can_collapse_with_children),
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

    let collapsed_through = collapsible_margins_in_children.collapsed_through &&
        pbm.padding_border_sums.block == Au::zero() &&
        block_size_is_zero_or_auto(computed_block_size, containing_block) &&
        block_size_is_zero_or_auto(style.min_block_size(), containing_block);
    block_margins_collapsed_with_children.collapsed_through = collapsed_through;

    let end_margin_can_collapse_with_children = collapsed_through ||
        (pbm.padding.block_end == Au::zero() &&
            pbm.border.block_end == Au::zero() &&
            computed_block_size.is_auto());
    if end_margin_can_collapse_with_children {
        block_margins_collapsed_with_children
            .end
            .adjoin_assign(&collapsible_margins_in_children.end);
    } else {
        content_block_size += collapsible_margins_in_children.end.solve().into();
    }

    let block_size = containing_block_for_children.block_size.auto_is(|| {
        content_block_size
            .clamp_between_extremums(min_box_size.block, max_box_size.block)
            .into()
    });

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
            block_size - content_block_size.into() + pbm.padding.block_end + pbm.border.block_end,
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
            inline: containing_block_for_children.inline_size,
        },
    };

    BoxFragment::new(
        base_fragment_info,
        style.clone(),
        flow_layout.fragments,
        content_rect,
        pbm.padding,
        pbm.border,
        margin,
        clearance,
        block_margins_collapsed_with_children,
    )
    .with_baselines(flow_layout.baselines)
}

impl NonReplacedFormattingContext {
    /// Lay out a normal in flow non-replaced block that establishes an independent
    /// formatting context in its containing formatting context.
    ///
    /// - <https://drafts.csswg.org/css2/visudet.html#blockwidth>
    /// - <https://drafts.csswg.org/css2/visudet.html#normal-block>
    pub(crate) fn layout_in_flow_block_level(
        &self,
        layout_context: &LayoutContext,
        positioning_context: &mut PositioningContext,
        containing_block: &ContainingBlock,
        sequential_layout_state: Option<&mut SequentialLayoutState>,
    ) -> BoxFragment {
        if let Some(sequential_layout_state) = sequential_layout_state {
            return self.layout_in_flow_block_level_sequentially(
                layout_context,
                positioning_context,
                containing_block,
                sequential_layout_state,
            );
        }

        let ContainingBlockPaddingAndBorder {
            containing_block: containing_block_for_children,
            pbm,
            min_box_size,
            max_box_size,
        } = solve_containing_block_padding_and_border_for_in_flow_box(
            containing_block,
            &self.style,
        );

        let layout = self.layout(
            layout_context,
            positioning_context,
            &containing_block_for_children,
            containing_block,
        );

        let (block_size, inline_size) = match layout.content_inline_size_for_table {
            Some(inline_size) => (layout.content_block_size, inline_size),
            None => (
                containing_block_for_children.block_size.auto_is(|| {
                    layout.content_block_size.clamp_between_extremums(
                        min_box_size.block.into(),
                        max_box_size.block.map(|t| t.into()),
                    )
                }),
                containing_block_for_children.inline_size,
            ),
        };

        let ResolvedMargins {
            margin,
            effective_margin_inline_start,
        } = solve_margins(containing_block, &pbm, inline_size);

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

        BoxFragment::new(
            self.base_fragment_info,
            self.style.clone(),
            layout.fragments,
            content_rect,
            pbm.padding,
            pbm.border,
            margin,
            None, /* clearance */
            block_margins_collapsed_with_children,
        )
        .with_baselines(layout.baselines)
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
    ) -> BoxFragment {
        let pbm = self.style.padding_border_margin(containing_block);
        let box_size = self.style.content_box_size(containing_block, &pbm);
        let max_box_size = self.style.content_max_box_size(containing_block, &pbm);
        let min_box_size = self
            .style
            .content_min_box_size(containing_block, &pbm)
            .auto_is(Length::zero);
        let block_size = box_size.block.map(|block_size| {
            block_size.clamp_between_extremums(min_box_size.block, max_box_size.block)
        });

        let margin_inline_start;
        let margin_inline_end;
        let effective_margin_inline_start;
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
        let clearance;
        let mut content_size;
        let mut layout;
        if let LengthOrAuto::LengthPercentage(ref inline_size) = box_size.inline {
            let inline_size =
                inline_size.clamp_between_extremums(min_box_size.inline, max_box_size.inline);
            layout = self.layout(
                layout_context,
                positioning_context,
                &ContainingBlock {
                    inline_size: inline_size.into(),
                    block_size: block_size.map(|t| t.into()),
                    style: &self.style,
                },
                containing_block,
            );

            if let Some(inline_size) = layout.content_inline_size_for_table {
                content_size = LogicalVec2 {
                    block: layout.content_block_size,
                    inline: inline_size,
                }
                .into();
            } else {
                content_size = LogicalVec2 {
                    block: block_size.auto_is(|| {
                        Length::from(layout.content_block_size)
                            .clamp_between_extremums(min_box_size.block, max_box_size.block)
                    }),
                    inline: inline_size,
                };
            }

            (
                clearance,
                (margin_inline_start, margin_inline_end),
                effective_margin_inline_start,
            ) = solve_clearance_and_inline_margins_avoiding_floats(
                sequential_layout_state,
                &collapsed_margin_block_start,
                containing_block,
                &pbm,
                content_size + pbm.padding_border_sums.into(),
                &self.style,
            );
        } else {
            // First compute the clear position required by the 'clear' property.
            // The code below may then add extra clearance when the element can't fit
            // next to floats not covered by 'clear'.
            let clear_position = sequential_layout_state.calculate_clear_position(
                self.style.get_box().clear,
                &collapsed_margin_block_start,
            );
            let ceiling = clear_position.unwrap_or_else(|| {
                sequential_layout_state.position_without_clearance(&collapsed_margin_block_start)
            });

            // Create a PlacementAmongFloats using the minimum size in all dimensions as the object size.
            let minimum_size_of_block = LogicalVec2 {
                inline: min_box_size.inline.into(),
                block: block_size.auto_is(|| min_box_size.block).into(),
            } + pbm.padding_border_sums;
            let mut placement = PlacementAmongFloats::new(
                &sequential_layout_state.floats,
                ceiling,
                minimum_size_of_block,
                &pbm,
            );
            let mut placement_rect;

            loop {
                // First try to place the block using the minimum size as the object size.
                placement_rect = placement.place();
                let proposed_inline_size =
                    Length::from(placement_rect.size.inline - pbm.padding_border_sums.inline)
                        .clamp_between_extremums(min_box_size.inline, max_box_size.inline);

                // Now lay out the block using the inline size we calculated from the placement.
                // Later we'll check to see if the resulting block size is compatible with the
                // placement.
                let positioning_context_length = positioning_context.len();
                layout = self.layout(
                    layout_context,
                    positioning_context,
                    &ContainingBlock {
                        inline_size: proposed_inline_size.into(),
                        block_size: block_size.map(|t| t.into()),
                        style: &self.style,
                    },
                    containing_block,
                );

                if let Some(inline_size) = layout.content_inline_size_for_table {
                    // If this is a table, it's impossible to know the inline size it will take
                    // up until after trying to place it. If the table doesn't fit into this
                    // positioning rectangle due to incompatibility in the inline axis,
                    // then retry at another location.
                    // Even if it would fit in the inline axis, we may end up having to retry
                    // at another location due to incompatibility in the block axis. Therefore,
                    // always update the size in the PlacementAmongFloats as an optimization.
                    let outer_inline_size = inline_size + pbm.padding_border_sums.inline;
                    placement.set_inline_size(outer_inline_size, &pbm);
                    if outer_inline_size > placement_rect.size.inline {
                        positioning_context.truncate(&positioning_context_length);
                        continue;
                    }
                    content_size = LogicalVec2 {
                        block: layout.content_block_size,
                        inline: inline_size,
                    }
                    .into();
                } else {
                    content_size = LogicalVec2 {
                        block: block_size.auto_is(|| {
                            Length::from(layout.content_block_size)
                                .clamp_between_extremums(min_box_size.block, max_box_size.block)
                        }),
                        inline: proposed_inline_size,
                    };
                }

                // Now we know the block size of this attempted layout of a box with block
                // size of auto. Try to fit it into our precalculated placement among the
                // floats. If it fits, then we can stop trying layout candidates.
                if placement.try_to_expand_for_auto_block_size(
                    Au::from(content_size.block) + pbm.padding_border_sums.block,
                    &placement_rect.size,
                ) {
                    break;
                }

                // The previous attempt to lay out this independent formatting context
                // among the floats did not work, so we must unhoist any boxes from that
                // attempt.
                positioning_context.truncate(&positioning_context_length);
            }

            // Only set clearance if we would have cleared or the placement among floats moves
            // the block further in the block direction. These two situations are the ones that
            // prevent margin collapse.
            clearance = if clear_position.is_some() || placement_rect.start_corner.block > ceiling {
                Some(
                    placement_rect.start_corner.block -
                        sequential_layout_state
                            .position_with_zero_clearance(&collapsed_margin_block_start),
                )
            } else {
                None
            };

            (
                (margin_inline_start, margin_inline_end),
                effective_margin_inline_start,
            ) = solve_inline_margins_avoiding_floats(
                sequential_layout_state,
                containing_block,
                &pbm,
                content_size.inline + pbm.padding_border_sums.inline.into(),
                placement_rect.into(),
            );
        }

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
            pbm.padding_border_sums.block +
                Au::from(content_size.block) +
                clearance.unwrap_or_else(Au::zero),
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
            size: content_size.into(),
        };
        let block_margins_collapsed_with_children = CollapsedBlockMargins::from_margin(&margin);

        BoxFragment::new(
            self.base_fragment_info,
            self.style.clone(),
            layout.fragments,
            content_rect,
            pbm.padding,
            pbm.border,
            margin,
            clearance,
            block_margins_collapsed_with_children,
        )
        .with_baselines(layout.baselines)
    }
}

/// <https://drafts.csswg.org/css2/visudet.html#block-replaced-width>
/// <https://drafts.csswg.org/css2/visudet.html#inline-replaced-width>
/// <https://drafts.csswg.org/css2/visudet.html#inline-replaced-height>
fn layout_in_flow_replaced_block_level(
    containing_block: &ContainingBlock,
    base_fragment_info: BaseFragmentInfo,
    style: &Arc<ComputedValues>,
    replaced: &ReplacedContent,
    mut sequential_layout_state: Option<&mut SequentialLayoutState>,
) -> BoxFragment {
    let pbm = style.padding_border_margin(containing_block);
    let content_size = replaced.used_size_as_if_inline_element(containing_block, style, None, &pbm);

    let margin_inline_start;
    let margin_inline_end;
    let effective_margin_inline_start;
    let (margin_block_start, margin_block_end) = solve_block_margins_for_in_flow_block_level(&pbm);
    let fragments = replaced.make_fragments(style, content_size);

    let clearance;
    if let Some(ref mut sequential_layout_state) = sequential_layout_state {
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
        let collapsed_margin_block_start = CollapsedMargin::new(margin_block_start);
        let size = content_size + pbm.padding_border_sums;
        (
            clearance,
            (margin_inline_start, margin_inline_end),
            effective_margin_inline_start,
        ) = solve_clearance_and_inline_margins_avoiding_floats(
            sequential_layout_state,
            &collapsed_margin_block_start,
            containing_block,
            &pbm,
            size.into(),
            style,
        );

        // Clearance prevents margin collapse between this block and previous ones,
        // so in that case collapse margins before adjoining them below.
        if clearance.is_some() {
            sequential_layout_state.collapse_margins();
        }
        sequential_layout_state.adjoin_assign(&collapsed_margin_block_start);

        // Margins can never collapse into replaced elements.
        sequential_layout_state.collapse_margins();
        sequential_layout_state
            .advance_block_position(size.block + clearance.unwrap_or_else(Au::zero));
        sequential_layout_state.adjoin_assign(&CollapsedMargin::new(margin_block_end));
    } else {
        clearance = None;
        (
            (margin_inline_start, margin_inline_end),
            effective_margin_inline_start,
        ) = solve_inline_margins_for_in_flow_block_level(
            containing_block,
            &pbm,
            content_size.inline,
        );
    };

    let margin = LogicalSides {
        inline_start: margin_inline_start,
        inline_end: margin_inline_end,
        block_start: margin_block_start,
        block_end: margin_block_end,
    };

    let start_corner = LogicalVec2 {
        block: pbm.padding.block_start +
            pbm.border.block_start +
            clearance.unwrap_or_else(Au::zero),
        inline: pbm.padding.inline_start + pbm.border.inline_start + effective_margin_inline_start,
    };

    let content_rect = LogicalRect {
        start_corner,
        size: content_size,
    };
    let block_margins_collapsed_with_children = CollapsedBlockMargins::from_margin(&margin);

    BoxFragment::new(
        base_fragment_info,
        style.clone(),
        fragments,
        content_rect,
        pbm.padding,
        pbm.border,
        margin,
        clearance,
        block_margins_collapsed_with_children,
    )
}

struct ContainingBlockPaddingAndBorder<'a> {
    containing_block: ContainingBlock<'a>,
    pbm: PaddingBorderMargin,
    min_box_size: LogicalVec2<Length>,
    max_box_size: LogicalVec2<Option<Length>>,
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
    style: &'a Arc<ComputedValues>,
) -> ContainingBlockPaddingAndBorder<'a> {
    let pbm = style.padding_border_margin(containing_block);
    let box_size = style.content_box_size(containing_block, &pbm);
    let max_box_size = style.content_max_box_size(containing_block, &pbm);
    let min_box_size = style
        .content_min_box_size(containing_block, &pbm)
        .auto_is(Length::zero);

    // https://drafts.csswg.org/css2/#the-width-property
    // https://drafts.csswg.org/css2/visudet.html#min-max-widths
    let inline_size = box_size
        .inline
        .auto_is(|| {
            let margin_inline_start = pbm.margin.inline_start.auto_is(Au::zero);
            let margin_inline_end = pbm.margin.inline_end.auto_is(Au::zero);
            (containing_block.inline_size -
                pbm.padding_border_sums.inline -
                margin_inline_start -
                margin_inline_end)
                .into()
        })
        .clamp_between_extremums(min_box_size.inline, max_box_size.inline);

    // https://drafts.csswg.org/css2/#the-height-property
    // https://drafts.csswg.org/css2/visudet.html#min-max-heights
    let mut block_size = box_size.block;
    if let LengthOrAuto::LengthPercentage(ref mut block_size) = block_size {
        *block_size = block_size.clamp_between_extremums(min_box_size.block, max_box_size.block);
    }

    let containing_block_for_children = ContainingBlock {
        inline_size: inline_size.into(),
        block_size: block_size.map(|t| t.into()),
        style,
    };
    // https://drafts.csswg.org/css-writing-modes/#orthogonal-flows
    assert_eq!(
        containing_block.style.writing_mode, containing_block_for_children.style.writing_mode,
        "Mixed writing modes are not supported yet"
    );
    ContainingBlockPaddingAndBorder {
        containing_block: containing_block_for_children,
        pbm,
        min_box_size,
        max_box_size,
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
) -> ResolvedMargins {
    let (inline_margins, effective_margin_inline_start) =
        solve_inline_margins_for_in_flow_block_level(containing_block, pbm, inline_size);
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

/// This is supposed to handle 'justify-self', but no browser supports it on block boxes.
/// Instead, `<center>` and `<div align>` are implemented via internal 'text-align' values.
/// The provided free space should already take margins into account. In particular,
/// it should be zero if there is an auto margin.
/// <https://drafts.csswg.org/css-align/#justify-block>
fn justify_self_alignment(containing_block: &ContainingBlock, free_space: Au) -> Au {
    let style = containing_block.style;
    debug_assert!(free_space >= Au::zero());
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
) -> ((Au, Au), Au) {
    let free_space = containing_block.inline_size - pbm.padding_border_sums.inline - inline_size;
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
            let free_space = Au::zero().max(free_space - start - end);
            justification = justify_self_alignment(containing_block, free_space);
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
    inline_size: Length,
    placement_rect: LogicalRect<Length>,
) -> ((Au, Au), Au) {
    let free_space = Au::from(placement_rect.size.inline - inline_size);
    debug_assert!(free_space >= Au::zero());
    let cb_info = &sequential_layout_state.floats.containing_block_info;
    let start_adjustment = Au::from(placement_rect.start_corner.inline) - cb_info.inline_start;
    let end_adjustment = cb_info.inline_end - placement_rect.max_inline_position().into();
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
            justification = justify_self_alignment(containing_block, free_space);
            (start, end)
        },
    };
    let effective_margin_inline_start = inline_margins.0.max(start_adjustment) + justification;
    (inline_margins, effective_margin_inline_start)
}

/// A block-level element that establishes an independent formatting context (or is replaced)
/// must not overlap floats.
/// This can be achieved by adding clearance (to adjust the position in the block axis)
/// and/or modifying the margins in the inline axis.
/// This function takes care of calculating them.
fn solve_clearance_and_inline_margins_avoiding_floats(
    sequential_layout_state: &SequentialLayoutState,
    block_start_margin: &CollapsedMargin,
    containing_block: &ContainingBlock,
    pbm: &PaddingBorderMargin,
    size: LogicalVec2<Length>,
    style: &Arc<ComputedValues>,
) -> (Option<Au>, (Au, Au), Au) {
    let (clearance, placement_rect) = sequential_layout_state
        .calculate_clearance_and_inline_adjustment(
            style.get_box().clear,
            block_start_margin,
            pbm,
            size.into(),
        );
    let (inline_margins, effective_margin_inline_start) = solve_inline_margins_avoiding_floats(
        sequential_layout_state,
        containing_block,
        pbm,
        size.inline,
        placement_rect.into(),
    );
    (clearance, inline_margins, effective_margin_inline_start)
}

/// State that we maintain when placing blocks.
///
/// In parallel mode, this placement is done after all child blocks are laid out. In
/// sequential mode, this is done right after each block is laid out.
struct PlacementState {
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
}

impl PlacementState {
    fn new(
        collapsible_with_parent_start_margin: CollapsibleWithParentStartMargin,
        containing_block_style: &ComputedValues,
    ) -> PlacementState {
        let is_inline_block_context =
            containing_block_style.get_box().clone_display() == Display::InlineBlock;
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

        // From <https://drafts.csswg.org/css-align-3/#baseline-export>:
        // > When finding the first/last baseline set of an inline-block, any baselines
        // > contributed by table boxes must be skipped. (This quirk is a legacy behavior from
        // > [CSS2].)
        let display = box_fragment.style.clone_display();
        let is_table = display == Display::Table;
        if self.is_inline_block_context && is_table {
            return;
        }

        let box_block_offset = box_fragment.content_rect.start_corner.block;
        if let (None, Some(first)) = (self.inflow_baselines.first, box_fragment.baselines.first) {
            self.inflow_baselines.first = Some(first + box_block_offset);
        }
        if let Some(last) = box_fragment.baselines.last {
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
                let is_outside_marker = fragment
                    .base
                    .flags
                    .contains(FragmentFlags::IS_OUTSIDE_LIST_ITEM_MARKER);
                if is_outside_marker {
                    assert!(self.marker_block_size.is_none());
                    self.marker_block_size = Some(fragment.content_rect.size.block);
                    return;
                }

                let fragment_block_margins = &fragment.block_margins_collapsed_with_children;
                let mut fragment_block_size = fragment.padding.block_sum() +
                    fragment.border.block_sum() +
                    fragment.content_rect.size.block;

                // We use `last_in_flow_margin_collapses_with_parent_end_margin` to implement
                // this quote from https://drafts.csswg.org/css2/#collapsing-margins
                // > If the top and bottom margins of an element with clearance are adjoining,
                // > its margins collapse with the adjoining margins of following siblings but that
                // > resulting margin does not collapse with the bottom margin of the parent block.
                if let Some(clearance) = fragment.clearance {
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
                fragment.content_rect.start_corner.block +=
                    self.current_margin.solve() + self.current_block_direction_position;

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
                let offset = LogicalVec2 {
                    block: (self.current_margin.solve() + self.current_block_direction_position),
                    inline: Au::zero(),
                };
                fragment.borrow_mut().adjust_offsets(offset);
            },
            Fragment::Float(box_fragment) => {
                let sequential_layout_state = sequential_layout_state
                    .expect("Found float fragment without SequentialLayoutState");
                let block_offset_from_containing_block_top =
                    self.current_block_direction_position + self.current_margin.solve();
                sequential_layout_state.place_float_fragment(
                    box_fragment,
                    self.start_margin,
                    block_offset_from_containing_block_top,
                );
            },
            Fragment::Positioning(_) => {},
            _ => unreachable!(),
        }
    }

    fn finish(mut self) -> (Length, CollapsedBlockMargins, Baselines) {
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
            total_block_size.into(),
            CollapsedBlockMargins {
                collapsed_through,
                start: self.start_margin,
                end: self.current_margin,
            },
            self.inflow_baselines,
        )
    }
}

fn block_size_is_zero_or_auto(size: &Size, containing_block: &ContainingBlock) -> bool {
    match size {
        Size::Auto => true,
        Size::LengthPercentage(ref lp) => {
            // TODO: Should this resolve definite percentages? Blink does it, Gecko and WebKit don't.
            lp.is_definitely_zero() ||
                (lp.0.has_percentage() && containing_block.block_size.is_auto())
        },
    }
}
