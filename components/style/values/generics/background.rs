/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Generic types for CSS values related to backgrounds.

use crate::values::generics::length::{LengthPercentageOrAuto, GenericLengthPercentageOrAuto};
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
#[repr(C, u8)]
pub enum GenericBackgroundSize<LengthPercent> {
    /// `<width> <height>`
    ExplicitSize {
        /// Explicit width.
        width: GenericLengthPercentageOrAuto<LengthPercent>,
        /// Explicit height.
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

impl<LengthPercentage> ToCss for BackgroundSize<LengthPercentage>
where
    LengthPercentage: ToCss,
{
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        match self {
            BackgroundSize::ExplicitSize { width, height } => {
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

impl<LengthPercentage> BackgroundSize<LengthPercentage> {
    /// Returns `auto auto`.
    pub fn auto() -> Self {
        GenericBackgroundSize::ExplicitSize {
            width: LengthPercentageOrAuto::Auto,
            height: LengthPercentageOrAuto::Auto,
        }
    }
}
