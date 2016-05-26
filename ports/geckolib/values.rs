/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use app_units::Au;
use cssparser::RGBA;
use gecko_bindings::structs::{nsStyleCoord, nsStyleUnion, nsStyleUnit};
use std::cmp::max;
use style::values::computed::{LengthOrPercentage, LengthOrPercentageOrAuto, LengthOrPercentageOrNone};

pub trait StyleCoordHelpers {
    fn set<T: ToGeckoStyleCoord>(&mut self, val: T);
    fn set_auto(&mut self);
    fn set_normal(&mut self);
    fn set_coord(&mut self, val: Au);
    fn set_int(&mut self, val: i32);
    fn set_enum(&mut self, val: i32);
    fn set_percent(&mut self, val: f32);
    fn set_factor(&mut self, val: f32);
}

impl StyleCoordHelpers for nsStyleCoord {
    fn set<T: ToGeckoStyleCoord>(&mut self, val: T) {
        val.to_gecko_style_coord(&mut self.mUnit, &mut self.mValue);
    }

    fn set_auto(&mut self) {
        self.mUnit = nsStyleUnit::eStyleUnit_Auto;
        unsafe { *self.mValue.mInt.as_mut() = 0; }
    }

    fn set_normal(&mut self) {
        self.mUnit = nsStyleUnit::eStyleUnit_Normal;
        unsafe { *self.mValue.mInt.as_mut() = 0; }
    }

    fn set_coord(&mut self, val: Au) {
        self.mUnit = nsStyleUnit::eStyleUnit_Coord;
        unsafe { *self.mValue.mInt.as_mut() = val.0; }
    }

    fn set_percent(&mut self, val: f32) {
        self.mUnit = nsStyleUnit::eStyleUnit_Percent;
        unsafe { *self.mValue.mFloat.as_mut() = val; }
    }

    fn set_int(&mut self, val: i32) {
        self.mUnit = nsStyleUnit::eStyleUnit_Integer;
        unsafe { *self.mValue.mInt.as_mut() = val; }
    }

    fn set_enum(&mut self, val: i32) {
        self.mUnit = nsStyleUnit::eStyleUnit_Enumerated;
        unsafe { *self.mValue.mInt.as_mut() = val; }
    }

    fn set_factor(&mut self, val: f32) {
        self.mUnit = nsStyleUnit::eStyleUnit_Factor;
        unsafe { *self.mValue.mFloat.as_mut() = val; }
    }
}

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

#[inline]
pub fn round_border_to_device_pixels(width: Au, au_per_device_px: Au) -> Au {
    // Round width down to the nearest device pixel, but any non-zero value that
    // would round down to zero is clamped to 1 device pixel.  Used for storing
    // computed values of border-*-width and outline-width.
    if width == Au(0) {
        Au(0)
    } else {
        max(au_per_device_px, Au(width.0 / au_per_device_px.0 * au_per_device_px.0))
    }
}

pub fn debug_assert_unit_is_safe_to_copy(unit: nsStyleUnit) {
    debug_assert!(unit != nsStyleUnit::eStyleUnit_Calc, "stylo: Can't yet handle refcounted Calc");
}
