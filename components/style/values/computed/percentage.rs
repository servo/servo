/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computed percentages.

use std::fmt;
use style_traits::ToCss;
use values::{CSSFloat, serialize_percentage};
use values::animated::{Animate, Procedure, ToAnimatedZero};

/// A computed percentage.
#[derive(Clone, ComputeSquaredDistance, Copy, Debug, Default, HasViewportPercentage, PartialEq, PartialOrd)]
#[cfg_attr(feature = "servo", derive(Deserialize, HeapSizeOf, Serialize))]
pub struct Percentage(pub CSSFloat);

impl Percentage {
    /// 0%
    #[inline]
    pub fn zero() -> Self {
        Percentage(0.)
    }

    /// 100%
    #[inline]
    pub fn hundred() -> Self {
        Percentage(1.)
    }

    /// Returns the absolute value for this percentage.
    #[inline]
    pub fn abs(&self) -> Self {
        Percentage(self.0.abs())
    }
}

/// https://drafts.csswg.org/css-transitions/#animtype-percentage
impl Animate for Percentage {
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        Ok(Percentage(self.0.animate(&other.0, procedure)?))
    }
}

impl ToAnimatedZero for Percentage {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        Ok(Percentage(0.))
    }
}

impl ToCss for Percentage {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        serialize_percentage(self.0, dest)
    }
}
