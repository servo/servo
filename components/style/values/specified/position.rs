/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS handling for the specified value of
//! [`position`][position]s
//!
//! [position]: https://drafts.csswg.org/css-backgrounds-3/#position

use app_units::Au;
use cssparser::{Parser, Token};
use parser::Parse;
use std::fmt;
use style_traits::ToCss;
use values::HasViewportPercentage;
use values::computed::{CalcLengthOrPercentage, Context};
use values::computed::{LengthOrPercentage as ComputedLengthOrPercentage, ToComputedValue};
use values::computed::position as computed_position;
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
        let mut space_at_last = false;
        if let Some(horiz_key) = self.horiz_keyword {
            try!(horiz_key.to_css(dest));
            try!(dest.write_str(" "));
            space_at_last = true;
        };
        if let Some(horiz_pos) = self.horiz_position {
            try!(horiz_pos.to_css(dest));
            try!(dest.write_str(" "));
            space_at_last = true;
        };
        if let Some(vert_key) = self.vert_keyword {
            try!(vert_key.to_css(dest));
            space_at_last = false;
        };
        if let Some(vert_pos) = self.vert_position {
            if space_at_last == false {
                try!(dest.write_str(" "));
            }
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

#[derive(Debug, Clone, PartialEq, Copy)]
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
    pub fn new(mut first_position: Option<PositionComponent>, mut second_position: Option<PositionComponent>,
               first_keyword: Option<PositionComponent>, second_keyword: Option<PositionComponent>)
            -> Result<Position, ()> {
        // Unwrap for checking if values are at right place.
        let first_key = first_keyword.unwrap_or(PositionComponent::Keyword(Keyword::Left));
        let second_key = second_keyword.unwrap_or(PositionComponent::Keyword(Keyword::Top));

        // Check if position specified after center keyword.
        if let PositionCategory::OtherKeyword = category(first_key) {
            if let Some(_) = first_position {
                return Err(());
            };
        };
        if let PositionCategory::OtherKeyword = category(second_key) {
            if let Some(_) = second_position {
                return Err(());
            };
        };

        // Check first and second keywords for both 2 and 4 value positions.
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
                let tmp = first_position;
                first_position = second_position;
                second_position = tmp;

                (second_keyword, first_keyword)
            },
            // By default, horizontal is first.
            _ => (first_keyword, second_keyword),
        };

        // Unwrap positions from PositionComponent and wrap with Option
        let (first_position, second_position) = if let Some(PositionComponent::Length(horiz_pos)) = first_position {
            if let Some(PositionComponent::Length(vert_pos)) = second_position {
                (Some(horiz_pos), Some(vert_pos))
            } else {
                (Some(horiz_pos), None)
            }
        } else {
            if let Some(PositionComponent::Length(vert_pos)) = second_position {
                (None, Some(vert_pos))
            } else {
                (None, None)
            }
        };

        // Unwrap keywords from PositionComponent and wrap with Option.
        let (horizontal_keyword, vertical_keyword) = if let Some(PositionComponent::Keyword(horiz_key)) =
                                                     horiz_keyword {
            if let Some(PositionComponent::Keyword(vert_key)) = vert_keyword {
                (Some(horiz_key), Some(vert_key))
            } else {
                (Some(horiz_key), None)
            }
        } else {
            if let Some(PositionComponent::Keyword(vert_key)) = vert_keyword {
                (None, Some(vert_key))
            } else {
                (None, None)
            }
        };

        Ok(Position {
            horiz_keyword: horizontal_keyword,
            horiz_position: first_position,
            vert_keyword: vertical_keyword,
            vert_position: second_position,
        })
    }

    pub fn center() -> Position {
        Position {
            horiz_keyword: Some(Keyword::Center),
            horiz_position: None,
            vert_keyword: Some(Keyword::Center),
            vert_position: None,
        }
    }
}

