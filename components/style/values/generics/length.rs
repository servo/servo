/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Generic types for CSS values related to length.

use crate::parser::{Parse, ParserContext};
#[cfg(feature = "gecko")]
use crate::values::computed::ExtremumLength;
use crate::Zero;
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
    ToResolvedValue,
    ToShmem,
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

impl<LengthPercentage: Zero> Zero for LengthPercentageOrAuto<LengthPercentage> {
    fn zero() -> Self {
        LengthPercentageOrAuto::LengthPercentage(Zero::zero())
    }

    fn is_zero(&self) -> bool {
        match *self {
            LengthPercentageOrAuto::LengthPercentage(ref l) => l.is_zero(),
            LengthPercentageOrAuto::Auto => false,
        }
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
/// Unlike `max-width` or `max-height` properties, a Size can be `auto`,
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
    ToResolvedValue,
    ToShmem,
)]
#[repr(C, u8)]
pub enum GenericSize<LengthPercent> {
    LengthPercentage(LengthPercent),
    Auto,
    #[cfg(feature = "gecko")]
    #[animation(error)]
    ExtremumLength(ExtremumLength),
}

pub use self::GenericSize as Size;

impl<LengthPercentage> Size<LengthPercentage> {
    /// `auto` value.
    #[inline]
    pub fn auto() -> Self {
        Size::Auto
    }

    /// Returns whether we're the auto value.
    #[inline]
    pub fn is_auto(&self) -> bool {
        matches!(*self, Size::Auto)
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
    ToResolvedValue,
    ToShmem,
)]
#[repr(C, u8)]
pub enum GenericMaxSize<LengthPercent> {
    LengthPercentage(LengthPercent),
    None,
    #[cfg(feature = "gecko")]
    #[animation(error)]
    ExtremumLength(ExtremumLength),
}

pub use self::GenericMaxSize as MaxSize;

impl<LengthPercentage> MaxSize<LengthPercentage> {
    /// `none` value.
    #[inline]
    pub fn none() -> Self {
        MaxSize::None
    }
}

/// A generic `<length>` | `<number>` value for the `-moz-tab-size` property.
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Copy,
    Debug,
    MallocSizeOf,
    Parse,
    PartialEq,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToAnimatedZero,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C, u8)]
pub enum GenericLengthOrNumber<L, N> {
    /// A number.
    ///
    /// NOTE: Numbers need to be before lengths, in order to parse them
    /// first, since `0` should be a number, not the `0px` length.
    Number(N),
    /// A length.
    Length(L),
}

pub use self::GenericLengthOrNumber as LengthOrNumber;

impl<L, N: Zero> Zero for LengthOrNumber<L, N> {
    fn zero() -> Self {
        LengthOrNumber::Number(Zero::zero())
    }

    fn is_zero(&self) -> bool {
        match *self {
            LengthOrNumber::Number(ref n) => n.is_zero(),
            LengthOrNumber::Length(..) => false,
        }
    }
}

/// A generic `<length-percentage>` | normal` value.
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Copy,
    Debug,
    MallocSizeOf,
    Parse,
    PartialEq,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToAnimatedZero,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C, u8)]
#[allow(missing_docs)]
pub enum GenericLengthPercentageOrNormal<LengthPercent> {
    LengthPercentage(LengthPercent),
    Normal,
}

pub use self::GenericLengthPercentageOrNormal as LengthPercentageOrNormal;

impl<LengthPercent> LengthPercentageOrNormal<LengthPercent> {
    /// Returns the normal value.
    #[inline]
    pub fn normal() -> Self {
        LengthPercentageOrNormal::Normal
    }
}
