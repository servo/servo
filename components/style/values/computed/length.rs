/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! `<length>` computed values, and related ones.

use app_units::{Au, AU_PER_PX};
use ordered_float::NotNaN;
use std::fmt;
use style_traits::ToCss;
use style_traits::values::specified::AllowedLengthType;
use super::{Number, ToComputedValue, Context, Percentage};
use values::{Auto, CSSFloat, Either, ExtremumLength, None_, Normal, specified};
use values::animated::ToAnimatedZero;
use values::computed::{NonNegativeAu, NonNegativeNumber};
use values::distance::{ComputeSquaredDistance, SquaredDistance};
use values::generics::NonNegative;
use values::specified::length::{AbsoluteLength, FontBaseSize, FontRelativeLength};
use values::specified::length::ViewportPercentageLength;

pub use super::image::Image;
pub use values::specified::{Angle, BorderStyle, Time, UrlOrNone};

impl ToComputedValue for specified::NoCalcLength {
    type ComputedValue = Au;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Au {
        match *self {
            specified::NoCalcLength::Absolute(length) =>
                length.to_computed_value(context),
            specified::NoCalcLength::FontRelative(length) =>
                length.to_computed_value(context, FontBaseSize::CurrentStyle),
            specified::NoCalcLength::ViewportPercentage(length) =>
                length.to_computed_value(context.viewport_size()),
            specified::NoCalcLength::ServoCharacterWidth(length) =>
                length.to_computed_value(context.style().get_font().clone_font_size().0),
            #[cfg(feature = "gecko")]
            specified::NoCalcLength::Physical(length) =>
                length.to_computed_value(context),
        }
    }

    #[inline]
    fn from_computed_value(computed: &Au) -> Self {
        specified::NoCalcLength::Absolute(AbsoluteLength::Px(computed.to_f32_px()))
    }
}

impl ToComputedValue for specified::Length {
    type ComputedValue = Au;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Au {
        match *self {
            specified::Length::NoCalc(l) => l.to_computed_value(context),
            specified::Length::Calc(ref calc) => calc.to_computed_value(context).length(),
        }
    }

    #[inline]
    fn from_computed_value(computed: &Au) -> Self {
        specified::Length::NoCalc(specified::NoCalcLength::from_computed_value(computed))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub struct CalcLengthOrPercentage {
    pub clamping_mode: AllowedLengthType,
    length: Au,
    pub percentage: Option<Percentage>,
}

impl ToAnimatedZero for CalcLengthOrPercentage {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        Ok(CalcLengthOrPercentage {
            clamping_mode: self.clamping_mode,
            length: self.length.to_animated_zero()?,
            percentage: self.percentage.to_animated_zero()?,
        })
    }
}

impl ComputeSquaredDistance for CalcLengthOrPercentage {
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        // FIXME(nox): This looks incorrect to me, to add a distance between lengths
        // with a distance between percentages.
        Ok(
            self.unclamped_length().to_f64_px().compute_squared_distance(
                &other.unclamped_length().to_f64_px())? +
            self.percentage().compute_squared_distance(&other.percentage())?,
        )
    }
}

impl CalcLengthOrPercentage {
    /// Returns a new `CalcLengthOrPercentage`.
    #[inline]
    pub fn new(length: Au, percentage: Option<Percentage>) -> Self {
        Self::with_clamping_mode(length, percentage, AllowedLengthType::All)
    }

    /// Returns a new `CalcLengthOrPercentage` with a specific clamping mode.
    #[inline]
    pub fn with_clamping_mode(length: Au,
                              percentage: Option<Percentage>,
                              clamping_mode: AllowedLengthType)
                              -> Self {
        Self {
            clamping_mode: clamping_mode,
            length: length,
            percentage: percentage,
        }
    }

