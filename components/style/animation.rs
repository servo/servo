/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! CSS transitions and animations.

// NOTE(emilio): This code isn't really executed in Gecko, but we don't want to
// compile it out so that people remember it exists.

use crate::context::{CascadeInputs, SharedStyleContext};
use crate::dom::{OpaqueNode, TDocument, TElement, TNode};
use crate::properties::animated_properties::{AnimationValue, AnimationValueMap};
use crate::properties::longhands::animation_direction::computed_value::single_value::T as AnimationDirection;
use crate::properties::longhands::animation_fill_mode::computed_value::single_value::T as AnimationFillMode;
use crate::properties::longhands::animation_play_state::computed_value::single_value::T as AnimationPlayState;
use crate::properties::AnimationDeclarations;
use crate::properties::{
    ComputedValues, Importance, LonghandId, LonghandIdSet, PropertyDeclarationBlock,
    PropertyDeclarationId,
};
use crate::rule_tree::CascadeLevel;
use crate::selector_parser::PseudoElement;
use crate::shared_lock::{Locked, SharedRwLock};
use crate::style_resolver::StyleResolverForElement;
use crate::stylesheets::keyframes_rule::{KeyframesAnimation, KeyframesStep, KeyframesStepValue};
use crate::stylesheets::layer_rule::LayerOrder;
use crate::values::animated::{Animate, Procedure};
use crate::values::computed::{Time, TimingFunction};
use crate::values::generics::box_::AnimationIterationCount;
use crate::values::generics::easing::BeforeFlag;
use crate::Atom;
use fxhash::FxHashMap;
use parking_lot::RwLock;
use servo_arc::Arc;
use std::fmt;

/// Represents an animation for a given property.
#[derive(Clone, Debug, MallocSizeOf)]
pub struct PropertyAnimation {
    /// The value we are animating from.
    from: AnimationValue,

    /// The value we are animating to.
    to: AnimationValue,

    /// The timing function of this `PropertyAnimation`.
    timing_function: TimingFunction,

    /// The duration of this `PropertyAnimation` in seconds.
    pub duration: f64,
}

impl PropertyAnimation {
    /// Returns the given property longhand id.
    pub fn property_id(&self) -> LonghandId {
        debug_assert_eq!(self.from.id(), self.to.id());
        self.from.id()
    }

    fn from_longhand(
        longhand: LonghandId,
        timing_function: TimingFunction,
        duration: Time,
        old_style: &ComputedValues,
        new_style: &ComputedValues,
    ) -> Option<PropertyAnimation> {
        // FIXME(emilio): Handle the case where old_style and new_style's writing mode differ.
        let longhand = longhand.to_physical(new_style.writing_mode);
        let from = AnimationValue::from_computed_values(longhand, old_style)?;
        let to = AnimationValue::from_computed_values(longhand, new_style)?;
        let duration = duration.seconds() as f64;

        if from == to || duration == 0.0 {
            return None;
        }

        Some(PropertyAnimation {
            from,
            to,
            timing_function,
            duration,
        })
    }

    /// The output of the timing function given the progress ration of this animation.
    fn timing_function_output(&self, progress: f64) -> f64 {
        let epsilon = 1. / (200. * self.duration);
        // FIXME: Need to set the before flag correctly.
        // In order to get the before flag, we have to know the current animation phase
        // and whether the iteration is reversed. For now, we skip this calculation
        // by treating as if the flag is unset at all times.
        // https://drafts.csswg.org/css-easing/#step-timing-function-algo
        self.timing_function.calculate_output(progress, BeforeFlag::Unset, epsilon)
    }

    /// Update the given animation at a given point of progress.
    fn calculate_value(&self, progress: f64) -> Result<AnimationValue, ()> {
        let procedure = Procedure::Interpolate {
            progress: self.timing_function_output(progress),
        };
        self.from.animate(&self.to, procedure)
    }
}

/// This structure represents the state of an animation.
#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub enum AnimationState {
    /// The animation has been created, but is not running yet. This state
    /// is also used when an animation is still in the first delay phase.
    Pending,
    /// This animation is currently running.
    Running,
    /// This animation is paused. The inner field is the percentage of progress
    /// when it was paused, from 0 to 1.
    Paused(f64),
    /// This animation has finished.
    Finished,
    /// This animation has been canceled.
    Canceled,
}

impl AnimationState {
    /// Whether or not this state requires its owning animation to be ticked.
    fn needs_to_be_ticked(&self) -> bool {
        *self == AnimationState::Running || *self == AnimationState::Pending
    }
}

/// This structure represents a keyframes animation current iteration state.
///
/// If the iteration count is infinite, there's no other state, otherwise we
/// have to keep track the current iteration and the max iteration count.
#[derive(Clone, Debug, MallocSizeOf)]
pub enum KeyframesIterationState {
    /// Infinite iterations with the current iteration count.
    Infinite(f64),
    /// Current and max iterations.
    Finite(f64, f64),
}

/// A temporary data structure used when calculating ComputedKeyframes for an
/// animation. This data structure is used to collapse information for steps
/// which may be spread across multiple keyframe declarations into a single
/// instance per `start_percentage`.
struct IntermediateComputedKeyframe {
    declarations: PropertyDeclarationBlock,
    timing_function: Option<TimingFunction>,
    start_percentage: f32,
}

impl IntermediateComputedKeyframe {
    fn new(start_percentage: f32) -> Self {
        IntermediateComputedKeyframe {
            declarations: PropertyDeclarationBlock::new(),
            timing_function: None,
            start_percentage,
        }
    }

