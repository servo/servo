/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS handling for the specified value of
//! [`position`][position]s
//!
//! [position]: https://drafts.csswg.org/css-backgrounds-3/#position

use app_units::Au;
use cssparser::{Parser, Token};
use parser::{Parse, ParserContext};
use std::fmt;
use style_traits::ToCss;
use values::HasViewportPercentage;
use values::computed::{CalcLengthOrPercentage, Context};
use values::computed::{LengthOrPercentage as ComputedLengthOrPercentage, ToComputedValue};
use values::computed::position as computed_position;
use values::specified::{LengthOrPercentage, Percentage};

#[derive(Debug, Clone, PartialEq, Copy)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
/// A [position][pos].
///
/// [pos]: https://drafts.csswg.org/css-values/#position
pub struct Position {
    /// The horizontal component.
    pub horizontal: HorizontalPosition,
    /// The vertical component.
    pub vertical: VerticalPosition,
}

impl ToCss for Position {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        let mut space_present = false;
        if let Some(horiz_key) = self.horizontal.keyword {
            try!(horiz_key.to_css(dest));
            try!(dest.write_str(" "));
            space_present = true;
        };
        if let Some(horiz_pos) = self.horizontal.position {
            try!(horiz_pos.to_css(dest));
            try!(dest.write_str(" "));
            space_present = true;
        };
        if let Some(vert_key) = self.vertical.keyword {
            try!(vert_key.to_css(dest));
            space_present = false;
        };
        if let Some(vert_pos) = self.vertical.position {
            if space_present == false {
                try!(dest.write_str(" "));
            }
            try!(vert_pos.to_css(dest));
        };
        Ok(())
    }
}

impl HasViewportPercentage for Position {
    fn has_viewport_percentage(&self) -> bool {
        self.horizontal.has_viewport_percentage() || self.vertical.has_viewport_percentage()
    }
}

impl Position {
    /// Create a new position value.
    pub fn new(mut first_position: Option<PositionComponent>,
               mut second_position: Option<PositionComponent>,
               first_keyword: Option<PositionComponent>,
               second_keyword: Option<PositionComponent>)
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

            // FIXME(canaltinova): Allow logical keywords for Position. They are not in current spec yet.
            (PositionCategory::HorizontalLogicalKeyword, _) |
            (PositionCategory::VerticalLogicalKeyword, _) |
            (_, PositionCategory::HorizontalLogicalKeyword) |
            (_, PositionCategory::VerticalLogicalKeyword) => return Err(()),

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
            horizontal: HorizontalPosition {
                keyword: horizontal_keyword,
                position: first_position,
            },
            vertical: VerticalPosition {
                keyword: vertical_keyword,
                position: second_position,
            },
        })
    }

    /// Returns a "centered" position, as in "center center".
    pub fn center() -> Position {
        Position {
            horizontal: HorizontalPosition {
                keyword: Some(Keyword::Center),
                position: None,
            },
            vertical: VerticalPosition {
                keyword: Some(Keyword::Center),
                position: None,
            },
        }
    }
}

impl Parse for Position {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        let first = try!(PositionComponent::parse(context, input));
        let second = input.try(|i| PositionComponent::parse(context, i))
            .unwrap_or(PositionComponent::Keyword(Keyword::Center));

