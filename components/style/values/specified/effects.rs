/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Specified types for CSS values related to effects.

use cssparser::{self, BasicParseErrorKind, Parser, Token};
use parser::{Parse, ParserContext};
use style_traits::{ParseError, StyleParseErrorKind, ValueParseErrorKind};
#[cfg(not(feature = "gecko"))]
use values::Impossible;
use values::computed::{Context, NonNegativeNumber as ComputedNonNegativeNumber, ToComputedValue};
use values::computed::effects::BoxShadow as ComputedBoxShadow;
use values::computed::effects::SimpleShadow as ComputedSimpleShadow;
use values::generics::NonNegative;
use values::generics::effects::BoxShadow as GenericBoxShadow;
use values::generics::effects::Filter as GenericFilter;
use values::generics::effects::SimpleShadow as GenericSimpleShadow;
use values::specified::{Angle, NumberOrPercentage};
use values::specified::color::RGBAColor;
use values::specified::length::{Length, NonNegativeLength};
#[cfg(feature = "gecko")]
use values::specified::url::SpecifiedUrl;

/// A specified value for a single shadow of the `box-shadow` property.
pub type BoxShadow =
    GenericBoxShadow<Option<RGBAColor>, Length, Option<NonNegativeLength>, Option<Length>>;

/// A specified value for a single `filter`.
#[cfg(feature = "gecko")]
pub type Filter = GenericFilter<Angle, Factor, NonNegativeLength, SimpleShadow, SpecifiedUrl>;

/// A specified value for a single `filter`.
#[cfg(not(feature = "gecko"))]
pub type Filter = GenericFilter<Angle, Factor, NonNegativeLength, Impossible, Impossible>;

/// A value for the `<factor>` parts in `Filter`.
#[derive(Clone, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo, ToCss)]
pub struct Factor(NumberOrPercentage);

impl Factor {
    /// Parse this factor but clamp to one if the value is over 100%.
    #[inline]
    pub fn parse_with_clamping_to_one<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        Factor::parse(context, input).map(|v| v.clamp_to_one())
    }

    /// Clamp the value to 1 if the value is over 100%.
    #[inline]
    fn clamp_to_one(self) -> Self {
        match self.0 {
            NumberOrPercentage::Percentage(percent) => {
                Factor(NumberOrPercentage::Percentage(percent.clamp_to_hundred()))
            },
            NumberOrPercentage::Number(number) => {
                Factor(NumberOrPercentage::Number(number.clamp_to_one()))
            },
        }
    }
}

impl Parse for Factor {
    #[inline]
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        NumberOrPercentage::parse_non_negative(context, input).map(Factor)
    }
}

impl ToComputedValue for Factor {
    type ComputedValue = ComputedNonNegativeNumber;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        use values::computed::NumberOrPercentage;
        match self.0.to_computed_value(context) {
            NumberOrPercentage::Number(n) => n.into(),
            NumberOrPercentage::Percentage(p) => p.0.into(),
        }
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        Factor(NumberOrPercentage::Number(
            ToComputedValue::from_computed_value(&computed.0),
        ))
    }
}