    /// Walk through all keyframe declarations and combine all declarations with the
    /// same `start_percentage` into individual `IntermediateComputedKeyframe`s.
    fn generate_for_keyframes(
        animation: &KeyframesAnimation,
        context: &SharedStyleContext,
        base_style: &ComputedValues,
    ) -> Vec<Self> {
        let mut intermediate_steps: Vec<Self> = Vec::with_capacity(animation.steps.len());
        let mut current_step = IntermediateComputedKeyframe::new(0.);
        for step in animation.steps.iter() {
            let start_percentage = step.start_percentage.0;
            if start_percentage != current_step.start_percentage {
                let new_step = IntermediateComputedKeyframe::new(start_percentage);
                intermediate_steps.push(std::mem::replace(&mut current_step, new_step));
            }

            current_step.update_from_step(step, context, base_style);
        }
        intermediate_steps.push(current_step);

        // We should always have a first and a last step, even if these are just
        // generated by KeyframesStepValue::ComputedValues.
        debug_assert!(intermediate_steps.first().unwrap().start_percentage == 0.);
        debug_assert!(intermediate_steps.last().unwrap().start_percentage == 1.);

        intermediate_steps
    }

    fn update_from_step(
        &mut self,
        step: &KeyframesStep,
        context: &SharedStyleContext,
        base_style: &ComputedValues,
    ) {
        // Each keyframe declaration may optionally specify a timing function, falling
        // back to the one defined global for the animation.
        let guard = &context.guards.author;
        if let Some(timing_function) = step.get_animation_timing_function(&guard) {
            self.timing_function = Some(timing_function.to_computed_value_without_context());
        }

        let block = match step.value {
            KeyframesStepValue::ComputedValues => return,
            KeyframesStepValue::Declarations { ref block } => block,
        };

        // Filter out !important, non-animatable properties, and the
        // 'display' property (which is only animatable from SMIL).
        let guard = block.read_with(&guard);
        for declaration in guard.normal_declaration_iter() {
            if let PropertyDeclarationId::Longhand(id) = declaration.id() {
                if id == LonghandId::Display {
                    continue;
                }

                if !id.is_animatable() {
                    continue;
                }
            }

            self.declarations.push(
                declaration.to_physical(base_style.writing_mode),
                Importance::Normal,
            );
        }
    }

    fn resolve_style<E>(
        self,
        element: E,
        context: &SharedStyleContext,
        base_style: &Arc<ComputedValues>,
        resolver: &mut StyleResolverForElement<E>,
    ) -> Arc<ComputedValues>
    where
        E: TElement,
    {
        if !self.declarations.any_normal() {
            return base_style.clone();
        }

        let document = element.as_node().owner_doc();
        let locked_block = Arc::new(document.shared_lock().wrap(self.declarations));
        let mut important_rules_changed = false;
        let rule_node = base_style.rules().clone();
        let new_node = context.stylist.rule_tree().update_rule_at_level(
            CascadeLevel::Animations,
            LayerOrder::root(),
            Some(locked_block.borrow_arc()),
            &rule_node,
            &context.guards,
            &mut important_rules_changed,
        );

        if new_node.is_none() {
            return base_style.clone();
        }

        let inputs = CascadeInputs {
            rules: new_node,
            visited_rules: base_style.visited_rules().cloned(),
        };
        resolver
            .cascade_style_and_visited_with_default_parents(inputs)
            .0
    }
}

/// A single computed keyframe for a CSS Animation.
#[derive(Clone, MallocSizeOf)]
struct ComputedKeyframe {
    /// The timing function to use for transitions between this step
    /// and the next one.
    timing_function: TimingFunction,

    /// The starting percentage (a number between 0 and 1) which represents
    /// at what point in an animation iteration this step is.
    start_percentage: f32,

    /// The animation values to transition to and from when processing this
    /// keyframe animation step.
    values: Vec<AnimationValue>,
}

impl ComputedKeyframe {
    fn generate_for_keyframes<E>(
        element: E,
        animation: &KeyframesAnimation,
        context: &SharedStyleContext,
        base_style: &Arc<ComputedValues>,
        default_timing_function: TimingFunction,
        resolver: &mut StyleResolverForElement<E>,
    ) -> Vec<Self>
    where
        E: TElement,
    {
        let mut animating_properties = LonghandIdSet::new();
        for property in animation.properties_changed.iter() {
            debug_assert!(property.is_animatable());
            animating_properties.insert(property.to_physical(base_style.writing_mode));
        }

        let animation_values_from_style: Vec<AnimationValue> = animating_properties
            .iter()
            .map(|property| {
                AnimationValue::from_computed_values(property, &**base_style)
                    .expect("Unexpected non-animatable property.")
            })
            .collect();

        let intermediate_steps =
            IntermediateComputedKeyframe::generate_for_keyframes(animation, context, base_style);

        let mut computed_steps: Vec<Self> = Vec::with_capacity(intermediate_steps.len());
        for (step_index, step) in intermediate_steps.into_iter().enumerate() {
            let start_percentage = step.start_percentage;
            let properties_changed_in_step = step.declarations.longhands().clone();
            let step_timing_function = step.timing_function.clone();
            let step_style = step.resolve_style(element, context, base_style, resolver);
            let timing_function =
                step_timing_function.unwrap_or_else(|| default_timing_function.clone());

            let values = {
                // If a value is not set in a property declaration we use the value from
                // the style for the first and last keyframe. For intermediate ones, we
                // use the value from the previous keyframe.
                //
                // TODO(mrobinson): According to the spec, we should use an interpolated
                // value for properties missing from keyframe declarations.
                let default_values = if start_percentage == 0. || start_percentage == 1.0 {
                    &animation_values_from_style
                } else {
                    debug_assert!(step_index != 0);
                    &computed_steps[step_index - 1].values
                };

                // For each property that is animating, pull the value from the resolved
                // style for this step if it's in one of the declarations. Otherwise, we
                // use the default value from the set we calculated above.
                animating_properties
                    .iter()
                    .zip(default_values.iter())
                    .map(|(longhand, default_value)| {
                        if properties_changed_in_step.contains(longhand) {
                            AnimationValue::from_computed_values(longhand, &step_style)
                                .unwrap_or_else(|| default_value.clone())
                        } else {
                            default_value.clone()
                        }
                    })
                    .collect()
            };

            computed_steps.push(ComputedKeyframe {
                timing_function,
                start_percentage,
                values,
            });
        }
        computed_steps
    }
}

