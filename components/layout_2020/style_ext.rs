/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use app_units::Au;
use style::computed_values::direction::T as Direction;
use style::computed_values::mix_blend_mode::T as ComputedMixBlendMode;
use style::computed_values::position::T as ComputedPosition;
use style::computed_values::transform_style::T as ComputedTransformStyle;
use style::computed_values::unicode_bidi::T as UnicodeBidi;
use style::logical_geometry::{Direction as AxisDirection, WritingMode};
use style::properties::longhands::backface_visibility::computed_value::T as BackfaceVisiblity;
use style::properties::longhands::box_sizing::computed_value::T as BoxSizing;
use style::properties::longhands::column_span::computed_value::T as ColumnSpan;
use style::properties::ComputedValues;
use style::servo::selector_parser::PseudoElement;
use style::values::computed::basic_shape::ClipPath;
use style::values::computed::image::Image as ComputedImageLayer;
use style::values::computed::{AlignItems, BorderStyle, Inset, LengthPercentage, Margin};
use style::values::generics::box_::Perspective;
use style::values::generics::position::{GenericAspectRatio, PreferredRatio};
use style::values::specified::align::AlignFlags;
use style::values::specified::{box_ as stylo, Overflow};
use style::values::CSSFloat;
use style::Zero;
use webrender_api as wr;

