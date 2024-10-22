/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::cmp::Ordering;

use app_units::Au;
use atomic_refcell::AtomicRefMut;
use itertools::izip;
use style::computed_values::position::T as Position;
use style::logical_geometry::Direction;
use style::properties::longhands::align_items::computed_value::T as AlignItems;
use style::properties::longhands::box_sizing::computed_value::T as BoxSizing;
use style::properties::longhands::flex_wrap::computed_value::T as FlexWrap;
use style::properties::ComputedValues;
use style::values::computed::length::Size;
use style::values::generics::flex::GenericFlexBasis as FlexBasis;
use style::values::generics::length::{GenericLengthPercentageOrAuto, LengthPercentageOrNormal};
use style::values::specified::align::AlignFlags;
use style::Zero;

use super::geom::{FlexAxis, FlexRelativeRect, FlexRelativeSides, FlexRelativeVec2};
use super::{
    FlexContainer, FlexContainerConfig, FlexItemBox, FlexItemLayoutCache,
    FlexItemLayoutCacheDescriptor, FlexLevelBox,
};
use crate::cell::ArcRefCell;
use crate::context::LayoutContext;
use crate::formatting_contexts::{Baselines, IndependentFormattingContext, IndependentLayout};
use crate::fragment_tree::{BoxFragment, CollapsedBlockMargins, Fragment, FragmentFlags};
use crate::geom::{AuOrAuto, LogicalRect, LogicalSides, LogicalVec2};
use crate::positioned::{
    relative_adjustement, AbsolutelyPositionedBox, PositioningContext, PositioningContextLength,
};
use crate::sizing::{ContentSizes, InlineContentSizesResult, IntrinsicSizingMode};
use crate::style_ext::{Clamp, ComputedValuesExt, PaddingBorderMargin};
use crate::{ContainingBlock, IndefiniteContainingBlock};

// FIMXE: “Flex items […] `z-index` values other than `auto` create a stacking context
// even if `position` is `static` (behaving exactly as if `position` were `relative`).”
// https://drafts.csswg.org/css-flexbox/#painting
// (likely in `display_list/stacking_context.rs`)

/// Layout parameters and intermediate results about a flex container,
/// grouped to avoid passing around many parameters
struct FlexContext<'a> {
    config: FlexContainerConfig,
    layout_context: &'a LayoutContext<'a>,
    positioning_context: &'a mut PositioningContext,
    containing_block: &'a ContainingBlock<'a>, // For items
    container_min_cross_size: Au,
    container_max_cross_size: Option<Au>,
    container_definite_inner_size: FlexRelativeVec2<Option<Au>>,
}

/// A flex item with some intermediate results
struct FlexItem<'a> {
    box_: &'a mut FlexItemBox,
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

    /// Whether the [`Self::flex_base_size`] comes from a definite `flex-basis`.
    /// If false and the container main size is also indefinite, percentages in the item's
    /// content that resolve against its main size should be indefinite.
    flex_base_size_is_definite: bool,

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

/// Return type of `FlexItem::layout`
struct FlexItemLayoutResult {
    hypothetical_cross_size: Au,
    fragments: Vec<Fragment>,
    positioning_context: PositioningContext,

    // Either the first or the last baseline, depending on ‘align-self’.
    baseline_relative_to_margin_box: Option<Au>,
}

impl FlexItemLayoutResult {
    fn get_or_synthesize_baseline_with_cross_size(&self, cross_size: Au, item: &FlexItem) -> Au {
        self.baseline_relative_to_margin_box
            .unwrap_or_else(|| item.synthesized_baseline_relative_to_margin_box(cross_size))
    }

    #[allow(clippy::too_many_arguments)]
    fn collect_fragment(
        mut self,
        initial_flex_layout: &InitialFlexLineLayout,
        item: &FlexItem,
        item_used_size: FlexRelativeVec2<Au>,
        item_margin: FlexRelativeSides<Au>,
        item_main_interval: Au,
        final_line_cross_size: Au,
        shared_alignment_baseline: &Option<Au>,
        flex_context: &mut FlexContext,
        all_baselines: &mut Baselines,
        main_position_cursor: &mut Au,
    ) -> (BoxFragment, PositioningContext) {
        // https://drafts.csswg.org/css-flexbox/#algo-main-align
        // “Align the items along the main-axis”
        *main_position_cursor +=
            item_margin.main_start + item.border.main_start + item.padding.main_start;
        let item_content_main_start_position = *main_position_cursor;

        *main_position_cursor += item_used_size.main +
            item.padding.main_end +
            item.border.main_end +
            item_margin.main_end +
            item_main_interval;

        // https://drafts.csswg.org/css-flexbox/#algo-cross-align
        let item_content_cross_start_position = item.align_along_cross_axis(
            &item_margin,
            &item_used_size.cross,
            final_line_cross_size,
            self.baseline_relative_to_margin_box.unwrap_or_default(),
            shared_alignment_baseline.unwrap_or_default(),
            flex_context.config.flex_wrap_is_reversed,
        );

        let start_corner = FlexRelativeVec2 {
            main: item_content_main_start_position,
            cross: item_content_cross_start_position,
        };

        // Need to collect both baselines from baseline participation and other baselines.
        let final_line_size = FlexRelativeVec2 {
            main: initial_flex_layout.line_size.main,
            cross: final_line_cross_size,
        };
        let content_rect = flex_context.rect_to_flow_relative(
            final_line_size,
            FlexRelativeRect {
                start_corner,
                size: item_used_size,
            },
        );
        let margin = flex_context.sides_to_flow_relative(item_margin);
        let collapsed_margin = CollapsedBlockMargins::from_margin(&margin);

        if let Some(item_baseline) = self.baseline_relative_to_margin_box.as_ref() {
            let item_baseline = *item_baseline + item_content_cross_start_position -
                item.border.cross_start -
                item.padding.cross_start -
                item_margin.cross_start;
            all_baselines.first.get_or_insert(item_baseline);
            all_baselines.last = Some(item_baseline);
        }

        let mut fragment_info = item.box_.base_fragment_info();
        fragment_info.flags.insert(FragmentFlags::IS_FLEX_ITEM);
        let flags = fragment_info.flags;

        let containing_block = flex_context.containing_block;
        let container_writing_mode = containing_block.style.writing_mode;
        let style = item.box_.style();
        let mut fragment = BoxFragment::new(
            fragment_info,
            style.clone(),
            self.fragments,
            content_rect.to_physical(Some(flex_context.containing_block)),
            flex_context
                .sides_to_flow_relative(item.padding)
                .to_physical(container_writing_mode),
            flex_context
                .sides_to_flow_relative(item.border)
                .to_physical(container_writing_mode),
            margin.to_physical(container_writing_mode),
            None, /* clearance */
            collapsed_margin,
        );

        // If this flex item establishes a containing block for absolutely-positioned
        // descendants, then lay out any relevant absolutely-positioned children. This
        // will remove those children from `self.positioning_context`.
        if style.establishes_containing_block_for_absolute_descendants(flags) {
            self.positioning_context
                .layout_collected_children(flex_context.layout_context, &mut fragment);
        }

        if style.clone_position() == Position::Relative {
            fragment.content_rect.origin += relative_adjustement(style, containing_block)
                .to_physical_size(containing_block.style.writing_mode)
        }

        (fragment, self.positioning_context)
    }
}

struct InitialFlexLineLayout<'a> {
    /// The items that are placed in this line.
    items: &'a mut [FlexItem<'a>],

    /// The initial size of this flex line, not taking into account `align-content: stretch`.
    line_size: FlexRelativeVec2<Au>,

    /// The layout results of the initial layout pass of this flex line. These may be replaced
    /// if necessary due to the use of `align-content: stretch` or `align-self: stretch`.
    item_layout_results: Vec<FlexItemLayoutResult>,

    /// The used main size of each item in this line.
    item_used_main_sizes: Vec<Au>,

    /// The free space available to this line after the initial layout.
    free_space_in_main_axis: Au,
}

/// Return type of `FlexLine::layout`
struct FinalFlexLineLayout {
    /// The final cross size of this flex line.
    cross_size: Au,
    /// The [`BoxFragment`]s and [`PositioningContext`]s of all flex items,
    /// one per flex item in "order-modified document order."
    item_fragments: Vec<(BoxFragment, PositioningContext)>,
    /// The 'shared alignment baseline' of this flex line. This is the baseline used for
    /// baseline-aligned items if there are any, otherwise `None`.
    shared_alignment_baseline: Option<Au>,
    /// This is the baseline of the first and last items with compatible writing mode, regardless of
    /// whether they particpate in baseline alignement. This is used as a fallback baseline for the
    /// container, if there are no items participating in baseline alignment in the first or last
    /// flex lines.
    all_baselines: Baselines,
}

impl FlexContainerConfig {
    /// Whether an item with an `auto` preferred cross size needs to be stretched
    /// to fill the flex container.
    /// <https://drafts.csswg.org/css-flexbox/#stretched>
    fn item_with_auto_cross_size_stretches_to_container_size(
        &self,
        item_style: &ComputedValues,
        item_margin: &FlexRelativeSides<AuOrAuto>,
    ) -> bool {
        self.container_is_single_line &&
            item_with_auto_cross_size_stretches_to_line_size(
                AlignItems(self.resolve_align_self_for_child(item_style)),
                item_margin,
            )
    }

    fn resolve_reversable_flex_alignment(
        &self,
        align_flags: AlignFlags,
        reversed: bool,
    ) -> AlignFlags {
        match (align_flags.value(), reversed) {
            (AlignFlags::FLEX_START, false) => AlignFlags::START | align_flags.flags(),
            (AlignFlags::FLEX_START, true) => AlignFlags::END | align_flags.flags(),
            (AlignFlags::FLEX_END, false) => AlignFlags::END | align_flags.flags(),
            (AlignFlags::FLEX_END, true) => AlignFlags::START | align_flags.flags(),
            (_, _) => align_flags,
        }
    }

    fn resolve_align_self_for_child(&self, child_style: &ComputedValues) -> AlignFlags {
        self.resolve_reversable_flex_alignment(
            child_style
                .resolve_align_self(self.align_items, AlignItems(AlignFlags::STRETCH))
                .0,
            self.flex_wrap_is_reversed,
        )
    }

    fn resolve_justify_content_for_child(&self) -> AlignFlags {
        self.resolve_reversable_flex_alignment(
            self.justify_content.0.primary(),
            self.flex_direction_is_reversed,
        )
    }
}

