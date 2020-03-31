/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::geom::{flow_relative, PhysicalSides, PhysicalSize};
use crate::ContainingBlock;
use style::computed_values::mix_blend_mode::T as ComputedMixBlendMode;
use style::computed_values::position::T as ComputedPosition;
use style::computed_values::transform_style::T as ComputedTransformStyle;
use style::properties::ComputedValues;
use style::values::computed::{Length, LengthOrAuto, LengthPercentage, LengthPercentageOrAuto};
use style::values::computed::{NonNegativeLengthPercentage, Size};
use style::values::generics::box_::Perspective;
use style::values::generics::length::MaxSize;
use style::values::specified::box_ as stylo;

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum Display {
    None,
    Contents,
    GeneratingBox(DisplayGeneratingBox),
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum DisplayGeneratingBox {
    OutsideInside {
        outside: DisplayOutside,
        inside: DisplayInside,
        // list_item: bool,
    },
    // Layout-internal display types go here:
    // https://drafts.csswg.org/css-display-3/#layout-specific-display
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum DisplayOutside {
    Block,
    Inline,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum DisplayInside {
    Flow,
    FlowRoot,
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
    fn box_offsets(&self) -> flow_relative::Sides<LengthPercentageOrAuto>;
    fn box_size(&self) -> flow_relative::Vec2<LengthPercentageOrAuto>;
    fn min_box_size(&self) -> flow_relative::Vec2<LengthPercentageOrAuto>;
    fn max_box_size(&self) -> flow_relative::Vec2<MaxSize<LengthPercentage>>;
    fn padding_border_margin(&self, containing_block: &ContainingBlock) -> PaddingBorderMargin;
    fn padding(&self) -> flow_relative::Sides<LengthPercentage>;
    fn border_width(&self) -> flow_relative::Sides<Length>;
    fn margin(&self) -> flow_relative::Sides<LengthPercentageOrAuto>;
    fn has_transform_or_perspective(&self) -> bool;
    fn effective_z_index(&self) -> i32;
    fn establishes_stacking_context(&self) -> bool;
    fn establishes_containing_block(&self) -> bool;
    fn establishes_containing_block_for_all_descendants(&self) -> bool;
}

impl ComputedValuesExt for ComputedValues {
    fn inline_size_is_length(&self) -> bool {
        let position = self.get_position();
        let size = if self.writing_mode.is_horizontal() {
            &position.width
        } else {
            &position.height
        };
        matches!(size, Size::LengthPercentage(lp) if lp.0.to_length().is_some())
    }

    fn inline_box_offsets_are_both_non_auto(&self) -> bool {
        let position = self.get_position();
        let (a, b) = if self.writing_mode.is_horizontal() {
            (&position.left, &position.right)
        } else {
            (&position.top, &position.bottom)
        };
        !a.is_auto() && !b.is_auto()
    }

    #[inline]
    fn box_offsets(&self) -> flow_relative::Sides<LengthPercentageOrAuto> {
        let position = self.get_position();
        flow_relative::Sides::from_physical(
            &PhysicalSides::new(
                position.top.clone(),
                position.right.clone(),
                position.bottom.clone(),
                position.left.clone(),
            ),
            self.writing_mode,
        )
    }

    #[inline]
    fn box_size(&self) -> flow_relative::Vec2<LengthPercentageOrAuto> {
        let position = self.get_position();
        flow_relative::Vec2::from_physical_size(
            &PhysicalSize::new(
                size_to_length(position.width.clone()),
                size_to_length(position.height.clone()),
            ),
            self.writing_mode,
        )
    }

    #[inline]
    fn min_box_size(&self) -> flow_relative::Vec2<LengthPercentageOrAuto> {
        let position = self.get_position();
        flow_relative::Vec2::from_physical_size(
            &PhysicalSize::new(
                size_to_length(position.min_width.clone()),
                size_to_length(position.min_height.clone()),
            ),
            self.writing_mode,
        )
    }

    #[inline]
    fn max_box_size(&self) -> flow_relative::Vec2<MaxSize<LengthPercentage>> {
        let unwrap = |max_size: MaxSize<NonNegativeLengthPercentage>| match max_size {
            MaxSize::LengthPercentage(length) => MaxSize::LengthPercentage(length.0),
            MaxSize::None => MaxSize::None,
        };
        let position = self.get_position();
        flow_relative::Vec2::from_physical_size(
            &PhysicalSize::new(
                unwrap(position.max_width.clone()),
                unwrap(position.max_height.clone()),
            ),
            self.writing_mode,
        )
    }

    fn padding_border_margin(&self, containing_block: &ContainingBlock) -> PaddingBorderMargin {
        let cbis = containing_block.inline_size;
        let padding = self.padding().percentages_relative_to(cbis);
        let border = self.border_width();
        PaddingBorderMargin {
            padding_border_sums: flow_relative::Vec2 {
                inline: padding.inline_sum() + border.inline_sum(),
                block: padding.block_sum() + border.block_sum(),
            },
            padding,
            border,
            margin: self.margin().percentages_relative_to(cbis),
        }
    }

    fn padding(&self) -> flow_relative::Sides<LengthPercentage> {
        let padding = self.get_padding();
        flow_relative::Sides::from_physical(
            &PhysicalSides::new(
                padding.padding_top.0.clone(),
                padding.padding_right.0.clone(),
                padding.padding_bottom.0.clone(),
                padding.padding_left.0.clone(),
            ),
            self.writing_mode,
        )
    }

    fn border_width(&self) -> flow_relative::Sides<Length> {
        let border = self.get_border();
        flow_relative::Sides::from_physical(
            &PhysicalSides::new(
                border.border_top_width.0,
                border.border_right_width.0,
                border.border_bottom_width.0,
                border.border_left_width.0,
            ),
            self.writing_mode,
        )
    }

    fn margin(&self) -> flow_relative::Sides<LengthPercentageOrAuto> {
        let margin = self.get_margin();
        flow_relative::Sides::from_physical(
            &PhysicalSides::new(
                margin.margin_top.clone(),
                margin.margin_right.clone(),
                margin.margin_bottom.clone(),
                margin.margin_left.clone(),
            ),
            self.writing_mode,
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
}

impl From<stylo::Display> for Display {
    fn from(packed: stylo::Display) -> Self {
        let inside = match packed.inside() {
            stylo::DisplayInside::Flow => DisplayInside::Flow,
            stylo::DisplayInside::FlowRoot => DisplayInside::FlowRoot,

            // These should not be values of DisplayInside, but oh well
            stylo::DisplayInside::None => return Display::None,
            stylo::DisplayInside::Contents => return Display::Contents,
        };
        let outside = match packed.outside() {
            stylo::DisplayOutside::Block => DisplayOutside::Block,
            stylo::DisplayOutside::Inline => DisplayOutside::Inline,

            // This should not be a value of DisplayInside, but oh well
            stylo::DisplayOutside::None => return Display::None,
        };
        Display::GeneratingBox(DisplayGeneratingBox::OutsideInside {
            outside,
            inside,
            // list_item: packed.is_list_item(),
        })
    }
}

fn size_to_length(size: Size) -> LengthPercentageOrAuto {
    match size {
        Size::LengthPercentage(length) => {
            LengthPercentageOrAuto::LengthPercentage(length.0.clone())
        },
        Size::Auto => LengthPercentageOrAuto::Auto,
    }
}
