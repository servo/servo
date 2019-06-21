/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Generic types for CSS values related to backgrounds.

use crate::values::generics::length::{GenericLengthPercentageOrAuto, LengthPercentageOrAuto};

/// A generic value for the `background-size` property.
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
pub enum GenericBackgroundSize<LengthPercent> {
    /// `<width> <height>`
    ExplicitSize {
        /// Explicit width.
        width: GenericLengthPercentageOrAuto<LengthPercent>,
        /// Explicit height.
        #[css(skip_if = "GenericLengthPercentageOrAuto::is_auto")]
        height: GenericLengthPercentageOrAuto<LengthPercent>,
    },
    /// `cover`
    #[animation(error)]
    Cover,
    /// `contain`
    #[animation(error)]
    Contain,
}

pub use self::GenericBackgroundSize as BackgroundSize;

impl<LengthPercentage> BackgroundSize<LengthPercentage> {
    /// Returns `auto auto`.
    pub fn auto() -> Self {
        GenericBackgroundSize::ExplicitSize {
            width: LengthPercentageOrAuto::Auto,
            height: LengthPercentageOrAuto::Auto,
        }
    }
}
