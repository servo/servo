/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS handling for the specified value of
//! [`position`][position]s
//!
//! [position]: https://drafts.csswg.org/css-backgrounds-3/#position

use cssparser::Parser;
use parser::{Parse, ParserContext};
use std::fmt;
use style_traits::{HasViewportPercentage, ToCss, ParseError};
use values::computed::{CalcLengthOrPercentage, LengthOrPercentage as ComputedLengthOrPercentage};
use values::computed::{Context, ToComputedValue};
use values::generics::position::Position as GenericPosition;
use values::specified::{AllowQuirks, LengthOrPercentage, Percentage};
use values::specified::transform::OriginComponent;

/// The specified value of a CSS `<position>`
pub type Position = GenericPosition<HorizontalPosition, VerticalPosition>;

/// The specified value of a horizontal position.
pub type HorizontalPosition = PositionComponent<X>;

/// The specified value of a vertical position.
pub type VerticalPosition = PositionComponent<Y>;

/// The specified value of a component of a CSS `<position>`.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Debug, PartialEq, ToCss)]
pub enum PositionComponent<S> {
    /// `center`
    Center,
    /// `<lop>`
    Length(LengthOrPercentage),
    /// `<side> <lop>?`
    Side(S, Option<LengthOrPercentage>),
}

define_css_keyword_enum! { X:
    "left" => Left,
    "right" => Right,
}
add_impls_for_keyword_enum!(X);

define_css_keyword_enum! { Y:
    "top" => Top,
    "bottom" => Bottom,
}
add_impls_for_keyword_enum!(Y);

impl Parse for Position {
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        Self::parse_quirky(context, input, AllowQuirks::No)
    }
}

impl Position {
    /// Parses a `<position>`, with quirks.
    pub fn parse_quirky<'i, 't>(context: &ParserContext,
                                input: &mut Parser<'i, 't>,
                                allow_quirks: AllowQuirks)
                                -> Result<Self, ParseError<'i>> {
        match input.try(|i| PositionComponent::parse_quirky(context, i, allow_quirks)) {
            Ok(x_pos @ PositionComponent::Center) => {
                if let Ok(y_pos) = input.try(|i|
                    PositionComponent::parse_quirky(context, i, allow_quirks)) {
                    return Ok(Self::new(x_pos, y_pos));
                }
                let x_pos = input
                    .try(|i| PositionComponent::parse_quirky(context, i, allow_quirks))
                    .unwrap_or(x_pos);
                let y_pos = PositionComponent::Center;
                return Ok(Self::new(x_pos, y_pos));
            },
            Ok(PositionComponent::Side(x_keyword, lop)) => {
                if input.try(|i| i.expect_ident_matching("center")).is_ok() {
                    let x_pos = PositionComponent::Side(x_keyword, lop);
                    let y_pos = PositionComponent::Center;
                    return Ok(Self::new(x_pos, y_pos));
                }
                if let Ok(y_keyword) = input.try(Y::parse) {
                    let y_lop = input.try(|i| LengthOrPercentage::parse_quirky(context, i, allow_quirks)).ok();
                    let x_pos = PositionComponent::Side(x_keyword, lop);
                    let y_pos = PositionComponent::Side(y_keyword, y_lop);
                    return Ok(Self::new(x_pos, y_pos));
                }
                let x_pos = PositionComponent::Side(x_keyword, None);
                let y_pos = lop.map_or(PositionComponent::Center, PositionComponent::Length);
                return Ok(Self::new(x_pos, y_pos));
            },
            Ok(x_pos @ PositionComponent::Length(_)) => {
                if let Ok(y_keyword) = input.try(Y::parse) {
                    let y_pos = PositionComponent::Side(y_keyword, None);
                    return Ok(Self::new(x_pos, y_pos));
                }
                if let Ok(y_lop) = input.try(|i| LengthOrPercentage::parse_quirky(context, i, allow_quirks)) {
                    let y_pos = PositionComponent::Length(y_lop);
                    return Ok(Self::new(x_pos, y_pos));
                }
                let y_pos = PositionComponent::Center;
                let _ = input.try(|i| i.expect_ident_matching("center"));
                return Ok(Self::new(x_pos, y_pos));
            },
            Err(_) => {},
        }
        let y_keyword = Y::parse(input)?;
        let lop_and_x_pos: Result<_, ParseError> = input.try(|i| {
            let y_lop = i.try(|i| LengthOrPercentage::parse_quirky(context, i, allow_quirks)).ok();
            if let Ok(x_keyword) = i.try(X::parse) {
                let x_lop = i.try(|i| LengthOrPercentage::parse_quirky(context, i, allow_quirks)).ok();
                let x_pos = PositionComponent::Side(x_keyword, x_lop);
                return Ok((y_lop, x_pos));
            };
            i.expect_ident_matching("center")?;
            let x_pos = PositionComponent::Center;
            Ok((y_lop, x_pos))
        });
        if let Ok((y_lop, x_pos)) = lop_and_x_pos {
            let y_pos = PositionComponent::Side(y_keyword, y_lop);
            return Ok(Self::new(x_pos, y_pos));
        }
        let x_pos = PositionComponent::Center;
        let y_pos = PositionComponent::Side(y_keyword, None);
        Ok(Self::new(x_pos, y_pos))
    }

    /// `center center`
    #[inline]
    pub fn center() -> Self {
        Self::new(PositionComponent::Center, PositionComponent::Center)
    }
}

