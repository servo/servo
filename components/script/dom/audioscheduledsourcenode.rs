/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use servo_media::audio::node::{
    AudioNodeInit, AudioNodeMessage, AudioScheduledSourceNodeMessage, OnEndedCallback,
};

use crate::dom::audionode::{AudioNode, UnwrappedAudioNodeOptions};
use crate::dom::baseaudiocontext::BaseAudioContext;
use crate::dom::bindings::codegen::Bindings::AudioScheduledSourceNodeBinding::AudioScheduledSourceNodeMethods;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomGlobal;

#[dom_struct]
pub(crate) struct AudioScheduledSourceNode {
    node: AudioNode,
    has_start: Cell<bool>,
    has_stop: Cell<bool>,
}

impl AudioScheduledSourceNode {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new_inherited(
        node_type: AudioNodeInit,
        context: &BaseAudioContext,
        options: UnwrappedAudioNodeOptions,
        number_of_inputs: u32,
        number_of_outputs: u32,
    ) -> Fallible<AudioScheduledSourceNode> {
        Ok(AudioScheduledSourceNode {
            node: AudioNode::new_inherited(
                node_type,
                context,
                options,
                number_of_inputs,
                number_of_outputs,
            )?,
            has_start: Cell::new(false),
            has_stop: Cell::new(false),
        })
    }

    pub(crate) fn node(&self) -> &AudioNode {
        &self.node
    }

    pub(crate) fn has_start(&self) -> bool {
        self.has_start.get()
    }
}

impl AudioScheduledSourceNodeMethods<crate::DomTypeHolder> for AudioScheduledSourceNode {
    // https://webaudio.github.io/web-audio-api/#dom-audioscheduledsourcenode-onended
    event_handler!(ended, GetOnended, SetOnended);

    // https://webaudio.github.io/web-audio-api/#dom-audioscheduledsourcenode-start
    fn Start(&self, when: Finite<f64>) -> Fallible<()> {
        if *when < 0. {
            return Err(Error::Range("'when' must be a positive value".to_owned()));
        }

        if self.has_start.get() || self.has_stop.get() {
            return Err(Error::InvalidState);
        }

        let this = Trusted::new(self);
        let task_source = self
            .global()
            .task_manager()
            .dom_manipulation_task_source()
            .to_sendable();
        let callback = OnEndedCallback::new(move || {
            task_source.queue(task!(ended: move || {
                let this = this.root();
                this.global().task_manager().dom_manipulation_task_source().queue_simple_event(
                    this.upcast(),
                    atom!("ended"),
                    );
            }));
        });

        self.node()
            .message(AudioNodeMessage::AudioScheduledSourceNode(
                AudioScheduledSourceNodeMessage::RegisterOnEndedCallback(callback),
            ));

        self.has_start.set(true);
        self.node
            .message(AudioNodeMessage::AudioScheduledSourceNode(
                AudioScheduledSourceNodeMessage::Start(*when),
            ));
        Ok(())
    }

    // https://webaudio.github.io/web-audio-api/#dom-audioscheduledsourcenode-stop
    fn Stop(&self, when: Finite<f64>) -> Fallible<()> {
        if *when < 0. {
            return Err(Error::Range("'when' must be a positive value".to_owned()));
        }

        if !self.has_start.get() {
            return Err(Error::InvalidState);
        }
        self.has_stop.set(true);
        self.node
            .message(AudioNodeMessage::AudioScheduledSourceNode(
                AudioScheduledSourceNodeMessage::Stop(*when),
            ));
        Ok(())
    }
}