    /// Returns this `calc()` as a `<length>`.
    ///
    /// Panics in debug mode if a percentage is present in the expression.
    #[inline]
    pub fn length(&self) -> Au {
        debug_assert!(self.percentage.is_none());
        self.clamping_mode.clamp(self.length)
    }

    /// Returns the `<length>` component of this `calc()`, unclamped.
    #[inline]
    pub fn unclamped_length(&self) -> Au {
        self.length
    }

    #[inline]
    #[allow(missing_docs)]
    pub fn percentage(&self) -> CSSFloat {
        self.percentage.map_or(0., |p| p.0)
    }

    /// If there are special rules for computing percentages in a value (e.g. the height property),
    /// they apply whenever a calc() expression contains percentages.
    pub fn to_used_value(&self, container_len: Option<Au>) -> Option<Au> {
        match (container_len, self.percentage) {
            (Some(len), Some(percent)) => {
                Some(self.clamping_mode.clamp(self.length + len.scale_by(percent.0)))
            },
            (_, None) => Some(self.length()),
            _ => None,
        }
    }
}

impl From<LengthOrPercentage> for CalcLengthOrPercentage {
    fn from(len: LengthOrPercentage) -> CalcLengthOrPercentage {
        match len {
            LengthOrPercentage::Percentage(this) => {
                CalcLengthOrPercentage::new(Au(0), Some(this))
            }
            LengthOrPercentage::Length(this) => {
                CalcLengthOrPercentage::new(this, None)
            }
            LengthOrPercentage::Calc(this) => {
                this
            }
        }
    }
}

impl From<LengthOrPercentageOrAuto> for Option<CalcLengthOrPercentage> {
    fn from(len: LengthOrPercentageOrAuto) -> Option<CalcLengthOrPercentage> {
        match len {
            LengthOrPercentageOrAuto::Percentage(this) => {
                Some(CalcLengthOrPercentage::new(Au(0), Some(this)))
            }
            LengthOrPercentageOrAuto::Length(this) => {
                Some(CalcLengthOrPercentage::new(this, None))
            }
            LengthOrPercentageOrAuto::Calc(this) => {
                Some(this)
            }
            LengthOrPercentageOrAuto::Auto => {
                None
            }
        }
    }
}

impl From<LengthOrPercentageOrNone> for Option<CalcLengthOrPercentage> {
    fn from(len: LengthOrPercentageOrNone) -> Option<CalcLengthOrPercentage> {
        match len {
            LengthOrPercentageOrNone::Percentage(this) => {
                Some(CalcLengthOrPercentage::new(Au(0), Some(this)))
            }
            LengthOrPercentageOrNone::Length(this) => {
                Some(CalcLengthOrPercentage::new(this, None))
            }
            LengthOrPercentageOrNone::Calc(this) => {
                Some(this)
            }
            LengthOrPercentageOrNone::None => {
                None
            }
        }
    }
}

impl ToCss for CalcLengthOrPercentage {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        use num_traits::Zero;

        let (length, percentage) = match (self.length, self.percentage) {
            (l, None) => return l.to_css(dest),
            (l, Some(p)) if l == Au(0) => return p.to_css(dest),
            (l, Some(p)) => (l, p),
        };

        dest.write_str("calc(")?;
        percentage.to_css(dest)?;

        dest.write_str(if length < Zero::zero() { " - " } else { " + " })?;
        length.abs().to_css(dest)?;

        dest.write_str(")")
    }
}

