/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(missing_docs)]

//! The set of animations for a document.

use crate::dom::animationevent::AnimationEvent;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::AnimationEventBinding::AnimationEventInit;
use crate::dom::bindings::codegen::Bindings::EventBinding::EventInit;
use crate::dom::bindings::codegen::Bindings::TransitionEventBinding::TransitionEventInit;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::event::Event;
use crate::dom::node::{from_untrusted_node_address, window_from_node, Node, NodeDamage};
use crate::dom::transitionevent::TransitionEvent;
use crate::dom::window::Window;
use fxhash::FxHashMap;
use libc::c_void;
use msg::constellation_msg::PipelineId;
use parking_lot::RwLock;
use script_traits::{AnimationState as AnimationsPresentState, ScriptMsg, UntrustedNodeAddress};
use servo_arc::Arc;
use std::cell::Cell;
use style::animation::{
    Animation, AnimationState, ElementAnimationSet, KeyframesIterationState, Transition,
};
use style::dom::OpaqueNode;

/// The set of animations for a document.
#[derive(Default, JSTraceable, MallocSizeOf)]
#[unrooted_must_root_lint::must_root]
pub(crate) struct Animations {
    /// The map of nodes to their animation states.
    #[ignore_malloc_size_of = "Arc is hard"]
    pub sets: Arc<RwLock<FxHashMap<OpaqueNode, ElementAnimationSet>>>,

    /// Whether or not we have animations that are running.
    have_running_animations: Cell<bool>,

    /// A list of nodes with in-progress CSS transitions or pending events.
    rooted_nodes: DomRefCell<FxHashMap<OpaqueNode, Dom<Node>>>,

    /// A list of pending animation-related events.
    pending_events: DomRefCell<Vec<TransitionOrAnimationEvent>>,
}

impl Animations {
    pub(crate) fn new() -> Self {
        Animations {
            sets: Default::default(),
            have_running_animations: Cell::new(false),
            rooted_nodes: Default::default(),
            pending_events: Default::default(),
        }
    }

    pub(crate) fn clear(&self) {
        self.sets.write().clear();
        self.rooted_nodes.borrow_mut().clear();
        self.pending_events.borrow_mut().clear();
    }

    pub(crate) fn mark_animating_nodes_as_dirty(&self) {
        let sets = self.sets.read();
        let rooted_nodes = self.rooted_nodes.borrow();
        for node in sets.keys().filter_map(|node| rooted_nodes.get(&node)) {
            node.dirty(NodeDamage::NodeStyleDamaged);
        }
    }

    pub(crate) fn update_for_new_timeline_value(&self, window: &Window, now: f64) {
        let pipeline_id = window.pipeline_id();
        let mut sets = self.sets.write();

        for set in sets.values_mut() {
            // When necessary, iterate our running animations to the next iteration.
            for animation in set.animations.iter_mut() {
                if animation.iterate_if_necessary(now) {
                    self.add_animation_event(
                        animation,
                        TransitionOrAnimationEventType::AnimationIteration,
                        pipeline_id,
                    );
                }
            }

            self.finish_running_animations(set, now, pipeline_id);
        }

        self.unroot_unused_nodes(&sets);
    }

    /// Processes any new animations that were discovered after reflow. Collect messages
    /// that trigger events for any animations that changed state.
    /// TODO(mrobinson): The specification dictates that this should happen before reflow.
    pub(crate) fn do_post_reflow_update(&self, window: &Window, now: f64) {
        let pipeline_id = window.pipeline_id();
        let mut sets = self.sets.write();
        self.root_newly_animating_dom_nodes(&sets, window);

        for set in sets.values_mut() {
            self.handle_canceled_animations(set, now, pipeline_id);
            self.handle_new_animations(set, now, pipeline_id);
        }

        // Remove empty states from our collection of states in order to free
        // up space as soon as we are no longer tracking any animations for
        // a node.
        sets.retain(|_, state| !state.is_empty());
        let have_running_animations = sets.values().any(|state| state.needs_animation_ticks());

        self.update_running_animations_presence(window, have_running_animations);
    }

    fn update_running_animations_presence(&self, window: &Window, new_value: bool) {
        let have_running_animations = self.have_running_animations.get();
        if new_value == have_running_animations {
            return;
        }

        self.have_running_animations.set(new_value);
        let state = match new_value {
            true => AnimationsPresentState::AnimationsPresent,
            false => AnimationsPresentState::NoAnimationsPresent,
        };

        window.send_to_constellation(ScriptMsg::ChangeRunningAnimationsState(state));
    }

