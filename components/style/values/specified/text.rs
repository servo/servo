/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Specified types for text properties.

use cssparser::Parser;
use parser::{Parse, ParserContext};
use std::ascii::AsciiExt;
use values::computed::{Context, ToComputedValue};
use values::computed::text::LineHeight as ComputedLineHeight;
use values::generics::text::LineHeight as GenericLineHeight;
use values::specified::Number;
use values::specified::length::{FontRelativeLength, Length, LengthOrPercentage, NoCalcLength};

/// A specified value for the `line-height` property.
pub type LineHeight = GenericLineHeight<Number, LengthOrPercentage>;

impl Parse for LineHeight {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        if let Ok(number) = input.try(|i| Number::parse_non_negative(context, i)) {
            return Ok(GenericLineHeight::Number(number))
        }
        if let Ok(lop) = input.try(|i| LengthOrPercentage::parse_non_negative(context, i)) {
            return Ok(GenericLineHeight::Length(lop))
        }
        match &input.expect_ident()? {
            ident if ident.eq_ignore_ascii_case("normal") => {
                Ok(GenericLineHeight::Normal)
            },
            #[cfg(feature = "gecko")]
            ident if ident.eq_ignore_ascii_case("-moz-block-height") => {
                Ok(GenericLineHeight::MozBlockHeight)
            },
            _ => Err(()),
        }
    }
}

impl ToComputedValue for LineHeight {
    type ComputedValue = ComputedLineHeight;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
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
            GenericLineHeight::Length(LengthOrPercentage::Length(ref length)) => {
                GenericLineHeight::Length(length.to_computed_value(context))
            },
            GenericLineHeight::Length(LengthOrPercentage::Percentage(p)) => {
                let font_relative_length =
                    Length::NoCalc(NoCalcLength::FontRelative(FontRelativeLength::Em(p.0)));
                GenericLineHeight::Length(font_relative_length.to_computed_value(context))
            },
            GenericLineHeight::Length(LengthOrPercentage::Calc(ref calc)) => {
                let computed_calc = calc.to_computed_value(context);
                let font_relative_length =
                    Length::NoCalc(NoCalcLength::FontRelative(FontRelativeLength::Em(computed_calc.percentage())));
                let absolute_length = computed_calc.unclamped_length();
                let computed_length = computed_calc.clamping_mode.clamp(
                    absolute_length + font_relative_length.to_computed_value(context)
                );
                GenericLineHeight::Length(computed_length)
            },
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
                GenericLineHeight::Number(Number::from_computed_value(number))
            },
            GenericLineHeight::Length(ref length) => {
                GenericLineHeight::Length(LengthOrPercentage::Length(
                    NoCalcLength::from_computed_value(length)
                ))
            }
        }
    }
}
