/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

//! Different kind of helpers to interact with Gecko values.

use app_units::Au;
use counter_style::Symbol;
use cssparser::RGBA;
use gecko_bindings::structs::{CounterStylePtr, nsStyleCoord};
use gecko_bindings::structs::{StyleGridTrackBreadth, StyleShapeRadius};
use gecko_bindings::sugar::ns_style_coord::{CoordData, CoordDataMut, CoordDataValue};
use nsstring::{nsACString, nsCString};
use std::cmp::max;
use values::{Auto, Either, ExtremumLength, None_, Normal};
use values::computed::{Angle, LengthOrPercentage, LengthOrPercentageOrAuto};
use values::computed::{LengthOrPercentageOrNone, Number, NumberOrPercentage};
use values::computed::{MaxLength, MozLength};
use values::computed::basic_shape::ShapeRadius as ComputedShapeRadius;
use values::generics::CounterStyleOrNone;
use values::generics::basic_shape::ShapeRadius;
use values::generics::grid::{TrackBreadth, TrackKeyword};
use values::specified::Percentage;

/// A trait that defines an interface to convert from and to `nsStyleCoord`s.
pub trait GeckoStyleCoordConvertible : Sized {
    /// Convert this to a `nsStyleCoord`.
    fn to_gecko_style_coord<T: CoordDataMut>(&self, coord: &mut T);
    /// Given a `nsStyleCoord`, try to get a value of this type..
    fn from_gecko_style_coord<T: CoordData>(coord: &T) -> Option<Self>;
}

impl nsStyleCoord {
    #[inline]
    /// Set this `nsStyleCoord` value to `val`.
    pub fn set<T: GeckoStyleCoordConvertible>(&mut self, val: T) {
        val.to_gecko_style_coord(self);
    }
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

impl GeckoStyleCoordConvertible for Percentage {
    fn to_gecko_style_coord<T: CoordDataMut>(&self, coord: &mut T) {
        coord.set_value(CoordDataValue::Percent(self.0));
    }

    fn from_gecko_style_coord<T: CoordData>(coord: &T) -> Option<Self> {
        match coord.as_value() {
            CoordDataValue::Percent(p) => Some(Percentage(p)),
            _ => None,
        }
    }
}

impl GeckoStyleCoordConvertible for NumberOrPercentage {
    fn to_gecko_style_coord<T: CoordDataMut>(&self, coord: &mut T) {
        match *self {
            NumberOrPercentage::Number(ref n) => n.to_gecko_style_coord(coord),
            NumberOrPercentage::Percentage(ref p) => p.to_gecko_style_coord(coord),
        }
    }

