/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS transitions and animations.

use context::SharedLayoutContext;
use flow::{self, Flow};
use gfx::display_list::OpaqueNode;
use ipc_channel::ipc::IpcSender;
use msg::constellation_msg::PipelineId;
use script_traits::{AnimationState, ConstellationControlMsg, LayoutMsg as ConstellationMsg};
use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use style::animation::{Animation, update_style_for_animation};
use style::selector_parser::RestyleDamage;
use style::timer::Timer;

/// Processes any new animations that were discovered after style recalculation.
/// Also expire any old animations that have completed, inserting them into
/// `expired_animations`.
pub fn update_animation_state(constellation_chan: &IpcSender<ConstellationMsg>,
                              script_chan: &IpcSender<ConstellationControlMsg>,
                              running_animations: &mut HashMap<OpaqueNode, Vec<Animation>>,
                              expired_animations: &mut HashMap<OpaqueNode, Vec<Animation>>,
                              new_animations_receiver: &Receiver<Animation>,
                              pipeline_id: PipelineId,
                              timer: &Timer) {
    let mut new_running_animations = vec![];
    while let Ok(animation) = new_animations_receiver.try_recv() {
        let mut should_push = true;
        if let Animation::Keyframes(ref node, ref name, ref state) = animation {
            // If the animation was already present in the list for the
            // node, just update its state, else push the new animation to
            // run.
            if let Some(ref mut animations) = running_animations.get_mut(node) {
                // TODO: This being linear is probably not optimal.
                for mut anim in animations.iter_mut() {
                    if let Animation::Keyframes(_, ref anim_name, ref mut anim_state) = *anim {
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
        return
    }

    let now = timer.seconds();
    // Expire old running animations.
    //
    // TODO: Do not expunge Keyframes animations, since we need that state if
    // the animation gets re-triggered. Probably worth splitting in two
    // different maps, or at least using a linked list?
    let mut keys_to_remove = vec![];
    for (key, running_animations) in running_animations.iter_mut() {
        let mut animations_still_running = vec![];
        for mut running_animation in running_animations.drain(..) {
            let still_running = !running_animation.is_expired() && match running_animation {
                Animation::Transition(_, _, started_at, ref frame, _expired) => {
                    now < started_at + frame.duration
                }
                Animation::Keyframes(_, _, ref mut state) => {
                    // This animation is still running, or we need to keep
                    // iterating.
                    now < state.started_at + state.duration || state.tick()
                }
            };

            if still_running {
                animations_still_running.push(running_animation);
                continue
            }

            if let Animation::Transition(_, unsafe_node, _, ref frame, _) = running_animation {
                script_chan.send(ConstellationControlMsg::TransitionEnd(unsafe_node,
                                                                        frame.property_animation
                                                                             .property_name().into(),
                                                                        frame.duration))
                           .unwrap();
            }

            expired_animations.entry(*key)
                              .or_insert_with(Vec::new)
                              .push(running_animation);
        }

        if animations_still_running.is_empty() {
            keys_to_remove.push(*key);
        } else {
            *running_animations = animations_still_running
        }
    }

    for key in keys_to_remove {
        running_animations.remove(&key).unwrap();
    }

    // Add new running animations.
    for new_running_animation in new_running_animations {
        running_animations.entry(*new_running_animation.node())
                          .or_insert_with(Vec::new)
                          .push(new_running_animation)
    }

    let animation_state = if running_animations.is_empty() {
        AnimationState::NoAnimationsPresent
    } else {
        AnimationState::AnimationsPresent
    };

    constellation_chan.send(ConstellationMsg::ChangeRunningAnimationsState(pipeline_id,
                                                                           animation_state))
                      .unwrap();
}

/// Recalculates style for a set of animations. This does *not* run with the DOM
/// lock held.
// NB: This is specific for SelectorImpl, since the layout context and the
// flows are SelectorImpl specific too. If that goes away at some point,
// this should be made generic.
pub fn recalc_style_for_animations(context: &SharedLayoutContext,
                                   flow: &mut Flow,
                                   animations: &HashMap<OpaqueNode,
                                                        Vec<Animation>>) {
    let mut damage = RestyleDamage::empty();
    flow.mutate_fragments(&mut |fragment| {
        if let Some(ref animations) = animations.get(&fragment.node) {
            for animation in animations.iter() {
                let old_style = fragment.style.clone();
                update_style_for_animation(&context.style_context,
                                           animation,
                                           &mut fragment.style);
                damage |= RestyleDamage::compute(&old_style, &fragment.style);
            }
        }
    });

    let base = flow::mut_base(flow);
    base.restyle_damage.insert(damage);
    for kid in base.children.iter_mut() {
        recalc_style_for_animations(context, kid, animations)
    }
}
