/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! CSS transitions and animations.

// NOTE(emilio): This code isn't really executed in Gecko, but we don't want to
// compile it out so that people remember it exists.

use crate::bezier::Bezier;
use crate::context::SharedStyleContext;
use crate::dom::{OpaqueNode, TElement, TNode};
use crate::font_metrics::FontMetricsProvider;
use crate::properties::animated_properties::AnimationValue;
use crate::properties::longhands::animation_direction::computed_value::single_value::T as AnimationDirection;
use crate::properties::longhands::animation_fill_mode::computed_value::single_value::T as AnimationFillMode;
use crate::properties::longhands::animation_play_state::computed_value::single_value::T as AnimationPlayState;
use crate::properties::LonghandIdSet;
use crate::properties::{self, CascadeMode, ComputedValues, LonghandId};
use crate::stylesheets::keyframes_rule::{KeyframesAnimation, KeyframesStep, KeyframesStepValue};
use crate::stylesheets::Origin;
use crate::values::animated::{Animate, Procedure};
use crate::values::computed::Time;
use crate::values::computed::TimingFunction;
use crate::values::generics::box_::AnimationIterationCount;
use crate::values::generics::easing::{StepPosition, TimingFunction as GenericTimingFunction};
use crate::Atom;
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
        match self.timing_function {
            GenericTimingFunction::CubicBezier { x1, y1, x2, y2 } => {
                Bezier::new(x1, y1, x2, y2).solve(progress, epsilon)
            },
            GenericTimingFunction::Steps(steps, pos) => {
                let mut current_step = (progress * (steps as f64)).floor() as i32;

                if pos == StepPosition::Start ||
                    pos == StepPosition::JumpStart ||
                    pos == StepPosition::JumpBoth
                {
                    current_step = current_step + 1;
                }

                // FIXME: We should update current_step according to the "before flag".
                // In order to get the before flag, we have to know the current animation phase
                // and whether the iteration is reversed. For now, we skip this calculation.
                // (i.e. Treat before_flag is unset,)
                // https://drafts.csswg.org/css-easing/#step-timing-function-algo

                if progress >= 0.0 && current_step < 0 {
                    current_step = 0;
                }

                let jumps = match pos {
                    StepPosition::JumpBoth => steps + 1,
                    StepPosition::JumpNone => steps - 1,
                    StepPosition::JumpStart |
                    StepPosition::JumpEnd |
                    StepPosition::Start |
                    StepPosition::End => steps,
                };

                if progress <= 1.0 && current_step > jumps {
                    current_step = jumps;
                }

                (current_step as f64) / (jumps as f64)
            },
            GenericTimingFunction::Keyword(keyword) => {
                let (x1, x2, y1, y2) = keyword.to_bezier();
                Bezier::new(x1, x2, y1, y2).solve(progress, epsilon)
            },
        }
    }

    /// Update the given animation at a given point of progress.
    fn update(&self, style: &mut ComputedValues, progress: f64) {
        let procedure = Procedure::Interpolate {
            progress: self.timing_function_output(progress),
        };
        if let Ok(new_value) = self.from.animate(&self.to, procedure) {
            new_value.set_in_style_for_servo(style);
        }
    }
}

/// This structure represents the state of an animation.
#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub enum AnimationState {
    /// This animation is paused. The inner field is the percentage of progress
    /// when it was paused, from 0 to 1.
    Paused(f64),
    /// This animation is currently running.
    Running,
    /// This animation has finished.
    Finished,
    /// This animation has been canceled.
    Canceled,
}

/// This structure represents a keyframes animation current iteration state.
///
/// If the iteration count is infinite, there's no other state, otherwise we
/// have to keep track the current iteration and the max iteration count.
#[derive(Clone, Debug, MallocSizeOf)]
pub enum KeyframesIterationState {
    /// Infinite iterations, so no need to track a state.
    Infinite,
    /// Current and max iterations.
    /// TODO: Everything else in this file uses f64, so perhaps this should
    /// be as well.
    Finite(f32, f32),
}

