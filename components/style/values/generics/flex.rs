/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generic types for CSS values related to flexbox.

use values::specified::Percentage;

/// A generic value for the `flex-basis` property.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Copy, Debug, HasViewportPercentage, PartialEq, ToComputedValue, ToCss)]
pub enum FlexBasis<LengthOrPercentage> {
    /// `auto`
    Auto,
    /// `content`
    Content,
    /// `<length-percentage>`
    Length(LengthOrPercentage),
}

impl<L> FlexBasis<L> {
    /// Returns `auto`.
    #[inline]
    pub fn auto() -> Self {
        FlexBasis::Auto
    }
}

impl<L> FlexBasis<L>
where Percentage: Into<L>,
{
    /// Returns `0%`.
    #[inline]
    pub fn zero_percent() -> Self {
        FlexBasis::Length(Percentage(0.).into())
    }
}
