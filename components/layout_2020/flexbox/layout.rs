/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::cmp::Ordering;

use app_units::Au;
use atomic_refcell::AtomicRefMut;
use itertools::izip;
use style::properties::longhands::align_content::computed_value::T as AlignContent;
use style::properties::longhands::align_items::computed_value::T as AlignItems;
use style::properties::longhands::align_self::computed_value::T as AlignSelf;
use style::properties::longhands::box_sizing::computed_value::T as BoxSizing;
use style::properties::longhands::flex_direction::computed_value::T as FlexDirection;
use style::properties::longhands::flex_wrap::computed_value::T as FlexWrap;
use style::properties::longhands::justify_content::computed_value::T as JustifyContent;
use style::properties::ComputedValues;
use style::values::computed::length::Size;
use style::values::computed::Length;
use style::values::generics::flex::GenericFlexBasis as FlexBasis;
use style::values::specified::align::AlignFlags;
use style::Zero;

use super::geom::{
    FlexAxis, FlexRelativeRect, FlexRelativeSides, FlexRelativeVec2, MainStartCrossStart,
};
use super::{FlexContainer, FlexLevelBox};
use crate::cell::ArcRefCell;
use crate::context::LayoutContext;
use crate::formatting_contexts::{Baselines, IndependentFormattingContext, IndependentLayout};
use crate::fragment_tree::{BoxFragment, CollapsedBlockMargins, Fragment};
use crate::geom::{AuOrAuto, LogicalRect, LogicalSides, LogicalVec2};
use crate::positioned::{AbsolutelyPositionedBox, PositioningContext, PositioningContextLength};
use crate::sizing::ContentSizes;
use crate::style_ext::{Clamp, ComputedValuesExt};
use crate::ContainingBlock;

// FIMXE: “Flex items […] `z-index` values other than `auto` create a stacking context
// even if `position` is `static` (behaving exactly as if `position` were `relative`).”
// https://drafts.csswg.org/css-flexbox/#painting
// (likely in `display_list/stacking_context.rs`)

/// Layout parameters and intermediate results about a flex container,
/// grouped to avoid passing around many parameters
struct FlexContext<'a> {
    layout_context: &'a LayoutContext<'a>,
    positioning_context: &'a mut PositioningContext,
    containing_block: &'a ContainingBlock<'a>, // For items
    container_is_single_line: bool,
    container_min_cross_size: Au,
    container_max_cross_size: Option<Au>,
    flex_axis: FlexAxis,
    flex_direction_is_reversed: bool,
    flex_wrap_reverse: bool,
    main_start_cross_start_sides_are: MainStartCrossStart,
    container_definite_inner_size: FlexRelativeVec2<Option<Au>>,
    align_content: AlignContent,
    align_items: AlignItems,
    justify_content: JustifyContent,
}

/// A flex item with some intermediate results
struct FlexItem<'a> {
    box_: &'a mut IndependentFormattingContext,
    content_box_size: FlexRelativeVec2<AuOrAuto>,
    content_min_size: FlexRelativeVec2<Au>,
    content_max_size: FlexRelativeVec2<Option<Au>>,
    padding: FlexRelativeSides<Au>,
    border: FlexRelativeSides<Au>,
    margin: FlexRelativeSides<AuOrAuto>,

    /// Sum of padding, border, and margin (with `auto` assumed to be zero) in each axis.
    /// This is the difference between an outer and inner size.
    pbm_auto_is_zero: FlexRelativeVec2<Au>,

    /// <https://drafts.csswg.org/css-flexbox/#algo-main-item>
    flex_base_size: Au,

    /// <https://drafts.csswg.org/css-flexbox/#algo-main-item>
    hypothetical_main_size: Au,
    /// This is `align-self`, defaulting to `align-items` if `auto`
    align_self: AlignItems,
}

/// Child of a FlexContainer. Can either be absolutely positioned, or not. If not,
/// a placeholder is used and flex content is stored outside of this enum.
enum FlexContent {
    AbsolutelyPositionedBox(ArcRefCell<AbsolutelyPositionedBox>),
    FlexItemPlaceholder,
}

/// A flex line with some intermediate results
struct FlexLine<'a> {
    items: &'a mut [FlexItem<'a>],
    outer_hypothetical_main_sizes_sum: Au,
}

/// Return type of `FlexItem::layout`
struct FlexItemLayoutResult {
    hypothetical_cross_size: Au,
    fragments: Vec<Fragment>,
    positioning_context: PositioningContext,

    // Either the first or the last baseline, depending on ‘align-self’.
    baseline_relative_to_margin_box: Option<Au>,
}

/// Return type of `FlexLine::layout`
struct FlexLineLayoutResult {
    cross_size: Au,
    item_fragments: Vec<(BoxFragment, PositioningContext)>, // One per flex item, in the given order
}

impl FlexContext<'_> {
    fn vec2_to_flex_relative<T>(&self, x: LogicalVec2<T>) -> FlexRelativeVec2<T> {
        self.flex_axis.vec2_to_flex_relative(x)
    }

    fn sides_to_flex_relative<T>(&self, x: LogicalSides<T>) -> FlexRelativeSides<T> {
        self.main_start_cross_start_sides_are
            .sides_to_flex_relative(x)
    }

    fn sides_to_flow_relative<T>(&self, x: FlexRelativeSides<T>) -> LogicalSides<T> {
        self.main_start_cross_start_sides_are
            .sides_to_flow_relative(x)
    }

    fn rect_to_flow_relative(
        &self,
        base_rect_size: FlexRelativeVec2<Au>,
        rect: FlexRelativeRect<Au>,
    ) -> LogicalRect<Au> {
        super::geom::rect_to_flow_relative(
            self.flex_axis,
            self.main_start_cross_start_sides_are,
            base_rect_size,
            rect,
        )
    }

    fn align_for(&self, align_self: &AlignSelf) -> AlignItems {
        let value = align_self.0 .0.value();
        let mapped_value = match value {
            AlignFlags::AUTO | AlignFlags::NORMAL => self.align_items.0,
            _ => value,
        };
        AlignItems(mapped_value)
    }
}

impl FlexContainer {
    pub fn inline_content_sizes(&self) -> ContentSizes {
        // FIXME: implement this. The spec for it is the same as for "normal" layout:
        // https://drafts.csswg.org/css-flexbox/#layout-algorithm
        // … except that the parts that say “the flex container is being sized
        // under a min or max-content constraint” apply.
        ContentSizes::zero() // Return an incorrect result rather than panic
    }