use crate::dom_traversal::Contents;
use crate::fragment_tree::FragmentFlags;
use crate::geom::{
    AuOrAuto, LengthPercentageOrAuto, LogicalSides, LogicalVec2, PhysicalSides, PhysicalSize,
    PhysicalVec, Size,
};
use crate::{ContainingBlock, IndefiniteContainingBlock};

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum Display {
    None,
    Contents,
    GeneratingBox(DisplayGeneratingBox),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DisplayGeneratingBox {
    OutsideInside {
        outside: DisplayOutside,
        inside: DisplayInside,
    },
    /// <https://drafts.csswg.org/css-display-3/#layout-specific-display>
    LayoutInternal(DisplayLayoutInternal),
}

impl DisplayGeneratingBox {
    pub(crate) fn display_inside(&self) -> DisplayInside {
        match *self {
            DisplayGeneratingBox::OutsideInside { inside, .. } => inside,
            DisplayGeneratingBox::LayoutInternal(layout_internal) => {
                layout_internal.display_inside()
            },
        }
    }

    pub(crate) fn used_value_for_contents(&self, contents: &Contents) -> Self {
        // From <https://www.w3.org/TR/css-display-3/#layout-specific-display>:
        // > When the display property of a replaced element computes to one of
        // > the layout-internal values, it is handled as having a used value of
        // > inline.
        if matches!(self, Self::LayoutInternal(_)) && contents.is_replaced() {
            Self::OutsideInside {
                outside: DisplayOutside::Inline,
                inside: DisplayInside::Flow {
                    is_list_item: false,
                },
            }
        } else {
            *self
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DisplayOutside {
    Block,
    Inline,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DisplayInside {
    // “list-items are limited to the Flow Layout display types”
    // <https://drafts.csswg.org/css-display/#list-items>
    Flow { is_list_item: bool },
    FlowRoot { is_list_item: bool },
    Flex,
    Table,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(clippy::enum_variant_names)]
/// <https://drafts.csswg.org/css-display-3/#layout-specific-display>
pub(crate) enum DisplayLayoutInternal {
    TableCaption,
    TableCell,
    TableColumn,
    TableColumnGroup,
    TableFooterGroup,
    TableHeaderGroup,
    TableRow,
    TableRowGroup,
}

impl DisplayLayoutInternal {
    /// <https://drafts.csswg.org/css-display-3/#layout-specific-displa>
    pub(crate) fn display_inside(&self) -> DisplayInside {
        // When we add ruby, the display_inside of ruby must be Flow.
        // TODO: this should be unreachable for everything but
        // table cell and caption, once we have box tree fixups.
        DisplayInside::FlowRoot {
            is_list_item: false,
        }
    }
}

/// Percentages resolved but not `auto` margins
#[derive(Clone, Debug)]
pub(crate) struct PaddingBorderMargin {
    pub padding: LogicalSides<Au>,
    pub border: LogicalSides<Au>,
    pub margin: LogicalSides<AuOrAuto>,

    /// Pre-computed sums in each axis
    pub padding_border_sums: LogicalVec2<Au>,
}

impl PaddingBorderMargin {
    pub(crate) fn zero() -> Self {
        Self {
            padding: LogicalSides::zero(),
            border: LogicalSides::zero(),
            margin: LogicalSides::zero(),
            padding_border_sums: LogicalVec2::zero(),
        }
    }
}

/// Resolved `aspect-ratio` property with respect to a specific element. Depends
/// on that element's `box-sizing` (and padding and border, if that `box-sizing`
/// is `border-box`).
#[derive(Clone, Copy, Debug)]
pub(crate) struct AspectRatio {
    /// If the element that this aspect ratio belongs to uses box-sizing:
    /// border-box, and the aspect-ratio property does not contain "auto", then
    /// the aspect ratio is in respect to the border box. This will then contain
    /// the summed sizes of the padding and border. Otherwise, it's 0.
    box_sizing_adjustment: LogicalVec2<Au>,
    /// The ratio itself (inline over block).
    i_over_b: CSSFloat,
}

impl AspectRatio {
    /// Given one side length, compute the other one.
    pub(crate) fn compute_dependent_size(
        &self,
        ratio_dependent_axis: AxisDirection,
        ratio_determining_size: Au,
    ) -> Au {
        match ratio_dependent_axis {
            // Calculate the inline size from the block size
            AxisDirection::Inline => {
                (ratio_determining_size + self.box_sizing_adjustment.block).scale_by(self.i_over_b) -
                    self.box_sizing_adjustment.inline
            },
            // Calculate the block size from the inline size
            AxisDirection::Block => {
                (ratio_determining_size + self.box_sizing_adjustment.inline)
                    .scale_by(1.0 / self.i_over_b) -
                    self.box_sizing_adjustment.block
            },
        }
    }
}

pub(crate) trait ComputedValuesExt {
    fn physical_box_offsets(&self) -> PhysicalSides<LengthPercentageOrAuto<'_>>;
    fn box_offsets(
        &self,
        containing_block: &ContainingBlock,
    ) -> LogicalSides<LengthPercentageOrAuto<'_>>;
    fn box_size(
        &self,
        containing_block_writing_mode: WritingMode,
    ) -> LogicalVec2<Size<LengthPercentage>>;
    fn min_box_size(
        &self,
        containing_block_writing_mode: WritingMode,
    ) -> LogicalVec2<Size<LengthPercentage>>;
    fn max_box_size(
        &self,
        containing_block_writing_mode: WritingMode,
    ) -> LogicalVec2<Size<LengthPercentage>>;
    fn content_box_size(
        &self,
        containing_block: &ContainingBlock,
        pbm: &PaddingBorderMargin,
    ) -> LogicalVec2<Size<Au>>;
    fn content_box_size_deprecated(
        &self,
        containing_block: &ContainingBlock,
        pbm: &PaddingBorderMargin,
    ) -> LogicalVec2<AuOrAuto>;
    fn content_box_size_for_box_size(
        &self,
        box_size: LogicalVec2<Size<Au>>,
        pbm: &PaddingBorderMargin,
    ) -> LogicalVec2<Size<Au>>;
    fn content_min_box_size(
        &self,
        containing_block: &ContainingBlock,
        pbm: &PaddingBorderMargin,
    ) -> LogicalVec2<Size<Au>>;
    fn content_min_box_size_deprecated(
        &self,
        containing_block: &ContainingBlock,
        pbm: &PaddingBorderMargin,
    ) -> LogicalVec2<AuOrAuto>;
    fn content_min_box_size_for_min_size(
        &self,
        box_size: LogicalVec2<Size<Au>>,
        pbm: &PaddingBorderMargin,
    ) -> LogicalVec2<Size<Au>>;
    fn content_max_box_size(
        &self,
        containing_block: &ContainingBlock,
        pbm: &PaddingBorderMargin,
    ) -> LogicalVec2<Size<Au>>;
    fn content_max_box_size_deprecated(
        &self,
        containing_block: &ContainingBlock,
        pbm: &PaddingBorderMargin,
    ) -> LogicalVec2<Option<Au>>;
    fn content_max_box_size_for_max_size(
        &self,
        box_size: LogicalVec2<Size<Au>>,
        pbm: &PaddingBorderMargin,
    ) -> LogicalVec2<Size<Au>>;
    #[allow(clippy::type_complexity)]
    fn content_box_sizes_and_padding_border_margin(
        &self,
        containing_block: &IndefiniteContainingBlock,
    ) -> (
        LogicalVec2<Size<Au>>,
        LogicalVec2<Size<Au>>,
        LogicalVec2<Size<Au>>,
        PaddingBorderMargin,
        bool, /* depends_on_block_constraints */
    );
    fn content_box_sizes_and_padding_border_margin_deprecated(
        &self,
        containing_block: &IndefiniteContainingBlock,
    ) -> (
        LogicalVec2<AuOrAuto>,
        LogicalVec2<AuOrAuto>,
        LogicalVec2<Option<Au>>,
        PaddingBorderMargin,
        bool, /* depends_on_block_constraints */
    );
    fn padding_border_margin(&self, containing_block: &ContainingBlock) -> PaddingBorderMargin;
    fn padding_border_margin_for_intrinsic_size(
        &self,
        writing_mode: WritingMode,
    ) -> PaddingBorderMargin;
    fn padding_border_margin_with_writing_mode_and_containing_block_inline_size(
        &self,
        writing_mode: WritingMode,
        containing_block_inline_size: Au,
    ) -> PaddingBorderMargin;
    fn padding(
        &self,
        containing_block_writing_mode: WritingMode,
    ) -> LogicalSides<&LengthPercentage>;
    fn border_style(&self, containing_block_writing_mode: WritingMode)
        -> LogicalSides<BorderStyle>;
    fn border_width(&self, containing_block_writing_mode: WritingMode) -> LogicalSides<Au>;
    fn physical_margin(&self) -> PhysicalSides<LengthPercentageOrAuto<'_>>;
    fn margin(
        &self,
        containing_block_writing_mode: WritingMode,
    ) -> LogicalSides<LengthPercentageOrAuto<'_>>;
    fn has_transform_or_perspective(&self, fragment_flags: FragmentFlags) -> bool;
    fn effective_z_index(&self, fragment_flags: FragmentFlags) -> i32;
    fn effective_overflow(&self) -> PhysicalVec<Overflow>;
    fn establishes_block_formatting_context(&self) -> bool;
    fn establishes_stacking_context(&self, fragment_flags: FragmentFlags) -> bool;
    fn establishes_scroll_container(&self) -> bool;
    fn establishes_containing_block_for_absolute_descendants(
        &self,
        fragment_flags: FragmentFlags,
    ) -> bool;
    fn establishes_containing_block_for_all_descendants(
        &self,
        fragment_flags: FragmentFlags,
    ) -> bool;
    fn preferred_aspect_ratio(
        &self,
        natural_aspect_ratio: Option<CSSFloat>,
        containing_block: Option<&ContainingBlock>,
        containing_block_writing_mode: WritingMode,
    ) -> Option<AspectRatio>;
    fn background_is_transparent(&self) -> bool;
    fn get_webrender_primitive_flags(&self) -> wr::PrimitiveFlags;
    fn bidi_control_chars(&self) -> (&'static str, &'static str);
    fn resolve_align_self(
        &self,
        resolved_auto_value: AlignItems,
        resolved_normal_value: AlignItems,
    ) -> AlignItems;
}

impl ComputedValuesExt for ComputedValues {
    fn physical_box_offsets(&self) -> PhysicalSides<LengthPercentageOrAuto<'_>> {
        fn convert(inset: &Inset) -> LengthPercentageOrAuto<'_> {
            match inset {
                Inset::LengthPercentage(ref v) => LengthPercentageOrAuto::LengthPercentage(v),
                Inset::Auto => LengthPercentageOrAuto::Auto,
                Inset::AnchorFunction(_) => unreachable!("anchor() should be disabled"),
                Inset::AnchorSizeFunction(_) => unreachable!("anchor-size() should be disabled"),
            }
        }
        let position = self.get_position();
        PhysicalSides::new(
            convert(&position.top),
            convert(&position.right),
            convert(&position.bottom),
            convert(&position.left),
        )
    }

    fn box_offsets(
        &self,
        containing_block: &ContainingBlock,
    ) -> LogicalSides<LengthPercentageOrAuto<'_>> {
        LogicalSides::from_physical(
            &self.physical_box_offsets(),
            containing_block.style.writing_mode,
        )
    }

    fn box_size(
        &self,
        containing_block_writing_mode: WritingMode,
    ) -> LogicalVec2<Size<LengthPercentage>> {
        let position = self.get_position();
        LogicalVec2::from_physical_size(
            &PhysicalSize::new(
                position.clone_width().into(),
                position.clone_height().into(),
            ),
            containing_block_writing_mode,
        )
    }

    fn min_box_size(
        &self,
        containing_block_writing_mode: WritingMode,
    ) -> LogicalVec2<Size<LengthPercentage>> {
        let position = self.get_position();
        LogicalVec2::from_physical_size(
            &PhysicalSize::new(
                position.clone_min_width().into(),
                position.clone_min_height().into(),
            ),
            containing_block_writing_mode,
        )
    }

    fn max_box_size(
        &self,
        containing_block_writing_mode: WritingMode,
    ) -> LogicalVec2<Size<LengthPercentage>> {
        let position = self.get_position();
        LogicalVec2::from_physical_size(
            &PhysicalSize::new(
                position.clone_max_width().into(),
                position.clone_max_height().into(),
            ),
            containing_block_writing_mode,
        )
    }

    fn content_box_size(
        &self,
        containing_block: &ContainingBlock,
        pbm: &PaddingBorderMargin,
    ) -> LogicalVec2<Size<Au>> {
        let box_size = self
            .box_size(containing_block.style.writing_mode)
            .percentages_relative_to(containing_block);
        self.content_box_size_for_box_size(box_size, pbm)
    }

    fn content_box_size_deprecated(
        &self,
        containing_block: &ContainingBlock,
        pbm: &PaddingBorderMargin,
    ) -> LogicalVec2<AuOrAuto> {
        self.content_box_size(containing_block, pbm)
            .map(Size::to_auto_or)
    }

    fn content_box_size_for_box_size(
        &self,
        box_size: LogicalVec2<Size<Au>>,
        pbm: &PaddingBorderMargin,
    ) -> LogicalVec2<Size<Au>> {
        match self.get_position().box_sizing {
            BoxSizing::ContentBox => box_size,
            BoxSizing::BorderBox => LogicalVec2 {
                // These may be negative, but will later be clamped by `min-width`/`min-height`
                // which is clamped to zero.
                inline: box_size
                    .inline
                    .map(|value| value - pbm.padding_border_sums.inline),
                block: box_size
                    .block
                    .map(|value| value - pbm.padding_border_sums.block),
            },
        }
    }

    fn content_min_box_size(
        &self,
        containing_block: &ContainingBlock,
        pbm: &PaddingBorderMargin,
    ) -> LogicalVec2<Size<Au>> {
        let box_size = self
            .min_box_size(containing_block.style.writing_mode)
            .percentages_relative_to(containing_block);
        self.content_min_box_size_for_min_size(box_size, pbm)
    }

    fn content_min_box_size_deprecated(
        &self,
        containing_block: &ContainingBlock,
        pbm: &PaddingBorderMargin,
    ) -> LogicalVec2<AuOrAuto> {
        self.content_min_box_size(containing_block, pbm)
            .map(Size::to_auto_or)
    }

    fn content_min_box_size_for_min_size(
        &self,
        min_box_size: LogicalVec2<Size<Au>>,
        pbm: &PaddingBorderMargin,
    ) -> LogicalVec2<Size<Au>> {
        match self.get_position().box_sizing {
            BoxSizing::ContentBox => min_box_size,
            BoxSizing::BorderBox => LogicalVec2 {
                // Clamp to zero to make sure the used size components are non-negative
                inline: min_box_size
                    .inline
                    .map(|value| (value - pbm.padding_border_sums.inline).max(Au::zero())),
                block: min_box_size
                    .block
                    .map(|value| (value - pbm.padding_border_sums.block).max(Au::zero())),
            },
        }
    }

    fn content_max_box_size(
        &self,
        containing_block: &ContainingBlock,
        pbm: &PaddingBorderMargin,
    ) -> LogicalVec2<Size<Au>> {
        let max_box_size = self
            .max_box_size(containing_block.style.writing_mode)
            .percentages_relative_to(containing_block);

        self.content_max_box_size_for_max_size(max_box_size, pbm)
    }

    fn content_max_box_size_deprecated(
        &self,
        containing_block: &ContainingBlock,
        pbm: &PaddingBorderMargin,
    ) -> LogicalVec2<Option<Au>> {
        self.content_max_box_size(containing_block, pbm)
            .map(Size::to_numeric)
    }

    fn content_max_box_size_for_max_size(
        &self,
        max_box_size: LogicalVec2<Size<Au>>,
        pbm: &PaddingBorderMargin,
    ) -> LogicalVec2<Size<Au>> {
        match self.get_position().box_sizing {
            BoxSizing::ContentBox => max_box_size,
            BoxSizing::BorderBox => {
                // This may be negative, but will later be clamped by `min-width`
                // which itself is clamped to zero.
                LogicalVec2 {
                    inline: max_box_size
                        .inline
                        .map(|value| value - pbm.padding_border_sums.inline),
                    block: max_box_size
                        .block
                        .map(|value| value - pbm.padding_border_sums.block),
                }
            },
        }
    }

    fn content_box_sizes_and_padding_border_margin(
        &self,
        containing_block: &IndefiniteContainingBlock,
    ) -> (
        LogicalVec2<Size<Au>>,
        LogicalVec2<Size<Au>>,
        LogicalVec2<Size<Au>>,
        PaddingBorderMargin,
        bool, /* depends_on_block_constraints */
    ) {
        // <https://drafts.csswg.org/css-sizing-3/#cyclic-percentage-contribution>
        // If max size properties or preferred size properties are set to a value containing
        // indefinite percentages, we treat the entire value as the initial value of the property.
        // However, for min size properties, as well as for margins and paddings,
        // we instead resolve indefinite percentages against zero.
        let containing_block_size = containing_block.size.map(|value| value.non_auto());
        let containing_block_size_auto_is_zero =
            containing_block_size.map(|value| value.unwrap_or_else(Au::zero));
        let writing_mode = containing_block.style.writing_mode;
        let pbm = self.padding_border_margin_with_writing_mode_and_containing_block_inline_size(
            writing_mode,
            containing_block.size.inline.auto_is(Au::zero),
        );
        let box_size = self.box_size(writing_mode);
        let min_size = self.min_box_size(writing_mode);
        let max_size = self.max_box_size(writing_mode);
        let depends_on_block_constraints = |size: &Size<LengthPercentage>| {
            match size {
                // fit-content is like clamp(min-content, stretch, max-content), but currently
                // min-content and max-content have the same behavior in the block axis,
                // so there is no dependency on block constraints.
                // TODO: for flex and grid layout, min-content and max-content should be different.
                // TODO: We are assuming that Size::Initial doesn't stretch. However, it may actually
                // stretch flex and grid items depending on the CSS Align properties, in that case
                // the caller needs to take care of it.
                Size::Stretch => true,
                Size::Numeric(length_percentage) => length_percentage.has_percentage(),
                _ => false,
            }
        };
        let depends_on_block_constraints = depends_on_block_constraints(&box_size.block) ||
            depends_on_block_constraints(&min_size.block) ||
            depends_on_block_constraints(&max_size.block);
        let box_size = box_size.maybe_percentages_relative_to_basis(&containing_block_size);
        let content_box_size = self
            .content_box_size_for_box_size(box_size, &pbm)
            .map(|v| v.map(Au::from));
        let min_size = min_size.percentages_relative_to_basis(&containing_block_size_auto_is_zero);
        let content_min_size = self
            .content_min_box_size_for_min_size(min_size, &pbm)
            .map(|v| v.map(Au::from));
        let max_size = max_size.maybe_percentages_relative_to_basis(&containing_block_size);
        let content_max_size = self
            .content_max_box_size_for_max_size(max_size, &pbm)
            .map(|v| v.map(Au::from));
        (
            content_box_size,
            content_min_size,
            content_max_size,
            pbm,
            depends_on_block_constraints,
        )
    }

    fn content_box_sizes_and_padding_border_margin_deprecated(
        &self,
        containing_block: &IndefiniteContainingBlock,
    ) -> (
        LogicalVec2<AuOrAuto>,
        LogicalVec2<AuOrAuto>,
        LogicalVec2<Option<Au>>,
        PaddingBorderMargin,
        bool, /* depends_on_block_constraints */
    ) {
        let (
            content_box_size,
            content_min_size,
            content_max_size,
            pbm,
            depends_on_block_constraints,
        ) = self.content_box_sizes_and_padding_border_margin(containing_block);
        (
            content_box_size.map(Size::to_auto_or),
            content_min_size.map(Size::to_auto_or),
            content_max_size.map(Size::to_numeric),
            pbm,
            depends_on_block_constraints,
        )
    }

    fn padding_border_margin(&self, containing_block: &ContainingBlock) -> PaddingBorderMargin {
        self.padding_border_margin_with_writing_mode_and_containing_block_inline_size(
            containing_block.style.writing_mode,
            containing_block.inline_size,
        )
    }

    fn padding_border_margin_for_intrinsic_size(
        &self,
        writing_mode: WritingMode,
    ) -> PaddingBorderMargin {
        let padding = self
            .padding(writing_mode)
            .percentages_relative_to(Au::zero());
        let border = self.border_width(writing_mode);
        let margin = self
            .margin(writing_mode)
            .percentages_relative_to(Au::zero());
        PaddingBorderMargin {
            padding_border_sums: LogicalVec2 {
                inline: padding.inline_sum() + border.inline_sum(),
                block: padding.block_sum() + border.block_sum(),
            },
            padding,
            border,
            margin,
        }
    }

    fn padding_border_margin_with_writing_mode_and_containing_block_inline_size(
        &self,
        writing_mode: WritingMode,
        containing_block_inline_size: Au,
    ) -> PaddingBorderMargin {
        let padding = self
            .padding(writing_mode)
            .percentages_relative_to(containing_block_inline_size);
        let border = self.border_width(writing_mode);
        let margin = self
            .margin(writing_mode)
            .percentages_relative_to(containing_block_inline_size);
        PaddingBorderMargin {
            padding_border_sums: LogicalVec2 {
                inline: padding.inline_sum() + border.inline_sum(),
                block: padding.block_sum() + border.block_sum(),
            },
            padding,
            border,
            margin,
        }
    }

    fn padding(
        &self,
        containing_block_writing_mode: WritingMode,
    ) -> LogicalSides<&LengthPercentage> {
        let padding = self.get_padding();
        LogicalSides::from_physical(
            &PhysicalSides::new(
                &padding.padding_top.0,
                &padding.padding_right.0,
                &padding.padding_bottom.0,
                &padding.padding_left.0,
            ),
            containing_block_writing_mode,
        )
    }

    fn border_style(
        &self,
        containing_block_writing_mode: WritingMode,
    ) -> LogicalSides<BorderStyle> {
        let border = self.get_border();
        LogicalSides::from_physical(
            &PhysicalSides::new(
                border.border_top_style,
                border.border_right_style,
                border.border_bottom_style,
                border.border_left_style,
            ),
            containing_block_writing_mode,
        )
    }

    fn border_width(&self, containing_block_writing_mode: WritingMode) -> LogicalSides<Au> {
        let border = self.get_border();
        LogicalSides::from_physical(
            &PhysicalSides::new(
                border.border_top_width,
                border.border_right_width,
                border.border_bottom_width,
                border.border_left_width,
            ),
            containing_block_writing_mode,
        )
    }

    fn physical_margin(&self) -> PhysicalSides<LengthPercentageOrAuto<'_>> {
        fn convert(inset: &Margin) -> LengthPercentageOrAuto<'_> {
            match inset {
                Margin::LengthPercentage(ref v) => LengthPercentageOrAuto::LengthPercentage(v),
                Margin::Auto => LengthPercentageOrAuto::Auto,
                Margin::AnchorSizeFunction(_) => unreachable!("anchor-size() should be disabled"),
            }
        }
        let margin = self.get_margin();
        PhysicalSides::new(
            convert(&margin.margin_top),
            convert(&margin.margin_right),
            convert(&margin.margin_bottom),
            convert(&margin.margin_left),
        )
    }

    fn margin(
        &self,
        containing_block_writing_mode: WritingMode,
    ) -> LogicalSides<LengthPercentageOrAuto<'_>> {
        LogicalSides::from_physical(&self.physical_margin(), containing_block_writing_mode)
    }

    /// Returns true if this style has a transform, or perspective property set and
    /// it applies to this element.
    fn has_transform_or_perspective(&self, fragment_flags: FragmentFlags) -> bool {
        // "A transformable element is an element in one of these categories:
        //   * all elements whose layout is governed by the CSS box model except for
        //     non-replaced inline boxes, table-column boxes, and table-column-group
        //     boxes,
        //   * all SVG paint server elements, the clipPath element  and SVG renderable
        //     elements with the exception of any descendant element of text content
        //     elements."
        // https://drafts.csswg.org/css-transforms/#transformable-element
        if self.get_box().display.is_inline_flow() &&
            !fragment_flags.contains(FragmentFlags::IS_REPLACED)
        {
            return false;
        }

        !self.get_box().transform.0.is_empty() || self.get_box().perspective != Perspective::None
    }

    /// Get the effective z-index of this fragment. Z-indices only apply to positioned elements
    /// per CSS 2 9.9.1 (<http://www.w3.org/TR/CSS2/visuren.html#z-index>), so this value may differ
    /// from the value specified in the style.
    fn effective_z_index(&self, fragment_flags: FragmentFlags) -> i32 {
        // From <https://drafts.csswg.org/css-flexbox/#painting>:
        // > Flex items paint exactly the same as inline blocks [CSS2], except that order-modified
        // > document order is used in place of raw document order, and z-index values other than auto
        // > create a stacking context even if position is static (behaving exactly as if position
        // > were relative).
        match self.get_box().position {
            ComputedPosition::Static if !fragment_flags.contains(FragmentFlags::IS_FLEX_ITEM) => 0,
            _ => self.get_position().z_index.integer_or(0),
        }
    }

    /// Get the effective overflow of this box. The property only applies to block containers,
    /// flex containers, and grid containers. And some box types only accept a few values.
    /// <https://www.w3.org/TR/css-overflow-3/#overflow-control>
    fn effective_overflow(&self) -> PhysicalVec<Overflow> {
        let style_box = self.get_box();
        let overflow_x = style_box.overflow_x;
        let overflow_y = style_box.overflow_y;
        let ignores_overflow = match style_box.display.inside() {
            stylo::DisplayInside::Table => {
                // According to <https://drafts.csswg.org/css-tables/#global-style-overrides>,
                // - overflow applies to table-wrapper boxes and not to table grid boxes.
                //   That's what Blink and WebKit do, however Firefox matches a CSSWG resolution that says
                //   the opposite: <https://lists.w3.org/Archives/Public/www-style/2012Aug/0298.html>
                //   Due to the way that we implement table-wrapper boxes, it's easier to align with Firefox.
                // - Tables ignore overflow values different than visible, clip and hidden.
                //   This affects both axes, to ensure they have the same scrollability.
                !matches!(self.pseudo(), Some(PseudoElement::ServoTableGrid)) ||
                    matches!(overflow_x, Overflow::Auto | Overflow::Scroll) ||
                    matches!(overflow_y, Overflow::Auto | Overflow::Scroll)
            },
            stylo::DisplayInside::TableColumn |
            stylo::DisplayInside::TableColumnGroup |
            stylo::DisplayInside::TableRow |
            stylo::DisplayInside::TableRowGroup |
            stylo::DisplayInside::TableHeaderGroup |
            stylo::DisplayInside::TableFooterGroup => {
                // <https://drafts.csswg.org/css-tables/#global-style-overrides>
                // Table-track and table-track-group boxes ignore overflow.
                true
            },
            _ => false,
        };
        if ignores_overflow {
            PhysicalVec::new(Overflow::Visible, Overflow::Visible)
        } else {
            PhysicalVec::new(overflow_x, overflow_y)
        }
    }

    /// Return true if this style is a normal block and establishes
    /// a new block formatting context.
    fn establishes_block_formatting_context(&self) -> bool {
        if self.establishes_scroll_container() {
            return true;
        }

        if self.get_column().is_multicol() {
            return true;
        }

        if self.get_column().column_span == ColumnSpan::All {
            return true;
        }

        // TODO: We need to handle CSS Contain here.
        false
    }

    /// Whether or not the `overflow` value of this style establishes a scroll container.
    fn establishes_scroll_container(&self) -> bool {
        self.effective_overflow().x.is_scrollable()
    }

    /// Returns true if this fragment establishes a new stacking context and false otherwise.
    fn establishes_stacking_context(&self, fragment_flags: FragmentFlags) -> bool {
        let effects = self.get_effects();
        if effects.opacity != 1.0 {
            return true;
        }

        if effects.mix_blend_mode != ComputedMixBlendMode::Normal {
            return true;
        }

        if self.has_transform_or_perspective(fragment_flags) {
            return true;
        }

        if !self.get_effects().filter.0.is_empty() {
            return true;
        }

        if self.get_box().transform_style == ComputedTransformStyle::Preserve3d ||
            self.overrides_transform_style()
        {
            return true;
        }

        // Fixed position and sticky position always create stacking contexts.
        // TODO(mrobinson): We need to handle sticky positioning here when we support it.
        if self.get_box().position == ComputedPosition::Fixed {
            return true;
        }

        if self.get_svg().clip_path != ClipPath::None {
            return true;
        }

        // Statically positioned fragments don't establish stacking contexts if the previous
        // conditions are not fulfilled. Furthermore, z-index doesn't apply to statically
        // positioned fragments (except for flex items, see below).
        //
        // From <https://drafts.csswg.org/css-flexbox/#painting>:
        // > Flex items paint exactly the same as inline blocks [CSS2], except that order-modified
        // > document order is used in place of raw document order, and z-index values other than auto
        // > create a stacking context even if position is static (behaving exactly as if position
        // > were relative).
        if self.get_box().position == ComputedPosition::Static &&
            !fragment_flags.contains(FragmentFlags::IS_FLEX_ITEM)
        {
            return false;
        }

        // For absolutely and relatively positioned fragments we only establish a stacking
        // context if there is a z-index set.
        // See https://www.w3.org/TR/CSS2/visuren.html#z-index
        !self.get_position().z_index.is_auto()
    }

    /// Returns true if this style establishes a containing block for absolute
    /// descendants (`position: absolute`). If this style happens to establish a
    /// containing block for “all descendants” (ie including `position: fixed`
    /// descendants) this method will return true, but a true return value does
    /// not imply that the style establishes a containing block for all descendants.
    /// Use `establishes_containing_block_for_all_descendants()` instead.
    fn establishes_containing_block_for_absolute_descendants(
        &self,
        fragment_flags: FragmentFlags,
    ) -> bool {
        if self.establishes_containing_block_for_all_descendants(fragment_flags) {
            return true;
        }

        self.clone_position() != ComputedPosition::Static
    }

    /// Returns true if this style establishes a containing block for
    /// all descendants, including fixed descendants (`position: fixed`).
    /// Note that this also implies that it establishes a containing block
    /// for absolute descendants (`position: absolute`).
    fn establishes_containing_block_for_all_descendants(
        &self,
        fragment_flags: FragmentFlags,
    ) -> bool {
        if self.has_transform_or_perspective(fragment_flags) {
            return true;
        }

        if !self.get_effects().filter.0.is_empty() {
            return true;
        }

        // TODO: We need to handle CSS Contain here.
        false
    }

    /// Resolve the preferred aspect ratio according to the given natural aspect
    /// ratio and the `aspect-ratio` property.
    /// See <https://drafts.csswg.org/css-sizing-4/#aspect-ratio>.
    fn preferred_aspect_ratio(
        &self,
        natural_aspect_ratio: Option<CSSFloat>,
        containing_block: Option<&ContainingBlock>,
        containing_block_writing_mode: WritingMode,
    ) -> Option<AspectRatio> {
        let GenericAspectRatio {
            auto,
            ratio: mut preferred_ratio,
        } = self.clone_aspect_ratio();

        // For all cases where a ratio is specified:
        // "If the <ratio> is degenerate, the property instead behaves as auto."
        if matches!(preferred_ratio, PreferredRatio::Ratio(ratio) if ratio.is_degenerate()) {
            preferred_ratio = PreferredRatio::None;
        }

        match (auto, preferred_ratio) {
            // The value `auto`. Either the ratio was not specified, or was
            // degenerate and set to PreferredRatio::None above.
            //
            // "Replaced elements with a natural aspect ratio use that aspect
            // ratio; otherwise the box has no preferred aspect ratio. Size
            // calculations involving the aspect ratio work with the content box
            // dimensions always."
            (_, PreferredRatio::None) => natural_aspect_ratio.map(|natural_ratio| AspectRatio {
                i_over_b: natural_ratio,
                box_sizing_adjustment: LogicalVec2::zero(),
            }),
            // "If both auto and a <ratio> are specified together, the preferred
            // aspect ratio is the specified ratio of width / height unless it
            // is a replaced element with a natural aspect ratio, in which case
            // that aspect ratio is used instead. In all cases, size
            // calculations involving the aspect ratio work with the content box
            // dimensions always."
            (true, PreferredRatio::Ratio(preferred_ratio)) => match natural_aspect_ratio {
                Some(natural_ratio) => Some(AspectRatio {
                    i_over_b: natural_ratio,
                    box_sizing_adjustment: LogicalVec2::zero(),
                }),
                None => Some(AspectRatio {
                    i_over_b: (preferred_ratio.0).0 / (preferred_ratio.1).0,
                    box_sizing_adjustment: LogicalVec2::zero(),
                }),
            },

            // "The box’s preferred aspect ratio is the specified ratio of width
            // / height. Size calculations involving the aspect ratio work with
            // the dimensions of the box specified by box-sizing."
            (false, PreferredRatio::Ratio(preferred_ratio)) => {
                // If the `box-sizing` is `border-box`, use the padding and
                // border when calculating the aspect ratio.
                let box_sizing_adjustment = match self.clone_box_sizing() {
                    BoxSizing::ContentBox => LogicalVec2::zero(),
                    BoxSizing::BorderBox => {
                        match containing_block {
                            Some(containing_block) => self.padding_border_margin(containing_block),
                            None => self.padding_border_margin_for_intrinsic_size(
                                containing_block_writing_mode,
                            ),
                        }
                        .padding_border_sums
                    },
                };
                Some(AspectRatio {
                    i_over_b: (preferred_ratio.0).0 / (preferred_ratio.1).0,
                    box_sizing_adjustment,
                })
            },
        }
    }

    /// Whether or not this style specifies a non-transparent background.
    fn background_is_transparent(&self) -> bool {
        let background = self.get_background();
        let color = self.resolve_color(background.background_color.clone());
        color.alpha == 0.0 &&
            background
                .background_image
                .0
                .iter()
                .all(|layer| matches!(layer, ComputedImageLayer::None))
    }

    /// Generate appropriate WebRender `PrimitiveFlags` that should be used
    /// for display items generated by the `Fragment` which owns this style.
    fn get_webrender_primitive_flags(&self) -> wr::PrimitiveFlags {
        match self.get_box().backface_visibility {
            BackfaceVisiblity::Visible => wr::PrimitiveFlags::default(),
            BackfaceVisiblity::Hidden => wr::PrimitiveFlags::empty(),
        }
    }

    /// If the 'unicode-bidi' property has a value other than 'normal', return the bidi control codes
    /// to inject before and after the text content of the element.
    /// See the table in <http://dev.w3.org/csswg/css-writing-modes/#unicode-bidi>.
    fn bidi_control_chars(&self) -> (&'static str, &'static str) {
        match (
            self.get_text().unicode_bidi,
            self.get_inherited_box().direction,
        ) {
            (UnicodeBidi::Normal, _) => ("", ""),
            (UnicodeBidi::Embed, Direction::Ltr) => ("\u{202a}", "\u{202c}"),
            (UnicodeBidi::Embed, Direction::Rtl) => ("\u{202b}", "\u{202c}"),
            (UnicodeBidi::Isolate, Direction::Ltr) => ("\u{2066}", "\u{2069}"),
            (UnicodeBidi::Isolate, Direction::Rtl) => ("\u{2067}", "\u{2069}"),
            (UnicodeBidi::BidiOverride, Direction::Ltr) => ("\u{202d}", "\u{202c}"),
            (UnicodeBidi::BidiOverride, Direction::Rtl) => ("\u{202e}", "\u{202c}"),
            (UnicodeBidi::IsolateOverride, Direction::Ltr) => {
                ("\u{2068}\u{202d}", "\u{202c}\u{2069}")
            },
            (UnicodeBidi::IsolateOverride, Direction::Rtl) => {
                ("\u{2068}\u{202e}", "\u{202c}\u{2069}")
            },
            (UnicodeBidi::Plaintext, _) => ("\u{2068}", "\u{2069}"),
        }
    }

    fn resolve_align_self(
        &self,
        resolved_auto_value: AlignItems,
        resolved_normal_value: AlignItems,
    ) -> AlignItems {
        match self.clone_align_self().0 .0 {
            AlignFlags::AUTO => resolved_auto_value,
            AlignFlags::NORMAL => resolved_normal_value,
            value => AlignItems(value),
        }
    }
}

