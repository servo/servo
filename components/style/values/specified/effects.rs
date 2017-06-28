/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Specified types for CSS values related to effects.

use cssparser::{BasicParseError, Parser, Token};
use parser::{Parse, ParserContext};
use style_traits::{ParseError, StyleParseError};
#[cfg(not(feature = "gecko"))]
use values::Impossible;
use values::computed::{Context, Number as ComputedNumber, ToComputedValue};
use values::computed::effects::BoxShadow as ComputedBoxShadow;
use values::computed::effects::SimpleShadow as ComputedSimpleShadow;
use values::generics::effects::BoxShadow as GenericBoxShadow;
use values::generics::effects::Filter as GenericFilter;
use values::generics::effects::SimpleShadow as GenericSimpleShadow;
use values::specified::{Angle, Percentage};
use values::specified::color::Color;
use values::specified::length::Length;
#[cfg(feature = "gecko")]
use values::specified::url::SpecifiedUrl;

/// A specified value for a single shadow of the `box-shadow` property.
pub type BoxShadow = GenericBoxShadow<Option<Color>, Length, Option<Length>>;

/// A specified value for a single `filter`.
#[cfg(feature = "gecko")]
pub type Filter = GenericFilter<Angle, Factor, Length, SimpleShadow>;

/// A specified value for a single `filter`.
#[cfg(not(feature = "gecko"))]
pub type Filter = GenericFilter<Angle, Factor, Length, Impossible>;

/// A value for the `<factor>` parts in `Filter`.
///
/// FIXME: Should be `NumberOrPercentage`, but Gecko doesn't support that yet.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Debug, HasViewportPercentage, PartialEq, ToCss)]
pub enum Factor {
    /// Literal number.
    Number(ComputedNumber),
    /// Literal percentage.
    Percentage(Percentage),
}

/// A specified value for the `drop-shadow()` filter.
pub type SimpleShadow = GenericSimpleShadow<Option<Color>, Length, Option<Length>>;

impl Parse for BoxShadow {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let mut lengths = None;
        let mut color = None;
        let mut inset = false;

        loop {
            if !inset {
                if input.try(|input| input.expect_ident_matching("inset")).is_ok() {
                    inset = true;
                    continue;
                }
            }
            if lengths.is_none() {
                let value = input.try::<_, _, ParseError>(|i| {
                    let horizontal = Length::parse(context, i)?;
                    let vertical = Length::parse(context, i)?;
                    let (blur, spread) = match i.try::<_, _, ParseError>(|i| Length::parse_non_negative(context, i)) {
                        Ok(blur) => {
                            let spread = i.try(|i| Length::parse(context, i)).ok();
                            (Some(blur), spread)
                        },
                        Err(_) => (None, None),
                    };
                    Ok((horizontal, vertical, blur, spread))
                });
                if let Ok(value) = value {
                    lengths = Some(value);
                    continue;
                }
            }
            if color.is_none() {
                if let Ok(value) = input.try(|i| Color::parse(context, i)) {
                    color = Some(value);
                    continue;
                }
            }
            break;
        }

        let lengths = lengths.ok_or(StyleParseError::UnspecifiedError)?;
        Ok(BoxShadow {
            base: SimpleShadow {
                color: color,
                horizontal: lengths.0,
                vertical: lengths.1,
                blur: lengths.2,
            },
            spread: lengths.3,
            inset: inset,
        })
    }
}

impl ToComputedValue for BoxShadow {
    type ComputedValue = ComputedBoxShadow;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        ComputedBoxShadow {
            base: self.base.to_computed_value(context),
            spread: self.spread.as_ref().unwrap_or(&Length::zero()).to_computed_value(context),
            inset: self.inset,
        }
    }

    #[inline]
    fn from_computed_value(computed: &ComputedBoxShadow) -> Self {
        BoxShadow {
            base: ToComputedValue::from_computed_value(&computed.base),
            spread: Some(ToComputedValue::from_computed_value(&computed.spread)),
            inset: computed.inset,
        }
    }
}

