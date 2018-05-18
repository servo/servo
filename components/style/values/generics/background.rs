/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generic types for CSS values related to backgrounds.

/// A generic value for the `background-size` property.
#[derive(Animate, Clone, ComputeSquaredDistance, Copy, Debug, MallocSizeOf,
         PartialEq, SpecifiedValueInfo, ToAnimatedValue, ToAnimatedZero,
         ToComputedValue, ToCss)]
pub enum BackgroundSize<LengthOrPercentageOrAuto> {
    /// `<width> <height>`
    Explicit {
        /// Explicit width.
        width: LengthOrPercentageOrAuto,
        /// Explicit height.
        height: LengthOrPercentageOrAuto,
    },
    /// `cover`
    #[animation(error)]
    Cover,
    /// `contain`
    #[animation(error)]
    Contain,
}