/// A CSS Animation
#[derive(Clone, MallocSizeOf)]
pub struct Animation {
    /// The name of this animation as defined by the style.
    pub name: Atom,

    /// The properties that change in this animation.
    properties_changed: LonghandIdSet,

    /// The computed style for each keyframe of this animation.
    computed_steps: Vec<ComputedKeyframe>,

    /// The time this animation started at, which is the current value of the animation
    /// timeline when this animation was created plus any animation delay.
    pub started_at: f64,

    /// The duration of this animation.
    pub duration: f64,

    /// The delay of the animation.
    pub delay: f64,

    /// The `animation-fill-mode` property of this animation.
    pub fill_mode: AnimationFillMode,

    /// The current iteration state for the animation.
    pub iteration_state: KeyframesIterationState,

    /// Whether this animation is paused.
    pub state: AnimationState,

    /// The declared animation direction of this animation.
    pub direction: AnimationDirection,

    /// The current animation direction. This can only be `normal` or `reverse`.
    pub current_direction: AnimationDirection,

    /// The original cascade style, needed to compute the generated keyframes of
    /// the animation.
    #[ignore_malloc_size_of = "ComputedValues"]
    pub cascade_style: Arc<ComputedValues>,

    /// Whether or not this animation is new and or has already been tracked
    /// by the script thread.
    pub is_new: bool,
}

impl Animation {
    /// Whether or not this animation is cancelled by changes from a new style.
    fn is_cancelled_in_new_style(&self, new_style: &Arc<ComputedValues>) -> bool {
        let new_ui = new_style.get_ui();
        let index = new_ui
            .animation_name_iter()
            .position(|animation_name| Some(&self.name) == animation_name.as_atom());
        let index = match index {
            Some(index) => index,
            None => return true,
        };

        new_ui.animation_duration_mod(index).seconds() == 0.
    }

    /// Given the current time, advances this animation to the next iteration,
    /// updates times, and then toggles the direction if appropriate. Otherwise
    /// does nothing. Returns true if this animation has iterated.
    pub fn iterate_if_necessary(&mut self, time: f64) -> bool {
        if !self.iteration_over(time) {
            return false;
        }

        // Only iterate animations that are currently running.
        if self.state != AnimationState::Running {
            return false;
        }

        if self.on_last_iteration() {
            return false;
        }

        self.iterate();
        true
    }

    fn iterate(&mut self) {
        debug_assert!(!self.on_last_iteration());

        if let KeyframesIterationState::Finite(ref mut current, max) = self.iteration_state {
            *current = (*current + 1.).min(max);
        }

        if let AnimationState::Paused(ref mut progress) = self.state {
            debug_assert!(*progress > 1.);
            *progress -= 1.;
        }

        // Update the next iteration direction if applicable.
        self.started_at += self.duration;
        match self.direction {
            AnimationDirection::Alternate | AnimationDirection::AlternateReverse => {
                self.current_direction = match self.current_direction {
                    AnimationDirection::Normal => AnimationDirection::Reverse,
                    AnimationDirection::Reverse => AnimationDirection::Normal,
                    _ => unreachable!(),
                };
            },
            _ => {},
        }
    }

    /// A number (> 0 and <= 1) which represents the fraction of a full iteration
    /// that the current iteration of the animation lasts. This will be less than 1
    /// if the current iteration is the fractional remainder of a non-integral
    /// iteration count.
    pub fn current_iteration_end_progress(&self) -> f64 {
        match self.iteration_state {
            KeyframesIterationState::Finite(current, max) => (max - current).min(1.),
            KeyframesIterationState::Infinite(_) => 1.,
        }
    }

    /// The duration of the current iteration of this animation which may be less
    /// than the animation duration if it has a non-integral iteration count.
    pub fn current_iteration_duration(&self) -> f64 {
        self.current_iteration_end_progress() * self.duration
    }

    /// Whether or not the current iteration is over. Note that this method assumes that
    /// the animation is still running.
    fn iteration_over(&self, time: f64) -> bool {
        time > (self.started_at + self.current_iteration_duration())
    }

    /// Assuming this animation is running, whether or not it is on the last iteration.
    fn on_last_iteration(&self) -> bool {
        match self.iteration_state {
            KeyframesIterationState::Finite(current, max) => current >= (max - 1.),
            KeyframesIterationState::Infinite(_) => false,
        }
    }

    /// Whether or not this animation has finished at the provided time. This does
    /// not take into account canceling i.e. when an animation or transition is
    /// canceled due to changes in the style.
    pub fn has_ended(&self, time: f64) -> bool {
        if !self.on_last_iteration() {
            return false;
        }

        let progress = match self.state {
            AnimationState::Finished => return true,
            AnimationState::Paused(progress) => progress,
            AnimationState::Running => (time - self.started_at) / self.duration,
            AnimationState::Pending | AnimationState::Canceled => return false,
        };

        progress >= self.current_iteration_end_progress()
    }

