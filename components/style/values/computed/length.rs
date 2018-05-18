/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! `<length>` computed values, and related ones.

use app_units::Au;
use logical_geometry::WritingMode;
use ordered_float::NotNaN;
use properties::LonghandId;
use std::fmt::{self, Write};
use std::ops::{Add, Neg};
use style_traits::{CssWriter, ToCss};
use style_traits::values::specified::AllowedNumericType;
use super::{Context, Number, Percentage, ToComputedValue};
use values::{specified, Auto, CSSFloat, Either, Normal};
use values::animated::{Animate, Procedure, ToAnimatedValue, ToAnimatedZero};
use values::distance::{ComputeSquaredDistance, SquaredDistance};
use values::generics::NonNegative;
use values::specified::length::{AbsoluteLength, FontBaseSize, FontRelativeLength};
use values::specified::length::ViewportPercentageLength;

pub use super::image::Image;
pub use values::specified::url::UrlOrNone;
pub use values::specified::{Angle, BorderStyle, Time};

impl ToComputedValue for specified::NoCalcLength {
    type ComputedValue = CSSPixelLength;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        match *self {
            specified::NoCalcLength::Absolute(length) => length.to_computed_value(context),
            specified::NoCalcLength::FontRelative(length) => {
                length.to_computed_value(context, FontBaseSize::CurrentStyle)
            },
            specified::NoCalcLength::ViewportPercentage(length) => {
                length.to_computed_value(context.viewport_size_for_viewport_unit_resolution())
            },
            specified::NoCalcLength::ServoCharacterWidth(length) => {
                length.to_computed_value(context.style().get_font().clone_font_size().size())
            },
        }
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        specified::NoCalcLength::Absolute(AbsoluteLength::Px(computed.px()))
    }
}

impl ToComputedValue for specified::Length {
    type ComputedValue = CSSPixelLength;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        match *self {
            specified::Length::NoCalc(l) => l.to_computed_value(context),
            specified::Length::Calc(ref calc) => calc.to_computed_value(context).length(),
        }
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        specified::Length::NoCalc(specified::NoCalcLength::from_computed_value(computed))
    }
}

#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, ToAnimatedZero)]
pub struct CalcLengthOrPercentage {
    #[animation(constant)]
    pub clamping_mode: AllowedNumericType,
    length: Length,
    pub percentage: Option<Percentage>,
}

impl ComputeSquaredDistance for CalcLengthOrPercentage {
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        // FIXME(nox): This looks incorrect to me, to add a distance between lengths
        // with a distance between percentages.
        Ok(self.unclamped_length()
            .compute_squared_distance(&other.unclamped_length())? +
            self.percentage()
                .compute_squared_distance(&other.percentage())?)
    }
}

impl CalcLengthOrPercentage {
    /// Returns a new `CalcLengthOrPercentage`.
    #[inline]
    pub fn new(length: Length, percentage: Option<Percentage>) -> Self {
        Self::with_clamping_mode(length, percentage, AllowedNumericType::All)
    }

    /// Returns a new `CalcLengthOrPercentage` with a specific clamping mode.
    #[inline]
    pub fn with_clamping_mode(
        length: Length,
        percentage: Option<Percentage>,
        clamping_mode: AllowedNumericType,
    ) -> Self {
        Self {
            clamping_mode,
            length,
            percentage,
        }
    }

    /// Returns this `calc()` as a `<length>`.
    ///
    /// Panics in debug mode if a percentage is present in the expression.
    #[inline]
    pub fn length(&self) -> CSSPixelLength {
        debug_assert!(self.percentage.is_none());
        self.length_component()
    }

    /// Returns the length component of this `calc()`
    #[inline]
    pub fn length_component(&self) -> CSSPixelLength {
        CSSPixelLength::new(self.clamping_mode.clamp(self.length.px()))
    }

    /// Returns the `<length>` component of this `calc()`, unclamped.
    #[inline]
    pub fn unclamped_length(&self) -> CSSPixelLength {
        self.length
    }

