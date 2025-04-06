/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, LazyCell};
use std::cmp::Ordering;

use app_units::Au;
use atomic_refcell::AtomicRef;
use itertools::izip;
use rayon::iter::{
    IndexedParallelIterator, IntoParallelRefIterator, ParallelDrainRange, ParallelIterator,
};
use style::Zero;
use style::computed_values::position::T as Position;
use style::logical_geometry::Direction;
use style::properties::ComputedValues;
use style::properties::longhands::align_items::computed_value::T as AlignItems;
use style::properties::longhands::box_sizing::computed_value::T as BoxSizing;
use style::properties::longhands::flex_direction::computed_value::T as FlexDirection;
use style::properties::longhands::flex_wrap::computed_value::T as FlexWrap;
use style::values::computed::LengthPercentage;
use style::values::generics::flex::GenericFlexBasis as FlexBasis;
use style::values::generics::length::LengthPercentageOrNormal;
use style::values::specified::align::AlignFlags;

use super::geom::{FlexAxis, FlexRelativeRect, FlexRelativeSides, FlexRelativeVec2};
use super::{FlexContainer, FlexContainerConfig, FlexItemBox, FlexLevelBox};
use crate::cell::ArcRefCell;
use crate::context::LayoutContext;
use crate::formatting_contexts::{Baselines, IndependentFormattingContextContents};
use crate::fragment_tree::{BoxFragment, CollapsedBlockMargins, Fragment, FragmentFlags};
use crate::geom::{AuOrAuto, LogicalRect, LogicalSides, LogicalVec2, Size, Sizes};
use crate::layout_box_base::CacheableLayoutResult;
use crate::positioned::{
    AbsolutelyPositionedBox, PositioningContext, PositioningContextLength, relative_adjustement,
};
use crate::sizing::{
    ComputeInlineContentSizes, ContentSizes, InlineContentSizesResult, IntrinsicSizingMode,
};
use crate::style_ext::{AspectRatio, Clamp, ComputedValuesExt, ContentBoxSizesAndPBM, LayoutStyle};
use crate::{
    ConstraintSpace, ContainingBlock, ContainingBlockSize, IndefiniteContainingBlock,
    SizeConstraint,
};

/// Layout parameters and intermediate results about a flex container,
/// grouped to avoid passing around many parameters
struct FlexContext<'a> {
    config: FlexContainerConfig,
    layout_context: &'a LayoutContext<'a>,
    positioning_context: &'a mut PositioningContext,
    containing_block: &'a ContainingBlock<'a>, // For items
    container_inner_size_constraint: FlexRelativeVec2<SizeConstraint>,
}

/// A flex item with some intermediate results
struct FlexItem<'a> {
    box_: &'a FlexItemBox,

    /// The preferred, min and max inner cross sizes. If the flex container is single-line
    /// and [`Self::cross_size_stretches_to_line`] is true, then the preferred cross size
    /// is set to [`Size::Stretch`].
    content_cross_sizes: Sizes,

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

    /// The used min main size of the flex item.
    /// <https://drafts.csswg.org/css-flexbox/#min-main-size-property>
    content_min_main_size: Au,

    /// The used max main size of the flex item.
    /// <https://drafts.csswg.org/css-flexbox/#max-main-size-property>
    content_max_main_size: Option<Au>,

    /// This is `align-self`, defaulting to `align-items` if `auto`
    align_self: AlignItems,

    /// Whether or not the size of this [`FlexItem`] depends on its block constraints.
    depends_on_block_constraints: bool,

    /// <https://drafts.csswg.org/css-sizing-4/#preferred-aspect-ratio>
    preferred_aspect_ratio: Option<AspectRatio>,

    /// Whether the preferred cross size of the item stretches to fill the flex line.
    /// This happens when the size computes to `auto`, the used value of `align-self`
    /// is `stretch`, and neither of the cross-axis margins are `auto`.
    /// <https://drafts.csswg.org/css-flexbox-1/#stretched>
    ///
    /// Note the following sizes are not sufficient:
    ///  - A size that only behaves as `auto` (like a cyclic percentage).
    ///    The computed value needs to be `auto` too.
    ///  - A `stretch` size. It stretches to the containing block, not to the line
    ///    (under discussion in <https://github.com/w3c/csswg-drafts/issues/11784>).
    cross_size_stretches_to_line: bool,
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

    // The content size of this layout. For replaced elements this is known before layout,
    // but for non-replaced it's only known after layout.
    content_size: LogicalVec2<Au>,

    // The containing block inline size used to generate this layout.
    containing_block_inline_size: Au,

    // The containing block block size used to generate this layout.
    containing_block_block_size: SizeConstraint,

    // Whether or not this layout depended on block constraints.
    depends_on_block_constraints: bool,

    // Whether or not this layout had a child that dependeded on block constraints.
    has_child_which_depends_on_block_constraints: bool,
}

impl FlexItemLayoutResult {
    fn compatible_with_containing_block_size(&self, containing_block: &ContainingBlock) -> bool {
        if containing_block.size.inline == self.containing_block_inline_size &&
            (containing_block.size.block == self.containing_block_block_size ||
                (!self.depends_on_block_constraints &&
                    !self.has_child_which_depends_on_block_constraints))
        {
            return true;
        }

        #[cfg(feature = "tracing")]
        tracing::warn!(
            name: "NonReplaced stretch cache miss",
            cached_inline = ?self.containing_block_inline_size,
            cached_block = ?self.containing_block_block_size,
            required_inline = ?containing_block.size.inline,
            required_block = ?containing_block.size.block,
            depends_on_block_constraints = self.depends_on_block_constraints,
            has_child_which_depends_on_block_constraints = self.has_child_which_depends_on_block_constraints,
        );

        false
    }

    fn compatible_with_containing_block_size_and_content_size(
        &self,
        containing_block: &ContainingBlock,
        size: LogicalVec2<Au>,
    ) -> bool {
        size == self.content_size && self.compatible_with_containing_block_size(containing_block)
    }
}

/// A data structure to hold all of the information about a flex item that has been placed
/// into a flex line. This happens once the item is laid out and its line has been determined.
struct FlexLineItem<'a> {
    /// The items that are placed in this line.
    item: FlexItem<'a>,

    /// The layout results of the initial layout pass for a flex line. These may be replaced
    /// if necessary due to the use of `align-content: stretch` or `align-self: stretch`.
    layout_result: FlexItemLayoutResult,

    /// The used main size of this item in its line.
    used_main_size: Au,
}

