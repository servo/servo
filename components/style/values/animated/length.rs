/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Animation implementation for various length-related types.

use super::{Animate, Procedure};
use crate::values::computed::length::LengthPercentage;
use crate::values::computed::Percentage;

/// <https://drafts.csswg.org/css-transitions/#animtype-lpcalc>
impl Animate for LengthPercentage {
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        let animate_percentage_half = |this: Option<Percentage>, other: Option<Percentage>| {
            if this.is_none() && other.is_none() {
                return Ok(None);
            }
            let this = this.unwrap_or_default();
            let other = other.unwrap_or_default();
            Ok(Some(this.animate(&other, procedure)?))
        };

        let length = self
            .unclamped_length()
            .animate(&other.unclamped_length(), procedure)?;
        let percentage =
            animate_percentage_half(self.specified_percentage(), other.specified_percentage())?;
        Ok(Self::with_clamping_mode(
            length,
            percentage,
            self.clamping_mode,
        ))
    }
}
