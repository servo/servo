/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! CSS transitions and animations.

use crate::display_list::items::OpaqueNode;
use crate::flow::{Flow, GetBaseFlow};
use crate::opaque_node::OpaqueNodeMethods;
use fxhash::{FxHashMap, FxHashSet};
use ipc_channel::ipc::IpcSender;
use msg::constellation_msg::PipelineId;
use script_traits::UntrustedNodeAddress;
use script_traits::{
    AnimationState, ConstellationControlMsg, LayoutMsg as ConstellationMsg,
    TransitionOrAnimationEventType,
};
use style::animation::{Animation, ElementAnimationState};

/// Processes any new animations that were discovered after style recalculation and
/// remove animations for any disconnected nodes. Send messages that trigger events
/// for any events that changed state.
pub fn do_post_style_animations_update(
    constellation_chan: &IpcSender<ConstellationMsg>,
    script_chan: &IpcSender<ConstellationControlMsg>,
    animation_states: &mut FxHashMap<OpaqueNode, ElementAnimationState>,
    pipeline_id: PipelineId,
    now: f64,
    out: &mut Vec<UntrustedNodeAddress>,
    root_flow: &mut dyn Flow,
) {
    let had_running_animations = animation_states
        .values()
        .any(|state| !state.running_animations.is_empty());

    cancel_animations_for_disconnected_nodes(animation_states, root_flow);
    collect_newly_animating_nodes(animation_states, out);

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

/// Collect newly animating nodes, which is used by the script process during
/// forced, synchronous reflows to root DOM nodes for the duration of their
/// animations or transitions.
pub fn collect_newly_animating_nodes(
    animation_states: &FxHashMap<OpaqueNode, ElementAnimationState>,
    out: &mut Vec<UntrustedNodeAddress>,
) {
    // This extends the output vector with an iterator that contains a copy of the node
    // address for every new animation. This is a bit goofy, but the script thread
    // currently stores a rooted node for every property that is transitioning.
    out.extend(animation_states.iter().flat_map(|(node, state)| {
        std::iter::repeat(node.to_untrusted_node_address()).take(state.new_animations.len())
    }));
}

/// Cancel animations for any nodes which have been removed from the DOM or are display:none.
/// We detect this by looking for nodes that are used in the flow tree.
/// TODO(mrobinson): We should look into a way of doing this during flow tree construction.
/// This also doesn't yet handles nodes that have been reparented.
pub fn cancel_animations_for_disconnected_nodes(
    animation_states: &mut FxHashMap<OpaqueNode, ElementAnimationState>,
    root_flow: &mut dyn Flow,
) {
    // Assume all nodes have been removed until proven otherwise.
    let mut invalid_nodes: FxHashSet<OpaqueNode> = animation_states.keys().cloned().collect();
    fn traverse_flow(flow: &mut dyn Flow, invalid_nodes: &mut FxHashSet<OpaqueNode>) {
        flow.mutate_fragments(&mut |fragment| {
            invalid_nodes.remove(&fragment.node);
        });
        for kid in flow.mut_base().children.iter_mut() {
            traverse_flow(kid, invalid_nodes)
        }
    }
    traverse_flow(root_flow, &mut invalid_nodes);

    // Cancel animations for any nodes that are no longer in the flow tree.
    for node in &invalid_nodes {
        if let Some(state) = animation_states.get_mut(node) {
            state.cancel_all_animations();
        }
    }
}

fn update_animation_state(
    script_channel: &IpcSender<ConstellationControlMsg>,
    animation_state: &mut ElementAnimationState,
    pipeline_id: PipelineId,
    now: f64,
    node: OpaqueNode,
) {
    let send_event = |animation: &Animation, event_type, elapsed_time| {
        let property_or_animation_name = match *animation {
            Animation::Transition(_, _, ref property_animation) => {
                property_animation.property_name().into()
            },
            Animation::Keyframes(_, _, ref name, _) => name.to_string(),
        };

        script_channel
            .send(ConstellationControlMsg::TransitionOrAnimationEvent {
                pipeline_id,
                event_type,
                node: node.to_untrusted_node_address(),
                property_or_animation_name,
                elapsed_time,
            })
            .unwrap()
    };

    handle_cancelled_animations(animation_state, now, send_event);
    handle_running_animations(animation_state, now, send_event);
    handle_new_animations(animation_state, send_event);
}

/// Walk through the list of running animations and remove all of the ones that
/// have ended.
fn handle_running_animations(
    animation_state: &mut ElementAnimationState,
    now: f64,
    mut send_event: impl FnMut(&Animation, TransitionOrAnimationEventType, f64),
) {
    if animation_state.running_animations.is_empty() {
        return;
    }

    let mut running_animations =
        std::mem::replace(&mut animation_state.running_animations, Vec::new());
    for running_animation in running_animations.drain(..) {
        // If the animation is still running, add it back to the list of running animations.
        if !running_animation.has_ended(now) {
            animation_state.running_animations.push(running_animation);
        } else {
            let (event_type, elapsed_time) = match running_animation {
                Animation::Transition(_, _, ref property_animation) => (
                    TransitionOrAnimationEventType::TransitionEnd,
                    property_animation.duration,
                ),
                Animation::Keyframes(_, _, _, ref state) => (
                    TransitionOrAnimationEventType::AnimationEnd,
                    state.active_duration(),
                ),
            };

            send_event(&running_animation, event_type, elapsed_time);
            animation_state.finished_animations.push(running_animation);
        }
    }
}

/// Send events for cancelled animations. Currently this only handles cancelled
/// transitions, but eventually this should handle cancelled CSS animations as
/// well.
fn handle_cancelled_animations(
    animation_state: &mut ElementAnimationState,
    now: f64,
    mut send_event: impl FnMut(&Animation, TransitionOrAnimationEventType, f64),
) {
    for animation in animation_state.cancelled_animations.drain(..) {
        match animation {
            Animation::Transition(_, start_time, _) => {
                // TODO(mrobinson): We need to properly compute the elapsed_time here
                // according to https://drafts.csswg.org/css-transitions/#event-transitionevent
                send_event(
                    &animation,
                    TransitionOrAnimationEventType::TransitionCancel,
                    (now - start_time).max(0.),
                );
            },
            // TODO(mrobinson): We should send animationcancel events.
            Animation::Keyframes(..) => {},
        }
    }
}

fn handle_new_animations(
    animation_state: &mut ElementAnimationState,
    mut send_event: impl FnMut(&Animation, TransitionOrAnimationEventType, f64),
) {
    for animation in animation_state.new_animations.drain(..) {
        match animation {
            Animation::Transition(..) => {
                // TODO(mrobinson): We need to properly compute the elapsed_time here
                // according to https://drafts.csswg.org/css-transitions/#event-transitionevent
                send_event(
                    &animation,
                    TransitionOrAnimationEventType::TransitionRun,
                    0.,
                )
            },
            Animation::Keyframes(..) => {},
        }
        animation_state.running_animations.push(animation);
    }
}