impl specified::CalcLengthOrPercentage {
    /// Compute the value, zooming any absolute units by the zoom function.
    fn to_computed_value_with_zoom<F>(&self, context: &Context, zoom_fn: F) -> CalcLengthOrPercentage
        where F: Fn(Au) -> Au {
        let mut length = Au(0);

        if let Some(absolute) = self.absolute {
            length += zoom_fn(absolute);
        }

        for val in &[self.vw.map(ViewportPercentageLength::Vw),
                     self.vh.map(ViewportPercentageLength::Vh),
                     self.vmin.map(ViewportPercentageLength::Vmin),
                     self.vmax.map(ViewportPercentageLength::Vmax)] {
            if let Some(val) = *val {
                length += val.to_computed_value(context.viewport_size());
            }
        }

        for val in &[self.ch.map(FontRelativeLength::Ch),
                     self.em.map(FontRelativeLength::Em),
                     self.ex.map(FontRelativeLength::Ex),
                     self.rem.map(FontRelativeLength::Rem)] {
            if let Some(val) = *val {
                length += val.to_computed_value(context, FontBaseSize::CurrentStyle);
            }
        }

        CalcLengthOrPercentage {
            clamping_mode: self.clamping_mode,
            length: length,
            percentage: self.percentage,
        }
    }

    /// Compute font-size or line-height taking into account text-zoom if necessary.
    pub fn to_computed_value_zoomed(&self, context: &Context) -> CalcLengthOrPercentage {
        self.to_computed_value_with_zoom(context, |abs| context.maybe_zoom_text(abs.into()).0)
    }
}

impl ToComputedValue for specified::CalcLengthOrPercentage {
    type ComputedValue = CalcLengthOrPercentage;

    fn to_computed_value(&self, context: &Context) -> CalcLengthOrPercentage {
        self.to_computed_value_with_zoom(context, |abs| abs)
    }

    #[inline]
    fn from_computed_value(computed: &CalcLengthOrPercentage) -> Self {
        specified::CalcLengthOrPercentage {
            clamping_mode: computed.clamping_mode,
            absolute: Some(computed.length),
            percentage: computed.percentage,
            ..Default::default()
        }
    }
}

#[allow(missing_docs)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Copy, PartialEq, ToAnimatedZero, ToCss)]
pub enum LengthOrPercentage {
    Length(Au),
    Percentage(Percentage),
    Calc(CalcLengthOrPercentage),
}

impl ComputeSquaredDistance for LengthOrPercentage {
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        match (self, other) {
            (&LengthOrPercentage::Length(ref this), &LengthOrPercentage::Length(ref other)) => {
                this.compute_squared_distance(other)
            },
            (&LengthOrPercentage::Percentage(ref this), &LengthOrPercentage::Percentage(ref other)) => {
                this.compute_squared_distance(other)
            },
            (this, other) => {
                CalcLengthOrPercentage::compute_squared_distance(&(*this).into(), &(*other).into())
            }
        }
    }
}

impl From<Au> for LengthOrPercentage {
    #[inline]
    fn from(length: Au) -> Self {
        LengthOrPercentage::Length(length)
    }
}

impl LengthOrPercentage {
    #[inline]
    #[allow(missing_docs)]
    pub fn zero() -> LengthOrPercentage {
        LengthOrPercentage::Length(Au(0))
    }

    #[inline]
    /// 1px length value for SVG defaults
    pub fn one() -> LengthOrPercentage {
        LengthOrPercentage::Length(Au(AU_PER_PX))
    }

    /// Returns true if the computed value is absolute 0 or 0%.
    ///
    /// (Returns false for calc() values, even if ones that may resolve to zero.)
    #[inline]
    pub fn is_definitely_zero(&self) -> bool {
        use self::LengthOrPercentage::*;
        match *self {
            Length(Au(0)) => true,
            Percentage(p) => p.0 == 0.0,
            Length(_) | Calc(_) => false
        }
    }

    #[allow(missing_docs)]
    pub fn to_hash_key(&self) -> (Au, NotNaN<f32>) {
        use self::LengthOrPercentage::*;
        match *self {
            Length(l) => (l, NotNaN::new(0.0).unwrap()),
            Percentage(p) => (Au(0), NotNaN::new(p.0).unwrap()),
            Calc(c) => (c.unclamped_length(), NotNaN::new(c.percentage()).unwrap()),
        }
    }

