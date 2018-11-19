/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

//! Different kind of helpers to interact with Gecko values.

use app_units::Au;
use crate::counter_style::{Symbol, Symbols};
use crate::gecko_bindings::structs::{self, nsStyleCoord, CounterStylePtr};
use crate::gecko_bindings::structs::{StyleGridTrackBreadth, StyleShapeRadius};
use crate::gecko_bindings::sugar::ns_style_coord::{CoordData, CoordDataMut, CoordDataValue};
use crate::media_queries::Device;
use crate::values::computed::basic_shape::ShapeRadius as ComputedShapeRadius;
use crate::values::computed::FlexBasis as ComputedFlexBasis;
use crate::values::computed::NonNegativeNumber;
use crate::values::computed::{Angle, ExtremumLength, Length, LengthOrPercentage};
use crate::values::computed::{LengthOrPercentageOrAuto, Percentage};
use crate::values::computed::{LengthOrPercentageOrNone, Number, NumberOrPercentage};
use crate::values::computed::{MaxLength as ComputedMaxLength, MozLength as ComputedMozLength};
use crate::values::computed::{NonNegativeLength, NonNegativeLengthOrPercentage};
use crate::values::generics::basic_shape::ShapeRadius;
use crate::values::generics::box_::Perspective;
use crate::values::generics::flex::FlexBasis;
use crate::values::generics::gecko::ScrollSnapPoint;
use crate::values::generics::grid::{TrackBreadth, TrackKeyword};
use crate::values::generics::length::{MaxLength, MozLength};
use crate::values::generics::{CounterStyleOrNone, NonNegative};
use crate::values::{Auto, Either, None_, Normal};
use crate::Atom;
use cssparser::RGBA;
use nsstring::{nsACString, nsCStr};
use std::cmp::max;

/// A trait that defines an interface to convert from and to `nsStyleCoord`s.
pub trait GeckoStyleCoordConvertible: Sized {
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

impl<A: GeckoStyleCoordConvertible, B: GeckoStyleCoordConvertible> GeckoStyleCoordConvertible
    for Either<A, B>
{
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

impl GeckoStyleCoordConvertible for ComputedFlexBasis {
    fn to_gecko_style_coord<T: CoordDataMut>(&self, coord: &mut T) {
        match *self {
            FlexBasis::Content => coord.set_value(CoordDataValue::Enumerated(
                structs::NS_STYLE_FLEX_BASIS_CONTENT,
            )),
            FlexBasis::Width(ref w) => w.to_gecko_style_coord(coord),
        }
    }

    fn from_gecko_style_coord<T: CoordData>(coord: &T) -> Option<Self> {
        if let Some(width) = ComputedMozLength::from_gecko_style_coord(coord) {
            return Some(FlexBasis::Width(width));
        }

        if let CoordDataValue::Enumerated(structs::NS_STYLE_FLEX_BASIS_CONTENT) = coord.as_value() {
            return Some(FlexBasis::Content);
        }

        None
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
            CoordDataValue::Percent(p) => Some(NumberOrPercentage::Percentage(Percentage(p))),
            _ => None,
        }
    }
}

impl GeckoStyleCoordConvertible for LengthOrPercentage {
    fn to_gecko_style_coord<T: CoordDataMut>(&self, coord: &mut T) {
        let value = match *self {
            LengthOrPercentage::Length(px) => CoordDataValue::Coord(px.to_i32_au()),
            LengthOrPercentage::Percentage(p) => CoordDataValue::Percent(p.0),
            LengthOrPercentage::Calc(calc) => CoordDataValue::Calc(calc.into()),
        };
        coord.set_value(value);
    }

    fn from_gecko_style_coord<T: CoordData>(coord: &T) -> Option<Self> {
        match coord.as_value() {
            CoordDataValue::Coord(coord) => Some(LengthOrPercentage::Length(Au(coord).into())),
            CoordDataValue::Percent(p) => Some(LengthOrPercentage::Percentage(Percentage(p))),
            CoordDataValue::Calc(calc) => Some(LengthOrPercentage::Calc(calc.into())),
            _ => None,
        }
    }
}

impl GeckoStyleCoordConvertible for NonNegativeLengthOrPercentage {
    fn to_gecko_style_coord<T: CoordDataMut>(&self, coord: &mut T) {
        self.0.to_gecko_style_coord(coord);
    }

    fn from_gecko_style_coord<T: CoordData>(coord: &T) -> Option<Self> {
        LengthOrPercentage::from_gecko_style_coord(coord).map(NonNegative::<LengthOrPercentage>)
    }
}

impl GeckoStyleCoordConvertible for Length {
    fn to_gecko_style_coord<T: CoordDataMut>(&self, coord: &mut T) {
        coord.set_value(CoordDataValue::Coord(self.to_i32_au()));
    }

