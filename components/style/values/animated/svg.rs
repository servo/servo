/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Animated types for CSS values in SVG

use app_units::Au;
use values::animated::{Animate, Procedure, ToAnimatedZero};
use values::computed::{NonNegativeNumber, Number, Percentage};
use values::computed::length::{CalcLengthOrPercentage, LengthOrPercentage};
use values::computed::length::NonNegativeLengthOrPercentage;


/// | number | percentage | calc| for stroke-*
/// Stroke-* value support unit less value, so servo interpolate length value as number.
#[derive(Clone, ComputeSquaredDistance, PartialEq, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub enum SvgStrokeNumberOrPercentageOrCalc {
    Number(Number),
    Percentage(Percentage),
    Calc(CalcLengthOrPercentage),
}

impl From<NonNegativeLengthOrPercentage> for SvgStrokeNumberOrPercentageOrCalc {
    fn from(lop: NonNegativeLengthOrPercentage) -> SvgStrokeNumberOrPercentageOrCalc {
        lop.0.into()
    }
}

impl From<NonNegativeNumber> for SvgStrokeNumberOrPercentageOrCalc {
    fn from(num: NonNegativeNumber) -> SvgStrokeNumberOrPercentageOrCalc {
        num.0.into()
    }
}

impl From<LengthOrPercentage> for SvgStrokeNumberOrPercentageOrCalc {
    fn from(lop: LengthOrPercentage) -> SvgStrokeNumberOrPercentageOrCalc {
        match lop {
            LengthOrPercentage::Length(len) => {
                SvgStrokeNumberOrPercentageOrCalc::Number(len.to_f32_px())
            },
            LengthOrPercentage::Percentage(p) => {
                SvgStrokeNumberOrPercentageOrCalc::Percentage(p)
            },
            LengthOrPercentage::Calc(calc) => {
                SvgStrokeNumberOrPercentageOrCalc::Calc(calc.into())
            },
        }
    }

}

impl From<Number> for SvgStrokeNumberOrPercentageOrCalc {
    fn from(num: Number) -> SvgStrokeNumberOrPercentageOrCalc {
        SvgStrokeNumberOrPercentageOrCalc::Number(num)
    }
}

impl From<SvgStrokeNumberOrPercentageOrCalc> for CalcLengthOrPercentage {
    fn from(nopoc: SvgStrokeNumberOrPercentageOrCalc) -> CalcLengthOrPercentage {
        match nopoc {
            SvgStrokeNumberOrPercentageOrCalc::Number(num) =>
                CalcLengthOrPercentage::new(Au::from_f32_px(num), None),
            SvgStrokeNumberOrPercentageOrCalc::Percentage(p) =>
                CalcLengthOrPercentage::new(Au(0), Some(p)),
            SvgStrokeNumberOrPercentageOrCalc::Calc(calc) =>
                calc.into(),
        }
    }
}

impl Animate for SvgStrokeNumberOrPercentageOrCalc {
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        match (self, other) {
            (
                &SvgStrokeNumberOrPercentageOrCalc::Number(ref this),
                &SvgStrokeNumberOrPercentageOrCalc::Number(ref other)
            ) => {
                Ok(SvgStrokeNumberOrPercentageOrCalc::Number(this.animate(other, procedure)?))
            },
            (
                    &SvgStrokeNumberOrPercentageOrCalc::Percentage(ref this),
                    &SvgStrokeNumberOrPercentageOrCalc::Percentage(ref other)
            ) => {
                Ok(SvgStrokeNumberOrPercentageOrCalc::Percentage(this.animate(other, procedure)?))
            },
            // TODO: We need to support calc value.
            // https://bugzilla.mozilla.org/show_bug.cgi?id=1386967
            _ => Err(()),
        }
    }
}

impl ToAnimatedZero for SvgStrokeNumberOrPercentageOrCalc {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        match *self {
            SvgStrokeNumberOrPercentageOrCalc::Number(num) =>
                Ok(SvgStrokeNumberOrPercentageOrCalc::Number(num.to_animated_zero()?)),
            SvgStrokeNumberOrPercentageOrCalc::Percentage(p) =>
                Ok(SvgStrokeNumberOrPercentageOrCalc::Percentage(p.to_animated_zero()?)),
            SvgStrokeNumberOrPercentageOrCalc::Calc(calc) =>
                Ok(SvgStrokeNumberOrPercentageOrCalc::Calc(calc.to_animated_zero()?)),
        }
    }
}

