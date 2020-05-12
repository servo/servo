/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(missing_docs)]

//! The set of animations for a document.

use crate::dom::window::Window;
use fxhash::FxHashMap;
use libc::c_void;
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use msg::constellation_msg::PipelineId;
use parking_lot::RwLock;
use script_traits::{AnimationState as AnimationsPresentState, ScriptMsg, UntrustedNodeAddress};
use servo_arc::Arc;
use style::animation::{AnimationState, ElementAnimationSet};
use style::dom::OpaqueNode;

/// The set of animations for a document.
///
/// Make sure to update the MallocSizeOf implementation when changing the
/// contents of this struct.
#[derive(Clone, Debug, Default, JSTraceable)]
pub(crate) struct Animations {
    pub sets: Arc<RwLock<FxHashMap<OpaqueNode, ElementAnimationSet>>>,
    have_running_animations: bool,
}

impl Animations {
    pub(crate) fn new() -> Self {
        Animations {
            sets: Default::default(),
            have_running_animations: false,
        }
    }

    /// Processes any new animations that were discovered after reflow. Collect messages
    /// that trigger events for any animations that changed state.
    /// TODO(mrobinson): The specification dictates that this should happen before reflow.
    pub(crate) fn do_post_reflow_update(&mut self, window: &Window, now: f64) -> AnimationsUpdate {
        let mut update = AnimationsUpdate::new(window.pipeline_id());

        {
            let mut sets = self.sets.write();
            update.collect_newly_animating_nodes(&sets);

            for set in sets.values_mut() {
                Self::handle_canceled_animations(set, now, &mut update);
                Self::finish_running_animations(set, now, &mut update);
                Self::handle_new_animations(set, &mut update);
            }

            // Remove empty states from our collection of states in order to free
            // up space as soon as we are no longer tracking any animations for
            // a node.
            sets.retain(|_, state| !state.is_empty());
        }

        self.update_running_animations_presence(window);

        update
    }

    pub(crate) fn running_animation_count(&self) -> usize {
        self.sets
            .read()
            .values()
            .map(|state| state.running_animation_and_transition_count())
            .sum()
    }

    fn update_running_animations_presence(&mut self, window: &Window) {
        let have_running_animations = self
            .sets
            .read()
            .values()
            .any(|state| state.needs_animation_ticks());
        if have_running_animations == self.have_running_animations {
            return;
        }

        self.have_running_animations = have_running_animations;
        let state = match have_running_animations {
            true => AnimationsPresentState::AnimationsPresent,
            false => AnimationsPresentState::NoAnimationsPresent,
        };

        window.send_to_constellation(ScriptMsg::ChangeRunningAnimationsState(state));
    }

    /// Walk through the list of running animations and remove all of the ones that
    /// have ended.
    fn finish_running_animations(
        set: &mut ElementAnimationSet,
        now: f64,
        update: &mut AnimationsUpdate,
    ) {
        for animation in set.animations.iter_mut() {
            if animation.state == AnimationState::Running && animation.has_ended(now) {
                animation.state = AnimationState::Finished;
                update.add_event(
                    animation.node,
                    animation.name.to_string(),
                    TransitionOrAnimationEventType::AnimationEnd,
                    animation.active_duration(),
                );
            }
        }

        for transition in set.transitions.iter_mut() {
            if transition.state == AnimationState::Running && transition.has_ended(now) {
                transition.state = AnimationState::Finished;
                update.add_event(
                    transition.node,
                    transition.property_animation.property_id().name().into(),
                    TransitionOrAnimationEventType::TransitionEnd,
                    transition.property_animation.duration,
                );
            }
        }
    }

    /// Send events for canceled animations. Currently this only handles canceled
    /// transitions, but eventually this should handle canceled CSS animations as
    /// well.
    fn handle_canceled_animations(
        set: &mut ElementAnimationSet,
        now: f64,
        update: &mut AnimationsUpdate,
    ) {
        for transition in &set.transitions {
            if transition.state == AnimationState::Canceled {
                // TODO(mrobinson): We need to properly compute the elapsed_time here
                // according to https://drafts.csswg.org/css-transitions/#event-transitionevent
                update.add_event(
                    transition.node,
                    transition.property_animation.property_id().name().into(),
                    TransitionOrAnimationEventType::TransitionCancel,
                    (now - transition.start_time).max(0.),
                );
            }
        }

        // TODO(mrobinson): We need to send animationcancel events.
        set.clear_canceled_animations();
    }