    /// <https://drafts.csswg.org/css-flexbox/#layout-algorithm>
    pub(crate) fn layout(
        &self,
        layout_context: &LayoutContext,
        positioning_context: &mut PositioningContext,
        containing_block: &ContainingBlock,
        containing_block_for_container: &ContainingBlock,
    ) -> IndependentLayout {
        // Actual length may be less, but we guess that usually not by a lot
        let mut flex_items = Vec::with_capacity(self.children.len());

        // Absolutely-positioned children of the flex container may be interleaved
        // with flex items. We need to preserve their relative order for correct painting order,
        // which is the order of `Fragment`s in this function’s return value.
        //
        // Example:
        // absolutely_positioned_items_with_original_order = [Some(item), Some(item), None, Some(item), None]
        // flex_items                                      =                         [item,             item]
        let absolutely_positioned_items_with_original_order = self
            .children
            .iter()
            .map(|arcrefcell| {
                let borrowed = arcrefcell.borrow_mut();
                match &*borrowed {
                    FlexLevelBox::OutOfFlowAbsolutelyPositionedBox(absolutely_positioned) => {
                        FlexContent::AbsolutelyPositionedBox(absolutely_positioned.clone())
                    },
                    FlexLevelBox::FlexItem(_) => {
                        let item = AtomicRefMut::map(borrowed, |child| match child {
                            FlexLevelBox::FlexItem(item) => item,
                            _ => unreachable!(),
                        });
                        flex_items.push(item);
                        FlexContent::FlexItemPlaceholder
                    },
                }
            })
            .collect::<Vec<_>>();

        // Column flex containers are not fully implemented yet,
        // so give a different layout instead of panicking.
        // FIXME: implement `todo!`s for FlexAxis::Column below, and remove this
        let container_style = containing_block.style;
        let flex_direction = match container_style.clone_flex_direction() {
            FlexDirection::Row | FlexDirection::Column => FlexDirection::Row,
            FlexDirection::RowReverse | FlexDirection::ColumnReverse => FlexDirection::RowReverse,
        };

        let flex_axis = FlexAxis::from(flex_direction);
        let (container_min_cross_size, container_max_cross_size) = self
            .available_cross_space_for_flex_items(
                container_style,
                flex_axis,
                containing_block_for_container,
            );

        let flex_wrap = container_style.get_position().flex_wrap;
        let container_is_single_line = match flex_wrap {
            FlexWrap::Nowrap => true,
            FlexWrap::Wrap | FlexWrap::WrapReverse => false,
        };
        let flex_direction_is_reversed = match flex_direction {
            FlexDirection::Row | FlexDirection::Column => false,
            FlexDirection::RowReverse | FlexDirection::ColumnReverse => true,
        };
        let flex_wrap_reverse = match flex_wrap {
            FlexWrap::Nowrap | FlexWrap::Wrap => false,
            FlexWrap::WrapReverse => true,
        };

        let align_content = container_style.clone_align_content();
        let align_items = AlignItems(match container_style.clone_align_items().0 {
            AlignFlags::AUTO | AlignFlags::NORMAL => AlignFlags::STRETCH,
            align => align,
        });
        let justify_content = container_style.clone_justify_content();

        let mut flex_context = FlexContext {
            layout_context,
            positioning_context,
            containing_block,
            container_min_cross_size,
            container_max_cross_size,
            container_is_single_line,
            flex_axis,
            flex_direction_is_reversed,
            flex_wrap_reverse,
            align_content,
            align_items,
            justify_content,
            main_start_cross_start_sides_are: MainStartCrossStart::from(
                flex_direction,
                flex_wrap_reverse,
            ),
            // https://drafts.csswg.org/css-flexbox/#definite-sizes
            container_definite_inner_size: flex_axis.vec2_to_flex_relative(LogicalVec2 {
                inline: Some(containing_block.inline_size),
                block: containing_block.block_size.non_auto(),
            }),
        };

        let flex_item_boxes = flex_items.iter_mut().map(|child| &mut **child);
        let mut flex_items = flex_item_boxes
            .map(|box_| FlexItem::new(&flex_context, box_))
            .collect::<Vec<_>>();

        // “Determine the main size of the flex container”
        // https://drafts.csswg.org/css-flexbox/#algo-main-container
        let container_main_size = match flex_axis {
            FlexAxis::Row => containing_block.inline_size,
            FlexAxis::Column => {
                // FIXME “using the rules of the formatting context in which it participates”
                // but if block-level with `block-size: max-auto` that requires
                // layout of the content to be fully done:
                // https://github.com/w3c/csswg-drafts/issues/4905
                // Gecko reportedly uses `block-size: fit-content` in this case
                // (which requires running another pass of the "full" layout algorithm)
                todo!()
                // Note: this panic shouldn’t happen since the start of `FlexContainer::layout`
                // forces `FlexAxis::Row`.
            },
        };

        // “Resolve the flexible lengths of all the flex items to find their *used main size*.”
        // https://drafts.csswg.org/css-flexbox/#algo-flex
        let flex_lines = collect_flex_lines(
            &mut flex_context,
            container_main_size,
            &mut flex_items,
            |flex_context, mut line| line.layout(flex_context, container_main_size),
        );

        let content_cross_size = flex_lines.iter().map(|line| line.cross_size).sum();

        // https://drafts.csswg.org/css-flexbox/#algo-cross-container
        let container_cross_size = flex_context
            .container_definite_inner_size
            .cross
            .unwrap_or(content_cross_size)
            .clamp_between_extremums(
                flex_context.container_min_cross_size,
                flex_context.container_max_cross_size,
            );

        // https://drafts.csswg.org/css-flexbox/#algo-line-align
        // Align all flex lines per `align-content`.
        let line_count = flex_lines.len();
        let mut cross_start_position_cursor = Au::zero();
        let mut line_interval = Au::zero();

        if let Some(cross_size) = flex_context.container_definite_inner_size.cross {
            let free_space = cross_size - content_cross_size;
            let layout_is_flex_reversed = flex_context.flex_wrap_reverse;

            // Implement fallback alignment.
            //
            // In addition to the spec at https://www.w3.org/TR/css-align-3/ this implementation follows
            // the resolution of https://github.com/w3c/csswg-drafts/issues/10154
            let resolved_align_content: AlignFlags = {
                let align_content_style = flex_context.align_content.0.primary();

                // Inital values from the style system
                let mut resolved_align_content = align_content_style.value();
                let mut is_safe = align_content_style.flags() == AlignFlags::SAFE;

                // Fallback occurs in two cases:

                // 1. If there is only a single item being aligned and alignment is a distributed alignment keyword
                //    https://www.w3.org/TR/css-align-3/#distribution-values
                if line_count <= 1 || free_space <= Au::zero() {
                    (resolved_align_content, is_safe) = match resolved_align_content {
                        AlignFlags::STRETCH => (AlignFlags::FLEX_START, true),
                        AlignFlags::SPACE_BETWEEN => (AlignFlags::FLEX_START, true),
                        AlignFlags::SPACE_AROUND => (AlignFlags::CENTER, true),
                        AlignFlags::SPACE_EVENLY => (AlignFlags::CENTER, true),
                        _ => (resolved_align_content, is_safe),
                    }
                };

                // 2. If free space is negative the "safe" alignment variants all fallback to Start alignment
                if free_space <= Au::zero() && is_safe {
                    resolved_align_content = AlignFlags::START;
                }

                resolved_align_content
            };

            // Implement "unsafe" alignment. "safe" alignment is handled by the fallback process above.
            cross_start_position_cursor = match resolved_align_content {
                AlignFlags::START => Au::zero(),
                AlignFlags::FLEX_START => {
                    if layout_is_flex_reversed {
                        free_space
                    } else {
                        Au::zero()
                    }
                },
                AlignFlags::END => free_space,
                AlignFlags::FLEX_END => {
                    if layout_is_flex_reversed {
                        Au::zero()
                    } else {
                        free_space
                    }
                },
                AlignFlags::CENTER => free_space / 2,
                AlignFlags::STRETCH => Au::zero(),
                AlignFlags::SPACE_BETWEEN => Au::zero(),
                AlignFlags::SPACE_AROUND => free_space / line_count as i32 / 2,
                AlignFlags::SPACE_EVENLY => free_space / (line_count + 1) as i32,

                // TODO: Implement all alignments. Note: not all alignment values are valid for content distribution
                _ => Au::zero(),
            };

            // TODO: Implement gap property
            line_interval = /*gap + */ match resolved_align_content {
                AlignFlags::START => Au::zero(),
                AlignFlags::FLEX_START => Au::zero(),
                AlignFlags::END => Au::zero(),
                AlignFlags::FLEX_END => Au::zero(),
                AlignFlags::CENTER => Au::zero(),
                AlignFlags::STRETCH => Au::zero(),
                AlignFlags::SPACE_BETWEEN => free_space / (line_count - 1) as i32,
                AlignFlags::SPACE_AROUND => free_space / line_count as i32,
                AlignFlags::SPACE_EVENLY => free_space / (line_count + 1) as i32,

                // TODO: Implement all alignments. Note: not all alignment values are valid for content distribution
                _ => Au::zero(),
            };
        };

        let line_cross_start_positions = flex_lines
            .iter()
            .map(|line| {
                let cross_start = cross_start_position_cursor;
                let cross_end = cross_start + line.cross_size + line_interval;
                cross_start_position_cursor = cross_end;
                cross_start
            })
            .collect::<Vec<_>>();

        let content_block_size = match flex_context.flex_axis {
            FlexAxis::Row => {
                // `container_main_size` ends up unused here but in this case that’s fine
                // since it was already exactly the one decided by the outer formatting context.
                container_cross_size
            },
            FlexAxis::Column => {
                // FIXME: `container_cross_size` ends up unused here, which is a bug.
                // It is meant to be the used inline-size, but the parent formatting context
                // has already decided a possibly-different used inline-size.
                // The spec is missing something to resolve this conflict:
                // https://github.com/w3c/csswg-drafts/issues/5190
                // And we’ll need to change the signature of `IndependentFormattingContext::layout`
                // to allow the inner formatting context to “negotiate” a used inline-size
                // with the outer one somehow.
                container_main_size
            },
        };

        let mut flex_item_fragments = flex_lines
            .into_iter()
            .zip(line_cross_start_positions)
            .flat_map(move |(mut line, line_cross_start_position)| {
                let flow_relative_line_position = match (flex_axis, flex_wrap_reverse) {
                    (FlexAxis::Row, false) => LogicalVec2 {
                        block: line_cross_start_position,
                        inline: Au::zero(),
                    },
                    (FlexAxis::Row, true) => LogicalVec2 {
                        block: container_cross_size - line_cross_start_position - line.cross_size,
                        inline: Au::zero(),
                    },
                    (FlexAxis::Column, false) => LogicalVec2 {
                        block: Au::zero(),
                        inline: line_cross_start_position,
                    },
                    (FlexAxis::Column, true) => LogicalVec2 {
                        block: Au::zero(),
                        inline: container_cross_size - line_cross_start_position - line.cross_size,
                    },
                };
                for (fragment, _) in &mut line.item_fragments {
                    fragment.content_rect.start_corner += flow_relative_line_position
                }
                line.item_fragments
            });

        let fragments = absolutely_positioned_items_with_original_order
            .into_iter()
            .map(|child_as_abspos| match child_as_abspos {
                FlexContent::AbsolutelyPositionedBox(absolutely_positioned) => {
                    let hoisted_box = AbsolutelyPositionedBox::to_hoisted(
                        absolutely_positioned,
                        LogicalVec2::zero(),
                        containing_block,
                    );
                    let hoisted_fragment = hoisted_box.fragment.clone();
                    positioning_context.push(hoisted_box);
                    Fragment::AbsoluteOrFixedPositioned(hoisted_fragment)
                },
                FlexContent::FlexItemPlaceholder => {
                    // The `flex_item_fragments` iterator yields one fragment
                    // per flex item, in the original order.
                    let (fragment, mut child_positioning_context) =
                        flex_item_fragments.next().unwrap();
                    let fragment = Fragment::Box(fragment);
                    child_positioning_context.adjust_static_position_of_hoisted_fragments(
                        &fragment,
                        PositioningContextLength::zero(),
                    );
                    positioning_context.append(child_positioning_context);
                    fragment
                },
            })
            .collect::<Vec<_>>();

        // There should be no more flex items
        assert!(flex_item_fragments.next().is_none());

        IndependentLayout {
            fragments,
            content_block_size,
            content_inline_size_for_table: None,
            // TODO: compute baseline for flex container
            baselines: Baselines::default(),
        }
    }

