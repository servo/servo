/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::geom::{flow_relative, physical};
use style::properties::ComputedValues;
use style::values::computed::{Length, LengthPercentage, LengthPercentageOrAuto, Size};
use style::values::specified::box_ as stylo;

pub use style::computed_values::direction::T as Direction;
pub use style::computed_values::position::T as Position;
pub use style::computed_values::writing_mode::T as WritingMode;

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

pub(crate) trait ComputedValuesExt {
    fn writing_mode(&self) -> (WritingMode, Direction);
    fn writing_mode_is_horizontal(&self) -> bool;
    fn inline_size_is_auto(&self) -> bool;
    fn inline_box_offsets_are_both_auto(&self) -> bool;
    fn box_offsets(&self) -> flow_relative::Sides<LengthPercentageOrAuto>;
    fn box_size(&self) -> flow_relative::Vec2<LengthPercentageOrAuto>;
    fn padding(&self) -> flow_relative::Sides<LengthPercentage>;
    fn border_width(&self) -> flow_relative::Sides<Length>;
    fn margin(&self) -> flow_relative::Sides<LengthPercentageOrAuto>;
}

impl ComputedValuesExt for ComputedValues {
    fn writing_mode(&self) -> (WritingMode, Direction) {
        let inherited_box = self.get_inherited_box();
        let writing_mode = inherited_box.writing_mode;
        let direction = inherited_box.direction;
        (writing_mode, direction)
    }

    fn writing_mode_is_horizontal(&self) -> bool {
        match self.get_inherited_box().writing_mode {
            WritingMode::HorizontalTb => true,
            WritingMode::VerticalLr | WritingMode::VerticalRl => false,
        }
    }

    fn inline_size_is_auto(&self) -> bool {
        let position = self.get_position();
        let size = if self.writing_mode_is_horizontal() {
            position.width
        } else {
            position.height
        };
        matches!(size, Size::Auto)
    }

    fn inline_box_offsets_are_both_auto(&self) -> bool {
        let position = self.get_position();
        let offsets = if self.writing_mode_is_horizontal() {
            (position.left, position.right)
        } else {
            (position.top, position.bottom)
        };
        matches!(
            offsets,
            (LengthPercentageOrAuto::Auto, LengthPercentageOrAuto::Auto)
        )
    }

    #[inline]
    fn box_offsets(&self) -> flow_relative::Sides<LengthPercentageOrAuto> {
        let position = self.get_position();
        physical::Sides {
            top: position.top,
            left: position.left,
            bottom: position.bottom,
            right: position.right,
        }
        .to_flow_relative(self.writing_mode())
    }

    #[inline]
    fn box_size(&self) -> flow_relative::Vec2<LengthPercentageOrAuto> {
        let position = self.get_position();
        physical::Vec2 {
            x: size_to_length(position.width),
            y: size_to_length(position.height),
        }
        .size_to_flow_relative(self.writing_mode())
    }

    #[inline]
    fn padding(&self) -> flow_relative::Sides<LengthPercentage> {
        let padding = self.get_padding();
        physical::Sides {
            top: padding.padding_top.0,
            left: padding.padding_left.0,
            bottom: padding.padding_bottom.0,
            right: padding.padding_right.0,
        }
        .to_flow_relative(self.writing_mode())
    }

    fn border_width(&self) -> flow_relative::Sides<Length> {
        let border = self.get_border();
        physical::Sides {
            top: border.border_top_width.0,
            left: border.border_left_width.0,
            bottom: border.border_bottom_width.0,
            right: border.border_right_width.0,
        }
        .to_flow_relative(self.writing_mode())
    }

    fn margin(&self) -> flow_relative::Sides<LengthPercentageOrAuto> {
        let margin = self.get_margin();
        physical::Sides {
            top: margin.margin_top,
            left: margin.margin_left,
            bottom: margin.margin_bottom,
            right: margin.margin_right,
        }
        .to_flow_relative(self.writing_mode())
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
        Size::LengthPercentage(length) => LengthPercentageOrAuto::LengthPercentage(length.0),
        Size::Auto => LengthPercentageOrAuto::Auto,
    }
}
