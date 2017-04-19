/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS transitions and animations.
#![deny(missing_docs)]

use Atom;
use bezier::Bezier;
use context::SharedStyleContext;
use dom::{OpaqueNode, UnsafeNode};
use euclid::point::Point2D;
use font_metrics::FontMetricsProvider;
use keyframes::{KeyframesStep, KeyframesStepValue};
use properties::{self, CascadeFlags, ComputedValues, Importance};
use properties::animated_properties::{AnimatedProperty, TransitionProperty};
use properties::longhands::animation_direction::computed_value::single_value::T as AnimationDirection;
use properties::longhands::animation_iteration_count::single_value::computed_value::T as AnimationIterationCount;
use properties::longhands::animation_play_state::computed_value::single_value::T as AnimationPlayState;
use properties::longhands::transition_timing_function::single_value::computed_value::StartEnd;
use properties::longhands::transition_timing_function::single_value::computed_value::T as TransitionTimingFunction;
use std::sync::Arc;
use std::sync::mpsc::Sender;
use timer::Timer;
use values::computed::Time;

/// This structure represents a keyframes animation current iteration state.
///
/// If the iteration count is infinite, there's no other state, otherwise we
/// have to keep track the current iteration and the max iteration count.
#[derive(Debug, Clone)]
pub enum KeyframesIterationState {
    /// Infinite iterations, so no need to track a state.
    Infinite,
    /// Current and max iterations.
    Finite(f32, f32),
}

/// This structure represents wether an animation is actually running.
///
/// An animation can be running, or paused at a given time.
#[derive(Debug, Clone)]
pub enum KeyframesRunningState {
    /// This animation is paused. The inner field is the percentage of progress
    /// when it was paused, from 0 to 1.
    Paused(f64),
    /// This animation is actually running.
    Running,
}

/// This structure represents the current keyframe animation state, i.e., the
/// duration, the current and maximum iteration count, and the state (either
/// playing or paused).
// TODO: unify the use of f32/f64 in this file.
#[derive(Debug, Clone)]
pub struct KeyframesAnimationState {
    /// The time this animation started at.
    pub started_at: f64,
    /// The duration of this animation.
    pub duration: f64,
    /// The delay of the animation.
    pub delay: f64,
    /// The current iteration state for the animation.
    pub iteration_state: KeyframesIterationState,
    /// Werther this animation is paused.
    pub running_state: KeyframesRunningState,
    /// The declared animation direction of this animation.
    pub direction: AnimationDirection,
    /// The current animation direction. This can only be `normal` or `reverse`.
    pub current_direction: AnimationDirection,
    /// Werther this keyframe animation is outdated due to a restyle.
    pub expired: bool,
    /// The original cascade style, needed to compute the generated keyframes of
    /// the animation.
    pub cascade_style: Arc<ComputedValues>,
}

impl KeyframesAnimationState {
    /// Performs a tick in the animation state, i.e., increments the counter of
    /// the current iteration count, updates times and then toggles the
    /// direction if appropriate.
    ///
    /// Returns true if the animation should keep running.
    pub fn tick(&mut self) -> bool {
        debug!("KeyframesAnimationState::tick");
        debug_assert!(!self.expired);

        self.started_at += self.duration + self.delay;
        match self.running_state {
            // If it's paused, don't update direction or iteration count.
            KeyframesRunningState::Paused(_) => return true,
            KeyframesRunningState::Running => {},
        }

        if let KeyframesIterationState::Finite(ref mut current, ref max) = self.iteration_state {
            *current += 1.0;
            // NB: This prevent us from updating the direction, which might be
            // needed for the correct handling of animation-fill-mode.
            if *current >= *max {
                return false;
            }
        }

        // Update the next iteration direction if applicable.
        match self.direction {
            AnimationDirection::alternate |
            AnimationDirection::alternate_reverse => {
                self.current_direction = match self.current_direction {
                    AnimationDirection::normal => AnimationDirection::reverse,
                    AnimationDirection::reverse => AnimationDirection::normal,
                    _ => unreachable!(),
                };
            }
            _ => {},
        }

        true
    }