    fn available_cross_space_for_flex_items(
        &self,
        style: &ComputedValues,
        flex_axis: FlexAxis,
        containing_block_for_container: &ContainingBlock,
    ) -> (Au, Option<Au>) {
        let pbm = style.padding_border_margin(containing_block_for_container);
        let max_box_size = style.content_max_box_size(containing_block_for_container, &pbm);
        let min_box_size = style
            .content_min_box_size(containing_block_for_container, &pbm)
            .auto_is(Length::zero);

        let max_box_size = flex_axis.vec2_to_flex_relative(max_box_size);
        let min_box_size = flex_axis.vec2_to_flex_relative(min_box_size);

        (
            min_box_size.cross.into(),
            max_box_size.cross.map(Into::into),
        )
    }
}

impl<'a> FlexItem<'a> {
    fn new(flex_context: &FlexContext, box_: &'a mut IndependentFormattingContext) -> Self {
        let containing_block = flex_context.containing_block;

        // https://drafts.csswg.org/css-writing-modes/#orthogonal-flows
        assert_eq!(
            containing_block.style.writing_mode,
            box_.style().writing_mode,
            "Mixed writing modes are not supported yet"
        );

        let container_is_horizontal = containing_block.style.writing_mode.is_horizontal();
        let item_is_horizontal = box_.style().writing_mode.is_horizontal();
        let item_is_orthogonal = item_is_horizontal != container_is_horizontal;
        let container_is_row = flex_context.flex_axis == FlexAxis::Row;
        let cross_axis_is_item_block_axis = container_is_row ^ item_is_orthogonal;

        let pbm = box_.style().padding_border_margin(containing_block);
        let content_box_size = box_
            .style()
            .content_box_size(containing_block, &pbm)
            .map(|v| v.map(Au::from));
        let max_size = box_
            .style()
            .content_max_box_size(containing_block, &pbm)
            .map(|v| v.map(Au::from));
        let min_size = box_
            .style()
            .content_min_box_size(containing_block, &pbm)
            .map(|v| v.map(Au::from));

        // https://drafts.csswg.org/css-flexbox/#min-size-auto
        let automatic_min_size = || {
            // FIXME(stshine): Consider more situations when auto min size is not needed.
            if box_.style().get_box().overflow_x.is_scrollable() {
                return Au::zero();
            }

            if cross_axis_is_item_block_axis {
                let specified_size_suggestion = content_box_size.inline;

                let transferred_size_suggestion = match box_ {
                    IndependentFormattingContext::NonReplaced(_) => None,
                    IndependentFormattingContext::Replaced(ref bfc) => {
                        match (
                            bfc.contents
                                .inline_size_over_block_size_intrinsic_ratio(box_.style()),
                            content_box_size.block,
                        ) {
                            (Some(ratio), AuOrAuto::LengthPercentage(block_size)) => {
                                let block_size = block_size.clamp_between_extremums(
                                    min_size.block.auto_is(Au::zero),
                                    max_size.block,
                                );
                                Some(block_size.scale_by(ratio))
                            },
                            _ => None,
                        }
                    },
                };

                let inline_content_size = box_
                    .inline_content_sizes(flex_context.layout_context)
                    .min_content;
                let content_size_suggestion = match box_ {
                    IndependentFormattingContext::NonReplaced(_) => inline_content_size,
                    IndependentFormattingContext::Replaced(ref replaced) => {
                        if let Some(ratio) = replaced
                            .contents
                            .inline_size_over_block_size_intrinsic_ratio(box_.style())
                        {
                            inline_content_size.clamp_between_extremums(
                                min_size.block.auto_is(Au::zero).scale_by(ratio),
                                max_size.block.map(|l| l.scale_by(ratio)),
                            )
                        } else {
                            inline_content_size
                        }
                    },
                };

                let result = match specified_size_suggestion {
                    AuOrAuto::LengthPercentage(l) => l.min(content_size_suggestion),
                    AuOrAuto::Auto => {
                        if let Some(l) = transferred_size_suggestion {
                            l.min(content_size_suggestion)
                        } else {
                            content_size_suggestion
                        }
                    },
                };
                result.clamp_below_max(max_size.inline)
            } else {
                // FIXME(stshine): Implement this when main axis is item's block axis.
                Au::zero()
            }
        };

        let min_size = LogicalVec2 {
            inline: min_size.inline.auto_is(automatic_min_size),
            block: min_size.block.auto_is(Au::zero),
        };
        let margin_auto_is_zero = pbm.margin.auto_is(Au::zero);

        let content_box_size = flex_context.vec2_to_flex_relative(content_box_size);
        let content_max_size = flex_context.vec2_to_flex_relative(max_size);
        let content_min_size = flex_context.vec2_to_flex_relative(min_size);
        let margin_auto_is_zero = flex_context.sides_to_flex_relative(margin_auto_is_zero);
        let padding = flex_context.sides_to_flex_relative(pbm.padding);
        let border = flex_context.sides_to_flex_relative(pbm.border);
        let padding_border = padding.sum_by_axis() + border.sum_by_axis();
        let pbm_auto_is_zero = FlexRelativeVec2 {
            main: padding_border.main,
            cross: padding_border.cross,
        } + margin_auto_is_zero.sum_by_axis();

        let align_self = flex_context.align_for(&box_.style().clone_align_self());

        let flex_base_size = flex_base_size(
            flex_context,
            box_,
            cross_axis_is_item_block_axis,
            content_box_size,
            padding_border,
        );

        let hypothetical_main_size =
            flex_base_size.clamp_between_extremums(content_min_size.main, content_max_size.main);
        let margin: FlexRelativeSides<AuOrAuto> = flex_context.sides_to_flex_relative(pbm.margin);

        Self {
            box_,
            content_box_size,
            content_min_size,
            content_max_size,
            padding,
            border,
            margin,
            pbm_auto_is_zero,
            flex_base_size,
            hypothetical_main_size,
            align_self,
        }
    }
}

