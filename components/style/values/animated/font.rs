/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Animation implementation for various font-related types.

use super::{Animate, Procedure, ToAnimatedZero};
use crate::values::computed::font::FontVariationSettings;
use crate::values::distance::{ComputeSquaredDistance, SquaredDistance};

/// <https://drafts.csswg.org/css-fonts-4/#font-variation-settings-def>
///
/// Note that the ComputedValue implementation will already have sorted and de-dup'd
/// the lists of settings, so we can just iterate over the two lists together and
/// animate their individual values.
impl Animate for FontVariationSettings {
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        let result: Vec<_> =
            super::lists::by_computed_value::animate(&self.0, &other.0, procedure)?;
        Ok(Self(result.into_boxed_slice()))
    }
}

impl ComputeSquaredDistance for FontVariationSettings {
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        super::lists::by_computed_value::squared_distance(&self.0, &other.0)
    }
}

impl ToAnimatedZero for FontVariationSettings {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        Err(())
    }
}