    /// Returns the used value.
    pub fn to_used_value(&self, containing_length: Au) -> Au {
        match *self {
            LengthOrPercentage::Length(length) => length,
            LengthOrPercentage::Percentage(p) => containing_length.scale_by(p.0),
            LengthOrPercentage::Calc(ref calc) => {
                calc.to_used_value(Some(containing_length)).unwrap()
            },
        }
    }

    /// Returns the clamped non-negative values.
    #[inline]
    pub fn clamp_to_non_negative(self) -> Self {
        match self {
            LengthOrPercentage::Length(length) => {
                LengthOrPercentage::Length(Au(::std::cmp::max(length.0, 0)))
            },
            LengthOrPercentage::Percentage(percentage) => {
                LengthOrPercentage::Percentage(Percentage(percentage.0.max(0.)))
            },
            _ => self
        }
    }
}

impl fmt::Debug for LengthOrPercentage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            LengthOrPercentage::Length(length) => write!(f, "{:?}", length),
            LengthOrPercentage::Percentage(percentage) => write!(f, "{}%", percentage.0 * 100.),
            LengthOrPercentage::Calc(calc) => write!(f, "{:?}", calc),
        }
    }
}

impl ToComputedValue for specified::LengthOrPercentage {
    type ComputedValue = LengthOrPercentage;

    fn to_computed_value(&self, context: &Context) -> LengthOrPercentage {
        match *self {
            specified::LengthOrPercentage::Length(ref value) => {
                LengthOrPercentage::Length(value.to_computed_value(context))
            }
            specified::LengthOrPercentage::Percentage(value) => {
                LengthOrPercentage::Percentage(value)
            }
            specified::LengthOrPercentage::Calc(ref calc) => {
                LengthOrPercentage::Calc(calc.to_computed_value(context))
            }
        }
    }

    fn from_computed_value(computed: &LengthOrPercentage) -> Self {
        match *computed {
            LengthOrPercentage::Length(value) => {
                specified::LengthOrPercentage::Length(
                    ToComputedValue::from_computed_value(&value)
                )
            }
            LengthOrPercentage::Percentage(value) => {
                specified::LengthOrPercentage::Percentage(value)
            }
            LengthOrPercentage::Calc(ref calc) => {
                specified::LengthOrPercentage::Calc(
                    Box::new(ToComputedValue::from_computed_value(calc))
                )
            }
        }
    }
}

#[allow(missing_docs)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Copy, PartialEq, ToCss)]
pub enum LengthOrPercentageOrAuto {
    Length(Au),
    Percentage(Percentage),
    Auto,
    Calc(CalcLengthOrPercentage),
}

impl ComputeSquaredDistance for LengthOrPercentageOrAuto {
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        match (self, other) {
            (&LengthOrPercentageOrAuto::Length(ref this), &LengthOrPercentageOrAuto::Length(ref other)) => {
                this.compute_squared_distance(other)
            },
            (&LengthOrPercentageOrAuto::Percentage(ref this), &LengthOrPercentageOrAuto::Percentage(ref other)) => {
                this.compute_squared_distance(other)
            },
            (this, other) => {
                <Option<CalcLengthOrPercentage>>::compute_squared_distance(&(*this).into(), &(*other).into())
            }
        }
    }
}

impl LengthOrPercentageOrAuto {
    /// Returns true if the computed value is absolute 0 or 0%.
    ///
    /// (Returns false for calc() values, even if ones that may resolve to zero.)
    #[inline]
    pub fn is_definitely_zero(&self) -> bool {
        use self::LengthOrPercentageOrAuto::*;
        match *self {
            Length(Au(0)) => true,
            Percentage(p) => p.0 == 0.0,
            Length(_) | Calc(_) | Auto => false
        }
    }
}