    /// Return the percentage value as CSSFloat.
    #[inline]
    pub fn percentage(&self) -> CSSFloat {
        self.percentage.map_or(0., |p| p.0)
    }

    /// Convert the computed value into used value.
    #[inline]
    pub fn to_used_value(&self, container_len: Option<Au>) -> Option<Au> {
        self.to_pixel_length(container_len).map(Au::from)
    }

    /// If there are special rules for computing percentages in a value (e.g.
    /// the height property), they apply whenever a calc() expression contains
    /// percentages.
    pub fn to_pixel_length(&self, container_len: Option<Au>) -> Option<Length> {
        match (container_len, self.percentage) {
            (Some(len), Some(percent)) => {
                let pixel = self.length.px() + len.scale_by(percent.0).to_f32_px();
                Some(Length::new(self.clamping_mode.clamp(pixel)))
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
                CalcLengthOrPercentage::new(Length::new(0.), Some(this))
            },
            LengthOrPercentage::Length(this) => CalcLengthOrPercentage::new(this, None),
            LengthOrPercentage::Calc(this) => this,
        }
    }
}

impl From<LengthOrPercentageOrAuto> for Option<CalcLengthOrPercentage> {
    fn from(len: LengthOrPercentageOrAuto) -> Option<CalcLengthOrPercentage> {
        match len {
            LengthOrPercentageOrAuto::Percentage(this) => {
                Some(CalcLengthOrPercentage::new(Length::new(0.), Some(this)))
            },
            LengthOrPercentageOrAuto::Length(this) => Some(CalcLengthOrPercentage::new(this, None)),
            LengthOrPercentageOrAuto::Calc(this) => Some(this),
            LengthOrPercentageOrAuto::Auto => None,
        }
    }
}

impl From<LengthOrPercentageOrNone> for Option<CalcLengthOrPercentage> {
    fn from(len: LengthOrPercentageOrNone) -> Option<CalcLengthOrPercentage> {
        match len {
            LengthOrPercentageOrNone::Percentage(this) => {
                Some(CalcLengthOrPercentage::new(Length::new(0.), Some(this)))
            },
            LengthOrPercentageOrNone::Length(this) => Some(CalcLengthOrPercentage::new(this, None)),
            LengthOrPercentageOrNone::Calc(this) => Some(this),
            LengthOrPercentageOrNone::None => None,
        }
    }
}

impl ToCss for CalcLengthOrPercentage {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        use num_traits::Zero;

        let (length, percentage) = match (self.length, self.percentage) {
            (l, None) => return l.to_css(dest),
            (l, Some(p)) if l.px() == 0. => return p.to_css(dest),
            (l, Some(p)) => (l, p),
        };

        dest.write_str("calc(")?;
        percentage.to_css(dest)?;

        dest.write_str(if length.px() < Zero::zero() {
            " - "
        } else {
            " + "
        })?;
        length.abs().to_css(dest)?;

        dest.write_str(")")
    }
}

impl specified::CalcLengthOrPercentage {
    /// Compute the value, zooming any absolute units by the zoom function.
    fn to_computed_value_with_zoom<F>(
        &self,
        context: &Context,
        zoom_fn: F,
        base_size: FontBaseSize,
    ) -> CalcLengthOrPercentage
    where
        F: Fn(Length) -> Length,
    {
        use std::f32;
        let mut length = 0.;

        if let Some(absolute) = self.absolute {
            length += zoom_fn(absolute.to_computed_value(context)).px();
        }

        for val in &[
            self.vw.map(ViewportPercentageLength::Vw),
            self.vh.map(ViewportPercentageLength::Vh),
            self.vmin.map(ViewportPercentageLength::Vmin),
            self.vmax.map(ViewportPercentageLength::Vmax),
        ] {
            if let Some(val) = *val {
                let viewport_size = context.viewport_size_for_viewport_unit_resolution();
                length += val.to_computed_value(viewport_size).px();
            }
        }

        for val in &[
            self.ch.map(FontRelativeLength::Ch),
            self.em.map(FontRelativeLength::Em),
            self.ex.map(FontRelativeLength::Ex),
            self.rem.map(FontRelativeLength::Rem),
        ] {
            if let Some(val) = *val {
                length += val.to_computed_value(context, base_size).px();
            }
        }

        CalcLengthOrPercentage {
            clamping_mode: self.clamping_mode,
            length: Length::new(length.min(f32::MAX).max(f32::MIN)),
            percentage: self.percentage,
        }
    }

