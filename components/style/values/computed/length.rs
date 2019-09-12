/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! `<length>` computed values, and related ones.

use super::{Context, Number, Percentage, ToComputedValue};
use crate::values::animated::ToAnimatedValue;
use crate::values::computed::NonNegativeNumber;
use crate::values::distance::{ComputeSquaredDistance, SquaredDistance};
use crate::values::generics::length as generics;
use crate::values::generics::length::{
    GenericLengthOrNumber, GenericLengthPercentageOrNormal, GenericMaxSize, GenericSize,
};
use crate::values::generics::NonNegative;
use crate::values::specified::length::ViewportPercentageLength;
use crate::values::specified::length::{AbsoluteLength, FontBaseSize, FontRelativeLength};
use crate::values::{specified, CSSFloat};
use crate::Zero;
use app_units::Au;
use ordered_float::NotNan;
use std::fmt::{self, Write};
use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub};
use style_traits::values::specified::AllowedNumericType;
use style_traits::{CSSPixel, CssWriter, ToCss};

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
#[derive(Clone, Copy, Debug, MallocSizeOf, ToAnimatedZero, ToResolvedValue)]
#[repr(C)]
pub struct LengthPercentage {
    length: Length,
    percentage: Percentage,
    #[animation(constant)]
    pub clamping_mode: AllowedNumericType,
    /// Whether we specified a percentage or not.
    #[animation(constant)]
    pub has_percentage: bool,
}

// NOTE(emilio): We don't compare `clamping_mode` since we want to preserve the
// invariant that `from_computed_value(length).to_computed_value(..) == length`.
//
// Right now for e.g. a non-negative length, we set clamping_mode to `All`
// unconditionally for non-calc values, and to `NonNegative` for calc.
//
// If we determine that it's sound, from_computed_value() can generate an
// absolute length, which then would get `All` as the clamping mode.
//
// We may want to just eagerly-detect whether we can clamp in
// `LengthPercentage::new` and switch to `AllowedNumericType::NonNegative` then,
// maybe.
impl PartialEq for LengthPercentage {
    fn eq(&self, other: &Self) -> bool {
        self.length == other.length &&
            self.percentage == other.percentage &&
            self.has_percentage == other.has_percentage
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
            self.percentage
                .compute_squared_distance(&other.percentage)?)
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
    ) -> Self {
        Self {
            clamping_mode,
            length,
            percentage: percentage.unwrap_or_default(),
            has_percentage: percentage.is_some(),
        }
    }

    /// Returns this `calc()` as a `<length>`.
    ///
    /// Panics in debug mode if a percentage is present in the expression.
    #[inline]
    pub fn length(&self) -> CSSPixelLength {
        debug_assert!(!self.has_percentage);
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
        self.percentage.0
    }

    /// Return the specified percentage if any.
    #[inline]
    pub fn specified_percentage(&self) -> Option<Percentage> {
        if self.has_percentage {
            Some(self.percentage)
        } else {
            None
        }
    }

    /// Returns the percentage component if this could be represented as a
    /// non-calc percentage.
    pub fn as_percentage(&self) -> Option<Percentage> {
        if !self.has_percentage || self.length.px() != 0. {
            return None;
        }

        Some(Percentage(self.clamping_mode.clamp(self.percentage.0)))
    }

    /// Resolves the percentage.
    #[inline]
    pub fn percentage_relative_to(&self, basis: Length) -> Length {
        let length = self.unclamped_length().0 + basis.0 * self.percentage.0;
        Length::new(self.clamping_mode.clamp(length))
    }

    /// Convert the computed value into used value.
    #[inline]
    pub fn maybe_to_used_value(&self, container_len: Option<Length>) -> Option<Au> {
        self.maybe_percentage_relative_to(container_len)
            .map(Au::from)
    }

    /// If there are special rules for computing percentages in a value (e.g.
    /// the height property), they apply whenever a calc() expression contains
    /// percentages.
    pub fn maybe_percentage_relative_to(&self, container_len: Option<Length>) -> Option<Length> {
        if self.has_percentage {
            return Some(self.percentage_relative_to(container_len?));
        }
        Some(self.length())
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

        LengthPercentage::with_clamping_mode(
            Length::new(length.min(f32::MAX).max(f32::MIN)),
            self.percentage,
            self.clamping_mode,
        )
    }

    /// Compute font-size or line-height taking into account text-zoom if necessary.
    pub fn to_computed_value_zoomed(
        &self,
        context: &Context,
        base_size: FontBaseSize,
    ) -> LengthPercentage {
        self.to_computed_value_with_zoom(
            context,
            |abs| context.maybe_zoom_text(abs.into()),
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
            percentage: computed.specified_percentage(),
            ..Default::default()
        }
    }
}

impl LengthPercentage {
    /// 1px length value for SVG defaults
    #[inline]
    pub fn one() -> LengthPercentage {
        LengthPercentage::new(Length::new(1.), None)
    }

    /// Returns true if the computed value is absolute 0 or 0%.
    #[inline]
    pub fn is_definitely_zero(&self) -> bool {
        self.unclamped_length().px() == 0.0 && self.percentage.0 == 0.0
    }