    /// Updates the appropiate state from other animation.
    ///
    /// This happens when an animation is re-submitted to layout, presumably
    /// because of an state change.
    ///
    /// There are some bits of state we can't just replace, over all taking in
    /// account times, so here's that logic.
    pub fn update_from_other(&mut self,
                             other: &Self,
                             timer: &Timer) {
        use self::KeyframesRunningState::*;

        debug!("KeyframesAnimationState::update_from_other({:?}, {:?})", self, other);

        // NB: We shall not touch the started_at field, since we don't want to
        // restart the animation.
        let old_started_at = self.started_at;
        let old_duration = self.duration;
        let old_direction = self.current_direction;
        let old_running_state = self.running_state.clone();
        let old_iteration_state = self.iteration_state.clone();
        *self = other.clone();

        let mut new_started_at = old_started_at;

        // If we're unpausing the animation, fake the start time so we seem to
        // restore it.
        //
        // If the animation keeps paused, keep the old value.
        //
        // If we're pausing the animation, compute the progress value.
        match (&mut self.running_state, old_running_state) {
            (&mut Running, Paused(progress))
                => new_started_at = timer.seconds() - (self.duration * progress),
            (&mut Paused(ref mut new), Paused(old))
                => *new = old,
            (&mut Paused(ref mut progress), Running)
                => *progress = (timer.seconds() - old_started_at) / old_duration,
            _ => {},
        }

        // Don't update the iteration count, just the iteration limit.
        // TODO: see how changing the limit affects rendering in other browsers.
        // We might need to keep the iteration count even when it's infinite.
        match (&mut self.iteration_state, old_iteration_state) {
            (&mut KeyframesIterationState::Finite(ref mut iters, _), KeyframesIterationState::Finite(old_iters, _))
                => *iters = old_iters,
            _ => {}
        }

        self.current_direction = old_direction;
        self.started_at = new_started_at;
    }

    #[inline]
    fn is_paused(&self) -> bool {
        match self.running_state {
            KeyframesRunningState::Paused(..) => true,
            KeyframesRunningState::Running => false,
        }
    }
}

/// State relating to an animation.
#[derive(Clone, Debug)]
pub enum Animation {
    /// A transition is just a single frame triggered at a time, with a reflow.
    ///
    /// the f64 field is the start time as returned by `time::precise_time_s()`.
    ///
    /// The `bool` field is werther this animation should no longer run.
    Transition(OpaqueNode, UnsafeNode, f64, AnimationFrame, bool),
    /// A keyframes animation is identified by a name, and can have a
    /// node-dependent state (i.e. iteration count, etc.).
    Keyframes(OpaqueNode, Atom, KeyframesAnimationState),
}

impl Animation {
    /// Mark this animation as expired.
    #[inline]
    pub fn mark_as_expired(&mut self) {
        debug_assert!(!self.is_expired());
        match *self {
            Animation::Transition(_, _, _, _, ref mut expired) => *expired = true,
            Animation::Keyframes(_, _, ref mut state) => state.expired = true,
        }
    }

    /// Whether this animation is expired.
    #[inline]
    pub fn is_expired(&self) -> bool {
        match *self {
            Animation::Transition(_, _, _, _, expired) => expired,
            Animation::Keyframes(_, _, ref state) => state.expired,
        }
    }

    /// The opaque node that owns the animation.
    #[inline]
    pub fn node(&self) -> &OpaqueNode {
        match *self {
            Animation::Transition(ref node, _, _, _, _) => node,
            Animation::Keyframes(ref node, _, _) => node,
        }
    }