    /// Compute font-size or line-height taking into account text-zoom if necessary.
    pub fn to_computed_value_zoomed(
        &self,
        context: &Context,
        base_size: FontBaseSize,
    ) -> CalcLengthOrPercentage {
        self.to_computed_value_with_zoom(
            context,
            |abs| context.maybe_zoom_text(abs.into()).0,
            base_size,
        )
    }

    /// Compute the value into pixel length as CSSFloat without context,
    /// so it returns Err(()) if there is any non-absolute unit.
    pub fn to_computed_pixel_length_without_context(&self) -> Result<CSSFloat, ()> {
        if self.vw.is_some() || self.vh.is_some() || self.vmin.is_some() || self.vmax.is_some() ||
            self.em.is_some() || self.ex.is_some() || self.ch.is_some() ||
            self.rem.is_some() || self.percentage.is_some()
        {
            return Err(());
        }

        match self.absolute {
            Some(abs) => Ok(abs.to_px()),
            None => {
                debug_assert!(false, "Someone forgot to handle an unit here: {:?}", self);
                Err(())
            },
        }
    }
}

impl ToComputedValue for specified::CalcLengthOrPercentage {
    type ComputedValue = CalcLengthOrPercentage;

    fn to_computed_value(&self, context: &Context) -> CalcLengthOrPercentage {
        // normal properties don't zoom, and compute em units against the current style's font-size
        self.to_computed_value_with_zoom(context, |abs| abs, FontBaseSize::CurrentStyle)
    }

    #[inline]
    fn from_computed_value(computed: &CalcLengthOrPercentage) -> Self {
        specified::CalcLengthOrPercentage {
            clamping_mode: computed.clamping_mode,
            absolute: Some(AbsoluteLength::from_computed_value(&computed.length)),
            percentage: computed.percentage,
            ..Default::default()
        }
    }
}

#[allow(missing_docs)]
#[animate(fallback = "Self::animate_fallback")]
#[css(derive_debug)]
#[derive(Animate, Clone, ComputeSquaredDistance, Copy, MallocSizeOf, PartialEq,
         ToAnimatedValue, ToAnimatedZero, ToCss)]
#[distance(fallback = "Self::compute_squared_distance_fallback")]
pub enum LengthOrPercentage {
    Length(Length),
    Percentage(Percentage),
    Calc(CalcLengthOrPercentage),
}

impl LengthOrPercentage {
    /// <https://drafts.csswg.org/css-transitions/#animtype-lpcalc>
    fn animate_fallback(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        // Special handling for zero values since these should not require calc().
        if self.is_definitely_zero() {
            return other.to_animated_zero()?.animate(other, procedure);
        }
        if other.is_definitely_zero() {
            return self.animate(&self.to_animated_zero()?, procedure);
        }

        let this = CalcLengthOrPercentage::from(*self);
        let other = CalcLengthOrPercentage::from(*other);
        Ok(LengthOrPercentage::Calc(this.animate(&other, procedure)?))
    }

    #[inline]
    fn compute_squared_distance_fallback(&self, other: &Self) -> Result<SquaredDistance, ()> {
        CalcLengthOrPercentage::compute_squared_distance(&(*self).into(), &(*other).into())
    }
}

impl From<Au> for LengthOrPercentage {
    #[inline]
    fn from(length: Au) -> Self {
        LengthOrPercentage::Length(length.into())
    }
}

impl LengthOrPercentage {
    #[inline]
    #[allow(missing_docs)]
    pub fn zero() -> LengthOrPercentage {
        LengthOrPercentage::Length(Length::new(0.))
    }

    #[inline]
    /// 1px length value for SVG defaults
    pub fn one() -> LengthOrPercentage {
        LengthOrPercentage::Length(Length::new(1.))
    }

