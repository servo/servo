/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! CSS transitions and animations.

use crate::context::LayoutContext;
use crate::display_list::items::OpaqueNode;
use crate::flow::{Flow, GetBaseFlow};
use crate::opaque_node::OpaqueNodeMethods;
use fxhash::{FxHashMap, FxHashSet};
use ipc_channel::ipc::IpcSender;
use msg::constellation_msg::PipelineId;
use script_traits::UntrustedNodeAddress;
use script_traits::{
    AnimationState, ConstellationControlMsg, LayoutMsg as ConstellationMsg, TransitionEventType,
};
use servo_arc::Arc;
use style::animation::{
    update_style_for_animation, Animation, ElementAnimationState, PropertyAnimation,
};
use style::dom::TElement;
use style::font_metrics::ServoMetricsProvider;
use style::selector_parser::RestyleDamage;
use style::timer::Timer;

/// Collect newly transitioning nodes, which is used by the script process during
/// forced, synchronous reflows to root DOM nodes for the duration of their transitions.
pub fn collect_newly_transitioning_nodes(
    animation_states: &FxHashMap<OpaqueNode, ElementAnimationState>,
    mut out: Option<&mut Vec<UntrustedNodeAddress>>,
) {
    // This extends the output vector with an iterator that contains a copy of the node
    // address for every new animation. This is a bit goofy, but the script thread
    // currently stores a rooted node for every property that is transitioning.
    if let Some(ref mut out) = out {
        out.extend(animation_states.iter().flat_map(|(node, state)| {
            let num_transitions = state
                .new_animations
                .iter()
                .filter(|animation| animation.is_transition())
                .count();
            std::iter::repeat(node.to_untrusted_node_address()).take(num_transitions)
        }));
    }
}

/// Processes any new animations that were discovered after style recalculation. Also
/// finish any animations that have completed, inserting them into `finished_animations`.
pub fn update_animation_states(
    constellation_chan: &IpcSender<ConstellationMsg>,
    script_chan: &IpcSender<ConstellationControlMsg>,
    animation_states: &mut FxHashMap<OpaqueNode, ElementAnimationState>,
    invalid_nodes: FxHashSet<OpaqueNode>,
    pipeline_id: PipelineId,
    timer: &Timer,
) {
    let had_running_animations = animation_states
        .values()
        .any(|state| !state.running_animations.is_empty());

    // Cancel all animations on any invalid nodes. These entries will later
    // be removed from the list of states, because their states will become
    // empty.
    for node in &invalid_nodes {
        if let Some(mut state) = animation_states.remove(node) {
            state.cancel_all_animations();
        }
    }

    let now = timer.seconds();
    let mut have_running_animations = false;
    for (node, animation_state) in animation_states.iter_mut() {
        update_animation_state(script_chan, animation_state, pipeline_id, now, *node);
        have_running_animations =
            have_running_animations || !animation_state.running_animations.is_empty();
    }

    // Remove empty states from our collection of states in order to free
    // up space as soon as we are no longer tracking any animations for
    // a node.
    animation_states.retain(|_, state| !state.is_empty());

    let present = match (had_running_animations, have_running_animations) {
        (true, false) => AnimationState::NoAnimationsPresent,
        (false, true) => AnimationState::AnimationsPresent,
        _ => return,
    };
    constellation_chan
        .send(ConstellationMsg::ChangeRunningAnimationsState(
            pipeline_id,
            present,
        ))
        .unwrap();
}

pub fn update_animation_state(
    script_channel: &IpcSender<ConstellationControlMsg>,
    animation_state: &mut ElementAnimationState,
    pipeline_id: PipelineId,
    now: f64,
    node: OpaqueNode,
) {
    let send_transition_event = |property_animation: &PropertyAnimation, event_type| {
        script_channel
            .send(ConstellationControlMsg::TransitionEvent {
                pipeline_id,
                event_type,
                node: node.to_untrusted_node_address(),
                property_name: property_animation.property_name().into(),
                elapsed_time: property_animation.duration,
            })
            .unwrap()
    };

    handle_cancelled_animations(animation_state, send_transition_event);
    handle_running_animations(animation_state, now, send_transition_event);
    handle_new_animations(animation_state, send_transition_event);
}