impl fmt::Debug for LengthOrPercentageOrAuto {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            LengthOrPercentageOrAuto::Length(length) => write!(f, "{:?}", length),
            LengthOrPercentageOrAuto::Percentage(percentage) => write!(f, "{}%", percentage.0 * 100.),
            LengthOrPercentageOrAuto::Auto => write!(f, "auto"),
            LengthOrPercentageOrAuto::Calc(calc) => write!(f, "{:?}", calc),
        }
    }
}

impl ToComputedValue for specified::LengthOrPercentageOrAuto {
    type ComputedValue = LengthOrPercentageOrAuto;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> LengthOrPercentageOrAuto {
        match *self {
            specified::LengthOrPercentageOrAuto::Length(ref value) => {
                LengthOrPercentageOrAuto::Length(value.to_computed_value(context))
            }
            specified::LengthOrPercentageOrAuto::Percentage(value) => {
                LengthOrPercentageOrAuto::Percentage(value)
            }
            specified::LengthOrPercentageOrAuto::Auto => {
                LengthOrPercentageOrAuto::Auto
            }
            specified::LengthOrPercentageOrAuto::Calc(ref calc) => {
                LengthOrPercentageOrAuto::Calc(calc.to_computed_value(context))
            }
        }
    }

    #[inline]
    fn from_computed_value(computed: &LengthOrPercentageOrAuto) -> Self {
        match *computed {
            LengthOrPercentageOrAuto::Auto => specified::LengthOrPercentageOrAuto::Auto,
            LengthOrPercentageOrAuto::Length(value) => {
                specified::LengthOrPercentageOrAuto::Length(
                    ToComputedValue::from_computed_value(&value)
                )
            }
            LengthOrPercentageOrAuto::Percentage(value) => {
                specified::LengthOrPercentageOrAuto::Percentage(value)
            }
            LengthOrPercentageOrAuto::Calc(calc) => {
                specified::LengthOrPercentageOrAuto::Calc(
                    Box::new(ToComputedValue::from_computed_value(&calc))
                )
            }
        }
    }
}

#[allow(missing_docs)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Copy, PartialEq, ToCss)]
pub enum LengthOrPercentageOrNone {
    Length(Au),
    Percentage(Percentage),
    Calc(CalcLengthOrPercentage),
    None,
}

impl ComputeSquaredDistance for LengthOrPercentageOrNone {
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        match (self, other) {
            (&LengthOrPercentageOrNone::Length(ref this), &LengthOrPercentageOrNone::Length(ref other)) => {
                this.compute_squared_distance(other)
            },
            (&LengthOrPercentageOrNone::Percentage(ref this), &LengthOrPercentageOrNone::Percentage(ref other)) => {
                this.compute_squared_distance(other)
            },
            (this, other) => {
                <Option<CalcLengthOrPercentage>>::compute_squared_distance(&(*this).into(), &(*other).into())
            }
        }
    }
}

impl LengthOrPercentageOrNone {
    /// Returns the used value.
    pub fn to_used_value(&self, containing_length: Au) -> Option<Au> {
        match *self {
            LengthOrPercentageOrNone::None => None,
            LengthOrPercentageOrNone::Length(length) => Some(length),
            LengthOrPercentageOrNone::Percentage(percent) => Some(containing_length.scale_by(percent.0)),
            LengthOrPercentageOrNone::Calc(ref calc) => calc.to_used_value(Some(containing_length)),
        }
    }
}

impl fmt::Debug for LengthOrPercentageOrNone {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            LengthOrPercentageOrNone::Length(length) => write!(f, "{:?}", length),
            LengthOrPercentageOrNone::Percentage(percentage) => write!(f, "{}%", percentage.0 * 100.),
            LengthOrPercentageOrNone::Calc(calc) => write!(f, "{:?}", calc),
            LengthOrPercentageOrNone::None => write!(f, "none"),
        }
    }
}