    /// Returns true if the computed value is absolute 0 or 0%.
    ///
    /// (Returns false for calc() values, even if ones that may resolve to zero.)
    #[inline]
    pub fn is_definitely_zero(&self) -> bool {
        use self::LengthOrPercentage::*;
        match *self {
            Length(l) => l.px() == 0.0,
            Percentage(p) => p.0 == 0.0,
            Calc(_) => false,
        }
    }

    // CSSFloat doesn't implement Hash, so does CSSPixelLength. Therefore, we still use Au as the
    // hash key.
    #[allow(missing_docs)]
    pub fn to_hash_key(&self) -> (Au, NotNaN<f32>) {
        use self::LengthOrPercentage::*;
        match *self {
            Length(l) => (Au::from(l), NotNaN::new(0.0).unwrap()),
            Percentage(p) => (Au(0), NotNaN::new(p.0).unwrap()),
            Calc(c) => (
                Au::from(c.unclamped_length()),
                NotNaN::new(c.percentage()).unwrap(),
            ),
        }
    }

    /// Returns the used value.
    pub fn to_used_value(&self, containing_length: Au) -> Au {
        Au::from(self.to_pixel_length(containing_length))
    }

    /// Returns the used value as CSSPixelLength.
    pub fn to_pixel_length(&self, containing_length: Au) -> Length {
        match *self {
            LengthOrPercentage::Length(length) => length,
            LengthOrPercentage::Percentage(p) => containing_length.scale_by(p.0).into(),
            LengthOrPercentage::Calc(ref calc) => {
                calc.to_pixel_length(Some(containing_length)).unwrap()
            },
        }
    }

    /// Returns the clamped non-negative values.
    #[inline]
    pub fn clamp_to_non_negative(self) -> Self {
        match self {
            LengthOrPercentage::Length(length) => {
                LengthOrPercentage::Length(length.clamp_to_non_negative())
            },
            LengthOrPercentage::Percentage(percentage) => {
                LengthOrPercentage::Percentage(percentage.clamp_to_non_negative())
            },
            _ => self,
        }
    }
}

impl ToComputedValue for specified::LengthOrPercentage {
    type ComputedValue = LengthOrPercentage;

    fn to_computed_value(&self, context: &Context) -> LengthOrPercentage {
        match *self {
            specified::LengthOrPercentage::Length(ref value) => {
                LengthOrPercentage::Length(value.to_computed_value(context))
            },
            specified::LengthOrPercentage::Percentage(value) => {
                LengthOrPercentage::Percentage(value)
            },
            specified::LengthOrPercentage::Calc(ref calc) => {
                LengthOrPercentage::Calc((**calc).to_computed_value(context))
            },
        }
    }

    fn from_computed_value(computed: &LengthOrPercentage) -> Self {
        match *computed {
            LengthOrPercentage::Length(value) => {
                specified::LengthOrPercentage::Length(ToComputedValue::from_computed_value(&value))
            },
            LengthOrPercentage::Percentage(value) => {
                specified::LengthOrPercentage::Percentage(value)
            },
            LengthOrPercentage::Calc(ref calc) => specified::LengthOrPercentage::Calc(Box::new(
                ToComputedValue::from_computed_value(calc),
            )),
        }
    }
}

#[allow(missing_docs)]
#[animate(fallback = "Self::animate_fallback")]
#[css(derive_debug)]
#[derive(Animate, Clone, ComputeSquaredDistance, Copy, MallocSizeOf, PartialEq, ToCss)]
#[distance(fallback = "Self::compute_squared_distance_fallback")]
pub enum LengthOrPercentageOrAuto {
    Length(Length),
    Percentage(Percentage),
    Auto,
    Calc(CalcLengthOrPercentage),
}

impl LengthOrPercentageOrAuto {
    /// <https://drafts.csswg.org/css-transitions/#animtype-lpcalc>
    fn animate_fallback(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        let this = <Option<CalcLengthOrPercentage>>::from(*self);
        let other = <Option<CalcLengthOrPercentage>>::from(*other);
        Ok(LengthOrPercentageOrAuto::Calc(this.animate(
            &other,
            procedure,
        )?
            .ok_or(())?))
    }

