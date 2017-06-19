/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Specified types for CSS values related to effects.

use cssparser::{BasicParseError, Parser, Token};
use parser::{Parse, ParserContext};
#[cfg(feature = "gecko")]
use std::fmt;
use style_traits::ParseError;
#[cfg(not(feature = "gecko"))]
use style_traits::StyleParseError;
#[cfg(feature = "gecko")]
use style_traits::ToCss;
use values::computed::{Context, Number as ComputedNumber, ToComputedValue};
use values::computed::effects::DropShadow as ComputedDropShadow;
use values::generics::effects::Filter as GenericFilter;
use values::generics::effects::FilterList as GenericFilterList;
use values::specified::{Angle, Percentage};
#[cfg(feature = "gecko")]
use values::specified::color::Color;
use values::specified::length::Length;
#[cfg(feature = "gecko")]
use values::specified::url::SpecifiedUrl;

/// A specified value for the `filter` property.
pub type FilterList = GenericFilterList<Filter>;

/// A specified value for a single `filter`.
pub type Filter = GenericFilter<Angle, Factor, Length, DropShadow>;

/// A value for the `<factor>` parts in `Filter`.
///
/// FIXME: Should be `NumberOrPercentage`, but Gecko doesn't support that yet.
#[cfg_attr(feature = "servo", derive(Deserialize, HeapSizeOf, Serialize))]
#[derive(Clone, Debug, HasViewportPercentage, PartialEq, ToCss)]
pub enum Factor {
    /// Literal number.
    Number(ComputedNumber),
    /// Literal percentage.
    Percentage(Percentage),
}

/// A specified value for the `drop-shadow()` filter.
///
/// Currently unsupported outside of Gecko.
#[cfg(not(feature = "gecko"))]
#[cfg_attr(feature = "servo", derive(Deserialize, HeapSizeOf, Serialize))]
#[derive(Clone, Debug, HasViewportPercentage, PartialEq, ToCss)]
pub enum DropShadow {}

/// A specified value for the `drop-shadow()` filter.
///
/// Contrary to the canonical order from the spec, the color is serialised
/// first, like in Gecko's computed values and in all Webkit's values.
#[cfg(feature = "gecko")]
#[derive(Clone, Debug, HasViewportPercentage, PartialEq)]
pub struct DropShadow {
    /// Color.
    pub color: Option<Color>,
    /// Horizontal radius.
    pub horizontal: Length,
    /// Vertical radius.
    pub vertical: Length,
    /// Blur radius.
    pub blur: Option<Length>,
}

impl Parse for FilterList {
    #[inline]
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>
    ) -> Result<Self, ParseError<'i>> {
        let mut filters = vec![];
        while let Ok(filter) = input.try(|i| Filter::parse(context, i)) {
            filters.push(filter);
        }
        if filters.is_empty() {
            input.expect_ident_matching("none")?;
        }
        Ok(GenericFilterList(filters.into_boxed_slice()))
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
                "drop-shadow" => Ok(GenericFilter::DropShadow(DropShadow::parse(context, i)?)),
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
    /// `computed::effects::FilterList` directly in `StackingContext`.
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

impl Parse for DropShadow {
    #[cfg(not(feature = "gecko"))]
    #[inline]
    fn parse<'i, 't>(
        _context: &ParserContext,
        _input: &mut Parser<'i, 't>
    ) -> Result<Self, ParseError<'i>> {
        Err(StyleParseError::UnspecifiedError.into())
    }

    #[cfg(feature = "gecko")]
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
        Ok(DropShadow {
            color: color,
            horizontal: horizontal,
            vertical: vertical,
            blur: blur,
        })
    }
}

impl ToComputedValue for DropShadow {
    type ComputedValue = ComputedDropShadow;

    #[cfg(not(feature = "gecko"))]
    #[inline]
    fn to_computed_value(&self, _context: &Context) -> Self::ComputedValue {
        match *self {}
    }

    #[cfg(feature = "gecko")]
    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        ComputedDropShadow {
            color:
                self.color.as_ref().unwrap_or(&Color::CurrentColor).to_computed_value(context),
            horizontal: self.horizontal.to_computed_value(context),
            vertical: self.vertical.to_computed_value(context),
            blur:
                self.blur.as_ref().unwrap_or(&Length::zero()).to_computed_value(context),
        }
    }

    #[cfg(not(feature = "gecko"))]
    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        match *computed {}
    }

    #[cfg(feature = "gecko")]
    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        DropShadow {
            color: Some(ToComputedValue::from_computed_value(&computed.color)),
            horizontal: ToComputedValue::from_computed_value(&computed.horizontal),
            vertical: ToComputedValue::from_computed_value(&computed.vertical),
            blur: Some(ToComputedValue::from_computed_value(&computed.blur)),
        }
    }
}

#[cfg(feature = "gecko")]
impl ToCss for DropShadow {
    #[inline]
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        if let Some(ref color) = self.color {
            color.to_css(dest)?;
            dest.write_str(" ")?;
        }
        self.horizontal.to_css(dest)?;
        dest.write_str(" ")?;
        self.vertical.to_css(dest)?;
        if let Some(ref blur) = self.blur {
            dest.write_str(" ")?;
            blur.to_css(dest)?;
        }
        Ok(())
    }
}