/// <https://drafts.csswg.org/css-flexbox/#algo-main-item>
fn flex_base_size(
    flex_context: &FlexContext,
    flex_item: &mut IndependentFormattingContext,
    cross_axis_is_item_block_axis: bool,
    content_box_size: FlexRelativeVec2<AuOrAuto>,
    padding_border_sums: FlexRelativeVec2<Au>,
) -> Au {
    let used_flex_basis = match &flex_item.style().get_position().flex_basis {
        FlexBasis::Content => FlexBasis::Content,
        FlexBasis::Size(Size::LengthPercentage(length_percentage)) => {
            let apply_box_sizing = |length: Au| {
                match flex_item.style().get_position().box_sizing {
                    BoxSizing::ContentBox => length,
                    BoxSizing::BorderBox => {
                        // This may make `length` negative,
                        // but it will be clamped in the hypothetical main size
                        length - padding_border_sums.main
                    },
                }
            };
            // “For example, percentage values of flex-basis are resolved
            //  against the flex item’s containing block (i.e. its flex container);”
            match flex_context.container_definite_inner_size.main {
                Some(container_definite_main_size) => {
                    let length = length_percentage
                        .0
                        .percentage_relative_to(container_definite_main_size.into());
                    FlexBasis::Size(apply_box_sizing(length.into()))
                },
                None => {
                    if let Some(length) = length_percentage.0.to_length() {
                        FlexBasis::Size(apply_box_sizing(length.into()))
                    } else {
                        // “and if that containing block’s size is indefinite,
                        //  the used value for `flex-basis` is `content`.”
                        // https://drafts.csswg.org/css-flexbox/#flex-basis-property
                        FlexBasis::Content
                    }
                },
            }
        },
        FlexBasis::Size(Size::Auto) => {
            // “When specified on a flex item, the `auto` keyword retrieves
            //  the value of the main size property as the used `flex-basis`.”
            match content_box_size.main {
                AuOrAuto::LengthPercentage(length) => FlexBasis::Size(length),
                // “If that value is itself `auto`, then the used value is `content`.”
                AuOrAuto::Auto => FlexBasis::Content,
            }
        },
    };

    // NOTE: at this point the flex basis is either `content` or a definite length.
    // However when we add support for additional values for `width` and `height`
    // from https://drafts.csswg.org/css-sizing/#preferred-size-properties,
    // it could have those values too.

    match used_flex_basis {
        FlexBasis::Size(length) => {
            // Case A: definite flex basis
            length
        },
        FlexBasis::Content => {
            // FIXME: implement cases B, C, D.

            // Case E: everything else
            // “treating a value of content as max-content.”
            if cross_axis_is_item_block_axis {
                // The main axis is the inline axis
                flex_item
                    .inline_content_sizes(flex_context.layout_context)
                    .max_content
            } else {
                // FIXME: block-axis content sizing requires another pass
                // of "full" layout
                todo!()
                // Note: this panic shouldn’t happen since the start of `FlexContainer::layout`
                // forces `FlexAxis::Row` and the `writing-mode` property is disabled.
            }
        },
    }
}