    #[inline]
    fn compute_squared_distance_fallback(&self, other: &Self) -> Result<SquaredDistance, ()> {
        <Option<CalcLengthOrPercentage>>::compute_squared_distance(
            &(*self).into(),
            &(*other).into(),
        )
    }
}

/// A wrapper of LengthOrPercentageOrAuto, whose value must be >= 0.
pub type NonNegativeLengthOrPercentageOrAuto = NonNegative<LengthOrPercentageOrAuto>;

impl NonNegativeLengthOrPercentageOrAuto {
    /// `auto`
    #[inline]
    pub fn auto() -> Self {
        NonNegative(LengthOrPercentageOrAuto::Auto)
    }
}

impl ToAnimatedValue for NonNegativeLengthOrPercentageOrAuto {
    type AnimatedValue = LengthOrPercentageOrAuto;

    #[inline]
    fn to_animated_value(self) -> Self::AnimatedValue {
        self.0
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        NonNegative(animated.clamp_to_non_negative())
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
            Length(l) => l.px() == 0.0,
            Percentage(p) => p.0 == 0.0,
            Calc(_) | Auto => false,
        }
    }

    fn clamp_to_non_negative(self) -> Self {
        use self::LengthOrPercentageOrAuto::*;
        match self {
            Length(l) => Length(l.clamp_to_non_negative()),
            Percentage(p) => Percentage(p.clamp_to_non_negative()),
            _ => self,
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
            },
            specified::LengthOrPercentageOrAuto::Percentage(value) => {
                LengthOrPercentageOrAuto::Percentage(value)
            },
            specified::LengthOrPercentageOrAuto::Auto => LengthOrPercentageOrAuto::Auto,
            specified::LengthOrPercentageOrAuto::Calc(ref calc) => {
                LengthOrPercentageOrAuto::Calc((**calc).to_computed_value(context))
            },
        }
    }

    #[inline]
    fn from_computed_value(computed: &LengthOrPercentageOrAuto) -> Self {
        match *computed {
            LengthOrPercentageOrAuto::Auto => specified::LengthOrPercentageOrAuto::Auto,
            LengthOrPercentageOrAuto::Length(value) => specified::LengthOrPercentageOrAuto::Length(
                ToComputedValue::from_computed_value(&value),
            ),
            LengthOrPercentageOrAuto::Percentage(value) => {
                specified::LengthOrPercentageOrAuto::Percentage(value)
            },
            LengthOrPercentageOrAuto::Calc(calc) => specified::LengthOrPercentageOrAuto::Calc(
                Box::new(ToComputedValue::from_computed_value(&calc)),
            ),
        }
    }
}

#[allow(missing_docs)]
#[animate(fallback = "Self::animate_fallback")]
#[cfg_attr(feature = "servo", derive(MallocSizeOf))]
#[css(derive_debug)]
#[derive(Animate, Clone, ComputeSquaredDistance, Copy, PartialEq, ToCss)]
#[distance(fallback = "Self::compute_squared_distance_fallback")]
pub enum LengthOrPercentageOrNone {
    Length(Length),
    Percentage(Percentage),
    Calc(CalcLengthOrPercentage),
    None,
}

impl LengthOrPercentageOrNone {
    /// <https://drafts.csswg.org/css-transitions/#animtype-lpcalc>
    fn animate_fallback(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        let this = <Option<CalcLengthOrPercentage>>::from(*self);
        let other = <Option<CalcLengthOrPercentage>>::from(*other);
        Ok(LengthOrPercentageOrNone::Calc(this.animate(
            &other,
            procedure,
        )?
            .ok_or(())?))
    }

    fn compute_squared_distance_fallback(&self, other: &Self) -> Result<SquaredDistance, ()> {
        <Option<CalcLengthOrPercentage>>::compute_squared_distance(
            &(*self).into(),
            &(*other).into(),
        )
    }
}