    /// Updates the appropiate state from other animation.
    ///
    /// This happens when an animation is re-submitted to layout, presumably
    /// because of an state change.
    ///
    /// There are some bits of state we can't just replace, over all taking in
    /// account times, so here's that logic.
    pub fn update_from_other(&mut self, other: &Self, now: f64) {
        use self::AnimationState::*;

        debug!(
            "KeyframesAnimationState::update_from_other({:?}, {:?})",
            self, other
        );

        // NB: We shall not touch the started_at field, since we don't want to
        // restart the animation.
        let old_started_at = self.started_at;
        let old_duration = self.duration;
        let old_direction = self.current_direction;
        let old_state = self.state.clone();
        let old_iteration_state = self.iteration_state.clone();

        *self = other.clone();

        self.started_at = old_started_at;
        self.current_direction = old_direction;

        // Don't update the iteration count, just the iteration limit.
        // TODO: see how changing the limit affects rendering in other browsers.
        // We might need to keep the iteration count even when it's infinite.
        match (&mut self.iteration_state, old_iteration_state) {
            (
                &mut KeyframesIterationState::Finite(ref mut iters, _),
                KeyframesIterationState::Finite(old_iters, _),
            ) => *iters = old_iters,
            _ => {},
        }

        // Don't pause or restart animations that should remain finished.
        // We call mem::replace because `has_ended(...)` looks at `Animation::state`.
        let new_state = std::mem::replace(&mut self.state, Running);
        if old_state == Finished && self.has_ended(now) {
            self.state = Finished;
        } else {
            self.state = new_state;
        }

        // If we're unpausing the animation, fake the start time so we seem to
        // restore it.
        //
        // If the animation keeps paused, keep the old value.
        //
        // If we're pausing the animation, compute the progress value.
        match (&mut self.state, &old_state) {
            (&mut Pending, &Paused(progress)) => {
                self.started_at = now - (self.duration * progress);
            },
            (&mut Paused(ref mut new), &Paused(old)) => *new = old,
            (&mut Paused(ref mut progress), &Running) => {
                *progress = (now - old_started_at) / old_duration
            },
            _ => {},
        }

        // Try to detect when we should skip straight to the running phase to
        // avoid sending multiple animationstart events.
        if self.state == Pending && self.started_at <= now && old_state != Pending {
            self.state = Running;
        }
    }

    /// Fill in an `AnimationValueMap` with values calculated from this animation at
    /// the given time value.
    fn get_property_declaration_at_time(&self, now: f64, map: &mut AnimationValueMap) {
        debug_assert!(!self.computed_steps.is_empty());

        let total_progress = match self.state {
            AnimationState::Running | AnimationState::Pending | AnimationState::Finished => {
                (now - self.started_at) / self.duration
            },
            AnimationState::Paused(progress) => progress,
            AnimationState::Canceled => return,
        };

        if total_progress < 0. &&
            self.fill_mode != AnimationFillMode::Backwards &&
            self.fill_mode != AnimationFillMode::Both
        {
            return;
        }
        if self.has_ended(now) &&
            self.fill_mode != AnimationFillMode::Forwards &&
            self.fill_mode != AnimationFillMode::Both
        {
            return;
        }
        let total_progress = total_progress
            .min(self.current_iteration_end_progress())
            .max(0.0);

        // Get the indices of the previous (from) keyframe and the next (to) keyframe.
        let next_keyframe_index;
        let prev_keyframe_index;
        let num_steps = self.computed_steps.len();
        match self.current_direction {
            AnimationDirection::Normal => {
                next_keyframe_index = self
                    .computed_steps
                    .iter()
                    .position(|step| total_progress as f32 <= step.start_percentage);
                prev_keyframe_index = next_keyframe_index
                    .and_then(|pos| if pos != 0 { Some(pos - 1) } else { None })
                    .unwrap_or(0);
            },
            AnimationDirection::Reverse => {
                next_keyframe_index = self
                    .computed_steps
                    .iter()
                    .rev()
                    .position(|step| total_progress as f32 <= 1. - step.start_percentage)
                    .map(|pos| num_steps - pos - 1);
                prev_keyframe_index = next_keyframe_index
                    .and_then(|pos| {
                        if pos != num_steps - 1 {
                            Some(pos + 1)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(num_steps - 1)
            },
            _ => unreachable!(),
        }

        debug!(
            "Animation::get_property_declaration_at_time: keyframe from {:?} to {:?}",
            prev_keyframe_index, next_keyframe_index
        );

        let prev_keyframe = &self.computed_steps[prev_keyframe_index];
        let next_keyframe = match next_keyframe_index {
            Some(index) => &self.computed_steps[index],
            None => return,
        };

        // If we only need to take into account one keyframe, then exit early
        // in order to avoid doing more work.
        let mut add_declarations_to_map = |keyframe: &ComputedKeyframe| {
            for value in keyframe.values.iter() {
                map.insert(value.id(), value.clone());
            }
        };
        if total_progress <= 0.0 {
            add_declarations_to_map(&prev_keyframe);
            return;
        }
        if total_progress >= 1.0 {
            add_declarations_to_map(&next_keyframe);
            return;
        }

        let percentage_between_keyframes =
            (next_keyframe.start_percentage - prev_keyframe.start_percentage).abs() as f64;
        let duration_between_keyframes = percentage_between_keyframes * self.duration;
        let direction_aware_prev_keyframe_start_percentage = match self.current_direction {
            AnimationDirection::Normal => prev_keyframe.start_percentage as f64,
            AnimationDirection::Reverse => 1. - prev_keyframe.start_percentage as f64,
            _ => unreachable!(),
        };
        let progress_between_keyframes = (total_progress -
            direction_aware_prev_keyframe_start_percentage) /
            percentage_between_keyframes;

        for (from, to) in prev_keyframe.values.iter().zip(next_keyframe.values.iter()) {
            let animation = PropertyAnimation {
                from: from.clone(),
                to: to.clone(),
                timing_function: prev_keyframe.timing_function.clone(),
                duration: duration_between_keyframes as f64,
            };

            if let Ok(value) = animation.calculate_value(progress_between_keyframes) {
                map.insert(value.id(), value);
            }
        }
    }
}

impl fmt::Debug for Animation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Animation")
            .field("name", &self.name)
            .field("started_at", &self.started_at)
            .field("duration", &self.duration)
            .field("delay", &self.delay)
            .field("iteration_state", &self.iteration_state)
            .field("state", &self.state)
            .field("direction", &self.direction)
            .field("current_direction", &self.current_direction)
            .field("cascade_style", &())
            .finish()
    }
}

/// A CSS Transition
#[derive(Clone, Debug, MallocSizeOf)]
pub struct Transition {
    /// The start time of this transition, which is the current value of the animation
    /// timeline when this transition was created plus any animation delay.
    pub start_time: f64,

