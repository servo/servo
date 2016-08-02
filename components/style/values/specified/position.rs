/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS handling for the specified value of
//! [`position`][position]s
//!
//! [position]: https://drafts.csswg.org/css-backgrounds-3/#position

use cssparser::{Parser, ToCss, Token};
use std::fmt;
use values::HasViewportPercentage;
use values::computed::position as computed_position;
use values::computed::{Context, ToComputedValue};
use values::specified::{LengthOrPercentage, Percentage};

#[derive(Debug, Clone, PartialEq, Copy)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct Position {
    pub horizontal: LengthOrPercentage,
    pub vertical: LengthOrPercentage,
}

impl ToCss for Position {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        try!(self.horizontal.to_css(dest));
        try!(dest.write_str(" "));
        try!(self.vertical.to_css(dest));
        Ok(())
    }
}

impl HasViewportPercentage for Position {
    fn has_viewport_percentage(&self) -> bool {
        self.horizontal.has_viewport_percentage() || self.vertical.has_viewport_percentage()
    }
}
// http://dev.w3.org/csswg/css2/colors.html#propdef-background-position
#[derive(Clone, PartialEq, Copy)]
pub enum PositionComponent {
    LengthOrPercentage(LengthOrPercentage),
    Center,
    Left,
    Right,
    Top,
    Bottom,
}

impl Position {
    pub fn new(first: PositionComponent, second: PositionComponent)
            -> Result<Position, ()> {
        let (horiz, vert) = match (category(first), category(second)) {
            // Don't allow two vertical keywords or two horizontal keywords.
            // also don't allow length/percentage values in the wrong position
            (PositionCategory::HorizontalKeyword, PositionCategory::HorizontalKeyword) |
            (PositionCategory::VerticalKeyword, PositionCategory::VerticalKeyword) |
            (PositionCategory::LengthOrPercentage, PositionCategory::HorizontalKeyword) |
            (PositionCategory::VerticalKeyword, PositionCategory::LengthOrPercentage) => return Err(()),

            // Swap if both are keywords and vertical precedes horizontal.
            (PositionCategory::VerticalKeyword, PositionCategory::HorizontalKeyword) |
            (PositionCategory::VerticalKeyword, PositionCategory::OtherKeyword) |
            (PositionCategory::OtherKeyword, PositionCategory::HorizontalKeyword) => (second, first),
            // By default, horizontal is first.
            _ => (first, second),
        };
        Ok(Position {
            horizontal: horiz.to_length_or_percentage(),
            vertical: vert.to_length_or_percentage(),
        })
    }

    pub fn parse(input: &mut Parser) -> Result<Position, ()> {
        let first = try!(PositionComponent::parse(input));
        let second = input.try(PositionComponent::parse)
            .unwrap_or(PositionComponent::Center);
        Position::new(first, second)
    }
}

// Collapse `Position` into a few categories to simplify the above `match` expression.
enum PositionCategory {
    HorizontalKeyword,
    VerticalKeyword,
    OtherKeyword,
    LengthOrPercentage,
}

fn category(p: PositionComponent) -> PositionCategory {
    match p {
        PositionComponent::Left |
        PositionComponent::Right =>
            PositionCategory::HorizontalKeyword,
        PositionComponent::Top |
        PositionComponent::Bottom =>
            PositionCategory::VerticalKeyword,
        PositionComponent::Center =>
            PositionCategory::OtherKeyword,
        PositionComponent::LengthOrPercentage(_) =>
            PositionCategory::LengthOrPercentage,
    }
}

impl ToComputedValue for Position {
    type ComputedValue = computed_position::Position;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> computed_position::Position {
        computed_position::Position {
            horizontal: self.horizontal.to_computed_value(context),
            vertical: self.vertical.to_computed_value(context),
        }
    }
}

impl HasViewportPercentage for PositionComponent {
    fn has_viewport_percentage(&self) -> bool {
        match *self {
            PositionComponent::LengthOrPercentage(length) => length.has_viewport_percentage(),
            _ => false
        }
    }
}

impl PositionComponent {
    pub fn parse(input: &mut Parser) -> Result<PositionComponent, ()> {
        input.try(LengthOrPercentage::parse)
        .map(PositionComponent::LengthOrPercentage)
        .or_else(|()| {
            match try!(input.next()) {
                Token::Ident(value) => {
                    match_ignore_ascii_case! { value,
                        "center" => Ok(PositionComponent::Center),
                        "left" => Ok(PositionComponent::Left),
                        "right" => Ok(PositionComponent::Right),
                        "top" => Ok(PositionComponent::Top),
                        "bottom" => Ok(PositionComponent::Bottom),
                        _ => Err(())
                    }
                },
                _ => Err(())
            }
        })
    }
    #[inline]
    pub fn to_length_or_percentage(self) -> LengthOrPercentage {
        match self {
            PositionComponent::LengthOrPercentage(value) => value,
            PositionComponent::Center => LengthOrPercentage::Percentage(Percentage(0.5)),
            PositionComponent::Left |
            PositionComponent::Top => LengthOrPercentage::Percentage(Percentage(0.0)),
            PositionComponent::Right |
            PositionComponent::Bottom => LengthOrPercentage::Percentage(Percentage(1.0)),
        }
    }
}
