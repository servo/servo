/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::RGBA;
use gecko_style_structs::{nsStyleUnion, nsStyleUnit};
use style::values::computed::{LengthOrPercentage, LengthOrPercentageOrAuto, LengthOrPercentageOrNone};

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

impl ToGeckoStyleCoord for LengthOrPercentageOrNone {
    fn to_gecko_style_coord(&self, unit: &mut nsStyleUnit, union: &mut nsStyleUnion) {
        match *self {
            LengthOrPercentageOrNone::Length(au) => {
                *unit = nsStyleUnit::eStyleUnit_Coord;
                unsafe { *union.mInt.as_mut() = au.0; }
            },
            LengthOrPercentageOrNone::Percentage(p) => {
                *unit = nsStyleUnit::eStyleUnit_Percent;
                unsafe { *union.mFloat.as_mut() = p; }
            },
            LengthOrPercentageOrNone::None => {
                *unit = nsStyleUnit::eStyleUnit_None;
                unsafe { *union.mInt.as_mut() = 0; }
            },
            LengthOrPercentageOrNone::Calc(_) => unimplemented!(),
        };
    }
}

pub fn convert_rgba_to_nscolor(rgba: &RGBA) -> u32 {
    (((rgba.alpha * 255.0).round() as u32) << 24) |
    (((rgba.blue  * 255.0).round() as u32) << 16) |
    (((rgba.green * 255.0).round() as u32) << 8) |
     ((rgba.red   * 255.0).round() as u32)
}

pub fn convert_nscolor_to_rgba(color: u32) -> RGBA {
    RGBA {
        red:    ((color        & 0xff) as f32) / 255.0,
        green: (((color >>  8) & 0xff) as f32) / 255.0,
        blue:  (((color >> 16) & 0xff) as f32) / 255.0,
        alpha: (((color >> 24) & 0xff) as f32) / 255.0,
    }
}