    /// Whether this animation is paused. A transition can never be paused.
    #[inline]
    pub fn is_paused(&self) -> bool {
        match *self {
            Animation::Transition(..) => false,
            Animation::Keyframes(_, _, ref state) => state.is_paused(),
        }
    }
}


/// A single animation frame of a single property.
#[derive(Debug, Clone)]
pub struct AnimationFrame {
    /// A description of the property animation that is occurring.
    pub property_animation: PropertyAnimation,
    /// The duration of the animation. This is either relative in the keyframes
    /// case (a number between 0 and 1), or absolute in the transition case.
    pub duration: f64,
}

/// Represents an animation for a given property.
#[derive(Debug, Clone)]
pub struct PropertyAnimation {
    property: AnimatedProperty,
    timing_function: TransitionTimingFunction,
    duration: Time, // TODO: isn't this just repeated?
}

impl PropertyAnimation {
    /// Returns the given property name.
    pub fn property_name(&self) -> &'static str {
        self.property.name()
    }

    /// Creates a new property animation for the given transition index and old
    /// and new styles.  Any number of animations may be returned, from zero (if
    /// the property did not animate) to one (for a single transition property)
    /// to arbitrarily many (for `all`).
    pub fn from_transition(transition_index: usize,
                           old_style: &ComputedValues,
                           new_style: &mut ComputedValues)
                           -> Vec<PropertyAnimation> {
        let mut result = vec![];
        let box_style = new_style.get_box();
        let transition_property = box_style.transition_property_at(transition_index);
        let timing_function = box_style.transition_timing_function_mod(transition_index);
        let duration = box_style.transition_duration_mod(transition_index);

        if transition_property.is_shorthand() {
            return transition_property.longhands().iter().filter_map(|transition_property| {
                PropertyAnimation::from_transition_property(*transition_property,
                                                            timing_function,
                                                            duration,
                                                            old_style,
                                                            new_style)
            }).collect();
        }

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
                                old_style: &ComputedValues,
                                new_style: &ComputedValues)
                                -> Option<PropertyAnimation> {
        debug_assert!(!transition_property.is_shorthand() &&
                      transition_property != TransitionProperty::All);
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

    /// Update the given animation at a given point of progress.
    pub fn update(&self, style: &mut ComputedValues, time: f64) {
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
        self.property.does_animate() && self.duration.seconds() != 0.0
    }

    /// Whether this animation has the same end value as another one.
    #[inline]
    pub fn has_the_same_end_value_as(&self, other: &Self) -> bool {
        self.property.has_the_same_end_value_as(&other.property)
    }
}

/// Inserts transitions into the queue of running animations as applicable for
/// the given style difference. This is called from the layout worker threads.
/// Returns true if any animations were kicked off and false otherwise.
//
// TODO(emilio): Take rid of this mutex splitting SharedLayoutContex into a
// cloneable part and a non-cloneable part..
pub fn start_transitions_if_applicable(new_animations_sender: &Sender<Animation>,
                                       opaque_node: OpaqueNode,
                                       unsafe_node: UnsafeNode,
                                       old_style: &ComputedValues,
                                       new_style: &mut Arc<ComputedValues>,
                                       timer: &Timer,
                                       possibly_expired_animations: &[PropertyAnimation])
                                       -> bool {
    let mut had_animations = false;
    for i in 0..new_style.get_box().transition_property_count() {
        // Create any property animations, if applicable.
        let property_animations = PropertyAnimation::from_transition(i,
                                                                     old_style,
                                                                     Arc::make_mut(new_style));
        for property_animation in property_animations {
            // Per [1], don't trigger a new transition if the end state for that transition is
            // the same as that of a transition that's already running on the same node.
            //
            // [1]: https://drafts.csswg.org/css-transitions/#starting
            if possibly_expired_animations.iter().any(|animation| {
                    animation.has_the_same_end_value_as(&property_animation)
                }) {
                continue
            }

            // Set the property to the initial value.
            // NB: get_mut is guaranteed to succeed since we called make_mut()
            // above.
            property_animation.update(Arc::get_mut(new_style).unwrap(), 0.0);

            // Kick off the animation.
            let box_style = new_style.get_box();
            let now = timer.seconds();
            let start_time =
                now + (box_style.transition_delay_mod(i).seconds() as f64);
            new_animations_sender
                .send(Animation::Transition(opaque_node, unsafe_node, start_time, AnimationFrame {
                    duration: box_style.transition_duration_mod(i).seconds() as f64,
                    property_animation: property_animation,
                }, /* is_expired = */ false)).unwrap();

            had_animations = true;
        }
    }

    had_animations
}

