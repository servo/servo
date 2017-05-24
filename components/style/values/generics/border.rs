/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generic types for CSS values related to borders.

use std::fmt;
use style_traits::ToCss;
use values::generics::rect::Rect;

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

/// A generic value for the `border-image-slice` property.
#[derive(Clone, Copy, Debug, HasViewportPercentage, PartialEq, ToComputedValue)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct BorderImageSlice<NumberOrPercentage> {
    /// The offsets.
    pub offsets: Rect<NumberOrPercentage>,
    /// Whether to fill the middle part.
    pub fill: bool,
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

impl<N> From<N> for BorderImageSlice<N>
    where N: Clone,
{
    #[inline]
    fn from(value: N) -> Self {
        Self {
            offsets: value.into(),
            fill: false,
        }
    }
}

impl<N> ToCss for BorderImageSlice<N>
    where N: PartialEq + ToCss,
{
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write
    {
        self.offsets.to_css(dest)?;
        if self.fill {
            dest.write_str(" fill")?;
        }
        Ok(())
    }
}
