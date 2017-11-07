/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
use dom::audionode::AudioNode;
use dom::baseaudiocontext::BaseAudioContext;
use dom::bindings::codegen::Bindings::AudioScheduledSourceNodeBinding::AudioScheduledSourceNodeMethods;
use dom::bindings::codegen::Bindings::AudioNodeBinding::AudioNodeOptions;
use dom::bindings::num::Finite;
use dom_struct::dom_struct;

#[dom_struct]
pub struct AudioScheduledSourceNode {
    node: AudioNode,
}

impl AudioScheduledSourceNode {
    pub fn new_inherited(context: &BaseAudioContext,
                         options: &AudioNodeOptions,
                         number_of_inputs: u32,
                         number_of_outputs: u32) -> AudioScheduledSourceNode {
        AudioScheduledSourceNode {
            node: AudioNode::new_inherited(context, options, number_of_inputs, number_of_outputs),
        }
    }
}

impl AudioScheduledSourceNodeMethods for AudioScheduledSourceNode {
    // https://webaudio.github.io/web-audio-api/#dom-audioscheduledsourcenode-onended
    event_handler!(ended, GetOnended, SetOnended);

    // https://webaudio.github.io/web-audio-api/#dom-audioscheduledsourcenode-start
    fn Start(&self, _when: Finite<f64>) {
    }

    // https://webaudio.github.io/web-audio-api/#dom-audioscheduledsourcenode-stop
    fn Stop(&self, _when: Finite<f64>) {
    }
}