impl ToComputedValue for specified::LengthOrPercentageOrNone {
    type ComputedValue = LengthOrPercentageOrNone;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> LengthOrPercentageOrNone {
        match *self {
            specified::LengthOrPercentageOrNone::Length(ref value) => {
                LengthOrPercentageOrNone::Length(value.to_computed_value(context))
            }
            specified::LengthOrPercentageOrNone::Percentage(value) => {
                LengthOrPercentageOrNone::Percentage(value)
            }
            specified::LengthOrPercentageOrNone::Calc(ref calc) => {
                LengthOrPercentageOrNone::Calc(calc.to_computed_value(context))
            }
            specified::LengthOrPercentageOrNone::None => {
                LengthOrPercentageOrNone::None
            }
        }
    }

    #[inline]
    fn from_computed_value(computed: &LengthOrPercentageOrNone) -> Self {
        match *computed {
            LengthOrPercentageOrNone::None => specified::LengthOrPercentageOrNone::None,
            LengthOrPercentageOrNone::Length(value) => {
                specified::LengthOrPercentageOrNone::Length(
                    ToComputedValue::from_computed_value(&value)
                )
            }
            LengthOrPercentageOrNone::Percentage(value) => {
                specified::LengthOrPercentageOrNone::Percentage(value)
            }
            LengthOrPercentageOrNone::Calc(calc) => {
                specified::LengthOrPercentageOrNone::Calc(
                    Box::new(ToComputedValue::from_computed_value(&calc))
                )
            }
        }
    }
}

/// A wrapper of LengthOrPercentage, whose value must be >= 0.
pub type NonNegativeLengthOrPercentage = NonNegative<LengthOrPercentage>;

impl From<NonNegativeAu> for NonNegativeLengthOrPercentage {
    #[inline]
    fn from(length: NonNegativeAu) -> Self {
        LengthOrPercentage::Length(length.0).into()
    }
}

impl From<LengthOrPercentage> for NonNegativeLengthOrPercentage {
    #[inline]
    fn from(lop: LengthOrPercentage) -> Self {
        NonNegative::<LengthOrPercentage>(lop)
    }
}

impl From<NonNegativeLengthOrPercentage> for LengthOrPercentage {
    #[inline]
    fn from(lop: NonNegativeLengthOrPercentage) -> LengthOrPercentage {
        lop.0
    }
}

impl NonNegativeLengthOrPercentage {
    /// Get zero value.
    #[inline]
    pub fn zero() -> Self {
        NonNegative::<LengthOrPercentage>(LengthOrPercentage::zero())
    }

    /// Returns true if the computed value is absolute 0 or 0%.
    #[inline]
    pub fn is_definitely_zero(&self) -> bool {
        self.0.is_definitely_zero()
    }

    /// Returns the used value.
    #[inline]
    pub fn to_used_value(&self, containing_length: Au) -> Au {
        self.0.to_used_value(containing_length)
    }
}

/// A computed `<length>` value.
pub type Length = Au;

/// Either a computed `<length>` or the `none` keyword.
pub type LengthOrNone = Either<Length, None_>;

/// Either a computed `<length>` or the `auto` keyword.
pub type LengthOrAuto = Either<Length, Auto>;

/// Either a computed `<length>` or a `<number>` value.
pub type LengthOrNumber = Either<Length, Number>;

impl LengthOrNumber {
    /// Returns `0`.
    #[inline]
    pub fn zero() -> Self {
        Either::Second(0.)
    }
}

/// Either a computed `<length>` or the `normal` keyword.
pub type LengthOrNormal = Either<Length, Normal>;

/// A wrapper of Length, whose value must be >= 0.
pub type NonNegativeLength = NonNegativeAu;

/// Either a computed NonNegativeLength or the `auto` keyword.
pub type NonNegativeLengthOrAuto = Either<NonNegativeLength, Auto>;

/// Either a computed NonNegativeLength or the `normal` keyword.
pub type NonNegativeLengthOrNormal = Either<NonNegativeLength, Normal>;

/// Either a computed NonNegativeLength or a NonNegativeNumber value.
pub type NonNegativeLengthOrNumber = Either<NonNegativeLength, NonNegativeNumber>;