impl LengthOrPercentageOrNone {
    /// Returns the used value.
    pub fn to_used_value(&self, containing_length: Au) -> Option<Au> {
        match *self {
            LengthOrPercentageOrNone::None => None,
            LengthOrPercentageOrNone::Length(length) => Some(Au::from(length)),
            LengthOrPercentageOrNone::Percentage(percent) => {
                Some(containing_length.scale_by(percent.0))
            },
            LengthOrPercentageOrNone::Calc(ref calc) => calc.to_used_value(Some(containing_length)),
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
            },
            specified::LengthOrPercentageOrNone::Percentage(value) => {
                LengthOrPercentageOrNone::Percentage(value)
            },
            specified::LengthOrPercentageOrNone::Calc(ref calc) => {
                LengthOrPercentageOrNone::Calc((**calc).to_computed_value(context))
            },
            specified::LengthOrPercentageOrNone::None => LengthOrPercentageOrNone::None,
        }
    }

    #[inline]
    fn from_computed_value(computed: &LengthOrPercentageOrNone) -> Self {
        match *computed {
            LengthOrPercentageOrNone::None => specified::LengthOrPercentageOrNone::None,
            LengthOrPercentageOrNone::Length(value) => specified::LengthOrPercentageOrNone::Length(
                ToComputedValue::from_computed_value(&value),
            ),
            LengthOrPercentageOrNone::Percentage(value) => {
                specified::LengthOrPercentageOrNone::Percentage(value)
            },
            LengthOrPercentageOrNone::Calc(calc) => specified::LengthOrPercentageOrNone::Calc(
                Box::new(ToComputedValue::from_computed_value(&calc)),
            ),
        }
    }
}

/// A wrapper of LengthOrPercentage, whose value must be >= 0.
pub type NonNegativeLengthOrPercentage = NonNegative<LengthOrPercentage>;

impl ToAnimatedValue for NonNegativeLengthOrPercentage {
    type AnimatedValue = LengthOrPercentage;

    #[inline]
    fn to_animated_value(self) -> Self::AnimatedValue {
        self.into()
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        animated.clamp_to_non_negative().into()
    }
}

impl From<NonNegativeLength> for NonNegativeLengthOrPercentage {
    #[inline]
    fn from(length: NonNegativeLength) -> Self {
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

/// The computed `<length>` value.
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[derive(Animate, Clone, ComputeSquaredDistance, Copy, Debug, MallocSizeOf, PartialEq,
         PartialOrd, ToAnimatedValue, ToAnimatedZero)]
pub struct CSSPixelLength(CSSFloat);

impl CSSPixelLength {
    /// Return a new CSSPixelLength.
    #[inline]
    pub fn new(px: CSSFloat) -> Self {
        CSSPixelLength(px)
    }

    /// Return the containing pixel value.
    #[inline]
    pub fn px(&self) -> CSSFloat {
        self.0
    }

    #[inline]
    fn clamp_to_non_negative(self) -> Self {
        Self::new(self.px().max(0.))
    }

    /// Return the length with app_unit i32 type.
    #[inline]
    pub fn to_i32_au(&self) -> i32 {
        Au::from(*self).0
    }

    /// Return the absolute value of this length.
    pub fn abs(self) -> Self {
        CSSPixelLength::new(self.0.abs())
    }

    /// Zero value
    pub fn zero() -> Self {
        CSSPixelLength::new(0.)
    }
}

impl ToCss for CSSPixelLength {
    #[inline]
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        self.0.to_css(dest)?;
        dest.write_str("px")
    }
}

impl Neg for CSSPixelLength {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self {
        CSSPixelLength::new(-self.0)
    }
}

impl From<CSSPixelLength> for Au {
    #[inline]
    fn from(len: CSSPixelLength) -> Self {
        Au::from_f32_px(len.0)
    }
}

impl From<Au> for CSSPixelLength {
    #[inline]
    fn from(len: Au) -> Self {
        CSSPixelLength::new(len.to_f32_px())
    }
}

/// An alias of computed `<length>` value.
pub type Length = CSSPixelLength;

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
pub type NonNegativeLength = NonNegative<Length>;

impl ToAnimatedValue for NonNegativeLength {
    type AnimatedValue = Length;

