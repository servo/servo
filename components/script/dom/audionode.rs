/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::baseaudiocontext::BaseAudioContext;
use dom::bindings::codegen::Bindings::AudioNodeBinding::{AudioNodeMethods, AudioNodeOptions};
use dom::bindings::codegen::Bindings::AudioNodeBinding::{ChannelCountMode, ChannelInterpretation};
use dom::bindings::error::{Error, ErrorResult, Fallible};
use dom::bindings::reflector::Reflector;
use dom::bindings::root::DomRoot;
use dom::audioparam::AudioParam;
use dom_struct::dom_struct;
use servo_media::audio::graph::NodeId;
use servo_media::audio::node::{AudioNodeMessage, AudioNodeType};
use std::cell::Cell;

// 32 is the minimum required by the spec for createBuffer() and the deprecated
// createScriptProcessor() and matches what is used by Blink and Gecko.
// The limit protects against large memory allocations.
pub static MAX_CHANNEL_COUNT: u32 = 32;

#[dom_struct]
pub struct AudioNode {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "servo_media"]
    node_id: NodeId,
    context: DomRoot<BaseAudioContext>,
    number_of_inputs: u32,
    number_of_outputs: u32,
    channel_count: Cell<u32>,
    channel_count_mode: Cell<ChannelCountMode>,
    channel_interpretation: Cell<ChannelInterpretation>,
}

impl AudioNode {
    pub fn new_inherited(node_type: AudioNodeType,
                         node_id: Option<NodeId>,
                         context: &BaseAudioContext,
                         options: &AudioNodeOptions,
                         number_of_inputs: u32,
                         number_of_outputs: u32) -> AudioNode {
        let node_id = node_id.unwrap_or_else(|| {
            context.audio_context_impl().create_node(node_type)
        });
        AudioNode {
            reflector_: Reflector::new(),
            node_id,
            context: DomRoot::from_ref(context),
            number_of_inputs,
            number_of_outputs,
            channel_count: Cell::new(options.channelCount.unwrap_or(2)),
            channel_count_mode: Cell::new(options.channelCountMode.unwrap_or_default()),
            channel_interpretation: Cell::new(options.channelInterpretation.unwrap_or_default()),
        }
    }

    pub fn message(&self, message: AudioNodeMessage) {
        self.context.audio_context_impl().message_node(self.node_id, message);
    }

    pub fn node(&self) -> NodeId {
        self.node_id
    }
}

impl AudioNodeMethods for AudioNode {
    // https://webaudio.github.io/web-audio-api/#dom-audionode-connect
    fn Connect(&self,
               destination: &AudioNode,
               output: u32,
               input: u32) -> Fallible<DomRoot<AudioNode>> {
        if *(self.context) != *(destination.Context()) {
            //XXX return Err(Error::InvalidAccess);
        }

        if output >= self.NumberOfOutputs() ||
            input >= destination.NumberOfInputs() {
                return Err(Error::IndexSize);
            }

        // XXX Check previous connections.

        self.context.audio_context_impl().connect_ports(
            self.node().output(output), destination.node().input(input)
            );

        Ok(DomRoot::from_ref(destination))
    }

    fn Connect_(&self,
                _: &AudioParam,
                _: u32) -> Fallible<()> {
        // TODO
        Ok(())
    }

    // https://webaudio.github.io/web-audio-api/#dom-audionode-disconnect
    fn Disconnect(&self) -> ErrorResult {
        // TODO
        Ok(())
    }

    // https://webaudio.github.io/web-audio-api/#dom-audionode-disconnect
    fn Disconnect_(&self, _: u32) -> ErrorResult {
        // TODO
        Ok(())
    }

    // https://webaudio.github.io/web-audio-api/#dom-audionode-disconnect
    fn Disconnect__(&self, _: &AudioNode) -> ErrorResult {
        // TODO
        Ok(())
    }

    // https://webaudio.github.io/web-audio-api/#dom-audionode-disconnect
    fn Disconnect___(&self, _: &AudioNode, _: u32) -> ErrorResult{
        // TODO
        Ok(())
    }

    // https://webaudio.github.io/web-audio-api/#dom-audionode-disconnect
    fn Disconnect____(&self, _: &AudioNode, _: u32, _: u32) -> ErrorResult {
        // TODO
        Ok(())
    }

    // https://webaudio.github.io/web-audio-api/#dom-audionode-disconnect
    fn Disconnect_____(&self, _: &AudioParam) -> ErrorResult {
        // TODO
        Ok(())
    }

    // https://webaudio.github.io/web-audio-api/#dom-audionode-disconnect
    fn Disconnect______(&self, _: &AudioParam, _: u32) -> ErrorResult {
        // TODO
        Ok(())
    }

    // https://webaudio.github.io/web-audio-api/#dom-audionode-context
    fn Context(&self) -> DomRoot<BaseAudioContext> {
        DomRoot::from_ref(&self.context)
    }

    // https://webaudio.github.io/web-audio-api/#dom-audionode-numberofinputs
    fn NumberOfInputs(&self) -> u32 {
        self.number_of_inputs
    }

    // https://webaudio.github.io/web-audio-api/#dom-audionode-numberofoutputs
    fn NumberOfOutputs(&self) -> u32 {
        self.number_of_outputs
    }

    // https://webaudio.github.io/web-audio-api/#dom-audionode-channelcount
    fn ChannelCount(&self) -> u32 {
        self.channel_count.get()
    }

    fn SetChannelCount(&self, value: u32) -> ErrorResult {
        if value == 0 || value > MAX_CHANNEL_COUNT {
            return Err(Error::NotSupported);
        }
        self.channel_count.set(value);
        Ok(())
    }

    fn ChannelCountMode(&self) -> ChannelCountMode {
        self.channel_count_mode.get()
    }

    fn SetChannelCountMode(&self, value: ChannelCountMode) -> ErrorResult {
        self.channel_count_mode.set(value);
        Ok(())
    }

    fn ChannelInterpretation(&self) -> ChannelInterpretation {
        self.channel_interpretation.get()
    }

    fn SetChannelInterpretation(&self, value: ChannelInterpretation) {
        self.channel_interpretation.set(value);
    }
}