impl ToCss for Position {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match (&self.horizontal, &self.vertical) {
            (x_pos @ &PositionComponent::Side(_, Some(_)), &PositionComponent::Length(ref y_lop)) => {
                x_pos.to_css(dest)?;
                dest.write_str(" top ")?;
                y_lop.to_css(dest)
            },
            (&PositionComponent::Length(ref x_lop), y_pos @ &PositionComponent::Side(_, Some(_))) => {
                dest.write_str("left ")?;
                x_lop.to_css(dest)?;
                dest.write_str(" ")?;
                y_pos.to_css(dest)
            },
            (x_pos, y_pos) => {
                x_pos.to_css(dest)?;
                dest.write_str(" ")?;
                y_pos.to_css(dest)
            },
        }
    }
}

impl<S> HasViewportPercentage for PositionComponent<S> {
    fn has_viewport_percentage(&self) -> bool {
        match *self {
            PositionComponent::Length(ref lop) |
            PositionComponent::Side(_, Some(ref lop)) => {
                lop.has_viewport_percentage()
            },
            _ => false,
        }
    }
}

impl<S: Parse> Parse for PositionComponent<S> {
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        Self::parse_quirky(context, input, AllowQuirks::No)
    }
}

impl<S: Parse> PositionComponent<S> {
    /// Parses a component of a CSS position, with quirks.
    pub fn parse_quirky<'i, 't>(context: &ParserContext,
                                input: &mut Parser<'i, 't>,
                                allow_quirks: AllowQuirks)
                                -> Result<Self, ParseError<'i>> {
        if input.try(|i| i.expect_ident_matching("center")).is_ok() {
            return Ok(PositionComponent::Center);
        }
        if let Ok(lop) = input.try(|i| LengthOrPercentage::parse_quirky(context, i, allow_quirks)) {
            return Ok(PositionComponent::Length(lop));
        }
        let keyword = S::parse(context, input)?;
        let lop = input.try(|i| LengthOrPercentage::parse_quirky(context, i, allow_quirks)).ok();
        Ok(PositionComponent::Side(keyword, lop))
    }
}

impl<S> PositionComponent<S> {
    /// `0%`
    pub fn zero() -> Self {
        PositionComponent::Length(LengthOrPercentage::Percentage(Percentage(0.)))
    }
}

impl<S: Side> ToComputedValue for PositionComponent<S> {
    type ComputedValue = ComputedLengthOrPercentage;

    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        match *self {
            PositionComponent::Center => {
                ComputedLengthOrPercentage::Percentage(Percentage(0.5))
            },
            PositionComponent::Side(ref keyword, None) => {
                let p = Percentage(if keyword.is_start() { 0. } else { 1. });
                ComputedLengthOrPercentage::Percentage(p)
            },
            PositionComponent::Side(ref keyword, Some(ref length)) if !keyword.is_start() => {
                match length.to_computed_value(context) {
                    ComputedLengthOrPercentage::Length(length) => {
                        ComputedLengthOrPercentage::Calc(CalcLengthOrPercentage::new(-length, Some(Percentage(1.0))))
                    },
                    ComputedLengthOrPercentage::Percentage(p) => {
                        ComputedLengthOrPercentage::Percentage(Percentage(1.0 - p.0))
                    },
                    ComputedLengthOrPercentage::Calc(calc) => {
                        let p = Percentage(1. - calc.percentage.map_or(0., |p| p.0));
                        ComputedLengthOrPercentage::Calc(CalcLengthOrPercentage::new(-calc.unclamped_length(), Some(p)))
                    },
                }
            },
            PositionComponent::Side(_, Some(ref length)) |
            PositionComponent::Length(ref length) => {
                length.to_computed_value(context)
            },
        }
    }

    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        PositionComponent::Length(ToComputedValue::from_computed_value(computed))
    }
}

impl<S: Side> PositionComponent<S> {
    /// The initial specified value of a position component, i.e. the start side.
    pub fn initial_specified_value() -> Self {
        PositionComponent::Side(S::start(), None)
    }
}

/// Represents a side, either horizontal or vertical, of a CSS position.
pub trait Side {
    /// Returns the start side.
    fn start() -> Self;

    /// Returns whether this side is the start side.
    fn is_start(&self) -> bool;
}

