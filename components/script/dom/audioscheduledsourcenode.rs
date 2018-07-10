/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
use dom::audionode::AudioNode;
use dom::baseaudiocontext::BaseAudioContext;
use dom::bindings::codegen::Bindings::AudioScheduledSourceNodeBinding::AudioScheduledSourceNodeMethods;
use dom::bindings::codegen::Bindings::AudioNodeBinding::AudioNodeOptions;
use dom::bindings::error::{Error, Fallible};
use dom::bindings::num::Finite;
use dom_struct::dom_struct;
use servo_media::audio::node::{AudioNodeMessage, AudioNodeInit, AudioScheduledSourceNodeMessage};
use std::cell::Cell;

#[dom_struct]
pub struct AudioScheduledSourceNode {
    node: AudioNode,
    started: Cell<bool>,
    stopped: Cell<bool>,
}

impl AudioScheduledSourceNode {
    pub fn new_inherited(node_type: AudioNodeInit,
                         context: &BaseAudioContext,
                         options: &AudioNodeOptions,
                         number_of_inputs: u32,
                         number_of_outputs: u32) -> AudioScheduledSourceNode {
        AudioScheduledSourceNode {
            node: AudioNode::new_inherited(node_type, None /* node_id */,
                                           context, options, number_of_inputs, number_of_outputs),
                                           started: Cell::new(false),
                                           stopped: Cell::new(false),
        }
    }

    pub fn node(&self) -> &AudioNode {
        &self.node
    }

    pub fn started(&self) -> bool {
        self.started.get()
    }
}

impl AudioScheduledSourceNodeMethods for AudioScheduledSourceNode {
    // https://webaudio.github.io/web-audio-api/#dom-audioscheduledsourcenode-onended
    event_handler!(ended, GetOnended, SetOnended);

    // https://webaudio.github.io/web-audio-api/#dom-audioscheduledsourcenode-start
    fn Start(&self, when: Finite<f64>) -> Fallible<()> {
        if self.started.get() || self.stopped.get() {
            return Err(Error::InvalidState);
        }
        self.started.set(true);
        self.node.message(
            AudioNodeMessage::AudioScheduledSourceNode(AudioScheduledSourceNodeMessage::Start(*when))
            );
        Ok(())
    }

    // https://webaudio.github.io/web-audio-api/#dom-audioscheduledsourcenode-stop
    fn Stop(&self, when: Finite<f64>) -> Fallible<()> {
        if !self.started.get() {
            return Err(Error::InvalidState);
        }
        self.stopped.set(true);
        self.node.message(
            AudioNodeMessage::AudioScheduledSourceNode(AudioScheduledSourceNodeMessage::Stop(*when))
            );
        Ok(())
    }
}
