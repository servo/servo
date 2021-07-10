/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Specified types for CSS values related to effects.

use crate::parser::{Parse, ParserContext};
use crate::values::computed::effects::BoxShadow as ComputedBoxShadow;
use crate::values::computed::effects::SimpleShadow as ComputedSimpleShadow;
use crate::values::computed::NonNegativeNumber as ComputedNonNegativeNumber;
use crate::values::computed::ZeroToOneNumber as ComputedZeroToOneNumber;
use crate::values::computed::{Context, ToComputedValue};
use crate::values::generics::effects::BoxShadow as GenericBoxShadow;
use crate::values::generics::effects::Filter as GenericFilter;
use crate::values::generics::effects::SimpleShadow as GenericSimpleShadow;
use crate::values::generics::NonNegative;
use crate::values::specified::color::Color;
use crate::values::specified::length::{Length, NonNegativeLength};
#[cfg(feature = "gecko")]
use crate::values::specified::url::SpecifiedUrl;
use crate::values::specified::{Angle, Number, NumberOrPercentage};
#[cfg(feature = "servo")]
use crate::values::Impossible;
use crate::Zero;
use cssparser::{self, BasicParseErrorKind, Parser, Token};
use style_traits::{ParseError, StyleParseErrorKind, ValueParseErrorKind};

/// A specified value for a single shadow of the `box-shadow` property.
pub type BoxShadow =
    GenericBoxShadow<Option<Color>, Length, Option<NonNegativeLength>, Option<Length>>;

/// A specified value for a single `filter`.
#[cfg(feature = "gecko")]
pub type SpecifiedFilter = GenericFilter<
    Angle,
    NonNegativeFactor,
    ZeroToOneFactor,
    NonNegativeLength,
    SimpleShadow,
    SpecifiedUrl,
>;

/// A specified value for a single `filter`.
#[cfg(feature = "servo")]
pub type SpecifiedFilter = GenericFilter<
    Angle,
    NonNegativeFactor,
    ZeroToOneFactor,
    NonNegativeLength,
    Impossible,
    Impossible,
>;

pub use self::SpecifiedFilter as Filter;

/// A value for the `<factor>` parts in `Filter`.
#[derive(Clone, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo, ToCss, ToShmem)]
pub struct NonNegativeFactor(NumberOrPercentage);

/// A value for the `<factor>` parts in `Filter` which clamps to one.
#[derive(Clone, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo, ToCss, ToShmem)]
pub struct ZeroToOneFactor(NumberOrPercentage);

/// Clamp the value to 1 if the value is over 100%.
#[inline]
fn clamp_to_one(number: NumberOrPercentage) -> NumberOrPercentage {
    match number {
        NumberOrPercentage::Percentage(percent) => {
            NumberOrPercentage::Percentage(percent.clamp_to_hundred())
        },
        NumberOrPercentage::Number(number) => NumberOrPercentage::Number(number.clamp_to_one()),
    }
}

macro_rules! factor_impl_common {
    ($ty:ty, $computed_ty:ty) => {
        impl $ty {
            fn one() -> Self {
                Self(NumberOrPercentage::Number(Number::new(1.)))
            }
        }

        impl ToComputedValue for $ty {
            type ComputedValue = $computed_ty;

            #[inline]
            fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
                use crate::values::computed::NumberOrPercentage;
                match self.0.to_computed_value(context) {
                    NumberOrPercentage::Number(n) => n.into(),
                    NumberOrPercentage::Percentage(p) => p.0.into(),
                }
            }

            #[inline]
            fn from_computed_value(computed: &Self::ComputedValue) -> Self {
                Self(NumberOrPercentage::Number(
                    ToComputedValue::from_computed_value(&computed.0),
                ))
            }
        }
    };
}
factor_impl_common!(NonNegativeFactor, ComputedNonNegativeNumber);
factor_impl_common!(ZeroToOneFactor, ComputedZeroToOneNumber);

impl Parse for NonNegativeFactor {
    #[inline]
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        NumberOrPercentage::parse_non_negative(context, input).map(Self)
    }
}

impl Parse for ZeroToOneFactor {
    #[inline]
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        NumberOrPercentage::parse_non_negative(context, input)
            .map(clamp_to_one)
            .map(Self)
    }
}

