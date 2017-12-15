/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Animated values.
//!
//! Some values, notably colors, cannot be interpolated directly with their
//! computed values and need yet another intermediate representation. This
//! module's raison d'être is to ultimately contain all these types.

use app_units::Au;
use euclid::{Point2D, Size2D};
use smallvec::SmallVec;
use std::cmp::max;
use values::computed::Angle as ComputedAngle;
use values::computed::BorderCornerRadius as ComputedBorderCornerRadius;
#[cfg(feature = "servo")]
use values::computed::ComputedUrl;
use values::computed::GreaterThanOrEqualToOneNumber as ComputedGreaterThanOrEqualToOneNumber;
use values::computed::MaxLength as ComputedMaxLength;
use values::computed::MozLength as ComputedMozLength;
use values::computed::NonNegativeLength as ComputedNonNegativeLength;
use values::computed::NonNegativeLengthOrPercentage as ComputedNonNegativeLengthOrPercentage;
use values::computed::NonNegativeNumber as ComputedNonNegativeNumber;
use values::computed::PositiveInteger as ComputedPositiveInteger;
use values::specified::url::SpecifiedUrl;

pub mod color;
pub mod effects;

/// Animate from one value to another.
///
/// This trait is derivable with `#[derive(Animate)]`. The derived
/// implementation uses a `match` expression with identical patterns for both
/// `self` and `other`, calling `Animate::animate` on each fields of the values.
/// If a field is annotated with `#[animation(constant)]`, the two values should
/// be equal or an error is returned.
///
/// If a variant is annotated with `#[animation(error)]`, the corresponding
/// `match` arm is not generated.
///
/// If the two values are not similar, an error is returned unless a fallback
/// function has been specified through `#[animate(fallback)]`.
pub trait Animate: Sized {
    /// Animate a value towards another one, given an animation procedure.
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()>;
}

/// An animation procedure.
///
/// <https://drafts.csswg.org/web-animations/#procedures-for-animating-properties>
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Procedure {
    /// <https://drafts.csswg.org/web-animations/#animation-interpolation>
    Interpolate { progress: f64 },
    /// <https://drafts.csswg.org/web-animations/#animation-addition>
    Add,
    /// <https://drafts.csswg.org/web-animations/#animation-accumulation>
    Accumulate { count: u64 },
}

/// Conversion between computed values and intermediate values for animations.
///
/// Notably, colors are represented as four floats during animations.
///
/// This trait is derivable with `#[derive(ToAnimatedValue)]`.
pub trait ToAnimatedValue {
    /// The type of the animated value.
    type AnimatedValue;

    /// Converts this value to an animated value.
    fn to_animated_value(self) -> Self::AnimatedValue;

    /// Converts back an animated value into a computed value.
    fn from_animated_value(animated: Self::AnimatedValue) -> Self;
}

/// Marker trait for computed values with the same representation during animations.
pub trait AnimatedValueAsComputed {}

/// Returns a value similar to `self` that represents zero.
///
/// This trait is derivable with `#[derive(ToAnimatedValue)]`. If a field is
/// annotated with `#[animation(constant)]`, a clone of its value will be used
/// instead of calling `ToAnimatedZero::to_animated_zero` on it.
///
/// If a variant is annotated with `#[animation(error)]`, the corresponding
/// `match` arm is not generated.
pub trait ToAnimatedZero: Sized {
    /// Returns a value that, when added with an underlying value, will produce the underlying
    /// value. This is used for SMIL animation's "by-animation" where SMIL first interpolates from
    /// the zero value to the 'by' value, and then adds the result to the underlying value.
    ///
    /// This is not the necessarily the same as the initial value of a property. For example, the
    /// initial value of 'stroke-width' is 1, but the zero value is 0, since adding 1 to the
    /// underlying value will not produce the underlying value.
    fn to_animated_zero(&self) -> Result<Self, ()>;
}