    fn from_gecko_style_coord<T: CoordData>(coord: &T) -> Option<Self> {
        match coord.as_value() {
            CoordDataValue::Coord(coord) => Some(Au(coord).into()),
            _ => None,
        }
    }
}

impl GeckoStyleCoordConvertible for NonNegativeLength {
    fn to_gecko_style_coord<T: CoordDataMut>(&self, coord: &mut T) {
        self.0.to_gecko_style_coord(coord);
    }

    fn from_gecko_style_coord<T: CoordData>(coord: &T) -> Option<Self> {
        Length::from_gecko_style_coord(coord).map(NonNegative::<Length>)
    }
}

impl GeckoStyleCoordConvertible for NonNegativeNumber {
    fn to_gecko_style_coord<T: CoordDataMut>(&self, coord: &mut T) {
        self.0.to_gecko_style_coord(coord);
    }

    fn from_gecko_style_coord<T: CoordData>(coord: &T) -> Option<Self> {
        Number::from_gecko_style_coord(coord).map(NonNegative::<Number>)
    }
}

impl GeckoStyleCoordConvertible for LengthOrPercentageOrAuto {
    fn to_gecko_style_coord<T: CoordDataMut>(&self, coord: &mut T) {
        let value = match *self {
            LengthOrPercentageOrAuto::Length(px) => CoordDataValue::Coord(px.to_i32_au()),
            LengthOrPercentageOrAuto::Percentage(p) => CoordDataValue::Percent(p.0),
            LengthOrPercentageOrAuto::Auto => CoordDataValue::Auto,
            LengthOrPercentageOrAuto::Calc(calc) => CoordDataValue::Calc(calc.into()),
        };
        coord.set_value(value);
    }

    fn from_gecko_style_coord<T: CoordData>(coord: &T) -> Option<Self> {
        match coord.as_value() {
            CoordDataValue::Coord(coord) => {
                Some(LengthOrPercentageOrAuto::Length(Au(coord).into()))
            },
            CoordDataValue::Percent(p) => Some(LengthOrPercentageOrAuto::Percentage(Percentage(p))),
            CoordDataValue::Auto => Some(LengthOrPercentageOrAuto::Auto),
            CoordDataValue::Calc(calc) => Some(LengthOrPercentageOrAuto::Calc(calc.into())),
            _ => None,
        }
    }
}

impl GeckoStyleCoordConvertible for LengthOrPercentageOrNone {
    fn to_gecko_style_coord<T: CoordDataMut>(&self, coord: &mut T) {
        let value = match *self {
            LengthOrPercentageOrNone::Length(px) => CoordDataValue::Coord(px.to_i32_au()),
            LengthOrPercentageOrNone::Percentage(p) => CoordDataValue::Percent(p.0),
            LengthOrPercentageOrNone::None => CoordDataValue::None,
            LengthOrPercentageOrNone::Calc(calc) => CoordDataValue::Calc(calc.into()),
        };
        coord.set_value(value);
    }

    fn from_gecko_style_coord<T: CoordData>(coord: &T) -> Option<Self> {
        match coord.as_value() {
            CoordDataValue::Coord(coord) => {
                Some(LengthOrPercentageOrNone::Length(Au(coord).into()))
            },
            CoordDataValue::Percent(p) => Some(LengthOrPercentageOrNone::Percentage(Percentage(p))),
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
            TrackBreadth::Fr(fr) => coord.set_value(CoordDataValue::FlexFraction(fr)),
            TrackBreadth::Keyword(TrackKeyword::Auto) => coord.set_value(CoordDataValue::Auto),
            TrackBreadth::Keyword(TrackKeyword::MinContent) => coord.set_value(
                CoordDataValue::Enumerated(StyleGridTrackBreadth::MinContent as u32),
            ),
            TrackBreadth::Keyword(TrackKeyword::MaxContent) => coord.set_value(
                CoordDataValue::Enumerated(StyleGridTrackBreadth::MaxContent as u32),
            ),
        }
    }

