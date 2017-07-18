/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Animated values.
//!
//! Some values, notably colors, cannot be interpolated directly with their
//! computed values and need yet another intermediate representation. This
//! module's raison d'être is to ultimately contain all these types.

use app_units::Au;
use values::computed::Angle as ComputedAngle;
use values::specified::url::SpecifiedUrl;

pub mod effects;

/// Conversion between computed values and intermediate values for animations.
///
/// Notably, colors are represented as four floats during animations.
pub trait ToAnimatedValue {
    /// The type of the animated value.
    type AnimatedValue;

    /// Converts this value to an animated value.
    fn to_animated_value(self) -> Self::AnimatedValue;

    /// Converts back an animated value into a computed value.
    fn from_animated_value(animated: Self::AnimatedValue) -> Self;
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

/// Marker trait for computed values with the same representation during animations.
pub trait AnimatedValueAsComputed {}

impl AnimatedValueAsComputed for Au {}
impl AnimatedValueAsComputed for ComputedAngle {}
impl AnimatedValueAsComputed for SpecifiedUrl {}
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

/// Returns a value similar to `self` that represents zero.
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
