/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! `<length>` computed values, and related ones.

use super::{Context, Number, Percentage, ToComputedValue};
use crate::values::animated::ToAnimatedValue;
use crate::values::distance::{ComputeSquaredDistance, SquaredDistance};
use crate::values::generics::length::MaxLength as GenericMaxLength;
use crate::values::generics::length::MozLength as GenericMozLength;
use crate::values::generics::transform::IsZeroLength;
use crate::values::generics::NonNegative;
use crate::values::specified::length::ViewportPercentageLength;
use crate::values::specified::length::{AbsoluteLength, FontBaseSize, FontRelativeLength};
use crate::values::{specified, Auto, CSSFloat, Either, IsAuto, Normal};
use app_units::Au;
use ordered_float::NotNan;
use std::fmt::{self, Write};
use std::ops::{Add, Neg};
use style_traits::values::specified::AllowedNumericType;
use style_traits::{CssWriter, ToCss};

pub use super::image::Image;
pub use crate::values::specified::url::UrlOrNone;
pub use crate::values::specified::{Angle, BorderStyle, Time};

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

/// A `<length-percentage>` value. This can be either a `<length>`, a
/// `<percentage>`, or a combination of both via `calc()`.
///
/// https://drafts.csswg.org/css-values-4/#typedef-length-percentage
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, MallocSizeOf, ToAnimatedZero)]
pub struct LengthPercentage {
    #[animation(constant)]
    pub clamping_mode: AllowedNumericType,
    length: Length,
    pub percentage: Option<Percentage>,
    /// Whether this was from a calc() expression. This is needed because right
    /// now we don't treat calc() the same way as non-calc everywhere, but
    /// that's a bug in most cases.
    ///
    /// Please don't add new uses of this that aren't for converting to Gecko's
    /// representation, or to interpolate values.
    ///
    /// See https://github.com/w3c/csswg-drafts/issues/3482.
    #[animation(constant)]
    pub was_calc: bool,
}

// FIXME(emilio): This is a bit of a hack that can disappear as soon as we share
// representation of LengthPercentage with Gecko. The issue here is that Gecko
// uses CalcValue to represent position components, so they always come back as
// was_calc == true, and we mess up in the transitions code.
//
// This was a pre-existing bug, though arguably so only in pretty obscure cases
// like calc(0px + 5%) and such.
impl PartialEq for LengthPercentage {
    fn eq(&self, other: &Self) -> bool {
        self.length == other.length && self.percentage == other.percentage
    }
}

impl ComputeSquaredDistance for LengthPercentage {
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        // FIXME(nox): This looks incorrect to me, to add a distance between lengths
        // with a distance between percentages.
        Ok(self
            .unclamped_length()
            .compute_squared_distance(&other.unclamped_length())? +
            self.percentage()
                .compute_squared_distance(&other.percentage())?)
    }
}

impl LengthPercentage {
    /// Returns a new `LengthPercentage`.
    #[inline]
    pub fn new(length: Length, percentage: Option<Percentage>) -> Self {
        Self::with_clamping_mode(
            length,
            percentage,
            AllowedNumericType::All,
            /* was_calc = */ false,
        )
    }

    /// Returns a new `LengthPercentage` with zero length and some percentage.
    pub fn new_percent(percentage: Percentage) -> Self {
        Self::new(Length::zero(), Some(percentage))
    }