    fn from_gecko_style_coord<T: CoordData>(coord: &T) -> Option<Self> {
        L::from_gecko_style_coord(coord)
            .map(TrackBreadth::Breadth)
            .or_else(|| match coord.as_value() {
                CoordDataValue::Enumerated(v) => {
                    if v == StyleGridTrackBreadth::MinContent as u32 {
                        Some(TrackBreadth::Keyword(TrackKeyword::MinContent))
                    } else if v == StyleGridTrackBreadth::MaxContent as u32 {
                        Some(TrackBreadth::Keyword(TrackKeyword::MaxContent))
                    } else {
                        None
                    }
                },
                CoordDataValue::FlexFraction(fr) => Some(TrackBreadth::Fr(fr)),
                CoordDataValue::Auto => Some(TrackBreadth::Keyword(TrackKeyword::Auto)),
                _ => L::from_gecko_style_coord(coord).map(TrackBreadth::Breadth),
            })
    }
}

impl GeckoStyleCoordConvertible for ComputedShapeRadius {
    fn to_gecko_style_coord<T: CoordDataMut>(&self, coord: &mut T) {
        match *self {
            ShapeRadius::ClosestSide => coord.set_value(CoordDataValue::Enumerated(
                StyleShapeRadius::ClosestSide as u32,
            )),
            ShapeRadius::FarthestSide => coord.set_value(CoordDataValue::Enumerated(
                StyleShapeRadius::FarthestSide as u32,
            )),
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
            },
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
            CoordDataValue::Degree(val) => Some(Angle::from_degrees(val)),
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
        use crate::gecko_bindings::structs::{
            NS_STYLE_WIDTH_AVAILABLE, NS_STYLE_WIDTH_FIT_CONTENT,
        };
        use crate::gecko_bindings::structs::{
            NS_STYLE_WIDTH_MAX_CONTENT, NS_STYLE_WIDTH_MIN_CONTENT,
        };
        coord.set_value(CoordDataValue::Enumerated(match *self {
            ExtremumLength::MozMaxContent => NS_STYLE_WIDTH_MAX_CONTENT,
            ExtremumLength::MozMinContent => NS_STYLE_WIDTH_MIN_CONTENT,
            ExtremumLength::MozFitContent => NS_STYLE_WIDTH_FIT_CONTENT,
            ExtremumLength::MozAvailable => NS_STYLE_WIDTH_AVAILABLE,
        }))
    }

    fn from_gecko_style_coord<T: CoordData>(coord: &T) -> Option<Self> {
        use crate::gecko_bindings::structs::{
            NS_STYLE_WIDTH_AVAILABLE, NS_STYLE_WIDTH_FIT_CONTENT,
        };
        use crate::gecko_bindings::structs::{
            NS_STYLE_WIDTH_MAX_CONTENT, NS_STYLE_WIDTH_MIN_CONTENT,
        };
        match coord.as_value() {
            CoordDataValue::Enumerated(NS_STYLE_WIDTH_MAX_CONTENT) => {
                Some(ExtremumLength::MozMaxContent)
            },
            CoordDataValue::Enumerated(NS_STYLE_WIDTH_MIN_CONTENT) => {
                Some(ExtremumLength::MozMinContent)
            },
            CoordDataValue::Enumerated(NS_STYLE_WIDTH_FIT_CONTENT) => {
                Some(ExtremumLength::MozFitContent)
            },
            CoordDataValue::Enumerated(NS_STYLE_WIDTH_AVAILABLE) => {
                Some(ExtremumLength::MozAvailable)
            },
            _ => None,
        }
    }
}

impl GeckoStyleCoordConvertible for ComputedMozLength {
    fn to_gecko_style_coord<T: CoordDataMut>(&self, coord: &mut T) {
        match *self {
            MozLength::LengthOrPercentageOrAuto(ref lopoa) => lopoa.to_gecko_style_coord(coord),
            MozLength::ExtremumLength(ref e) => e.to_gecko_style_coord(coord),
        }
    }

    fn from_gecko_style_coord<T: CoordData>(coord: &T) -> Option<Self> {
        LengthOrPercentageOrAuto::from_gecko_style_coord(coord)
            .map(MozLength::LengthOrPercentageOrAuto)
            .or_else(|| {
                ExtremumLength::from_gecko_style_coord(coord).map(MozLength::ExtremumLength)
            })
    }
}

impl GeckoStyleCoordConvertible for ComputedMaxLength {
    fn to_gecko_style_coord<T: CoordDataMut>(&self, coord: &mut T) {
        match *self {
            MaxLength::LengthOrPercentageOrNone(ref lopon) => lopon.to_gecko_style_coord(coord),
            MaxLength::ExtremumLength(ref e) => e.to_gecko_style_coord(coord),
        }
    }

    fn from_gecko_style_coord<T: CoordData>(coord: &T) -> Option<Self> {
        LengthOrPercentageOrNone::from_gecko_style_coord(coord)
            .map(MaxLength::LengthOrPercentageOrNone)
            .or_else(|| {
                ExtremumLength::from_gecko_style_coord(coord).map(MaxLength::ExtremumLength)
            })
    }
}

impl GeckoStyleCoordConvertible for ScrollSnapPoint<LengthOrPercentage> {
    fn to_gecko_style_coord<T: CoordDataMut>(&self, coord: &mut T) {
        match self.repeated() {
            None => coord.set_value(CoordDataValue::None),
            Some(l) => l.to_gecko_style_coord(coord),
        };
    }