    /// The delay used for this transition.
    pub delay: f64,

    /// The internal style `PropertyAnimation` for this transition.
    pub property_animation: PropertyAnimation,

    /// The state of this transition.
    pub state: AnimationState,

    /// Whether or not this transition is new and or has already been tracked
    /// by the script thread.
    pub is_new: bool,

    /// If this `Transition` has been replaced by a new one this field is
    /// used to help produce better reversed transitions.
    pub reversing_adjusted_start_value: AnimationValue,

    /// If this `Transition` has been replaced by a new one this field is
    /// used to help produce better reversed transitions.
    pub reversing_shortening_factor: f64,
}

impl Transition {
    fn update_for_possibly_reversed_transition(
        &mut self,
        replaced_transition: &Transition,
        delay: f64,
        now: f64,
    ) {
        // If we reach here, we need to calculate a reversed transition according to
        // https://drafts.csswg.org/css-transitions/#starting
        //
        //  "...if the reversing-adjusted start value of the running transition
        //  is the same as the value of the property in the after-change style (see
        //  the section on reversing of transitions for why these case exists),
        //  implementations must cancel the running transition and start
        //  a new transition..."
        if replaced_transition.reversing_adjusted_start_value != self.property_animation.to {
            return;
        }

        // "* reversing-adjusted start value is the end value of the running transition"
        let replaced_animation = &replaced_transition.property_animation;
        self.reversing_adjusted_start_value = replaced_animation.to.clone();

        // "* reversing shortening factor is the absolute value, clamped to the
        //    range [0, 1], of the sum of:
        //    1. the output of the timing function of the old transition at the
        //      time of the style change event, times the reversing shortening
        //      factor of the old transition
        //    2.  1 minus the reversing shortening factor of the old transition."
        let transition_progress = ((now - replaced_transition.start_time) /
            (replaced_transition.property_animation.duration))
            .min(1.0)
            .max(0.0);
        let timing_function_output = replaced_animation.timing_function_output(transition_progress);
        let old_reversing_shortening_factor = replaced_transition.reversing_shortening_factor;
        self.reversing_shortening_factor = ((timing_function_output *
            old_reversing_shortening_factor) +
            (1.0 - old_reversing_shortening_factor))
            .abs()
            .min(1.0)
            .max(0.0);

        // "* start time is the time of the style change event plus:
        //    1. if the matching transition delay is nonnegative, the matching
        //       transition delay, or.
        //    2. if the matching transition delay is negative, the product of the new
        //       transition’s reversing shortening factor and the matching transition delay,"
        self.start_time = if delay >= 0. {
            now + delay
        } else {
            now + (self.reversing_shortening_factor * delay)
        };

        // "* end time is the start time plus the product of the matching transition
        //    duration and the new transition’s reversing shortening factor,"
        self.property_animation.duration *= self.reversing_shortening_factor;

        // "* start value is the current value of the property in the running transition,
        //  * end value is the value of the property in the after-change style,"
        let procedure = Procedure::Interpolate {
            progress: timing_function_output,
        };
        match replaced_animation
            .from
            .animate(&replaced_animation.to, procedure)
        {
            Ok(new_start) => self.property_animation.from = new_start,
            Err(..) => {},
        }
    }

    /// Whether or not this animation has ended at the provided time. This does
    /// not take into account canceling i.e. when an animation or transition is
    /// canceled due to changes in the style.
    pub fn has_ended(&self, time: f64) -> bool {
        time >= self.start_time + (self.property_animation.duration)
    }

    /// Update the given animation at a given point of progress.
    pub fn calculate_value(&self, time: f64) -> Option<AnimationValue> {
        let progress = (time - self.start_time) / (self.property_animation.duration);
        if progress < 0.0 {
            return None;
        }

        self.property_animation
            .calculate_value(progress.min(1.0))
            .ok()
    }
}

/// Holds the animation state for a particular element.
#[derive(Debug, Default, MallocSizeOf)]
pub struct ElementAnimationSet {
    /// The animations for this element.
    pub animations: Vec<Animation>,

    /// The transitions for this element.
    pub transitions: Vec<Transition>,