// “Collect flex items into flex lines”
// https://drafts.csswg.org/css-flexbox/#algo-line-break
fn collect_flex_lines<'items, LineResult>(
    flex_context: &mut FlexContext,
    container_main_size: Au,
    mut items: &'items mut [FlexItem<'items>],
    mut each: impl FnMut(&mut FlexContext, FlexLine<'items>) -> LineResult,
) -> Vec<LineResult> {
    if flex_context.container_is_single_line {
        let line = FlexLine {
            outer_hypothetical_main_sizes_sum: items
                .iter()
                .map(|item| item.hypothetical_main_size + item.pbm_auto_is_zero.main)
                .sum(),
            items,
        };
        vec![each(flex_context, line)]
    } else {
        let mut lines = Vec::new();
        let mut line_size_so_far = Au::zero();
        let mut line_so_far_is_empty = true;
        let mut index = 0;
        while let Some(item) = items.get(index) {
            let item_size = item.hypothetical_main_size + item.pbm_auto_is_zero.main;
            let line_size_would_be = line_size_so_far + item_size;
            let item_fits = line_size_would_be <= container_main_size;
            if item_fits || line_so_far_is_empty {
                line_size_so_far = line_size_would_be;
                line_so_far_is_empty = false;
                index += 1;
            } else {
                // We found something that doesn’t fit. This line ends *before* this item.
                let (line_items, rest) = items.split_at_mut(index);
                let line = FlexLine {
                    items: line_items,
                    outer_hypothetical_main_sizes_sum: line_size_so_far,
                };
                items = rest;
                lines.push(each(flex_context, line));
                // The next line has this item.
                line_size_so_far = item_size;
                index = 1;
            }
        }
        // The last line is added even without finding an item that doesn’t fit
        let line = FlexLine {
            items,
            outer_hypothetical_main_sizes_sum: line_size_so_far,
        };
        lines.push(each(flex_context, line));
        lines
    }
}