        // Try to parse third and fourth values
        if let Ok(third) = input.try(|i| PositionComponent::parse(context, i)) {
            if let Ok(fourth) = input.try(|i| PositionComponent::parse(context, i)) {
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

impl ToComputedValue for Position {
    type ComputedValue = computed_position::Position;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> computed_position::Position {
        computed_position::Position {
            horizontal: self.horizontal.to_computed_value(context).0,
            vertical: self.vertical.to_computed_value(context).0,
        }
    }

    #[inline]
    fn from_computed_value(computed: &computed_position::Position) -> Position {
        Position {
            horizontal: HorizontalPosition {
                keyword: None,
                position: Some(ToComputedValue::from_computed_value(&computed.horizontal)),
            },
            vertical: VerticalPosition {
                keyword: None,
                position: Some(ToComputedValue::from_computed_value(&computed.vertical)),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub struct HorizontalPosition {
    pub keyword: Option<Keyword>,
    pub position: Option<LengthOrPercentage>,
}

impl HasViewportPercentage for HorizontalPosition {
    fn has_viewport_percentage(&self) -> bool {
        self.position.map_or(false, |pos| pos.has_viewport_percentage())
    }
}

impl ToCss for HorizontalPosition {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        let mut keyword_present = false;
        if let Some(keyword) = self.keyword {
            try!(keyword.to_css(dest));
            keyword_present = true;
        };

        if let Some(position) = self.position {
            if keyword_present {
                try!(dest.write_str(" "));
            }
            try!(position.to_css(dest));
        };
        Ok(())
    }
}

impl Parse for HorizontalPosition {
    #[inline]
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        let first = try!(PositionComponent::parse(context, input));
        let second = input.try(|i| PositionComponent::parse(context, i)).ok();

        let (keyword, position) = if let PositionCategory::LengthOrPercentage = category(first) {
            // "length keyword?"
            (second, Some(first))
        } else {
            // "keyword length?"
            (Some(first), second)
        };

        // Unwrapping and checking keyword.
        let keyword = match keyword {
            Some(PositionComponent::Keyword(key)) => {
                match category(keyword.unwrap()) {
                    PositionCategory::VerticalKeyword |
                    PositionCategory::VerticalLogicalKeyword => return Err(()),
                    _ => Some(key),
                }
            },
            Some(_) => return Err(()),
            None => None,
        };

        // Unwrapping and checking position.
        let position = match position {
            Some(PositionComponent::Length(pos)) => {
                // "center <length>" is not allowed
                if let Some(Keyword::Center) = keyword {
                    return Err(());
                }
                Some(pos)
            },
            Some(_) => return Err(()),
            None => None,
        };

        Ok(HorizontalPosition {
            keyword: keyword,
            position: position,
        })
    }
}

impl ToComputedValue for HorizontalPosition {
    type ComputedValue = computed_position::HorizontalPosition;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> computed_position::HorizontalPosition {
        let keyword = self.keyword.unwrap_or(Keyword::Left);

        // Construct horizontal computed LengthOrPercentage
        let horizontal = match keyword {
            // FIXME(canaltinova): Support logical keywords.
            Keyword::Right | Keyword::XEnd => {
                if let Some(x) = self.position {
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
                keyword.to_length_or_percentage().to_computed_value(context)
            },
             _ => {
                let horiz = self.position
                                .unwrap_or(LengthOrPercentage::Percentage(Percentage(0.0)));
                horiz.to_computed_value(context)
            },
        };

        computed_position::HorizontalPosition(horizontal)
    }

    #[inline]
    fn from_computed_value(computed: &computed_position::HorizontalPosition) -> HorizontalPosition {
        HorizontalPosition {
            keyword: None,
            position: Some(ToComputedValue::from_computed_value(&computed.0)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub struct VerticalPosition {
    pub keyword: Option<Keyword>,
    pub position: Option<LengthOrPercentage>,
}

impl HasViewportPercentage for VerticalPosition {
    fn has_viewport_percentage(&self) -> bool {
        self.position.map_or(false, |pos| pos.has_viewport_percentage())
    }
}

impl ToCss for VerticalPosition {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        let mut keyword_present = false;
        if let Some(keyword) = self.keyword {
            try!(keyword.to_css(dest));
            keyword_present = true;
        };

        if let Some(position) = self.position {
            if keyword_present {
                try!(dest.write_str(" "));
            }
            try!(position.to_css(dest));
        };
        Ok(())
    }
}

impl Parse for VerticalPosition {
    #[inline]
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        let first = try!(PositionComponent::parse(context, input));
        let second = input.try(|i| PositionComponent::parse(context, i)).ok();

        let (keyword, position) = if let PositionCategory::LengthOrPercentage = category(first) {
            // "length keyword?"
            (second, Some(first))
        } else {
            // "keyword length?"
            (Some(first), second)
        };

        // Unwrapping and checking keyword.
        let keyword = match keyword {
            Some(PositionComponent::Keyword(key)) => {
                match category(keyword.unwrap()) {
                    PositionCategory::HorizontalKeyword |
                    PositionCategory::HorizontalLogicalKeyword => return Err(()),
                    _ => Some(key),
                }
            },
            Some(_) => return Err(()),
            None => None,
        };

        // Unwrapping and checking position.
        let position = match position {
            Some(PositionComponent::Length(pos)) => {
                // "center <length>" is not allowed
                if let Some(Keyword::Center) = keyword {
                    return Err(());
                }
                Some(pos)
            },
            Some(_) => return Err(()),
            None => None,
        };

        Ok(VerticalPosition {
            keyword: keyword,
            position: position,
        })
    }
}

impl ToComputedValue for VerticalPosition {
    type ComputedValue = computed_position::VerticalPosition;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> computed_position::VerticalPosition {
        let keyword = self.keyword.unwrap_or(Keyword::Left);

        // Construct vertical computed LengthOrPercentage
        let vertical = match keyword {
            // FIXME(canaltinova): Support logical keywords.
            Keyword::Bottom | Keyword::YEnd => {
                if let Some(x) = self.position {
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
                keyword.to_length_or_percentage().to_computed_value(context)
            },
             _ => {
                let vert = self.position
                               .unwrap_or(LengthOrPercentage::Percentage(Percentage(0.0)));
                vert.to_computed_value(context)
            },
        };

        computed_position::VerticalPosition(vertical)
    }

    #[inline]
    fn from_computed_value(computed: &computed_position::VerticalPosition) -> VerticalPosition {
        VerticalPosition {
            keyword: None,
            position: Some(ToComputedValue::from_computed_value(&computed.0)),
        }
    }
}

define_css_keyword_enum!(Keyword:
                         "center" => Center,
                         "left" => Left,
                         "right" => Right,
                         "top" => Top,
                         "bottom" => Bottom,
                         "x-start" => XStart,
                         "x-end" => XEnd,
                         "y-start" => YStart,
                         "y-end" => YEnd);

impl Keyword {
    /// Convert the given keyword to a length or a percentage.
    pub fn to_length_or_percentage(self) -> LengthOrPercentage {
        match self {
            Keyword::Center => LengthOrPercentage::Percentage(Percentage(0.5)),
            Keyword::Left | Keyword::Top => LengthOrPercentage::Percentage(Percentage(0.0)),
            Keyword::Right | Keyword::Bottom => LengthOrPercentage::Percentage(Percentage(1.0)),
            // FIXME(canaltinova): Support logical keywords
            Keyword::XStart | Keyword::YStart => LengthOrPercentage::Percentage(Percentage(0.0)),
            Keyword::XEnd | Keyword::YEnd => LengthOrPercentage::Percentage(Percentage(1.0)),
        }
    }
}

// Collapse `Position` into a few categories to simplify the above `match` expression.
enum PositionCategory {
    HorizontalKeyword,
    VerticalKeyword,
    HorizontalLogicalKeyword,
    VerticalLogicalKeyword,
    OtherKeyword,
    LengthOrPercentage,
}

/// A position component.
///
/// http://dev.w3.org/csswg/css2/colors.html#propdef-background-position
#[derive(Clone, PartialEq, Copy)]
pub enum PositionComponent {
    /// A `<length>`
    Length(LengthOrPercentage),
    /// A position keyword.
    Keyword(Keyword),
}

fn category(p: PositionComponent) -> PositionCategory {
    if let PositionComponent::Keyword(keyword) = p {
        match keyword {
            Keyword::Left | Keyword::Right =>
                PositionCategory::HorizontalKeyword,
            Keyword::Top | Keyword::Bottom =>
                PositionCategory::VerticalKeyword,
            Keyword::XStart | Keyword::XEnd =>
                PositionCategory::HorizontalLogicalKeyword,
            Keyword::YStart | Keyword::YEnd =>
                PositionCategory::VerticalLogicalKeyword,
            Keyword::Center =>
                PositionCategory::OtherKeyword,
        }
    } else {
        PositionCategory::LengthOrPercentage
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
    /// Convert the given position component to a length or a percentage.
    #[inline]
    pub fn to_length_or_percentage(self) -> LengthOrPercentage {
        match self {
            PositionComponent::Length(value) => value,
            PositionComponent::Keyword(keyword) => keyword.to_length_or_percentage(),
        }
    }
}

impl Parse for PositionComponent {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        input.try(|i| LengthOrPercentage::parse(context, i))
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
                                                   "x-start" => Ok(PositionComponent::Keyword(Keyword::XStart)),
                                                   "x-end" => Ok(PositionComponent::Keyword(Keyword::XEnd)),
                                                   "y-start" => Ok(PositionComponent::Keyword(Keyword::YStart)),
                                                   "y-end" => Ok(PositionComponent::Keyword(Keyword::YEnd)),
                                                   _ => Err(())
                        }
                    },
                    _ => Err(())
                }
            })
    }
}