impl Procedure {
    /// Returns this procedure as a pair of weights.
    ///
    /// This is useful for animations that don't animate differently
    /// depending on the used procedure.
    #[inline]
    pub fn weights(self) -> (f64, f64) {
        match self {
            Procedure::Interpolate { progress } => (1. - progress, progress),
            Procedure::Add => (1., 1.),
            Procedure::Accumulate { count } => (count as f64, 1.),
        }
    }
}

/// <https://drafts.csswg.org/css-transitions/#animtype-number>
impl Animate for i32 {
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        Ok(((*self as f64).animate(&(*other as f64), procedure)? + 0.5).floor() as i32)
    }
}

/// <https://drafts.csswg.org/css-transitions/#animtype-number>
impl Animate for f32 {
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        use std::f32;

        let ret = (*self as f64).animate(&(*other as f64), procedure)?;
        Ok(ret.min(f32::MAX as f64).max(f32::MIN as f64) as f32)
    }
}

/// <https://drafts.csswg.org/css-transitions/#animtype-number>
impl Animate for f64 {
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        use std::f64;

        let (self_weight, other_weight) = procedure.weights();

        let ret = *self * self_weight + *other * other_weight;
        Ok(ret.min(f64::MAX).max(f64::MIN))
    }
}

impl<T> Animate for Option<T>
where
    T: Animate,
{
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        match (self.as_ref(), other.as_ref()) {
            (Some(ref this), Some(ref other)) => Ok(Some(this.animate(other, procedure)?)),
            (None, None) => Ok(None),
            _ => Err(()),
        }
    }
}

impl Animate for Au {
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        Ok(Au::new(self.0.animate(&other.0, procedure)?))
    }
}

impl<T> Animate for Size2D<T>
where
    T: Animate + Copy,
{
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        Ok(Size2D::new(
            self.width.animate(&other.width, procedure)?,
            self.height.animate(&other.height, procedure)?,
        ))
    }
}

impl<T> Animate for Point2D<T>
where
    T: Animate + Copy,
{
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        Ok(Point2D::new(
            self.x.animate(&other.x, procedure)?,
            self.y.animate(&other.y, procedure)?,
        ))
    }
}

impl<T> ToAnimatedValue for Option<T>
where
    T: ToAnimatedValue,
{
    type AnimatedValue = Option<<T as ToAnimatedValue>::AnimatedValue>;

    #[inline]
    fn to_animated_value(self) -> Self::AnimatedValue {
        self.map(T::to_animated_value)
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        animated.map(T::from_animated_value)
    }
}

impl<T> ToAnimatedValue for Vec<T>
where
    T: ToAnimatedValue,
{
    type AnimatedValue = Vec<<T as ToAnimatedValue>::AnimatedValue>;

    #[inline]
    fn to_animated_value(self) -> Self::AnimatedValue {
        self.into_iter().map(T::to_animated_value).collect()
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        animated.into_iter().map(T::from_animated_value).collect()
    }
}

impl<T> ToAnimatedValue for SmallVec<[T; 1]>
where
    T: ToAnimatedValue,
{
    type AnimatedValue = SmallVec<[T::AnimatedValue; 1]>;

    #[inline]
    fn to_animated_value(self) -> Self::AnimatedValue {
        self.into_iter().map(T::to_animated_value).collect()
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        animated.into_iter().map(T::from_animated_value).collect()
    }
}

impl AnimatedValueAsComputed for Au {}
impl AnimatedValueAsComputed for ComputedAngle {}
impl AnimatedValueAsComputed for SpecifiedUrl {}
#[cfg(feature = "servo")]
impl AnimatedValueAsComputed for ComputedUrl {}
impl AnimatedValueAsComputed for bool {}
impl AnimatedValueAsComputed for f32 {}

impl<T> ToAnimatedValue for T
where
    T: AnimatedValueAsComputed,
{
    type AnimatedValue = Self;

    #[inline]
    fn to_animated_value(self) -> Self {
        self
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        animated
    }
}