impl Parse for Filter {
    #[inline]
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>
    ) -> Result<Self, ParseError<'i>> {
        #[cfg(feature = "gecko")]
        {
            if let Ok(url) = input.try(|i| SpecifiedUrl::parse(context, i)) {
                return Ok(GenericFilter::Url(url));
            }
        }
        let function = input.expect_function()?;
        input.parse_nested_block(|i| {
            try_match_ident_ignore_ascii_case! { function,
                "blur" => Ok(GenericFilter::Blur(Length::parse_non_negative(context, i)?)),
                "brightness" => Ok(GenericFilter::Brightness(Factor::parse(context, i)?)),
                "contrast" => Ok(GenericFilter::Contrast(Factor::parse(context, i)?)),
                "grayscale" => Ok(GenericFilter::Grayscale(Factor::parse(context, i)?)),
                "hue-rotate" => Ok(GenericFilter::HueRotate(Angle::parse(context, i)?)),
                "invert" => Ok(GenericFilter::Invert(Factor::parse(context, i)?)),
                "opacity" => Ok(GenericFilter::Opacity(Factor::parse(context, i)?)),
                "saturate" => Ok(GenericFilter::Saturate(Factor::parse(context, i)?)),
                "sepia" => Ok(GenericFilter::Sepia(Factor::parse(context, i)?)),
                "drop-shadow" => Ok(GenericFilter::DropShadow(Parse::parse(context, i)?)),
            }
        })
    }
}

impl Parse for Factor {
    #[inline]
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>
    ) -> Result<Self, ParseError<'i>> {
        match input.next()? {
            Token::Number { value, .. } if value.is_sign_positive() => {
                Ok(Factor::Number(value))
            },
            Token::Percentage { unit_value, .. } if unit_value.is_sign_positive() => {
                Ok(Factor::Percentage(Percentage(unit_value)))
            },
            other => Err(BasicParseError::UnexpectedToken(other).into()),
        }
    }
}

impl ToComputedValue for Factor {
    /// This should actually be `ComputedNumberOrPercentage`, but layout uses
    /// `computed::effects::Filter` directly in `StackingContext`.
    type ComputedValue = ComputedNumber;

    #[inline]
    fn to_computed_value(&self, _context: &Context) -> Self::ComputedValue {
        match *self {
            Factor::Number(number) => number,
            Factor::Percentage(percentage) => percentage.0,
        }
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        Factor::Number(*computed)
    }
}

impl Parse for SimpleShadow {
    #[inline]
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>
    ) -> Result<Self, ParseError<'i>> {
        let color = input.try(|i| Color::parse(context, i)).ok();
        let horizontal = Length::parse(context, input)?;
        let vertical = Length::parse(context, input)?;
        let blur = input.try(|i| Length::parse_non_negative(context, i)).ok();
        let color = color.or_else(|| input.try(|i| Color::parse(context, i)).ok());
        Ok(SimpleShadow {
            color: color,
            horizontal: horizontal,
            vertical: vertical,
            blur: blur,
        })
    }
}

impl ToComputedValue for SimpleShadow {
    type ComputedValue = ComputedSimpleShadow;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        ComputedSimpleShadow {
            color:
                self.color.as_ref().unwrap_or(&Color::CurrentColor).to_computed_value(context),
            horizontal: self.horizontal.to_computed_value(context),
            vertical: self.vertical.to_computed_value(context),
            blur:
                self.blur.as_ref().unwrap_or(&Length::zero()).to_computed_value(context),
        }
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        SimpleShadow {
            color: Some(ToComputedValue::from_computed_value(&computed.color)),
            horizontal: ToComputedValue::from_computed_value(&computed.horizontal),
            vertical: ToComputedValue::from_computed_value(&computed.vertical),
            blur: Some(ToComputedValue::from_computed_value(&computed.blur)),
        }
    }
}