    fn handle_new_animations(set: &mut ElementAnimationSet, update: &mut AnimationsUpdate) {
        for animation in set.animations.iter_mut() {
            animation.is_new = false;
        }

        for transition in set.transitions.iter_mut() {
            if transition.is_new {
                // TODO(mrobinson): We need to properly compute the elapsed_time here
                // according to https://drafts.csswg.org/css-transitions/#event-transitionevent
                update.add_event(
                    transition.node,
                    transition.property_animation.property_id().name().into(),
                    TransitionOrAnimationEventType::TransitionRun,
                    0.,
                );
                transition.is_new = false;
            }
        }
    }
}

impl MallocSizeOf for Animations {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        self.sets.read().size_of(ops) + self.have_running_animations.size_of(ops)
    }
}

pub(crate) struct AnimationsUpdate {
    pub pipeline_id: PipelineId,
    pub events: Vec<TransitionOrAnimationEvent>,
    pub newly_animating_nodes: Vec<UntrustedNodeAddress>,
}

impl AnimationsUpdate {
    fn new(pipeline_id: PipelineId) -> Self {
        AnimationsUpdate {
            pipeline_id,
            events: Default::default(),
            newly_animating_nodes: Default::default(),
        }
    }

    fn add_event(
        &mut self,
        node: OpaqueNode,
        property_or_animation_name: String,
        event_type: TransitionOrAnimationEventType,
        elapsed_time: f64,
    ) {
        let node = UntrustedNodeAddress(node.0 as *const c_void);
        self.events.push(TransitionOrAnimationEvent {
            pipeline_id: self.pipeline_id,
            event_type,
            node,
            property_or_animation_name,
            elapsed_time,
        });
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.events.is_empty() && self.newly_animating_nodes.is_empty()
    }

    /// Collect newly animating nodes, which is used by the script process during
    /// forced, synchronous reflows to root DOM nodes for the duration of their
    /// animations or transitions.
    /// TODO(mrobinson): Look into handling the rooting inside this class.
    fn collect_newly_animating_nodes(
        &mut self,
        animation_states: &FxHashMap<OpaqueNode, ElementAnimationSet>,
    ) {
        // This extends the output vector with an iterator that contains a copy of the node
        // address for every new animation. The script thread currently stores a rooted node
        // for every property that is transitioning. The current strategy of repeating the
        // node address is a holdover from when the code here looked different.
        self.newly_animating_nodes
            .extend(animation_states.iter().flat_map(|(node, state)| {
                let mut num_new_animations = state
                    .animations
                    .iter()
                    .filter(|animation| animation.is_new)
                    .count();
                num_new_animations += state
                    .transitions
                    .iter()
                    .filter(|transition| transition.is_new)
                    .count();

                let node = UntrustedNodeAddress(node.0 as *const c_void);
                std::iter::repeat(node).take(num_new_animations)
            }));
    }
}

/// The type of transition event to trigger. These are defined by
/// CSS Transitions § 6.1 and CSS Animations § 4.2
#[derive(Clone, Debug, Deserialize, JSTraceable, Serialize)]
pub enum TransitionOrAnimationEventType {
    /// "The transitionrun event occurs when a transition is created (i.e., when it
    /// is added to the set of running transitions)."
    TransitionRun,
    /// "The transitionend event occurs at the completion of the transition. In the
    /// case where a transition is removed before completion, such as if the
    /// transition-property is removed, then the event will not fire."
    TransitionEnd,
    /// "The transitioncancel event occurs when a transition is canceled."
    TransitionCancel,
    /// "The animationend event occurs when the animation finishes"
    AnimationEnd,
}

impl TransitionOrAnimationEventType {
    /// Whether or not this event finalizes the animation or transition. During finalization
    /// the DOM object associated with this transition or animation is unrooted.
    pub fn finalizes_transition_or_animation(&self) -> bool {
        match *self {
            Self::TransitionEnd | Self::TransitionCancel | Self::AnimationEnd => true,
            Self::TransitionRun => false,
        }
    }

    /// Whether or not this event is a transition-related event.
    pub fn is_transition_event(&self) -> bool {
        match *self {
            Self::TransitionRun | Self::TransitionEnd | Self::TransitionCancel => true,
            Self::AnimationEnd => false,
        }
    }
}

#[derive(Deserialize, JSTraceable, Serialize)]
/// A transition or animation event.
pub struct TransitionOrAnimationEvent {
    /// The pipeline id of the layout task that sent this message.
    pub pipeline_id: PipelineId,
    /// The type of transition event this should trigger.
    pub event_type: TransitionOrAnimationEventType,
    /// The address of the node which owns this transition.
    pub node: UntrustedNodeAddress,
    /// The name of the property that is transitioning (in the case of a transition)
    /// or the name of the animation (in the case of an animation).
    pub property_or_animation_name: String,
    /// The elapsed time property to send with this transition event.
    pub elapsed_time: f64,
}