fn compute_style_for_animation_step(context: &SharedStyleContext,
                                    step: &KeyframesStep,
                                    previous_style: &ComputedValues,
                                    style_from_cascade: &ComputedValues,
                                    font_metrics_provider: &FontMetricsProvider)
                                    -> ComputedValues {
    match step.value {
        KeyframesStepValue::ComputedValues => style_from_cascade.clone(),
        KeyframesStepValue::Declarations { block: ref declarations } => {
            let guard = declarations.read_with(context.guards.author);

            // No !important in keyframes.
            debug_assert!(guard.declarations().iter()
                            .all(|&(_, importance)| importance == Importance::Normal));

            let iter = || {
                guard.declarations().iter().rev().map(|&(ref decl, _importance)| decl)
            };

            let computed =
                properties::apply_declarations(&context.stylist.device,
                                               /* is_root = */ false,
                                               iter,
                                               previous_style,
                                               previous_style,
                                               /* cascade_info = */ None,
                                               &*context.error_reporter,
                                               font_metrics_provider,
                                               CascadeFlags::empty());
            computed
        }
    }
}

/// Triggers animations for a given node looking at the animation property
/// values.
pub fn maybe_start_animations(context: &SharedStyleContext,
                              new_animations_sender: &Sender<Animation>,
                              node: OpaqueNode,
                              new_style: &Arc<ComputedValues>)
                              -> bool {
    let mut had_animations = false;

    let box_style = new_style.get_box();
    for (i, name) in box_style.animation_name_iter().enumerate() {
        debug!("maybe_start_animations: name={}", name);
        let total_duration = box_style.animation_duration_mod(i).seconds();
        if total_duration == 0. {
            continue
        }

        if let Some(ref anim) = context.stylist.animations().get(&name.0) {
            debug!("maybe_start_animations: animation {} found", name);

            // If this animation doesn't have any keyframe, we can just continue
            // without submitting it to the compositor, since both the first and
            // the second keyframes would be synthetised from the computed
            // values.
            if anim.steps.is_empty() {
                continue;
            }

            let delay = box_style.animation_delay_mod(i).seconds();
            let now = context.timer.seconds();
            let animation_start = now + delay as f64;
            let duration = box_style.animation_duration_mod(i).seconds();
            let iteration_state = match box_style.animation_iteration_count_mod(i) {
                AnimationIterationCount::Infinite => KeyframesIterationState::Infinite,
                AnimationIterationCount::Number(n) => KeyframesIterationState::Finite(0.0, n),
            };

            let animation_direction = box_style.animation_direction_mod(i);

            let initial_direction = match animation_direction {
                AnimationDirection::normal |
                AnimationDirection::alternate => AnimationDirection::normal,
                AnimationDirection::reverse |
                AnimationDirection::alternate_reverse => AnimationDirection::reverse,
            };

            let running_state = match box_style.animation_play_state_mod(i) {
                AnimationPlayState::paused => KeyframesRunningState::Paused(0.),
                AnimationPlayState::running => KeyframesRunningState::Running,
            };


            new_animations_sender
                .send(Animation::Keyframes(node, name.0.clone(), KeyframesAnimationState {
                    started_at: animation_start,
                    duration: duration as f64,
                    delay: delay as f64,
                    iteration_state: iteration_state,
                    running_state: running_state,
                    direction: animation_direction,
                    current_direction: initial_direction,
                    expired: false,
                    cascade_style: new_style.clone(),
                })).unwrap();
            had_animations = true;
        }
    }

    had_animations
}

