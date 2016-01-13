/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS transitions and animations.

use flow::{self, Flow};
use gfx::display_list::OpaqueNode;
use incremental::RestyleDamage;
use msg::constellation_msg::{ConstellationChan, PipelineId};
use script_traits::{AnimationState, LayoutMsg as ConstellationMsg};
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::sync::mpsc::Receiver;
use style::animation::{Animation, update_style_for_animation};
use time;

/// Processes any new animations that were discovered after style recalculation.
/// Also expire any old animations that have completed, inserting them into `expired_animations`.
pub fn update_animation_state(constellation_chan: &ConstellationChan<ConstellationMsg>,
                              running_animations: &mut HashMap<OpaqueNode, Vec<Animation>>,
                              expired_animations: &mut HashMap<OpaqueNode, Vec<Animation>>,
                              new_animations_receiver: &Receiver<Animation>,
                              pipeline_id: PipelineId) {
    let mut new_running_animations = Vec::new();
    while let Ok(animation) = new_animations_receiver.try_recv() {
        new_running_animations.push(animation)
    }

    if running_animations.is_empty() && new_running_animations.is_empty() {
        // Nothing to do. Return early so we don't flood the compositor with
        // `ChangeRunningAnimationsState` messages.
        return
    }

    // Expire old running animations.
    let now = time::precise_time_s();
    let mut keys_to_remove = Vec::new();
    for (key, running_animations) in running_animations.iter_mut() {
        let mut animations_still_running = vec![];
        for running_animation in running_animations.drain(..) {
            if now < running_animation.end_time {
                animations_still_running.push(running_animation);
                continue
            }
            match expired_animations.entry(*key) {
                Entry::Vacant(entry) => {
                    entry.insert(vec![running_animation]);
                }
                Entry::Occupied(mut entry) => entry.get_mut().push(running_animation),
            }
        }
        if animations_still_running.len() == 0 {
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
        match running_animations.entry(new_running_animation.node) {
            Entry::Vacant(entry) => {
                entry.insert(vec![new_running_animation]);
            }
            Entry::Occupied(mut entry) => entry.get_mut().push(new_running_animation),
        }
    }

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
        if let Some(ref animations) = animations.get(&fragment.node) {
            for animation in *animations {
                update_style_for_animation(animation, &mut fragment.style, Some(&mut damage));
            }
        }
    });

    let base = flow::mut_base(flow);
    base.restyle_damage.insert(damage);
    for kid in base.children.iter_mut() {
        recalc_style_for_animations(kid, animations)
    }
}