impl FlexLineItem<'_> {
    fn get_or_synthesize_baseline_with_cross_size(&self, cross_size: Au) -> Au {
        self.layout_result
            .baseline_relative_to_margin_box
            .unwrap_or_else(|| {
                self.item
                    .synthesized_baseline_relative_to_margin_box(cross_size)
            })
    }

    #[allow(clippy::too_many_arguments)]
    fn collect_fragment(
        mut self,
        initial_flex_layout: &InitialFlexLineLayout,
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
            item_margin.main_start + self.item.border.main_start + self.item.padding.main_start;
        let item_content_main_start_position = *main_position_cursor;

        *main_position_cursor += item_used_size.main +
            self.item.padding.main_end +
            self.item.border.main_end +
            item_margin.main_end +
            item_main_interval;

        // https://drafts.csswg.org/css-flexbox/#algo-cross-align
        let item_content_cross_start_position = self.item.align_along_cross_axis(
            &item_margin,
            &item_used_size.cross,
            final_line_cross_size,
            self.layout_result
                .baseline_relative_to_margin_box
                .unwrap_or_default(),
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

        if let Some(item_baseline) = self.layout_result.baseline_relative_to_margin_box.as_ref() {
            let item_baseline = *item_baseline + item_content_cross_start_position -
                self.item.border.cross_start -
                self.item.padding.cross_start -
                item_margin.cross_start;
            all_baselines.first.get_or_insert(item_baseline);
            all_baselines.last = Some(item_baseline);
        }

        let mut fragment_info = self.item.box_.base_fragment_info();
        fragment_info
            .flags
            .insert(FragmentFlags::IS_FLEX_OR_GRID_ITEM);
        if self.item.depends_on_block_constraints {
            fragment_info.flags.insert(
                FragmentFlags::SIZE_DEPENDS_ON_BLOCK_CONSTRAINTS_AND_CAN_BE_CHILD_OF_FLEX_ITEM,
            );
        }
        let flags = fragment_info.flags;

        let containing_block = flex_context.containing_block;
        let container_writing_mode = containing_block.style.writing_mode;
        let style = self.item.box_.style();

        let mut fragment = BoxFragment::new(
            fragment_info,
            style.clone(),
            self.layout_result.fragments,
            content_rect.as_physical(Some(flex_context.containing_block)),
            flex_context
                .sides_to_flow_relative(self.item.padding)
                .to_physical(container_writing_mode),
            flex_context
                .sides_to_flow_relative(self.item.border)
                .to_physical(container_writing_mode),
            margin.to_physical(container_writing_mode),
            None, /* clearance */
            collapsed_margin,
        );

        // If this flex item establishes a containing block for absolutely-positioned
        // descendants, then lay out any relevant absolutely-positioned children. This
        // will remove those children from `self.positioning_context`.
        if style.establishes_containing_block_for_absolute_descendants(flags) {
            self.layout_result
                .positioning_context
                .layout_collected_children(flex_context.layout_context, &mut fragment);
        }

        if style.clone_position() == Position::Relative {
            fragment.content_rect.origin += relative_adjustement(style, containing_block)
                .to_physical_size(containing_block.style.writing_mode)
        }

        (fragment, self.layout_result.positioning_context)
    }
}

/// Once the final cross size of a line is known, the line can go through their final
/// layout and this the return value. See [`InitialFlexLineLayout::finish_with_final_cross_size`].
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

    fn sides_to_flex_relative<T>(&self, sides: LogicalSides<T>) -> FlexRelativeSides<T> {
        self.main_start_cross_start_sides_are
            .sides_to_flex_relative(sides)
    }

    fn sides_to_flow_relative<T>(&self, sides: FlexRelativeSides<T>) -> LogicalSides<T> {
        self.main_start_cross_start_sides_are
            .sides_to_flow_relative(sides)
    }
}

impl FlexContext<'_> {
    #[inline]
    fn sides_to_flow_relative<T>(&self, x: FlexRelativeSides<T>) -> LogicalSides<T> {
        self.config.sides_to_flow_relative(x)
    }

    #[inline]
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
    content_min_main_size: Au,
    content_max_main_size: Option<Au>,
    pbm_auto_is_zero: FlexRelativeVec2<Au>,
    min_flex_factors: DesiredFlexFractionAndGrowOrShrinkFactor,
    max_flex_factors: DesiredFlexFractionAndGrowOrShrinkFactor,
    min_content_main_size_for_multiline_container: Au,
    depends_on_block_constraints: bool,
}

impl ComputeInlineContentSizes for FlexContainer {
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            name = "FlexContainer::compute_inline_content_sizes",
            skip_all,
            fields(servo_profiling = true),
            level = "trace",
        )
    )]
    fn compute_inline_content_sizes(
        &self,
        layout_context: &LayoutContext,
        constraint_space: &ConstraintSpace,
    ) -> InlineContentSizesResult {
        match self.config.flex_axis {
            FlexAxis::Row => {
                self.main_content_sizes(layout_context, &constraint_space.into(), || {
                    unreachable!(
                        "Unexpected FlexContext query during row flex intrinsic size calculation."
                    )
                })
            },
            FlexAxis::Column => self.cross_content_sizes(layout_context, &constraint_space.into()),
        }
    }
}