    /// Returns a new `LengthPercentage` with a specific clamping mode.
    #[inline]
    pub fn with_clamping_mode(
        length: Length,
        percentage: Option<Percentage>,
        clamping_mode: AllowedNumericType,
        was_calc: bool,
    ) -> Self {
        Self {
            clamping_mode,
            length,
            percentage,
            was_calc,
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

    /// Returns the percentage component if this could be represented as a
    /// non-calc percentage.
    pub fn as_percentage(&self) -> Option<Percentage> {
        if self.length.px() != 0. {
            return None;
        }

        let p = self.percentage?;
        if self.clamping_mode.clamp(p.0) != p.0 {
            return None;
        }

        Some(p)
    }

    /// Convert the computed value into used value.
    #[inline]
    pub fn maybe_to_used_value(&self, container_len: Option<Au>) -> Option<Au> {
        self.maybe_to_pixel_length(container_len).map(Au::from)
    }

    /// If there are special rules for computing percentages in a value (e.g.
    /// the height property), they apply whenever a calc() expression contains
    /// percentages.
    pub fn maybe_to_pixel_length(&self, container_len: Option<Au>) -> Option<Length> {
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

impl ToCss for LengthPercentage {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        specified::LengthPercentage::from_computed_value(self).to_css(dest)
    }
}

impl specified::CalcLengthPercentage {
    /// Compute the value, zooming any absolute units by the zoom function.
    fn to_computed_value_with_zoom<F>(
        &self,
        context: &Context,
        zoom_fn: F,
        base_size: FontBaseSize,
    ) -> LengthPercentage
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

        LengthPercentage {
            clamping_mode: self.clamping_mode,
            length: Length::new(length.min(f32::MAX).max(f32::MIN)),
            percentage: self.percentage,
            was_calc: true,
        }
    }

    /// Compute font-size or line-height taking into account text-zoom if necessary.
    pub fn to_computed_value_zoomed(
        &self,
        context: &Context,
        base_size: FontBaseSize,
    ) -> LengthPercentage {
        self.to_computed_value_with_zoom(
            context,
            |abs| context.maybe_zoom_text(abs.into()).0,
            base_size,
        )
    }

    /// Compute the value into pixel length as CSSFloat without context,
    /// so it returns Err(()) if there is any non-absolute unit.
    pub fn to_computed_pixel_length_without_context(&self) -> Result<CSSFloat, ()> {
        if self.vw.is_some() ||
            self.vh.is_some() ||
            self.vmin.is_some() ||
            self.vmax.is_some() ||
            self.em.is_some() ||
            self.ex.is_some() ||
            self.ch.is_some() ||
            self.rem.is_some() ||
            self.percentage.is_some()
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

impl ToComputedValue for specified::CalcLengthPercentage {
    type ComputedValue = LengthPercentage;

    fn to_computed_value(&self, context: &Context) -> LengthPercentage {
        // normal properties don't zoom, and compute em units against the current style's font-size
        self.to_computed_value_with_zoom(context, |abs| abs, FontBaseSize::CurrentStyle)
    }

    #[inline]
    fn from_computed_value(computed: &LengthPercentage) -> Self {
        specified::CalcLengthPercentage {
            clamping_mode: computed.clamping_mode,
            absolute: Some(AbsoluteLength::from_computed_value(&computed.length)),
            percentage: computed.percentage,
            ..Default::default()
        }
    }
}

impl LengthPercentage {
    #[inline]
    #[allow(missing_docs)]
    pub fn zero() -> LengthPercentage {
        LengthPercentage::new(Length::new(0.), None)
    }

    /// 1px length value for SVG defaults
    #[inline]
    pub fn one() -> LengthPercentage {
        LengthPercentage::new(Length::new(1.), None)
    }

    /// Returns true if the computed value is absolute 0 or 0%.
    #[inline]
    pub fn is_definitely_zero(&self) -> bool {
        self.unclamped_length().px() == 0.0 && self.percentage.map_or(true, |p| p.0 == 0.0)
    }

    // CSSFloat doesn't implement Hash, so does CSSPixelLength. Therefore, we still use Au as the
    // hash key.
    #[allow(missing_docs)]
    pub fn to_hash_key(&self) -> (Au, NotNan<f32>) {
        (
            Au::from(self.unclamped_length()),
            NotNan::new(self.percentage()).unwrap(),
        )
    }

    /// Returns the used value.
    pub fn to_used_value(&self, containing_length: Au) -> Au {
        Au::from(self.to_pixel_length(containing_length))
    }

    /// Returns the used value as CSSPixelLength.
    pub fn to_pixel_length(&self, containing_length: Au) -> Length {
        self.maybe_to_pixel_length(Some(containing_length)).unwrap()
    }

    /// Returns the clamped non-negative values.
    ///
    /// TODO(emilio): It's a bit unfortunate that this depends on whether the
    /// value was a `calc()` value or not. Should it?
    #[inline]
    pub fn clamp_to_non_negative(self) -> Self {
        if self.was_calc {
            return Self::with_clamping_mode(
                self.length,
                self.percentage,
                AllowedNumericType::NonNegative,
                self.was_calc,
            );
        }

        debug_assert!(self.percentage.is_none() || self.unclamped_length() == Length::zero());
        if let Some(p) = self.percentage {
            return Self::with_clamping_mode(
                Length::zero(),
                Some(p.clamp_to_non_negative()),
                AllowedNumericType::NonNegative,
                self.was_calc,
            );
        }

        Self::with_clamping_mode(
            self.length.clamp_to_non_negative(),
            None,
            AllowedNumericType::NonNegative,
            self.was_calc,
        )
    }
}

impl ToComputedValue for specified::LengthPercentage {
    type ComputedValue = LengthPercentage;

    fn to_computed_value(&self, context: &Context) -> LengthPercentage {
        match *self {
            specified::LengthPercentage::Length(ref value) => {
                LengthPercentage::new(value.to_computed_value(context), None)
            },
            specified::LengthPercentage::Percentage(value) => LengthPercentage::new_percent(value),
            specified::LengthPercentage::Calc(ref calc) => (**calc).to_computed_value(context),
        }
    }

    fn from_computed_value(computed: &LengthPercentage) -> Self {
        let length = computed.unclamped_length();
        if let Some(p) = computed.as_percentage() {
            return specified::LengthPercentage::Percentage(p);
        }

        let percentage = computed.percentage;
        if percentage.is_none() && computed.clamping_mode.clamp(length.px()) == length.px() {
            return specified::LengthPercentage::Length(ToComputedValue::from_computed_value(
                &length,
            ));
        }

        specified::LengthPercentage::Calc(Box::new(ToComputedValue::from_computed_value(computed)))
    }
}

impl IsZeroLength for LengthPercentage {
    #[inline]
    fn is_zero_length(&self) -> bool {
        self.is_definitely_zero()
    }
}

#[allow(missing_docs)]
#[css(derive_debug)]
#[derive(
    Animate, Clone, ComputeSquaredDistance, Copy, MallocSizeOf, PartialEq, ToAnimatedZero, ToCss,
)]
pub enum LengthPercentageOrAuto {
    LengthPercentage(LengthPercentage),
    Auto,
}

impl LengthPercentageOrAuto {
    /// Returns the `0` value.
    #[inline]
    pub fn zero() -> Self {
        LengthPercentageOrAuto::LengthPercentage(LengthPercentage::zero())
    }
}

/// A wrapper of LengthPercentageOrAuto, whose value must be >= 0.
pub type NonNegativeLengthPercentageOrAuto = NonNegative<LengthPercentageOrAuto>;

impl IsAuto for NonNegativeLengthPercentageOrAuto {
    #[inline]
    fn is_auto(&self) -> bool {
        *self == Self::auto()
    }
}

impl NonNegativeLengthPercentageOrAuto {
    /// `auto`
    #[inline]
    pub fn auto() -> Self {
        NonNegative(LengthPercentageOrAuto::Auto)
    }
}

impl ToAnimatedValue for NonNegativeLengthPercentageOrAuto {
    type AnimatedValue = LengthPercentageOrAuto;

    #[inline]
    fn to_animated_value(self) -> Self::AnimatedValue {
        self.0
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        NonNegative(animated.clamp_to_non_negative())
    }
}

impl LengthPercentageOrAuto {
    /// Returns true if the computed value is absolute 0 or 0%.
    #[inline]
    pub fn is_definitely_zero(&self) -> bool {
        use self::LengthPercentageOrAuto::*;
        match *self {
            LengthPercentage(ref l) => l.is_definitely_zero(),
            Auto => false,
        }
    }

    /// Clamps the value to a non-negative value.
    pub fn clamp_to_non_negative(self) -> Self {
        use self::LengthPercentageOrAuto::*;
        match self {
            LengthPercentage(l) => LengthPercentage(l.clamp_to_non_negative()),
            Auto => Auto,
        }
    }
}

impl ToComputedValue for specified::LengthPercentageOrAuto {
    type ComputedValue = LengthPercentageOrAuto;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> LengthPercentageOrAuto {
        match *self {
            specified::LengthPercentageOrAuto::LengthPercentage(ref value) => {
                LengthPercentageOrAuto::LengthPercentage(value.to_computed_value(context))
            },
            specified::LengthPercentageOrAuto::Auto => LengthPercentageOrAuto::Auto,
        }
    }

    #[inline]
    fn from_computed_value(computed: &LengthPercentageOrAuto) -> Self {
        match *computed {
            LengthPercentageOrAuto::Auto => specified::LengthPercentageOrAuto::Auto,
            LengthPercentageOrAuto::LengthPercentage(ref value) => {
                specified::LengthPercentageOrAuto::LengthPercentage(
                    ToComputedValue::from_computed_value(value),
                )
            },
        }
    }
}

#[allow(missing_docs)]
#[css(derive_debug)]
#[derive(
    Animate, Clone, ComputeSquaredDistance, Copy, MallocSizeOf, PartialEq, ToAnimatedZero, ToCss,
)]
pub enum LengthPercentageOrNone {
    LengthPercentage(LengthPercentage),
    None,
}