/// A specified value for the `drop-shadow()` filter.
pub type SimpleShadow = GenericSimpleShadow<Option<RGBAColor>, Length, Option<NonNegativeLength>>;

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
                if input
                    .try(|input| input.expect_ident_matching("inset"))
                    .is_ok()
                {
                    inset = true;
                    continue;
                }
            }
            if lengths.is_none() {
                let value = input.try::<_, _, ParseError>(|i| {
                    let horizontal = Length::parse(context, i)?;
                    let vertical = Length::parse(context, i)?;
                    let (blur, spread) = match i.try::<_, _, ParseError>(|i| {
                        Length::parse_non_negative(context, i)
                    }) {
                        Ok(blur) => {
                            let spread = i.try(|i| Length::parse(context, i)).ok();
                            (Some(blur.into()), spread)
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
                if let Ok(value) = input.try(|i| RGBAColor::parse(context, i)) {
                    color = Some(value);
                    continue;
                }
            }
            break;
        }

        let lengths = lengths.ok_or(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))?;
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
            spread: self.spread
                .as_ref()
                .unwrap_or(&Length::zero())
                .to_computed_value(context),
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
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        #[cfg(feature = "gecko")]
        {
            if let Ok(url) = input.try(|i| SpecifiedUrl::parse(context, i)) {
                return Ok(GenericFilter::Url(url));
            }
        }
        let location = input.current_source_location();
        let function = match input.expect_function() {
            Ok(f) => f.clone(),
            Err(cssparser::BasicParseError {
                kind: BasicParseErrorKind::UnexpectedToken(t),
                location,
            }) => return Err(location.new_custom_error(ValueParseErrorKind::InvalidFilter(t))),
            Err(e) => return Err(e.into()),
        };
        input.parse_nested_block(|i| {
            match_ignore_ascii_case! { &*function,
                "blur" => Ok(GenericFilter::Blur((Length::parse_non_negative(context, i)?).into())),
                "brightness" => Ok(GenericFilter::Brightness(Factor::parse(context, i)?)),
                "contrast" => Ok(GenericFilter::Contrast(Factor::parse(context, i)?)),
                "grayscale" => {
                    // Values of amount over 100% are allowed but UAs must clamp the values to 1.
                    // https://drafts.fxtf.org/filter-effects/#funcdef-filter-grayscale
                    Ok(GenericFilter::Grayscale(Factor::parse_with_clamping_to_one(context, i)?))
                },
                "hue-rotate" => {
                    // We allow unitless zero here, see:
                    // https://github.com/w3c/fxtf-drafts/issues/228
                    Ok(GenericFilter::HueRotate(Angle::parse_with_unitless(context, i)?))
                },
                "invert" => {
                    // Values of amount over 100% are allowed but UAs must clamp the values to 1.
                    // https://drafts.fxtf.org/filter-effects/#funcdef-filter-invert
                    Ok(GenericFilter::Invert(Factor::parse_with_clamping_to_one(context, i)?))
                },
                "opacity" => {
                    // Values of amount over 100% are allowed but UAs must clamp the values to 1.
                    // https://drafts.fxtf.org/filter-effects/#funcdef-filter-opacity
                    Ok(GenericFilter::Opacity(Factor::parse_with_clamping_to_one(context, i)?))
                },
                "saturate" => Ok(GenericFilter::Saturate(Factor::parse(context, i)?)),
                "sepia" => {
                    // Values of amount over 100% are allowed but UAs must clamp the values to 1.
                    // https://drafts.fxtf.org/filter-effects/#funcdef-filter-sepia
                    Ok(GenericFilter::Sepia(Factor::parse_with_clamping_to_one(context, i)?))
                },
                "drop-shadow" => Ok(GenericFilter::DropShadow(Parse::parse(context, i)?)),
                _ => Err(location.new_custom_error(
                    ValueParseErrorKind::InvalidFilter(Token::Function(function.clone()))
                )),
            }
        })
    }
}

impl Parse for SimpleShadow {
    #[inline]
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let color = input.try(|i| RGBAColor::parse(context, i)).ok();
        let horizontal = Length::parse(context, input)?;
        let vertical = Length::parse(context, input)?;
        let blur = input.try(|i| Length::parse_non_negative(context, i)).ok();
        let color = color.or_else(|| input.try(|i| RGBAColor::parse(context, i)).ok());
        Ok(SimpleShadow {
            color: color,
            horizontal: horizontal,
            vertical: vertical,
            blur: blur.map(NonNegative::<Length>),
        })
    }
}

impl ToComputedValue for SimpleShadow {
    type ComputedValue = ComputedSimpleShadow;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        ComputedSimpleShadow {
            color: self.color.to_computed_value(context),
            horizontal: self.horizontal.to_computed_value(context),
            vertical: self.vertical.to_computed_value(context),
            blur: self.blur
                .as_ref()
                .unwrap_or(&NonNegativeLength::zero())
                .to_computed_value(context),
        }
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        SimpleShadow {
            color: ToComputedValue::from_computed_value(&computed.color),
            horizontal: ToComputedValue::from_computed_value(&computed.horizontal),
            vertical: ToComputedValue::from_computed_value(&computed.vertical),
            blur: Some(ToComputedValue::from_computed_value(&computed.blur)),
        }
    }
}