impl FlexContext<'_> {
    fn vec2_to_flex_relative<T>(&self, x: LogicalVec2<T>) -> FlexRelativeVec2<T> {
        self.config.flex_axis.vec2_to_flex_relative(x)
    }

    fn sides_to_flex_relative<T>(&self, x: LogicalSides<T>) -> FlexRelativeSides<T> {
        self.config
            .main_start_cross_start_sides_are
            .sides_to_flex_relative(x)
    }

    fn sides_to_flow_relative<T>(&self, x: FlexRelativeSides<T>) -> LogicalSides<T> {
        self.config
            .main_start_cross_start_sides_are
            .sides_to_flow_relative(x)
    }

    fn rect_to_flow_relative(
        &self,
        base_rect_size: FlexRelativeVec2<Au>,
        rect: FlexRelativeRect<Au>,
    ) -> LogicalRect<Au> {
        super::geom::rect_to_flow_relative(
            self.config.flex_axis,
            self.config.main_start_cross_start_sides_are,
            base_rect_size,
            rect,
        )
    }
}

#[derive(Debug, Default)]
struct DesiredFlexFractionAndGrowOrShrinkFactor {
    desired_flex_fraction: f32,
    flex_grow_or_shrink_factor: f32,
}

#[derive(Default)]
struct FlexItemBoxInlineContentSizesInfo {
    outer_flex_base_size: Au,
    content_min_size_no_auto: FlexRelativeVec2<Au>,
    content_max_size: FlexRelativeVec2<Option<Au>>,
    pbm_auto_is_zero: FlexRelativeVec2<Au>,
    min_flex_factors: DesiredFlexFractionAndGrowOrShrinkFactor,
    max_flex_factors: DesiredFlexFractionAndGrowOrShrinkFactor,
    min_content_main_size_for_multiline_container: Au,
    depends_on_block_constraints: bool,
}

