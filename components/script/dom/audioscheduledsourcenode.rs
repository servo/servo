/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
use dom::audionode::AudioNode;
use dom::baseaudiocontext::BaseAudioContext;
use dom::bindings::codegen::Bindings::AudioNodeBinding::AudioNodeOptions;
use dom::bindings::codegen::Bindings::AudioScheduledSourceNodeBinding::AudioScheduledSourceNodeMethods;
use dom::bindings::error::{Error, Fallible};
use dom::bindings::inheritance::Castable;
use dom::bindings::num::Finite;
use dom::bindings::refcounted::Trusted;
use dom::bindings::reflector::DomObject;
use dom_struct::dom_struct;
use servo_media::audio::node::{AudioNodeMessage, AudioNodeInit, AudioScheduledSourceNodeMessage};
use servo_media::audio::node::OnEndedCallback;
use std::cell::Cell;
use task_source::{TaskSource, TaskSourceName};
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct AudioScheduledSourceNode<TH: TypeHolderTrait> {
    node: AudioNode<TH>,
    started: Cell<bool>,
    stopped: Cell<bool>,
}

impl<TH: TypeHolderTrait> AudioScheduledSourceNode<TH> {
    pub fn new_inherited(
        node_type: AudioNodeInit,
        context: &BaseAudioContext<TH>,
        options: &AudioNodeOptions,
        number_of_inputs: u32,
        number_of_outputs: u32,
    ) -> AudioScheduledSourceNode<TH> {
        AudioScheduledSourceNode {
            node: AudioNode::new_inherited(
                node_type,
                None, /* node_id */
                context,
                options,
                number_of_inputs,
                number_of_outputs,
            ),
            started: Cell::new(false),
            stopped: Cell::new(false),
        }
    }

    pub fn node(&self) -> &AudioNode<TH> {
        &self.node
    }

    pub fn started(&self) -> bool {
        self.started.get()
    }
}

impl<TH: TypeHolderTrait> AudioScheduledSourceNodeMethods<TH> for AudioScheduledSourceNode<TH> {
    // https://webaudio.github.io/web-audio-api/#dom-audioscheduledsourcenode-onended
    event_handler!(ended, GetOnended, SetOnended);

    // https://webaudio.github.io/web-audio-api/#dom-audioscheduledsourcenode-start
    fn Start(&self, when: Finite<f64>) -> Fallible<()> {
        if self.started.get() || self.stopped.get() {
            return Err(Error::InvalidState);
        }

        let this = Trusted::new(self);
        let global = self.global();
        let window = global.as_window();
        let task_source = window.dom_manipulation_task_source();
        let canceller = window.task_canceller(TaskSourceName::DOMManipulation);
        let callback = OnEndedCallback::new(move || {
            let _ = task_source.queue_with_canceller(
                task!(ended: move || {
                    let this = this.root();
                    let global = this.global();
                    let window = global.as_window();
                    window.dom_manipulation_task_source().queue_simple_event(
                        this.upcast(),
                        atom!("ended"),
                        &window
                        );
                }),
                &canceller,
            );
        });

        self.node().message(
            AudioNodeMessage::AudioScheduledSourceNode(
                AudioScheduledSourceNodeMessage::RegisterOnEndedCallback(callback)));

        self.started.set(true);
        self.node
            .message(AudioNodeMessage::AudioScheduledSourceNode(
                AudioScheduledSourceNodeMessage::Start(*when),
            ));
        Ok(())
    }

    // https://webaudio.github.io/web-audio-api/#dom-audioscheduledsourcenode-stop
    fn Stop(&self, when: Finite<f64>) -> Fallible<()> {
        if !self.started.get() {
            return Err(Error::InvalidState);
        }
        self.stopped.set(true);
        self.node
            .message(AudioNodeMessage::AudioScheduledSourceNode(
                AudioScheduledSourceNodeMessage::Stop(*when),
            ));
        Ok(())
    }
}
