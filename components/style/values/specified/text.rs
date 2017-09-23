/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Specified types for text properties.

use cssparser::Parser;
use parser::{Parse, ParserContext};
use selectors::parser::SelectorParseError;
use std::ascii::AsciiExt;
use style_traits::ParseError;
use values::computed::{Context, ToComputedValue};
use values::computed::text::LineHeight as ComputedLineHeight;
use values::generics::text::InitialLetter as GenericInitialLetter;
use values::generics::text::LineHeight as GenericLineHeight;
use values::generics::text::Spacing;
use values::specified::{AllowQuirks, Integer, NonNegativeNumber, Number};
use values::specified::length::{FontRelativeLength, Length, LengthOrPercentage, NoCalcLength};
use values::specified::length::NonNegativeLengthOrPercentage;

/// A specified type for the `initial-letter` property.
pub type InitialLetter = GenericInitialLetter<Number, Integer>;

/// A specified value for the `letter-spacing` property.
pub type LetterSpacing = Spacing<Length>;

/// A specified value for the `word-spacing` property.
pub type WordSpacing = Spacing<LengthOrPercentage>;

/// A specified value for the `line-height` property.
pub type LineHeight = GenericLineHeight<NonNegativeNumber, NonNegativeLengthOrPercentage>;

impl Parse for InitialLetter {
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        if input.try(|i| i.expect_ident_matching("normal")).is_ok() {
            return Ok(GenericInitialLetter::Normal);
        }
        let size = Number::parse_at_least_one(context, input)?;
        let sink = input.try(|i| Integer::parse_positive(context, i)).ok();
        Ok(GenericInitialLetter::Specified(size, sink))
    }
}

impl Parse for LetterSpacing {
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        Spacing::parse_with(context, input, |c, i| {
            Length::parse_quirky(c, i, AllowQuirks::Yes)
        })
    }
}

impl Parse for WordSpacing {
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        Spacing::parse_with(context, input, |c, i| {
            LengthOrPercentage::parse_quirky(c, i, AllowQuirks::Yes)
        })
    }
}

impl Parse for LineHeight {
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        if let Ok(number) = input.try(|i| NonNegativeNumber::parse(context, i)) {
            return Ok(GenericLineHeight::Number(number))
        }
        if let Ok(nlop) = input.try(|i| NonNegativeLengthOrPercentage::parse(context, i)) {
            return Ok(GenericLineHeight::Length(nlop))
        }
        let ident = input.expect_ident()?;
        match ident {
            ref ident if ident.eq_ignore_ascii_case("normal") => {
                Ok(GenericLineHeight::Normal)
            },
            #[cfg(feature = "gecko")]
            ref ident if ident.eq_ignore_ascii_case("-moz-block-height") => {
                Ok(GenericLineHeight::MozBlockHeight)
            },
            ident => Err(SelectorParseError::UnexpectedIdent(ident.clone()).into()),
        }
    }
}

impl ToComputedValue for LineHeight {
    type ComputedValue = ComputedLineHeight;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        use values::computed::Length as ComputedLength;
        use values::specified::length::FontBaseSize;
        match *self {
            GenericLineHeight::Normal => {
                GenericLineHeight::Normal
            },
            #[cfg(feature = "gecko")]
            GenericLineHeight::MozBlockHeight => {
                GenericLineHeight::MozBlockHeight
            },
            GenericLineHeight::Number(number) => {
                GenericLineHeight::Number(number.to_computed_value(context))
            },
            GenericLineHeight::Length(ref non_negative_lop) => {
                let result = match non_negative_lop.0 {
                    LengthOrPercentage::Length(NoCalcLength::Absolute(ref abs)) => {
                        context.maybe_zoom_text(abs.to_computed_value(context).into()).0
                    }
                    LengthOrPercentage::Length(ref length) => {
                        length.to_computed_value(context)
                    },
                    LengthOrPercentage::Percentage(ref p) => {
                        FontRelativeLength::Em(p.0)
                            .to_computed_value(
                                context,
                                FontBaseSize::CurrentStyle,
                            )
                    }
                    LengthOrPercentage::Calc(ref calc) => {
                        let computed_calc =
                            calc.to_computed_value_zoomed(context, FontBaseSize::CurrentStyle);
                        let font_relative_length =
                            FontRelativeLength::Em(computed_calc.percentage())
                                .to_computed_value(
                                    context,
                                    FontBaseSize::CurrentStyle,
                                ).px();

                        let absolute_length = computed_calc.unclamped_length().px();
                        let pixel = computed_calc
                            .clamping_mode
                            .clamp(absolute_length + font_relative_length);
                        ComputedLength::new(pixel)
                    }
                };
                GenericLineHeight::Length(result.into())
            }
        }
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        match *computed {
            GenericLineHeight::Normal => {
                GenericLineHeight::Normal
            },
            #[cfg(feature = "gecko")]
            GenericLineHeight::MozBlockHeight => {
                GenericLineHeight::MozBlockHeight
            },
            GenericLineHeight::Number(ref number) => {
                GenericLineHeight::Number(NonNegativeNumber::from_computed_value(number))
            },
            GenericLineHeight::Length(ref length) => {
                GenericLineHeight::Length(NoCalcLength::from_computed_value(&length.0).into())
            }
        }
    }
}