/// A specified value for the `drop-shadow()` filter.
pub type SimpleShadow = GenericSimpleShadow<Option<Color>, Length, Option<NonNegativeLength>>;

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
                    .try_parse(|input| input.expect_ident_matching("inset"))
                    .is_ok()
                {
                    inset = true;
                    continue;
                }
            }
            if lengths.is_none() {
                let value = input.try_parse::<_, _, ParseError>(|i| {
                    let horizontal = Length::parse(context, i)?;
                    let vertical = Length::parse(context, i)?;
                    let (blur, spread) =
                        match i.try_parse(|i| Length::parse_non_negative(context, i)) {
                            Ok(blur) => {
                                let spread = i.try_parse(|i| Length::parse(context, i)).ok();
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
                if let Ok(value) = input.try_parse(|i| Color::parse(context, i)) {
                    color = Some(value);
                    continue;
                }
            }
            break;
        }

        let lengths =
            lengths.ok_or(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))?;
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
            spread: self
                .spread
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
            if let Ok(url) = input.try_parse(|i| SpecifiedUrl::parse(context, i)) {
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
                "blur" => Ok(GenericFilter::Blur(
                    i.try_parse(|i| NonNegativeLength::parse(context, i))
                     .unwrap_or(Zero::zero()),
                )),
                "brightness" => Ok(GenericFilter::Brightness(
                    i.try_parse(|i| NonNegativeFactor::parse(context, i))
                     .unwrap_or(NonNegativeFactor::one()),
                )),
                "contrast" => Ok(GenericFilter::Contrast(
                    i.try_parse(|i| NonNegativeFactor::parse(context, i))
                     .unwrap_or(NonNegativeFactor::one()),
                )),
                "grayscale" => {
                    // Values of amount over 100% are allowed but UAs must clamp the values to 1.
                    // https://drafts.fxtf.org/filter-effects/#funcdef-filter-grayscale
                    Ok(GenericFilter::Grayscale(
                        i.try_parse(|i| ZeroToOneFactor::parse(context, i))
                         .unwrap_or(ZeroToOneFactor::one()),
                    ))
                },
                "hue-rotate" => {
                    // We allow unitless zero here, see:
                    // https://github.com/w3c/fxtf-drafts/issues/228
                    Ok(GenericFilter::HueRotate(
                        i.try_parse(|i| Angle::parse_with_unitless(context, i))
                         .unwrap_or(Zero::zero()),
                    ))
                },
                "invert" => {
                    // Values of amount over 100% are allowed but UAs must clamp the values to 1.
                    // https://drafts.fxtf.org/filter-effects/#funcdef-filter-invert
                    Ok(GenericFilter::Invert(
                        i.try_parse(|i| ZeroToOneFactor::parse(context, i))
                         .unwrap_or(ZeroToOneFactor::one()),
                    ))
                },
                "opacity" => {
                    // Values of amount over 100% are allowed but UAs must clamp the values to 1.
                    // https://drafts.fxtf.org/filter-effects/#funcdef-filter-opacity
                    Ok(GenericFilter::Opacity(
                        i.try_parse(|i| ZeroToOneFactor::parse(context, i))
                         .unwrap_or(ZeroToOneFactor::one()),
                    ))
                },
                "saturate" => Ok(GenericFilter::Saturate(
                    i.try_parse(|i| NonNegativeFactor::parse(context, i))
                     .unwrap_or(NonNegativeFactor::one()),
                )),
                "sepia" => {
                    // Values of amount over 100% are allowed but UAs must clamp the values to 1.
                    // https://drafts.fxtf.org/filter-effects/#funcdef-filter-sepia
                    Ok(GenericFilter::Sepia(
                        i.try_parse(|i| ZeroToOneFactor::parse(context, i))
                         .unwrap_or(ZeroToOneFactor::one()),
                    ))
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
        let color = input.try_parse(|i| Color::parse(context, i)).ok();
        let horizontal = Length::parse(context, input)?;
        let vertical = Length::parse(context, input)?;
        let blur = input
            .try_parse(|i| Length::parse_non_negative(context, i))
            .ok();
        let blur = blur.map(NonNegative::<Length>);
        let color = color.or_else(|| input.try_parse(|i| Color::parse(context, i)).ok());

        Ok(SimpleShadow {
            color,
            horizontal,
            vertical,
            blur,
        })
    }
}

impl ToComputedValue for SimpleShadow {
    type ComputedValue = ComputedSimpleShadow;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        ComputedSimpleShadow {
            color: self
                .color
                .as_ref()
                .unwrap_or(&Color::currentcolor())
                .to_computed_value(context),
            horizontal: self.horizontal.to_computed_value(context),
            vertical: self.vertical.to_computed_value(context),
            blur: self
                .blur
                .as_ref()
                .unwrap_or(&NonNegativeLength::zero())
                .to_computed_value(context),
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
