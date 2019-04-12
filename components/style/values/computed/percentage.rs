/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Computed percentages.

use crate::values::animated::ToAnimatedValue;
use crate::values::generics::NonNegative;
use crate::values::{serialize_percentage, CSSFloat};
use crate::Zero;
use std::fmt;
use style_traits::{CssWriter, ToCss};

/// A computed percentage.
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Copy,
    Debug,
    Default,
    MallocSizeOf,
    PartialEq,
    PartialOrd,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToAnimatedZero,
    ToComputedValue,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C)]
pub struct Percentage(pub CSSFloat);

impl Percentage {
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

    /// Clamps this percentage to a non-negative percentage.
    #[inline]
    pub fn clamp_to_non_negative(self) -> Self {
        Percentage(self.0.max(0.))
    }
}

impl Zero for Percentage {
    fn zero() -> Self {
        Percentage(0.)
    }

    fn is_zero(&self) -> bool {
        self.0 == 0.
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

/// A wrapper over a `Percentage`, whose value should be clamped to 0.
pub type NonNegativePercentage = NonNegative<Percentage>;

impl NonNegativePercentage {
    /// 100%
    #[inline]
    pub fn hundred() -> Self {
        NonNegative(Percentage::hundred())
    }
}

impl ToAnimatedValue for NonNegativePercentage {
    type AnimatedValue = Percentage;

    #[inline]
    fn to_animated_value(self) -> Self::AnimatedValue {
        self.0
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        NonNegative(animated.clamp_to_non_negative())
    }
}
