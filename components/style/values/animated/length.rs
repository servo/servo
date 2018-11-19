/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Animation implementation for various length-related types.

use super::{Animate, Procedure, ToAnimatedValue, ToAnimatedZero};
use crate::values::computed::length::{CalcLengthOrPercentage, Length};
use crate::values::computed::length::{LengthOrPercentageOrAuto, LengthOrPercentageOrNone};
use crate::values::computed::MaxLength as ComputedMaxLength;
use crate::values::computed::MozLength as ComputedMozLength;
use crate::values::computed::Percentage;

/// <https://drafts.csswg.org/css-transitions/#animtype-lpcalc>
impl Animate for CalcLengthOrPercentage {
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
        Ok(CalcLengthOrPercentage::with_clamping_mode(
            length,
            percentage,
            self.clamping_mode,
        ))
    }
}

impl ToAnimatedZero for LengthOrPercentageOrAuto {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        match *self {
            LengthOrPercentageOrAuto::Length(_) |
            LengthOrPercentageOrAuto::Percentage(_) |
            LengthOrPercentageOrAuto::Calc(_) => {
                Ok(LengthOrPercentageOrAuto::Length(Length::new(0.)))
            },
            LengthOrPercentageOrAuto::Auto => Err(()),
        }
    }
}

impl ToAnimatedZero for LengthOrPercentageOrNone {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        match *self {
            LengthOrPercentageOrNone::Length(_) |
            LengthOrPercentageOrNone::Percentage(_) |
            LengthOrPercentageOrNone::Calc(_) => {
                Ok(LengthOrPercentageOrNone::Length(Length::new(0.)))
            },
            LengthOrPercentageOrNone::None => Err(()),
        }
    }
}

impl ToAnimatedValue for ComputedMaxLength {
    type AnimatedValue = Self;

    #[inline]
    fn to_animated_value(self) -> Self {
        self
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        use crate::values::computed::{Length, LengthOrPercentageOrNone, Percentage};
        use crate::values::generics::length::MaxLength as GenericMaxLength;
        match animated {
            GenericMaxLength::LengthOrPercentageOrNone(lopn) => {
                let result = match lopn {
                    LengthOrPercentageOrNone::Length(px) => {
                        LengthOrPercentageOrNone::Length(Length::new(px.px().max(0.)))
                    },
                    LengthOrPercentageOrNone::Percentage(percentage) => {
                        LengthOrPercentageOrNone::Percentage(Percentage(percentage.0.max(0.)))
                    },
                    _ => lopn,
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
        use crate::values::computed::{Length, LengthOrPercentageOrAuto, Percentage};
        use crate::values::generics::length::MozLength as GenericMozLength;
        match animated {
            GenericMozLength::LengthOrPercentageOrAuto(lopa) => {
                let result = match lopa {
                    LengthOrPercentageOrAuto::Length(px) => {
                        LengthOrPercentageOrAuto::Length(Length::new(px.px().max(0.)))
                    },
                    LengthOrPercentageOrAuto::Percentage(percentage) => {
                        LengthOrPercentageOrAuto::Percentage(Percentage(percentage.0.max(0.)))
                    },
                    _ => lopa,
                };
                GenericMozLength::LengthOrPercentageOrAuto(result)
            },
            _ => animated,
        }
    }
}
