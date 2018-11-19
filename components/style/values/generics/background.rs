/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Generic types for CSS values related to backgrounds.

use crate::values::IsAuto;
use std::fmt::{self, Write};
use style_traits::{CssWriter, ToCss};

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
)]
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

impl<LengthOrPercentageOrAuto> ToCss for BackgroundSize<LengthOrPercentageOrAuto>
where
    LengthOrPercentageOrAuto: ToCss + IsAuto,
{
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        match self {
            BackgroundSize::Explicit { width, height } => {
                width.to_css(dest)?;
                // NOTE(emilio): We should probably simplify all these in case
                // `width == `height`, but all other browsers agree on only
                // special-casing `auto`.
                if !width.is_auto() || !height.is_auto() {
                    dest.write_str(" ")?;
                    height.to_css(dest)?;
                }
                Ok(())
            },
            BackgroundSize::Cover => dest.write_str("cover"),
            BackgroundSize::Contain => dest.write_str("contain"),
        }
    }
}
