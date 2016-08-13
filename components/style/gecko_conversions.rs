/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! This module contains conversion helpers between Servo and Gecko types
//! Ideally, it would be in geckolib itself, but coherence
//! forces us to keep the traits and implementations here

#![allow(unsafe_code)]

use app_units::Au;
use gecko_bindings::bindings::{RawServoStyleSheet, ServoComputedValues};
use gecko_bindings::structs::nsStyleCoord_CalcValue;
use gecko_bindings::sugar::refptr::HasArcFFI;
use properties::ComputedValues;
use stylesheets::Stylesheet;
use values::computed::{CalcLengthOrPercentage, LengthOrPercentage};

unsafe impl HasArcFFI for Stylesheet {
    type FFIType = RawServoStyleSheet;
}
unsafe impl HasArcFFI for ComputedValues {
    type FFIType = ServoComputedValues;
}

impl From<CalcLengthOrPercentage> for nsStyleCoord_CalcValue {
    fn from(other: CalcLengthOrPercentage) -> nsStyleCoord_CalcValue {
        let has_percentage = other.percentage.is_some();
        nsStyleCoord_CalcValue {
            mLength: other.length.map_or(0, |l| l.0),
            mPercent: other.percentage.unwrap_or(0.0),
            mHasPercent: has_percentage,
        }
    }
}

impl From<nsStyleCoord_CalcValue> for CalcLengthOrPercentage {
    fn from(other: nsStyleCoord_CalcValue) -> CalcLengthOrPercentage {
        let percentage = if other.mHasPercent {
            Some(other.mPercent)
        } else {
            None
        };
        CalcLengthOrPercentage {
            length: Some(Au(other.mLength)),
            percentage: percentage,
        }
    }
}

impl From<LengthOrPercentage> for nsStyleCoord_CalcValue {
    fn from(other: LengthOrPercentage) -> nsStyleCoord_CalcValue {
        match other {
            LengthOrPercentage::Length(au) => {
                nsStyleCoord_CalcValue {
                    mLength: au.0,
                    mPercent: 0.0,
                    mHasPercent: false,
                }
            },
            LengthOrPercentage::Percentage(pc) => {
                nsStyleCoord_CalcValue {
                    mLength: 0,
                    mPercent: pc,
                    mHasPercent: true,
                }
            },
            LengthOrPercentage::Calc(calc) => calc.into(),
        }
    }
}

impl From<nsStyleCoord_CalcValue> for LengthOrPercentage {
    fn from(other: nsStyleCoord_CalcValue) -> LengthOrPercentage {
        match (other.mHasPercent, other.mLength) {
            (false, _) => LengthOrPercentage::Length(Au(other.mLength)),
            (true, 0) => LengthOrPercentage::Percentage(other.mPercent),
            _ => LengthOrPercentage::Calc(other.into()),
        }
    }
}
