/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Animated values.
//!
//! Some values, notably colors, cannot be interpolated directly with their
//! computed values and need yet another intermediate representation. This
//! module's raison d'être is to ultimately contain all these types.

use app_units::Au;
use std::cmp;
use values::computed::Angle as ComputedAngle;
use values::computed::BorderCornerRadius as ComputedBorderCornerRadius;
use values::computed::LengthOrPercentage as ComputedLengthOrPercentage;
use values::computed::Percentage as ComputedPercentage;
use values::specified::url::SpecifiedUrl;

pub mod effects;

/// Restriction types.
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum Restriction {
    /// No restriction.
    None,
    /// Value must be >= 0.
    NonNegative,
    /// Value must be >= 1.
    GreaterThanOrEqualToOne
}

/// Conversion between computed values and intermediate values for animations.
///
/// Notably, colors are represented as four floats during animations.
pub trait ToAnimatedValue: Sized {
    /// The type of the animated value.
    type AnimatedValue;

    /// Converts this value to an animated value.
    fn to_animated_value(self) -> Self::AnimatedValue;

    /// Converts back an animated value into a computed value.
    fn from_animated_value(animated: Self::AnimatedValue) -> Self;

    /// Converts back an animated value into a computed value with a restriction.
    #[inline]
    fn from_animated_value_with_restriction(animated: Self::AnimatedValue,
                                            _restriction: Restriction) -> Self {
        Self::from_animated_value(animated)
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

    #[inline]
    fn from_animated_value_with_restriction(animated: Self::AnimatedValue,
                                            restriction: Restriction) -> Self {
        animated.map(|value| T::from_animated_value_with_restriction(value, restriction))
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

    #[inline]
    fn from_animated_value_with_restriction(animated: Self::AnimatedValue,
                                            restriction: Restriction) -> Self {
        animated.into_iter().map(|value| {
            T::from_animated_value_with_restriction(value, restriction)
        }).collect()
    }
}

/// Marker trait for computed values with the same representation during animations.
pub trait AnimatedValueAsComputed: Sized {
    /// Return clamped value of Self.
    fn restrict_value(self, _restriction_type: Restriction) -> Self {
        self
    }
}

impl AnimatedValueAsComputed for ComputedAngle {}
impl AnimatedValueAsComputed for SpecifiedUrl {}
impl AnimatedValueAsComputed for bool {}

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

    #[inline]
    fn from_animated_value_with_restriction(animated: Self::AnimatedValue,
                                            restriction: Restriction) -> Self {
        animated.restrict_value(restriction)
    }
}

impl AnimatedValueAsComputed for f32 {
    #[inline]
    fn restrict_value(self, restriction_type: Restriction) -> Self {
        match restriction_type {
            Restriction::None => self,
            Restriction::NonNegative => self.max(0.),
            Restriction::GreaterThanOrEqualToOne => self.max(1.),
        }
    }
}

impl AnimatedValueAsComputed for i32 {
    #[inline]
    fn restrict_value(self, restriction_type: Restriction) -> Self {
        match restriction_type {
            Restriction::None => self,
            Restriction::NonNegative => cmp::max(self, 0),
            Restriction::GreaterThanOrEqualToOne => cmp::max(self, 1),
        }
    }
}

impl AnimatedValueAsComputed for Au {
    #[inline]
    fn restrict_value(self, restriction_type: Restriction) -> Self {
        Au(self.0.restrict_value(restriction_type))
    }
}

impl AnimatedValueAsComputed for ComputedPercentage {
    #[inline]
    fn restrict_value(self, restriction_type: Restriction) -> Self {
        ComputedPercentage(self.0.restrict_value(restriction_type))
    }
}

impl AnimatedValueAsComputed for ComputedLengthOrPercentage {
    #[inline]
    fn restrict_value(self, restriction_type: Restriction) -> Self {
        match self {
            ComputedLengthOrPercentage::Length(length) => {
                ComputedLengthOrPercentage::Length(length.restrict_value(restriction_type))
            },
            ComputedLengthOrPercentage::Percentage(percentage) => {
                ComputedLengthOrPercentage::Percentage(percentage.restrict_value(restriction_type))
            },
            _ => self
        }
    }
}

impl AnimatedValueAsComputed for ComputedBorderCornerRadius {
    #[inline]
    fn restrict_value(self, restriction_type: Restriction) -> Self {
        ComputedBorderCornerRadius::new(self.0.width.restrict_value(restriction_type),
                                        self.0.height.restrict_value(restriction_type))
    }
}