    /// Whether or not this ElementAnimationSet has had animations or transitions
    /// which have been added, removed, or had their state changed.
    pub dirty: bool,
}

impl ElementAnimationSet {
    /// Cancel all animations in this `ElementAnimationSet`. This is typically called
    /// when the element has been removed from the DOM.
    pub fn cancel_all_animations(&mut self) {
        self.dirty = !self.animations.is_empty();
        for animation in self.animations.iter_mut() {
            animation.state = AnimationState::Canceled;
        }
        self.cancel_active_transitions();
    }

    fn cancel_active_transitions(&mut self) {
        for transition in self.transitions.iter_mut() {
            if transition.state != AnimationState::Finished {
                self.dirty = true;
                transition.state = AnimationState::Canceled;
            }
        }
    }

    /// Apply all active animations.
    pub fn apply_active_animations(
        &self,
        context: &SharedStyleContext,
        style: &mut Arc<ComputedValues>,
    ) {
        let now = context.current_time_for_animations;
        let mutable_style = Arc::make_mut(style);
        if let Some(map) = self.get_value_map_for_active_animations(now) {
            for value in map.values() {
                value.set_in_style_for_servo(mutable_style);
            }
        }

        if let Some(map) = self.get_value_map_for_active_transitions(now) {
            for value in map.values() {
                value.set_in_style_for_servo(mutable_style);
            }
        }
    }

    /// Clear all canceled animations and transitions from this `ElementAnimationSet`.
    pub fn clear_canceled_animations(&mut self) {
        self.animations
            .retain(|animation| animation.state != AnimationState::Canceled);
        self.transitions
            .retain(|animation| animation.state != AnimationState::Canceled);
    }

    /// Whether this `ElementAnimationSet` is empty, which means it doesn't
    /// hold any animations in any state.
    pub fn is_empty(&self) -> bool {
        self.animations.is_empty() && self.transitions.is_empty()
    }

    /// Whether or not this state needs animation ticks for its transitions
    /// or animations.
    pub fn needs_animation_ticks(&self) -> bool {
        self.animations
            .iter()
            .any(|animation| animation.state.needs_to_be_ticked()) ||
            self.transitions
                .iter()
                .any(|transition| transition.state.needs_to_be_ticked())
    }

    /// The number of running animations and transitions for this `ElementAnimationSet`.
    pub fn running_animation_and_transition_count(&self) -> usize {
        self.animations
            .iter()
            .filter(|animation| animation.state.needs_to_be_ticked())
            .count() +
            self.transitions
                .iter()
                .filter(|transition| transition.state.needs_to_be_ticked())
                .count()
    }

    /// If this `ElementAnimationSet` has any any active animations.
    pub fn has_active_animation(&self) -> bool {
        self.animations
            .iter()
            .any(|animation| animation.state != AnimationState::Canceled)
    }

    /// If this `ElementAnimationSet` has any any active transitions.
    pub fn has_active_transition(&self) -> bool {
        self.transitions
            .iter()
            .any(|transition| transition.state != AnimationState::Canceled)
    }

    /// Update our animations given a new style, canceling or starting new animations
    /// when appropriate.
    pub fn update_animations_for_new_style<E>(
        &mut self,
        element: E,
        context: &SharedStyleContext,
        new_style: &Arc<ComputedValues>,
        resolver: &mut StyleResolverForElement<E>,
    ) where
        E: TElement,
    {
        for animation in self.animations.iter_mut() {
            if animation.is_cancelled_in_new_style(new_style) {
                animation.state = AnimationState::Canceled;
            }
        }

        maybe_start_animations(element, &context, &new_style, self, resolver);
    }

    /// Update our transitions given a new style, canceling or starting new animations
    /// when appropriate.
    pub fn update_transitions_for_new_style(
        &mut self,
        might_need_transitions_update: bool,
        context: &SharedStyleContext,
        old_style: Option<&Arc<ComputedValues>>,
        after_change_style: &Arc<ComputedValues>,
    ) {
        // If this is the first style, we don't trigger any transitions and we assume
        // there were no previously triggered transitions.
        let mut before_change_style = match old_style {
            Some(old_style) => Arc::clone(old_style),
            None => return,
        };

        // If the style of this element is display:none, then cancel all active transitions.
        if after_change_style.get_box().clone_display().is_none() {
            self.cancel_active_transitions();
            return;
        }

        if !might_need_transitions_update {
            return;
        }

        // We convert old values into `before-change-style` here.
        if self.has_active_transition() || self.has_active_animation() {
            self.apply_active_animations(context, &mut before_change_style);
        }

        let transitioning_properties = start_transitions_if_applicable(
            context,
            &before_change_style,
            after_change_style,
            self,
        );

        // Cancel any non-finished transitions that have properties which no longer transition.
        for transition in self.transitions.iter_mut() {
            if transition.state == AnimationState::Finished {
                continue;
            }
            if transitioning_properties.contains(transition.property_animation.property_id()) {
                continue;
            }
            transition.state = AnimationState::Canceled;
            self.dirty = true;
        }
    }