impl FlexContainer {
    fn cross_content_sizes(
        &self,
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
            let kid = &*kid.borrow();
            match kid {
                FlexLevelBox::FlexItem(item) => {
                    // TODO: For the max-content size we should distribute items into
                    // columns, and sum the column sizes and gaps.
                    // TODO: Use the proper automatic minimum size.
                    let ifc = &item.independent_formatting_context;
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

        for kid in self.children.iter() {
            let kid = &*kid.borrow();
            match kid {
                FlexLevelBox::FlexItem(item) => {
                    sum_of_flex_grow_factors += item.style().get_position().flex_grow.0;
                    sum_of_flex_shrink_factors += item.style().get_position().flex_shrink.0;

                    let info = item.main_content_size_info(
                        layout_context,
                        containing_block_for_children,
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
            content_min_main_size,
            content_max_main_size,
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
            let outer_min_main_size = *content_min_main_size + pbm_auto_is_zero.main;
            let outer_max_main_size = content_max_main_size.map(|v| v + pbm_auto_is_zero.main);

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
            fields(servo_profiling = true, self_address = self as *const _ as usize),
            level = "trace",
        )
    )]
    pub(crate) fn layout(
        &self,
        layout_context: &LayoutContext,
        positioning_context: &mut PositioningContext,
        containing_block: &ContainingBlock,
        depends_on_block_constraints: bool,
    ) -> CacheableLayoutResult {
        let depends_on_block_constraints =
            depends_on_block_constraints || self.config.flex_direction == FlexDirection::Column;

        let mut flex_context = FlexContext {
            config: self.config.clone(),
            layout_context,
            positioning_context,
            containing_block,
            // https://drafts.csswg.org/css-flexbox/#definite-sizes
            container_inner_size_constraint: self.config.flex_axis.vec2_to_flex_relative(
                LogicalVec2 {
                    inline: SizeConstraint::Definite(containing_block.size.inline),
                    block: containing_block.size.block,
                },
            ),
        };

        // “Determine the main size of the flex container”
        // https://drafts.csswg.org/css-flexbox/#algo-main-container
        let container_main_size = match self.config.flex_axis {
            FlexAxis::Row => containing_block.size.inline,
            FlexAxis::Column => match containing_block.size.block {
                SizeConstraint::Definite(size) => size,
                SizeConstraint::MinMax(min, max) => self
                    .main_content_sizes(layout_context, &containing_block.into(), || &flex_context)
                    .sizes
                    .max_content
                    .clamp_between_extremums(min, max),
            },
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
                let borrowed = arcrefcell.borrow();
                match &*borrowed {
                    FlexLevelBox::OutOfFlowAbsolutelyPositionedBox(absolutely_positioned) => {
                        FlexContent::AbsolutelyPositionedBox(absolutely_positioned.clone())
                    },
                    FlexLevelBox::FlexItem(_) => {
                        let item = AtomicRef::map(borrowed, |child| match child {
                            FlexLevelBox::FlexItem(item) => item,
                            _ => unreachable!(),
                        });
                        flex_items.push(item);
                        FlexContent::FlexItemPlaceholder
                    },
                }
            })
            .collect::<Vec<_>>();

        let flex_item_boxes = flex_items.iter().map(|child| &**child);
        let flex_items = flex_item_boxes
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
                .maybe_to_used_value(
                    flex_context
                        .container_inner_size_constraint
                        .cross
                        .to_definite(),
                )
                .unwrap_or_default(),
            LengthPercentageOrNormal::Normal => Au::zero(),
        };
        let main_gap = match main_gap {
            LengthPercentageOrNormal::LengthPercentage(length_percent) => length_percent
                .maybe_to_used_value(
                    flex_context
                        .container_inner_size_constraint
                        .main
                        .to_definite(),
                )
                .unwrap_or_default(),
            LengthPercentageOrNormal::Normal => Au::zero(),
        };

        // “Resolve the flexible lengths of all the flex items to find their *used main size*.”
        // https://drafts.csswg.org/css-flexbox/#algo-flex
        let initial_line_layouts = do_initial_flex_line_layout(
            &mut flex_context,
            container_main_size,
            flex_items,
            main_gap,
        );

        let line_count = initial_line_layouts.len();
        let content_cross_size = initial_line_layouts
            .iter()
            .map(|layout| layout.line_size.cross)
            .sum::<Au>() +
            cross_gap * (line_count as i32 - 1);

        // https://drafts.csswg.org/css-flexbox/#algo-cross-container
        let container_cross_size = match flex_context.container_inner_size_constraint.cross {
            SizeConstraint::Definite(cross_size) => cross_size,
            SizeConstraint::MinMax(min, max) => {
                content_cross_size.clamp_between_extremums(min, max)
            },
        };

        let container_size = FlexRelativeVec2 {
            main: container_main_size,
            cross: container_cross_size,
        };
        let content_block_size = flex_context
            .config
            .flex_axis
            .vec2_to_flow_relative(container_size)
            .block;

        let mut remaining_free_cross_space =
            match flex_context.container_inner_size_constraint.cross {
                SizeConstraint::Definite(cross_size) => cross_size - content_cross_size,
                _ => Au::zero(),
            };

        // Implement fallback alignment.
        //
        // In addition to the spec at https://www.w3.org/TR/css-align-3/ this implementation follows
        // the resolution of https://github.com/w3c/csswg-drafts/issues/10154
        let num_lines = initial_line_layouts.len();
        let resolved_align_content: AlignFlags = {
            // Computed value from the style system
            let align_content_style = flex_context.config.align_content.0.primary();
            let mut is_safe = align_content_style.flags() == AlignFlags::SAFE;

            // From https://drafts.csswg.org/css-align/#distribution-flex
            // > `normal` behaves as `stretch`.
            let mut resolved_align_content = match align_content_style.value() {
                AlignFlags::NORMAL => AlignFlags::STRETCH,
                align_content => align_content,
            };

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
                    let fragment = Fragment::Box(ArcRefCell::new(fragment));
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

        CacheableLayoutResult {
            fragments,
            content_block_size,
            content_inline_size_for_table: None,
            baselines,
            depends_on_block_constraints,
            specific_layout_info: None,
            collapsible_margins_in_children: CollapsedBlockMargins::zero(),
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
        .as_physical(Some(containing_block));

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

    #[inline]
    pub(crate) fn layout_style(&self) -> LayoutStyle {
        LayoutStyle::Default(&self.style)
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
    fn new(flex_context: &FlexContext, box_: &'a FlexItemBox) -> Self {
        let containing_block = IndefiniteContainingBlock::from(flex_context.containing_block);
        let content_box_sizes_and_pbm = box_
            .independent_formatting_context
            .layout_style()
            .content_box_sizes_and_padding_border_margin(&containing_block);
        box_.to_flex_item(
            flex_context.layout_context,
            &containing_block,
            &content_box_sizes_and_pbm,
            &flex_context.config,
            &|| flex_context,
        )
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

/// Whether an item with a computed preferred cross size of `auto` will stretch
/// to fill the cross size of its flex line.
/// <https://drafts.csswg.org/css-flexbox/#stretched>
fn item_with_auto_cross_size_stretches_to_line_size(
    align_self: AlignItems,
    margin: &FlexRelativeSides<AuOrAuto>,
) -> bool {
    align_self.0.value() == AlignFlags::STRETCH &&
        !margin.cross_start.is_auto() &&
        !margin.cross_end.is_auto()
}

/// “Collect flex items into flex lines”
/// <https://drafts.csswg.org/css-flexbox/#algo-line-break>
fn do_initial_flex_line_layout<'items>(
    flex_context: &mut FlexContext,
    container_main_size: Au,
    mut items: Vec<FlexItem<'items>>,
    main_gap: Au,
) -> Vec<InitialFlexLineLayout<'items>> {
    let construct_line = |(items, outer_hypothetical_main_size)| {
        InitialFlexLineLayout::new(
            flex_context,
            items,
            outer_hypothetical_main_size,
            container_main_size,
            main_gap,
        )
    };

    if flex_context.config.container_is_single_line {
        let outer_hypothetical_main_sizes_sum = items
            .iter()
            .map(|item| item.hypothetical_main_size + item.pbm_auto_is_zero.main)
            .sum();
        return vec![construct_line((items, outer_hypothetical_main_sizes_sum))];
    }

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
            continue;
        }

        // We found something that doesn’t fit. This line ends *before* this item.
        let remaining = items.split_off(index);
        lines.push((items, line_size_so_far));
        items = remaining;

        // The next line has this item.
        line_size_so_far = item_size;
        index = 1;
    }

    // We didn't reach the end of the last line, so add all remaining items there.
    lines.push((items, line_size_so_far));

    lines.par_drain(..).map(construct_line).collect()
}

/// The result of splitting the flex items into lines using their intrinsic sizes and doing an
/// initial layout of each item. A final layout still needs to happen after this is produced to
/// handle stretching.
struct InitialFlexLineLayout<'a> {
    /// The items that are placed in this line.
    items: Vec<FlexLineItem<'a>>,

    /// The initial size of this flex line, not taking into account `align-content: stretch`.
    line_size: FlexRelativeVec2<Au>,

    /// The free space available to this line after the initial layout.
    free_space_in_main_axis: Au,
}

impl InitialFlexLineLayout<'_> {
    fn new<'items>(
        flex_context: &FlexContext,
        items: Vec<FlexItem<'items>>,
        outer_hypothetical_main_sizes_sum: Au,
        container_main_size: Au,
        main_gap: Au,
    ) -> InitialFlexLineLayout<'items> {
        let item_count = items.len();
        let (item_used_main_sizes, free_space_in_main_axis) = Self::resolve_flexible_lengths(
            &items,
            outer_hypothetical_main_sizes_sum,
            container_main_size - main_gap * (item_count as i32 - 1),
        );

        // https://drafts.csswg.org/css-flexbox/#algo-cross-item
        let layout_results = items
            .par_iter()
            .zip(&item_used_main_sizes)
            .map(|(item, used_main_size)| {
                item.layout(*used_main_size, flex_context, None, None)
                    .unwrap()
            })
            .collect::<Vec<_>>();

        let items: Vec<_> = izip!(
            items.into_iter(),
            layout_results.into_iter(),
            item_used_main_sizes.into_iter()
        )
        .map(|(item, layout_result, used_main_size)| FlexLineItem {
            item,
            layout_result,
            used_main_size,
        })
        .collect();

        // https://drafts.csswg.org/css-flexbox/#algo-cross-line
        let line_cross_size = Self::cross_size(&items, flex_context);
        let line_size = FlexRelativeVec2 {
            main: container_main_size,
            cross: line_cross_size,
        };

        InitialFlexLineLayout {
            items,
            line_size,
            free_space_in_main_axis,
        }
    }