/// A value suitable for a `min-width`, `min-height`, `width` or `height` property.
/// See values/specified/length.rs for more details.
#[allow(missing_docs)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Copy, Debug, PartialEq, ToCss)]
pub enum MozLength {
    LengthOrPercentageOrAuto(LengthOrPercentageOrAuto),
    ExtremumLength(ExtremumLength),
}

impl ComputeSquaredDistance for MozLength {
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        match (self, other) {
            (&MozLength::LengthOrPercentageOrAuto(ref this), &MozLength::LengthOrPercentageOrAuto(ref other)) => {
                this.compute_squared_distance(other)
            },
            _ => {
                // FIXME(nox): Should this return `Ok(SquaredDistance::Value(1.))`
                // when `self` and `other` are the same extremum value?
                Err(())
            },
        }
    }
}

impl MozLength {
    /// Returns the `auto` value.
    pub fn auto() -> Self {
        MozLength::LengthOrPercentageOrAuto(LengthOrPercentageOrAuto::Auto)
    }
}

impl ToComputedValue for specified::MozLength {
    type ComputedValue = MozLength;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> MozLength {
        match *self {
            specified::MozLength::LengthOrPercentageOrAuto(ref lopoa) => {
                MozLength::LengthOrPercentageOrAuto(lopoa.to_computed_value(context))
            }
            specified::MozLength::ExtremumLength(ref ext) => {
                MozLength::ExtremumLength(ext.clone())
            }
        }
    }

    #[inline]
    fn from_computed_value(computed: &MozLength) -> Self {
        match *computed {
            MozLength::LengthOrPercentageOrAuto(ref lopoa) =>
                specified::MozLength::LengthOrPercentageOrAuto(
                    specified::LengthOrPercentageOrAuto::from_computed_value(&lopoa)),
            MozLength::ExtremumLength(ref ext) =>
                specified::MozLength::ExtremumLength(ext.clone()),
        }
    }
}

/// A value suitable for a `max-width` or `max-height` property.
/// See values/specified/length.rs for more details.
#[allow(missing_docs)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Copy, Debug, PartialEq, ToCss)]
pub enum MaxLength {
    LengthOrPercentageOrNone(LengthOrPercentageOrNone),
    ExtremumLength(ExtremumLength),
}

impl ComputeSquaredDistance for MaxLength {
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        match (self, other) {
            (&MaxLength::LengthOrPercentageOrNone(ref this), &MaxLength::LengthOrPercentageOrNone(ref other)) => {
                this.compute_squared_distance(other)
            },
            _ => {
                // FIXME(nox): Should this return `Ok(SquaredDistance::Value(1.))`
                // when `self` and `other` are the same extremum value?
                Err(())
            },
        }
    }
}

impl MaxLength {
    /// Returns the `none` value.
    pub fn none() -> Self {
        MaxLength::LengthOrPercentageOrNone(LengthOrPercentageOrNone::None)
    }
}
impl ToComputedValue for specified::MaxLength {
    type ComputedValue = MaxLength;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> MaxLength {
        match *self {
            specified::MaxLength::LengthOrPercentageOrNone(ref lopon) => {
                MaxLength::LengthOrPercentageOrNone(lopon.to_computed_value(context))
            }
            specified::MaxLength::ExtremumLength(ref ext) => {
                MaxLength::ExtremumLength(ext.clone())
            }
        }
    }

    #[inline]
    fn from_computed_value(computed: &MaxLength) -> Self {
        match *computed {
            MaxLength::LengthOrPercentageOrNone(ref lopon) =>
                specified::MaxLength::LengthOrPercentageOrNone(
                    specified::LengthOrPercentageOrNone::from_computed_value(&lopon)),
            MaxLength::ExtremumLength(ref ext) =>
                specified::MaxLength::ExtremumLength(ext.clone()),
        }
    }
}
