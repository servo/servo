/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use gecko_style_structs::{nsStyleUnion, nsStyleUnit};
use style::values::computed::{LengthOrPercentage, LengthOrPercentageOrAuto};

pub trait ToGeckoStyleCoord {
    fn to_gecko_style_coord(&self, unit: &mut nsStyleUnit, union: &mut nsStyleUnion);
}

impl ToGeckoStyleCoord for LengthOrPercentage {
    fn to_gecko_style_coord(&self, unit: &mut nsStyleUnit, union: &mut nsStyleUnion) {
        match *self {
            LengthOrPercentage::Length(au) => {
                *unit = nsStyleUnit::eStyleUnit_Coord;
                unsafe { *union.mInt.as_mut() = au.0; }
            },
            LengthOrPercentage::Percentage(p) => {
                *unit = nsStyleUnit::eStyleUnit_Percent;
                unsafe { *union.mFloat.as_mut() = p; }
            },
            LengthOrPercentage::Calc(_) => unimplemented!(),
        };
    }
}

impl ToGeckoStyleCoord for LengthOrPercentageOrAuto {
    fn to_gecko_style_coord(&self, unit: &mut nsStyleUnit, union: &mut nsStyleUnion) {
        match *self {
            LengthOrPercentageOrAuto::Length(au) => {
                *unit = nsStyleUnit::eStyleUnit_Coord;
                unsafe { *union.mInt.as_mut() = au.0; }
            },
            LengthOrPercentageOrAuto::Percentage(p) => {
                *unit = nsStyleUnit::eStyleUnit_Percent;
                unsafe { *union.mFloat.as_mut() = p; }
            },
            LengthOrPercentageOrAuto::Auto => {
                *unit = nsStyleUnit::eStyleUnit_Auto;
                unsafe { *union.mInt.as_mut() = 0; }
            },
            LengthOrPercentageOrAuto::Calc(_) => unimplemented!(),
        };
    }
}
