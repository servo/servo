/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use bezier::Bezier;
use euclid::point::Point2D;
use dom::{OpaqueNode, TRestyleDamage};
use properties::animated_properties::{AnimatedProperty, TransitionProperty};
use properties::longhands::transition_timing_function::computed_value::StartEnd;
use properties::longhands::transition_timing_function::computed_value::TransitionTimingFunction;
use properties::style_struct_traits::Box;
use properties::{ComputedValues, ServoComputedValues};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use time;
use values::computed::Time;

/// State relating to an animation.
#[derive(Clone)]
pub struct Animation {
    /// An opaque reference to the DOM node participating in the animation.
    pub node: OpaqueNode,
    /// A description of the property animation that is occurring.
    pub property_animation: PropertyAnimation,
    /// The start time of the animation, as returned by `time::precise_time_s()`.
    pub start_time: f64,
    /// The end time of the animation, as returned by `time::precise_time_s()`.
    pub end_time: f64,
}

impl Animation {
    /// Returns the duration of this animation in seconds.
    #[inline]
    pub fn duration(&self) -> f64 {
        self.end_time - self.start_time
    }
}


#[derive(Clone, Debug)]
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
                           old_style: &ServoComputedValues,
                           new_style: &mut ServoComputedValues)
                           -> Vec<PropertyAnimation> {
        let mut result = vec![];
        let box_style = new_style.as_servo().get_box();
        let transition_property = box_style.transition_property.0[transition_index];
        let timing_function = *box_style.transition_timing_function.0.get_mod(transition_index);
        let duration = *box_style.transition_duration.0.get_mod(transition_index);


        if transition_property != TransitionProperty::All {
            if let Some(property_animation) =
                    PropertyAnimation::from_transition_property(transition_property,
                                                                timing_function,
                                                                duration,
                                                                old_style,
                                                                new_style) {
                result.push(property_animation)
            }
            return result
        }

        TransitionProperty::each(|transition_property| {
            if let Some(property_animation) =
                    PropertyAnimation::from_transition_property(transition_property,
                                                                timing_function,
                                                                duration,
                                                                old_style,
                                                                new_style) {
                result.push(property_animation)
            }
        });

        result
    }

    fn from_transition_property(transition_property: TransitionProperty,
                                timing_function: TransitionTimingFunction,
                                duration: Time,
                                old_style: &ServoComputedValues,
                                new_style: &ServoComputedValues)
                                -> Option<PropertyAnimation> {
        let animated_property = AnimatedProperty::from_transition_property(&transition_property,
                                                                           old_style,
                                                                           new_style);

        let property_animation = PropertyAnimation {
            property: animated_property,
            timing_function: timing_function,
            duration: duration,
        };

        if property_animation.does_animate() {
            Some(property_animation)
        } else {
            None
        }
    }

    pub fn update(&self, style: &mut ServoComputedValues, time: f64) {
        let progress = match self.timing_function {
            TransitionTimingFunction::CubicBezier(p1, p2) => {
                // See `WebCore::AnimationBase::solveEpsilon(double)` in WebKit.
                let epsilon = 1.0 / (200.0 * (self.duration.seconds() as f64));
                Bezier::new(Point2D::new(p1.x as f64, p1.y as f64),
                            Point2D::new(p2.x as f64, p2.y as f64)).solve(time, epsilon)
            }
            TransitionTimingFunction::Steps(steps, StartEnd::Start) => {
                (time * (steps as f64)).ceil() / (steps as f64)
            }
            TransitionTimingFunction::Steps(steps, StartEnd::End) => {
                (time * (steps as f64)).floor() / (steps as f64)
            }
        };

        self.property.update(style, progress);
    }

    #[inline]
    fn does_animate(&self) -> bool {
        self.property.does_animate() && self.duration != Time(0.0)
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
    #[inline]
    fn get_mod(&self, i: usize) -> &T {
        &(*self)[i % self.len()]
    }
}

/// Inserts transitions into the queue of running animations as applicable for the given style
/// difference. This is called from the layout worker threads. Returns true if any animations were
/// kicked off and false otherwise.
pub fn start_transitions_if_applicable<C: ComputedValues>(new_animations_sender: &Mutex<Sender<Animation>>,
                                                          node: OpaqueNode,
                                                          old_style: &C,
                                                          new_style: &mut C)
                                                          -> bool {
    let mut had_animations = false;
    for i in 0..new_style.get_box().transition_count() {
        // Create any property animations, if applicable.
        let property_animations = PropertyAnimation::from_transition(i, old_style.as_servo(), new_style.as_servo_mut());
        for property_animation in property_animations {
            // Set the property to the initial value.
            property_animation.update(new_style.as_servo_mut(), 0.0);

            // Kick off the animation.
            let now = time::precise_time_s();
            let box_style = new_style.as_servo().get_box();
            let start_time =
                now + (box_style.transition_delay.0.get_mod(i).seconds() as f64);
            new_animations_sender.lock().unwrap().send(Animation {
                node: node,
                property_animation: property_animation,
                start_time: start_time,
                end_time: start_time +
                    (box_style.transition_duration.0.get_mod(i).seconds() as f64),
            }).unwrap();

            had_animations = true
        }
    }

    had_animations
}

/// Updates a single animation and associated style based on the current time. If `damage` is
/// provided, inserts the appropriate restyle damage.
pub fn update_style_for_animation<Damage: TRestyleDamage>(animation: &Animation,
                                                          style: &mut Arc<Damage::ConcreteComputedValues>,
                                                          damage: Option<&mut Damage>) {
    let now = time::precise_time_s();
    let mut progress = (now - animation.start_time) / animation.duration();
    if progress > 1.0 {
        progress = 1.0
    }
    if progress <= 0.0 {
        return
    }

    let mut new_style = (*style).clone();
    animation.property_animation.update(Arc::make_mut(&mut new_style).as_servo_mut(), progress);
    if let Some(damage) = damage {
        *damage = *damage | Damage::compute(Some(style), &new_style);
    }

    *style = new_style
}
