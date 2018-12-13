/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! CSS transitions and animations.

use crate::context::LayoutContext;
use crate::display_list::items::OpaqueNode;
use crate::flow::{Flow, GetBaseFlow};
use crate::opaque_node::OpaqueNodeMethods;
use crossbeam_channel::Receiver;
use fxhash::{FxHashMap, FxHashSet};
use ipc_channel::ipc::IpcSender;
use msg::constellation_msg::PipelineId;
use script_traits::UntrustedNodeAddress;
use script_traits::{AnimationState, ConstellationControlMsg, LayoutMsg as ConstellationMsg};
use style::animation::{update_style_for_animation, Animation};
use style::dom::TElement;
use style::font_metrics::ServoMetricsProvider;
use style::selector_parser::RestyleDamage;
use style::timer::Timer;

/// Processes any new animations that were discovered after style recalculation.
/// Also expire any old animations that have completed, inserting them into
/// `expired_animations`.
pub fn update_animation_state<E>(
    constellation_chan: &IpcSender<ConstellationMsg>,
    script_chan: &IpcSender<ConstellationControlMsg>,
    running_animations: &mut FxHashMap<OpaqueNode, Vec<Animation>>,
    expired_animations: &mut FxHashMap<OpaqueNode, Vec<Animation>>,
    mut keys_to_remove: FxHashSet<OpaqueNode>,
    mut newly_transitioning_nodes: Option<&mut Vec<UntrustedNodeAddress>>,
    new_animations_receiver: &Receiver<Animation>,
    pipeline_id: PipelineId,
    timer: &Timer,
) where
    E: TElement,
{
    let mut new_running_animations = vec![];
    while let Ok(animation) = new_animations_receiver.try_recv() {
        let mut should_push = true;
        if let Animation::Keyframes(ref node, _, ref name, ref state) = animation {
            // If the animation was already present in the list for the
            // node, just update its state, else push the new animation to
            // run.
            if let Some(ref mut animations) = running_animations.get_mut(node) {
                // TODO: This being linear is probably not optimal.
                for anim in animations.iter_mut() {
                    if let Animation::Keyframes(_, _, ref anim_name, ref mut anim_state) = *anim {
                        if *name == *anim_name {
                            debug!("update_animation_state: Found other animation {}", name);
                            anim_state.update_from_other(&state, timer);
                            should_push = false;
                            break;
                        }
                    }
                }
            }
        }

        if should_push {
            new_running_animations.push(animation);
        }
    }

    if running_animations.is_empty() && new_running_animations.is_empty() {
        // Nothing to do. Return early so we don't flood the compositor with
        // `ChangeRunningAnimationsState` messages.
        return;
    }

    let now = timer.seconds();
    // Expire old running animations.
    //
    // TODO: Do not expunge Keyframes animations, since we need that state if
    // the animation gets re-triggered. Probably worth splitting in two
    // different maps, or at least using a linked list?
    for (key, running_animations) in running_animations.iter_mut() {
        let mut animations_still_running = vec![];
        for mut running_animation in running_animations.drain(..) {
            let still_running = !running_animation.is_expired() && match running_animation {
                Animation::Transition(_, started_at, ref frame) => {
                    now < started_at + frame.duration
                },
                Animation::Keyframes(_, _, _, ref mut state) => {
                    // This animation is still running, or we need to keep
                    // iterating.
                    now < state.started_at + state.duration || state.tick()
                },
            };

            debug!(
                "update_animation_state({:?}): {:?}",
                still_running, running_animation
            );

            if still_running {
                animations_still_running.push(running_animation);
                continue;
            }

            if let Animation::Transition(node, _, ref frame) = running_animation {
                script_chan
                    .send(ConstellationControlMsg::TransitionEnd(
                        node.to_untrusted_node_address(),
                        frame.property_animation.property_name().into(),
                        frame.duration,
                    ))
                    .unwrap();
            }

            expired_animations
                .entry(*key)
                .or_insert_with(Vec::new)
                .push(running_animation);
        }

        if animations_still_running.is_empty() {
            keys_to_remove.insert(*key);
        } else {
            *running_animations = animations_still_running
        }
    }

    for key in keys_to_remove {
        running_animations.remove(&key).unwrap();
    }

    // Add new running animations.
    for new_running_animation in new_running_animations {
        if new_running_animation.is_transition() {
            match newly_transitioning_nodes {
                Some(ref mut nodes) => {
                    nodes.push(new_running_animation.node().to_untrusted_node_address());
                },
                None => {
                    warn!("New transition encountered from compositor-initiated layout.");
                },
            }
        }

        running_animations
            .entry(*new_running_animation.node())
            .or_insert_with(Vec::new)
            .push(new_running_animation)
    }

    let animation_state = if running_animations.is_empty() {
        AnimationState::NoAnimationsPresent
    } else {
        AnimationState::AnimationsPresent
    };

    constellation_chan
        .send(ConstellationMsg::ChangeRunningAnimationsState(
            pipeline_id,
            animation_state,
        ))
        .unwrap();
}

/// Recalculates style for a set of animations. This does *not* run with the DOM
/// lock held. Returns a set of nodes associated with animations that are no longer
/// valid.
pub fn recalc_style_for_animations<E>(
    context: &LayoutContext,
    flow: &mut dyn Flow,
    animations: &FxHashMap<OpaqueNode, Vec<Animation>>,
) -> FxHashSet<OpaqueNode>
where
    E: TElement,
{
    let mut invalid_nodes = animations.keys().cloned().collect();
    do_recalc_style_for_animations::<E>(context, flow, animations, &mut invalid_nodes);
    invalid_nodes
}

fn do_recalc_style_for_animations<E>(
    context: &LayoutContext,
    flow: &mut dyn Flow,
    animations: &FxHashMap<OpaqueNode, Vec<Animation>>,
    invalid_nodes: &mut FxHashSet<OpaqueNode>,
) where
    E: TElement,
{
    let mut damage = RestyleDamage::empty();
    flow.mutate_fragments(&mut |fragment| {
        if let Some(ref animations) = animations.get(&fragment.node) {
            invalid_nodes.remove(&fragment.node);
            for animation in animations.iter() {
                let old_style = fragment.style.clone();
                update_style_for_animation::<E>(
                    &context.style_context,
                    animation,
                    &mut fragment.style,
                    &ServoMetricsProvider,
                );
                let difference =
                    RestyleDamage::compute_style_difference(&old_style, &fragment.style);
                damage |= difference.damage;
            }
        }
    });

    let base = flow.mut_base();
    base.restyle_damage.insert(damage);
    for kid in base.children.iter_mut() {
        do_recalc_style_for_animations::<E>(context, kid, animations, invalid_nodes)
    }
}
