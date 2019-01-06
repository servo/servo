/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Animation implementation for various length-related types.

use super::{Animate, Procedure, ToAnimatedValue};
use crate::values::computed::length::LengthOrPercentage;
use crate::values::computed::MaxLength as ComputedMaxLength;
use crate::values::computed::MozLength as ComputedMozLength;
use crate::values::computed::Percentage;

/// <https://drafts.csswg.org/css-transitions/#animtype-lpcalc>
impl Animate for LengthOrPercentage {
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
        let percentage = animate_percentage_half(self.percentage, other.percentage)?;
        let is_calc = self.was_calc || other.was_calc || self.percentage.is_some() != other.percentage.is_some();
        Ok(Self::with_clamping_mode(
            length,
            percentage,
            self.clamping_mode,
            is_calc,
        ))
    }
}

// FIXME(emilio): These should use NonNegative<> instead.
impl ToAnimatedValue for ComputedMaxLength {
    type AnimatedValue = Self;

    #[inline]
    fn to_animated_value(self) -> Self {
        self
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        use crate::values::computed::LengthOrPercentageOrNone;
        use crate::values::generics::length::MaxLength as GenericMaxLength;
        match animated {
            GenericMaxLength::LengthOrPercentageOrNone(lopn) => {
                let result = match lopn {
                    LengthOrPercentageOrNone::LengthOrPercentage(len) => {
                        LengthOrPercentageOrNone::LengthOrPercentage(len.clamp_to_non_negative())
                    },
                    LengthOrPercentageOrNone::None => lopn,
                };
                GenericMaxLength::LengthOrPercentageOrNone(result)
            },
            _ => animated,
        }
    }
}

impl ToAnimatedValue for ComputedMozLength {
    type AnimatedValue = Self;

    #[inline]
    fn to_animated_value(self) -> Self {
        self
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        use crate::values::generics::length::MozLength as GenericMozLength;
        match animated {
            GenericMozLength::LengthOrPercentageOrAuto(lopa) => {
                GenericMozLength::LengthOrPercentageOrAuto(lopa.clamp_to_non_negative())
            },
            _ => animated,
        }
    }
}
