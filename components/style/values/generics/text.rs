/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generic types for text properties.

use std::fmt;
use style_traits::ToCss;

/// A generic value for the `line-height` property.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Copy, Debug, HasViewportPercentage, PartialEq)]
pub enum LineHeight<Number, LengthOrPercentage> {
    /// `normal`
    Normal,
    /// `-moz-block-height`
    #[cfg(feature = "gecko")]
    MozBlockHeight,
    /// `<number>`
    Number(Number),
    /// `<length-or-percentage>`
    Length(LengthOrPercentage),
}

impl<N, L> LineHeight<N, L> {
    /// Returns `normal`.
    #[inline]
    pub fn normal() -> Self {
        LineHeight::Normal
    }
}

impl<N, L> ToCss for LineHeight<N, L>
    where N: ToCss, L: ToCss,
{
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write,
    {
        match *self {
            LineHeight::Normal => dest.write_str("normal"),
            #[cfg(feature = "gecko")]
            LineHeight::MozBlockHeight => dest.write_str("-moz-block-height"),
            LineHeight::Number(ref number) => number.to_css(dest),
            LineHeight::Length(ref value) => value.to_css(dest),
        }
    }
}
