/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS handling for the specified value of
//! [`position`][position]s
//!
//! [position]: https://drafts.csswg.org/css-backgrounds-3/#position

use app_units::Au;
use cssparser::Parser;
use parser::{Parse, ParserContext};
use properties::longhands::parse_origin;
use std::mem;
use values::Either;
use values::computed::{CalcLengthOrPercentage, Context};
use values::computed::{LengthOrPercentage as ComputedLengthOrPercentage, ToComputedValue};
use values::computed::position as computed_position;
use values::generics::position::{Position as GenericPosition, PositionValue, PositionWithKeyword};
use values::generics::position::HorizontalPosition as GenericHorizontalPosition;
use values::generics::position::VerticalPosition as GenericVerticalPosition;
use values::specified::{LengthOrPercentage, Percentage};

pub use values::generics::position::Keyword;

/// The specified value of a CSS `<position>`
pub type Position = PositionWithKeyword<PositionValue<LengthOrPercentage>>;

/// The specified value for `<position>` values without a keyword.
pub type OriginPosition = GenericPosition<LengthOrPercentage, LengthOrPercentage>;

impl Parse for OriginPosition {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        let result = parse_origin(context, input)?;
        match result.depth {
            Some(_) => Err(()),
            None => Ok(GenericPosition {
                horizontal: result.horizontal.unwrap_or(LengthOrPercentage::Percentage(Percentage(0.5))),
                vertical: result.vertical.unwrap_or(LengthOrPercentage::Percentage(Percentage(0.5))),
            })
        }
    }
}

type PositionComponent = Either<LengthOrPercentage, Keyword>;

impl Position {
    /// Create a new position value from either a length or a keyword.
    pub fn from_components(mut first_position: Option<PositionComponent>,
                           mut second_position: Option<PositionComponent>,
                           first_keyword: Option<PositionComponent>,
                           second_keyword: Option<PositionComponent>) -> Result<Position, ()> {
        // Unwrap for checking if values are at right place.
        let first_key = first_keyword.clone().unwrap_or(Either::Second(Keyword::Left));
        let second_key = second_keyword.clone().unwrap_or(Either::Second(Keyword::Top));

        let (horiz_keyword, vert_keyword) = match (&first_key, &second_key) {
            // Check if a position is specified after center keyword.
            (&Either::Second(Keyword::Center), _) if first_position.is_some() => return Err(()),
            (_, &Either::Second(Keyword::Center)) if second_position.is_some() => return Err(()),

            // Check first and second keywords for both 2 and 4 value positions.

            // FIXME(canaltinova): Allow logical keywords for Position. They are not in current spec yet.
            (&Either::Second(k), _) if k.is_logical() => return Err(()),
            (_, &Either::Second(k)) if k.is_logical() => return Err(()),

             // Don't allow two vertical keywords or two horizontal keywords.
            (&Either::Second(k1), &Either::Second(k2))
                if (k1.is_horizontal() && k2.is_horizontal()) || (k1.is_vertical() && k2.is_vertical()) =>
                    return Err(()),

            // Also don't allow <length-percentage> values in the wrong position
            (&Either::First(_), &Either::Second(k)) if k.is_horizontal() => return Err(()),
            (&Either::Second(k), &Either::First(_)) if k.is_vertical() => return Err(()),

            // Swap if both are keywords and vertical precedes horizontal.
            (&Either::Second(k1), &Either::Second(k2))
                if (k1.is_vertical() && k2.is_horizontal()) || (k1.is_vertical() && k2 == Keyword::Center) ||
                   (k1 == Keyword::Center && k2.is_horizontal()) => {
                mem::swap(&mut first_position, &mut second_position);
                (second_keyword, first_keyword)
            },

            // By default, horizontal is first.
            _ => (first_keyword, second_keyword),
        };

        let (mut h_pos, mut h_key, mut v_pos, mut v_key) = (None, None, None, None);
        if let Some(Either::First(l)) = first_position {
            h_pos = Some(l);
        }

        if let Some(Either::First(l)) = second_position {
            v_pos = Some(l);
        }

        if let Some(Either::Second(k)) = horiz_keyword {
            h_key = Some(k);
        }

        if let Some(Either::Second(k)) = vert_keyword {
            v_key = Some(k);
        }

        Ok(Position {
            horizontal: GenericHorizontalPosition(PositionValue {
                keyword: h_key,
                position: h_pos,
            }),
            vertical: GenericVerticalPosition(PositionValue {
                keyword: v_key,
                position: v_pos,
            }),
        })
    }

    /// Returns a "centered" position, as in "center center".
    pub fn center() -> Position {
        Position {
            horizontal: GenericHorizontalPosition(PositionValue {
                keyword: Some(Keyword::Center),
                position: None,
            }),
            vertical: GenericVerticalPosition(PositionValue {
                keyword: Some(Keyword::Center),
                position: None,
            }),
        }
    }
}

