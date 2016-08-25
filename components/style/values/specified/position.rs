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
    pub horiz_keyword: Option<Keyword>,
    pub horiz_position: Option<LengthOrPercentage>,
    pub vert_keyword: Option<Keyword>,
    pub vert_position: Option<LengthOrPercentage>,
}

impl ToCss for Position {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        // TODO: canaltinova: We should add keywords, probably?
        if let Some(horiz_pos) = self.horiz_position {
            if let Some(horiz_key) = self.horiz_keyword {
                try!(horiz_key.to_css(dest));
            };
            try!(horiz_pos.to_css(dest));
        };

        try!(dest.write_str(" "));

        if let Some(vert_pos) = self.vert_position {
            if let Some(vert_key) = self.vert_keyword {
                try!(vert_key.to_css(dest));
            };
            try!(vert_pos.to_css(dest));
        };
        Ok(())
    }
}

impl HasViewportPercentage for Position {
    fn has_viewport_percentage(&self) -> bool {
        let horiz_viewport = if let Some(horiz_pos) = self.horiz_position {
            horiz_pos.has_viewport_percentage()
        } else {
            false
        };

        let vert_viewport = if let Some(vert_pos) = self.vert_position {
            vert_pos.has_viewport_percentage()
        } else {
            false
        };
        horiz_viewport || vert_viewport
    }
}

#[derive(Clone, PartialEq, Copy)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum Keyword {
    Center,
    Left,
    Right,
    Top,
    Bottom,
}

// http://dev.w3.org/csswg/css2/colors.html#propdef-background-position
#[derive(Clone, PartialEq, Copy)]
pub enum PositionComponent {
    Length(LengthOrPercentage),
    Keyword(Keyword),
}

impl Position {
    pub fn new(first_position: Option<PositionComponent>, second_position: Option<PositionComponent>,
               first_keyword: Option<PositionComponent>, second_keyword: Option<PositionComponent>)
            -> Result<Position, ()> {
        // Check firts and second positions, this is more like for 2 value backgrounds.
        let (mut horiz, mut vert) = if let Some(first_pos) = first_position  {
            if let Some(second_pos) = second_position {
                match (category(first_pos), category(second_pos)) {
                    // Don't allow two vertical keywords or two horizontal keywords.
                    // also don't allow length/percentage values in the wrong position
                    (PositionCategory::HorizontalKeyword, PositionCategory::HorizontalKeyword) |
                    (PositionCategory::VerticalKeyword, PositionCategory::VerticalKeyword) |
                    (PositionCategory::LengthOrPercentage, PositionCategory::HorizontalKeyword) |
                    (PositionCategory::VerticalKeyword, PositionCategory::LengthOrPercentage) => return Err(()),

                    // Swap if both are keywords and vertical precedes horizontal.
                    (PositionCategory::VerticalKeyword, PositionCategory::HorizontalKeyword) |
                    (PositionCategory::VerticalKeyword, PositionCategory::OtherKeyword) |
                    (PositionCategory::OtherKeyword, PositionCategory::HorizontalKeyword) =>
                        (second_position, first_position),
                    // By default, horizontal is first.
                    _ => (first_position, second_position),
                }
            } else {
                (first_position, second_position)
            }
        } else {
            (first_position, second_position)
        };

        // Unwrap for checking if values are at right place.
        let first_key = first_keyword.unwrap_or(PositionComponent::Keyword(Keyword::Left));
        let second_key = second_keyword.unwrap_or(PositionComponent::Keyword(Keyword::Top));

        // Check first and second keywords. This is for 4 value swapping.
        let (horiz_keyword, vert_keyword) = match (category(first_key), category(second_key)) {
            // Don't allow two vertical keywords or two horizontal keywords.
            // also don't allow length/percentage values in the wrong position
            (PositionCategory::HorizontalKeyword, PositionCategory::HorizontalKeyword) |
            (PositionCategory::VerticalKeyword, PositionCategory::VerticalKeyword) |
            (PositionCategory::LengthOrPercentage, PositionCategory::HorizontalKeyword) |
            (PositionCategory::VerticalKeyword, PositionCategory::LengthOrPercentage) => return Err(()),

            // Swap if both are keywords and vertical precedes horizontal.
            (PositionCategory::VerticalKeyword, PositionCategory::HorizontalKeyword) |
            (PositionCategory::VerticalKeyword, PositionCategory::OtherKeyword) |
            (PositionCategory::OtherKeyword, PositionCategory::HorizontalKeyword) => {
                let tmp = horiz;
                horiz = vert;
                vert = tmp;

                (second_keyword, first_keyword)
            },
            // By default, horizontal is first.
            _ => (first_keyword, second_keyword),
        };

        // Unwrap keywords from PositionComponent and wrap with Option.
        let (wrapped_horiz_key, wrapped_vert_key) = if let Some(PositionComponent::Keyword(horiz_key)) = horiz_keyword {
            if let Some(PositionComponent::Keyword(vert_key)) = vert_keyword {
                (Some(horiz_key), Some(vert_key))
            } else {
                (Some(horiz_key), None)
            }
        } else {
            (None, None)
        };

        Ok(Position {
            horiz_keyword: wrapped_horiz_key,
            horiz_position: Some(horiz.unwrap().to_length_or_percentage()),
            vert_keyword: wrapped_vert_key,
            vert_position: Some(vert.unwrap().to_length_or_percentage()),
        })
    }

