/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! `<ratio>` computed values.

use crate::values::animated::{Animate, Procedure};
use crate::values::computed::NonNegativeNumber;
use crate::values::distance::{ComputeSquaredDistance, SquaredDistance};
use crate::values::generics::ratio::Ratio as GenericRatio;
use crate::{One, Zero};
use std::cmp::{Ordering, PartialOrd};

/// A computed <ratio> value.
pub type Ratio = GenericRatio<NonNegativeNumber>;

impl PartialOrd for Ratio {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        f64::partial_cmp(
            &((self.0).0 as f64 * (other.1).0 as f64),
            &((self.1).0 as f64 * (other.0).0 as f64),
        )
    }
}

/// https://drafts.csswg.org/css-values/#combine-ratio
impl Animate for Ratio {
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        // If either <ratio> is degenerate, the values cannot be interpolated.
        if self.is_degenerate() || other.is_degenerate() {
            return Err(());
        }

        // Addition of <ratio>s is not possible, and based on
        // https://drafts.csswg.org/css-values-4/#not-additive,
        // we simply use the first value as the result value.
        // Besides, the procedure for accumulation should be identical to addition here.
        if matches!(procedure, Procedure::Add | Procedure::Accumulate { .. }) {
            return Ok(self.clone());
        }

        // The interpolation of a <ratio> is defined by converting each <ratio> to a number by
        // dividing the first value by the second (so a ratio of 3 / 2 would become 1.5), taking
        // the logarithm of that result (so the 1.5 would become approximately 0.176), then
        // interpolating those values.
        //
        // The result during the interpolation is converted back to a <ratio> by inverting the
        // logarithm, then interpreting the result as a <ratio> with the result as the first value
        // and 1 as the second value.
        let start = self.to_f32().ln();
        let end = other.to_f32().ln();
        let e = std::f32::consts::E;
        let result = e.powf(start.animate(&end, procedure)?);
        // The range of the result is [0, inf), based on the easing function.
        if result.is_zero() || result.is_infinite() {
            return Err(());
        }
        Ok(Ratio::new(result, 1.0f32))
    }
}

impl ComputeSquaredDistance for Ratio {
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        if self.is_degenerate() || other.is_degenerate() {
            return Err(());
        }
        // Use the distance of their logarithm values. (This is used by testing, so don't need to
        // care about the base. Here we use the same base as that in animate().)
        self.to_f32()
            .ln()
            .compute_squared_distance(&other.to_f32().ln())
    }
}

impl Zero for Ratio {
    fn zero() -> Self {
        Self::new(Zero::zero(), One::one())
    }

    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}

impl Ratio {
    /// Returns a new Ratio.
    #[inline]
    pub fn new(a: f32, b: f32) -> Self {
        GenericRatio(a.into(), b.into())
    }

    /// Returns the used value. A ratio of 0/0 behaves as the ratio 1/0.
    /// https://drafts.csswg.org/css-values-4/#ratios
    pub fn used_value(self) -> Self {
        if self.0.is_zero() && self.1.is_zero() {
            Ratio::new(One::one(), Zero::zero())
        } else {
            self
        }
    }

    /// Returns true if this is a degenerate ratio.
    /// https://drafts.csswg.org/css-values/#degenerate-ratio
    #[inline]
    pub fn is_degenerate(&self) -> bool {
        self.0.is_zero() || self.1.is_zero()
    }

    /// Returns the f32 value by dividing the first value by the second one.
    #[inline]
    fn to_f32(&self) -> f32 {
        debug_assert!(!self.is_degenerate());
        (self.0).0 / (self.1).0
    }
}