impl FlexContainer {
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            name = "FlexContainer::inline_content_sizes",
            skip_all,
            fields(servo_profiling = true)
        )
    )]
    pub fn inline_content_sizes(
        &mut self,
        layout_context: &LayoutContext,
        containing_block_for_children: &IndefiniteContainingBlock,
    ) -> InlineContentSizesResult {
        match self.config.flex_axis {
            FlexAxis::Row => {
                self.main_content_sizes(layout_context, containing_block_for_children, || {
                    unreachable!(
                        "Unexpected FlexContext query during row flex intrinsic size calculation."
                    )
                })
            },
            FlexAxis::Column => {
                self.cross_content_sizes(layout_context, containing_block_for_children)
            },
        }
    }

    fn cross_content_sizes(
        &mut self,
        layout_context: &LayoutContext,
        containing_block_for_children: &IndefiniteContainingBlock,
    ) -> InlineContentSizesResult {
        // <https://drafts.csswg.org/css-flexbox/#intrinsic-cross-sizes>
        assert_eq!(
            self.config.flex_axis,
            FlexAxis::Column,
            "The cross axis should be the inline one"
        );
        let mut sizes = ContentSizes::zero();
        let mut depends_on_block_constraints = false;
        for kid in self.children.iter() {
            let kid = &mut *kid.borrow_mut();
            match kid {
                FlexLevelBox::FlexItem(item) => {
                    // TODO: For the max-content size we should distribute items into
                    // columns, and sum the column sizes and gaps.
                    // TODO: Use the proper automatic minimum size.
                    let ifc = &mut item.independent_formatting_context;
                    let result = ifc.outer_inline_content_sizes(
                        layout_context,
                        containing_block_for_children,
                        &LogicalVec2::zero(),
                        false, /* auto_block_size_stretches_to_containing_block */
                    );
                    sizes.max_assign(result.sizes);
                    depends_on_block_constraints |= result.depends_on_block_constraints;
                },
                FlexLevelBox::OutOfFlowAbsolutelyPositionedBox(_) => {},
            }
        }
        InlineContentSizesResult {
            sizes,
            depends_on_block_constraints,
        }
    }

    fn main_content_sizes<'a>(
        &self,
        layout_context: &LayoutContext,
        containing_block_for_children: &IndefiniteContainingBlock,
        flex_context_getter: impl Fn() -> &'a FlexContext<'a>,
    ) -> InlineContentSizesResult {
        // - TODO: calculate intrinsic cross sizes when container is a column
        // (and check for ‘writing-mode’?)
        // - TODO: Collapsed flex items need to be skipped for intrinsic size calculation.

        // <https://drafts.csswg.org/css-flexbox-1/#intrinsic-main-sizes>
        // > It is calculated, considering only non-collapsed flex items, by:
        // > 1. For each flex item, subtract its outer flex base size from its max-content
        // > contribution size.
        let mut chosen_max_flex_fraction = f32::NEG_INFINITY;
        let mut chosen_min_flex_fraction = f32::NEG_INFINITY;
        let mut sum_of_flex_grow_factors = 0.0;
        let mut sum_of_flex_shrink_factors = 0.0;
        let mut item_infos = vec![];

        let container_is_horizontal = self.style.writing_mode.is_horizontal();
        for kid in self.children.iter() {
            let kid = &mut *kid.borrow_mut();
            match kid {
                FlexLevelBox::FlexItem(item) => {
                    sum_of_flex_grow_factors += item.style().get_position().flex_grow.0;
                    sum_of_flex_shrink_factors += item.style().get_position().flex_shrink.0;

                    let info = item.main_content_size_info(
                        layout_context,
                        containing_block_for_children,
                        container_is_horizontal,
                        &self.config,
                        &flex_context_getter,
                    );

                    // > 2. Place all flex items into lines of infinite length. Within
                    // > each line, find the greatest (most positive) desired flex
                    // > fraction among all the flex items. This is the line’s chosen flex
                    // > fraction.
                    chosen_max_flex_fraction =
                        chosen_max_flex_fraction.max(info.max_flex_factors.desired_flex_fraction);
                    chosen_min_flex_fraction =
                        chosen_min_flex_fraction.max(info.min_flex_factors.desired_flex_fraction);

                    item_infos.push(info)
                },
                FlexLevelBox::OutOfFlowAbsolutelyPositionedBox(_) => {},
            }
        }

        let normalize_flex_fraction = |chosen_flex_fraction| {
            if chosen_flex_fraction > 0.0 && sum_of_flex_grow_factors < 1.0 {
                // > 3. If the chosen flex fraction is positive, and the sum of the line’s
                // > flex grow factors is less than 1, > divide the chosen flex fraction by that
                // > sum.
                chosen_flex_fraction / sum_of_flex_grow_factors
            } else if chosen_flex_fraction < 0.0 && sum_of_flex_shrink_factors < 1.0 {
                // > If the chosen flex fraction is negative, and the sum of the line’s flex
                // > shrink factors is less than 1, > multiply the chosen flex fraction by that
                // > sum.
                chosen_flex_fraction * sum_of_flex_shrink_factors
            } else {
                chosen_flex_fraction
            }
        };

        let chosen_min_flex_fraction = normalize_flex_fraction(chosen_min_flex_fraction);
        let chosen_max_flex_fraction = normalize_flex_fraction(chosen_max_flex_fraction);

        let main_gap = match self.config.flex_axis {
            FlexAxis::Row => self.style.clone_column_gap(),
            FlexAxis::Column => self.style.clone_row_gap(),
        };
        let main_gap = match main_gap {
            LengthPercentageOrNormal::LengthPercentage(length_percentage) => {
                length_percentage.to_used_value(Au::zero())
            },
            LengthPercentageOrNormal::Normal => Au::zero(),
        };
        let extra_space_from_main_gap = main_gap * (item_infos.len() as i32 - 1);
        let mut container_max_content_size = extra_space_from_main_gap;
        let mut container_min_content_size = if self.config.flex_wrap == FlexWrap::Nowrap {
            extra_space_from_main_gap
        } else {
            Au::zero()
        };
        let mut container_depends_on_block_constraints = false;

        for FlexItemBoxInlineContentSizesInfo {
            outer_flex_base_size,
            content_min_size_no_auto,
            content_max_size,
            pbm_auto_is_zero,
            min_flex_factors,
            max_flex_factors,
            min_content_main_size_for_multiline_container,
            depends_on_block_constraints,
        } in item_infos.iter()
        {
            // > 4. Add each item’s flex base size to the product of its flex grow factor (scaled flex shrink
            // > factor, if shrinking) and the chosen flex fraction, then clamp that result by the max main size
            // > floored by the min main size.
            let outer_min_main_size = content_min_size_no_auto.main + pbm_auto_is_zero.main;
            let outer_max_main_size = content_max_size.main.map(|v| v + pbm_auto_is_zero.main);

            // > 5. The flex container’s max-content size is the largest sum (among all the lines) of the
            // > afore-calculated sizes of all items within a single line.
            container_max_content_size += (*outer_flex_base_size +
                Au::from_f32_px(
                    max_flex_factors.flex_grow_or_shrink_factor * chosen_max_flex_fraction,
                ))
            .clamp_between_extremums(outer_min_main_size, outer_max_main_size);

            // > The min-content main size of a single-line flex container is calculated
            // > identically to the max-content main size, except that the flex items’
            // > min-content contributions are used instead of their max-content contributions.
            //
            // > However, for a multi-line container, the min-content main size is simply the
            // > largest min-content contribution of all the non-collapsed flex items in the
            // > flex container. For this purpose, each item’s contribution is capped by the
            // > item’s flex base size if the item is not growable, floored by the item’s flex
            // > base size if the item is not shrinkable, and then further clamped by the item’s
            // > min and max main sizes.
            if self.config.flex_wrap == FlexWrap::Nowrap {
                container_min_content_size += (*outer_flex_base_size +
                    Au::from_f32_px(
                        min_flex_factors.flex_grow_or_shrink_factor * chosen_min_flex_fraction,
                    ))
                .clamp_between_extremums(outer_min_main_size, outer_max_main_size);
            } else {
                container_min_content_size
                    .max_assign(*min_content_main_size_for_multiline_container);
            }

            container_depends_on_block_constraints |= depends_on_block_constraints;
        }

        InlineContentSizesResult {
            sizes: ContentSizes {
                min_content: container_min_content_size,
                max_content: container_max_content_size,
            },
            depends_on_block_constraints: container_depends_on_block_constraints,
        }
    }

    /// <https://drafts.csswg.org/css-flexbox/#layout-algorithm>
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            name = "FlexContainer::layout",
            skip_all,
            fields(servo_profiling = true)
        )
    )]
    pub(crate) fn layout(
        &self,
        layout_context: &LayoutContext,
        positioning_context: &mut PositioningContext,
        containing_block: &ContainingBlock,
        containing_block_for_container: &ContainingBlock,
    ) -> IndependentLayout {
        let (container_min_cross_size, container_max_cross_size) =
            self.available_cross_space_for_flex_items(containing_block_for_container);
        let mut flex_context = FlexContext {
            config: self.config.clone(),
            layout_context,
            positioning_context,
            containing_block,
            container_min_cross_size,
            container_max_cross_size,
            // https://drafts.csswg.org/css-flexbox/#definite-sizes
            container_definite_inner_size: self.config.flex_axis.vec2_to_flex_relative(
                LogicalVec2 {
                    inline: Some(containing_block.inline_size),
                    block: containing_block.block_size.non_auto(),
                },
            ),
        };

        // “Determine the main size of the flex container”
        // https://drafts.csswg.org/css-flexbox/#algo-main-container
        let container_main_size = match self.config.flex_axis {
            FlexAxis::Row => containing_block.inline_size,
            FlexAxis::Column => containing_block.block_size.auto_is(|| {
                self.main_content_sizes(layout_context, &containing_block.into(), || &flex_context)
                    .sizes
                    .max_content
            }),
        };

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

        let flex_item_boxes = flex_items.iter_mut().map(|child| &mut **child);
        let mut flex_items = flex_item_boxes
            .map(|flex_item_box| FlexItem::new(&flex_context, flex_item_box))
            .collect::<Vec<_>>();

        let row_gap = self.style.clone_row_gap();
        let column_gap = self.style.clone_column_gap();
        let (cross_gap, main_gap) = match flex_context.config.flex_axis {
            FlexAxis::Row => (row_gap, column_gap),
            FlexAxis::Column => (column_gap, row_gap),
        };
        let cross_gap = match cross_gap {
            LengthPercentageOrNormal::LengthPercentage(length_percent) => length_percent
                .maybe_to_used_value(flex_context.container_definite_inner_size.cross)
                .unwrap_or_default(),
            LengthPercentageOrNormal::Normal => Au::zero(),
        };
        let main_gap = match main_gap {
            LengthPercentageOrNormal::LengthPercentage(length_percent) => length_percent
                .maybe_to_used_value(flex_context.container_definite_inner_size.main)
                .unwrap_or_default(),
            LengthPercentageOrNormal::Normal => Au::zero(),
        };

        // “Resolve the flexible lengths of all the flex items to find their *used main size*.”
        // https://drafts.csswg.org/css-flexbox/#algo-flex
        let initial_line_layouts = do_initial_flex_line_layout(
            &mut flex_context,
            container_main_size,
            &mut flex_items,
            main_gap,
        );

        let line_count = initial_line_layouts.len();
        let content_cross_size = initial_line_layouts
            .iter()
            .map(|layout| layout.line_size.cross)
            .sum::<Au>() +
            cross_gap * (line_count as i32 - 1);

        // https://drafts.csswg.org/css-flexbox/#algo-cross-container
        let container_cross_size = flex_context
            .container_definite_inner_size
            .cross
            .unwrap_or(content_cross_size)
            .clamp_between_extremums(
                flex_context.container_min_cross_size,
                flex_context.container_max_cross_size,
            );

        let container_size = FlexRelativeVec2 {
            main: container_main_size,
            cross: container_cross_size,
        };
        let content_block_size = flex_context
            .config
            .flex_axis
            .vec2_to_flow_relative(container_size)
            .block;

        let mut remaining_free_cross_space = flex_context
            .container_definite_inner_size
            .cross
            .map(|cross_size| cross_size - content_cross_size)
            .unwrap_or_default();

        // Implement fallback alignment.
        //
        // In addition to the spec at https://www.w3.org/TR/css-align-3/ this implementation follows
        // the resolution of https://github.com/w3c/csswg-drafts/issues/10154
        let num_lines = initial_line_layouts.len();
        let resolved_align_content: AlignFlags = {
            let align_content_style = flex_context.config.align_content.0.primary();

            // Inital values from the style system
            let mut resolved_align_content = align_content_style.value();
            let mut is_safe = align_content_style.flags() == AlignFlags::SAFE;

            // From https://drafts.csswg.org/css-flexbox/#algo-line-align:
            // > Some alignments can only be fulfilled in certain situations or are
            // > limited in how much space they can consume; for example, space-between
            // > can only operate when there is more than one alignment subject, and
            // > baseline alignment, once fulfilled, might not be enough to absorb all
            // > the excess space. In these cases a fallback alignment takes effect (as
            // > defined below) to fully consume the excess space.
            let fallback_is_needed = match resolved_align_content {
                _ if remaining_free_cross_space <= Au::zero() => true,
                AlignFlags::STRETCH => num_lines < 1,
                AlignFlags::SPACE_BETWEEN | AlignFlags::SPACE_AROUND | AlignFlags::SPACE_EVENLY => {
                    num_lines < 2
                },
                _ => false,
            };

            if fallback_is_needed {
                (resolved_align_content, is_safe) = match resolved_align_content {
                    AlignFlags::STRETCH => (AlignFlags::FLEX_START, true),
                    AlignFlags::SPACE_BETWEEN => (AlignFlags::FLEX_START, true),
                    AlignFlags::SPACE_AROUND => (AlignFlags::CENTER, true),
                    AlignFlags::SPACE_EVENLY => (AlignFlags::CENTER, true),
                    _ => (resolved_align_content, is_safe),
                }
            };

            // 2. If free space is negative the "safe" alignment variants all fallback to Start alignment
            if remaining_free_cross_space <= Au::zero() && is_safe {
                resolved_align_content = AlignFlags::START;
            }

            resolved_align_content
        };

        // Implement "unsafe" alignment. "safe" alignment is handled by the fallback process above.
        let flex_wrap_is_reversed = flex_context.config.flex_wrap_is_reversed;
        let resolved_align_content = self
            .config
            .resolve_reversable_flex_alignment(resolved_align_content, flex_wrap_is_reversed);
        let mut cross_start_position_cursor = match resolved_align_content {
            AlignFlags::START if flex_wrap_is_reversed => remaining_free_cross_space,
            AlignFlags::START => Au::zero(),
            AlignFlags::END if flex_wrap_is_reversed => Au::zero(),
            AlignFlags::END => remaining_free_cross_space,
            AlignFlags::CENTER => remaining_free_cross_space / 2,
            AlignFlags::STRETCH => Au::zero(),
            AlignFlags::SPACE_BETWEEN => Au::zero(),
            AlignFlags::SPACE_AROUND => remaining_free_cross_space / num_lines as i32 / 2,
            AlignFlags::SPACE_EVENLY => remaining_free_cross_space / (num_lines as i32 + 1),

            // TODO: Implement all alignments. Note: not all alignment values are valid for content distribution
            _ => Au::zero(),
        };

        let inline_axis_is_main_axis = self.config.flex_axis == FlexAxis::Row;
        let mut baseline_alignment_participating_baselines = Baselines::default();
        let mut all_baselines = Baselines::default();
        let flex_item_fragments: Vec<_> = initial_line_layouts
            .into_iter()
            .enumerate()
            .flat_map(|(index, initial_line_layout)| {
                // We call `allocate_free_cross_space_for_flex_line` for each line to avoid having
                // leftover space when the number of lines doesn't evenly divide the total free space,
                // considering the precision of app units.
                let (space_to_add_to_line, space_to_add_after_line) =
                    allocate_free_cross_space_for_flex_line(
                        resolved_align_content,
                        remaining_free_cross_space,
                        (num_lines - index) as i32,
                    );
                remaining_free_cross_space -= space_to_add_to_line + space_to_add_after_line;

                let final_line_cross_size =
                    initial_line_layout.line_size.cross + space_to_add_to_line;
                let mut final_line_layout = initial_line_layout.finish_with_final_cross_size(
                    &mut flex_context,
                    main_gap,
                    final_line_cross_size,
                );

                let line_cross_start_position = cross_start_position_cursor;
                cross_start_position_cursor = line_cross_start_position +
                    final_line_cross_size +
                    space_to_add_after_line +
                    cross_gap;

                let flow_relative_line_position =
                    match (self.config.flex_axis, flex_wrap_is_reversed) {
                        (FlexAxis::Row, false) => LogicalVec2 {
                            block: line_cross_start_position,
                            inline: Au::zero(),
                        },
                        (FlexAxis::Row, true) => LogicalVec2 {
                            block: container_cross_size -
                                line_cross_start_position -
                                final_line_layout.cross_size,
                            inline: Au::zero(),
                        },
                        (FlexAxis::Column, false) => LogicalVec2 {
                            block: Au::zero(),
                            inline: line_cross_start_position,
                        },
                        (FlexAxis::Column, true) => LogicalVec2 {
                            block: Au::zero(),
                            inline: container_cross_size -
                                line_cross_start_position -
                                final_line_cross_size,
                        },
                    };

                if inline_axis_is_main_axis {
                    let line_shared_alignment_baseline = final_line_layout
                        .shared_alignment_baseline
                        .map(|baseline| baseline + flow_relative_line_position.block);
                    if index == 0 {
                        baseline_alignment_participating_baselines.first =
                            line_shared_alignment_baseline;
                    }
                    if index == num_lines - 1 {
                        baseline_alignment_participating_baselines.last =
                            line_shared_alignment_baseline;
                    }
                }

                let line_all_baselines = final_line_layout
                    .all_baselines
                    .offset(flow_relative_line_position.block);
                if index == 0 {
                    all_baselines.first = line_all_baselines.first;
                }
                if index == num_lines - 1 {
                    all_baselines.last = line_all_baselines.last;
                }

                let physical_line_position =
                    flow_relative_line_position.to_physical_size(self.style.writing_mode);
                for (fragment, _) in &mut final_line_layout.item_fragments {
                    fragment.content_rect.origin += physical_line_position;
                }
                final_line_layout.item_fragments
            })
            .collect();

        let mut flex_item_fragments = flex_item_fragments.into_iter();
        let fragments = absolutely_positioned_items_with_original_order
            .into_iter()
            .map(|child_as_abspos| match child_as_abspos {
                FlexContent::AbsolutelyPositionedBox(absolutely_positioned_box) => self
                    .create_absolutely_positioned_flex_child_fragment(
                        absolutely_positioned_box,
                        containing_block,
                        container_size,
                        positioning_context,
                    ),
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

        let baselines = Baselines {
            first: baseline_alignment_participating_baselines
                .first
                .or(all_baselines.first),
            last: baseline_alignment_participating_baselines
                .last
                .or(all_baselines.last),
        };

        IndependentLayout {
            fragments,
            content_block_size,
            content_inline_size_for_table: None,
            baselines,
        }
    }

    /// Create a absolutely positioned flex child fragment, using the rules the
    /// specification dictates. This should take into account the alignment and
    /// justification values of the container and the child to position it within a
    /// "inset-modified containing block," which may be either the "static-position
    /// rectangle" that's calculated below or a modified version of the absolute's
    /// containing block adjusted by the insets specified in the item's style.
    ///
    /// From <https://drafts.csswg.org/css-flexbox/#abspos-items>:
    /// > The cross-axis edges of the static-position rectangle of an
    /// > absolutely-positioned child of a flex container are the content edges of the
    /// > flex container The main-axis edges of the static-position rectangle are where
    /// > the margin edges of the child would be positioned if it were the sole flex item
    /// > in the flex container, assuming both the child and the flex container were
    /// > fixed-size boxes of their used size. (For this purpose, auto margins are
    /// > treated as zero.)
    fn create_absolutely_positioned_flex_child_fragment(
        &self,
        absolutely_positioned_box: ArcRefCell<AbsolutelyPositionedBox>,
        containing_block: &ContainingBlock,
        container_size: FlexRelativeVec2<Au>,
        positioning_context: &mut PositioningContext,
    ) -> Fragment {
        let alignment = {
            let fragment = absolutely_positioned_box.borrow();
            let make_flex_only_values_directional_for_absolutes =
                |value: AlignFlags, reversed: bool| match (value.value(), reversed) {
                    (AlignFlags::NORMAL | AlignFlags::AUTO | AlignFlags::STRETCH, true) => {
                        AlignFlags::END | AlignFlags::SAFE
                    },
                    (AlignFlags::STRETCH, false) => AlignFlags::START | AlignFlags::SAFE,
                    (AlignFlags::SPACE_BETWEEN, false) => AlignFlags::START | AlignFlags::SAFE,
                    (AlignFlags::SPACE_BETWEEN, true) => AlignFlags::END | AlignFlags::SAFE,
                    _ => value,
                };
            let cross = make_flex_only_values_directional_for_absolutes(
                self.config
                    .resolve_align_self_for_child(fragment.context.style()),
                self.config.flex_wrap_is_reversed,
            );
            let main = make_flex_only_values_directional_for_absolutes(
                self.config.resolve_justify_content_for_child(),
                self.config.flex_direction_is_reversed,
            );

            FlexRelativeVec2 { cross, main }
        };
        let logical_alignment = self.config.flex_axis.vec2_to_flow_relative(alignment);

        let static_position_rect = LogicalRect {
            start_corner: LogicalVec2::zero(),
            size: self.config.flex_axis.vec2_to_flow_relative(container_size),
        }
        .to_physical(Some(containing_block));

        let hoisted_box = AbsolutelyPositionedBox::to_hoisted(
            absolutely_positioned_box,
            static_position_rect,
            logical_alignment,
            self.config.writing_mode,
        );
        let hoisted_fragment = hoisted_box.fragment.clone();
        positioning_context.push(hoisted_box);
        Fragment::AbsoluteOrFixedPositioned(hoisted_fragment)
    }

    fn available_cross_space_for_flex_items(
        &self,
        containing_block_for_container: &ContainingBlock,
    ) -> (Au, Option<Au>) {
        let pbm = self
            .style
            .padding_border_margin(containing_block_for_container);
        let max_box_size = self
            .style
            .content_max_box_size_deprecated(containing_block_for_container, &pbm);
        let min_box_size = self
            .style
            .content_min_box_size_deprecated(containing_block_for_container, &pbm)
            .auto_is(Au::zero);

        let max_box_size = self.config.flex_axis.vec2_to_flex_relative(max_box_size);
        let min_box_size = self.config.flex_axis.vec2_to_flex_relative(min_box_size);

        (min_box_size.cross, max_box_size.cross)
    }
}

/// Align all flex lines per `align-content` according to
/// <https://drafts.csswg.org/css-flexbox/#algo-line-align>. Returns the space to add to
/// each line or the space to add after each line.
fn allocate_free_cross_space_for_flex_line(
    resolved_align_content: AlignFlags,
    remaining_free_cross_space: Au,
    remaining_line_count: i32,
) -> (Au, Au) {
    if remaining_free_cross_space == Au::zero() {
        return (Au::zero(), Au::zero());
    }

    match resolved_align_content {
        AlignFlags::START => (Au::zero(), Au::zero()),
        AlignFlags::FLEX_START => (Au::zero(), Au::zero()),
        AlignFlags::END => (Au::zero(), Au::zero()),
        AlignFlags::FLEX_END => (Au::zero(), Au::zero()),
        AlignFlags::CENTER => (Au::zero(), Au::zero()),
        AlignFlags::STRETCH => (
            remaining_free_cross_space / remaining_line_count,
            Au::zero(),
        ),
        AlignFlags::SPACE_BETWEEN => {
            if remaining_line_count > 1 {
                (
                    Au::zero(),
                    remaining_free_cross_space / (remaining_line_count - 1),
                )
            } else {
                (Au::zero(), Au::zero())
            }
        },
        AlignFlags::SPACE_AROUND => (
            Au::zero(),
            remaining_free_cross_space / remaining_line_count,
        ),
        AlignFlags::SPACE_EVENLY => (
            Au::zero(),
            remaining_free_cross_space / (remaining_line_count + 1),
        ),

        // TODO: Implement all alignments. Note: not all alignment values are valid for content distribution
        _ => (Au::zero(), Au::zero()),
    }
}

impl<'a> FlexItem<'a> {
    fn new(flex_context: &FlexContext, box_: &'a mut FlexItemBox) -> Self {
        let containing_block = flex_context.containing_block;
        let parent_writing_mode = containing_block.style.writing_mode;
        let item_writing_mode = box_.style().writing_mode;

        let container_is_horizontal = parent_writing_mode.is_horizontal();
        let item_is_horizontal = item_writing_mode.is_horizontal();
        let cross_axis_is_item_block_axis = cross_axis_is_item_block_axis(
            container_is_horizontal,
            item_is_horizontal,
            flex_context.config.flex_axis,
        );

        let (content_box_size, min_size, max_size, pbm, _) = box_
            .style()
            .content_box_sizes_and_padding_border_margin_deprecated(&containing_block.into());

        let margin_auto_is_zero = flex_context.sides_to_flex_relative(pbm.margin.auto_is(Au::zero));
        let padding = flex_context.sides_to_flex_relative(pbm.padding);
        let border = flex_context.sides_to_flex_relative(pbm.border);
        let margin = flex_context.sides_to_flex_relative(pbm.margin);
        let padding_border = padding.sum_by_axis() + border.sum_by_axis();
        let pbm_auto_is_zero = FlexRelativeVec2 {
            main: padding_border.main,
            cross: padding_border.cross,
        } + margin_auto_is_zero.sum_by_axis();

        let item_with_auto_cross_size_stretches_to_container_size = flex_context
            .config
            .item_with_auto_cross_size_stretches_to_container_size(box_.style(), &margin);

        let flex_relative_content_box_size = flex_context.vec2_to_flex_relative(content_box_size);
        let flex_relative_content_max_size = flex_context.vec2_to_flex_relative(max_size);
        let flex_relative_content_min_size = flex_context.vec2_to_flex_relative(min_size);
        let flex_relative_content_min_size = FlexRelativeVec2 {
            main: flex_relative_content_min_size.main.auto_is(|| {
                box_.automatic_min_size(
                    flex_context.layout_context,
                    &containing_block.into(),
                    cross_axis_is_item_block_axis,
                    flex_relative_content_box_size,
                    flex_relative_content_min_size,
                    flex_relative_content_max_size,
                    &pbm_auto_is_zero,
                    item_with_auto_cross_size_stretches_to_container_size,
                    |item| {
                        let min_size_auto_is_zero = min_size.auto_is(Au::zero);

                        item.layout_for_block_content_size(
                            flex_context,
                            &pbm,
                            content_box_size,
                            min_size_auto_is_zero,
                            max_size,
                            item_with_auto_cross_size_stretches_to_container_size,
                            IntrinsicSizingMode::Size,
                        )
                    },
                )
            }),
            cross: flex_relative_content_min_size.cross.auto_is(Au::zero),
        };

        let align_self = AlignItems(
            flex_context
                .config
                .resolve_align_self_for_child(box_.style()),
        );
        let (flex_base_size, flex_base_size_is_definite) = box_.flex_base_size(
            flex_context.layout_context,
            &containing_block.into(),
            flex_context.container_definite_inner_size,
            cross_axis_is_item_block_axis,
            flex_relative_content_box_size,
            flex_relative_content_min_size,
            flex_relative_content_max_size,
            padding_border,
            &pbm_auto_is_zero,
            item_with_auto_cross_size_stretches_to_container_size,
            |item| {
                let min_size = flex_context
                    .config
                    .flex_axis
                    .vec2_to_flow_relative(flex_relative_content_min_size);
                item.layout_for_block_content_size(
                    flex_context,
                    &pbm,
                    content_box_size,
                    min_size,
                    max_size,
                    item_with_auto_cross_size_stretches_to_container_size,
                    IntrinsicSizingMode::Size,
                )
            },
        );

        let hypothetical_main_size = flex_base_size.clamp_between_extremums(
            flex_relative_content_min_size.main,
            flex_relative_content_max_size.main,
        );
        let margin: FlexRelativeSides<AuOrAuto> = flex_context.sides_to_flex_relative(pbm.margin);

        Self {
            box_,
            content_box_size: flex_relative_content_box_size,
            content_min_size: flex_relative_content_min_size,
            content_max_size: flex_relative_content_max_size,
            padding,
            border,
            margin,
            pbm_auto_is_zero,
            flex_base_size,
            flex_base_size_is_definite,
            hypothetical_main_size,
            align_self,
        }
    }

    fn stretches(&self) -> bool {
        self.content_box_size.cross.is_auto() &&
            item_with_auto_cross_size_stretches_to_line_size(self.align_self, &self.margin)
    }
}

fn cross_axis_is_item_block_axis(
    container_is_horizontal: bool,
    item_is_horizontal: bool,
    flex_axis: FlexAxis,
) -> bool {
    let item_is_orthogonal = item_is_horizontal != container_is_horizontal;
    let container_is_row = flex_axis == FlexAxis::Row;

    container_is_row ^ item_is_orthogonal
}

/// Whether an item with an `auto` preferred cross size will stretched to fill the cross size of its flex line.
/// <https://drafts.csswg.org/css-flexbox/#stretched>
fn item_with_auto_cross_size_stretches_to_line_size(
    align_self: AlignItems,
    margin: &FlexRelativeSides<AuOrAuto>,
) -> bool {
    align_self.0.value() == AlignFlags::STRETCH &&
        !margin.cross_start.is_auto() &&
        !margin.cross_end.is_auto()
}

// “Collect flex items into flex lines”
// https://drafts.csswg.org/css-flexbox/#algo-line-break
fn do_initial_flex_line_layout<'items>(
    flex_context: &mut FlexContext,
    container_main_size: Au,
    mut items: &'items mut [FlexItem<'items>],
    main_gap: Au,
) -> Vec<InitialFlexLineLayout<'items>> {
    if flex_context.config.container_is_single_line {
        let outer_hypothetical_main_sizes_sum = items
            .iter()
            .map(|item| item.hypothetical_main_size + item.pbm_auto_is_zero.main)
            .sum();
        vec![InitialFlexLineLayout::new(
            flex_context,
            items,
            outer_hypothetical_main_sizes_sum,
            container_main_size,
            main_gap,
        )]
    } else {
        let mut lines = Vec::new();
        let mut line_size_so_far = Au::zero();
        let mut line_so_far_is_empty = true;
        let mut index = 0;
        while let Some(item) = items.get(index) {
            let item_size = item.hypothetical_main_size + item.pbm_auto_is_zero.main;
            let mut line_size_would_be = line_size_so_far + item_size;
            if !line_so_far_is_empty {
                line_size_would_be += main_gap;
            }
            let item_fits = line_size_would_be <= container_main_size;
            if item_fits || line_so_far_is_empty {
                line_size_so_far = line_size_would_be;
                line_so_far_is_empty = false;
                index += 1;
            } else {
                // We found something that doesn’t fit. This line ends *before* this item.
                let (line_items, rest) = items.split_at_mut(index);
                items = rest;
                lines.push(InitialFlexLineLayout::new(
                    flex_context,
                    line_items,
                    line_size_so_far,
                    container_main_size,
                    main_gap,
                ));

                // The next line has this item.
                line_size_so_far = item_size;
                index = 1;
            }
        }
        // The last line is added even without finding an item that doesn’t fit
        lines.push(InitialFlexLineLayout::new(
            flex_context,
            items,
            line_size_so_far,
            container_main_size,
            main_gap,
        ));
        lines
    }
}