impl LengthPercentageOrNone {
    /// Returns the used value.
    pub fn to_used_value(&self, containing_length: Au) -> Option<Au> {
        match *self {
            LengthPercentageOrNone::None => None,
            LengthPercentageOrNone::LengthPercentage(ref lp) => {
                Some(lp.to_used_value(containing_length))
            },
        }
    }
}

// FIXME(emilio): Derive this.
impl ToComputedValue for specified::LengthPercentageOrNone {
    type ComputedValue = LengthPercentageOrNone;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> LengthPercentageOrNone {
        match *self {
            specified::LengthPercentageOrNone::LengthPercentage(ref value) => {
                LengthPercentageOrNone::LengthPercentage(value.to_computed_value(context))
            },
            specified::LengthPercentageOrNone::None => LengthPercentageOrNone::None,
        }
    }

    #[inline]
    fn from_computed_value(computed: &LengthPercentageOrNone) -> Self {
        match *computed {
            LengthPercentageOrNone::None => specified::LengthPercentageOrNone::None,
            LengthPercentageOrNone::LengthPercentage(value) => {
                specified::LengthPercentageOrNone::LengthPercentage(
                    ToComputedValue::from_computed_value(&value),
                )
            },
        }
    }
}

/// A wrapper of LengthPercentage, whose value must be >= 0.
pub type NonNegativeLengthPercentage = NonNegative<LengthPercentage>;