    fn start_transition_if_applicable(
        &mut self,
        context: &SharedStyleContext,
        longhand_id: LonghandId,
        index: usize,
        old_style: &ComputedValues,
        new_style: &Arc<ComputedValues>,
    ) {
        let style = new_style.get_ui();
        let timing_function = style.transition_timing_function_mod(index);
        let duration = style.transition_duration_mod(index);
        let delay = style.transition_delay_mod(index).seconds() as f64;
        let now = context.current_time_for_animations;

        // Only start a new transition if the style actually changes between
        // the old style and the new style.
        let property_animation = match PropertyAnimation::from_longhand(
            longhand_id,
            timing_function,
            duration,
            old_style,
            new_style,
        ) {
            Some(property_animation) => property_animation,
            None => return,
        };

        // Per [1], don't trigger a new transition if the end state for that
        // transition is the same as that of a transition that's running or
        // completed. We don't take into account any canceled animations.
        // [1]: https://drafts.csswg.org/css-transitions/#starting
        if self
            .transitions
            .iter()
            .filter(|transition| transition.state != AnimationState::Canceled)
            .any(|transition| transition.property_animation.to == property_animation.to)
        {
            return;
        }

        // We are going to start a new transition, but we might have to update
        // it if we are replacing a reversed transition.
        let reversing_adjusted_start_value = property_animation.from.clone();
        let mut new_transition = Transition {
            start_time: now + delay,
            delay,
            property_animation,
            state: AnimationState::Pending,
            is_new: true,
            reversing_adjusted_start_value,
            reversing_shortening_factor: 1.0,
        };

        if let Some(old_transition) = self
            .transitions
            .iter_mut()
            .filter(|transition| transition.state == AnimationState::Running)
            .find(|transition| transition.property_animation.property_id() == longhand_id)
        {
            // We always cancel any running transitions for the same property.
            old_transition.state = AnimationState::Canceled;
            new_transition.update_for_possibly_reversed_transition(old_transition, delay, now);
        }

        self.transitions.push(new_transition);
        self.dirty = true;
    }

    /// Generate a `AnimationValueMap` for this `ElementAnimationSet`'s
    /// active transitions at the given time value.
    pub fn get_value_map_for_active_transitions(&self, now: f64) -> Option<AnimationValueMap> {
        if !self.has_active_transition() {
            return None;
        }

        let mut map =
            AnimationValueMap::with_capacity_and_hasher(self.transitions.len(), Default::default());
        for transition in &self.transitions {
            if transition.state == AnimationState::Canceled {
                continue;
            }
            let value = match transition.calculate_value(now) {
                Some(value) => value,
                None => continue,
            };
            map.insert(value.id(), value);
        }

        Some(map)
    }

    /// Generate a `AnimationValueMap` for this `ElementAnimationSet`'s
    /// active animations at the given time value.
    pub fn get_value_map_for_active_animations(&self, now: f64) -> Option<AnimationValueMap> {
        if !self.has_active_animation() {
            return None;
        }

        let mut map = Default::default();
        for animation in &self.animations {
            animation.get_property_declaration_at_time(now, &mut map);
        }

        Some(map)
    }
}

#[derive(Clone, Debug, Eq, Hash, MallocSizeOf, PartialEq)]
/// A key that is used to identify nodes in the `DocumentAnimationSet`.
pub struct AnimationSetKey {
    /// The node for this `AnimationSetKey`.
    pub node: OpaqueNode,
    /// The pseudo element for this `AnimationSetKey`. If `None` this key will
    /// refer to the main content for its node.
    pub pseudo_element: Option<PseudoElement>,
}

impl AnimationSetKey {
    /// Create a new key given a node and optional pseudo element.
    pub fn new(node: OpaqueNode, pseudo_element: Option<PseudoElement>) -> Self {
        AnimationSetKey {
            node,
            pseudo_element,
        }
    }

    /// Create a new key for the main content of this node.
    pub fn new_for_non_pseudo(node: OpaqueNode) -> Self {
        AnimationSetKey {
            node,
            pseudo_element: None,
        }
    }

    /// Create a new key for given node and pseudo element.
    pub fn new_for_pseudo(node: OpaqueNode, pseudo_element: PseudoElement) -> Self {
        AnimationSetKey {
            node,
            pseudo_element: Some(pseudo_element),
        }
    }
}

#[derive(Clone, Debug, Default, MallocSizeOf)]
/// A set of animations for a document.
pub struct DocumentAnimationSet {
    /// The `ElementAnimationSet`s that this set contains.
    #[ignore_malloc_size_of = "Arc is hard"]
    pub sets: Arc<RwLock<FxHashMap<AnimationSetKey, ElementAnimationSet>>>,
}

impl DocumentAnimationSet {
    /// Return whether or not the provided node has active CSS animations.
    pub fn has_active_animations(&self, key: &AnimationSetKey) -> bool {
        self.sets
            .read()
            .get(key)
            .map_or(false, |set| set.has_active_animation())
    }

    /// Return whether or not the provided node has active CSS transitions.
    pub fn has_active_transitions(&self, key: &AnimationSetKey) -> bool {
        self.sets
            .read()
            .get(key)
            .map_or(false, |set| set.has_active_transition())
    }

    /// Return a locked PropertyDeclarationBlock with animation values for the given
    /// key and time.
    pub fn get_animation_declarations(
        &self,
        key: &AnimationSetKey,
        time: f64,
        shared_lock: &SharedRwLock,
    ) -> Option<Arc<Locked<PropertyDeclarationBlock>>> {
        self.sets
            .read()
            .get(key)
            .and_then(|set| set.get_value_map_for_active_animations(time))
            .map(|map| {
                let block = PropertyDeclarationBlock::from_animation_value_map(&map);
                Arc::new(shared_lock.wrap(block))
            })
    }

    /// Return a locked PropertyDeclarationBlock with transition values for the given
    /// key and time.
    pub fn get_transition_declarations(
        &self,
        key: &AnimationSetKey,
        time: f64,
        shared_lock: &SharedRwLock,
    ) -> Option<Arc<Locked<PropertyDeclarationBlock>>> {
        self.sets
            .read()
            .get(key)
            .and_then(|set| set.get_value_map_for_active_transitions(time))
            .map(|map| {
                let block = PropertyDeclarationBlock::from_animation_value_map(&map);
                Arc::new(shared_lock.wrap(block))
            })
    }