/// Walk through the list of running animations and remove all of the ones that
/// have ended.
pub fn handle_running_animations(
    animation_state: &mut ElementAnimationState,
    now: f64,
    mut send_transition_event: impl FnMut(&PropertyAnimation, TransitionEventType),
) {
    let mut running_animations =
        std::mem::replace(&mut animation_state.running_animations, Vec::new());
    for mut running_animation in running_animations.drain(..) {
        let still_running = !running_animation.is_expired() &&
            match running_animation {
                Animation::Transition(_, started_at, ref property_animation) => {
                    now < started_at + (property_animation.duration)
                },
                Animation::Keyframes(_, _, _, ref mut state) => {
                    // This animation is still running, or we need to keep
                    // iterating.
                    now < state.started_at + state.duration || state.tick()
                },
            };

        // If the animation is still running, add it back to the list of running animations.
        if still_running {
            animation_state.running_animations.push(running_animation);
        } else {
            debug!("Finishing transition: {:?}", running_animation);
            if let Animation::Transition(_, _, ref property_animation) = running_animation {
                send_transition_event(property_animation, TransitionEventType::TransitionEnd);
            }
            animation_state.finished_animations.push(running_animation);
        }
    }
}

/// Send events for cancelled animations. Currently this only handles cancelled
/// transitions, but eventually this should handle cancelled CSS animations as
/// well.
pub fn handle_cancelled_animations(
    animation_state: &mut ElementAnimationState,
    mut send_transition_event: impl FnMut(&PropertyAnimation, TransitionEventType),
) {
    for animation in animation_state.cancelled_animations.drain(..) {
        match animation {
            Animation::Transition(_, _, ref property_animation) => {
                send_transition_event(property_animation, TransitionEventType::TransitionCancel)
            },
            Animation::Keyframes(..) => {
                warn!("Got unexpected animation in finished transitions list.")
            },
        }
    }
}

pub fn handle_new_animations(
    animation_state: &mut ElementAnimationState,
    mut send_transition_event: impl FnMut(&PropertyAnimation, TransitionEventType),
) {
    for animation in animation_state.new_animations.drain(..) {
        match animation {
            Animation::Transition(_, _, ref property_animation) => {
                send_transition_event(property_animation, TransitionEventType::TransitionRun)
            },
            Animation::Keyframes(..) => {},
        }
        animation_state.running_animations.push(animation);
    }
}

/// Recalculates style for a set of animations. This does *not* run with the DOM
/// lock held. Returns a set of nodes associated with animations that are no longer
/// valid.
pub fn recalc_style_for_animations<E>(
    context: &LayoutContext,
    flow: &mut dyn Flow,
    animation_states: &FxHashMap<OpaqueNode, ElementAnimationState>,
) -> FxHashSet<OpaqueNode>
where
    E: TElement,
{
    let mut invalid_nodes = animation_states.keys().cloned().collect();
    do_recalc_style_for_animations::<E>(context, flow, animation_states, &mut invalid_nodes);
    invalid_nodes
}

fn do_recalc_style_for_animations<E>(
    context: &LayoutContext,
    flow: &mut dyn Flow,
    animation_states: &FxHashMap<OpaqueNode, ElementAnimationState>,
    invalid_nodes: &mut FxHashSet<OpaqueNode>,
) where
    E: TElement,
{
    let mut damage = RestyleDamage::empty();
    flow.mutate_fragments(&mut |fragment| {
        let animations = match animation_states.get(&fragment.node) {
            Some(state) => &state.running_animations,
            None => return,
        };

        invalid_nodes.remove(&fragment.node);
        for animation in animations.iter() {
            let old_style = fragment.style.clone();
            update_style_for_animation::<E>(
                &context.style_context,
                animation,
                Arc::make_mut(&mut fragment.style),
                &ServoMetricsProvider,
            );
            let difference = RestyleDamage::compute_style_difference(&old_style, &fragment.style);
            damage |= difference.damage;
        }
    });

    let base = flow.mut_base();
    base.restyle_damage.insert(damage);
    for kid in base.children.iter_mut() {
        do_recalc_style_for_animations::<E>(context, kid, animation_states, invalid_nodes)
    }
}