impl Parse for Position {
    fn parse(input: &mut Parser) -> Result<Self, ()> {
        let first = try!(PositionComponent::parse(input));
        let second = input.try(PositionComponent::parse)
            .unwrap_or(PositionComponent::Keyword(Keyword::Center));

        // Try to parse third and fourth values
        if let Ok(third) = input.try(PositionComponent::parse) {
            if let Ok(fourth) = input.try(PositionComponent::parse) {
                // Handle 4 value background position
                Position::new(Some(second), Some(fourth), Some(first), Some(third))
            } else {
                // Handle 3 value background position there are several options:
                if let PositionCategory::LengthOrPercentage = category(first) {
                    // "length keyword length"
                    Position::new(Some(first), Some(third), None, Some(second))
                } else {
                    if let PositionCategory::LengthOrPercentage = category(second) {
                        if let PositionCategory::LengthOrPercentage = category(third) {
                            // "keyword length length"
                            Position::new(Some(second), Some(third), Some(first), None)
                        } else {
                            // "keyword length keyword"
                            Position::new(Some(second), None, Some(first), Some(third))
                        }
                    } else {
                        // "keyword keyword length"
                        Position::new(None, Some(third), Some(first), Some(second))
                    }
                }
            }
        } else {
            // Handle 2 value background position.
            if let PositionCategory::LengthOrPercentage = category(first) {
                if let PositionCategory::LengthOrPercentage = category(second) {
                    Position::new(Some(first), Some(second), None, None)
                } else {
                    Position::new(Some(first), None, None, Some(second))
                }
            } else {
                if let PositionCategory::LengthOrPercentage = category(second) {
                    Position::new(None, Some(second), Some(first), None)
                } else {
                    Position::new(None, None, Some(first), Some(second))
                }
            }
        }
    }
}

impl Keyword {
    pub fn to_length_or_percentage(self) -> LengthOrPercentage {
        match self {
            Keyword::Center => LengthOrPercentage::Percentage(Percentage(0.5)),
            Keyword::Left | Keyword::Top => LengthOrPercentage::Percentage(Percentage(0.0)),
            Keyword::Right | Keyword::Bottom => LengthOrPercentage::Percentage(Percentage(1.0)),
        }
    }
}

impl ToCss for Keyword {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            Keyword::Center => try!(dest.write_str("center")),
            Keyword::Left => try!(dest.write_str("left")),
            Keyword::Right => try!(dest.write_str("right")),
            Keyword::Top => try!(dest.write_str("top")),
            Keyword::Bottom => try!(dest.write_str("bottom")),
        }
        Ok(())
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
        let horiz_keyword = self.horiz_keyword.unwrap_or(Keyword::Left);
        let vert_keyword = self.vert_keyword.unwrap_or(Keyword::Top);

        // Construct horizontal computed LengthOrPercentage
        let horizontal = match horiz_keyword {
            Keyword::Right => {
                if let Some(x) = self.horiz_position {
                    let (length, percentage) = match x {
                        LengthOrPercentage::Percentage(Percentage(y)) => (Au(0), Some(1.0 - y)),
                        LengthOrPercentage::Length(y) => (-y.to_computed_value(context), Some(1.0)),
                        _ => (Au(0), None),
                    };
                    ComputedLengthOrPercentage::Calc(CalcLengthOrPercentage {
                        length: length,
                        percentage: percentage
                    })
                } else {
                    ComputedLengthOrPercentage::Percentage(1.0)
                }
            },
            Keyword::Center => {
                horiz_keyword.to_length_or_percentage().to_computed_value(context)
            },
             _ => {
                let horiz = self.horiz_position.unwrap_or(LengthOrPercentage::Percentage(Percentage(0.0)));
                horiz.to_computed_value(context)
            },
        };

        // Construct vertical computed LengthOrPercentage
        let vertical = match vert_keyword {
            Keyword::Bottom => {
                if let Some(x) = self.vert_position {
                    let (length, percentage) = match x {
                        LengthOrPercentage::Percentage(Percentage(y)) => (Au(0), Some(1.0 - y)),
                        LengthOrPercentage::Length(y) => (-y.to_computed_value(context), Some(1.0)),
                        _ => (Au(0), None),
                    };
                    ComputedLengthOrPercentage::Calc(CalcLengthOrPercentage {
                        length: length,
                        percentage: percentage
                    })
                } else {
                    ComputedLengthOrPercentage::Percentage(1.0)
                }
            },
            Keyword::Center => {
                vert_keyword.to_length_or_percentage().to_computed_value(context)
            },
             _ => {
                let vert = self.vert_position.unwrap_or(LengthOrPercentage::Percentage(Percentage(0.0)));
                vert.to_computed_value(context)
            },
        };

        computed_position::Position {
            horizontal: horizontal,
            vertical: vertical,
        }
    }

    #[inline]
    fn from_computed_value(computed: &computed_position::Position) -> Position {
        Position {
            horiz_keyword: None,
            horiz_position: Some(ToComputedValue::from_computed_value(&computed.horizontal)),
            vert_keyword: None,
            vert_position: Some(ToComputedValue::from_computed_value(&computed.vertical)),
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
    #[inline]
    pub fn to_length_or_percentage(self) -> LengthOrPercentage {
        match self {
            PositionComponent::Length(value) => value,
            PositionComponent::Keyword(keyword) => keyword.to_length_or_percentage(),
        }
    }
}

impl Parse for PositionComponent {
    fn parse(input: &mut Parser) -> Result<Self, ()> {
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
}
