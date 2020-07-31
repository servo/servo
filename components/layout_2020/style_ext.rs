/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::geom::flow_relative;
use crate::geom::{LengthOrAuto, LengthPercentageOrAuto, PhysicalSides, PhysicalSize};
use crate::ContainingBlock;
use style::computed_values::mix_blend_mode::T as ComputedMixBlendMode;
use style::computed_values::position::T as ComputedPosition;
use style::computed_values::transform_style::T as ComputedTransformStyle;
use style::logical_geometry::WritingMode;
use style::properties::longhands::backface_visibility::computed_value::T as BackfaceVisiblity;
use style::properties::longhands::box_sizing::computed_value::T as BoxSizing;
use style::properties::ComputedValues;
use style::values::computed::image::Image as ComputedImageLayer;
use style::values::computed::{Length, LengthPercentage};
use style::values::computed::{NonNegativeLengthPercentage, Size};
use style::values::generics::box_::Perspective;
use style::values::generics::length::MaxSize;
use style::values::specified::box_ as stylo;
use style::Zero;
use webrender_api as wr;

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub(crate) enum Display {
    None,
    Contents,
    GeneratingBox(DisplayGeneratingBox),
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub(crate) enum DisplayGeneratingBox {
    OutsideInside {
        outside: DisplayOutside,
        inside: DisplayInside,
    },
    // Layout-internal display types go here:
    // https://drafts.csswg.org/css-display-3/#layout-specific-display
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub(crate) enum DisplayOutside {
    Block,
    Inline,
    TableCaption,
    InternalTable,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub(crate) enum DisplayInside {
    // “list-items are limited to the Flow Layout display types”
    // https://drafts.csswg.org/css-display/#list-items
    Flow { is_list_item: bool },
    FlowRoot { is_list_item: bool },
    Flex,
    Table,
    TableRowGroup,
    TableColumn,
    TableColumnGroup,
    TableHeaderGroup,
    TableFooterGroup,
    TableRow,
    TableCell,
}

/// Percentages resolved but not `auto` margins
pub(crate) struct PaddingBorderMargin {
    pub padding: flow_relative::Sides<Length>,
    pub border: flow_relative::Sides<Length>,
    pub margin: flow_relative::Sides<LengthOrAuto>,

    /// Pre-computed sums in each axis
    pub padding_border_sums: flow_relative::Vec2<Length>,
}

pub(crate) trait ComputedValuesExt {
    fn inline_size_is_length(&self) -> bool;
    fn inline_box_offsets_are_both_non_auto(&self) -> bool;
    fn box_offsets(
        &self,
        containing_block: &ContainingBlock,
    ) -> flow_relative::Sides<LengthPercentageOrAuto<'_>>;
    fn box_size(
        &self,
        containing_block_writing_mode: WritingMode,
    ) -> flow_relative::Vec2<LengthPercentageOrAuto<'_>>;
    fn min_box_size(
        &self,
        containing_block_writing_mode: WritingMode,
    ) -> flow_relative::Vec2<LengthPercentageOrAuto<'_>>;
    fn max_box_size(
        &self,
        containing_block_writing_mode: WritingMode,
    ) -> flow_relative::Vec2<Option<&LengthPercentage>>;
    fn content_box_size(
        &self,
        containing_block: &ContainingBlock,
        pbm: &PaddingBorderMargin,
    ) -> flow_relative::Vec2<LengthOrAuto>;
    fn content_min_box_size(
        &self,
        containing_block: &ContainingBlock,
        pbm: &PaddingBorderMargin,
    ) -> flow_relative::Vec2<LengthOrAuto>;
    fn content_max_box_size(
        &self,
        containing_block: &ContainingBlock,
        pbm: &PaddingBorderMargin,
    ) -> flow_relative::Vec2<Option<Length>>;
    fn padding_border_margin(&self, containing_block: &ContainingBlock) -> PaddingBorderMargin;
    fn padding(
        &self,
        containing_block_writing_mode: WritingMode,
    ) -> flow_relative::Sides<&LengthPercentage>;
    fn border_width(
        &self,
        containing_block_writing_mode: WritingMode,
    ) -> flow_relative::Sides<Length>;
    fn margin(
        &self,
        containing_block_writing_mode: WritingMode,
    ) -> flow_relative::Sides<LengthPercentageOrAuto<'_>>;
    fn has_transform_or_perspective(&self) -> bool;
    fn effective_z_index(&self) -> i32;
    fn establishes_stacking_context(&self) -> bool;
    fn establishes_containing_block(&self) -> bool;
    fn establishes_containing_block_for_all_descendants(&self) -> bool;
    fn background_is_transparent(&self) -> bool;
    fn get_webrender_primitive_flags(&self) -> wr::PrimitiveFlags;
}

impl ComputedValuesExt for ComputedValues {
    fn inline_size_is_length(&self) -> bool {
        let position = self.get_position();
        // FIXME: this is the wrong writing mode but we plan to remove eager content size computation.
        let size = if self.writing_mode.is_horizontal() {
            &position.width
        } else {
            &position.height
        };
        matches!(size, Size::LengthPercentage(lp) if lp.0.to_length().is_some())
    }

    fn inline_box_offsets_are_both_non_auto(&self) -> bool {
        let position = self.get_position();
        // FIXME: this is the wrong writing mode but we plan to remove eager content size computation.
        let (a, b) = if self.writing_mode.is_horizontal() {
            (&position.left, &position.right)
        } else {
            (&position.top, &position.bottom)
        };
        !a.is_auto() && !b.is_auto()
    }

    fn box_offsets(
        &self,
        containing_block: &ContainingBlock,
    ) -> flow_relative::Sides<LengthPercentageOrAuto<'_>> {
        let position = self.get_position();
        flow_relative::Sides::from_physical(
            &PhysicalSides::new(
                position.top.as_ref(),
                position.right.as_ref(),
                position.bottom.as_ref(),
                position.left.as_ref(),
            ),
            containing_block.style.writing_mode,
        )
    }

    fn box_size(
        &self,
        containing_block_writing_mode: WritingMode,
    ) -> flow_relative::Vec2<LengthPercentageOrAuto<'_>> {
        let position = self.get_position();
        flow_relative::Vec2::from_physical_size(
            &PhysicalSize::new(
                size_to_length(&position.width),
                size_to_length(&position.height),
            ),
            containing_block_writing_mode,
        )
    }

    fn min_box_size(
        &self,
        containing_block_writing_mode: WritingMode,
    ) -> flow_relative::Vec2<LengthPercentageOrAuto<'_>> {
        let position = self.get_position();
        flow_relative::Vec2::from_physical_size(
            &PhysicalSize::new(
                size_to_length(&position.min_width),
                size_to_length(&position.min_height),
            ),
            containing_block_writing_mode,
        )
    }

    fn max_box_size(
        &self,
        containing_block_writing_mode: WritingMode,
    ) -> flow_relative::Vec2<Option<&LengthPercentage>> {
        fn unwrap(max_size: &MaxSize<NonNegativeLengthPercentage>) -> Option<&LengthPercentage> {
            match max_size {
                MaxSize::LengthPercentage(length) => Some(&length.0),
                MaxSize::None => None,
            }
        }
        let position = self.get_position();
        flow_relative::Vec2::from_physical_size(
            &PhysicalSize::new(unwrap(&position.max_width), unwrap(&position.max_height)),
            containing_block_writing_mode,
        )
    }

    fn content_box_size(
        &self,
        containing_block: &ContainingBlock,
        pbm: &PaddingBorderMargin,
    ) -> flow_relative::Vec2<LengthOrAuto> {
        let box_size = self
            .box_size(containing_block.style.writing_mode)
            .percentages_relative_to(containing_block);
        match self.get_position().box_sizing {
            BoxSizing::ContentBox => box_size,
            BoxSizing::BorderBox => flow_relative::Vec2 {
                // These may be negative, but will later be clamped by `min-width`/`min-height`
                // which is clamped to zero.
                inline: box_size.inline.map(|i| i - pbm.padding_border_sums.inline),
                block: box_size.block.map(|b| b - pbm.padding_border_sums.block),
            },
        }
    }

    fn content_min_box_size(
        &self,
        containing_block: &ContainingBlock,
        pbm: &PaddingBorderMargin,
    ) -> flow_relative::Vec2<LengthOrAuto> {
        let min_box_size = self
            .min_box_size(containing_block.style.writing_mode)
            .percentages_relative_to(containing_block);
        match self.get_position().box_sizing {
            BoxSizing::ContentBox => min_box_size,
            BoxSizing::BorderBox => flow_relative::Vec2 {
                // Clamp to zero to make sure the used size components are non-negative
                inline: min_box_size
                    .inline
                    .map(|i| (i - pbm.padding_border_sums.inline).max(Length::zero())),
                block: min_box_size
                    .block
                    .map(|b| (b - pbm.padding_border_sums.block).max(Length::zero())),
            },
        }
    }

    fn content_max_box_size(
        &self,
        containing_block: &ContainingBlock,
        pbm: &PaddingBorderMargin,
    ) -> flow_relative::Vec2<Option<Length>> {
        let max_box_size = self
            .max_box_size(containing_block.style.writing_mode)
            .percentages_relative_to(containing_block);
        match self.get_position().box_sizing {
            BoxSizing::ContentBox => max_box_size,
            BoxSizing::BorderBox => {
                // This may be negative, but will later be clamped by `min-width`
                // which itself is clamped to zero.
                flow_relative::Vec2 {
                    inline: max_box_size
                        .inline
                        .map(|i| i - pbm.padding_border_sums.inline),
                    block: max_box_size
                        .block
                        .map(|b| b - pbm.padding_border_sums.block),
                }
            },
        }
    }

    fn padding_border_margin(&self, containing_block: &ContainingBlock) -> PaddingBorderMargin {
        let cbis = containing_block.inline_size;
        let padding = self
            .padding(containing_block.style.writing_mode)
            .percentages_relative_to(cbis);
        let border = self.border_width(containing_block.style.writing_mode);
        PaddingBorderMargin {
            padding_border_sums: flow_relative::Vec2 {
                inline: padding.inline_sum() + border.inline_sum(),
                block: padding.block_sum() + border.block_sum(),
            },
            padding,
            border,
            margin: self
                .margin(containing_block.style.writing_mode)
                .percentages_relative_to(cbis),
        }
    }

    fn padding(
        &self,
        containing_block_writing_mode: WritingMode,
    ) -> flow_relative::Sides<&LengthPercentage> {
        let padding = self.get_padding();
        flow_relative::Sides::from_physical(
            &PhysicalSides::new(
                &padding.padding_top.0,
                &padding.padding_right.0,
                &padding.padding_bottom.0,
                &padding.padding_left.0,
            ),
            containing_block_writing_mode,
        )
    }

    fn border_width(
        &self,
        containing_block_writing_mode: WritingMode,
    ) -> flow_relative::Sides<Length> {
        let border = self.get_border();
        flow_relative::Sides::from_physical(
            &PhysicalSides::new(
                border.border_top_width.0,
                border.border_right_width.0,
                border.border_bottom_width.0,
                border.border_left_width.0,
            ),
            containing_block_writing_mode,
        )
    }

    fn margin(
        &self,
        containing_block_writing_mode: WritingMode,
    ) -> flow_relative::Sides<LengthPercentageOrAuto<'_>> {
        let margin = self.get_margin();
        flow_relative::Sides::from_physical(
            &PhysicalSides::new(
                margin.margin_top.as_ref(),
                margin.margin_right.as_ref(),
                margin.margin_bottom.as_ref(),
                margin.margin_left.as_ref(),
            ),
            containing_block_writing_mode,
        )
    }

    /// Returns true if this style has a transform, or perspective property set.
    fn has_transform_or_perspective(&self) -> bool {
        !self.get_box().transform.0.is_empty() || self.get_box().perspective != Perspective::None
    }

    /// Get the effective z-index of this fragment. Z-indices only apply to positioned elements
    /// per CSS 2 9.9.1 (http://www.w3.org/TR/CSS2/visuren.html#z-index), so this value may differ
    /// from the value specified in the style.
    fn effective_z_index(&self) -> i32 {
        match self.get_box().position {
            ComputedPosition::Static => 0,
            _ => self.get_position().z_index.integer_or(0),
        }
    }

    /// Returns true if this fragment establishes a new stacking context and false otherwise.
    fn establishes_stacking_context(&self) -> bool {
        let effects = self.get_effects();
        if effects.opacity != 1.0 {
            return true;
        }

        if effects.mix_blend_mode != ComputedMixBlendMode::Normal {
            return true;
        }

        if self.has_transform_or_perspective() {
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

        // Statically positioned fragments don't establish stacking contexts if the previous
        // conditions are not fulfilled. Furthermore, z-index doesn't apply to statically
        // positioned fragments.
        if self.get_box().position == ComputedPosition::Static {
            return false;
        }

        // For absolutely and relatively positioned fragments we only establish a stacking
        // context if there is a z-index set.
        // See https://www.w3.org/TR/CSS2/visuren.html#z-index
        !self.get_position().z_index.is_auto()
    }

    fn establishes_containing_block(&self) -> bool {
        if self.establishes_containing_block_for_all_descendants() {
            return true;
        }

        self.clone_position() != ComputedPosition::Static
    }

    /// Returns true if this style establishes a containing block for all descendants
    /// including fixed and absolutely positioned ones.
    fn establishes_containing_block_for_all_descendants(&self) -> bool {
        if self.get_box().display.outside() != stylo::DisplayOutside::Inline &&
            self.has_transform_or_perspective()
        {
            return true;
        }

        if !self.get_effects().filter.0.is_empty() {
            return true;
        }

        // TODO: We need to handle CSS Contain here.
        false
    }

    /// Whether or not this style specifies a non-transparent background.
    fn background_is_transparent(&self) -> bool {
        let background = self.get_background();
        let color = self.resolve_color(background.background_color);
        color.alpha == 0 &&
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
}

impl From<stylo::Display> for Display {
    fn from(packed: stylo::Display) -> Self {
        let inside = match packed.inside() {
            stylo::DisplayInside::Flow => DisplayInside::Flow {
                is_list_item: packed.is_list_item(),
            },
            stylo::DisplayInside::FlowRoot => DisplayInside::FlowRoot {
                is_list_item: packed.is_list_item(),
            },
            stylo::DisplayInside::Flex => DisplayInside::Flex,
            stylo::DisplayInside::Table => DisplayInside::Table,
            stylo::DisplayInside::TableRowGroup => DisplayInside::TableRowGroup,
            stylo::DisplayInside::TableColumn => DisplayInside::TableColumn,
            stylo::DisplayInside::TableColumnGroup => DisplayInside::TableColumnGroup,
            stylo::DisplayInside::TableHeaderGroup => DisplayInside::TableHeaderGroup,
            stylo::DisplayInside::TableFooterGroup => DisplayInside::TableFooterGroup,
            stylo::DisplayInside::TableRow => DisplayInside::TableRow,
            stylo::DisplayInside::TableCell => DisplayInside::TableCell,
            // These should not be values of DisplayInside, but oh well
            stylo::DisplayInside::None => return Display::None,
            stylo::DisplayInside::Contents => return Display::Contents,
        };
        let outside = match packed.outside() {
            stylo::DisplayOutside::Block => DisplayOutside::Block,
            stylo::DisplayOutside::Inline => DisplayOutside::Inline,
            stylo::DisplayOutside::TableCaption => DisplayOutside::TableCaption,
            stylo::DisplayOutside::InternalTable => DisplayOutside::InternalTable,
            // This should not be a value of DisplayInside, but oh well
            stylo::DisplayOutside::None => return Display::None,
        };
        Display::GeneratingBox(DisplayGeneratingBox::OutsideInside { outside, inside })
    }
}

fn size_to_length(size: &Size) -> LengthPercentageOrAuto {
    match size {
        Size::LengthPercentage(length) => LengthPercentageOrAuto::LengthPercentage(&length.0),
        Size::Auto => LengthPercentageOrAuto::Auto,
    }
}