impl FlexLine<'_> {
    fn layout(
        &mut self,
        flex_context: &mut FlexContext,
        container_main_size: Au,
    ) -> FlexLineLayoutResult {
        let (item_used_main_sizes, mut free_space) =
            self.resolve_flexible_lengths(container_main_size);

        // https://drafts.csswg.org/css-flexbox/#algo-cross-item
        let mut item_layout_results = self
            .items
            .iter_mut()
            .zip(&item_used_main_sizes)
            .map(|(item, &used_main_size)| item.layout(used_main_size, flex_context, None))
            .collect::<Vec<_>>();

        // https://drafts.csswg.org/css-flexbox/#algo-cross-line
        let line_cross_size = self.cross_size(&item_layout_results, flex_context);
        let line_size = FlexRelativeVec2 {
            main: container_main_size,
            cross: line_cross_size,
        };

        // FIXME: Handle `align-content: stretch`
        // https://drafts.csswg.org/css-flexbox/#algo-line-stretch

        // FIXME: Collapse `visibility: collapse` items
        // This involves “restart layout from the beginning” with a modified second round,
        // which will make structuring the code… interesting.
        // https://drafts.csswg.org/css-flexbox/#algo-visibility

        // Determine the used cross size of each flex item
        // https://drafts.csswg.org/css-flexbox/#algo-stretch
        let item_count = self.items.len();
        let mut max_baseline = None;
        let mut item_used_cross_sizes = Vec::with_capacity(item_count);
        let mut item_cross_margins = Vec::with_capacity(item_count);
        for (item, item_layout_result, used_main_size) in izip!(
            self.items.iter_mut(),
            item_layout_results.iter_mut(),
            &item_used_main_sizes
        ) {
            let has_stretch = item.align_self.0.value() == AlignFlags::STRETCH;
            let used_cross_size = if has_stretch &&
                item.content_box_size.cross.is_auto() &&
                !(item.margin.cross_start.is_auto() || item.margin.cross_end.is_auto())
            {
                (line_cross_size - item.pbm_auto_is_zero.cross).clamp_between_extremums(
                    item.content_min_size.cross,
                    item.content_max_size.cross,
                )
            } else {
                item_layout_result.hypothetical_cross_size
            };
            item_used_cross_sizes.push(used_cross_size);

            if has_stretch {
                // “If the flex item has `align-self: stretch`, redo layout for its contents,
                //  treating this used size as its definite cross size
                //  so that percentage-sized children can be resolved.”
                *item_layout_result =
                    item.layout(*used_main_size, flex_context, Some(used_cross_size));
            }

            item_layout_result.baseline_relative_to_margin_box = match item.align_self.0.value() {
                AlignFlags::BASELINE | AlignFlags::LAST_BASELINE => {
                    let baseline = item_layout_result
                        .baseline_relative_to_margin_box
                        .unwrap_or_else(|| {
                            item.synthesized_baseline_relative_to_margin_box(used_cross_size)
                        });
                    max_baseline = Some(max_baseline.unwrap_or(baseline).max(baseline));
                    Some(baseline)
                },
                _ => None,
            };

            // https://drafts.csswg.org/css-flexbox/#algo-cross-margins
            item_cross_margins.push(item.resolve_auto_cross_margins(
                flex_context,
                line_cross_size,
                used_cross_size,
            ));
        }

        // Layout of items is over. These should no longer be mutable.
        let item_layout_results = item_layout_results;

        // Distribute any remaining free space
        // https://drafts.csswg.org/css-flexbox/#algo-main-align
        let (item_main_margins, free_space_distributed) =
            self.resolve_auto_main_margins(free_space);
        if free_space_distributed {
            free_space = Au::zero();
        }

        // Align the items along the main-axis per justify-content.
        let layout_is_flex_reversed = flex_context.flex_direction_is_reversed;

        // Implement fallback alignment.
        //
        // In addition to the spec at https://www.w3.org/TR/css-align-3/ this implementation follows
        // the resolution of https://github.com/w3c/csswg-drafts/issues/10154
        let resolved_justify_content: AlignFlags = {
            let justify_content_style = flex_context.justify_content.0.primary();

            // Inital values from the style system
            let mut resolved_justify_content = justify_content_style.value();
            let mut is_safe = justify_content_style.flags() == AlignFlags::SAFE;

            // Fallback occurs in two cases:

            // 1. If there is only a single item being aligned and alignment is a distributed alignment keyword
            //    https://www.w3.org/TR/css-align-3/#distribution-values
            if item_count <= 1 || free_space <= Au::zero() {
                (resolved_justify_content, is_safe) = match resolved_justify_content {
                    AlignFlags::STRETCH => (AlignFlags::FLEX_START, true),
                    AlignFlags::SPACE_BETWEEN => (AlignFlags::FLEX_START, true),
                    AlignFlags::SPACE_AROUND => (AlignFlags::CENTER, true),
                    AlignFlags::SPACE_EVENLY => (AlignFlags::CENTER, true),
                    _ => (resolved_justify_content, is_safe),
                }
            };

            // 2. If free space is negative the "safe" alignment variants all fallback to Start alignment
            if free_space <= Au::zero() && is_safe {
                resolved_justify_content = AlignFlags::START;
            }

            resolved_justify_content
        };

        // Implement "unsafe" alignment. "safe" alignment is handled by the fallback process above.
        let main_start_position = match resolved_justify_content {
            AlignFlags::START => Au::zero(),
            AlignFlags::FLEX_START => {
                if layout_is_flex_reversed {
                    free_space
                } else {
                    Au::zero()
                }
            },
            AlignFlags::END => free_space,
            AlignFlags::FLEX_END => {
                if layout_is_flex_reversed {
                    Au::zero()
                } else {
                    free_space
                }
            },
            AlignFlags::CENTER => free_space / 2,
            AlignFlags::STRETCH => Au::zero(),
            AlignFlags::SPACE_BETWEEN => Au::zero(),
            AlignFlags::SPACE_AROUND => (free_space / item_count as i32) / 2,
            AlignFlags::SPACE_EVENLY => free_space / (item_count + 1) as i32,

            // TODO: Implement all alignments. Note: not all alignment values are valid for content distribution
            _ => Au::zero(),
        };

        // TODO: Implement gap property
        let item_main_interval = /*gap + */ match resolved_justify_content {
            AlignFlags::START => Au::zero(),
            AlignFlags::FLEX_START => Au::zero(),
            AlignFlags::END => Au::zero(),
            AlignFlags::FLEX_END => Au::zero(),
            AlignFlags::CENTER => Au::zero(),
            AlignFlags::STRETCH => Au::zero(),
            AlignFlags::SPACE_BETWEEN => free_space / (item_count - 1) as i32,
            AlignFlags::SPACE_AROUND => free_space / item_count as i32,
            AlignFlags::SPACE_EVENLY => free_space / (item_count + 1) as i32,

            // TODO: Implement all alignments. Note: not all alignment values are valid for content distribution
            _ => Au::zero(),
        };

        let mut main_position_cursor = main_start_position;
        let item_fragments = izip!(
            self.items.iter(),
            item_main_margins,
            item_cross_margins,
            &item_used_main_sizes,
            &item_used_cross_sizes,
            item_layout_results.into_iter()
        )
        .map(
            |(
                item,
                item_main_margins,
                item_cross_margins,
                item_used_main_size,
                item_used_cross_size,
                item_layout_result,
            )| {
                let item_margin = FlexRelativeSides {
                    main_start: item_main_margins.0,
                    main_end: item_main_margins.1,
                    cross_start: item_cross_margins.0,
                    cross_end: item_cross_margins.1,
                };

                // https://drafts.csswg.org/css-flexbox/#algo-main-align
                // “Align the items along the main-axis”
                main_position_cursor +=
                    item_margin.main_start + item.border.main_start + item.padding.main_start;
                let item_content_main_start_position = main_position_cursor;
                main_position_cursor += *item_used_main_size +
                    item.padding.main_end +
                    item.border.main_end +
                    item_margin.main_end +
                    item_main_interval;

                // https://drafts.csswg.org/css-flexbox/#algo-cross-align
                let item_content_cross_start_position = item.align_along_cross_axis(
                    &item_margin,
                    item_used_cross_size,
                    line_cross_size,
                    item_layout_result
                        .baseline_relative_to_margin_box
                        .unwrap_or_default(),
                    max_baseline.unwrap_or_default(),
                );

                let start_corner = FlexRelativeVec2 {
                    main: item_content_main_start_position,
                    cross: item_content_cross_start_position,
                };
                let size = FlexRelativeVec2 {
                    main: *item_used_main_size,
                    cross: *item_used_cross_size,
                };

                let content_rect = flex_context
                    .rect_to_flow_relative(line_size, FlexRelativeRect { start_corner, size });
                let margin = flex_context.sides_to_flow_relative(item_margin);
                let collapsed_margin = CollapsedBlockMargins::from_margin(&margin);

                // TODO: We should likely propagate baselines from `display: flex`.
                (
                    BoxFragment::new(
                        item.box_.base_fragment_info(),
                        item.box_.style().clone(),
                        item_layout_result.fragments,
                        content_rect,
                        flex_context.sides_to_flow_relative(item.padding),
                        flex_context.sides_to_flow_relative(item.border),
                        margin,
                        None, /* clearance */
                        collapsed_margin,
                    ),
                    item_layout_result.positioning_context,
                )
            },
        )
        .collect();

        FlexLineLayoutResult {
            cross_size: line_cross_size,
            item_fragments,
        }
    }

    /// Return the *main size* of each item, and the line’s remainaing free space
    /// <https://drafts.csswg.org/css-flexbox/#resolve-flexible-lengths>
    fn resolve_flexible_lengths(&self, container_main_size: Au) -> (Vec<Au>, Au) {
        let mut frozen = vec![false; self.items.len()];
        let mut target_main_sizes_vec = self
            .items
            .iter()
            .map(|item| item.flex_base_size)
            .collect::<Vec<_>>();

        // Using `Cell`s reconciles mutability with multiple borrows in closures
        let target_main_sizes = Cell::from_mut(&mut *target_main_sizes_vec).as_slice_of_cells();
        let frozen = Cell::from_mut(&mut *frozen).as_slice_of_cells();
        let frozen_count = Cell::new(0);

        let grow = self.outer_hypothetical_main_sizes_sum < container_main_size;
        let flex_factor = |item: &FlexItem| {
            let position_style = item.box_.style().get_position();
            if grow {
                position_style.flex_grow.0
            } else {
                position_style.flex_shrink.0
            }
        };
        let items = || self.items.iter().zip(target_main_sizes).zip(frozen);

        // “Size inflexible items”
        for ((item, target_main_size), frozen) in items() {
            let is_inflexible = flex_factor(item) == 0. ||
                if grow {
                    item.flex_base_size > item.hypothetical_main_size
                } else {
                    item.flex_base_size < item.hypothetical_main_size
                };
            if is_inflexible {
                frozen_count.set(frozen_count.get() + 1);
                frozen.set(true);
                target_main_size.set(item.hypothetical_main_size);
            }
        }

        let check_for_flexible_items = || frozen_count.get() < self.items.len();
        let free_space = || {
            container_main_size -
                items()
                    .map(|((item, target_main_size), frozen)| {
                        item.pbm_auto_is_zero.main +
                            if frozen.get() {
                                target_main_size.get()
                            } else {
                                item.flex_base_size
                            }
                    })
                    .sum()
        };
        // https://drafts.csswg.org/css-flexbox/#initial-free-space
        let initial_free_space = free_space();
        let unfrozen_items = || {
            items().filter_map(|(item_and_target_main_size, frozen)| {
                if !frozen.get() {
                    Some(item_and_target_main_size)
                } else {
                    None
                }
            })
        };
        loop {
            // https://drafts.csswg.org/css-flexbox/#remaining-free-space
            let mut remaining_free_space = free_space();
            if !check_for_flexible_items() {
                return (target_main_sizes_vec, remaining_free_space);
            }
            let unfrozen_items_flex_factor_sum: f32 =
                unfrozen_items().map(|(item, _)| flex_factor(item)).sum();
            // FIXME: I (Simon) transcribed the spec but I don’t yet understand why this algorithm
            if unfrozen_items_flex_factor_sum < 1. {
                let multiplied = initial_free_space.scale_by(unfrozen_items_flex_factor_sum);
                if multiplied.abs() < remaining_free_space.abs() {
                    remaining_free_space = multiplied
                }
            }

            // “Distribute free space proportional to the flex factors.”
            // FIXME: is it a problem if floating point precision errors accumulate
            // and we get not-quite-zero remaining free space when we should get zero here?
            if remaining_free_space != Au::zero() {
                if grow {
                    for (item, target_main_size) in unfrozen_items() {
                        let grow_factor = item.box_.style().get_position().flex_grow.0;
                        let ratio = grow_factor / unfrozen_items_flex_factor_sum;
                        target_main_size
                            .set(item.flex_base_size + remaining_free_space.scale_by(ratio));
                    }
                } else {
                    // https://drafts.csswg.org/css-flexbox/#scaled-flex-shrink-factor
                    let scaled_shrink_factor = |item: &FlexItem| {
                        let shrink_factor = item.box_.style().get_position().flex_shrink.0;
                        item.flex_base_size.scale_by(shrink_factor)
                    };
                    let scaled_shrink_factors_sum: Au = unfrozen_items()
                        .map(|(item, _)| scaled_shrink_factor(item))
                        .sum();
                    if scaled_shrink_factors_sum > Au::zero() {
                        for (item, target_main_size) in unfrozen_items() {
                            let ratio = scaled_shrink_factor(item).0 as f32 /
                                scaled_shrink_factors_sum.0 as f32;
                            target_main_size.set(
                                item.flex_base_size - remaining_free_space.abs().scale_by(ratio),
                            );
                        }
                    }
                }
            }

            // “Fix min/max violations.”
            let violation = |(item, target_main_size): (&FlexItem, &Cell<Au>)| {
                let size = target_main_size.get();
                let clamped = size.clamp_between_extremums(
                    item.content_min_size.main,
                    item.content_max_size.main,
                );
                clamped - size
            };

            // “Freeze over-flexed items.”
            let total_violation: Au = unfrozen_items().map(violation).sum();
            match total_violation.cmp(&Au::zero()) {
                Ordering::Equal => {
                    // “Freeze all items.”
                    // Return instead, as that’s what the next loop iteration would do.
                    let remaining_free_space =
                        container_main_size - target_main_sizes_vec.iter().cloned().sum();
                    return (target_main_sizes_vec, remaining_free_space);
                },
                Ordering::Greater => {
                    // “Freeze all the items with min violations.”
                    // “If the item’s target main size was made larger by [clamping],
                    //  it’s a min violation.”
                    for (item_and_target_main_size, frozen) in items() {
                        if violation(item_and_target_main_size) > Au::zero() {
                            let (item, target_main_size) = item_and_target_main_size;
                            target_main_size.set(item.content_min_size.main);
                            frozen_count.set(frozen_count.get() + 1);
                            frozen.set(true);
                        }
                    }
                },
                Ordering::Less => {
                    // Negative total violation
                    // “Freeze all the items with max violations.”
                    // “If the item’s target main size was made smaller by [clamping],
                    //  it’s a max violation.”
                    for (item_and_target_main_size, frozen) in items() {
                        if violation(item_and_target_main_size) < Au::zero() {
                            let (item, target_main_size) = item_and_target_main_size;
                            let Some(max_size) = item.content_max_size.main else {
                                unreachable!()
                            };
                            target_main_size.set(max_size);
                            frozen_count.set(frozen_count.get() + 1);
                            frozen.set(true);
                        }
                    }
                },
            }
        }
    }
}

