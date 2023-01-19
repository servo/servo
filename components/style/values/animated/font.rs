/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Animation implementation for various font-related types.

use super::{Animate, Procedure, ToAnimatedZero};
use crate::values::computed::font::FontVariationSettings;
use crate::values::distance::{ComputeSquaredDistance, SquaredDistance};
use crate::values::generics::font::FontSettings as GenericFontSettings;

/// <https://drafts.csswg.org/css-fonts-4/#font-variation-settings-def>
///
/// Note that the ComputedValue implementation will already have sorted and de-dup'd
/// the lists of settings, so we can just iterate over the two lists together and
/// animate their individual values.
impl Animate for FontVariationSettings {
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        if self.0.len() == other.0.len() {
            self.0
                .iter()
                .zip(other.0.iter())
                .map(|(st, ot)| st.animate(&ot, procedure))
                .collect::<Result<Vec<_>, ()>>()
                .map(|v| GenericFontSettings(v.into_boxed_slice()))
        } else {
            Err(())
        }
    }
}

impl ComputeSquaredDistance for FontVariationSettings {
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        if self.0.len() == other.0.len() {
            self.0
                .iter()
                .zip(other.0.iter())
                .map(|(st, ot)| st.compute_squared_distance(&ot))
                .sum()
        } else {
            Err(())
        }
    }
}

impl ToAnimatedZero for FontVariationSettings {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        Err(())
    }
}