impl InitialFlexLineLayout<'_> {
    fn new<'items>(
        flex_context: &mut FlexContext,
        items: &'items mut [FlexItem<'items>],
        outer_hypothetical_main_sizes_sum: Au,
        container_main_size: Au,
        main_gap: Au,
    ) -> InitialFlexLineLayout<'items> {
        let item_count = items.len();
        let (item_used_main_sizes, free_space) = Self::resolve_flexible_lengths(
            items,
            outer_hypothetical_main_sizes_sum,
            container_main_size - main_gap * (item_count as i32 - 1),
        );

        // https://drafts.csswg.org/css-flexbox/#algo-cross-item
        let item_layout_results = items
            .iter_mut()
            .zip(&item_used_main_sizes)
            .map(|(item, &used_main_size)| item.layout(used_main_size, flex_context, None))
            .collect::<Vec<_>>();

        // https://drafts.csswg.org/css-flexbox/#algo-cross-line
        let line_cross_size = Self::cross_size(items, &item_layout_results, flex_context);
        let line_size = FlexRelativeVec2 {
            main: container_main_size,
            cross: line_cross_size,
        };

        InitialFlexLineLayout {
            items,
            line_size,
            item_layout_results,
            item_used_main_sizes,
            free_space_in_main_axis: free_space,
        }
    }

    /// Return the *main size* of each item, and the line’s remainaing free space
    /// <https://drafts.csswg.org/css-flexbox/#resolve-flexible-lengths>
    fn resolve_flexible_lengths<'items>(
        items: &'items [FlexItem<'items>],
        outer_hypothetical_main_sizes_sum: Au,
        container_main_size: Au,
    ) -> (Vec<Au>, Au) {
        let mut frozen = vec![false; items.len()];
        let mut target_main_sizes_vec = items
            .iter()
            .map(|item| item.flex_base_size)
            .collect::<Vec<_>>();

        // Using `Cell`s reconciles mutability with multiple borrows in closures
        let target_main_sizes = Cell::from_mut(&mut *target_main_sizes_vec).as_slice_of_cells();
        let frozen = Cell::from_mut(&mut *frozen).as_slice_of_cells();
        let frozen_count = Cell::new(0);

        let grow = outer_hypothetical_main_sizes_sum < container_main_size;
        let flex_factor = |item: &FlexItem| {
            let position_style = item.box_.style().get_position();
            if grow {
                position_style.flex_grow.0
            } else {
                position_style.flex_shrink.0
            }
        };
        let items_and_main_sizes = || items.iter().zip(target_main_sizes).zip(frozen);

        // “Size inflexible items”
        for ((item, target_main_size), frozen) in items_and_main_sizes() {
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

        let check_for_flexible_items = || frozen_count.get() < items.len();
        let free_space = || {
            container_main_size -
                items_and_main_sizes()
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
            items_and_main_sizes().filter_map(|(item_and_target_main_size, frozen)| {
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
                    for (item_and_target_main_size, frozen) in items_and_main_sizes() {
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
                    for (item_and_target_main_size, frozen) in items_and_main_sizes() {
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

    /// <https://drafts.csswg.org/css-flexbox/#algo-cross-line>
    fn cross_size<'items>(
        items: &'items [FlexItem<'items>],
        item_layout_results: &[FlexItemLayoutResult],
        flex_context: &FlexContext,
    ) -> Au {
        if flex_context.config.container_is_single_line {
            if let Some(size) = flex_context.container_definite_inner_size.cross {
                return size;
            }
        }

        let mut max_ascent = Au::zero();
        let mut max_descent = Au::zero();
        let mut max_outer_hypothetical_cross_size = Au::zero();
        for (item_result, item) in item_layout_results.iter().zip(items) {
            // TODO: check inline-axis is parallel to main axis, check no auto cross margins
            if matches!(
                item.align_self.0.value(),
                AlignFlags::BASELINE | AlignFlags::LAST_BASELINE
            ) {
                let baseline = item_result.get_or_synthesize_baseline_with_cross_size(
                    item_result.hypothetical_cross_size,
                    item,
                );
                let hypothetical_margin_box_cross_size =
                    item_result.hypothetical_cross_size + item.pbm_auto_is_zero.cross;
                max_ascent = max_ascent.max(baseline);
                max_descent = max_descent.max(hypothetical_margin_box_cross_size - baseline);
            } else {
                max_outer_hypothetical_cross_size = max_outer_hypothetical_cross_size
                    .max(item_result.hypothetical_cross_size + item.pbm_auto_is_zero.cross);
            }
        }

        // https://drafts.csswg.org/css-flexbox/#baseline-participation
        let largest = max_outer_hypothetical_cross_size.max(max_ascent + max_descent);
        if flex_context.config.container_is_single_line {
            largest.clamp_between_extremums(
                flex_context.container_min_cross_size,
                flex_context.container_max_cross_size,
            )
        } else {
            largest
        }
    }

    fn finish_with_final_cross_size(
        mut self,
        flex_context: &mut FlexContext,
        main_gap: Au,
        final_line_cross_size: Au,
    ) -> FinalFlexLineLayout {
        // FIXME: Collapse `visibility: collapse` items
        // This involves “restart layout from the beginning” with a modified second round,
        // which will make structuring the code… interesting.
        // https://drafts.csswg.org/css-flexbox/#algo-visibility

        // Distribute any remaining main free space to auto margins according to
        // https://drafts.csswg.org/css-flexbox/#algo-main-align.
        let auto_margins_count = self
            .items
            .iter()
            .map(|item| {
                item.margin.main_start.is_auto() as u32 + item.margin.main_end.is_auto() as u32
            })
            .sum::<u32>();
        let (space_distributed_to_auto_main_margins, free_space_in_main_axis) =
            if self.free_space_in_main_axis > Au::zero() && auto_margins_count > 0 {
                (
                    self.free_space_in_main_axis / auto_margins_count as i32,
                    Au::zero(),
                )
            } else {
                (Au::zero(), self.free_space_in_main_axis)
            };

        // Determine the used cross size of each flex item
        // https://drafts.csswg.org/css-flexbox/#algo-stretch
        let item_count = self.items.len();
        let mut shared_alignment_baseline = None;
        let mut item_used_cross_sizes = Vec::with_capacity(item_count);
        let mut item_margins = Vec::with_capacity(item_count);
        for (item, item_layout_result, used_main_size) in izip!(
            self.items.iter_mut(),
            self.item_layout_results.iter_mut(),
            self.item_used_main_sizes.iter(),
        ) {
            let stretches = item.stretches();
            let used_cross_size = if stretches {
                (final_line_cross_size - item.pbm_auto_is_zero.cross).clamp_between_extremums(
                    item.content_min_size.cross,
                    item.content_max_size.cross,
                )
            } else {
                item_layout_result.hypothetical_cross_size
            };
            item_used_cross_sizes.push(used_cross_size);

            if stretches {
                // “If the flex item has `align-self: stretch`, redo layout for its contents,
                //  treating this used size as its definite cross size
                //  so that percentage-sized children can be resolved.”
                *item_layout_result =
                    item.layout(*used_main_size, flex_context, Some(used_cross_size));
            }

            let baseline = item_layout_result
                .get_or_synthesize_baseline_with_cross_size(used_cross_size, item);
            if matches!(
                item.align_self.0.value(),
                AlignFlags::BASELINE | AlignFlags::LAST_BASELINE
            ) {
                shared_alignment_baseline =
                    Some(shared_alignment_baseline.unwrap_or(baseline).max(baseline));
            }
            item_layout_result.baseline_relative_to_margin_box = Some(baseline);

            item_margins.push(item.resolve_auto_margins(
                flex_context,
                final_line_cross_size,
                used_cross_size,
                space_distributed_to_auto_main_margins,
            ));
        }

        // Align the items along the main-axis per justify-content.
        // Implement fallback alignment.
        //
        // In addition to the spec at https://www.w3.org/TR/css-align-3/ this implementation follows
        // the resolution of https://github.com/w3c/csswg-drafts/issues/10154
        let resolved_justify_content: AlignFlags = {
            let justify_content_style = flex_context.config.justify_content.0.primary();

            // Inital values from the style system
            let mut resolved_justify_content = justify_content_style.value();
            let mut is_safe = justify_content_style.flags() == AlignFlags::SAFE;

            // Fallback occurs in two cases:

            // 1. If there is only a single item being aligned and alignment is a distributed alignment keyword
            //    https://www.w3.org/TR/css-align-3/#distribution-values
            if item_count <= 1 || free_space_in_main_axis <= Au::zero() {
                (resolved_justify_content, is_safe) = match resolved_justify_content {
                    AlignFlags::STRETCH => (AlignFlags::FLEX_START, true),
                    AlignFlags::SPACE_BETWEEN => (AlignFlags::FLEX_START, true),
                    AlignFlags::SPACE_AROUND => (AlignFlags::CENTER, true),
                    AlignFlags::SPACE_EVENLY => (AlignFlags::CENTER, true),
                    _ => (resolved_justify_content, is_safe),
                }
            };

            // 2. If free space is negative the "safe" alignment variants all fallback to Start alignment
            if free_space_in_main_axis <= Au::zero() && is_safe {
                resolved_justify_content = AlignFlags::START;
            }

            resolved_justify_content
        };

        // Implement "unsafe" alignment. "safe" alignment is handled by the fallback process above.
        let main_start_position = match resolved_justify_content {
            AlignFlags::START => Au::zero(),
            AlignFlags::FLEX_START => {
                if flex_context.config.flex_direction_is_reversed {
                    free_space_in_main_axis
                } else {
                    Au::zero()
                }
            },
            AlignFlags::END => free_space_in_main_axis,
            AlignFlags::FLEX_END => {
                if flex_context.config.flex_direction_is_reversed {
                    Au::zero()
                } else {
                    free_space_in_main_axis
                }
            },
            AlignFlags::CENTER => free_space_in_main_axis / 2,
            AlignFlags::STRETCH => Au::zero(),
            AlignFlags::SPACE_BETWEEN => Au::zero(),
            AlignFlags::SPACE_AROUND => (free_space_in_main_axis / item_count as i32) / 2,
            AlignFlags::SPACE_EVENLY => free_space_in_main_axis / (item_count + 1) as i32,

            // TODO: Implement all alignments. Note: not all alignment values are valid for content distribution
            _ => Au::zero(),
        };

        let item_main_interval = match resolved_justify_content {
            AlignFlags::START => Au::zero(),
            AlignFlags::FLEX_START => Au::zero(),
            AlignFlags::END => Au::zero(),
            AlignFlags::FLEX_END => Au::zero(),
            AlignFlags::CENTER => Au::zero(),
            AlignFlags::STRETCH => Au::zero(),
            AlignFlags::SPACE_BETWEEN => free_space_in_main_axis / (item_count - 1) as i32,
            AlignFlags::SPACE_AROUND => free_space_in_main_axis / item_count as i32,
            AlignFlags::SPACE_EVENLY => free_space_in_main_axis / (item_count + 1) as i32,

            // TODO: Implement all alignments. Note: not all alignment values are valid for content distribution
            _ => Au::zero(),
        };
        let item_main_interval = item_main_interval + main_gap;

        let mut all_baselines = Baselines::default();
        let mut main_position_cursor = main_start_position;

        let items = std::mem::take(&mut self.items);
        let item_layout_results = std::mem::take(&mut self.item_layout_results);
        let item_fragments = izip!(
            items,
            item_margins,
            self.item_used_main_sizes.iter(),
            item_used_cross_sizes.iter(),
            item_layout_results.into_iter(),
        )
        .map(
            |(item, item_margin, item_used_main_size, item_used_cross_size, item_layout_result)| {
                let item_used_size = FlexRelativeVec2 {
                    main: *item_used_main_size,
                    cross: *item_used_cross_size,
                };
                item_layout_result.collect_fragment(
                    &self,
                    item,
                    item_used_size,
                    item_margin,
                    item_main_interval,
                    final_line_cross_size,
                    &shared_alignment_baseline,
                    flex_context,
                    &mut all_baselines,
                    &mut main_position_cursor,
                )
            },
        )
        .collect();

        FinalFlexLineLayout {
            cross_size: final_line_cross_size,
            item_fragments,
            all_baselines,
            shared_alignment_baseline,
        }
    }
}

impl FlexItem<'_> {
    /// Return the hypothetical cross size together with laid out contents of the fragment.
    /// From <https://drafts.csswg.org/css-flexbox/#algo-cross-item>:
    /// > performing layout as if it were an in-flow block-level box with the used main
    /// > size and the given available space, treating `auto` as `fit-content`.
    fn layout(
        &mut self,
        used_main_size: Au,
        flex_context: &FlexContext,
        used_cross_size_override: Option<Au>,
    ) -> FlexItemLayoutResult {
        // Clear any layout cache information so that it doesn't persist until the next layout.
        self.box_.cached_layout.borrow_mut().take();

        let containing_block = flex_context.containing_block;
        let mut positioning_context = PositioningContext::new_for_style(self.box_.style())
            .unwrap_or_else(|| {
                PositioningContext::new_for_subtree(
                    flex_context
                        .positioning_context
                        .collects_for_nearest_positioned_ancestor(),
                )
            });

        let cross_size = match used_cross_size_override {
            Some(s) => AuOrAuto::LengthPercentage(s),
            None => self.content_box_size.cross.map(|cross_size| {
                cross_size.clamp_between_extremums(
                    self.content_min_size.cross,
                    self.content_max_size.cross,
                )
            }),
        };

        let ifc = &mut self.box_.independent_formatting_context;
        let item_writing_mode = ifc.style().writing_mode;
        let item_is_horizontal = item_writing_mode.is_horizontal();
        let flex_axis = flex_context.config.flex_axis;
        let cross_axis_is_item_block_axis = cross_axis_is_item_block_axis(
            containing_block.style.writing_mode.is_horizontal(),
            item_is_horizontal,
            flex_axis,
        );

        let (inline_size, block_size) = if cross_axis_is_item_block_axis {
            (used_main_size, cross_size)
        } else {
            (
                cross_size.auto_is(|| {
                    let style = ifc.style().clone();
                    let containing_block_for_children =
                        IndefiniteContainingBlock::new_for_style_and_block_size(
                            &style,
                            AuOrAuto::LengthPercentage(used_main_size),
                        );
                    let content_contributions = ifc
                        .inline_content_sizes(
                            flex_context.layout_context,
                            &containing_block_for_children,
                            &containing_block.into(),
                        )
                        .sizes
                        .map(|size| {
                            size.clamp_between_extremums(
                                self.content_min_size.cross,
                                self.content_max_size.cross,
                            )
                        });
                    content_contributions
                        .shrink_to_fit(containing_block.inline_size - self.pbm_auto_is_zero.cross)
                }),
                // The main size of a flex item is considered to be definite if its flex basis is definite
                // or the flex container has a definite main size.
                // <https://drafts.csswg.org/css-flexbox-1/#definite-sizes>
                if self.flex_base_size_is_definite ||
                    flex_context.container_definite_inner_size.main.is_some()
                {
                    AuOrAuto::LengthPercentage(used_main_size)
                } else {
                    AuOrAuto::Auto
                },
            )
        };

        let container_writing_mode = containing_block.style.writing_mode;
        match ifc {
            IndependentFormattingContext::Replaced(replaced) => {
                let size = replaced
                    .contents
                    .used_size_as_if_inline_element_from_content_box_sizes(
                        containing_block,
                        &replaced.style,
                        LogicalVec2 {
                            inline: AuOrAuto::LengthPercentage(inline_size),
                            block: block_size,
                        },
                        flex_axis.vec2_to_flow_relative(self.content_min_size),
                        flex_axis.vec2_to_flow_relative(self.content_max_size),
                    );
                let cross_size = flex_axis.vec2_to_flex_relative(size).cross;
                let fragments = replaced.contents.make_fragments(
                    &replaced.style,
                    containing_block,
                    size.to_physical_size(container_writing_mode),
                );

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
                let item_as_containing_block = ContainingBlock {
                    inline_size,
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
                    containing_block,
                );

                let item_writing_mode_is_orthogonal_to_container_writing_mode =
                    flex_context.config.writing_mode.is_horizontal() !=
                        non_replaced.style.writing_mode.is_horizontal();
                let has_compatible_baseline = match flex_context.config.flex_axis {
                    FlexAxis::Row => !item_writing_mode_is_orthogonal_to_container_writing_mode,
                    FlexAxis::Column => item_writing_mode_is_orthogonal_to_container_writing_mode,
                };

                let baselines_relative_to_margin_box = if has_compatible_baseline {
                    content_box_baselines.offset(
                        self.margin.cross_start.auto_is(Au::zero) +
                            self.padding.cross_start +
                            self.border.cross_start,
                    )
                } else {
                    Baselines::default()
                };

                let baseline_relative_to_margin_box = match self.align_self.0.value() {
                    // ‘baseline’ computes to ‘first baseline’.
                    AlignFlags::BASELINE => baselines_relative_to_margin_box.first,
                    AlignFlags::LAST_BASELINE => baselines_relative_to_margin_box.last,
                    _ => None,
                };

                let hypothetical_cross_size = self
                    .content_box_size
                    .cross
                    .auto_is(|| {
                        if cross_axis_is_item_block_axis {
                            content_block_size
                        } else {
                            inline_size
                        }
                    })
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

    /// Return the cross-start, cross-end, main-start, and main-end margins, with `auto` values resolved.
    /// See:
    ///
    /// - <https://drafts.csswg.org/css-flexbox/#algo-cross-margins>
    /// - <https://drafts.csswg.org/css-flexbox/#algo-main-align>
    fn resolve_auto_margins(
        &self,
        flex_context: &FlexContext,
        line_cross_size: Au,
        item_cross_content_size: Au,
        space_distributed_to_auto_main_margins: Au,
    ) -> FlexRelativeSides<Au> {
        let main_start = self
            .margin
            .main_start
            .auto_is(|| space_distributed_to_auto_main_margins);
        let main_end = self
            .margin
            .main_end
            .auto_is(|| space_distributed_to_auto_main_margins);

        let auto_count = match (self.margin.cross_start, self.margin.cross_end) {
            (AuOrAuto::LengthPercentage(cross_start), AuOrAuto::LengthPercentage(cross_end)) => {
                return FlexRelativeSides {
                    cross_start,
                    cross_end,
                    main_start,
                    main_end,
                }
            },
            (AuOrAuto::Auto, AuOrAuto::Auto) => 2,
            _ => 1,
        };
        let outer_cross_size = self.pbm_auto_is_zero.cross + item_cross_content_size;
        let available = line_cross_size - outer_cross_size;
        let cross_start;
        let cross_end;
        if available > Au::zero() {
            let each_auto_margin = available / auto_count;
            cross_start = self.margin.cross_start.auto_is(|| each_auto_margin);
            cross_end = self.margin.cross_end.auto_is(|| each_auto_margin);
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
                cross_start = self.margin.cross_start.auto_is(|| available);
                cross_end = self.margin.cross_end.auto_is(Au::zero);
            } else {
                cross_start = self.margin.cross_start.auto_is(Au::zero);
                cross_end = self.margin.cross_end.auto_is(|| available);
            }
        }

        FlexRelativeSides {
            cross_start,
            cross_end,
            main_start,
            main_end,
        }
    }

    /// Return the coordinate of the cross-start side of the content area
    fn align_along_cross_axis(
        &self,
        margin: &FlexRelativeSides<Au>,
        used_cross_size: &Au,
        line_cross_size: Au,
        propagated_baseline: Au,
        max_propagated_baseline: Au,
        wrap_reverse: bool,
    ) -> Au {
        let ending_alignment = line_cross_size - *used_cross_size - self.pbm_auto_is_zero.cross;
        let outer_cross_start =
            if self.margin.cross_start.is_auto() || self.margin.cross_end.is_auto() {
                Au::zero()
            } else {
                match self.align_self.0.value() {
                    AlignFlags::STRETCH => Au::zero(),
                    AlignFlags::CENTER => ending_alignment / 2,
                    AlignFlags::BASELINE | AlignFlags::LAST_BASELINE => {
                        max_propagated_baseline - propagated_baseline
                    },
                    AlignFlags::START => {
                        if !wrap_reverse {
                            Au::zero()
                        } else {
                            ending_alignment
                        }
                    },
                    AlignFlags::END => {
                        if !wrap_reverse {
                            ending_alignment
                        } else {
                            Au::zero()
                        }
                    },
                    _ => Au::zero(),
                }
            };
        outer_cross_start + margin.cross_start + self.border.cross_start + self.padding.cross_start
    }
}

impl FlexItemBox {
    fn main_content_size_info<'a>(
        &mut self,
        layout_context: &LayoutContext,
        containing_block: &IndefiniteContainingBlock,
        container_is_horizontal: bool,
        config: &FlexContainerConfig,
        flex_context_getter: &impl Fn() -> &'a FlexContext<'a>,
    ) -> FlexItemBoxInlineContentSizesInfo {
        let flex_axis = config.flex_axis;
        let main_start_cross_start = config.main_start_cross_start_sides_are;
        let style = self.style().clone();
        let item_writing_mode = style.writing_mode;
        let item_is_horizontal = item_writing_mode.is_horizontal();
        let cross_axis_is_item_block_axis =
            cross_axis_is_item_block_axis(container_is_horizontal, item_is_horizontal, flex_axis);

        let (content_box_size, content_min_size, content_max_size, pbm, _) =
            style.content_box_sizes_and_padding_border_margin_deprecated(containing_block);
        let padding = main_start_cross_start.sides_to_flex_relative(pbm.padding);
        let border = main_start_cross_start.sides_to_flex_relative(pbm.border);
        let margin = main_start_cross_start.sides_to_flex_relative(pbm.margin);
        let padding_border = padding.sum_by_axis() + border.sum_by_axis();
        let margin_auto_is_zero = pbm.margin.auto_is(Au::zero);
        let margin_auto_is_zero =
            main_start_cross_start.sides_to_flex_relative(margin_auto_is_zero);
        let pbm_auto_is_zero = FlexRelativeVec2 {
            main: padding_border.main,
            cross: padding_border.cross,
        } + margin_auto_is_zero.sum_by_axis();
        let item_with_auto_cross_size_stretches_to_container_size =
            config.item_with_auto_cross_size_stretches_to_container_size(&style, &margin);
        let automatic_min_size = self.automatic_min_size(
            layout_context,
            containing_block,
            cross_axis_is_item_block_axis,
            flex_axis.vec2_to_flex_relative(content_box_size),
            flex_axis.vec2_to_flex_relative(content_min_size),
            flex_axis.vec2_to_flex_relative(content_max_size),
            &pbm_auto_is_zero,
            item_with_auto_cross_size_stretches_to_container_size,
            |item| {
                item.layout_for_block_content_size(
                    flex_context_getter(),
                    &pbm,
                    content_box_size,
                    content_min_size.map(|v| v.auto_is(Au::zero)),
                    content_max_size,
                    item_with_auto_cross_size_stretches_to_container_size,
                    IntrinsicSizingMode::Size,
                )
            },
        );
        let content_min_size_no_auto = if cross_axis_is_item_block_axis {
            LogicalVec2 {
                inline: content_min_size.inline.auto_is(|| automatic_min_size),
                block: content_min_size.block.auto_is(Au::zero),
            }
        } else {
            LogicalVec2 {
                inline: content_min_size.inline.auto_is(Au::zero),
                block: content_min_size.block.auto_is(|| automatic_min_size),
            }
        };
        let block_content_size_callback = |item: &mut FlexItemBox| {
            item.layout_for_block_content_size(
                flex_context_getter(),
                &pbm,
                content_box_size,
                content_min_size_no_auto,
                content_max_size,
                item_with_auto_cross_size_stretches_to_container_size,
                IntrinsicSizingMode::Size,
            )
        };

        // Compute the min-content and max-content contributions of the item.
        // <https://drafts.csswg.org/css-flexbox/#intrinsic-item-contributions>
        let (content_contribution_sizes, depends_on_block_constraints) = match flex_axis {
            FlexAxis::Row => {
                let InlineContentSizesResult {
                    sizes,
                    depends_on_block_constraints,
                } = self
                    .independent_formatting_context
                    .outer_inline_content_sizes(
                        layout_context,
                        containing_block,
                        &content_min_size_no_auto,
                        item_with_auto_cross_size_stretches_to_container_size,
                    );
                (sizes, depends_on_block_constraints)
            },
            FlexAxis::Column => {
                let size = self.layout_for_block_content_size(
                    flex_context_getter(),
                    &pbm,
                    content_box_size,
                    content_min_size_no_auto,
                    content_max_size,
                    item_with_auto_cross_size_stretches_to_container_size,
                    IntrinsicSizingMode::Contribution,
                );
                (size.into(), true)
            },
        };

        let content_box_size = flex_axis.vec2_to_flex_relative(content_box_size);
        let content_min_size_no_auto = flex_axis.vec2_to_flex_relative(content_min_size_no_auto);
        let content_max_size = flex_axis.vec2_to_flex_relative(content_max_size);

        // TODO: when laying out a column container with an indefinite main size,
        // we compute the base sizes of the items twice. We should consider caching.
        let (flex_base_size, _) = self.flex_base_size(
            layout_context,
            containing_block,
            config
                .flex_axis
                .vec2_to_flex_relative(containing_block.size.map(|v| v.non_auto())),
            cross_axis_is_item_block_axis,
            content_box_size,
            content_min_size_no_auto,
            content_max_size,
            padding_border,
            &pbm_auto_is_zero,
            item_with_auto_cross_size_stretches_to_container_size,
            block_content_size_callback,
        );

        let outer_flex_base_size = flex_base_size + pbm_auto_is_zero.main;
        let max_flex_factors = self.desired_flex_factors_for_preferred_width(
            content_contribution_sizes.max_content,
            flex_base_size,
            outer_flex_base_size,
        );

        // > The min-content main size of a single-line flex container is calculated
        // > identically to the max-content main size, except that the flex items’
        // > min-content contributions are used instead of their max-content contributions.
        let min_flex_factors = self.desired_flex_factors_for_preferred_width(
            content_contribution_sizes.min_content,
            flex_base_size,
            outer_flex_base_size,
        );

        // > However, for a multi-line container, the min-content main size is simply the
        // > largest min-content contribution of all the non-collapsed flex items in the
        // > flex container. For this purpose, each item’s contribution is capped by the
        // > item’s flex base size if the item is not growable, floored by the item’s flex
        // > base size if the item is not shrinkable, and then further clamped by the item’s
        // > min and max main sizes.
        let mut min_content_main_size_for_multiline_container =
            content_contribution_sizes.min_content;
        if style.get_position().flex_grow.is_zero() {
            min_content_main_size_for_multiline_container.min_assign(flex_base_size);
        }
        if style.get_position().flex_shrink.is_zero() {
            min_content_main_size_for_multiline_container.max_assign(flex_base_size);
        }
        min_content_main_size_for_multiline_container =
            min_content_main_size_for_multiline_container
                .clamp_between_extremums(content_min_size_no_auto.main, content_max_size.main);

        FlexItemBoxInlineContentSizesInfo {
            outer_flex_base_size,
            content_min_size_no_auto,
            content_max_size,
            pbm_auto_is_zero,
            min_flex_factors,
            max_flex_factors,
            min_content_main_size_for_multiline_container,
            depends_on_block_constraints,
        }
    }

    fn desired_flex_factors_for_preferred_width(
        &self,
        preferred_width: Au,
        flex_base_size: Au,
        outer_flex_base_size: Au,
    ) -> DesiredFlexFractionAndGrowOrShrinkFactor {
        let difference = (preferred_width - outer_flex_base_size).to_f32_px();
        let (flex_grow_or_scaled_flex_shrink_factor, desired_flex_fraction) = if difference > 0.0 {
            // > If that result is positive, divide it by the item’s flex
            // > grow factor if the flex grow > factor is ≥ 1, or multiply
            // > it by the flex grow factor if the flex grow factor is < 1;
            let flex_grow_factor = self.style().get_position().flex_grow.0;

            (
                flex_grow_factor,
                if flex_grow_factor >= 1.0 {
                    difference / flex_grow_factor
                } else {
                    difference * flex_grow_factor
                },
            )
        } else if difference < 0.0 {
            // > if the result is negative, divide it by the item’s scaled
            // > flex shrink factor (if dividing > by zero, treat the result
            // > as negative infinity).
            let flex_shrink_factor = self.style().get_position().flex_shrink.0;
            let scaled_flex_shrink_factor = flex_shrink_factor * flex_base_size.to_f32_px();

            (
                scaled_flex_shrink_factor,
                if scaled_flex_shrink_factor != 0.0 {
                    difference / scaled_flex_shrink_factor
                } else {
                    f32::NEG_INFINITY
                },
            )
        } else {
            (0.0, 0.0)
        };

        DesiredFlexFractionAndGrowOrShrinkFactor {
            desired_flex_fraction,
            flex_grow_or_shrink_factor: flex_grow_or_scaled_flex_shrink_factor,
        }
    }

    /// This is an implementation of <https://drafts.csswg.org/css-flexbox/#min-size-auto>.
    #[allow(clippy::too_many_arguments)]
    fn automatic_min_size(
        &mut self,
        layout_context: &LayoutContext,
        containing_block: &IndefiniteContainingBlock,
        cross_axis_is_item_block_axis: bool,
        content_box_size: FlexRelativeVec2<AuOrAuto>,
        min_size: FlexRelativeVec2<GenericLengthPercentageOrAuto<Au>>,
        max_size: FlexRelativeVec2<Option<Au>>,
        pbm_auto_is_zero: &FlexRelativeVec2<Au>,
        auto_cross_size_stretches_to_container_size: bool,
        block_content_size_callback: impl FnOnce(&mut FlexItemBox) -> Au,
    ) -> Au {
        // FIXME(stshine): Consider more situations when auto min size is not needed.
        if self
            .independent_formatting_context
            .style()
            .establishes_scroll_container()
        {
            return Au::zero();
        }

        // > **specified size suggestion**
        // > If the item’s preferred main size is definite and not automatic, then the specified
        // > size suggestion is that size. It is otherwise undefined.
        let specified_size_suggestion = content_box_size.main.non_auto();

        let (is_replaced, ratio) = match self.independent_formatting_context {
            IndependentFormattingContext::NonReplaced(_) => (false, None),
            IndependentFormattingContext::Replaced(ref replaced) => {
                (true, replaced.preferred_aspect_ratio(containing_block))
            },
        };
        let main_axis = if cross_axis_is_item_block_axis {
            Direction::Inline
        } else {
            Direction::Block
        };

        let cross_size =
            if content_box_size.cross.is_auto() && auto_cross_size_stretches_to_container_size {
                if cross_axis_is_item_block_axis {
                    containing_block.size.block
                } else {
                    containing_block.size.inline
                }
                .map(|v| v - pbm_auto_is_zero.cross)
            } else {
                content_box_size.cross
            }
            .map(|v| v.clamp_between_extremums(min_size.cross.auto_is(Au::zero), max_size.cross));

        // > **transferred size suggestion**
        // > If the item has a preferred aspect ratio and its preferred cross size is definite, then the
        // > transferred size suggestion is that size (clamped by its minimum and maximum cross sizes if they
        // > are definite), converted through the aspect ratio. It is otherwise undefined.
        let transferred_size_suggestion = match (ratio, cross_size) {
            (Some(ratio), AuOrAuto::LengthPercentage(cross_size)) => {
                Some(ratio.compute_dependent_size(main_axis, cross_size))
            },
            _ => None,
        };

        // > **content size suggestion**
        // > The content size suggestion is the min-content size in the main axis, clamped, if it has a
        // > preferred aspect ratio, by any definite minimum and maximum cross sizes converted through the
        // > aspect ratio.
        let main_content_size = if cross_axis_is_item_block_axis {
            let style = self.independent_formatting_context.style().clone();
            let containing_block_for_children =
                IndefiniteContainingBlock::new_for_style_and_block_size(&style, cross_size);
            self.independent_formatting_context
                .inline_content_sizes(
                    layout_context,
                    &containing_block_for_children,
                    containing_block,
                )
                .sizes
                .min_content
        } else {
            block_content_size_callback(self)
        };
        let content_size_suggestion = ratio
            .map(|ratio| {
                main_content_size.clamp_between_extremums(
                    ratio.compute_dependent_size(main_axis, min_size.cross.auto_is(Au::zero)),
                    max_size
                        .cross
                        .map(|l| ratio.compute_dependent_size(main_axis, l)),
                )
            })
            .unwrap_or(main_content_size);

        // > The content-based minimum size of a flex item is the smaller of its specified size
        // > suggestion and its content size suggestion if its specified size suggestion exists;
        // > otherwise, the smaller of its transferred size suggestion and its content size
        // > suggestion if the element is replaced and its transferred size suggestion exists;
        // > otherwise its content size suggestion. In all cases, the size is clamped by the maximum
        // > main size if it’s definite.
        match (specified_size_suggestion, transferred_size_suggestion) {
            (Some(specified), _) => specified.min(content_size_suggestion),
            (_, Some(transferred)) if is_replaced => transferred.min(content_size_suggestion),
            _ => content_size_suggestion,
        }
        .clamp_below_max(max_size.main)
    }

    /// <https://drafts.csswg.org/css-flexbox/#algo-main-item>
    #[allow(clippy::too_many_arguments)]
    fn flex_base_size(
        &mut self,
        layout_context: &LayoutContext,
        containing_block: &IndefiniteContainingBlock,
        container_definite_inner_size: FlexRelativeVec2<Option<Au>>,
        cross_axis_is_item_block_axis: bool,
        content_box_size: FlexRelativeVec2<AuOrAuto>,
        content_min_box_size: FlexRelativeVec2<Au>,
        content_max_box_size: FlexRelativeVec2<Option<Au>>,
        padding_border_sums: FlexRelativeVec2<Au>,
        pbm_auto_is_zero: &FlexRelativeVec2<Au>,
        item_with_auto_cross_size_stretches_to_container_size: bool,
        block_content_size_callback: impl FnOnce(&mut FlexItemBox) -> Au,
    ) -> (Au, bool) {
        let flex_item = &mut self.independent_formatting_context;
        let style = flex_item.style();

        let used_flex_basis = match &style.get_position().flex_basis {
            FlexBasis::Content => FlexBasis::Content,
            FlexBasis::Size(Size::LengthPercentage(length_percentage)) => {
                let apply_box_sizing = |length: Au| {
                    match style.get_position().box_sizing {
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
                match container_definite_inner_size.main {
                    Some(container_definite_main_size) => {
                        let length = length_percentage
                            .0
                            .to_used_value(container_definite_main_size);
                        FlexBasis::Size(apply_box_sizing(length))
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
            FlexBasis::Size(_) => {
                // “When specified on a flex item, the `auto` keyword retrieves
                //  the value of the main size property as the used `flex-basis`.”
                // TODO(#32853): Handle other intrinsic keywords.
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
                // > A. If the item has a definite used flex basis, that’s the flex base size.
                (length, true)
            },
            FlexBasis::Content => {
                // > B: If the flex item has ...
                // >   - a preferred aspect ratio,
                // >   - a used flex basis of content, and
                // >   - a definite cross size,
                // > then the flex base size is calculated from its used cross size and the flex item’s aspect ratio.
                let ratio = flex_item.preferred_aspect_ratio(containing_block);
                let main_axis = if cross_axis_is_item_block_axis {
                    Direction::Inline
                } else {
                    Direction::Block
                };
                // > If a single-line flex container has a definite cross size, the automatic preferred
                // > outer cross size of any stretched flex items is the flex container’s inner cross size
                // > (clamped to the flex item’s min and max cross size) and is considered definite.
                let cross_size = if content_box_size.cross.is_auto() &&
                    item_with_auto_cross_size_stretches_to_container_size
                {
                    container_definite_inner_size
                        .cross
                        .map(|v| v - pbm_auto_is_zero.cross)
                } else {
                    content_box_size.cross.non_auto()
                };
                if let (Some(ratio), Some(cross_size)) = (ratio, cross_size) {
                    let cross_size = cross_size.clamp_between_extremums(
                        content_min_box_size.cross,
                        content_max_box_size.cross,
                    );
                    return (ratio.compute_dependent_size(main_axis, cross_size), true);
                }

                // FIXME: implement cases C, D.

                // > E. Otherwise, size the item into the available space using its used flex basis in place of
                // > its main size, treating a value of content as max-content. If a cross size is needed to
                // > determine the main size (e.g. when the flex item’s main size is in its block axis, or when
                // > it has a preferred aspect ratio) and the flex item’s cross size is auto and not definite,
                // > in this calculation use fit-content as the flex item’s cross size. The flex base size is
                // > the item’s resulting main size.
                let flex_basis = if cross_axis_is_item_block_axis {
                    // The main axis is the inline axis, so we can get the content size from the normal
                    // preferred widths calculation.
                    let style = flex_item.style().clone();
                    let block_size = content_box_size.cross.map(|v| {
                        v.clamp_between_extremums(
                            content_min_box_size.cross,
                            content_max_box_size.cross,
                        )
                    });
                    let containing_block_for_children =
                        IndefiniteContainingBlock::new_for_style_and_block_size(&style, block_size);
                    let max_content = flex_item
                        .inline_content_sizes(
                            layout_context,
                            &containing_block_for_children,
                            containing_block,
                        )
                        .sizes
                        .max_content;
                    if let Some(ratio) = ratio {
                        max_content.clamp_between_extremums(
                            ratio.compute_dependent_size(main_axis, content_min_box_size.cross),
                            content_max_box_size
                                .cross
                                .map(|v| ratio.compute_dependent_size(main_axis, v)),
                        )
                    } else {
                        max_content
                    }
                } else {
                    block_content_size_callback(self)
                };
                (flex_basis, false)
            },
        }
    }

    #[allow(clippy::too_many_arguments)]
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            name = "FlexContainer::layout_for_block_content_size",
            skip_all,
            fields(servo_profiling = true)
        )
    )]
    fn layout_for_block_content_size(
        &mut self,
        flex_context: &FlexContext,
        padding_border_margin: &PaddingBorderMargin,
        mut content_box_size: LogicalVec2<AuOrAuto>,
        mut min_size: LogicalVec2<Au>,
        mut max_size: LogicalVec2<Option<Au>>,
        item_with_auto_cross_size_stretches_to_container_size: bool,
        intrinsic_sizing_mode: IntrinsicSizingMode,
    ) -> Au {
        let mut positioning_context = PositioningContext::new_for_subtree(
            flex_context
                .positioning_context
                .collects_for_nearest_positioned_ancestor(),
        );

        match &mut self.independent_formatting_context {
            IndependentFormattingContext::Replaced(replaced) => {
                content_box_size.inline = content_box_size.inline.map(|v| v.max(Au::zero()));
                if intrinsic_sizing_mode == IntrinsicSizingMode::Size {
                    content_box_size.block = AuOrAuto::Auto;
                    min_size.block = Au::zero();
                    max_size.block = None;
                }
                replaced
                    .contents
                    .used_size_as_if_inline_element_from_content_box_sizes(
                        flex_context.containing_block,
                        &replaced.style,
                        content_box_size,
                        min_size,
                        max_size,
                    )
                    .block
            },
            IndependentFormattingContext::NonReplaced(non_replaced) => {
                // TODO: This is wrong if the item writing mode is different from the flex
                // container's writing mode.
                let inline_size = content_box_size
                    .inline
                    .auto_is(|| {
                        let containing_block_inline_size_minus_pbm =
                            flex_context.containing_block.inline_size -
                                padding_border_margin.padding_border_sums.inline -
                                padding_border_margin.margin.inline_start.auto_is(Au::zero) -
                                padding_border_margin.margin.inline_end.auto_is(Au::zero);

                        if item_with_auto_cross_size_stretches_to_container_size {
                            containing_block_inline_size_minus_pbm
                        } else {
                            let style = non_replaced.style.clone();
                            let containing_block_for_children =
                                IndefiniteContainingBlock::new_for_style(&style);
                            non_replaced
                                .inline_content_sizes(
                                    flex_context.layout_context,
                                    &containing_block_for_children,
                                )
                                .sizes
                                .shrink_to_fit(containing_block_inline_size_minus_pbm)
                        }
                    })
                    .clamp_between_extremums(min_size.inline, max_size.inline);
                let item_as_containing_block = ContainingBlock {
                    inline_size,
                    block_size: AuOrAuto::Auto,
                    style: &non_replaced.style,
                };
                let mut content_block_size = || {
                    if let Some(cache) = &*self.cached_layout.borrow() {
                        if cache.descriptor.compatible_with_size(inline_size) {
                            return cache.descriptor.content_block_size;
                        }
                    }

                    let layout = non_replaced.layout(
                        flex_context.layout_context,
                        &mut positioning_context,
                        &item_as_containing_block,
                        flex_context.containing_block,
                    );

                    let content_block_size = layout.content_block_size;
                    *self.cached_layout.borrow_mut() = Some(FlexItemLayoutCache {
                        descriptor: FlexItemLayoutCacheDescriptor {
                            containing_block_inline_size: item_as_containing_block.inline_size,
                            content_block_size: layout.content_block_size,
                        },
                    });
                    content_block_size
                };
                match intrinsic_sizing_mode {
                    IntrinsicSizingMode::Contribution => {
                        let inner_block_size = content_box_size
                            .block
                            .auto_is(content_block_size)
                            .clamp_between_extremums(min_size.block, max_size.block);
                        inner_block_size +
                            padding_border_margin.padding_border_sums.block +
                            padding_border_margin.margin.block_start.auto_is(Au::zero) +
                            padding_border_margin.margin.block_end.auto_is(Au::zero)
                    },
                    IntrinsicSizingMode::Size => content_block_size(),
                }
            },
        }
    }
}
