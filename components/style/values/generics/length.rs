/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Generic types for CSS values related to length.

use crate::parser::{Parse, ParserContext};
use crate::values::computed::ExtremumLength;
use cssparser::Parser;
use style_traits::ParseError;

/// A `<length-percentage> | auto` value.
#[allow(missing_docs)]
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Copy,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToAnimatedZero,
    ToComputedValue,
    ToCss,
)]
#[repr(C, u8)]
pub enum GenericLengthPercentageOrAuto<LengthPercent> {
    LengthPercentage(LengthPercent),
    Auto,
}

pub use self::GenericLengthPercentageOrAuto as LengthPercentageOrAuto;

impl<LengthPercentage> LengthPercentageOrAuto<LengthPercentage> {
    /// `auto` value.
    #[inline]
    pub fn auto() -> Self {
        LengthPercentageOrAuto::Auto
    }

    /// Whether this is the `auto` value.
    #[inline]
    pub fn is_auto(&self) -> bool {
        matches!(*self, LengthPercentageOrAuto::Auto)
    }

    /// A helper function to parse this with quirks or not and so forth.
    pub fn parse_with<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        parser: impl FnOnce(
            &ParserContext,
            &mut Parser<'i, 't>,
        ) -> Result<LengthPercentage, ParseError<'i>>,
    ) -> Result<Self, ParseError<'i>> {
        if input.try(|i| i.expect_ident_matching("auto")).is_ok() {
            return Ok(LengthPercentageOrAuto::Auto);
        }

        Ok(LengthPercentageOrAuto::LengthPercentage(parser(
            context, input,
        )?))
    }
}

impl<LengthPercentage: Parse> Parse for LengthPercentageOrAuto<LengthPercentage> {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        Self::parse_with(context, input, LengthPercentage::parse)
    }
}

/// A generic value for the `width`, `height`, `min-width`, or `min-height` property.
///
/// Unlike `max-width` or `max-height` properties, a MozLength can be `auto`,
/// and cannot be `none`.
///
/// Note that it only accepts non-negative values.
#[allow(missing_docs)]
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Copy,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToAnimatedZero,
    ToComputedValue,
    ToCss,
)]
pub enum MozLength<LengthPercentage> {
    LengthPercentage(LengthPercentage),
    Auto,
    #[animation(error)]
    ExtremumLength(ExtremumLength),
}

impl<LengthPercentage> MozLength<LengthPercentage> {
    /// `auto` value.
    #[inline]
    pub fn auto() -> Self {
        MozLength::Auto
    }
}

/// A generic value for the `max-width` or `max-height` property.
#[allow(missing_docs)]
#[cfg_attr(feature = "servo", derive(MallocSizeOf))]
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Copy,
    Debug,
    PartialEq,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToAnimatedZero,
    ToComputedValue,
    ToCss,
)]
pub enum MaxLength<LengthPercentage> {
    LengthPercentage(LengthPercentage),
    None,
    #[cfg(feature = "gecko")]
    #[animation(error)]
    ExtremumLength(ExtremumLength),
}

impl<LengthPercentage> MaxLength<LengthPercentage> {
    /// `none` value.
    #[inline]
    pub fn none() -> Self {
        MaxLength::None
    }
}