    #[inline]
    fn to_animated_value(self) -> Self::AnimatedValue {
        self.0
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        NonNegativeLength::new(animated.px().max(0.))
    }
}

impl NonNegativeLength {
    /// Create a NonNegativeLength.
    #[inline]
    pub fn new(px: CSSFloat) -> Self {
        NonNegative(Length::new(px.max(0.)))
    }

    /// Return a zero value.
    #[inline]
    pub fn zero() -> Self {
        Self::new(0.)
    }

    /// Return the pixel value of |NonNegativeLength|.
    #[inline]
    pub fn px(&self) -> CSSFloat {
        self.0.px()
    }

    #[inline]
    /// Ensures it is non negative
    pub fn clamp(self) -> Self {
        if (self.0).0 < 0. {
            Self::zero()
        } else {
            self
        }
    }

    /// Scale this NonNegativeLength.
    /// We scale NonNegativeLength by zero if the factor is negative because it doesn't
    /// make sense to scale a negative factor on a non-negative length.
    #[inline]
    pub fn scale_by(&self, factor: f32) -> Self {
        Self::new(self.0.px() * factor.max(0.))
    }
}

impl Add<NonNegativeLength> for NonNegativeLength {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        NonNegativeLength::new(self.px() + other.px())
    }
}

impl From<Length> for NonNegativeLength {
    #[inline]
    fn from(len: Length) -> Self {
        NonNegative(len)
    }
}

impl From<Au> for NonNegativeLength {
    #[inline]
    fn from(au: Au) -> Self {
        NonNegative(au.into())
    }
}

impl From<NonNegativeLength> for Au {
    #[inline]
    fn from(non_negative_len: NonNegativeLength) -> Self {
        Au::from(non_negative_len.0)
    }
}

/// Either a computed NonNegativeLength or the `auto` keyword.
pub type NonNegativeLengthOrAuto = Either<NonNegativeLength, Auto>;

/// Either a computed NonNegativeLength or the `normal` keyword.
pub type NonNegativeLengthOrNormal = Either<NonNegativeLength, Normal>;

/// Either a computed NonNegativeLengthOrPercentage or the `normal` keyword.
pub type NonNegativeLengthOrPercentageOrNormal = Either<NonNegativeLengthOrPercentage, Normal>;

/// A type for possible values for min- and max- flavors of width, height,
/// block-size, and inline-size.
#[allow(missing_docs)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, Parse, PartialEq,
         SpecifiedValueInfo, ToCss)]
pub enum ExtremumLength {
    MozMaxContent,
    MozMinContent,
    MozFitContent,
    MozAvailable,
}

impl ExtremumLength {
    /// Returns whether this size keyword can be used for the given writing-mode
    /// and property.
    ///
    /// TODO: After these values are supported for both axes (and maybe
    /// unprefixed, see bug 1322780) all this complexity can go away, and
    /// everything can be derived (no need for uncacheable stuff).
    fn valid_for(&self, wm: WritingMode, longhand: LonghandId) -> bool {
        // We only make sense on the inline axis.
        match longhand {
            // FIXME(emilio): The flex-basis thing is not quite clear...
            LonghandId::FlexBasis |
            LonghandId::MinWidth |
            LonghandId::MaxWidth |
            LonghandId::Width => !wm.is_vertical(),

            LonghandId::MinHeight | LonghandId::MaxHeight | LonghandId::Height => wm.is_vertical(),

            LonghandId::MinInlineSize | LonghandId::MaxInlineSize | LonghandId::InlineSize => true,
            // The block-* properties are rejected at parse-time, so they're
            // unexpected here.
            _ => {
                debug_assert!(
                    false,
                    "Unexpected property using ExtremumLength: {:?}",
                    longhand,
                );
                false
            },
        }
    }
}

