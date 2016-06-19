/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS transitions and animations.

use app_units::Au;
use bezier::Bezier;
use euclid::point::Point2D;
use dom::{OpaqueNode, TRestyleDamage};
use keyframes::KeyframesAnimation;
use keyframes::KeyframesStep;
use properties::animated_properties::{AnimatedProperty, TransitionProperty};
use properties::longhands::transition_timing_function::computed_value::StartEnd;
use properties::longhands::transition_timing_function::computed_value::TransitionTimingFunction;
use properties::style_struct_traits::Box;
use properties::{ComputedValues, ServoComputedValues};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use time;
use values::computed::Time;
use selector_impl::SelectorImplExt;
use context::SharedStyleContext;
use selectors::matching::DeclarationBlock;
use properties;

#[derive(Clone, Debug)]
pub enum AnimationKind {
    Transition,
    Keyframe,
}

/// State relating to an animation.
#[derive(Clone)]
pub struct Animation {
    /// The kind of animation, either a transition or a keyframe.
    pub kind: AnimationKind,
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
    pub fn from_transition<C: ComputedValues>(transition_index: usize,
                                              old_style: &C,
                                              new_style: &mut C)
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

    fn from_transition_property<C: ComputedValues>(transition_property: TransitionProperty,
                                                   timing_function: TransitionTimingFunction,
                                                   duration: Time,
                                                   old_style: &C,
                                                   new_style: &C)
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

    pub fn update<C: ComputedValues>(&self, style: &mut C, time: f64) {
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
/// to handle [repeatable lists][lists] of differing lengths.
///
/// [lists]: https://drafts.csswg.org/css-transitions/#animtype-repeatable-list
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
        let property_animations = PropertyAnimation::from_transition(i, old_style, new_style);
        for property_animation in property_animations {
            // Set the property to the initial value.
            property_animation.update(new_style, 0.0);

            // Kick off the animation.
            let now = time::precise_time_s();
            let box_style = new_style.as_servo().get_box();
            let start_time =
                now + (box_style.transition_delay.0.get_mod(i).seconds() as f64);
            new_animations_sender.lock().unwrap().send(Animation {
                kind: AnimationKind::Transition,
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

fn compute_style_for_animation_step<Impl: SelectorImplExt>(context: &SharedStyleContext<Impl>,
                                                           step: &KeyframesStep,
                                                           old_style: &Impl::ComputedValues) -> Impl::ComputedValues {
    let declaration_block = DeclarationBlock {
        declarations: step.declarations.clone(),
        source_order: 0,
        specificity: ::std::u32::MAX,
    };
    let (computed, _) = properties::cascade(context.viewport_size,
                                            &[declaration_block],
                                            false,
                                            Some(old_style),
                                            None,
                                            context.error_reporter.clone());
    computed
}

pub fn maybe_start_animations<Impl: SelectorImplExt>(context: &SharedStyleContext<Impl>,
                                                     node: OpaqueNode,
                                                     new_style: &Impl::ComputedValues) -> bool
{
    let mut had_animations = false;

    for (i, name) in new_style.as_servo().get_box().animation_name.0.iter().enumerate() {
        debug!("maybe_start_animations: name={}", name);
        let total_duration = new_style.as_servo().get_box().animation_duration.0.get_mod(i).seconds();
        if total_duration == 0. {
            continue
        }

        // TODO: This should be factored out, too much indentation.
        if let Some(ref animation) = context.stylist.animations().get(&**name) {
            debug!("maybe_start_animations: found animation {}", name);
            had_animations = true;
            let mut last_keyframe_style = compute_style_for_animation_step(context,
                                                                           &animation.steps[0],
                                                                           new_style);
            // Apply the style inmediately. TODO: clone()...
            // *new_style = last_keyframe_style.clone();

            let mut ongoing_animation_percentage = animation.steps[0].duration_percentage.0;
            let delay = new_style.as_servo().get_box().animation_delay.0.get_mod(i).seconds();
            let animation_start = time::precise_time_s() + delay as f64;

            // TODO: We can probably be smarter here and batch steps out or
            // something.
            for step in &animation.steps[1..] {
                for transition_property in &animation.properties_changed {
                    debug!("maybe_start_animations: processing animation prop {:?} for animation {}", transition_property, name);

                    let new_keyframe_style = compute_style_for_animation_step(context,
                                                                              step,
                                                                              &last_keyframe_style);
                    // NB: This will get the previous frame timing function, or
                    // the old one if caught, which is what the spec says.
                    //
                    // We might need to reset to the initial timing function
                    // though.
                    let timing_function =
                        *last_keyframe_style.as_servo()
                                            .get_box().animation_timing_function.0.get_mod(i);

                    let percentage = step.duration_percentage.0;
                    let this_keyframe_duration = total_duration * percentage;
                    if let Some(property_animation) = PropertyAnimation::from_transition_property(*transition_property,
                                                                                                  timing_function,
                                                                                                  Time(this_keyframe_duration),
                                                                                                  &last_keyframe_style,
                                                                                                  &new_keyframe_style) {
                        debug!("maybe_start_animations: got property animation for prop {:?}", transition_property);

                        let relative_start_time = ongoing_animation_percentage * total_duration;
                        let start_time = animation_start + relative_start_time as f64;
                        let end_time = start_time + (relative_start_time + this_keyframe_duration) as f64;
                        context.new_animations_sender.lock().unwrap().send(Animation {
                            kind: AnimationKind::Keyframe,
                            node: node,
                            property_animation: property_animation,
                            start_time: start_time,
                            end_time: end_time,
                        }).unwrap();
                    }

                    last_keyframe_style = new_keyframe_style;
                    ongoing_animation_percentage += percentage;
                }
            }
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
    animation.property_animation.update(Arc::make_mut(&mut new_style), progress);
    if let Some(damage) = damage {
        *damage = *damage | Damage::compute(Some(style), &new_style);
    }

    *style = new_style
}