    pub(crate) fn running_animation_count(&self) -> usize {
        self.sets
            .read()
            .values()
            .map(|state| state.running_animation_and_transition_count())
            .sum()
    }

    /// Walk through the list of running animations and remove all of the ones that
    /// have ended.
    fn finish_running_animations(
        &self,
        set: &mut ElementAnimationSet,
        now: f64,
        pipeline_id: PipelineId,
    ) {
        for animation in set.animations.iter_mut() {
            if animation.state == AnimationState::Running && animation.has_ended(now) {
                animation.state = AnimationState::Finished;
                self.add_animation_event(
                    animation,
                    TransitionOrAnimationEventType::AnimationEnd,
                    pipeline_id,
                );
            }
        }

        for transition in set.transitions.iter_mut() {
            if transition.state == AnimationState::Running && transition.has_ended(now) {
                transition.state = AnimationState::Finished;
                self.add_transition_event(
                    transition,
                    TransitionOrAnimationEventType::TransitionEnd,
                    now,
                    pipeline_id,
                );
            }
        }
    }

    /// Send events for canceled animations. Currently this only handles canceled
    /// transitions, but eventually this should handle canceled CSS animations as
    /// well.
    fn handle_canceled_animations(
        &self,
        set: &mut ElementAnimationSet,
        now: f64,
        pipeline_id: PipelineId,
    ) {
        for transition in &set.transitions {
            if transition.state == AnimationState::Canceled {
                self.add_transition_event(
                    transition,
                    TransitionOrAnimationEventType::TransitionCancel,
                    now,
                    pipeline_id,
                );
            }
        }

        // TODO(mrobinson): We need to send animationcancel events.
        set.clear_canceled_animations();
    }

    fn handle_new_animations(
        &self,
        set: &mut ElementAnimationSet,
        now: f64,
        pipeline_id: PipelineId,
    ) {
        for animation in set.animations.iter_mut() {
            animation.is_new = false;
        }

        for transition in set.transitions.iter_mut() {
            if transition.is_new {
                self.add_transition_event(
                    transition,
                    TransitionOrAnimationEventType::TransitionRun,
                    now,
                    pipeline_id,
                );
                transition.is_new = false;
            }
        }
    }

    /// Ensure that all nodes with new animations are rooted. This should be called
    /// immediately after a restyle, to ensure that these addresses are still valid.
    #[allow(unsafe_code)]
    fn root_newly_animating_dom_nodes(
        &self,
        sets: &FxHashMap<OpaqueNode, ElementAnimationSet>,
        window: &Window,
    ) {
        let js_runtime = window.get_js_runtime().as_ref().unwrap().rt();
        let mut rooted_nodes = self.rooted_nodes.borrow_mut();
        for (opaque_node, set) in sets.iter() {
            if rooted_nodes.contains_key(opaque_node) {
                continue;
            }

            if set.animations.iter().any(|animation| animation.is_new) ||
                set.transitions.iter().any(|transition| transition.is_new)
            {
                let address = UntrustedNodeAddress(opaque_node.0 as *const c_void);
                unsafe {
                    rooted_nodes.insert(
                        opaque_node.clone(),
                        Dom::from_ref(&*from_untrusted_node_address(js_runtime, address)),
                    )
                };
            }
        }
    }

    // Unroot any nodes that we have rooted but are no longer tracking animations for.
    fn unroot_unused_nodes(&self, sets: &FxHashMap<OpaqueNode, ElementAnimationSet>) {
        let pending_events = self.pending_events.borrow();
        self.rooted_nodes.borrow_mut().retain(|key, _| {
            sets.contains_key(key) || pending_events.iter().any(|event| event.node == *key)
        });
    }

    fn add_transition_event(
        &self,
        transition: &Transition,
        event_type: TransitionOrAnimationEventType,
        now: f64,
        pipeline_id: PipelineId,
    ) {
        let elapsed_time = match event_type {
            TransitionOrAnimationEventType::TransitionRun |
            TransitionOrAnimationEventType::TransitionEnd => transition.property_animation.duration,
            TransitionOrAnimationEventType::TransitionCancel => {
                (now - transition.start_time).max(0.)
            },
            _ => unreachable!(),
        };

        self.pending_events
            .borrow_mut()
            .push(TransitionOrAnimationEvent {
                pipeline_id,
                event_type,
                node: transition.node.clone(),
                property_or_animation_name: transition
                    .property_animation
                    .property_id()
                    .name()
                    .into(),
                elapsed_time,
            });
    }