    // CSSFloat doesn't implement Hash, so does CSSPixelLength. Therefore, we
    // still use Au as the hash key.
    #[allow(missing_docs)]
    pub fn to_hash_key(&self) -> (Au, NotNan<f32>) {
        (
            Au::from(self.unclamped_length()),
            NotNan::new(self.percentage.0).unwrap(),
        )
    }

    /// Returns the used value.
    pub fn to_used_value(&self, containing_length: Au) -> Au {
        Au::from(self.to_pixel_length(containing_length))
    }

    /// Returns the used value as CSSPixelLength.
    pub fn to_pixel_length(&self, containing_length: Au) -> Length {
        self.percentage_relative_to(containing_length.into())
    }

    /// Returns the clamped non-negative values.
    #[inline]
    pub fn clamp_to_non_negative(self) -> Self {
        if let Some(p) = self.specified_percentage() {
            // If we can eagerly clamp the percentage then just do that.
            if self.length.is_zero() {
                return Self::with_clamping_mode(
                    Length::zero(),
                    Some(p.clamp_to_non_negative()),
                    AllowedNumericType::NonNegative,
                );
            }

            return Self::with_clamping_mode(
                self.length,
                Some(p),
                AllowedNumericType::NonNegative,
            )
        }

        Self::with_clamping_mode(
            self.length.clamp_to_non_negative(),
            None,
            AllowedNumericType::NonNegative,
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
        if let Some(p) = computed.as_percentage() {
            return specified::LengthPercentage::Percentage(p);
        }

        if !computed.has_percentage {
            return specified::LengthPercentage::Length(ToComputedValue::from_computed_value(
                &computed.length(),
            ));
        }

        specified::LengthPercentage::Calc(Box::new(ToComputedValue::from_computed_value(computed)))
    }
}

impl Zero for LengthPercentage {
    fn zero() -> Self {
        LengthPercentage::new(Length::zero(), None)
    }

    #[inline]
    fn is_zero(&self) -> bool {
        self.is_definitely_zero()
    }
}

/// Some boilerplate to share between negative and non-negative
/// length-percentage or auto.
macro_rules! computed_length_percentage_or_auto {
    ($inner:ty) => {
        /// Returns the used value.
        #[inline]
        pub fn to_used_value(&self, percentage_basis: Au) -> Option<Au> {
            match *self {
                generics::GenericLengthPercentageOrAuto::Auto => None,
                generics::GenericLengthPercentageOrAuto::LengthPercentage(ref lp) => {
                    Some(lp.to_used_value(percentage_basis))
                }
            }
        }

        /// Returns true if the computed value is absolute 0 or 0%.
        #[inline]
        pub fn is_definitely_zero(&self) -> bool {
            use values::generics::length::LengthPercentageOrAuto::*;
            match *self {
                LengthPercentage(ref l) => l.is_definitely_zero(),
                Auto => false,
            }
        }
    }
}

/// A computed type for `<length-percentage> | auto`.
pub type LengthPercentageOrAuto = generics::GenericLengthPercentageOrAuto<LengthPercentage>;

impl LengthPercentageOrAuto {
    /// Clamps the value to a non-negative value.
    pub fn clamp_to_non_negative(self) -> Self {
        use values::generics::length::LengthPercentageOrAuto::*;
        match self {
            LengthPercentage(l) => LengthPercentage(l.clamp_to_non_negative()),
            Auto => Auto,
        }
    }

    computed_length_percentage_or_auto!(LengthPercentage);

    /// Resolves the percentage.
    #[inline]
    pub fn percentage_relative_to(&self, basis: Length) -> LengthOrAuto {
        use values::generics::length::LengthPercentageOrAuto::*;
        match self {
            LengthPercentage(length_percentage) => {
                LengthPercentage(length_percentage.percentage_relative_to(basis))
            },
            Auto => Auto,
        }
    }

    /// Maybe resolves the percentage.
    #[inline]
    pub fn maybe_percentage_relative_to(&self, basis: Option<Length>) -> LengthOrAuto {
        use values::generics::length::LengthPercentageOrAuto::*;
        match self {
            LengthPercentage(length_percentage) => length_percentage
                .maybe_percentage_relative_to(basis)
                .map_or(Auto, LengthPercentage),
            Auto => Auto,
        }
    }
}

/// A wrapper of LengthPercentageOrAuto, whose value must be >= 0.
pub type NonNegativeLengthPercentageOrAuto =
    generics::GenericLengthPercentageOrAuto<NonNegativeLengthPercentage>;

impl NonNegativeLengthPercentageOrAuto {
    computed_length_percentage_or_auto!(NonNegativeLengthPercentage);
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

// TODO(emilio): This is a really generic impl which is only needed to implement
// Animated and co for Spacing<>. Get rid of this, probably?
impl From<Au> for LengthPercentage {
    #[inline]
    fn from(length: Au) -> Self {
        LengthPercentage::new(length.into(), None)
    }
}

impl NonNegativeLengthPercentage {
    /// Returns true if the computed value is absolute 0 or 0%.
    #[inline]
    pub fn is_definitely_zero(&self) -> bool {
        self.0.is_definitely_zero()
    }

