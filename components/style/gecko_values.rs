/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

use app_units::Au;
use cssparser::RGBA;
use gecko_bindings::structs::{nsStyleCoord, nsStyleUnion, nsStyleUnit};
use std::cmp::max;
use values::computed::Angle;
use values::computed::{LengthOrPercentage, LengthOrPercentageOrAuto, LengthOrPercentageOrNone};

pub trait StyleCoordHelpers {
    fn copy_from(&mut self, other: &Self);
    fn set<T: GeckoStyleCoordConvertible>(&mut self, val: T);

    fn set_auto(&mut self);
    fn is_auto(&self) -> bool;

    fn set_normal(&mut self);
    fn is_normal(&self) -> bool;

    fn set_coord(&mut self, val: Au);
    fn is_coord(&self) -> bool;
    fn get_coord(&self) -> Au;

    fn set_int(&mut self, val: i32);
    fn is_int(&self) -> bool;
    fn get_int(&self) -> i32;

    fn set_enum(&mut self, val: i32);
    fn is_enum(&self) -> bool;
    fn get_enum(&self) -> i32;

    fn set_percent(&mut self, val: f32);
    fn is_percent(&self) -> bool;
    fn get_percent(&self) -> f32;

    fn set_factor(&mut self, val: f32);
    fn is_factor(&self) -> bool;
    fn get_factor(&self) -> f32;
}

impl StyleCoordHelpers for nsStyleCoord {
    #[inline]
    fn copy_from(&mut self, other: &Self) {
        unsafe { self.mValue.reset(&mut self.mUnit) };
        self.mUnit = other.mUnit;
        self.mValue = other.mValue;
        unsafe { self.addref_if_calc(); }
    }

    #[inline]
    fn set<T: GeckoStyleCoordConvertible>(&mut self, val: T) {
        val.to_gecko_style_coord(&mut self.mUnit, &mut self.mValue);
    }

    #[inline]
    fn set_auto(&mut self) {
        unsafe { self.mValue.reset(&mut self.mUnit) };
        self.mUnit = nsStyleUnit::eStyleUnit_Auto;
        unsafe { *self.mValue.mInt.as_mut() = 0; }
    }
    #[inline]
    fn is_auto(&self) -> bool {
        self.mUnit == nsStyleUnit::eStyleUnit_Auto
    }

    #[inline]
    fn set_normal(&mut self) {
        unsafe { self.mValue.reset(&mut self.mUnit) };
        self.mUnit = nsStyleUnit::eStyleUnit_Normal;
        unsafe { *self.mValue.mInt.as_mut() = 0; }
    }
    #[inline]
    fn is_normal(&self) -> bool {
        self.mUnit == nsStyleUnit::eStyleUnit_Normal
    }

    #[inline]
    fn set_coord(&mut self, val: Au) {
        unsafe { self.mValue.reset(&mut self.mUnit) };
        self.mUnit = nsStyleUnit::eStyleUnit_Coord;
        unsafe { *self.mValue.mInt.as_mut() = val.0; }
    }
    #[inline]
    fn is_coord(&self) -> bool {
        self.mUnit == nsStyleUnit::eStyleUnit_Coord
    }
    #[inline]
    fn get_coord(&self) -> Au {
        debug_assert!(self.is_coord());
        Au(unsafe { *self.mValue.mInt.as_ref() })
    }

    #[inline]
    fn set_int(&mut self, val: i32) {
        unsafe { self.mValue.reset(&mut self.mUnit) };
        self.mUnit = nsStyleUnit::eStyleUnit_Integer;
        unsafe { *self.mValue.mInt.as_mut() = val; }
    }
    #[inline]
    fn is_int(&self) -> bool {
        self.mUnit == nsStyleUnit::eStyleUnit_Integer
    }
    #[inline]
    fn get_int(&self) -> i32 {
        debug_assert!(self.is_int());
        unsafe { *self.mValue.mInt.as_ref() }
    }

    #[inline]
    fn set_enum(&mut self, val: i32) {
        unsafe { self.mValue.reset(&mut self.mUnit) };
        self.mUnit = nsStyleUnit::eStyleUnit_Enumerated;
        unsafe { *self.mValue.mInt.as_mut() = val; }
    }
    #[inline]
    fn is_enum(&self) -> bool {
        self.mUnit == nsStyleUnit::eStyleUnit_Enumerated
    }
    #[inline]
    fn get_enum(&self) -> i32 {
        debug_assert!(self.is_enum());
        unsafe { *self.mValue.mInt.as_ref() }
    }

    #[inline]
    fn set_percent(&mut self, val: f32) {
        unsafe { self.mValue.reset(&mut self.mUnit) };
        self.mUnit = nsStyleUnit::eStyleUnit_Percent;
        unsafe { *self.mValue.mFloat.as_mut() = val; }
    }
    #[inline]
    fn is_percent(&self) -> bool {
        self.mUnit == nsStyleUnit::eStyleUnit_Percent
    }
    #[inline]
    fn get_percent(&self) -> f32 {
        debug_assert!(self.is_percent());
        unsafe { *self.mValue.mFloat.as_ref() }
    }

    #[inline]
    fn set_factor(&mut self, val: f32) {
        unsafe { self.mValue.reset(&mut self.mUnit) };
        self.mUnit = nsStyleUnit::eStyleUnit_Factor;
        unsafe { *self.mValue.mFloat.as_mut() = val; }
    }
    #[inline]
    fn is_factor(&self) -> bool {
        self.mUnit == nsStyleUnit::eStyleUnit_Factor
    }
    #[inline]
    fn get_factor(&self) -> f32 {
        debug_assert!(self.is_factor());
        unsafe { *self.mValue.mFloat.as_ref() }
    }
}

