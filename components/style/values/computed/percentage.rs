/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computed percentages.

use std::fmt;
use style_traits::{CssWriter, ToCss};
use values::{CSSFloat, serialize_percentage};

/// A computed percentage.
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[derive(Animate, Clone, ComputeSquaredDistance, Copy, Debug, Default, MallocSizeOf)]
#[derive(PartialEq, PartialOrd, ToAnimatedZero)]
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

impl ToCss for Percentage {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: fmt::Write,
    {
        serialize_percentage(self.0, dest)
    }
}
