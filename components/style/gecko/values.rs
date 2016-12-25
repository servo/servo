/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

use app_units::Au;
use cssparser::RGBA;
use gecko_bindings::structs::{nsStyleCoord, StyleShapeRadius};
use gecko_bindings::sugar::ns_style_coord::{CoordData, CoordDataMut, CoordDataValue};
use std::cmp::max;
use values::{Auto, Either};
use values::computed::{Angle, LengthOrPercentageOrNone, Number};
use values::computed::{LengthOrPercentage, LengthOrPercentageOrAuto};
use values::computed::basic_shape::ShapeRadius;

pub trait StyleCoordHelpers {
    fn set<T: GeckoStyleCoordConvertible>(&mut self, val: T);
}

impl StyleCoordHelpers for nsStyleCoord {
    #[inline]
    fn set<T: GeckoStyleCoordConvertible>(&mut self, val: T) {
        val.to_gecko_style_coord(self);
    }
}

pub trait GeckoStyleCoordConvertible : Sized {
    fn to_gecko_style_coord<T: CoordDataMut>(&self, coord: &mut T);
    fn from_gecko_style_coord<T: CoordData>(coord: &T) -> Option<Self>;
}

impl<A: GeckoStyleCoordConvertible, B: GeckoStyleCoordConvertible> GeckoStyleCoordConvertible for Either<A, B> {
    fn to_gecko_style_coord<T: CoordDataMut>(&self, coord: &mut T) {
        match *self {
            Either::First(ref v) => v.to_gecko_style_coord(coord),
            Either::Second(ref v) => v.to_gecko_style_coord(coord),
        }
    }

    fn from_gecko_style_coord<T: CoordData>(coord: &T) -> Option<Self> {
        A::from_gecko_style_coord(coord)
          .map(Either::First)
          .or_else(|| B::from_gecko_style_coord(coord).map(Either::Second))
    }
}

impl GeckoStyleCoordConvertible for Number {
    fn to_gecko_style_coord<T: CoordDataMut>(&self, coord: &mut T) {
        coord.set_value(CoordDataValue::Factor(*self));
    }

    fn from_gecko_style_coord<T: CoordData>(coord: &T) -> Option<Self> {
        match coord.as_value() {
            CoordDataValue::Factor(f) => Some(f),
            _ => None,
        }
    }
}

impl GeckoStyleCoordConvertible for LengthOrPercentage {
    fn to_gecko_style_coord<T: CoordDataMut>(&self, coord: &mut T) {
        let value = match *self {
            LengthOrPercentage::Length(au) => CoordDataValue::Coord(au.0),
            LengthOrPercentage::Percentage(p) => CoordDataValue::Percent(p),
            LengthOrPercentage::Calc(calc) => CoordDataValue::Calc(calc.into()),
        };
        coord.set_value(value);
    }

    fn from_gecko_style_coord<T: CoordData>(coord: &T) -> Option<Self> {
        match coord.as_value() {
            CoordDataValue::Coord(coord) => Some(LengthOrPercentage::Length(Au(coord))),
            CoordDataValue::Percent(p) => Some(LengthOrPercentage::Percentage(p)),
            CoordDataValue::Calc(calc) => Some(LengthOrPercentage::Calc(calc.into())),
            _ => None,
        }
    }
}

impl GeckoStyleCoordConvertible for Au {
    fn to_gecko_style_coord<T: CoordDataMut>(&self, coord: &mut T) {
        coord.set_value(CoordDataValue::Coord(self.0));
    }

    fn from_gecko_style_coord<T: CoordData>(coord: &T) -> Option<Self> {
        match coord.as_value() {
            CoordDataValue::Coord(coord) => Some(Au(coord)),
            _ => None,
        }
    }
}

impl GeckoStyleCoordConvertible for LengthOrPercentageOrAuto {
    fn to_gecko_style_coord<T: CoordDataMut>(&self, coord: &mut T) {
        let value = match *self {
            LengthOrPercentageOrAuto::Length(au) => CoordDataValue::Coord(au.0),
            LengthOrPercentageOrAuto::Percentage(p) => CoordDataValue::Percent(p),
            LengthOrPercentageOrAuto::Auto => CoordDataValue::Auto,
            LengthOrPercentageOrAuto::Calc(calc) => CoordDataValue::Calc(calc.into()),
        };
        coord.set_value(value);
    }