impl Side for X {
    #[inline]
    fn start() -> Self {
        X::Left
    }

    #[inline]
    fn is_start(&self) -> bool {
        *self == X::Left
    }
}

impl Side for Y {
    #[inline]
    fn start() -> Self {
        Y::Top
    }

    #[inline]
    fn is_start(&self) -> bool {
        *self == Y::Top
    }
}

/// The specified value of a legacy CSS `<position>`
/// Modern position syntax supports 3 and 4-value syntax. That means:
/// If three or four values are given, then each <percentage> or <length> represents an offset
/// and must be preceded by a keyword, which specifies from which edge the offset is given.
/// For example, `bottom 10px right 20px` represents a `10px` vertical
/// offset up from the bottom edge and a `20px` horizontal offset leftward from the right edge.
/// If three values are given, the missing offset is assumed to be zero.
/// But for some historical reasons we need to keep CSS Level 2 syntax which only supports up to
/// 2-value. This type represents this 2-value syntax.
pub type LegacyPosition = GenericPosition<LegacyHPosition, LegacyVPosition>;

/// The specified value of a horizontal position.
pub type LegacyHPosition = OriginComponent<X>;

/// The specified value of a vertical position.
pub type LegacyVPosition = OriginComponent<Y>;

impl Parse for LegacyPosition {
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        Self::parse_quirky(context, input, AllowQuirks::No)
    }
}

impl LegacyPosition {
    /// Parses a `<position>`, with quirks.
    pub fn parse_quirky<'i, 't>(context: &ParserContext,
                                input: &mut Parser<'i, 't>,
                                allow_quirks: AllowQuirks)
                                -> Result<Self, ParseError<'i>> {
        match input.try(|i| OriginComponent::parse(context, i)) {
            Ok(x_pos @ OriginComponent::Center) => {
                if let Ok(y_pos) = input.try(|i|
                    OriginComponent::parse(context, i)) {
                    return Ok(Self::new(x_pos, y_pos));
                }
                let x_pos = input
                    .try(|i| OriginComponent::parse(context, i))
                    .unwrap_or(x_pos);
                let y_pos = OriginComponent::Center;
                return Ok(Self::new(x_pos, y_pos));
            },
            Ok(OriginComponent::Side(x_keyword)) => {
                if input.try(|i| i.expect_ident_matching("center")).is_ok() {
                    let x_pos = OriginComponent::Side(x_keyword);
                    let y_pos = OriginComponent::Center;
                    return Ok(Self::new(x_pos, y_pos));
                }
                if let Ok(y_keyword) = input.try(Y::parse) {
                    let x_pos = OriginComponent::Side(x_keyword);
                    let y_pos = OriginComponent::Side(y_keyword);
                    return Ok(Self::new(x_pos, y_pos));
                }
                let x_pos = OriginComponent::Side(x_keyword);
                if let Ok(y_lop) = input.try(|i| LengthOrPercentage::parse_quirky(context, i, allow_quirks)) {
                    return Ok(Self::new(x_pos, OriginComponent::Length(y_lop)))
                }
            },
            Ok(x_pos @ OriginComponent::Length(_)) => {
                if let Ok(y_keyword) = input.try(Y::parse) {
                    let y_pos = OriginComponent::Side(y_keyword);
                    return Ok(Self::new(x_pos, y_pos));
                }
                if let Ok(y_lop) = input.try(|i| LengthOrPercentage::parse_quirky(context, i, allow_quirks)) {
                    let y_pos = OriginComponent::Length(y_lop);
                    return Ok(Self::new(x_pos, y_pos));
                }
                let y_pos = OriginComponent::Center;
                let _ = input.try(|i| i.expect_ident_matching("center"));
                return Ok(Self::new(x_pos, y_pos));
            },
            Err(_) => {},
        }
        let y_keyword = Y::parse(input)?;
        let x_pos: Result<_, ParseError> = input.try(|i| {
            if let Ok(x_keyword) = i.try(X::parse) {
                let x_pos = OriginComponent::Side(x_keyword);
                return Ok(x_pos);
            }
            i.expect_ident_matching("center")?;
            Ok(OriginComponent::Center)
        });
        if let Ok(x_pos) = x_pos {
            let y_pos = OriginComponent::Side(y_keyword);
            return Ok(Self::new(x_pos, y_pos));
        }
        let x_pos = OriginComponent::Center;
        let y_pos = OriginComponent::Side(y_keyword);
        Ok(Self::new(x_pos, y_pos))
    }

    /// `center center`
    #[inline]
    pub fn center() -> Self {
        Self::new(OriginComponent::Center, OriginComponent::Center)
    }
}

impl ToCss for LegacyPosition {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        self.horizontal.to_css(dest)?;
        dest.write_str(" ")?;
        self.vertical.to_css(dest)
    }
}
