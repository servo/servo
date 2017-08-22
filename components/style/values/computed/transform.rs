/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computed types for CSS values that are related to transformations.

use values::animated::{Animate, Procedure, ToAnimatedZero};
use values::computed::{Length, LengthOrPercentage, Number, Percentage};
use values::generics::transform::TimingFunction as GenericTimingFunction;
use values::generics::transform::TransformOrigin as GenericTransformOrigin;

/// The computed value of a CSS `<transform-origin>`
pub type TransformOrigin = GenericTransformOrigin<LengthOrPercentage, LengthOrPercentage, Length>;

/// A computed timing function.
pub type TimingFunction = GenericTimingFunction<u32, Number>;

impl TransformOrigin {
    /// Returns the initial computed value for `transform-origin`.
    #[inline]
    pub fn initial_value() -> Self {
        Self::new(
            LengthOrPercentage::Percentage(Percentage(0.5)),
            LengthOrPercentage::Percentage(Percentage(0.5)),
            Length::from_px(0),
        )
    }
}

impl Animate for TransformOrigin {
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        Ok(Self::new(
            self.horizontal.animate(&other.horizontal, procedure)?,
            self.vertical.animate(&other.vertical, procedure)?,
            self.depth.animate(&other.depth, procedure)?,
        ))
    }
}

impl ToAnimatedZero for TransformOrigin {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        Ok(Self::new(
            self.horizontal.to_animated_zero()?,
            self.vertical.to_animated_zero()?,
            self.depth.to_animated_zero()?,
        ))
    }
}