    /// Get all the animation declarations for the given key, returning an empty
    /// `AnimationDeclarations` if there are no animations.
    pub fn get_all_declarations(
        &self,
        key: &AnimationSetKey,
        time: f64,
        shared_lock: &SharedRwLock,
    ) -> AnimationDeclarations {
        let sets = self.sets.read();
        let set = match sets.get(key) {
            Some(set) => set,
            None => return Default::default(),
        };

        let animations = set.get_value_map_for_active_animations(time).map(|map| {
            let block = PropertyDeclarationBlock::from_animation_value_map(&map);
            Arc::new(shared_lock.wrap(block))
        });
        let transitions = set.get_value_map_for_active_transitions(time).map(|map| {
            let block = PropertyDeclarationBlock::from_animation_value_map(&map);
            Arc::new(shared_lock.wrap(block))
        });
        AnimationDeclarations {
            animations,
            transitions,
        }
    }

    /// Cancel all animations for set at the given key.
    pub fn cancel_all_animations_for_key(&self, key: &AnimationSetKey) {
        if let Some(set) = self.sets.write().get_mut(key) {
            set.cancel_all_animations();
        }
    }
}

/// Kick off any new transitions for this node and return all of the properties that are
/// transitioning. This is at the end of calculating style for a single node.
pub fn start_transitions_if_applicable(
    context: &SharedStyleContext,
    old_style: &ComputedValues,
    new_style: &Arc<ComputedValues>,
    animation_state: &mut ElementAnimationSet,
) -> LonghandIdSet {
    let mut properties_that_transition = LonghandIdSet::new();
    for transition in new_style.transition_properties() {
        let physical_property = transition.longhand_id.to_physical(new_style.writing_mode);
        if properties_that_transition.contains(physical_property) {
            continue;
        }

        properties_that_transition.insert(physical_property);
        animation_state.start_transition_if_applicable(
            context,
            physical_property,
            transition.index,
            old_style,
            new_style,
        );
    }

    properties_that_transition
}

/// Triggers animations for a given node looking at the animation property
/// values.
pub fn maybe_start_animations<E>(
    element: E,
    context: &SharedStyleContext,
    new_style: &Arc<ComputedValues>,
    animation_state: &mut ElementAnimationSet,
    resolver: &mut StyleResolverForElement<E>,
) where
    E: TElement,
{
    let style = new_style.get_ui();
    for (i, name) in style.animation_name_iter().enumerate() {
        let name = match name.as_atom() {
            Some(atom) => atom,
            None => continue,
        };

        debug!("maybe_start_animations: name={}", name);
        let duration = style.animation_duration_mod(i).seconds() as f64;
        if duration == 0. {
            continue;
        }

        let keyframe_animation = match context.stylist.get_animation(name, element) {
            Some(animation) => animation,
            None => continue,
        };

        debug!("maybe_start_animations: animation {} found", name);

        // If this animation doesn't have any keyframe, we can just continue
        // without submitting it to the compositor, since both the first and
        // the second keyframes would be synthetised from the computed
        // values.
        if keyframe_animation.steps.is_empty() {
            continue;
        }

        // NB: This delay may be negative, meaning that the animation may be created
        // in a state where we have advanced one or more iterations or even that the
        // animation begins in a finished state.
        let delay = style.animation_delay_mod(i).seconds();

        let iteration_state = match style.animation_iteration_count_mod(i) {
            AnimationIterationCount::Infinite => KeyframesIterationState::Infinite(0.0),
            AnimationIterationCount::Number(n) => KeyframesIterationState::Finite(0.0, n.into()),
        };

        let animation_direction = style.animation_direction_mod(i);

        let initial_direction = match animation_direction {
            AnimationDirection::Normal | AnimationDirection::Alternate => {
                AnimationDirection::Normal
            },
            AnimationDirection::Reverse | AnimationDirection::AlternateReverse => {
                AnimationDirection::Reverse
            },
        };

        let now = context.current_time_for_animations;
        let started_at = now + delay as f64;
        let mut starting_progress = (now - started_at) / duration;
        let state = match style.animation_play_state_mod(i) {
            AnimationPlayState::Paused => AnimationState::Paused(starting_progress),
            AnimationPlayState::Running => AnimationState::Pending,
        };

        let computed_steps = ComputedKeyframe::generate_for_keyframes(
            element,
            &keyframe_animation,
            context,
            new_style,
            style.animation_timing_function_mod(i),
            resolver,
        );

        let mut new_animation = Animation {
            name: name.clone(),
            properties_changed: keyframe_animation.properties_changed,
            computed_steps,
            started_at,
            duration,
            fill_mode: style.animation_fill_mode_mod(i),
            delay: delay as f64,
            iteration_state,
            state,
            direction: animation_direction,
            current_direction: initial_direction,
            cascade_style: new_style.clone(),
            is_new: true,
        };

        // If we started with a negative delay, make sure we iterate the animation if
        // the delay moves us past the first iteration.
        while starting_progress > 1. && !new_animation.on_last_iteration() {
            new_animation.iterate();
            starting_progress -= 1.;
        }

        animation_state.dirty = true;

        // If the animation was already present in the list for the node, just update its state.
        for existing_animation in animation_state.animations.iter_mut() {
            if existing_animation.state == AnimationState::Canceled {
                continue;
            }

            if new_animation.name == existing_animation.name {
                existing_animation
                    .update_from_other(&new_animation, context.current_time_for_animations);
                return;
            }
        }

        animation_state.animations.push(new_animation);
    }
}