pub trait GeckoStyleCoordConvertible : Sized {
    fn to_gecko_style_coord(&self, unit: &mut nsStyleUnit, union: &mut nsStyleUnion);
    fn from_gecko_style_coord(unit: &nsStyleUnit, union: &nsStyleUnion) -> Option<Self>;
}

impl GeckoStyleCoordConvertible for LengthOrPercentage {
    fn to_gecko_style_coord(&self, unit: &mut nsStyleUnit, union: &mut nsStyleUnion) {
        unsafe { union.reset(unit) };
        match *self {
            LengthOrPercentage::Length(au) => {
                *unit = nsStyleUnit::eStyleUnit_Coord;
                unsafe { *union.mInt.as_mut() = au.0; }
            },
            LengthOrPercentage::Percentage(p) => {
                *unit = nsStyleUnit::eStyleUnit_Percent;
                unsafe { *union.mFloat.as_mut() = p; }
            },
            LengthOrPercentage::Calc(calc) => unsafe { union.set_calc_value(unit, calc.into()) },
        };
    }

    fn from_gecko_style_coord(unit: &nsStyleUnit, union: &nsStyleUnion) -> Option<Self> {
        match *unit {
            nsStyleUnit::eStyleUnit_Coord
                => Some(LengthOrPercentage::Length(Au(unsafe { *union.mInt.as_ref() }))),
            nsStyleUnit::eStyleUnit_Percent
                => Some(LengthOrPercentage::Percentage(unsafe { *union.mFloat.as_ref() })),
            nsStyleUnit::eStyleUnit_Calc
                => Some(LengthOrPercentage::Calc(unsafe { union.get_calc().into() })),
            _ => None,
        }
    }
}

impl GeckoStyleCoordConvertible for LengthOrPercentageOrAuto {
    fn to_gecko_style_coord(&self, unit: &mut nsStyleUnit, union: &mut nsStyleUnion) {
        unsafe { union.reset(unit) };
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
            LengthOrPercentageOrAuto::Calc(calc) => unsafe { union.set_calc_value(unit, calc.into()) },
        };
    }

    fn from_gecko_style_coord(unit: &nsStyleUnit, union: &nsStyleUnion) -> Option<Self> {
        match *unit {
            nsStyleUnit::eStyleUnit_Auto
                => Some(LengthOrPercentageOrAuto::Auto),
            nsStyleUnit::eStyleUnit_Coord
                => Some(LengthOrPercentageOrAuto::Length(Au(unsafe { *union.mInt.as_ref() }))),
            nsStyleUnit::eStyleUnit_Percent
                => Some(LengthOrPercentageOrAuto::Percentage(unsafe { *union.mFloat.as_ref() })),
            nsStyleUnit::eStyleUnit_Calc
                => Some(LengthOrPercentageOrAuto::Calc(unsafe { union.get_calc().into() })),
            _ => None,
        }
    }
}

impl GeckoStyleCoordConvertible for LengthOrPercentageOrNone {
    fn to_gecko_style_coord(&self, unit: &mut nsStyleUnit, union: &mut nsStyleUnion) {
        unsafe { union.reset(unit) };
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
            LengthOrPercentageOrNone::Calc(calc) => unsafe { union.set_calc_value(unit, calc.into()) },
        };
    }

    fn from_gecko_style_coord(unit: &nsStyleUnit, union: &nsStyleUnion) -> Option<Self> {
        match *unit {
            nsStyleUnit::eStyleUnit_None
                => Some(LengthOrPercentageOrNone::None),
            nsStyleUnit::eStyleUnit_Coord
                => Some(LengthOrPercentageOrNone::Length(Au(unsafe { *union.mInt.as_ref() }))),
            nsStyleUnit::eStyleUnit_Percent
                => Some(LengthOrPercentageOrNone::Percentage(unsafe { *union.mFloat.as_ref() })),
            nsStyleUnit::eStyleUnit_Calc
                => Some(LengthOrPercentageOrNone::Calc(unsafe { union.get_calc().into() })),
            _ => None,
        }
    }
}

impl<T: GeckoStyleCoordConvertible> GeckoStyleCoordConvertible for Option<T> {
    fn to_gecko_style_coord(&self, unit: &mut nsStyleUnit, union: &mut nsStyleUnion) {
        unsafe { union.reset(unit) };
        if let Some(ref me) = *self {
            me.to_gecko_style_coord(unit, union);
        } else {
            *unit = nsStyleUnit::eStyleUnit_None;
            unsafe { *union.mInt.as_mut() = 0; }
        }
    }

    fn from_gecko_style_coord(unit: &nsStyleUnit, union: &nsStyleUnion) -> Option<Self> {
        Some(T::from_gecko_style_coord(unit, union))
    }
}

impl GeckoStyleCoordConvertible for Angle {
    fn to_gecko_style_coord(&self,
                            unit: &mut nsStyleUnit,
                            union: &mut nsStyleUnion) {
        unsafe { union.reset(unit) };
        *unit = nsStyleUnit::eStyleUnit_Radian;
        unsafe { *union.mFloat.as_mut() = self.radians() };
    }

    fn from_gecko_style_coord(unit: &nsStyleUnit, union: &nsStyleUnion) -> Option<Self> {
        if *unit == nsStyleUnit::eStyleUnit_Radian {
            Some(Angle::from_radians(unsafe { *union.mFloat.as_ref() }))
        } else {
            None
        }
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
