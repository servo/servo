/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generic types for legacy Gecko-only properties that should probably be
//! unshipped at some point in the future.

use std::fmt;
use style_traits::ToCss;

/// A generic value for scroll snap points.
#[derive(Clone, Copy, Debug, HasViewportPercentage, PartialEq, ToComputedValue)]
pub enum ScrollSnapPoint<LengthOrPercentage> {
    /// `none`
    None,
    /// `repeat(<length-or-percentage>)`
    Repeat(LengthOrPercentage)
}

impl<L> ScrollSnapPoint<L> {
    /// Returns `none`.
    #[inline]
    pub fn none() -> Self {
        ScrollSnapPoint::None
    }

    /// Returns the repeat argument, if any.
    #[inline]
    pub fn repeated(&self) -> Option<&L> {
        match *self {
            ScrollSnapPoint::None => None,
            ScrollSnapPoint::Repeat(ref length) => Some(length),
        }
    }
}

impl<L> ToCss for ScrollSnapPoint<L>
where
    L: ToCss,
{
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        match *self {
            ScrollSnapPoint::None => dest.write_str("none"),
            ScrollSnapPoint::Repeat(ref length) => {
                dest.write_str("repeat(")?;
                length.to_css(dest)?;
                dest.write_str(")")
            },
        }
    }
}
