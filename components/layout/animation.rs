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
    AnimationState as AnimationsPresentState, ConstellationControlMsg,
    LayoutMsg as ConstellationMsg, TransitionOrAnimationEvent, TransitionOrAnimationEventType,
};
use style::animation::{AnimationState, ElementAnimationSet};

/// Processes any new animations that were discovered after style recalculation and
/// remove animations for any disconnected nodes. Send messages that trigger events
/// for any events that changed state.
pub fn do_post_style_animations_update(
    constellation_chan: &IpcSender<ConstellationMsg>,
    script_chan: &IpcSender<ConstellationControlMsg>,
    animation_states: &mut FxHashMap<OpaqueNode, ElementAnimationSet>,
    pipeline_id: PipelineId,
    now: f64,
    out: &mut Vec<UntrustedNodeAddress>,
    root_flow: &mut dyn Flow,
) {
    let had_running_animations = animation_states
        .values()
        .any(|state| state.needs_animation_ticks());

    cancel_animations_for_disconnected_nodes(animation_states, root_flow);
    collect_newly_animating_nodes(animation_states, out);

    for (node, animation_state) in animation_states.iter_mut() {
        update_animation_state(script_chan, animation_state, pipeline_id, now, *node);
    }

    // Remove empty states from our collection of states in order to free
    // up space as soon as we are no longer tracking any animations for
    // a node.
    animation_states.retain(|_, state| !state.is_empty());

    let have_running_animations = animation_states
        .values()
        .any(|state| state.needs_animation_ticks());
    let present = match (had_running_animations, have_running_animations) {
        (true, false) => AnimationsPresentState::NoAnimationsPresent,
        (false, true) => AnimationsPresentState::AnimationsPresent,
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
    animation_states: &FxHashMap<OpaqueNode, ElementAnimationSet>,
    out: &mut Vec<UntrustedNodeAddress>,
) {
    // This extends the output vector with an iterator that contains a copy of the node
    // address for every new animation. The script thread currently stores a rooted node
    // for every property that is transitioning. The current strategy of repeating the
    // node address is a holdover from when the code here looked different.
    out.extend(animation_states.iter().flat_map(|(node, state)| {
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
        std::iter::repeat(node.to_untrusted_node_address()).take(num_new_animations)
    }));
}

/// Cancel animations for any nodes which have been removed from the DOM or are display:none.
/// We detect this by looking for nodes that are used in the flow tree.
/// TODO(mrobinson): We should look into a way of doing this during flow tree construction.
/// This also doesn't yet handles nodes that have been reparented.
pub fn cancel_animations_for_disconnected_nodes(
    animation_states: &mut FxHashMap<OpaqueNode, ElementAnimationSet>,
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
    animation_state: &mut ElementAnimationSet,
    pipeline_id: PipelineId,
    now: f64,
    node: OpaqueNode,
) {
    let send_event = |property_or_animation_name, event_type, elapsed_time| {
        script_channel
            .send(ConstellationControlMsg::TransitionOrAnimationEvent(
                TransitionOrAnimationEvent {
                    pipeline_id,
                    event_type,
                    node: node.to_untrusted_node_address(),
                    property_or_animation_name,
                    elapsed_time,
                },
            ))
            .unwrap()
    };

    handle_canceled_animations(animation_state, now, send_event);
    finish_running_animations(animation_state, now, send_event);
    handle_new_animations(animation_state, send_event);
}

/// Walk through the list of running animations and remove all of the ones that
/// have ended.
fn finish_running_animations(
    animation_state: &mut ElementAnimationSet,
    now: f64,
    mut send_event: impl FnMut(String, TransitionOrAnimationEventType, f64),
) {
    for animation in animation_state.animations.iter_mut() {
        if animation.state == AnimationState::Running && animation.has_ended(now) {
            animation.state = AnimationState::Finished;
            send_event(
                animation.name.to_string(),
                TransitionOrAnimationEventType::AnimationEnd,
                animation.active_duration(),
            );
        }
    }

    for transition in animation_state.transitions.iter_mut() {
        if transition.state == AnimationState::Running && transition.has_ended(now) {
            transition.state = AnimationState::Finished;
            send_event(
                transition.property_animation.property_name().into(),
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
    animation_state: &mut ElementAnimationSet,
    now: f64,
    mut send_event: impl FnMut(String, TransitionOrAnimationEventType, f64),
) {
    for transition in &animation_state.transitions {
        if transition.state == AnimationState::Canceled {
            // TODO(mrobinson): We need to properly compute the elapsed_time here
            // according to https://drafts.csswg.org/css-transitions/#event-transitionevent
            send_event(
                transition.property_animation.property_name().into(),
                TransitionOrAnimationEventType::TransitionCancel,
                (now - transition.start_time).max(0.),
            );
        }
    }

    // TODO(mrobinson): We need to send animationcancel events.
    animation_state.clear_canceled_animations();
}

fn handle_new_animations(
    animation_state: &mut ElementAnimationSet,
    mut send_event: impl FnMut(String, TransitionOrAnimationEventType, f64),
) {
    for animation in animation_state.animations.iter_mut() {
        animation.is_new = false;
    }

    for transition in animation_state.transitions.iter_mut() {
        if transition.is_new {
            // TODO(mrobinson): We need to properly compute the elapsed_time here
            // according to https://drafts.csswg.org/css-transitions/#event-transitionevent
            send_event(
                transition.property_animation.property_name().into(),
                TransitionOrAnimationEventType::TransitionRun,
                0.,
            );
            transition.is_new = false;
        }
    }
}