impl<'a> FlexItem<'a> {
    // Return the hypothetical cross size together with laid out contents of the fragment.
    // https://drafts.csswg.org/css-flexbox/#algo-cross-item
    // “performing layout as if it were an in-flow block-level box
    //  with the used main size and the given available space, treating `auto` as `fit-content`.”
    fn layout(
        &mut self,
        used_main_size: Au,
        flex_context: &mut FlexContext,
        used_cross_size_override: Option<Au>,
    ) -> FlexItemLayoutResult {
        let mut positioning_context = PositioningContext::new_for_subtree(
            flex_context
                .positioning_context
                .collects_for_nearest_positioned_ancestor(),
        );
        match flex_context.flex_axis {
            FlexAxis::Row => {
                // The main axis is the container’s inline axis

                // https://drafts.csswg.org/css-writing-modes/#orthogonal-flows
                assert_eq!(
                    flex_context.containing_block.style.writing_mode,
                    self.box_.style().writing_mode,
                    "Mixed writing modes are not supported yet"
                );
                // … and also the item’s inline axis.

                match self.box_ {
                    IndependentFormattingContext::Replaced(replaced) => {
                        let pbm = replaced
                            .style
                            .padding_border_margin(flex_context.containing_block);
                        let box_size = used_cross_size_override.map(|size| LogicalVec2 {
                            inline: replaced
                                .style
                                .content_box_size(flex_context.containing_block, &pbm)
                                .inline
                                .map(Au::from),
                            block: AuOrAuto::LengthPercentage(size),
                        });
                        let size = replaced.contents.used_size_as_if_inline_element(
                            flex_context.containing_block,
                            &replaced.style,
                            box_size,
                            &pbm,
                        );
                        let cross_size = flex_context.vec2_to_flex_relative(size).cross;
                        let fragments = replaced.contents.make_fragments(&replaced.style, size);

                        FlexItemLayoutResult {
                            hypothetical_cross_size: cross_size,
                            fragments,
                            positioning_context,

                            // We will need to synthesize the baseline, but since the used cross
                            // size can differ from the hypothetical cross size, we should defer
                            // synthesizing until needed.
                            baseline_relative_to_margin_box: None,
                        }
                    },
                    IndependentFormattingContext::NonReplaced(non_replaced) => {
                        let block_size = match used_cross_size_override {
                            Some(s) => AuOrAuto::LengthPercentage(s),
                            None => self.content_box_size.cross.map(|t| t),
                        };

                        let item_as_containing_block = ContainingBlock {
                            inline_size: used_main_size,
                            block_size,
                            style: &non_replaced.style,
                        };
                        let IndependentLayout {
                            fragments,
                            content_block_size,
                            baselines: content_box_baselines,
                            ..
                        } = non_replaced.layout(
                            flex_context.layout_context,
                            &mut positioning_context,
                            &item_as_containing_block,
                            flex_context.containing_block,
                        );

                        let baselines_relative_to_margin_box =
                            self.layout_baselines_relative_to_margin_box(&content_box_baselines);

                        let baseline_relative_to_margin_box = match self.align_self.0.value() {
                            // ‘baseline’ computes to ‘first baseline’.
                            AlignFlags::BASELINE => baselines_relative_to_margin_box.first,
                            AlignFlags::LAST_BASELINE => baselines_relative_to_margin_box.last,
                            _ => None,
                        };

                        let hypothetical_cross_size = self
                            .content_box_size
                            .cross
                            .auto_is(|| content_block_size)
                            .clamp_between_extremums(
                                self.content_min_size.cross,
                                self.content_max_size.cross,
                            );

                        FlexItemLayoutResult {
                            hypothetical_cross_size,
                            fragments,
                            positioning_context,
                            baseline_relative_to_margin_box,
                        }
                    },
                }
            },
            FlexAxis::Column => {
                todo!()
                // Note: this panic shouldn’t happen since the start of `FlexContainer::layout`
                // forces `FlexAxis::Row`.
            },
        }
    }

    fn layout_baselines_relative_to_margin_box(
        &self,
        baselines_relative_to_content_box: &Baselines,
    ) -> Baselines {
        baselines_relative_to_content_box.offset(
            self.margin.cross_start.auto_is(Au::zero) +
                self.padding.cross_start +
                self.border.cross_start,
        )
    }

