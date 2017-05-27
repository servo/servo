/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generic types for CSS values related to backgrounds.

use std::fmt;
use style_traits::ToCss;

/// A generic value for the `background-size` property.
#[derive(Clone, Copy, Debug, HasViewportPercentage, PartialEq, ToComputedValue)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum BackgroundSize<LengthOrPercentageOrAuto> {
    /// `<width> <height>`
    Explicit {
        /// Explicit width.
        width: LengthOrPercentageOrAuto,
        /// Explicit height.
        height: LengthOrPercentageOrAuto
    },
    /// `cover`
    Cover,
    /// `contain`
    Contain,
}

impl<L> From<L> for BackgroundSize<L>
    where L: Clone,
{
    #[inline]
    fn from(value: L) -> Self {
        BackgroundSize::Explicit { width: value.clone(), height: value }
    }
}

impl<L> ToCss for BackgroundSize<L>
    where L: ToCss
{
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write
    {
        match *self {
            BackgroundSize::Explicit { ref width, ref height } => {
                width.to_css(dest)?;
                dest.write_str(" ")?;
                height.to_css(dest)
            },
            BackgroundSize::Cover => dest.write_str("cover"),
            BackgroundSize::Contain => dest.write_str("contain"),
        }
    }
}