impl From<stylo::Display> for Display {
    fn from(packed: stylo::Display) -> Self {
        let outside = packed.outside();
        let inside = packed.inside();

        let outside = match outside {
            stylo::DisplayOutside::Block => DisplayOutside::Block,
            stylo::DisplayOutside::Inline => DisplayOutside::Inline,
            stylo::DisplayOutside::TableCaption => {
                return Display::GeneratingBox(DisplayGeneratingBox::LayoutInternal(
                    DisplayLayoutInternal::TableCaption,
                ));
            },
            stylo::DisplayOutside::InternalTable => {
                let internal = match inside {
                    stylo::DisplayInside::TableRowGroup => DisplayLayoutInternal::TableRowGroup,
                    stylo::DisplayInside::TableColumn => DisplayLayoutInternal::TableColumn,
                    stylo::DisplayInside::TableColumnGroup => {
                        DisplayLayoutInternal::TableColumnGroup
                    },
                    stylo::DisplayInside::TableHeaderGroup => {
                        DisplayLayoutInternal::TableHeaderGroup
                    },
                    stylo::DisplayInside::TableFooterGroup => {
                        DisplayLayoutInternal::TableFooterGroup
                    },
                    stylo::DisplayInside::TableRow => DisplayLayoutInternal::TableRow,
                    stylo::DisplayInside::TableCell => DisplayLayoutInternal::TableCell,
                    _ => unreachable!("Non-internal DisplayInside found"),
                };
                return Display::GeneratingBox(DisplayGeneratingBox::LayoutInternal(internal));
            },
            // This should not be a value of DisplayInside, but oh well
            // special-case display: contents because we still want it to work despite the early return
            stylo::DisplayOutside::None if inside == stylo::DisplayInside::Contents => {
                return Display::Contents
            },
            stylo::DisplayOutside::None => return Display::None,
        };

        let inside = match packed.inside() {
            stylo::DisplayInside::Flow => DisplayInside::Flow {
                is_list_item: packed.is_list_item(),
            },
            stylo::DisplayInside::FlowRoot => DisplayInside::FlowRoot {
                is_list_item: packed.is_list_item(),
            },
            stylo::DisplayInside::Flex => DisplayInside::Flex,
            stylo::DisplayInside::Grid => todo!("Grid support is not yet implemented."),

            // These should not be values of DisplayInside, but oh well
            stylo::DisplayInside::None => return Display::None,
            stylo::DisplayInside::Contents => return Display::Contents,

            stylo::DisplayInside::Table => DisplayInside::Table,
            stylo::DisplayInside::TableRowGroup |
            stylo::DisplayInside::TableColumn |
            stylo::DisplayInside::TableColumnGroup |
            stylo::DisplayInside::TableHeaderGroup |
            stylo::DisplayInside::TableFooterGroup |
            stylo::DisplayInside::TableRow |
            stylo::DisplayInside::TableCell => unreachable!("Internal DisplayInside found"),
        };
        Display::GeneratingBox(DisplayGeneratingBox::OutsideInside { outside, inside })
    }
}

pub(crate) trait Clamp: Sized {
    fn clamp_below_max(self, max: Option<Self>) -> Self;
    fn clamp_between_extremums(self, min: Self, max: Option<Self>) -> Self;
}

impl Clamp for Au {
    fn clamp_below_max(self, max: Option<Self>) -> Self {
        match max {
            None => self,
            Some(max) => self.min(max),
        }
    }

    fn clamp_between_extremums(self, min: Self, max: Option<Self>) -> Self {
        self.clamp_below_max(max).max(min)
    }
}