    fn synthesized_baseline_relative_to_margin_box(&self, content_size: Au) -> Au {
        // If the item does not have a baseline in the necessary axis,
        // then one is synthesized from the flex item’s border box.
        // https://drafts.csswg.org/css-flexbox/#valdef-align-items-baseline
        content_size +
            self.margin.cross_start.auto_is(Au::zero) +
            self.padding.cross_start +
            self.border.cross_start +
            self.border.cross_end +
            self.padding.cross_end
    }
}

impl<'items> FlexLine<'items> {
    /// <https://drafts.csswg.org/css-flexbox/#algo-cross-line>
    fn cross_size(
        &self,
        item_layout_results: &[FlexItemLayoutResult],
        flex_context: &FlexContext,
    ) -> Au {
        if flex_context.container_is_single_line {
            if let Some(size) = flex_context.container_definite_inner_size.cross {
                return size;
            }
        }

        let mut max_ascent = Au::zero();
        let mut max_descent = Au::zero();
        let mut max_outer_hypothetical_cross_size = Au::zero();
        for (item_result, item) in item_layout_results.iter().zip(&*self.items) {
            // TODO: check inline-axis is parallel to main axis, check no auto cross margins
            if matches!(
                item.align_self.0.value(),
                AlignFlags::BASELINE | AlignFlags::LAST_BASELINE
            ) {
                let baseline = item_result
                    .baseline_relative_to_margin_box
                    .unwrap_or_else(|| {
                        item.synthesized_baseline_relative_to_margin_box(
                            item_result.hypothetical_cross_size,
                        )
                    });
                let hypothetical_margin_box_cross_size =
                    item_result.hypothetical_cross_size + item.pbm_auto_is_zero.cross;
                max_ascent = max_ascent.max(baseline);
                max_descent = max_descent.max(hypothetical_margin_box_cross_size - baseline);
            } else {
                max_outer_hypothetical_cross_size = max_outer_hypothetical_cross_size
                    .max(item_result.hypothetical_cross_size + item.pbm_auto_is_zero.cross);
            }
        }

        // FIXME: add support for `align-self: baseline`
        // and computing the baseline of flex items.
        // https://drafts.csswg.org/css-flexbox/#baseline-participation
        let largest = max_outer_hypothetical_cross_size.max(max_ascent + max_descent);
        if flex_context.container_is_single_line {
            largest.clamp_between_extremums(
                flex_context.container_min_cross_size,
                flex_context.container_max_cross_size,
            )
        } else {
            largest
        }
    }

    // Return the main-start and main-end margin of each item in the line,
    // with `auto` values resolved,
    // and return whether free space has been distributed.
    fn resolve_auto_main_margins(
        &self,
        remaining_free_space: Au,
    ) -> (impl Iterator<Item = (Au, Au)> + '_, bool) {
        let each_auto_margin = if remaining_free_space > Au::zero() {
            let auto_margins_count = self
                .items
                .iter()
                .map(|item| {
                    item.margin.main_start.is_auto() as u32 + item.margin.main_end.is_auto() as u32
                })
                .sum::<u32>();
            if auto_margins_count > 0 {
                remaining_free_space / auto_margins_count as i32
            } else {
                Au::zero()
            }
        } else {
            Au::zero()
        };
        (
            self.items.iter().map(move |item| {
                (
                    item.margin.main_start.auto_is(|| each_auto_margin),
                    item.margin.main_end.auto_is(|| each_auto_margin),
                )
            }),
            each_auto_margin > Au::zero(),
        )
    }
}

impl FlexItem<'_> {
    /// Return the cross-start and cross-end margin, with `auto` values resolved.
    /// <https://drafts.csswg.org/css-flexbox/#algo-cross-margins>
    fn resolve_auto_cross_margins(
        &self,
        flex_context: &FlexContext,
        line_cross_size: Au,
        item_cross_content_size: Au,
    ) -> (Au, Au) {
        let auto_count = match (self.margin.cross_start, self.margin.cross_end) {
            (AuOrAuto::LengthPercentage(start), AuOrAuto::LengthPercentage(end)) => {
                return (start, end);
            },
            (AuOrAuto::Auto, AuOrAuto::Auto) => 2,
            _ => 1,
        };
        let outer_size = self.pbm_auto_is_zero.cross + item_cross_content_size;
        let available = line_cross_size - outer_size;
        let start;
        let end;
        if available > Au::zero() {
            let each_auto_margin = available / auto_count;
            start = self.margin.cross_start.auto_is(|| each_auto_margin);
            end = self.margin.cross_end.auto_is(|| each_auto_margin);
        } else {
            // “the block-start or inline-start margin (whichever is in the cross axis)”
            // This margin is the cross-end on iff `flex-wrap` is `wrap-reverse`,
            // cross-start otherwise.
            // We know this because:
            // https://drafts.csswg.org/css-flexbox/#flex-wrap-property
            // “For the values that are not wrap-reverse,
            //  the cross-start direction is equivalent to
            //  either the inline-start or block-start direction of the current writing mode
            //  (whichever is in the cross axis)
            //  and the cross-end direction is the opposite direction of cross-start.
            //  When flex-wrap is wrap-reverse,
            //  the cross-start and cross-end directions are swapped.”
            let flex_wrap = flex_context.containing_block.style.get_position().flex_wrap;
            let flex_wrap_reverse = match flex_wrap {
                FlexWrap::Nowrap | FlexWrap::Wrap => false,
                FlexWrap::WrapReverse => true,
            };
            // “if the block-start or inline-start margin (whichever is in the cross axis) is auto,
            //  set it to zero. Set the opposite margin so that the outer cross size of the item
            //  equals the cross size of its flex line.”
            if flex_wrap_reverse {
                start = self.margin.cross_start.auto_is(|| available);
                end = self.margin.cross_end.auto_is(Au::zero);
            } else {
                start = self.margin.cross_start.auto_is(Au::zero);
                end = self.margin.cross_end.auto_is(|| available);
            }
        }
        (start, end)
    }

    /// Return the coordinate of the cross-start side of the content area
    fn align_along_cross_axis(
        &self,
        margin: &FlexRelativeSides<Au>,
        used_cross_size: &Au,
        line_cross_size: Au,
        propagated_baseline: Au,
        max_propagated_baseline: Au,
    ) -> Au {
        let outer_cross_start =
            if self.margin.cross_start.is_auto() || self.margin.cross_end.is_auto() {
                Au::zero()
            } else {
                match self.align_self.0.value() {
                    AlignFlags::STRETCH | AlignFlags::FLEX_START => Au::zero(),
                    AlignFlags::FLEX_END => {
                        let margin_box_cross = *used_cross_size + self.pbm_auto_is_zero.cross;
                        line_cross_size - margin_box_cross
                    },
                    AlignFlags::CENTER => {
                        let margin_box_cross = *used_cross_size + self.pbm_auto_is_zero.cross;
                        (line_cross_size - margin_box_cross) / 2
                    },
                    AlignFlags::BASELINE | AlignFlags::LAST_BASELINE => {
                        max_propagated_baseline - propagated_baseline
                    },
                    _ => Au::zero(),
                }
            };
        outer_cross_start + margin.cross_start + self.border.cross_start + self.padding.cross_start
    }
}