    fn from_gecko_style_coord<T: CoordData>(coord: &T) -> Option<Self> {
        match coord.as_value() {
            CoordDataValue::Coord(coord) => Some(LengthOrPercentageOrAuto::Length(Au(coord))),
            CoordDataValue::Percent(p) => Some(LengthOrPercentageOrAuto::Percentage(p)),
            CoordDataValue::Auto => Some(LengthOrPercentageOrAuto::Auto),
            CoordDataValue::Calc(calc) => Some(LengthOrPercentageOrAuto::Calc(calc.into())),
            _ => None,
        }
    }
}

impl GeckoStyleCoordConvertible for LengthOrPercentageOrNone {
    fn to_gecko_style_coord<T: CoordDataMut>(&self, coord: &mut T) {
        let value = match *self {
            LengthOrPercentageOrNone::Length(au) => CoordDataValue::Coord(au.0),
            LengthOrPercentageOrNone::Percentage(p) => CoordDataValue::Percent(p),
            LengthOrPercentageOrNone::None => CoordDataValue::None,
            LengthOrPercentageOrNone::Calc(calc) => CoordDataValue::Calc(calc.into()),
        };
        coord.set_value(value);
    }

    fn from_gecko_style_coord<T: CoordData>(coord: &T) -> Option<Self> {
        match coord.as_value() {
            CoordDataValue::Coord(coord) => Some(LengthOrPercentageOrNone::Length(Au(coord))),
            CoordDataValue::Percent(p) => Some(LengthOrPercentageOrNone::Percentage(p)),
            CoordDataValue::None => Some(LengthOrPercentageOrNone::None),
            CoordDataValue::Calc(calc) => Some(LengthOrPercentageOrNone::Calc(calc.into())),
            _ => None,
        }
    }
}

impl GeckoStyleCoordConvertible for ShapeRadius {
    fn to_gecko_style_coord<T: CoordDataMut>(&self, coord: &mut T) {
        match *self {
            ShapeRadius::ClosestSide => {
                coord.set_value(CoordDataValue::Enumerated(StyleShapeRadius::ClosestSide as u32))
            }
            ShapeRadius::FarthestSide => {
                coord.set_value(CoordDataValue::Enumerated(StyleShapeRadius::FarthestSide as u32))
            }
            ShapeRadius::Length(lop) => lop.to_gecko_style_coord(coord),
        }
    }

    fn from_gecko_style_coord<T: CoordData>(coord: &T) -> Option<Self> {
        match coord.as_value() {
            CoordDataValue::Enumerated(v) => {
                if v == StyleShapeRadius::ClosestSide as u32 {
                    Some(ShapeRadius::ClosestSide)
                } else if v == StyleShapeRadius::FarthestSide as u32 {
                    Some(ShapeRadius::FarthestSide)
                } else {
                    None
                }
            }
            _ => LengthOrPercentage::from_gecko_style_coord(coord).map(ShapeRadius::Length),
        }
    }
}

impl<T: GeckoStyleCoordConvertible> GeckoStyleCoordConvertible for Option<T> {
    fn to_gecko_style_coord<U: CoordDataMut>(&self, coord: &mut U) {
        if let Some(ref me) = *self {
            me.to_gecko_style_coord(coord);
        } else {
            coord.set_value(CoordDataValue::None);
        }
    }

    fn from_gecko_style_coord<U: CoordData>(coord: &U) -> Option<Self> {
        Some(T::from_gecko_style_coord(coord))
    }
}

impl GeckoStyleCoordConvertible for Angle {
    fn to_gecko_style_coord<T: CoordDataMut>(&self, coord: &mut T) {
        coord.set_value(CoordDataValue::Radian(self.radians()))
    }

    fn from_gecko_style_coord<T: CoordData>(coord: &T) -> Option<Self> {
        if let CoordDataValue::Radian(r) = coord.as_value() {
            Some(Angle::from_radians(r))
            // XXXManishearth should this handle Degree too?
        } else {
            None
        }
    }
}

impl GeckoStyleCoordConvertible for Auto {
    fn to_gecko_style_coord<T: CoordDataMut>(&self, coord: &mut T) {
        coord.set_value(CoordDataValue::Auto)
    }

    fn from_gecko_style_coord<T: CoordData>(coord: &T) -> Option<Self> {
        if let CoordDataValue::Auto = coord.as_value() {
            Some(Auto)
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