impl ToAnimatedValue for ComputedNonNegativeNumber {
    type AnimatedValue = Self;

    #[inline]
    fn to_animated_value(self) -> Self {
        self
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        animated.0.max(0.).into()
    }
}

impl ToAnimatedValue for ComputedGreaterThanOrEqualToOneNumber {
    type AnimatedValue = Self;

    #[inline]
    fn to_animated_value(self) -> Self {
        self
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        animated.0.max(1.).into()
    }
}

impl ToAnimatedValue for ComputedNonNegativeLength {
    type AnimatedValue = Self;

    #[inline]
    fn to_animated_value(self) -> Self {
        self
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        ComputedNonNegativeLength::new(animated.px().max(0.))
    }
}

impl ToAnimatedValue for ComputedPositiveInteger {
    type AnimatedValue = Self;

    #[inline]
    fn to_animated_value(self) -> Self {
        self
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        max(animated.0, 1).into()
    }
}

impl ToAnimatedValue for ComputedNonNegativeLengthOrPercentage {
    type AnimatedValue = Self;

    #[inline]
    fn to_animated_value(self) -> Self {
        self
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        animated.0.clamp_to_non_negative().into()
    }
}

impl ToAnimatedValue for ComputedBorderCornerRadius {
    type AnimatedValue = Self;

    #[inline]
    fn to_animated_value(self) -> Self {
        self
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        ComputedBorderCornerRadius::new((animated.0).0.width.clamp_to_non_negative(),
                                        (animated.0).0.height.clamp_to_non_negative())
    }
}

impl ToAnimatedValue for ComputedMaxLength {
    type AnimatedValue = Self;

    #[inline]
    fn to_animated_value(self) -> Self {
        self
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        use values::computed::{Length, LengthOrPercentageOrNone, Percentage};
        match animated {
            ComputedMaxLength::LengthOrPercentageOrNone(lopn) => {
                let result = match lopn {
                    LengthOrPercentageOrNone::Length(px) => {
                        LengthOrPercentageOrNone::Length(Length::new(px.px().max(0.)))
                    },
                    LengthOrPercentageOrNone::Percentage(percentage) => {
                        LengthOrPercentageOrNone::Percentage(Percentage(percentage.0.max(0.)))
                    }
                    _ => lopn
                };
                ComputedMaxLength::LengthOrPercentageOrNone(result)
            },
            _ => animated
        }
    }
}

impl ToAnimatedValue for ComputedMozLength {
    type AnimatedValue = Self;

    #[inline]
    fn to_animated_value(self) -> Self {
        self
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        use values::computed::{Length, LengthOrPercentageOrAuto, Percentage};
        match animated {
            ComputedMozLength::LengthOrPercentageOrAuto(lopa) => {
                let result = match lopa {
                    LengthOrPercentageOrAuto::Length(px) => {
                        LengthOrPercentageOrAuto::Length(Length::new(px.px().max(0.)))
                    },
                    LengthOrPercentageOrAuto::Percentage(percentage) => {
                        LengthOrPercentageOrAuto::Percentage(Percentage(percentage.0.max(0.)))
                    }
                    _ => lopa
                };
                ComputedMozLength::LengthOrPercentageOrAuto(result)
            },
            _ => animated
        }
    }
}

impl ToAnimatedZero for Au {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> { Ok(Au(0)) }
}

impl ToAnimatedZero for f32 {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> { Ok(0.) }
}

impl ToAnimatedZero for f64 {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> { Ok(0.) }
}

impl ToAnimatedZero for i32 {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> { Ok(0) }
}

impl<T> ToAnimatedZero for Option<T>
where
    T: ToAnimatedZero,
{
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        match *self {
            Some(ref value) => Ok(Some(value.to_animated_zero()?)),
            None => Ok(None),
        }
    }
}
