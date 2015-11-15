/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS transitions and animations.

use clock_ticks;
use flow::{self, Flow};
use gfx::display_list::OpaqueNode;
use incremental::{self, RestyleDamage};
use msg::constellation_msg::ScriptMsg as ConstellationMsg;
use msg::constellation_msg::{AnimationState, ConstellationChan, PipelineId};
use script::layout_interface::Animation;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::{Arc, Mutex};
use style::animation::{GetMod, PropertyAnimation};
use style::properties::ComputedValues;

/// Inserts transitions into the queue of running animations as applicable for the given style
/// difference. This is called from the layout worker threads.
pub fn start_transitions_if_applicable(new_animations_sender: &Mutex<Sender<Animation>>,
                                       node: OpaqueNode,
                                       old_style: &ComputedValues,
                                       new_style: &mut ComputedValues) {
    for i in 0..new_style.get_animation().transition_property.0.len() {
        // Create any property animations, if applicable.
        let property_animations = PropertyAnimation::from_transition(i, old_style, new_style);
        for property_animation in property_animations {
            // Set the property to the initial value.
            property_animation.update(new_style, 0.0);

            // Kick off the animation.
            let now = clock_ticks::precise_time_s();
            let animation_style = new_style.get_animation();
            let start_time =
                now + (animation_style.transition_delay.0.get_mod(i).seconds() as f64);
            new_animations_sender.lock().unwrap().send(Animation {
                node: node.id(),
                property_animation: property_animation,
                start_time: start_time,
                end_time: start_time +
                    (animation_style.transition_duration.0.get_mod(i).seconds() as f64),
            }).unwrap()
        }
    }
}

/// Processes any new animations that were discovered after style recalculation.
/// Also expire any old animations that have completed.
pub fn update_animation_state(constellation_chan: &ConstellationChan<ConstellationMsg>,
                              running_animations: &mut Arc<HashMap<OpaqueNode, Vec<Animation>>>,
                              new_animations_receiver: &Receiver<Animation>,
                              pipeline_id: PipelineId) {
    let mut new_running_animations = Vec::new();
    while let Ok(animation) = new_animations_receiver.try_recv() {
        new_running_animations.push(animation)
    }

    let mut running_animations_hash = (**running_animations).clone();

    if running_animations_hash.is_empty() && new_running_animations.is_empty() {
        // Nothing to do. Return early so we don't flood the compositor with
        // `ChangeRunningAnimationsState` messages.
        return
    }

    // Expire old running animations.
    let now = clock_ticks::precise_time_s();
    let mut keys_to_remove = Vec::new();
    for (key, running_animations) in &mut running_animations_hash {
        running_animations.retain(|running_animation| {
            now < running_animation.end_time
        });
        if running_animations.len() == 0 {
            keys_to_remove.push(*key);
        }
    }
    for key in keys_to_remove {
        running_animations_hash.remove(&key).unwrap();
    }

    // Add new running animations.
    for new_running_animation in new_running_animations {
        match running_animations_hash.entry(OpaqueNode(new_running_animation.node)) {
            Entry::Vacant(entry) => {
                entry.insert(vec![new_running_animation]);
            }
            Entry::Occupied(mut entry) => entry.get_mut().push(new_running_animation),
        }
    }

    *running_animations = Arc::new(running_animations_hash);

    let animation_state;
    if running_animations.is_empty() {
        animation_state = AnimationState::NoAnimationsPresent;
    } else {
        animation_state = AnimationState::AnimationsPresent;
    }

    constellation_chan.0
                      .send(ConstellationMsg::ChangeRunningAnimationsState(pipeline_id, animation_state))
                      .unwrap();
}

/// Recalculates style for a set of animations. This does *not* run with the DOM lock held.
pub fn recalc_style_for_animations(flow: &mut Flow,
                                   animations: &HashMap<OpaqueNode, Vec<Animation>>) {
    let mut damage = RestyleDamage::empty();
    flow.mutate_fragments(&mut |fragment| {
        if let Some(ref animations) = animations.get(&OpaqueNode(fragment.node.id())) {
            for animation in *animations {
                let now = clock_ticks::precise_time_s();
                let mut progress = (now - animation.start_time) / animation.duration();
                if progress > 1.0 {
                    progress = 1.0
                }
                if progress <= 0.0 {
                    continue
                }

                let mut new_style = fragment.style.clone();
                animation.property_animation.update(&mut *Arc::make_mut(&mut new_style),
                                                    progress);
                damage.insert(incremental::compute_damage(&Some(fragment.style.clone()),
                                                          &new_style));
                fragment.style = new_style
            }
        }
    });

    let base = flow::mut_base(flow);
    base.restyle_damage.insert(damage);
    for kid in base.children.iter_mut() {
        recalc_style_for_animations(kid, animations)
    }
}