/// Updates a given computed style for a given animation frame. Returns a bool
/// representing if the style was indeed updated.
pub fn update_style_for_animation_frame(mut new_style: &mut Arc<ComputedValues>,
                                        now: f64,
                                        start_time: f64,
                                        frame: &AnimationFrame) -> bool {
    let mut progress = (now - start_time) / frame.duration;
    if progress > 1.0 {
        progress = 1.0
    }

    if progress <= 0.0 {
        return false;
    }

    frame.property_animation.update(Arc::make_mut(&mut new_style), progress);

    true
}
/// Updates a single animation and associated style based on the current time.
/// If `damage` is provided, inserts the appropriate restyle damage.
pub fn update_style_for_animation(context: &SharedStyleContext,
                                  animation: &Animation,
                                  style: &mut Arc<ComputedValues>,
                                  font_metrics_provider: &FontMetricsProvider) {
    debug!("update_style_for_animation: entering");
    debug_assert!(!animation.is_expired());

    match *animation {
        Animation::Transition(_, _, start_time, ref frame, _) => {
            debug!("update_style_for_animation: transition found");
            let now = context.timer.seconds();
            let mut new_style = (*style).clone();
            let updated_style = update_style_for_animation_frame(&mut new_style,
                                                                 now, start_time,
                                                                 frame);
            if updated_style {
                *style = new_style
            }
        }
        Animation::Keyframes(_, ref name, ref state) => {
            debug!("update_style_for_animation: animation found: \"{}\", {:?}", name, state);
            let duration = state.duration;
            let started_at = state.started_at;

            let now = match state.running_state {
                KeyframesRunningState::Running => context.timer.seconds(),
                KeyframesRunningState::Paused(progress) => started_at + duration * progress,
            };

            let animation = match context.stylist.animations().get(name) {
                None => {
                    warn!("update_style_for_animation: Animation {:?} not found", name);
                    return;
                }
                Some(animation) => animation,
            };

            debug_assert!(!animation.steps.is_empty());

            let maybe_index = style.get_box()
                                   .animation_name_iter()
                                   .position(|animation_name| *name == animation_name.0);

            let index = match maybe_index {
                Some(index) => index,
                None => {
                    warn!("update_style_for_animation: Animation {:?} not found in style", name);
                    return;
                }
            };

            let total_duration = style.get_box().animation_duration_mod(index).seconds() as f64;
            if total_duration == 0. {
                debug!("update_style_for_animation: zero duration for animation {:?}", name);
                return;
            }

            let mut total_progress = (now - started_at) / total_duration;
            if total_progress < 0. {
                warn!("Negative progress found for animation {:?}", name);
                return;
            }
            if total_progress > 1. {
                total_progress = 1.;
            }

            debug!("update_style_for_animation: anim \"{}\", steps: {:?}, state: {:?}, progress: {}",
                   name, animation.steps, state, total_progress);

            // Get the target and the last keyframe position.
            let last_keyframe_position;
            let target_keyframe_position;
            match state.current_direction {
                AnimationDirection::normal => {
                    target_keyframe_position =
                        animation.steps.iter().position(|step| {
                            total_progress as f32 <= step.start_percentage.0
                        });

                    last_keyframe_position = target_keyframe_position.and_then(|pos| {
                        if pos != 0 { Some(pos - 1) } else { None }
                    }).unwrap_or(0);
                }
                AnimationDirection::reverse => {
                    target_keyframe_position =
                        animation.steps.iter().rev().position(|step| {
                            total_progress as f32 <= 1. - step.start_percentage.0
                        }).map(|pos| animation.steps.len() - pos - 1);

                    last_keyframe_position = target_keyframe_position.and_then(|pos| {
                        if pos != animation.steps.len() - 1 { Some(pos + 1) } else { None }
                    }).unwrap_or(animation.steps.len() - 1);
                }
                _ => unreachable!(),
            }

            debug!("update_style_for_animation: keyframe from {:?} to {:?}",
                   last_keyframe_position, target_keyframe_position);

            let target_keyframe = match target_keyframe_position {
                Some(target) => &animation.steps[target],
                None => {
                    warn!("update_style_for_animation: No current keyframe found for animation \"{}\" at progress {}",
                          name, total_progress);
                    return;
                }
            };

            let last_keyframe = &animation.steps[last_keyframe_position];

            let relative_timespan = (target_keyframe.start_percentage.0 - last_keyframe.start_percentage.0).abs();
            let relative_duration = relative_timespan as f64 * duration;
            let last_keyframe_ended_at = match state.current_direction {
                AnimationDirection::normal => {
                    state.started_at + (total_duration * last_keyframe.start_percentage.0 as f64)
                }
                AnimationDirection::reverse => {
                    state.started_at + (total_duration * (1. - last_keyframe.start_percentage.0 as f64))
                }
                _ => unreachable!(),
            };
            let relative_progress = (now - last_keyframe_ended_at) / relative_duration;

            // TODO: How could we optimise it? Is it such a big deal?
            let from_style = compute_style_for_animation_step(context,
                                                              last_keyframe,
                                                              &**style,
                                                              &state.cascade_style,
                                                              font_metrics_provider);

            // NB: The spec says that the timing function can be overwritten
            // from the keyframe style.
            let mut timing_function = style.get_box().animation_timing_function_mod(index);
            if last_keyframe.declared_timing_function {
                // NB: animation_timing_function can never be empty, always has
                // at least the default value (`ease`).
                timing_function = from_style.get_box().animation_timing_function_at(0);
            }

            let target_style = compute_style_for_animation_step(context,
                                                                target_keyframe,
                                                                &from_style,
                                                                &state.cascade_style,
                                                                font_metrics_provider);

            let mut new_style = (*style).clone();

            for transition_property in &animation.properties_changed {
                debug!("update_style_for_animation: scanning prop {:?} for animation \"{}\"",
                       transition_property, name);
                match PropertyAnimation::from_transition_property(*transition_property,
                                                                  timing_function,
                                                                  Time::from_seconds(relative_duration as f32),
                                                                  &from_style,
                                                                  &target_style) {
                    Some(property_animation) => {
                        debug!("update_style_for_animation: got property animation for prop {:?}", transition_property);
                        debug!("update_style_for_animation: {:?}", property_animation);
                        property_animation.update(Arc::make_mut(&mut new_style), relative_progress);
                    }
                    None => {
                        debug!("update_style_for_animation: property animation {:?} not animating",
                               transition_property);
                    }
                }
            }

            debug!("update_style_for_animation: got style change in animation \"{}\"", name);
            *style = new_style;
        }
    }
}

/// Update the style in the node when it finishes.
pub fn complete_expired_transitions(node: OpaqueNode, style: &mut Arc<ComputedValues>,
                                    context: &SharedStyleContext) -> bool {
    let had_animations_to_expire;
    {
        let all_expired_animations = context.expired_animations.read();
        let animations_to_expire = all_expired_animations.get(&node);
        had_animations_to_expire = animations_to_expire.is_some();
        if let Some(ref animations) = animations_to_expire {
            for animation in *animations {
                // TODO: support animation-fill-mode
                if let Animation::Transition(_, _, _, ref frame, _) = *animation {
                    frame.property_animation.update(Arc::make_mut(style), 1.0);
                }
            }
        }
    }

    if had_animations_to_expire {
        context.expired_animations.write().remove(&node);
    }

    had_animations_to_expire
}
