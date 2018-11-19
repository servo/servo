/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Generic types for CSS values related to length.

use crate::values::computed::ExtremumLength;

/// A generic value for the `width`, `height`, `min-width`, or `min-height` property.
///
/// Unlike `max-width` or `max-height` properties, a MozLength can be `auto`,
/// and cannot be `none`.
///
/// Note that it only accepts non-negative values.
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
    ToAnimatedZero,
    ToComputedValue,
    ToCss,
)]
pub enum MozLength<LengthOrPercentageOrAuto> {
    LengthOrPercentageOrAuto(LengthOrPercentageOrAuto),
    #[animation(error)]
    ExtremumLength(ExtremumLength),
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
    ToAnimatedZero,
    ToComputedValue,
    ToCss,
)]
pub enum MaxLength<LengthOrPercentageOrNone> {
    LengthOrPercentageOrNone(LengthOrPercentageOrNone),
    #[animation(error)]
    ExtremumLength(ExtremumLength),
}