    pub fn parse(input: &mut Parser) -> Result<Position, ()> {
        let first = try!(PositionComponent::parse(input));
        let second = input.try(PositionComponent::parse)
            .unwrap_or(PositionComponent::Keyword(Keyword::Center));

        // Try to parse third and fourth values
        if let Ok(third) = input.try(PositionComponent::parse) {
            if let Ok(fourth) = input.try(PositionComponent::parse) {
                // Handle 4 value background position
                Position::new(Some(second), Some(fourth), Some(first), Some(third))
            } else {
                // Handle 3 value background position
                if let PositionCategory::LengthOrPercentage = category(first) {
                    // "20px bottom 20%"
                    Position::new(Some(first), Some(third), None, Some(second))
                } else {
                    if let PositionCategory::LengthOrPercentage = category(second) {
                        if let PositionCategory::HorizontalKeyword = category(third) {
                            // "bottom 10% right"
                            Position::new(Some(second), None, Some(first), Some(third))
                        } else {
                            // "right 10px 50%"
                            Position::new(Some(second), Some(third), Some(first), None)
                        }
                    } else {
                        // "right bottom 10px"
                        Position::new(None, Some(third), Some(first), Some(second))
                    }
                }
            }
        } else {
            // Handle 2 value background position
            Position::new(Some(first), Some(second), None, None)
        }
    }
}

impl ToCss for Keyword {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            Keyword::Right => try!(dest.write_str("right ")),
            Keyword::Bottom => try!(dest.write_str("bottom ")),
            _ => (),
        };
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
    if let PositionComponent::Keyword(keyword) = p {
        match keyword {
            Keyword::Left |
            Keyword::Right =>
                PositionCategory::HorizontalKeyword,
            Keyword::Top |
            Keyword::Bottom =>
                PositionCategory::VerticalKeyword,
            Keyword::Center =>
                PositionCategory::OtherKeyword,
        }
    } else {
        PositionCategory::LengthOrPercentage
    }
}

impl ToComputedValue for Position {
    type ComputedValue = computed_position::Position;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> computed_position::Position {
        computed_position::Position {
            horizontal: self.horiz_position.to_computed_value(context),
            vertical: self.vert_position.to_computed_value(context),
        }
    }
}

impl HasViewportPercentage for PositionComponent {
    fn has_viewport_percentage(&self) -> bool {
        match *self {
            PositionComponent::Length(length) => length.has_viewport_percentage(),
            _ => false
        }
    }
}

impl PositionComponent {
    pub fn parse(input: &mut Parser) -> Result<PositionComponent, ()> {
        input.try(LengthOrPercentage::parse)
        .map(PositionComponent::Length)
        .or_else(|()| {
            match try!(input.next()) {
                Token::Ident(value) => {
                    match_ignore_ascii_case! { value,
                        "center" => Ok(PositionComponent::Keyword(Keyword::Center)),
                        "left" => Ok(PositionComponent::Keyword(Keyword::Left)),
                        "right" => Ok(PositionComponent::Keyword(Keyword::Right)),
                        "top" => Ok(PositionComponent::Keyword(Keyword::Top)),
                        "bottom" => Ok(PositionComponent::Keyword(Keyword::Bottom)),
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
            PositionComponent::Length(value) => value,
            PositionComponent::Keyword(keyword) if keyword == Keyword::Center =>
                LengthOrPercentage::Percentage(Percentage(0.5)),
            PositionComponent::Keyword(keyword) if keyword == Keyword::Left ||
            keyword == Keyword::Top => LengthOrPercentage::Percentage(Percentage(0.0)),
            PositionComponent::Keyword(keyword) if keyword == Keyword::Right ||
            keyword == Keyword::Bottom => LengthOrPercentage::Percentage(Percentage(1.0)),
            PositionComponent::Keyword(_) => unimplemented!(), // TODO: All keywords are covered but rust forcing me to add this too?
        }
    }
}
