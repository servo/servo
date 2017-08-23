/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Animated types for CSS values in SVG

use app_units::Au;
use values::animated::{Animate, Procedure};
use values::computed::{NonNegativeNumber, Number, Percentage};
use values::computed::length::{CalcLengthOrPercentage, LengthOrPercentage};
use values::computed::length::NonNegativeLengthOrPercentage;

/// Stroke-* value support unit less value, so servo interpolate length value as number.
#[derive(Clone, ComputeSquaredDistance, Copy, Debug, PartialEq, ToAnimatedZero)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub enum SvgNumberOrPercentageOrCalc {
    Number(Number),
    Percentage(Percentage),
    Calc(CalcLengthOrPercentage),
}

impl From<NonNegativeLengthOrPercentage> for SvgNumberOrPercentageOrCalc {
    fn from(lop: NonNegativeLengthOrPercentage) -> SvgNumberOrPercentageOrCalc {
        lop.0.into()
    }
}

impl From<NonNegativeNumber> for SvgNumberOrPercentageOrCalc {
    fn from(num: NonNegativeNumber) -> SvgNumberOrPercentageOrCalc {
        num.0.into()
    }
}

impl From<LengthOrPercentage> for SvgNumberOrPercentageOrCalc {
    fn from(lop: LengthOrPercentage) -> SvgNumberOrPercentageOrCalc {
        match lop {
            LengthOrPercentage::Length(len) => {
                SvgNumberOrPercentageOrCalc::Number(len.to_f32_px())
            },
            LengthOrPercentage::Percentage(p) => {
                SvgNumberOrPercentageOrCalc::Percentage(p)
            },
            LengthOrPercentage::Calc(calc) => {
                SvgNumberOrPercentageOrCalc::Calc(calc.into())
            },
        }
    }

}

impl From<Number> for SvgNumberOrPercentageOrCalc {
    fn from(num: Number) -> SvgNumberOrPercentageOrCalc {
        SvgNumberOrPercentageOrCalc::Number(num)
    }
}

impl From<SvgNumberOrPercentageOrCalc> for CalcLengthOrPercentage {
    fn from(nopoc: SvgNumberOrPercentageOrCalc) -> CalcLengthOrPercentage {
        match nopoc {
            SvgNumberOrPercentageOrCalc::Number(num) =>
                CalcLengthOrPercentage::new(Au::from_f32_px(num), None),
            SvgNumberOrPercentageOrCalc::Percentage(p) =>
                CalcLengthOrPercentage::new(Au(0), Some(p)),
            SvgNumberOrPercentageOrCalc::Calc(calc) =>
                calc.into(),
        }
    }
}

impl Animate for SvgNumberOrPercentageOrCalc {
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        match (self, other) {
            (
                &SvgNumberOrPercentageOrCalc::Number(ref this),
                &SvgNumberOrPercentageOrCalc::Number(ref other)
            ) => {
                Ok(SvgNumberOrPercentageOrCalc::Number(this.animate(other, procedure)?))
            },
            (
                    &SvgNumberOrPercentageOrCalc::Percentage(ref this),
                    &SvgNumberOrPercentageOrCalc::Percentage(ref other)
            ) => {
                Ok(SvgNumberOrPercentageOrCalc::Percentage(this.animate(other, procedure)?))
            },
            // TODO: We need to support calc value.
            // https://bugzilla.mozilla.org/show_bug.cgi?id=1386967
            _ => Err(()),
        }
    }
}