    fn add_animation_event(
        &self,
        animation: &Animation,
        event_type: TransitionOrAnimationEventType,
        pipeline_id: PipelineId,
    ) {
        let num_iterations = match animation.iteration_state {
            KeyframesIterationState::Finite(current, _) |
            KeyframesIterationState::Infinite(current) => current,
        };

        let elapsed_time = match event_type {
            TransitionOrAnimationEventType::AnimationIteration |
            TransitionOrAnimationEventType::AnimationEnd => num_iterations * animation.duration,
            _ => unreachable!(),
        };

        self.pending_events
            .borrow_mut()
            .push(TransitionOrAnimationEvent {
                pipeline_id,
                event_type,
                node: animation.node.clone(),
                property_or_animation_name: animation.name.to_string(),
                elapsed_time,
            });
    }

    pub(crate) fn send_pending_events(&self) {
        // Take all of the events here, in case sending one of these events
        // triggers adding new events by forcing a layout.
        let events = std::mem::replace(&mut *self.pending_events.borrow_mut(), Vec::new());

        for event in events.into_iter() {
            // We root the node here to ensure that sending this event doesn't
            // unroot it as a side-effect.
            let node = match self.rooted_nodes.borrow().get(&event.node) {
                Some(node) => DomRoot::from_ref(&**node),
                None => {
                    warn!("Tried to send an event for an unrooted node");
                    continue;
                },
            };

            let event_atom = match event.event_type {
                TransitionOrAnimationEventType::AnimationEnd => atom!("animationend"),
                TransitionOrAnimationEventType::AnimationIteration => atom!("animationiteration"),
                TransitionOrAnimationEventType::TransitionCancel => atom!("transitioncancel"),
                TransitionOrAnimationEventType::TransitionEnd => atom!("transitionend"),
                TransitionOrAnimationEventType::TransitionRun => atom!("transitionrun"),
            };
            let parent = EventInit {
                bubbles: true,
                cancelable: false,
            };

            // TODO: Handle pseudo-elements properly
            let property_or_animation_name =
                DOMString::from(event.property_or_animation_name.clone());
            let elapsed_time = Finite::new(event.elapsed_time as f32).unwrap();
            let window = window_from_node(&*node);

            if event.event_type.is_transition_event() {
                let event_init = TransitionEventInit {
                    parent,
                    propertyName: property_or_animation_name,
                    elapsedTime: elapsed_time,
                    pseudoElement: DOMString::new(),
                };
                TransitionEvent::new(&window, event_atom, &event_init)
                    .upcast::<Event>()
                    .fire(node.upcast());
            } else {
                let event_init = AnimationEventInit {
                    parent,
                    animationName: property_or_animation_name,
                    elapsedTime: elapsed_time,
                    pseudoElement: DOMString::new(),
                };
                AnimationEvent::new(&window, event_atom, &event_init)
                    .upcast::<Event>()
                    .fire(node.upcast());
            }
        }
    }
}

/// The type of transition event to trigger. These are defined by
/// CSS Transitions § 6.1 and CSS Animations § 4.2
#[derive(Clone, Debug, Deserialize, JSTraceable, MallocSizeOf, Serialize)]
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
    /// "The animationiteration event occurs at the end of each iteration of an
    /// animation, except when an animationend event would fire at the same time."
    AnimationIteration,
    /// "The animationend event occurs when the animation finishes"
    AnimationEnd,
}

impl TransitionOrAnimationEventType {
    /// Whether or not this event is a transition-related event.
    pub fn is_transition_event(&self) -> bool {
        match *self {
            Self::TransitionRun | Self::TransitionEnd | Self::TransitionCancel => true,
            Self::AnimationEnd | Self::AnimationIteration => false,
        }
    }
}

#[derive(Deserialize, JSTraceable, MallocSizeOf, Serialize)]
/// A transition or animation event.
pub struct TransitionOrAnimationEvent {
    /// The pipeline id of the layout task that sent this message.
    pub pipeline_id: PipelineId,
    /// The type of transition event this should trigger.
    pub event_type: TransitionOrAnimationEventType,
    /// The address of the node which owns this transition.
    pub node: OpaqueNode,
    /// The name of the property that is transitioning (in the case of a transition)
    /// or the name of the animation (in the case of an animation).
    pub property_or_animation_name: String,
    /// The elapsed time property to send with this transition event.
    pub elapsed_time: f64,
}