    /// Returns the used value.
    #[inline]
    pub fn to_used_value(&self, containing_length: Au) -> Au {
        let resolved = self.0.to_used_value(containing_length);
        ::std::cmp::max(resolved, Au(0))
    }

    /// Convert the computed value into used value.
    #[inline]
    pub fn maybe_to_used_value(&self, containing_length: Option<Au>) -> Option<Au> {
        let resolved = self
            .0
            .maybe_to_used_value(containing_length.map(|v| v.into()))?;
        Some(::std::cmp::max(resolved, Au(0)))
    }
}

#[cfg(feature = "servo")]
impl MaxSize {
    /// Convert the computed value into used value.
    #[inline]
    pub fn to_used_value(&self, percentage_basis: Au) -> Option<Au> {
        match *self {
            GenericMaxSize::None => None,
            GenericMaxSize::LengthPercentage(ref lp) => Some(lp.to_used_value(percentage_basis)),
        }
    }
}

impl Size {
    /// Convert the computed value into used value.
    #[inline]
    #[cfg(feature = "servo")]
    pub fn to_used_value(&self, percentage_basis: Au) -> Option<Au> {
        match *self {
            GenericSize::Auto => None,
            GenericSize::LengthPercentage(ref lp) => Some(lp.to_used_value(percentage_basis)),
        }
    }

    /// Returns true if the computed value is absolute 0 or 0%.
    #[inline]
    pub fn is_definitely_zero(&self) -> bool {
        match *self {
            GenericSize::Auto => false,
            GenericSize::LengthPercentage(ref lp) => lp.is_definitely_zero(),
            #[cfg(feature = "gecko")]
            GenericSize::ExtremumLength(..) => false,
        }
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
    ToResolvedValue,
    ToShmem,
)]
#[repr(C)]
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

    /// Returns the minimum between `self` and `other`.
    #[inline]
    pub fn min(self, other: Self) -> Self {
        CSSPixelLength::new(self.0.min(other.0))
    }

    /// Returns the maximum between `self` and `other`.
    #[inline]
    pub fn max(self, other: Self) -> Self {
        CSSPixelLength::new(self.0.max(other.0))
    }

    /// Sets `self` to the maximum between `self` and `other`.
    pub fn max_assign(&mut self, other: Self) {
        *self = self.max(other);
    }
}

impl Zero for CSSPixelLength {
    fn zero() -> Self {
        CSSPixelLength::new(0.)
    }

    fn is_zero(&self) -> bool {
        self.px() == 0.
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

impl Add for CSSPixelLength {
    type Output = Self;

    #[inline]
    fn add(self, other: Self) -> Self {
        Self::new(self.px() + other.px())
    }
}

impl AddAssign for CSSPixelLength {
    #[inline]
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}

impl Div<CSSFloat> for CSSPixelLength {
    type Output = Self;

    #[inline]
    fn div(self, other: CSSFloat) -> Self {
        Self::new(self.px() / other)
    }
}

impl Mul<CSSFloat> for CSSPixelLength {
    type Output = Self;

    #[inline]
    fn mul(self, other: CSSFloat) -> Self {
        Self::new(self.px() * other)
    }
}

impl Neg for CSSPixelLength {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self {
        CSSPixelLength::new(-self.0)
    }
}

impl Sub for CSSPixelLength {
    type Output = Self;

    #[inline]
    fn sub(self, other: Self) -> Self {
        Self::new(self.px() - other.px())
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

impl From<CSSPixelLength> for euclid::Length<CSSFloat, CSSPixel> {
    #[inline]
    fn from(length: CSSPixelLength) -> Self {
        Self::new(length.0)
    }
}

/// An alias of computed `<length>` value.
pub type Length = CSSPixelLength;

/// Either a computed `<length>` or the `auto` keyword.
pub type LengthOrAuto = generics::GenericLengthPercentageOrAuto<Length>;

/// Either a non-negative `<length>` or the `auto` keyword.
pub type NonNegativeLengthOrAuto = generics::GenericLengthPercentageOrAuto<NonNegativeLength>;

/// Either a computed `<length>` or a `<number>` value.
pub type LengthOrNumber = GenericLengthOrNumber<Length, Number>;

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

/// Either a computed NonNegativeLengthPercentage or the `normal` keyword.
pub type NonNegativeLengthPercentageOrNormal =
    GenericLengthPercentageOrNormal<NonNegativeLengthPercentage>;

/// Either a non-negative `<length>` or a `<number>`.
pub type NonNegativeLengthOrNumber = GenericLengthOrNumber<NonNegativeLength, NonNegativeNumber>;

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
    ToAnimatedValue,
    ToAnimatedZero,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
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
pub type Size = GenericSize<NonNegativeLengthPercentage>;

/// A computed value for `max-width` or `min-height` property.
pub type MaxSize = GenericMaxSize<NonNegativeLengthPercentage>;
