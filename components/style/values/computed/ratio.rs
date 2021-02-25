/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! `<ratio>` computed values.

use crate::values::computed::NonNegativeNumber;
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

impl Ratio {
    /// Returns a new Ratio.
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
}