impl ToAnimatedValue for NonNegativeLengthPercentage {
    type AnimatedValue = LengthPercentage;

    #[inline]
    fn to_animated_value(self) -> Self::AnimatedValue {
        self.0
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        NonNegative(animated.clamp_to_non_negative())
    }
}

impl From<NonNegativeLength> for NonNegativeLengthPercentage {
    #[inline]
    fn from(length: NonNegativeLength) -> Self {
        LengthPercentage::new(length.0, None).into()
    }
}

impl From<LengthPercentage> for NonNegativeLengthPercentage {
    #[inline]
    fn from(lp: LengthPercentage) -> Self {
        NonNegative::<LengthPercentage>(lp)
    }
}

impl From<NonNegativeLengthPercentage> for LengthPercentage {
    #[inline]
    fn from(lp: NonNegativeLengthPercentage) -> LengthPercentage {
        lp.0
    }
}

// TODO(emilio): This is a really generic impl which is only needed to implement
// Animated and co for Spacing<>. Get rid of this, probably?
impl From<Au> for LengthPercentage {
    #[inline]
    fn from(length: Au) -> Self {
        LengthPercentage::new(length.into(), None)
    }
}

impl NonNegativeLengthPercentage {
    /// Get zero value.
    #[inline]
    pub fn zero() -> Self {
        NonNegative::<LengthPercentage>(LengthPercentage::zero())
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
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Copy,
    Debug,
    MallocSizeOf,
    PartialEq,
    PartialOrd,
    ToAnimatedValue,
    ToAnimatedZero,
)]
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

    /// Return the length with app_unit i32 type.
    #[inline]
    pub fn to_i32_au(&self) -> i32 {
        Au::from(*self).0
    }

    /// Return the absolute value of this length.
    #[inline]
    pub fn abs(self) -> Self {
        CSSPixelLength::new(self.0.abs())
    }

    /// Return the clamped value of this length.
    #[inline]
    pub fn clamp_to_non_negative(self) -> Self {
        CSSPixelLength::new(self.0.max(0.))
    }

    /// Zero value
    #[inline]
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

/// Either a computed NonNegativeLengthPercentage or the `normal` keyword.
pub type NonNegativeLengthPercentageOrNormal = Either<NonNegativeLengthPercentage, Normal>;

/// A type for possible values for min- and max- flavors of width, height,
/// block-size, and inline-size.
#[allow(missing_docs)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    FromPrimitive,
    MallocSizeOf,
    Parse,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
)]
#[repr(u8)]
pub enum ExtremumLength {
    #[parse(aliases = "-moz-max-content")]
    MaxContent,
    #[parse(aliases = "-moz-min-content")]
    MinContent,
    MozFitContent,
    MozAvailable,
}

/// A computed value for `min-width`, `min-height`, `width` or `height` property.
pub type MozLength = GenericMozLength<LengthPercentageOrAuto>;

impl MozLength {
    /// Returns the `auto` value.
    #[inline]
    pub fn auto() -> Self {
        GenericMozLength::LengthPercentageOrAuto(LengthPercentageOrAuto::Auto)
    }
}

/// A computed value for `max-width` or `min-height` property.
pub type MaxLength = GenericMaxLength<LengthPercentageOrNone>;

impl MaxLength {
    /// Returns the `none` value.
    #[inline]
    pub fn none() -> Self {
        GenericMaxLength::LengthPercentageOrNone(LengthPercentageOrNone::None)
    }
}
