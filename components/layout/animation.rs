/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS transitions and animations.

use flow::{self, Flow};
use incremental::{self, RestyleDamage};

use clock_ticks;
use gfx::display_list::OpaqueNode;
use layout_task::{LayoutTask, LayoutTaskData};
use msg::constellation_msg::{AnimationState, Msg, PipelineId};
use script::layout_interface::Animation;
use script_traits::{ConstellationControlMsg, ScriptControlChan};
use std::mem;
use std::sync::Arc;
use std::sync::mpsc::Sender;
use style::animation::{GetMod, PropertyAnimation};
use style::properties::ComputedValues;

/// Inserts transitions into the queue of running animations as applicable for the given style
/// difference. This is called from the layout worker threads.
pub fn start_transitions_if_applicable(new_animations_sender: &Sender<Animation>,
                                       node: OpaqueNode,
                                       old_style: &ComputedValues,
                                       new_style: &mut ComputedValues) {
    for i in 0..new_style.get_animation().transition_property.0.len() {
        // Create any property animations, if applicable.
        let property_animations = PropertyAnimation::from_transition(i, old_style, new_style);
        for property_animation in property_animations.into_iter() {
            // Set the property to the initial value.
            property_animation.update(new_style, 0.0);

            // Kick off the animation.
            let now = clock_ticks::precise_time_s() as f32;
            let animation_style = new_style.get_animation();
            let start_time = now + animation_style.transition_delay.0.get_mod(i).seconds();
            new_animations_sender.send(Animation {
                node: node.id(),
                property_animation: property_animation,
                start_time: start_time,
                end_time: start_time +
                    animation_style.transition_duration.0.get_mod(i).seconds(),
            }).unwrap()
        }
    }
}

/// Processes any new animations that were discovered after style recalculation.
pub fn process_new_animations(rw_data: &mut LayoutTaskData, pipeline_id: PipelineId) {
    while let Ok(animation) = rw_data.new_animations_receiver.try_recv() {
        rw_data.running_animations.push(animation)
    }

    let animation_state;
    if rw_data.running_animations.is_empty() {
        animation_state = AnimationState::NoAnimationsPresent;
    } else {
        animation_state = AnimationState::AnimationsPresent;
    }

    rw_data.constellation_chan
           .0
           .send(Msg::ChangeRunningAnimationsState(pipeline_id, animation_state))
           .unwrap();

}

/// Recalculates style for an animation. This does *not* run with the DOM lock held.
pub fn recalc_style_for_animation(flow: &mut Flow, animation: &Animation) {
    let mut damage = RestyleDamage::empty();
    flow.mutate_fragments(&mut |fragment| {
        if fragment.node.id() != animation.node {
            return
        }

        let now = clock_ticks::precise_time_s() as f32;
        let mut progress = (now - animation.start_time) / animation.duration();
        if progress > 1.0 {
            progress = 1.0
        }
        if progress <= 0.0 {
            return
        }

        let mut new_style = fragment.style.clone();
        animation.property_animation.update(&mut *Arc::make_unique(&mut new_style), progress);
        damage.insert(incremental::compute_damage(&Some(fragment.style.clone()), &new_style));
        fragment.style = new_style
    });

    let base = flow::mut_base(flow);
    base.restyle_damage.insert(damage);
    for kid in base.children.iter_mut() {
        recalc_style_for_animation(kid, animation)
    }
}

/// Handles animation updates.
pub fn tick_all_animations(layout_task: &LayoutTask, rw_data: &mut LayoutTaskData) {
    let running_animations = mem::replace(&mut rw_data.running_animations, Vec::new());
    let now = clock_ticks::precise_time_s() as f32;
    for running_animation in running_animations.into_iter() {
        layout_task.tick_animation(&running_animation, rw_data);

        if now < running_animation.end_time {
            // Keep running the animation if it hasn't expired.
            rw_data.running_animations.push(running_animation)
        }
    }

    let ScriptControlChan(ref chan) = layout_task.script_chan;
    chan.send(ConstellationControlMsg::TickAllAnimations(layout_task.id)).unwrap();
}

