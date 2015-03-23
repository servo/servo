/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use properties::ComputedValues;
use properties::longhands::transition_property::computed_value::TransitionProperty;
use properties::longhands::transition_timing_function::computed_value::{StartEnd};
use properties::longhands::transition_timing_function::computed_value::{TransitionTimingFunction};
use properties::longhands::transition_property;
use values::computed::{LengthOrPercentageOrAuto, Time};

use std::num::Float;
use util::bezier::Bezier;
use util::geometry::Au;

#[derive(Copy, Clone, Debug)]
pub struct PropertyAnimation {
    property: AnimatedProperty,
    timing_function: TransitionTimingFunction,
    duration: Time,
}

impl PropertyAnimation {
    /// Creates a new property animation for the given transition index and old and new styles.
    /// Any number of animations may be returned, from zero (if the property did not animate) to
    /// one (for a single transition property) to arbitrarily many (for `all`).
    pub fn from_transition(transition_index: usize,
                           old_style: &ComputedValues,
                           new_style: &mut ComputedValues)
                           -> Vec<PropertyAnimation> {
        let mut result = Vec::new();
        let transition_property =
            new_style.get_animation().transition_property.0[transition_index];
        if transition_property != TransitionProperty::All {
            if let Some(property_animation) =
                    PropertyAnimation::from_transition_property(transition_property,
                                                                transition_index,
                                                                old_style,
                                                                new_style) {
                result.push(property_animation)
            }
            return result
        }

        for transition_property in
                transition_property::computed_value::ALL_TRANSITION_PROPERTIES.iter() {
            if let Some(property_animation) =
                    PropertyAnimation::from_transition_property(*transition_property,
                                                                transition_index,
                                                                old_style,
                                                                new_style) {
                result.push(property_animation)
            }
        }

        result
    }

    fn from_transition_property(transition_property: TransitionProperty,
                                transition_index: usize,
                                old_style: &ComputedValues,
                                new_style: &mut ComputedValues)
                                -> Option<PropertyAnimation> {
        let animation_style = new_style.get_animation();
        let animated_property = match transition_property {
            TransitionProperty::All => {
                panic!("Don't use `TransitionProperty::All` with \
                        `PropertyAnimation::from_transition_property`!")
            }
            TransitionProperty::Top => {
                AnimatedProperty::Top(old_style.get_positionoffsets().top,
                                      new_style.get_positionoffsets().top)
            }
            TransitionProperty::Right => {
                AnimatedProperty::Right(old_style.get_positionoffsets().right,
                                        new_style.get_positionoffsets().right)
            }
            TransitionProperty::Bottom => {
                AnimatedProperty::Bottom(old_style.get_positionoffsets().bottom,
                                         new_style.get_positionoffsets().bottom)
            }
            TransitionProperty::Left => {
                AnimatedProperty::Left(old_style.get_positionoffsets().left,
                                       new_style.get_positionoffsets().left)
            }
        };

        let property_animation = PropertyAnimation {
            property: animated_property,
            timing_function:
                *animation_style.transition_timing_function.0.get_mod(transition_index),
            duration: *animation_style.transition_duration.0.get_mod(transition_index),
        };
        if property_animation.does_not_animate() {
            None
        } else {
            Some(property_animation)
        }
    }

    pub fn update(&self, style: &mut ComputedValues, time: f64) {
        let progress = match self.timing_function {
            TransitionTimingFunction::CubicBezier(p1, p2) => {
                // See `WebCore::AnimationBase::solveEpsilon(double)` in WebKit.
                let epsilon = 1.0 / (200.0 * self.duration.seconds());
                Bezier::new(p1, p2).solve(time, epsilon)
            }
            TransitionTimingFunction::Steps(steps, StartEnd::Start) => {
                (time * (steps as f64)).ceil() / (steps as f64)
            }
            TransitionTimingFunction::Steps(steps, StartEnd::End) => {
                (time * (steps as f64)).floor() / (steps as f64)
            }
        };
        match self.property {
            AnimatedProperty::Top(ref start, ref end) => {
                if let Some(value) = start.interpolate(end, progress) {
                    style.mutate_positionoffsets().top = value
                }
            }
            AnimatedProperty::Right(ref start, ref end) => {
                if let Some(value) = start.interpolate(end, progress) {
                    style.mutate_positionoffsets().right = value
                }
            }
            AnimatedProperty::Bottom(ref start, ref end) => {
                if let Some(value) = start.interpolate(end, progress) {
                    style.mutate_positionoffsets().bottom = value
                }
            }
            AnimatedProperty::Left(ref start, ref end) => {
                if let Some(value) = start.interpolate(end, progress) {
                    style.mutate_positionoffsets().left = value
                }
            }
        }
    }

    #[inline]
    fn does_not_animate(&self) -> bool {
        self.property.does_not_animate() || self.duration == Time(0.0)
    }
}

#[derive(Copy, Clone, Debug)]
enum AnimatedProperty {
    Top(LengthOrPercentageOrAuto, LengthOrPercentageOrAuto),
    Right(LengthOrPercentageOrAuto, LengthOrPercentageOrAuto),
    Bottom(LengthOrPercentageOrAuto, LengthOrPercentageOrAuto),
    Left(LengthOrPercentageOrAuto, LengthOrPercentageOrAuto),
}

impl AnimatedProperty {
    #[inline]
    fn does_not_animate(&self) -> bool {
        match *self {
            AnimatedProperty::Top(ref a, ref b) |
            AnimatedProperty::Right(ref a, ref b) |
            AnimatedProperty::Bottom(ref a, ref b) |
            AnimatedProperty::Left(ref a, ref b) => a == b,
        }
    }
}

trait Interpolate {
    fn interpolate(&self, other: &Self, time: f64) -> Option<Self>;
}

impl Interpolate for Au {
    #[inline]
    fn interpolate(&self, other: &Au, time: f64) -> Option<Au> {
        Some(Au((self.0 as f64 + (other.0 as f64 - self.0 as f64) * time).round() as i32))
    }
}

impl Interpolate for f64 {
    #[inline]
    fn interpolate(&self, other: &f64, time: f64) -> Option<f64> {
        Some(*self + (*other - *self) * time)
    }
}

impl Interpolate for LengthOrPercentageOrAuto {
    #[inline]
    fn interpolate(&self, other: &LengthOrPercentageOrAuto, time: f64)
                   -> Option<LengthOrPercentageOrAuto> {
        match (*self, *other) {
            (LengthOrPercentageOrAuto::Length(ref this),
             LengthOrPercentageOrAuto::Length(ref other)) => {
                this.interpolate(other, time).and_then(|value| {
                    Some(LengthOrPercentageOrAuto::Length(value))
                })
            }
            (LengthOrPercentageOrAuto::Percentage(ref this),
             LengthOrPercentageOrAuto::Percentage(ref other)) => {
                this.interpolate(other, time).and_then(|value| {
                    Some(LengthOrPercentageOrAuto::Percentage(value))
                })
            }
            (LengthOrPercentageOrAuto::Auto, LengthOrPercentageOrAuto::Auto) => {
                Some(LengthOrPercentageOrAuto::Auto)
            }
            (_, _) => None,
        }
    }
}

/// Accesses an element of an array, "wrapping around" using modular arithmetic. This is needed
/// to handle values of differing lengths according to CSS-TRANSITIONS § 2.
pub trait GetMod {
    type Item;
    fn get_mod(&self, i: usize) -> &Self::Item;
}

impl<T> GetMod for Vec<T> {
    type Item = T;
    fn get_mod(&self, i: usize) -> &T {
        &(*self)[i % self.len()]
    }
}

