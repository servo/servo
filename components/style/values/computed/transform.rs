/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computed types for CSS values that are related to transformations.

use properties::animated_properties::Animatable;
use values::animated::ToAnimatedZero;
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

impl Animatable for TransformOrigin {
    #[inline]
    fn add_weighted(&self, other: &Self, self_portion: f64, other_portion: f64) -> Result<Self, ()> {
        Ok(Self::new(
            self.horizontal.add_weighted(&other.horizontal, self_portion, other_portion)?,
            self.vertical.add_weighted(&other.vertical, self_portion, other_portion)?,
            self.depth.add_weighted(&other.depth, self_portion, other_portion)?,
        ))
    }

    #[inline]
    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
        self.compute_squared_distance(other).map(f64::sqrt)
    }

    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<f64, ()> {
        Ok(
            self.horizontal.compute_squared_distance(&other.horizontal)? +
            self.vertical.compute_squared_distance(&other.vertical)? +
            self.depth.compute_squared_distance(&other.depth)?
        )
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
