/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Animated values.
//!
//! Some values, notably colors, cannot be interpolated directly with their
//! computed values and need yet another intermediate representation. This
//! module's raison d'être is to ultimately contain all these types.

use app_units::Au;
#[cfg(feature = "gecko")] use num_integer::lcm;
#[cfg(feature = "gecko")] use properties::animated_properties::Animatable;
#[cfg(feature = "gecko")] use properties::longhands::stroke_dasharray::computed_value::T as ComputedStrokeDashArrayList;
use values::computed::Angle as ComputedAngle;
#[cfg(feature = "gecko")] use values::computed::LengthOrPercentage;
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

/// An animated value for stroke-dasharray
///
/// https://www.w3.org/TR/SVG/animate.html#Animatable
#[cfg(feature = "gecko")]
#[derive(Clone, Debug, PartialEq)]
pub struct AnimatedStrokeDashArrayList(Vec<LengthOrPercentage>);

#[cfg(feature = "gecko")]
impl ToAnimatedValue for ComputedStrokeDashArrayList {
    type AnimatedValue = AnimatedStrokeDashArrayList;

    #[inline]
    fn to_animated_value(self) -> Self::AnimatedValue {
        AnimatedStrokeDashArrayList(self.0.to_animated_value())
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        ComputedStrokeDashArrayList(ToAnimatedValue::from_animated_value(animated.0))
    }
}

/// https://www.w3.org/TR/SVG/painting.html#StrokeDasharrayProperty
#[cfg(feature = "gecko")]
impl Animatable for AnimatedStrokeDashArrayList {
    #[inline]
    fn add_weighted(
        &self,
        other: &Self,
        self_portion: f64,
        other_portion: f64,
    ) -> Result<Self, ()> {
        if self.0.is_empty() || other.0.is_empty() {
            return Err(());
        }
        let loop_end = lcm(self.0.len(), other.0.len());
        let mut strokes = Vec::with_capacity(loop_end);
        for i in 0..loop_end {
            let from_item = self.0.get(i % self.0.len());
            let to_item = other.0.get(i % other.0.len());

            strokes.push(match (from_item, to_item) {
                (Some(from), Some(to)) => {
                    from.add_weighted(to, self_portion, other_portion)?
                },
                _ => unreachable!(),
            });
        }
        Ok(AnimatedStrokeDashArrayList(strokes))
    }

    #[allow(unused_variables)]
    fn add(&self, other: &Self) -> Result<Self, ()> {
        Err(())
    }

    #[allow(unused_variables)]
    fn accumulate(&self, other: &Self, count: u64) -> Result<Self, ()> {
        Err(())
    }
}