    /// Return the *main size* of each item, and the line’s remainaing free space
    /// <https://drafts.csswg.org/css-flexbox/#resolve-flexible-lengths>
    fn resolve_flexible_lengths<'items>(
        items: &'items [FlexItem<'items>],
        outer_hypothetical_main_sizes_sum: Au,
        container_main_size: Au,
    ) -> (Vec<Au>, Au) {
        struct FlexibleLengthResolutionItem<'items> {
            item: &'items FlexItem<'items>,
            frozen: Cell<bool>,
            target_main_size: Cell<Au>,
            flex_factor: f32,
        }

        // > 1. Determine the used flex factor. Sum the outer hypothetical main sizes of all
        // > items on the line. If the sum is less than the flex container’s inner main
        // > size, use the flex grow factor for the rest of this algorithm; otherwise, use
        // > the flex shrink factor.
        let grow = outer_hypothetical_main_sizes_sum < container_main_size;

        let mut frozen_count = 0;
        let items: Vec<_> = items
            .iter()
            .map(|item| {
                // > 2. Each item in the flex line has a target main size, initially set to its
                // > flex base size. Each item is initially unfrozen and may become frozen.
                let target_main_size = Cell::new(item.flex_base_size);

                // > 3. Size inflexible items. Freeze, setting its target main size to its hypothetical main size…
                // > - any item that has a flex factor of zero
                // > - if using the flex grow factor: any item that has a flex base size
                // >   greater than its hypothetical main size
                // > - if using the flex shrink factor: any item that has a flex base size
                // >   smaller than its hypothetical main size
                let flex_factor = if grow {
                    item.box_.style().get_position().flex_grow.0
                } else {
                    item.box_.style().get_position().flex_shrink.0
                };

                let is_inflexible = flex_factor == 0. ||
                    if grow {
                        item.flex_base_size > item.hypothetical_main_size
                    } else {
                        item.flex_base_size < item.hypothetical_main_size
                    };

                let frozen = Cell::new(false);
                if is_inflexible {
                    frozen_count += 1;
                    frozen.set(true);
                    target_main_size.set(item.hypothetical_main_size);
                }

                FlexibleLengthResolutionItem {
                    item,
                    frozen,
                    target_main_size,
                    flex_factor,
                }
            })
            .collect();

        let unfrozen_items = || items.iter().filter(|item| !item.frozen.get());
        let main_sizes = |items: Vec<FlexibleLengthResolutionItem>| {
            items
                .into_iter()
                .map(|item| item.target_main_size.get())
                .collect()
        };

        // https://drafts.csswg.org/css-flexbox/#initial-free-space
        // > 4. Calculate initial free space. Sum the outer sizes of all items on the line, and
        // > subtract this from the flex container’s inner main size. For frozen items, use
        // > their outer target main size; for other items, use their outer flex base size.
        let free_space = |all_items_frozen| {
            let items_size = items
                .iter()
                .map(|item| {
                    item.item.pbm_auto_is_zero.main +
                        if all_items_frozen || item.frozen.get() {
                            item.target_main_size.get()
                        } else {
                            item.item.flex_base_size
                        }
                })
                .sum();
            container_main_size - items_size
        };

        let initial_free_space = free_space(false);
        loop {
            // https://drafts.csswg.org/css-flexbox/#remaining-free-space
            let mut remaining_free_space = free_space(false);
            // > 5. a. Check for flexible items. If all the flex items on the line are
            // >       frozen, free space has been distributed; exit this loop.
            if frozen_count >= items.len() {
                return (main_sizes(items), remaining_free_space);
            }

            // > 5. b. Calculate the remaining free space as for initial free space, above. If the
            // > sum of the unfrozen flex items’ flex factors is less than one, multiply the
            // > initial free space by this sum. If the magnitude of this value is less than
            // > the magnitude of the remaining free space, use this as the remaining free
            // > space.
            let unfrozen_items_flex_factor_sum =
                unfrozen_items().map(|item| item.flex_factor).sum();
            if unfrozen_items_flex_factor_sum < 1. {
                let multiplied = initial_free_space.scale_by(unfrozen_items_flex_factor_sum);
                if multiplied.abs() < remaining_free_space.abs() {
                    remaining_free_space = multiplied
                }
            }

            // > 5. c. If the remaining free space is non-zero, distribute it proportional
            // to the flex factors:
            //
            // FIXME: is it a problem if floating point precision errors accumulate
            // and we get not-quite-zero remaining free space when we should get zero here?
            if remaining_free_space != Au::zero() {
                // > If using the flex grow factor:
                // > For every unfrozen item on the line, find the ratio of the item’s flex grow factor to
                // > the sum of the flex grow factors of all unfrozen items on the line. Set the item’s
                // > target main size to its flex base size plus a fraction of the remaining free space
                // > proportional to the ratio.
                if grow {
                    for item in unfrozen_items() {
                        let ratio = item.flex_factor / unfrozen_items_flex_factor_sum;
                        item.target_main_size
                            .set(item.item.flex_base_size + remaining_free_space.scale_by(ratio));
                    }
                // > If using the flex shrink factor
                // > For every unfrozen item on the line, multiply its flex shrink factor by its inner flex
                // > base size, and note this as its scaled flex shrink factor. Find the ratio of the
                // > item’s scaled flex shrink factor to the sum of the scaled flex shrink factors of all
                // > unfrozen items on the line. Set the item’s target main size to its flex base size
                // > minus a fraction of the absolute value of the remaining free space proportional to the
                // > ratio. Note this may result in a negative inner main size; it will be corrected in the
                // > next step.
                } else {
                    // https://drafts.csswg.org/css-flexbox/#scaled-flex-shrink-factor
                    let scaled_shrink_factor = |item: &FlexibleLengthResolutionItem| {
                        item.item.flex_base_size.scale_by(item.flex_factor)
                    };
                    let scaled_shrink_factors_sum: Au =
                        unfrozen_items().map(scaled_shrink_factor).sum();
                    if scaled_shrink_factors_sum > Au::zero() {
                        for item in unfrozen_items() {
                            let ratio = scaled_shrink_factor(item).0 as f32 /
                                scaled_shrink_factors_sum.0 as f32;
                            item.target_main_size.set(
                                item.item.flex_base_size -
                                    remaining_free_space.abs().scale_by(ratio),
                            );
                        }
                    }
                }
            }

            // > 5. d. Fix min/max violations. Clamp each non-frozen item’s target main size
            // > by its used min and max main sizes and floor its content-box size at zero.
            // > If the item’s target main size was made smaller by this, it’s a max
            // > violation. If the item’s target main size was made larger by this, it’s a
            // > min violation.
            let violation = |item: &FlexibleLengthResolutionItem| {
                let size = item.target_main_size.get();
                let clamped = size.clamp_between_extremums(
                    item.item.content_min_main_size,
                    item.item.content_max_main_size,
                );
                clamped - size
            };

            // > 5. e. Freeze over-flexed items. The total violation is the sum of the
            // > adjustments from the previous step ∑(clamped size - unclamped size). If the
            // > total violation is:
            // > - Zero:  Freeze all items.
            // > - Positive: Freeze all the items with min violations.
            // > - Negative:  Freeze all the items with max violations.
            let total_violation: Au = unfrozen_items().map(violation).sum();
            match total_violation.cmp(&Au::zero()) {
                Ordering::Equal => {
                    // “Freeze all items.”
                    // Return instead, as that’s what the next loop iteration would do.
                    let remaining_free_space = free_space(true);
                    return (main_sizes(items), remaining_free_space);
                },
                Ordering::Greater => {
                    // “Freeze all the items with min violations.”
                    // “If the item’s target main size was made larger by [clamping],
                    //  it’s a min violation.”
                    for item in items.iter() {
                        if violation(item) > Au::zero() {
                            item.target_main_size.set(item.item.content_min_main_size);
                            item.frozen.set(true);
                            frozen_count += 1;
                        }
                    }
                },
                Ordering::Less => {
                    // Negative total violation
                    // “Freeze all the items with max violations.”
                    // “If the item’s target main size was made smaller by [clamping],
                    //  it’s a max violation.”
                    for item in items.iter() {
                        if violation(item) < Au::zero() {
                            let Some(max_size) = item.item.content_max_main_size else {
                                unreachable!()
                            };
                            item.target_main_size.set(max_size);
                            item.frozen.set(true);
                            frozen_count += 1;
                        }
                    }
                },
            }
        }
    }

    /// <https://drafts.csswg.org/css-flexbox/#algo-cross-line>
    fn cross_size<'items>(items: &'items [FlexLineItem<'items>], flex_context: &FlexContext) -> Au {
        if flex_context.config.container_is_single_line {
            if let SizeConstraint::Definite(size) =
                flex_context.container_inner_size_constraint.cross
            {
                return size;
            }
        }

        let mut max_ascent = Au::zero();
        let mut max_descent = Au::zero();
        let mut max_outer_hypothetical_cross_size = Au::zero();
        for item in items.iter() {
            // TODO: check inline-axis is parallel to main axis, check no auto cross margins
            if matches!(
                item.item.align_self.0.value(),
                AlignFlags::BASELINE | AlignFlags::LAST_BASELINE
            ) {
                let baseline = item.get_or_synthesize_baseline_with_cross_size(
                    item.layout_result.hypothetical_cross_size,
                );
                let hypothetical_margin_box_cross_size =
                    item.layout_result.hypothetical_cross_size + item.item.pbm_auto_is_zero.cross;
                max_ascent = max_ascent.max(baseline);
                max_descent = max_descent.max(hypothetical_margin_box_cross_size - baseline);
            } else {
                max_outer_hypothetical_cross_size = max_outer_hypothetical_cross_size.max(
                    item.layout_result.hypothetical_cross_size + item.item.pbm_auto_is_zero.cross,
                );
            }
        }

        // https://drafts.csswg.org/css-flexbox/#baseline-participation
        let largest = max_outer_hypothetical_cross_size.max(max_ascent + max_descent);
        match flex_context.container_inner_size_constraint.cross {
            SizeConstraint::MinMax(min, max) if flex_context.config.container_is_single_line => {
                largest.clamp_between_extremums(min, max)
            },
            _ => largest,
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
                item.item.margin.main_start.is_auto() as u32 +
                    item.item.margin.main_end.is_auto() as u32
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
        for item in self.items.iter_mut() {
            let used_cross_size = if item.item.cross_size_stretches_to_line {
                let (axis, content_size) = match flex_context.config.flex_axis {
                    FlexAxis::Row => (Direction::Block, item.layout_result.content_size.block),
                    FlexAxis::Column => (Direction::Inline, item.layout_result.content_size.inline),
                };
                item.item.content_cross_sizes.resolve(
                    axis,
                    Size::Stretch,
                    Au::zero,
                    Some(final_line_cross_size - item.item.pbm_auto_is_zero.cross),
                    || content_size.into(),
                    // Tables have a special sizing in the block axis in that handles collapsed rows,
                    // but it would prevent stretching. So we only recognize tables in the inline axis.
                    // The interaction of collapsed table tracks and the flexbox algorithms is unclear,
                    // see https://github.com/w3c/csswg-drafts/issues/11408.
                    item.item.is_table() && axis == Direction::Inline,
                )
            } else {
                item.layout_result.hypothetical_cross_size
            };
            item_used_cross_sizes.push(used_cross_size);

            // “If the flex item has `align-self: stretch`, redo layout for its contents,
            // treating this used size as its definite cross size so that percentage-sized
            // children can be resolved.”
            if item.item.cross_size_stretches_to_line {
                let new_layout = item.item.layout(
                    item.used_main_size,
                    flex_context,
                    Some(used_cross_size),
                    Some(&mut item.layout_result),
                );
                if let Some(layout) = new_layout {
                    item.layout_result = layout;
                }
            }

            let baseline = item.get_or_synthesize_baseline_with_cross_size(used_cross_size);
            if matches!(
                item.item.align_self.0.value(),
                AlignFlags::BASELINE | AlignFlags::LAST_BASELINE
            ) {
                shared_alignment_baseline =
                    Some(shared_alignment_baseline.unwrap_or(baseline).max(baseline));
            }
            item.layout_result.baseline_relative_to_margin_box = Some(baseline);

            item_margins.push(item.item.resolve_auto_margins(
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
        let item_fragments = izip!(items, item_margins, item_used_cross_sizes.iter())
            .map(|(item, item_margin, item_used_cross_size)| {
                let item_used_size = FlexRelativeVec2 {
                    main: item.used_main_size,
                    cross: *item_used_cross_size,
                };
                item.collect_fragment(
                    &self,
                    item_used_size,
                    item_margin,
                    item_main_interval,
                    final_line_cross_size,
                    &shared_alignment_baseline,
                    flex_context,
                    &mut all_baselines,
                    &mut main_position_cursor,
                )
            })
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
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            name = "FlexItem::layout",
            skip_all,
            fields(
                servo_profiling = true,
                self_address = self as *const _ as usize,
                box_address = self.box_ as *const _ as usize,
                for_stretch = non_stretch_layout_result.is_some(),
            ),
            level = "trace",
        )
    )]
    #[allow(clippy::too_many_arguments)]
    fn layout(
        &self,
        used_main_size: Au,
        flex_context: &FlexContext,
        used_cross_size_override: Option<Au>,
        non_stretch_layout_result: Option<&mut FlexItemLayoutResult>,
    ) -> Option<FlexItemLayoutResult> {
        let containing_block = flex_context.containing_block;
        let mut positioning_context = PositioningContext::new_for_style(self.box_.style())
            .unwrap_or_else(|| {
                PositioningContext::new_for_subtree(
                    flex_context
                        .positioning_context
                        .collects_for_nearest_positioned_ancestor(),
                )
            });

        let independent_formatting_context = &self.box_.independent_formatting_context;
        let item_writing_mode = independent_formatting_context.style().writing_mode;
        let item_is_horizontal = item_writing_mode.is_horizontal();
        let flex_axis = flex_context.config.flex_axis;
        let cross_axis_is_item_block_axis = cross_axis_is_item_block_axis(
            containing_block.style.writing_mode.is_horizontal(),
            item_is_horizontal,
            flex_axis,
        );

        let (inline_size, block_size) = if cross_axis_is_item_block_axis {
            let cross_size = match used_cross_size_override {
                Some(s) => SizeConstraint::Definite(s),
                None => {
                    // This means that an auto size with stretch alignment will behave different than
                    // a stretch size. That's not what the spec says, but matches other browsers.
                    // To be discussed in https://github.com/w3c/csswg-drafts/issues/11784.
                    let stretch_size = containing_block
                        .size
                        .block
                        .to_definite()
                        .map(|size| Au::zero().max(size - self.pbm_auto_is_zero.cross));
                    self.content_cross_sizes.resolve_extrinsic(
                        Size::FitContent,
                        Au::zero(),
                        stretch_size,
                    )
                },
            };
            (used_main_size, cross_size)
        } else {
            let cross_size = used_cross_size_override.unwrap_or_else(|| {
                let stretch_size =
                    Au::zero().max(containing_block.size.inline - self.pbm_auto_is_zero.cross);
                let get_content_size = || {
                    let constraint_space = ConstraintSpace::new(
                        SizeConstraint::Definite(used_main_size),
                        item_writing_mode,
                        self.preferred_aspect_ratio,
                    );
                    independent_formatting_context
                        .inline_content_sizes(flex_context.layout_context, &constraint_space)
                        .sizes
                };
                self.content_cross_sizes.resolve(
                    Direction::Inline,
                    Size::FitContent,
                    Au::zero,
                    Some(stretch_size),
                    get_content_size,
                    self.is_table(),
                )
            });
            // The main size of a flex item is considered to be definite if its flex basis is definite
            // or the flex container has a definite main size.
            // <https://drafts.csswg.org/css-flexbox-1/#definite-sizes>
            let main_size = if self.flex_base_size_is_definite ||
                flex_context
                    .container_inner_size_constraint
                    .main
                    .is_definite()
            {
                SizeConstraint::Definite(used_main_size)
            } else {
                SizeConstraint::default()
            };
            (cross_size, main_size)
        };

        let container_writing_mode = containing_block.style.writing_mode;
        let item_style = independent_formatting_context.style();
        match &independent_formatting_context.contents {
            IndependentFormattingContextContents::Replaced(replaced) => {
                let min_size = flex_axis.vec2_to_flow_relative(FlexRelativeVec2 {
                    main: Size::Numeric(self.content_min_main_size),
                    cross: self.content_cross_sizes.min,
                });
                let max_size = flex_axis.vec2_to_flow_relative(FlexRelativeVec2 {
                    main: self
                        .content_max_main_size
                        .map_or(Size::Initial, Size::Numeric),
                    cross: self.content_cross_sizes.max,
                });
                let size = replaced.used_size_as_if_inline_element_from_content_box_sizes(
                    containing_block,
                    item_style,
                    self.preferred_aspect_ratio,
                    LogicalVec2 {
                        block: &Sizes::new(
                            block_size
                                .to_definite()
                                .map_or(Size::Initial, Size::Numeric),
                            min_size.block,
                            max_size.block,
                        ),
                        inline: &Sizes::new(
                            Size::Numeric(inline_size),
                            min_size.inline,
                            max_size.inline,
                        ),
                    },
                    Size::FitContent.into(),
                    flex_axis.vec2_to_flow_relative(self.pbm_auto_is_zero),
                );

                if let Some(non_stretch_layout_result) = non_stretch_layout_result {
                    if non_stretch_layout_result
                        .compatible_with_containing_block_size_and_content_size(
                            containing_block,
                            size,
                        )
                    {
                        return None;
                    }
                }

                let hypothetical_cross_size = flex_axis.vec2_to_flex_relative(size).cross;
                let fragments = replaced.make_fragments(
                    flex_context.layout_context,
                    item_style,
                    size.to_physical_size(container_writing_mode),
                );

                Some(FlexItemLayoutResult {
                    hypothetical_cross_size,
                    fragments,
                    positioning_context,
                    content_size: size,
                    containing_block_inline_size: containing_block.size.inline,
                    containing_block_block_size: containing_block.size.block,
                    depends_on_block_constraints: false,
                    has_child_which_depends_on_block_constraints: false,

                    // We will need to synthesize the baseline, but since the used cross
                    // size can differ from the hypothetical cross size, we should defer
                    // synthesizing until needed.
                    baseline_relative_to_margin_box: None,
                })
            },
            IndependentFormattingContextContents::NonReplaced(non_replaced) => {
                let item_as_containing_block = ContainingBlock {
                    size: ContainingBlockSize {
                        inline: inline_size,
                        block: block_size,
                    },
                    style: item_style,
                };

                if let Some(non_stretch_layout_result) = non_stretch_layout_result {
                    if non_stretch_layout_result
                        .compatible_with_containing_block_size(&item_as_containing_block)
                    {
                        return None;
                    }
                }

                let layout = non_replaced.layout_with_caching(
                    flex_context.layout_context,
                    &mut positioning_context,
                    &item_as_containing_block,
                    containing_block,
                    &independent_formatting_context.base,
                    flex_axis == FlexAxis::Column ||
                        self.cross_size_stretches_to_line ||
                        self.depends_on_block_constraints,
                );
                let CacheableLayoutResult {
                    fragments,
                    content_block_size,
                    baselines: content_box_baselines,
                    depends_on_block_constraints,
                    ..
                } = layout;

                let has_child_which_depends_on_block_constraints = fragments.iter().any(|fragment| {
                        fragment.base().is_some_and(|base|
                                base.flags.contains(
                                    FragmentFlags::SIZE_DEPENDS_ON_BLOCK_CONSTRAINTS_AND_CAN_BE_CHILD_OF_FLEX_ITEM))
                });

                let hypothetical_cross_size = if cross_axis_is_item_block_axis {
                    // This means that an auto size with stretch alignment will behave different than
                    // a stretch size. That's not what the spec says, but matches other browsers.
                    // To be discussed in https://github.com/w3c/csswg-drafts/issues/11784.
                    let stretch_size = containing_block
                        .size
                        .block
                        .to_definite()
                        .map(|size| Au::zero().max(size - self.pbm_auto_is_zero.cross));
                    self.content_cross_sizes.resolve(
                        Direction::Block,
                        Size::FitContent,
                        Au::zero,
                        stretch_size,
                        || content_block_size.into(),
                        self.is_table(),
                    )
                } else {
                    inline_size
                };

                let item_writing_mode_is_orthogonal_to_container_writing_mode =
                    flex_context.config.writing_mode.is_horizontal() !=
                        item_style.writing_mode.is_horizontal();
                let has_compatible_baseline = match flex_axis {
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

                Some(FlexItemLayoutResult {
                    hypothetical_cross_size,
                    fragments,
                    positioning_context,
                    baseline_relative_to_margin_box,
                    content_size: LogicalVec2 {
                        inline: item_as_containing_block.size.inline,
                        block: content_block_size,
                    },
                    containing_block_inline_size: item_as_containing_block.size.inline,
                    containing_block_block_size: item_as_containing_block.size.block,
                    depends_on_block_constraints,
                    has_child_which_depends_on_block_constraints,
                })
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
                };
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

    #[inline]
    fn is_table(&self) -> bool {
        match &self.box_.independent_formatting_context.contents {
            IndependentFormattingContextContents::NonReplaced(content) => content.is_table(),
            IndependentFormattingContextContents::Replaced(_) => false,
        }
    }
}

impl FlexItemBox {
    fn to_flex_item<'a>(
        &self,
        layout_context: &LayoutContext,
        containing_block: &IndefiniteContainingBlock,
        content_box_sizes_and_pbm: &ContentBoxSizesAndPBM,
        config: &FlexContainerConfig,
        flex_context_getter: &impl Fn() -> &'a FlexContext<'a>,
    ) -> FlexItem {
        let flex_axis = config.flex_axis;
        let style = self.style();
        let cross_axis_is_item_block_axis = cross_axis_is_item_block_axis(
            containing_block.writing_mode.is_horizontal(),
            style.writing_mode.is_horizontal(),
            flex_axis,
        );
        let main_axis = if cross_axis_is_item_block_axis {
            Direction::Inline
        } else {
            Direction::Block
        };
        let align_self = AlignItems(config.resolve_align_self_for_child(style));

        let ContentBoxSizesAndPBM {
            content_box_sizes,
            pbm,
            depends_on_block_constraints,
            preferred_size_computes_to_auto,
        } = content_box_sizes_and_pbm;

        let preferred_aspect_ratio = self
            .independent_formatting_context
            .preferred_aspect_ratio(&pbm.padding_border_sums);
        let padding = config.sides_to_flex_relative(pbm.padding);
        let border = config.sides_to_flex_relative(pbm.border);
        let margin = config.sides_to_flex_relative(pbm.margin);
        let padding_border = padding.sum_by_axis() + border.sum_by_axis();
        let margin_auto_is_zero = config.sides_to_flex_relative(pbm.margin.auto_is(Au::zero));
        let pbm_auto_is_zero = FlexRelativeVec2 {
            main: padding_border.main,
            cross: padding_border.cross,
        } + margin_auto_is_zero.sum_by_axis();
        let (content_main_sizes, mut content_cross_sizes, cross_size_computes_to_auto) =
            match flex_axis {
                FlexAxis::Row => (
                    &content_box_sizes.inline,
                    content_box_sizes.block.clone(),
                    preferred_size_computes_to_auto.block,
                ),
                FlexAxis::Column => (
                    &content_box_sizes.block,
                    content_box_sizes.inline.clone(),
                    preferred_size_computes_to_auto.inline,
                ),
            };
        let cross_size_stretches_to_line = cross_size_computes_to_auto &&
            item_with_auto_cross_size_stretches_to_line_size(align_self, &margin);
        if cross_size_stretches_to_line && config.container_is_single_line {
            // <https://drafts.csswg.org/css-flexbox-1/#definite-sizes>
            // > If a single-line flex container has a definite cross size, the automatic preferred
            // > outer cross size of any stretched flex items is the flex container’s inner cross size.
            // Therefore, set it to `stretch`, which has the desired behavior.
            content_cross_sizes.preferred = Size::Stretch;
        }
        let containing_block_size = flex_axis.vec2_to_flex_relative(containing_block.size);
        let stretch_size = FlexRelativeVec2 {
            main: containing_block_size
                .main
                .map(|v| Au::zero().max(v - pbm_auto_is_zero.main)),
            cross: containing_block_size
                .cross
                .map(|v| Au::zero().max(v - pbm_auto_is_zero.cross)),
        };

        // <https://drafts.csswg.org/css-flexbox/#definite-sizes>
        // > If a single-line flex container has a definite cross size, the automatic preferred
        // > outer cross size of any stretched flex items is the flex container’s inner cross size
        // > (clamped to the flex item’s min and max cross size) and is considered definite.
        let (preferred_cross_size, min_cross_size, max_cross_size) = content_cross_sizes
            .resolve_each_extrinsic(Size::FitContent, Au::zero(), stretch_size.cross);
        let cross_size = SizeConstraint::new(preferred_cross_size, min_cross_size, max_cross_size);

        // <https://drafts.csswg.org/css-flexbox/#transferred-size-suggestion>
        // > If the item has a preferred aspect ratio and its preferred cross size is definite, then the
        // > transferred size suggestion is that size (clamped by its minimum and maximum cross sizes if they
        // > are definite), converted through the aspect ratio. It is otherwise undefined.
        let transferred_size_suggestion =
            LazyCell::new(|| match (preferred_aspect_ratio, cross_size) {
                (Some(ratio), SizeConstraint::Definite(cross_size)) => {
                    Some(ratio.compute_dependent_size(main_axis, cross_size))
                },
                _ => None,
            });

        // <https://drafts.csswg.org/css-flexbox/#algo-main-item>
        let flex_base_size_is_definite = Cell::new(true);
        let main_content_sizes = LazyCell::new(|| {
            let flex_item = &self.independent_formatting_context;
            // > B: If the flex item has ...
            // >   - a preferred aspect ratio,
            // >   - a used flex basis of content, and
            // >   - a definite cross size,
            // > then the flex base size is calculated from its used cross size and the flex item’s aspect ratio.
            if let Some(transferred_size_suggestion) = *transferred_size_suggestion {
                return transferred_size_suggestion.into();
            }

            flex_base_size_is_definite.set(false);

            // FIXME: implement cases C, D.

            // > E. Otherwise, size the item into the available space using its used flex basis in place of
            // > its main size, treating a value of content as max-content. If a cross size is needed to
            // > determine the main size (e.g. when the flex item’s main size is in its block axis, or when
            // > it has a preferred aspect ratio) and the flex item’s cross size is auto and not definite,
            // > in this calculation use fit-content as the flex item’s cross size. The flex base size is
            // > the item’s resulting main size.
            if cross_axis_is_item_block_axis {
                // The main axis is the inline axis, so we can get the content size from the normal
                // preferred widths calculation.
                let constraint_space =
                    ConstraintSpace::new(cross_size, style.writing_mode, preferred_aspect_ratio);
                let content_sizes = flex_item
                    .inline_content_sizes(layout_context, &constraint_space)
                    .sizes;
                if let Some(ratio) = preferred_aspect_ratio {
                    let transferred_min = ratio.compute_dependent_size(main_axis, min_cross_size);
                    let transferred_max =
                        max_cross_size.map(|v| ratio.compute_dependent_size(main_axis, v));
                    content_sizes
                        .map(|size| size.clamp_between_extremums(transferred_min, transferred_max))
                } else {
                    content_sizes
                }
            } else {
                self.layout_for_block_content_size(
                    flex_context_getter(),
                    &pbm_auto_is_zero,
                    content_box_sizes,
                    preferred_aspect_ratio,
                    content_cross_sizes.preferred == Size::Stretch,
                    IntrinsicSizingMode::Size,
                )
                .into()
            }
        });

        let flex_base_size = self
            .flex_basis(
                containing_block_size.main,
                content_main_sizes.preferred,
                padding_border.main,
            )
            .resolve_for_preferred(Size::MaxContent, stretch_size.main, &main_content_sizes);
        let flex_base_size_is_definite = flex_base_size_is_definite.take();

        let content_max_main_size = content_main_sizes
            .max
            .resolve_for_max(stretch_size.main, &main_content_sizes);

        let get_automatic_minimum_size = || {
            // This is an implementation of <https://drafts.csswg.org/css-flexbox/#min-size-auto>.
            if style.establishes_scroll_container(self.base_fragment_info().flags) {
                return Au::zero();
            }

            // > **specified size suggestion**
            // > If the item’s preferred main size is definite and not automatic, then the specified
            // > size suggestion is that size. It is otherwise undefined.
            let specified_size_suggestion = content_main_sizes
                .preferred
                .maybe_resolve_extrinsic(stretch_size.main);

            let is_replaced = self.independent_formatting_context.is_replaced();

            // > **content size suggestion**
            // > The content size suggestion is the min-content size in the main axis, clamped, if it has a
            // > preferred aspect ratio, by any definite minimum and maximum cross sizes converted through the
            // > aspect ratio.
            let content_size_suggestion = match preferred_aspect_ratio {
                Some(ratio) => main_content_sizes.min_content.clamp_between_extremums(
                    ratio.compute_dependent_size(main_axis, min_cross_size),
                    max_cross_size.map(|l| ratio.compute_dependent_size(main_axis, l)),
                ),
                None => main_content_sizes.min_content,
            };

            // > The content-based minimum size of a flex item is the smaller of its specified size
            // > suggestion and its content size suggestion if its specified size suggestion exists;
            // > otherwise, the smaller of its transferred size suggestion and its content size
            // > suggestion if the element is replaced and its transferred size suggestion exists;
            // > otherwise its content size suggestion. In all cases, the size is clamped by the maximum
            // > main size if it’s definite.
            match (specified_size_suggestion, *transferred_size_suggestion) {
                (Some(specified), _) => specified.min(content_size_suggestion),
                (_, Some(transferred)) if is_replaced => transferred.min(content_size_suggestion),
                _ => content_size_suggestion,
            }
            .clamp_below_max(content_max_main_size)
        };
        let content_min_main_size = content_main_sizes.min.resolve_for_min(
            get_automatic_minimum_size,
            stretch_size.main,
            &main_content_sizes,
        );

        FlexItem {
            box_: self,
            content_cross_sizes,
            padding,
            border,
            margin: config.sides_to_flex_relative(pbm.margin),
            pbm_auto_is_zero,
            flex_base_size,
            flex_base_size_is_definite,
            hypothetical_main_size: flex_base_size
                .clamp_between_extremums(content_min_main_size, content_max_main_size),
            content_min_main_size,
            content_max_main_size,
            align_self,
            depends_on_block_constraints: *depends_on_block_constraints,
            preferred_aspect_ratio,
            cross_size_stretches_to_line,
        }
    }

    fn main_content_size_info<'a>(
        &self,
        layout_context: &LayoutContext,
        containing_block: &IndefiniteContainingBlock,
        config: &FlexContainerConfig,
        flex_context_getter: &impl Fn() -> &'a FlexContext<'a>,
    ) -> FlexItemBoxInlineContentSizesInfo {
        let content_box_sizes_and_pbm = self
            .independent_formatting_context
            .layout_style()
            .content_box_sizes_and_padding_border_margin(containing_block);

        // TODO: when laying out a column container with an indefinite main size,
        // we compute the base sizes of the items twice. We should consider caching.
        let FlexItem {
            content_cross_sizes,
            flex_base_size,
            content_min_main_size,
            content_max_main_size,
            pbm_auto_is_zero,
            preferred_aspect_ratio,
            ..
        } = self.to_flex_item(
            layout_context,
            containing_block,
            &content_box_sizes_and_pbm,
            config,
            flex_context_getter,
        );

        // Compute the min-content and max-content contributions of the item.
        // <https://drafts.csswg.org/css-flexbox/#intrinsic-item-contributions>
        let (content_contribution_sizes, depends_on_block_constraints) = match config.flex_axis {
            FlexAxis::Row => {
                let auto_minimum = LogicalVec2 {
                    inline: content_min_main_size,
                    block: Au::zero(),
                };
                let InlineContentSizesResult {
                    sizes,
                    depends_on_block_constraints,
                } = self
                    .independent_formatting_context
                    .outer_inline_content_sizes(
                        layout_context,
                        containing_block,
                        &auto_minimum,
                        content_cross_sizes.preferred == Size::Stretch,
                    );
                (sizes, depends_on_block_constraints)
            },
            FlexAxis::Column => {
                let size = self.layout_for_block_content_size(
                    flex_context_getter(),
                    &pbm_auto_is_zero,
                    &content_box_sizes_and_pbm.content_box_sizes,
                    preferred_aspect_ratio,
                    content_cross_sizes.preferred == Size::Stretch,
                    IntrinsicSizingMode::Contribution,
                );
                (size.into(), true)
            },
        };

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
        let style_position = &self.style().get_position();
        if style_position.flex_grow.is_zero() {
            min_content_main_size_for_multiline_container.min_assign(flex_base_size);
        }
        if style_position.flex_shrink.is_zero() {
            min_content_main_size_for_multiline_container.max_assign(flex_base_size);
        }
        min_content_main_size_for_multiline_container =
            min_content_main_size_for_multiline_container
                .clamp_between_extremums(content_min_main_size, content_max_main_size);

        FlexItemBoxInlineContentSizesInfo {
            outer_flex_base_size,
            content_min_main_size,
            content_max_main_size,
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

    /// <https://drafts.csswg.org/css-flexbox-1/#flex-basis-property>
    /// Returns the used value of the `flex-basis` property, after resolving percentages,
    /// resolving `auto`, and taking `box-sizing` into account.
    /// Note that a return value of `Size::Initial` represents `flex-basis: content`,
    /// not `flex-basis: auto`, since the latter always resolves to something else.
    fn flex_basis(
        &self,
        container_definite_main_size: Option<Au>,
        main_preferred_size: Size<Au>,
        main_padding_border_sum: Au,
    ) -> Size<Au> {
        let style_position = &self.independent_formatting_context.style().get_position();
        match &style_position.flex_basis {
            // https://drafts.csswg.org/css-flexbox-1/#valdef-flex-basis-content
            // > Indicates an automatic size based on the flex item’s content.
            FlexBasis::Content => Size::Initial,

            FlexBasis::Size(size) => match Size::<LengthPercentage>::from(size.clone()) {
                // https://drafts.csswg.org/css-flexbox-1/#valdef-flex-basis-auto
                // > When specified on a flex item, the `auto` keyword retrieves
                // > the value of the main size property as the used `flex-basis`.
                Size::Initial => main_preferred_size,

                // https://drafts.csswg.org/css-flexbox-1/#flex-basis-property
                // > For all values other than `auto` and `content` (defined above),
                // > `flex-basis` is resolved the same way as `width` in horizontal
                // > writing modes, except that if a value would resolve to `auto`
                // > for `width`, it instead resolves to `content` for `flex-basis`.
                // > For example, percentage values of `flex-basis` are resolved
                // > against the flex item’s containing block (i.e. its flex container);
                // > and if that containing block’s size is indefinite,
                // > the used value for `flex-basis` is `content`.
                size => {
                    let apply_box_sizing = |length: Au| {
                        match style_position.box_sizing {
                            BoxSizing::ContentBox => length,
                            // This may make `length` negative,
                            // but it will be clamped in the hypothetical main size
                            BoxSizing::BorderBox => length - main_padding_border_sum,
                        }
                    };
                    size.resolve_percentages_for_preferred(container_definite_main_size)
                        .map(apply_box_sizing)
                },
            },
        }
    }

    #[allow(clippy::too_many_arguments)]
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            name = "FlexContainer::layout_for_block_content_size",
            skip_all,
            fields(servo_profiling = true),
            level = "trace",
        )
    )]
    fn layout_for_block_content_size(
        &self,
        flex_context: &FlexContext,
        pbm_auto_is_zero: &FlexRelativeVec2<Au>,
        content_box_sizes: &LogicalVec2<Sizes>,
        preferred_aspect_ratio: Option<AspectRatio>,
        cross_size_stretches_to_container_size: bool,
        intrinsic_sizing_mode: IntrinsicSizingMode,
    ) -> Au {
        let mut positioning_context = PositioningContext::new_for_style(self.style())
            .unwrap_or_else(|| {
                PositioningContext::new_for_subtree(
                    flex_context
                        .positioning_context
                        .collects_for_nearest_positioned_ancestor(),
                )
            });

        let style = self.independent_formatting_context.style();
        match &self.independent_formatting_context.contents {
            IndependentFormattingContextContents::Replaced(replaced) => {
                let get_used_size = |block_sizes| {
                    replaced.used_size_as_if_inline_element_from_content_box_sizes(
                        flex_context.containing_block,
                        style,
                        preferred_aspect_ratio,
                        LogicalVec2 {
                            block: block_sizes,
                            inline: &content_box_sizes.inline,
                        },
                        Size::FitContent.into(),
                        LogicalVec2 {
                            inline: pbm_auto_is_zero.cross,
                            block: pbm_auto_is_zero.main,
                        },
                    )
                };
                if intrinsic_sizing_mode == IntrinsicSizingMode::Size {
                    get_used_size(&Sizes::default()).block
                } else {
                    get_used_size(&content_box_sizes.block).block
                }
            },
            IndependentFormattingContextContents::NonReplaced(non_replaced) => {
                // TODO: This is wrong if the item writing mode is different from the flex
                // container's writing mode.
                let inline_size = {
                    let initial_behavior = if cross_size_stretches_to_container_size {
                        Size::Stretch
                    } else {
                        Size::FitContent
                    };
                    let stretch_size =
                        flex_context.containing_block.size.inline - pbm_auto_is_zero.cross;
                    let get_content_size = || {
                        let constraint_space = ConstraintSpace::new(
                            SizeConstraint::default(),
                            style.writing_mode,
                            non_replaced.preferred_aspect_ratio(),
                        );
                        self.independent_formatting_context
                            .inline_content_sizes(flex_context.layout_context, &constraint_space)
                            .sizes
                    };
                    content_box_sizes.inline.resolve(
                        Direction::Inline,
                        initial_behavior,
                        Au::zero,
                        Some(stretch_size),
                        get_content_size,
                        false,
                    )
                };
                let item_as_containing_block = ContainingBlock {
                    size: ContainingBlockSize {
                        inline: inline_size,
                        block: SizeConstraint::default(),
                    },
                    style,
                };
                let mut content_block_size = || {
                    non_replaced
                        .layout_with_caching(
                            flex_context.layout_context,
                            &mut positioning_context,
                            &item_as_containing_block,
                            flex_context.containing_block,
                            &self.independent_formatting_context.base,
                            false, /* depends_on_block_constraints */
                        )
                        .content_block_size
                };
                match intrinsic_sizing_mode {
                    IntrinsicSizingMode::Contribution => {
                        let stretch_size = flex_context
                            .containing_block
                            .size
                            .block
                            .to_definite()
                            .map(|block_size| block_size - pbm_auto_is_zero.main);
                        let inner_block_size = content_box_sizes.block.resolve(
                            Direction::Block,
                            Size::FitContent,
                            Au::zero,
                            stretch_size,
                            || ContentSizes::from(content_block_size()),
                            // Tables have a special sizing in the block axis that handles collapsed rows
                            // by ignoring the sizing properties and instead relying on the content block size,
                            // which should indirectly take sizing properties into account.
                            // However, above we laid out the table with a SizeConstraint::default() block size,
                            // so the content block size doesn't take sizing properties into account.
                            // Therefore, pretending that it's never a table tends to provide a better result.
                            false, /* is_table */
                        );
                        inner_block_size + pbm_auto_is_zero.main
                    },
                    IntrinsicSizingMode::Size => content_block_size(),
                }
            },
        }
    }
}