impl Parse for Position {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        let first = input.try(|i| PositionComponent::parse(context, i))?;
        let second = input.try(|i| PositionComponent::parse(context, i))
                          .unwrap_or(Either::Second(Keyword::Center));

        if let Ok(third) = input.try(|i| PositionComponent::parse(context, i)) {
            // There's a 3rd value.
            if let Ok(fourth) = input.try(|i| PositionComponent::parse(context, i)) {
                // There's a 4th value.
                Position::from_components(Some(second), Some(fourth), Some(first), Some(third))
            } else {
                // For 3 value background position, there are several options.
                if let Either::First(_) = first {
                    return Err(())      // <length-percentage> must be preceded by <keyword>
                }

                // only 3 values.
                match (&second, &third) {
                    (&Either::First(_), &Either::First(_)) => Err(()),
                    // "keyword length keyword"
                    (&Either::First(_), _) => Position::from_components(Some(second), None,
                                                                        Some(first), Some(third)),
                    // "keyword keyword length"
                    _ => Position::from_components(None, Some(third), Some(first), Some(second)),
                }
            }
        } else {
            // only 2 values.
            match (&first, &second) {
                (&Either::First(_), &Either::First(_)) =>
                    Position::from_components(Some(first), Some(second), None, None),
                (&Either::First(_), &Either::Second(_)) =>
                    Position::from_components(Some(first), None, None, Some(second)),
                (&Either::Second(_), &Either::First(_)) =>
                    Position::from_components(None, Some(second), Some(first), None),
                (&Either::Second(_), &Either::Second(_)) =>
                    Position::from_components(None, None, Some(first), Some(second)),
            }
        }
    }
}

impl PositionValue<LengthOrPercentage> {
    /// Generic function for the computed value of a position.
    fn computed_value(&self, context: &Context) -> ComputedLengthOrPercentage {
        match self.keyword {
            Some(Keyword::Center) => ComputedLengthOrPercentage::Percentage(0.5),
            Some(k) if k.is_other_side() => match self.position {
                Some(ref x) => {
                    let (length, percentage) = match *x {
                        LengthOrPercentage::Percentage(Percentage(y)) => (Au(0), Some(1.0 - y)),
                        LengthOrPercentage::Length(ref y) => (-y.to_computed_value(context), Some(1.0)),
                        _ => (Au(0), None),
                    };

                    ComputedLengthOrPercentage::Calc(CalcLengthOrPercentage {
                        length: length,
                        percentage: percentage
                    })
                },
                None => ComputedLengthOrPercentage::Percentage(1.0),
            },
            _ => self.position.as_ref().map(|l| l.to_computed_value(context))
                              .unwrap_or(ComputedLengthOrPercentage::Percentage(0.0)),
        }
    }
}

/// The specified value of horizontal `<position>`
pub type HorizontalPosition = GenericHorizontalPosition<PositionValue<LengthOrPercentage>>;

impl ToComputedValue for HorizontalPosition {
    type ComputedValue = computed_position::HorizontalPosition;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> computed_position::HorizontalPosition {
        GenericHorizontalPosition(self.0.computed_value(context))
    }

    #[inline]
    fn from_computed_value(computed: &computed_position::HorizontalPosition) -> HorizontalPosition {
        GenericHorizontalPosition(PositionValue {
            keyword: None,
            position: Some(ToComputedValue::from_computed_value(&computed.0)),
        })
    }
}

impl HorizontalPosition {
    #[inline]
    /// Initial specified value for vertical position (`top` keyword).
    pub fn left() -> HorizontalPosition {
        GenericHorizontalPosition(PositionValue {
            keyword: Some(Keyword::Left),
            position: None,
        })
    }
}


/// The specified value of vertical `<position>`
pub type VerticalPosition = GenericVerticalPosition<PositionValue<LengthOrPercentage>>;

impl ToComputedValue for VerticalPosition {
    type ComputedValue = computed_position::VerticalPosition;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> computed_position::VerticalPosition {
        GenericVerticalPosition(self.0.computed_value(context))
    }

    #[inline]
    fn from_computed_value(computed: &computed_position::VerticalPosition) -> VerticalPosition {
        GenericVerticalPosition(PositionValue {
            keyword: None,
            position: Some(ToComputedValue::from_computed_value(&computed.0)),
        })
    }
}

impl VerticalPosition {
    #[inline]
    /// Initial specified value for vertical position (`top` keyword).
    pub fn top() -> VerticalPosition {
        GenericVerticalPosition(PositionValue {
            keyword: Some(Keyword::Top),
            position: None,
        })
    }
}