/// A value suitable for a `min-width`, `min-height`, `width` or `height`
/// property.
///
/// See values/specified/length.rs for more details.
#[allow(missing_docs)]
#[cfg_attr(feature = "servo", derive(MallocSizeOf))]
#[derive(Animate, Clone, ComputeSquaredDistance, Copy, Debug, PartialEq, ToAnimatedZero, ToCss)]
pub enum MozLength {
    LengthOrPercentageOrAuto(LengthOrPercentageOrAuto),
    #[animation(error)]
    ExtremumLength(ExtremumLength),
}

impl MozLength {
    /// Returns the `auto` value.
    #[inline]
    pub fn auto() -> Self {
        MozLength::LengthOrPercentageOrAuto(LengthOrPercentageOrAuto::Auto)
    }
}

impl ToComputedValue for specified::MozLength {
    type ComputedValue = MozLength;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> MozLength {
        debug_assert!(
            context.for_non_inherited_property.is_some(),
            "Someone added a MozLength to an inherited property? Evil!"
        );
        match *self {
            specified::MozLength::LengthOrPercentageOrAuto(ref lopoa) => {
                MozLength::LengthOrPercentageOrAuto(lopoa.to_computed_value(context))
            },
            specified::MozLength::ExtremumLength(ext) => {
                context
                    .rule_cache_conditions
                    .borrow_mut()
                    .set_writing_mode_dependency(context.builder.writing_mode);
                if !ext.valid_for(
                    context.builder.writing_mode,
                    context.for_non_inherited_property.unwrap(),
                ) {
                    MozLength::auto()
                } else {
                    MozLength::ExtremumLength(ext)
                }
            },
        }
    }

    #[inline]
    fn from_computed_value(computed: &MozLength) -> Self {
        match *computed {
            MozLength::LengthOrPercentageOrAuto(ref lopoa) => {
                specified::MozLength::LengthOrPercentageOrAuto(
                    specified::LengthOrPercentageOrAuto::from_computed_value(lopoa),
                )
            },
            MozLength::ExtremumLength(ext) => specified::MozLength::ExtremumLength(ext),
        }
    }
}

/// A value suitable for a `max-width` or `max-height` property.
/// See values/specified/length.rs for more details.
#[allow(missing_docs)]
#[cfg_attr(feature = "servo", derive(MallocSizeOf))]
#[derive(Animate, Clone, ComputeSquaredDistance, Copy, Debug, PartialEq, ToCss)]
pub enum MaxLength {
    LengthOrPercentageOrNone(LengthOrPercentageOrNone),
    #[animation(error)]
    ExtremumLength(ExtremumLength),
}

impl MaxLength {
    /// Returns the `none` value.
    #[inline]
    pub fn none() -> Self {
        MaxLength::LengthOrPercentageOrNone(LengthOrPercentageOrNone::None)
    }
}

impl ToComputedValue for specified::MaxLength {
    type ComputedValue = MaxLength;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> MaxLength {
        debug_assert!(
            context.for_non_inherited_property.is_some(),
            "Someone added a MaxLength to an inherited property? Evil!"
        );
        match *self {
            specified::MaxLength::LengthOrPercentageOrNone(ref lopon) => {
                MaxLength::LengthOrPercentageOrNone(lopon.to_computed_value(context))
            },
            specified::MaxLength::ExtremumLength(ext) => {
                context
                    .rule_cache_conditions
                    .borrow_mut()
                    .set_writing_mode_dependency(context.builder.writing_mode);
                if !ext.valid_for(
                    context.builder.writing_mode,
                    context.for_non_inherited_property.unwrap(),
                ) {
                    MaxLength::none()
                } else {
                    MaxLength::ExtremumLength(ext)
                }
            },
        }
    }

    #[inline]
    fn from_computed_value(computed: &MaxLength) -> Self {
        match *computed {
            MaxLength::LengthOrPercentageOrNone(ref lopon) => {
                specified::MaxLength::LengthOrPercentageOrNone(
                    specified::LengthOrPercentageOrNone::from_computed_value(&lopon),
                )
            },
            MaxLength::ExtremumLength(ref ext) => specified::MaxLength::ExtremumLength(ext.clone()),
        }
    }
}