    fn from_gecko_style_coord<T: CoordData>(coord: &T) -> Option<Self> {
        use crate::gecko_bindings::structs::root::nsStyleUnit;
        use crate::values::generics::gecko::ScrollSnapPoint;

        Some(match coord.unit() {
            nsStyleUnit::eStyleUnit_None => ScrollSnapPoint::None,
            _ => ScrollSnapPoint::Repeat(
                LengthOrPercentage::from_gecko_style_coord(coord)
                    .expect("coord could not convert to LengthOrPercentage"),
            ),
        })
    }
}

impl<L> GeckoStyleCoordConvertible for Perspective<L>
where
    L: GeckoStyleCoordConvertible,
{
    fn to_gecko_style_coord<T: CoordDataMut>(&self, coord: &mut T) {
        match *self {
            Perspective::None => coord.set_value(CoordDataValue::None),
            Perspective::Length(ref l) => l.to_gecko_style_coord(coord),
        };
    }

    fn from_gecko_style_coord<T: CoordData>(coord: &T) -> Option<Self> {
        use crate::gecko_bindings::structs::root::nsStyleUnit;

        if coord.unit() == nsStyleUnit::eStyleUnit_None {
            return Some(Perspective::None);
        }
        Some(Perspective::Length(L::from_gecko_style_coord(coord)?))
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
    RGBA::new(
        (color & 0xff) as u8,
        (color >> 8 & 0xff) as u8,
        (color >> 16 & 0xff) as u8,
        (color >> 24 & 0xff) as u8,
    )
}

/// Round `width` down to the nearest device pixel, but any non-zero value that
/// would round down to zero is clamped to 1 device pixel.  Used for storing
/// computed values of border-*-width and outline-width.
#[inline]
pub fn round_border_to_device_pixels(width: Au, au_per_device_px: Au) -> Au {
    if width == Au(0) {
        Au(0)
    } else {
        max(
            au_per_device_px,
            Au(width.0 / au_per_device_px.0 * au_per_device_px.0),
        )
    }
}

impl CounterStyleOrNone {
    /// Convert this counter style to a Gecko CounterStylePtr.
    pub fn to_gecko_value(self, gecko_value: &mut CounterStylePtr, device: &Device) {
        use crate::gecko_bindings::bindings::Gecko_SetCounterStyleToName as set_name;
        use crate::gecko_bindings::bindings::Gecko_SetCounterStyleToSymbols as set_symbols;
        let pres_context = device.pres_context();
        match self {
            CounterStyleOrNone::None => unsafe {
                set_name(gecko_value, atom!("none").into_addrefed(), pres_context);
            },
            CounterStyleOrNone::Name(name) => unsafe {
                set_name(gecko_value, name.0.into_addrefed(), pres_context);
            },
            CounterStyleOrNone::Symbols(symbols_type, symbols) => {
                let symbols: Vec<_> = symbols
                    .0
                    .iter()
                    .map(|symbol| match *symbol {
                        Symbol::String(ref s) => nsCStr::from(s),
                        Symbol::Ident(_) => unreachable!("Should not have identifier in symbols()"),
                    })
                    .collect();
                let symbols: Vec<_> = symbols
                    .iter()
                    .map(|symbol| symbol as &nsACString as *const _)
                    .collect();
                unsafe {
                    set_symbols(
                        gecko_value,
                        symbols_type.to_gecko_keyword(),
                        symbols.as_ptr(),
                        symbols.len() as u32,
                    )
                };
            },
        }
    }

    /// Convert Gecko CounterStylePtr to CounterStyleOrNone or String.
    pub fn from_gecko_value(gecko_value: &CounterStylePtr) -> Either<Self, String> {
        use crate::gecko_bindings::bindings;
        use crate::values::generics::SymbolsType;
        use crate::values::CustomIdent;

        let name = unsafe { bindings::Gecko_CounterStyle_GetName(gecko_value) };
        if !name.is_null() {
            let name = unsafe { Atom::from_raw(name) };
            if name == atom!("none") {
                Either::First(CounterStyleOrNone::None)
            } else {
                Either::First(CounterStyleOrNone::Name(CustomIdent(name)))
            }
        } else {
            let anonymous =
                unsafe { bindings::Gecko_CounterStyle_GetAnonymous(gecko_value).as_ref() }.unwrap();
            let symbols = &anonymous.mSymbols;
            if anonymous.mSingleString {
                debug_assert_eq!(symbols.len(), 1);
                Either::Second(symbols[0].to_string())
            } else {
                let symbol_type = SymbolsType::from_gecko_keyword(anonymous.mSystem as u32);
                let symbols = symbols
                    .iter()
                    .map(|gecko_symbol| Symbol::String(gecko_symbol.to_string()))
                    .collect();
                Either::First(CounterStyleOrNone::Symbols(symbol_type, Symbols(symbols)))
            }
        }
    }
}