    fn from_gecko_style_coord<T: CoordData>(coord: &T) -> Option<Self> {
        match coord.as_value() {
            CoordDataValue::Factor(f) => Some(NumberOrPercentage::Number(f)),
            CoordDataValue::Percent(p) => {
                Some(NumberOrPercentage::Percentage(Percentage(p)))
            },
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

impl<L: GeckoStyleCoordConvertible> GeckoStyleCoordConvertible for TrackBreadth<L> {
    fn to_gecko_style_coord<T: CoordDataMut>(&self, coord: &mut T) {
        match *self {
            TrackBreadth::Breadth(ref lop) => lop.to_gecko_style_coord(coord),
            TrackBreadth::Flex(fr) => coord.set_value(CoordDataValue::FlexFraction(fr)),
            TrackBreadth::Keyword(TrackKeyword::Auto) => coord.set_value(CoordDataValue::Auto),
            TrackBreadth::Keyword(TrackKeyword::MinContent) =>
                coord.set_value(CoordDataValue::Enumerated(StyleGridTrackBreadth::MinContent as u32)),
            TrackBreadth::Keyword(TrackKeyword::MaxContent) =>
                coord.set_value(CoordDataValue::Enumerated(StyleGridTrackBreadth::MaxContent as u32)),
        }
    }

    fn from_gecko_style_coord<T: CoordData>(coord: &T) -> Option<Self> {
        L::from_gecko_style_coord(coord).map(TrackBreadth::Breadth).or_else(|| {
            match coord.as_value() {
                CoordDataValue::Enumerated(v) => {
                    if v == StyleGridTrackBreadth::MinContent as u32 {
                        Some(TrackBreadth::Keyword(TrackKeyword::MinContent))
                    } else if v == StyleGridTrackBreadth::MaxContent as u32 {
                        Some(TrackBreadth::Keyword(TrackKeyword::MaxContent))
                    } else {
                        None
                    }
                },
                CoordDataValue::FlexFraction(fr) => Some(TrackBreadth::Flex(fr)),
                CoordDataValue::Auto => Some(TrackBreadth::Keyword(TrackKeyword::Auto)),
                _ => L::from_gecko_style_coord(coord).map(TrackBreadth::Breadth),
            }
        })
    }
}

impl GeckoStyleCoordConvertible for ComputedShapeRadius {
    fn to_gecko_style_coord<T: CoordDataMut>(&self, coord: &mut T) {
        match *self {
            ShapeRadius::ClosestSide =>
                coord.set_value(CoordDataValue::Enumerated(StyleShapeRadius::ClosestSide as u32)),
            ShapeRadius::FarthestSide =>
                coord.set_value(CoordDataValue::Enumerated(StyleShapeRadius::FarthestSide as u32)),
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
        coord.set_value(CoordDataValue::from(*self));
    }

    fn from_gecko_style_coord<T: CoordData>(coord: &T) -> Option<Self> {
        match coord.as_value() {
            CoordDataValue::Degree(val) => Some(Angle::Degree(val)),
            CoordDataValue::Grad(val) => Some(Angle::Gradian(val)),
            CoordDataValue::Radian(val) => Some(Angle::Radian(val)),
            CoordDataValue::Turn(val) => Some(Angle::Turn(val)),
            _ => None,
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

impl GeckoStyleCoordConvertible for None_ {
    fn to_gecko_style_coord<T: CoordDataMut>(&self, coord: &mut T) {
        coord.set_value(CoordDataValue::None)
    }

    fn from_gecko_style_coord<T: CoordData>(coord: &T) -> Option<Self> {
        if let CoordDataValue::None = coord.as_value() {
            Some(None_)
        } else {
            None
        }
    }
}

impl GeckoStyleCoordConvertible for Normal {
    fn to_gecko_style_coord<T: CoordDataMut>(&self, coord: &mut T) {
        coord.set_value(CoordDataValue::Normal)
    }

    fn from_gecko_style_coord<T: CoordData>(coord: &T) -> Option<Self> {
        if let CoordDataValue::Normal = coord.as_value() {
            Some(Normal)
        } else {
            None
        }
    }
}

impl GeckoStyleCoordConvertible for ExtremumLength {
    fn to_gecko_style_coord<T: CoordDataMut>(&self, coord: &mut T) {
        use gecko_bindings::structs::{NS_STYLE_WIDTH_AVAILABLE, NS_STYLE_WIDTH_FIT_CONTENT};
        use gecko_bindings::structs::{NS_STYLE_WIDTH_MAX_CONTENT, NS_STYLE_WIDTH_MIN_CONTENT};
        coord.set_value(CoordDataValue::Enumerated(
            match *self {
                ExtremumLength::MaxContent => NS_STYLE_WIDTH_MAX_CONTENT,
                ExtremumLength::MinContent => NS_STYLE_WIDTH_MIN_CONTENT,
                ExtremumLength::FitContent => NS_STYLE_WIDTH_FIT_CONTENT,
                ExtremumLength::FillAvailable => NS_STYLE_WIDTH_AVAILABLE,
            }
        ))
    }

    fn from_gecko_style_coord<T: CoordData>(coord: &T) -> Option<Self> {
        use gecko_bindings::structs::{NS_STYLE_WIDTH_AVAILABLE, NS_STYLE_WIDTH_FIT_CONTENT};
        use gecko_bindings::structs::{NS_STYLE_WIDTH_MAX_CONTENT, NS_STYLE_WIDTH_MIN_CONTENT};
        match coord.as_value() {
            CoordDataValue::Enumerated(NS_STYLE_WIDTH_MAX_CONTENT) =>
                Some(ExtremumLength::MaxContent),
            CoordDataValue::Enumerated(NS_STYLE_WIDTH_MIN_CONTENT) =>
                Some(ExtremumLength::MinContent),
            CoordDataValue::Enumerated(NS_STYLE_WIDTH_FIT_CONTENT) =>
                Some(ExtremumLength::FitContent),
            CoordDataValue::Enumerated(NS_STYLE_WIDTH_AVAILABLE) => Some(ExtremumLength::FillAvailable),
            _ => None,
        }
    }
}

impl GeckoStyleCoordConvertible for MozLength {
    fn to_gecko_style_coord<T: CoordDataMut>(&self, coord: &mut T) {
        match *self {
            MozLength::LengthOrPercentageOrAuto(ref lopoa) => lopoa.to_gecko_style_coord(coord),
            MozLength::ExtremumLength(ref e) => e.to_gecko_style_coord(coord),
        }
    }

    fn from_gecko_style_coord<T: CoordData>(coord: &T) -> Option<Self> {
        LengthOrPercentageOrAuto::from_gecko_style_coord(coord).map(MozLength::LengthOrPercentageOrAuto)
            .or_else(|| ExtremumLength::from_gecko_style_coord(coord).map(MozLength::ExtremumLength))
    }
}

impl GeckoStyleCoordConvertible for MaxLength {
    fn to_gecko_style_coord<T: CoordDataMut>(&self, coord: &mut T) {
        match *self {
            MaxLength::LengthOrPercentageOrNone(ref lopon) => lopon.to_gecko_style_coord(coord),
            MaxLength::ExtremumLength(ref e) => e.to_gecko_style_coord(coord),
        }
    }

    fn from_gecko_style_coord<T: CoordData>(coord: &T) -> Option<Self> {
        LengthOrPercentageOrNone::from_gecko_style_coord(coord).map(MaxLength::LengthOrPercentageOrNone)
            .or_else(|| ExtremumLength::from_gecko_style_coord(coord).map(MaxLength::ExtremumLength))
    }
}

/// Convert a given RGBA value to `nscolor`.
pub fn convert_rgba_to_nscolor(rgba: &RGBA) -> u32 {
    ((rgba.alpha as u32) << 24) |
    ((rgba.blue as u32) << 16) |
    ((rgba.green as u32) << 8) |
    (rgba.red as u32)
}

/// Convert a given `nscolor` to a Servo RGBA value.
pub fn convert_nscolor_to_rgba(color: u32) -> RGBA {
    RGBA::new((color & 0xff) as u8,
              (color >> 8 & 0xff) as u8,
              (color >> 16 & 0xff) as u8,
              (color >> 24 & 0xff) as u8)
}

/// Round `width` down to the nearest device pixel, but any non-zero value that
/// would round down to zero is clamped to 1 device pixel.  Used for storing
/// computed values of border-*-width and outline-width.
#[inline]
pub fn round_border_to_device_pixels(width: Au, au_per_device_px: Au) -> Au {
    if width == Au(0) {
        Au(0)
    } else {
        max(au_per_device_px, Au(width.0 / au_per_device_px.0 * au_per_device_px.0))
    }
}

impl CounterStyleOrNone {
    /// Convert this counter style to a Gecko CounterStylePtr.
    pub fn to_gecko_value(self, gecko_value: &mut CounterStylePtr) {
        use gecko_bindings::bindings::Gecko_SetCounterStyleToName as set_name;
        use gecko_bindings::bindings::Gecko_SetCounterStyleToSymbols as set_symbols;
        match self {
            CounterStyleOrNone::None => unsafe {
                set_name(gecko_value, atom!("none").into_addrefed());
            },
            CounterStyleOrNone::Name(name) => unsafe {
                set_name(gecko_value, name.0.into_addrefed());
            },
            CounterStyleOrNone::Symbols(symbols_type, symbols) => {
                let symbols: Vec<_> = symbols.0.iter().map(|symbol| match *symbol {
                    Symbol::String(ref s) => nsCString::from(s),
                    Symbol::Ident(_) => unreachable!("Should not have identifier in symbols()"),
                }).collect();
                let symbols: Vec<_> = symbols.iter()
                    .map(|symbol| symbol as &nsACString as *const _)
                    .collect();
                unsafe { set_symbols(gecko_value, symbols_type.to_gecko_keyword(),
                                     symbols.as_ptr(), symbols.len() as u32) };
            }
        }
    }
}