/// A CSS Animation
#[derive(Clone, MallocSizeOf)]
pub struct Animation {
    /// The node associated with this animation.
    pub node: OpaqueNode,

    /// The name of this animation as defined by the style.
    pub name: Atom,

    /// The internal animation from the style system.
    pub keyframes_animation: KeyframesAnimation,

    /// The time this animation started at.
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
        let index = new_style
            .get_box()
            .animation_name_iter()
            .position(|animation_name| Some(&self.name) == animation_name.as_atom());
        let index = match index {
            Some(index) => index,
            None => return true,
        };

        new_style.get_box().animation_duration_mod(index).seconds() == 0.
    }

    /// Given the current time, advances this animation to the next iteration,
    /// updates times, and then toggles the direction if appropriate. Otherwise
    /// does nothing.
    pub fn iterate_if_necessary(&mut self, time: f64) {
        if !self.iteration_over(time) {
            return;
        }

        // Only iterate animations that are currently running.
        if self.state != AnimationState::Running {
            return;
        }

        if let KeyframesIterationState::Finite(ref mut current, max) = self.iteration_state {
            // If we are already on the final iteration, just exit now.
            // NB: This prevent us from updating the direction, which might be
            // needed for the correct handling of animation-fill-mode.
            if (max - *current) <= 1.0 {
                return;
            }

            *current += 1.0;
        }

        // Update the next iteration direction if applicable.
        // TODO(mrobinson): The duration might now be wrong for floating point iteration counts.
        self.started_at += self.duration + self.delay;
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

    fn iteration_over(&self, time: f64) -> bool {
        time > (self.started_at + self.duration)
    }

    /// Whether or not this animation has finished at the provided time. This does
    /// not take into account canceling i.e. when an animation or transition is
    /// canceled due to changes in the style.
    pub fn has_ended(&self, time: f64) -> bool {
        match self.state {
            AnimationState::Canceled | AnimationState::Paused(_) => return false,
            AnimationState::Finished => return true,
            AnimationState::Running => {},
        }

        if !self.iteration_over(time) {
            return false;
        }

        // If we have a limited number of iterations and we cannot advance to another
        // iteration, then we have ended.
        return match self.iteration_state {
            KeyframesIterationState::Finite(current, max) if (max - current) <= 1.0 => true,
            KeyframesIterationState::Finite(..) | KeyframesIterationState::Infinite => false,
        };
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

        let mut new_started_at = old_started_at;

        // If we're unpausing the animation, fake the start time so we seem to
        // restore it.
        //
        // If the animation keeps paused, keep the old value.
        //
        // If we're pausing the animation, compute the progress value.
        match (&mut self.state, old_state) {
            (&mut Running, Paused(progress)) => new_started_at = now - (self.duration * progress),
            (&mut Paused(ref mut new), Paused(old)) => *new = old,
            (&mut Paused(ref mut progress), Running) => {
                *progress = (now - old_started_at) / old_duration
            },
            _ => {},
        }

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

        self.current_direction = old_direction;
        self.started_at = new_started_at;
    }

    /// Calculate the active-duration of this animation according to
    /// https://drafts.csswg.org/css-animations/#active-duration. active-duration
    /// is not really meaningful for infinite animations so we just return 0
    /// here in that case.
    pub fn active_duration(&self) -> f64 {
        match self.iteration_state {
            KeyframesIterationState::Finite(_, max) => self.duration * (max as f64),
            KeyframesIterationState::Infinite => 0.,
        }
    }

    /// Update the given style to reflect the values specified by this `Animation`
    /// at the time provided by the given `SharedStyleContext`.
    fn update_style<E>(
        &self,
        context: &SharedStyleContext,
        style: &mut Arc<ComputedValues>,
        font_metrics_provider: &dyn FontMetricsProvider,
    ) where
        E: TElement,
    {
        let duration = self.duration;
        let started_at = self.started_at;

        let now = match self.state {
            AnimationState::Running | AnimationState::Finished => {
                context.current_time_for_animations
            },
            AnimationState::Paused(progress) => started_at + duration * progress,
            AnimationState::Canceled => return,
        };

        debug_assert!(!self.keyframes_animation.steps.is_empty());

        let mut total_progress = (now - started_at) / duration;
        if total_progress < 0. &&
            self.fill_mode != AnimationFillMode::Backwards &&
            self.fill_mode != AnimationFillMode::Both
        {
            return;
        }

        if total_progress > 1. &&
            self.fill_mode != AnimationFillMode::Forwards &&
            self.fill_mode != AnimationFillMode::Both
        {
            return;
        }
        total_progress = total_progress.min(1.0).max(0.0);

        // Get the indices of the previous (from) keyframe and the next (to) keyframe.
        let next_keyframe_index;
        let prev_keyframe_index;
        match self.current_direction {
            AnimationDirection::Normal => {
                next_keyframe_index = self
                    .keyframes_animation
                    .steps
                    .iter()
                    .position(|step| total_progress as f32 <= step.start_percentage.0);
                prev_keyframe_index = next_keyframe_index
                    .and_then(|pos| if pos != 0 { Some(pos - 1) } else { None })
                    .unwrap_or(0);
            },
            AnimationDirection::Reverse => {
                next_keyframe_index = self
                    .keyframes_animation
                    .steps
                    .iter()
                    .rev()
                    .position(|step| total_progress as f32 <= 1. - step.start_percentage.0)
                    .map(|pos| self.keyframes_animation.steps.len() - pos - 1);
                prev_keyframe_index = next_keyframe_index
                    .and_then(|pos| {
                        if pos != self.keyframes_animation.steps.len() - 1 {
                            Some(pos + 1)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(self.keyframes_animation.steps.len() - 1)
            },
            _ => unreachable!(),
        }

        debug!(
            "Animation::update_style: keyframe from {:?} to {:?}",
            prev_keyframe_index, next_keyframe_index
        );

        let prev_keyframe = &self.keyframes_animation.steps[prev_keyframe_index];
        let next_keyframe = match next_keyframe_index {
            Some(target) => &self.keyframes_animation.steps[target],
            None => return,
        };

        let update_with_single_keyframe_style = |style, computed_style: &Arc<ComputedValues>| {
            let mutable_style = Arc::make_mut(style);
            for property in self
                .keyframes_animation
                .properties_changed
                .iter()
                .filter_map(|longhand| {
                    AnimationValue::from_computed_values(longhand, &**computed_style)
                })
            {
                property.set_in_style_for_servo(mutable_style);
            }
        };

        // TODO: How could we optimise it? Is it such a big deal?
        let prev_keyframe_style = compute_style_for_animation_step::<E>(
            context,
            prev_keyframe,
            style,
            &self.cascade_style,
            font_metrics_provider,
        );
        if total_progress <= 0.0 {
            update_with_single_keyframe_style(style, &prev_keyframe_style);
            return;
        }

        let next_keyframe_style = compute_style_for_animation_step::<E>(
            context,
            next_keyframe,
            &prev_keyframe_style,
            &self.cascade_style,
            font_metrics_provider,
        );
        if total_progress >= 1.0 {
            update_with_single_keyframe_style(style, &next_keyframe_style);
            return;
        }

        let relative_timespan =
            (next_keyframe.start_percentage.0 - prev_keyframe.start_percentage.0).abs();
        let relative_duration = relative_timespan as f64 * duration;
        let last_keyframe_ended_at = match self.current_direction {
            AnimationDirection::Normal => {
                self.started_at + (duration * prev_keyframe.start_percentage.0 as f64)
            },
            AnimationDirection::Reverse => {
                self.started_at + (duration * (1. - prev_keyframe.start_percentage.0 as f64))
            },
            _ => unreachable!(),
        };
        let relative_progress = (now - last_keyframe_ended_at) / relative_duration;

        // NB: The spec says that the timing function can be overwritten
        // from the keyframe style.
        let timing_function = if prev_keyframe.declared_timing_function {
            // NB: animation_timing_function can never be empty, always has
            // at least the default value (`ease`).
            prev_keyframe_style
                .get_box()
                .animation_timing_function_at(0)
        } else {
            // TODO(mrobinson): It isn't optimal to have to walk this list every
            // time. Perhaps this should be stored in the animation.
            let index = match style
                .get_box()
                .animation_name_iter()
                .position(|animation_name| Some(&self.name) == animation_name.as_atom())
            {
                Some(index) => index,
                None => return warn!("Tried to update a style with a cancelled animation."),
            };
            style.get_box().animation_timing_function_mod(index)
        };

        let mut new_style = (**style).clone();
        let mut update_style_for_longhand = |longhand| {
            let from = AnimationValue::from_computed_values(longhand, &prev_keyframe_style)?;
            let to = AnimationValue::from_computed_values(longhand, &next_keyframe_style)?;
            PropertyAnimation {
                from,
                to,
                timing_function,
                duration: relative_duration as f64,
            }
            .update(&mut new_style, relative_progress);
            None::<()>
        };

        for property in self.keyframes_animation.properties_changed.iter() {
            update_style_for_longhand(property);
        }

        *Arc::make_mut(style) = new_style;
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
    /// The node associated with this animation.
    pub node: OpaqueNode,

    /// The start time of this transition, which is the current value of the animation
    /// timeline when this transition created.
    pub start_time: f64,

    /// The internal style `PropertyAnimation` for this transition.
    pub property_animation: PropertyAnimation,

    /// The state of this transition.
    pub state: AnimationState,

    /// Whether or not this transition is new and or has already been tracked
    /// by the script thread.
    pub is_new: bool,
}

impl Transition {
    /// Whether or not this animation has ended at the provided time. This does
    /// not take into account canceling i.e. when an animation or transition is
    /// canceled due to changes in the style.
    pub fn has_ended(&self, time: f64) -> bool {
        time >= self.start_time + (self.property_animation.duration)
    }

    /// Whether this animation has the same end value as another one.
    #[inline]
    fn progress(&self, now: f64) -> f64 {
        let progress = (now - self.start_time) / (self.property_animation.duration);
        progress.min(1.0)
    }

    /// Update a style to the value specified by this `Transition` given a `SharedStyleContext`.
    fn update_style(&self, context: &SharedStyleContext, style: &mut Arc<ComputedValues>) {
        // Never apply canceled transitions to a style.
        if self.state == AnimationState::Canceled {
            return;
        }

        let progress = self.progress(context.current_time_for_animations);
        if progress >= 0.0 {
            self.property_animation
                .update(Arc::make_mut(style), progress);
        }
    }
}

/// Holds the animation state for a particular element.
#[derive(Debug, Default, MallocSizeOf)]
pub struct ElementAnimationSet {
    /// The animations for this element.
    pub animations: Vec<Animation>,

    /// The transitions for this element.
    pub transitions: Vec<Transition>,
}

impl ElementAnimationSet {
    /// Cancel all animations in this `ElementAnimationSet`. This is typically called
    /// when the element has been removed from the DOM.
    pub fn cancel_all_animations(&mut self) {
        for animation in self.animations.iter_mut() {
            animation.state = AnimationState::Canceled;
        }
        for transition in self.transitions.iter_mut() {
            transition.state = AnimationState::Canceled;
        }
    }

    pub(crate) fn apply_active_animations<E>(
        &mut self,
        context: &SharedStyleContext,
        style: &mut Arc<ComputedValues>,
        font_metrics: &dyn crate::font_metrics::FontMetricsProvider,
    ) where
        E: TElement,
    {
        for animation in &self.animations {
            animation.update_style::<E>(context, style, font_metrics);
        }

        for transition in &self.transitions {
            transition.update_style(context, style);
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
    /// or animations. New animations don't need ticks until they are no
    /// longer marked as new.
    pub fn needs_animation_ticks(&self) -> bool {
        self.animations
            .iter()
            .any(|animation| animation.state == AnimationState::Running && !animation.is_new) ||
            self.transitions.iter().any(|transition| {
                transition.state == AnimationState::Running && !transition.is_new
            })
    }

    /// The number of running animations and transitions for this `ElementAnimationSet`.
    pub fn running_animation_and_transition_count(&self) -> usize {
        self.animations
            .iter()
            .filter(|animation| animation.state == AnimationState::Running)
            .count() +
            self.transitions
                .iter()
                .filter(|transition| transition.state == AnimationState::Running)
                .count()
    }

    fn has_active_transition_or_animation(&self) -> bool {
        self.animations
            .iter()
            .any(|animation| animation.state != AnimationState::Canceled) ||
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
    ) where
        E: TElement,
    {
        for animation in self.animations.iter_mut() {
            if animation.is_cancelled_in_new_style(new_style) {
                animation.state = AnimationState::Canceled;
            }
        }

        maybe_start_animations(element, &context, &new_style, self);

        // When necessary, iterate our running animations to the next iteration.
        for animation in self.animations.iter_mut() {
            animation.iterate_if_necessary(context.current_time_for_animations);
        }
    }

    /// Update our transitions given a new style, canceling or starting new animations
    /// when appropriate.
    pub fn update_transitions_for_new_style<E>(
        &mut self,
        context: &SharedStyleContext,
        opaque_node: OpaqueNode,
        old_style: Option<&Arc<ComputedValues>>,
        after_change_style: &Arc<ComputedValues>,
        font_metrics: &dyn crate::font_metrics::FontMetricsProvider,
    ) where
        E: TElement,
    {
        // If this is the first style, we don't trigger any transitions and we assume
        // there were no previously triggered transitions.
        let mut before_change_style = match old_style {
            Some(old_style) => Arc::clone(old_style),
            None => return,
        };

        // We convert old values into `before-change-style` here.
        // See https://drafts.csswg.org/css-transitions/#starting. We need to clone the
        // style because this might still be a reference to the original `old_style` and
        // we want to preserve that so that we can later properly calculate restyle damage.
        if self.has_active_transition_or_animation() {
            before_change_style = before_change_style.clone();
            self.apply_active_animations::<E>(context, &mut before_change_style, font_metrics);
        }

        let transitioning_properties = start_transitions_if_applicable(
            context,
            opaque_node,
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
        }
    }
}

/// Kick off any new transitions for this node and return all of the properties that are
/// transitioning. This is at the end of calculating style for a single node.
pub fn start_transitions_if_applicable(
    context: &SharedStyleContext,
    opaque_node: OpaqueNode,
    old_style: &ComputedValues,
    new_style: &Arc<ComputedValues>,
    animation_state: &mut ElementAnimationSet,
) -> LonghandIdSet {
    // If the style of this element is display:none, then we don't start any transitions
    // and we cancel any currently running transitions by returning an empty LonghandIdSet.
    let box_style = new_style.get_box();
    if box_style.clone_display().is_none() {
        return LonghandIdSet::new();
    }

    let mut properties_that_transition = LonghandIdSet::new();
    for transition in new_style.transition_properties() {
        let physical_property = transition.longhand_id.to_physical(new_style.writing_mode);
        if properties_that_transition.contains(physical_property) {
            continue;
        } else {
            properties_that_transition.insert(physical_property);
        }

        let property_animation = match PropertyAnimation::from_longhand(
            transition.longhand_id,
            box_style.transition_timing_function_mod(transition.index),
            box_style.transition_duration_mod(transition.index),
            old_style,
            new_style,
        ) {
            Some(property_animation) => property_animation,
            None => continue,
        };

        // Per [1], don't trigger a new transition if the end state for that
        // transition is the same as that of a transition that's running or
        // completed. We don't take into account any canceled animations.
        // [1]: https://drafts.csswg.org/css-transitions/#starting
        if animation_state
            .transitions
            .iter()
            .filter(|transition| transition.state != AnimationState::Canceled)
            .any(|transition| transition.property_animation.to == property_animation.to)
        {
            continue;
        }

        // Kick off the animation.
        debug!("Kicking off transition of {:?}", property_animation);
        let box_style = new_style.get_box();
        let start_time = context.current_time_for_animations +
            (box_style.transition_delay_mod(transition.index).seconds() as f64);
        animation_state.transitions.push(Transition {
            node: opaque_node,
            start_time,
            property_animation,
            state: AnimationState::Running,
            is_new: true,
        });
    }

    properties_that_transition
}

fn compute_style_for_animation_step<E>(
    context: &SharedStyleContext,
    step: &KeyframesStep,
    previous_style: &ComputedValues,
    style_from_cascade: &Arc<ComputedValues>,
    font_metrics_provider: &dyn FontMetricsProvider,
) -> Arc<ComputedValues>
where
    E: TElement,
{
    match step.value {
        KeyframesStepValue::ComputedValues => style_from_cascade.clone(),
        KeyframesStepValue::Declarations {
            block: ref declarations,
        } => {
            let guard = declarations.read_with(context.guards.author);

            let iter = || {
                // It's possible to have !important properties in keyframes
                // so we have to filter them out.
                // See the spec issue https://github.com/w3c/csswg-drafts/issues/1824
                // Also we filter our non-animatable properties.
                guard
                    .normal_declaration_iter()
                    .filter(|declaration| declaration.is_animatable())
                    .map(|decl| (decl, Origin::Author))
            };

            // This currently ignores visited styles, which seems acceptable,
            // as existing browsers don't appear to animate visited styles.
            let computed = properties::apply_declarations::<E, _, _>(
                context.stylist.device(),
                /* pseudo = */ None,
                previous_style.rules(),
                &context.guards,
                iter,
                Some(previous_style),
                Some(previous_style),
                Some(previous_style),
                font_metrics_provider,
                CascadeMode::Unvisited {
                    visited_rules: None,
                },
                context.quirks_mode(),
                /* rule_cache = */ None,
                &mut Default::default(),
                /* element = */ None,
            );
            computed
        },
    }
}

/// Triggers animations for a given node looking at the animation property
/// values.
pub fn maybe_start_animations<E>(
    element: E,
    context: &SharedStyleContext,
    new_style: &Arc<ComputedValues>,
    animation_state: &mut ElementAnimationSet,
) where
    E: TElement,
{
    let box_style = new_style.get_box();
    for (i, name) in box_style.animation_name_iter().enumerate() {
        let name = match name.as_atom() {
            Some(atom) => atom,
            None => continue,
        };

        debug!("maybe_start_animations: name={}", name);
        let duration = box_style.animation_duration_mod(i).seconds();
        if duration == 0. {
            continue;
        }

        let anim = match context.stylist.get_animation(name, element) {
            Some(animation) => animation,
            None => continue,
        };

        debug!("maybe_start_animations: animation {} found", name);

        // If this animation doesn't have any keyframe, we can just continue
        // without submitting it to the compositor, since both the first and
        // the second keyframes would be synthetised from the computed
        // values.
        if anim.steps.is_empty() {
            continue;
        }

        let delay = box_style.animation_delay_mod(i).seconds();
        let animation_start = context.current_time_for_animations + delay as f64;
        let iteration_state = match box_style.animation_iteration_count_mod(i) {
            AnimationIterationCount::Infinite => KeyframesIterationState::Infinite,
            AnimationIterationCount::Number(n) => KeyframesIterationState::Finite(0.0, n),
        };

        let animation_direction = box_style.animation_direction_mod(i);

        let initial_direction = match animation_direction {
            AnimationDirection::Normal | AnimationDirection::Alternate => {
                AnimationDirection::Normal
            },
            AnimationDirection::Reverse | AnimationDirection::AlternateReverse => {
                AnimationDirection::Reverse
            },
        };

        let state = match box_style.animation_play_state_mod(i) {
            AnimationPlayState::Paused => AnimationState::Paused(0.),
            AnimationPlayState::Running => AnimationState::Running,
        };

        let new_animation = Animation {
            node: element.as_node().opaque(),
            name: name.clone(),
            keyframes_animation: anim.clone(),
            started_at: animation_start,
            duration: duration as f64,
            fill_mode: box_style.animation_fill_mode_mod(i),
            delay: delay as f64,
            iteration_state,
            state,
            direction: animation_direction,
            current_direction: initial_direction,
            cascade_style: new_style.clone(),
            is_new: true,
        };

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
