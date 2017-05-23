/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generic types for CSS values related to borders.

use std::fmt;
use style_traits::ToCss;

/// A generic value for a single side of a `border-image-width` property.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Copy, Debug, HasViewportPercentage, PartialEq, ToComputedValue)]
pub enum BorderImageWidthSide<LengthOrPercentage, Number> {
    /// `<length-or-percentage>`
    Length(LengthOrPercentage),
    /// `<number>`
    Number(Number),
    /// `auto`
    Auto,
}

impl<L, N> ToCss for BorderImageWidthSide<L, N>
    where L: ToCss, N: ToCss,
{
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write
    {
        match *self {
            BorderImageWidthSide::Length(ref length) => length.to_css(dest),
            BorderImageWidthSide::Number(ref number) => number.to_css(dest),
            BorderImageWidthSide::Auto => dest.write_str("auto"),
        }
    }
}
