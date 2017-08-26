/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computed types for CSS values related to backgrounds.

use properties::animated_properties::RepeatableListAnimatable;
use properties::longhands::background_size::computed_value::T as BackgroundSizeList;
use values::animated::{ToAnimatedValue, ToAnimatedZero};
use values::computed::length::LengthOrPercentageOrAuto;
use values::generics::background::BackgroundSize as GenericBackgroundSize;

/// A computed value for the `background-size` property.
pub type BackgroundSize = GenericBackgroundSize<LengthOrPercentageOrAuto>;

impl RepeatableListAnimatable for BackgroundSize {}

impl ToAnimatedZero for BackgroundSize {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> { Err(()) }
}

impl ToAnimatedValue for BackgroundSize {
    type AnimatedValue = Self;

    #[inline]
    fn to_animated_value(self) -> Self {
        self
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        use app_units::Au;
        use values::computed::Percentage;
        let clamp_animated_value = |value: LengthOrPercentageOrAuto| -> LengthOrPercentageOrAuto {
            match value {
                LengthOrPercentageOrAuto::Length(len) => {
                    LengthOrPercentageOrAuto::Length(Au(::std::cmp::max(len.0, 0)))
                },
                LengthOrPercentageOrAuto::Percentage(percent) => {
                    LengthOrPercentageOrAuto::Percentage(Percentage(percent.0.max(0.)))
                },
                _ => value
            }
        };
        match animated {
            GenericBackgroundSize::Explicit { width, height } => {
                GenericBackgroundSize::Explicit {
                    width: clamp_animated_value(width),
                    height: clamp_animated_value(height)
                }
            },
            _ => animated
        }
    }
}

impl ToAnimatedValue for BackgroundSizeList {
    type AnimatedValue = Self;

    #[inline]
    fn to_animated_value(self) -> Self {
        self
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        BackgroundSizeList(ToAnimatedValue::from_animated_value(animated.0))
    }
}
