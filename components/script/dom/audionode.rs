/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::audioparam::AudioParam;
use dom::baseaudiocontext::BaseAudioContext;
use dom::bindings::codegen::Bindings::AudioNodeBinding::{AudioNodeMethods, AudioNodeOptions};
use dom::bindings::codegen::Bindings::AudioNodeBinding::{ChannelCountMode, ChannelInterpretation};
use dom::bindings::codegen::InheritTypes::{AudioNodeTypeId, EventTargetTypeId};
use dom::bindings::error::{Error, ErrorResult, Fallible};
use dom::bindings::inheritance::Castable;
use dom::bindings::root::{Dom, DomRoot};
use dom::eventtarget::EventTarget;
use dom_struct::dom_struct;
use servo_media::audio::graph::NodeId;
use servo_media::audio::node::{AudioNodeMessage, AudioNodeInit};
use servo_media::audio::node::ChannelCountMode as ServoMediaChannelCountMode;
use servo_media::audio::node::ChannelInterpretation as ServoMediaChannelInterpretation;
use std::cell::Cell;
use typeholder::TypeHolderTrait;

// 32 is the minimum required by the spec for createBuffer() and the deprecated
// createScriptProcessor() and matches what is used by Blink and Gecko.
// The limit protects against large memory allocations.
pub const MAX_CHANNEL_COUNT: u32 = 32;

#[dom_struct]
pub struct AudioNode<TH: TypeHolderTrait> {
    eventtarget: EventTarget<TH>,
    #[ignore_malloc_size_of = "servo_media"]
    node_id: NodeId,
    context: Dom<BaseAudioContext<TH>>,
    number_of_inputs: u32,
    number_of_outputs: u32,
    channel_count: Cell<u32>,
    channel_count_mode: Cell<ChannelCountMode>,
    channel_interpretation: Cell<ChannelInterpretation>,
}

impl<TH: TypeHolderTrait> AudioNode<TH> {
    pub fn new_inherited(
        node_type: AudioNodeInit,
        node_id: Option<NodeId>,
        context: &BaseAudioContext<TH>,
        options: &AudioNodeOptions,
        number_of_inputs: u32,
        number_of_outputs: u32,
    ) -> AudioNode<TH> {
        let node_id =
            node_id.unwrap_or_else(|| context.audio_context_impl().create_node(node_type));
        AudioNode {
            eventtarget: EventTarget::new_inherited(),
            node_id,
            context: Dom::from_ref(context),
            number_of_inputs,
            number_of_outputs,
            channel_count: Cell::new(options.channelCount.unwrap_or(2)),
            channel_count_mode: Cell::new(options.channelCountMode.unwrap_or_default()),
            channel_interpretation: Cell::new(options.channelInterpretation.unwrap_or_default()),
        }
    }

    pub fn message(&self, message: AudioNodeMessage) {
        self.context
            .audio_context_impl()
            .message_node(self.node_id, message);
    }

    pub fn node_id(&self) -> NodeId {
        self.node_id
    }
}

impl<TH: TypeHolderTrait> AudioNodeMethods<TH> for AudioNode<TH> {
    // https://webaudio.github.io/web-audio-api/#dom-audionode-connect
    fn Connect(
        &self,
        destination: &AudioNode<TH>,
        output: u32,
        input: u32,
    ) -> Fallible<DomRoot<AudioNode<TH>>> {
        if self.context != destination.context {
            return Err(Error::InvalidAccess);
        }

        if output >= self.NumberOfOutputs() || input >= destination.NumberOfInputs() {
            return Err(Error::IndexSize);
        }

        // servo-media takes care of ignoring duplicated connections.

        self.context.audio_context_impl().connect_ports(
            self.node_id().output(output),
            destination.node_id().input(input),
        );

        Ok(DomRoot::from_ref(destination))
    }

    // https://webaudio.github.io/web-audio-api/#dom-audionode-connect-destinationparam-output
    fn Connect_(&self, dest: &AudioParam<TH>, output: u32) -> Fallible<()> {
        if self.context != dest.context() {
            return Err(Error::InvalidAccess);
        }

        if output >= self.NumberOfOutputs() {
            return Err(Error::IndexSize);
        }

        // servo-media takes care of ignoring duplicated connections.

        self.context.audio_context_impl().connect_ports(
            self.node_id().output(output),
            dest.node_id().param(dest.param_type()),
        );

        Ok(())
    }

    // https://webaudio.github.io/web-audio-api/#dom-audionode-disconnect
    fn Disconnect(&self) -> ErrorResult {
        self.context
            .audio_context_impl()
            .disconnect_all_from(self.node_id());
        Ok(())
    }

    // https://webaudio.github.io/web-audio-api/#dom-audionode-disconnect-output
    fn Disconnect_(&self, out: u32) -> ErrorResult {
        self.context
            .audio_context_impl()
            .disconnect_output(self.node_id().output(out));
        Ok(())
    }

    // https://webaudio.github.io/web-audio-api/#dom-audionode-disconnect-destinationnode
    fn Disconnect__(&self, to: &AudioNode<TH>) -> ErrorResult {
        self.context
            .audio_context_impl()
            .disconnect_between(self.node_id(), to.node_id());
        Ok(())
    }

    // https://webaudio.github.io/web-audio-api/#dom-audionode-disconnect-destinationnode-output
    fn Disconnect___(&self, to: &AudioNode<TH>, out: u32) -> ErrorResult {
        self.context
            .audio_context_impl()
            .disconnect_output_between(self.node_id().output(out), to.node_id());
        Ok(())
    }

    // https://webaudio.github.io/web-audio-api/#dom-audionode-disconnect-destinationnode-output-input
    fn Disconnect____(&self, to: &AudioNode<TH>, out: u32, inp: u32) -> ErrorResult {
        self.context
            .audio_context_impl()
            .disconnect_output_between_to(self.node_id().output(out), to.node_id().input(inp));
        Ok(())
    }

    // https://webaudio.github.io/web-audio-api/#dom-audionode-disconnect
    fn Disconnect_____(&self, param: &AudioParam<TH>) -> ErrorResult {
        self.context
            .audio_context_impl()
            .disconnect_to(self.node_id(),
                           param.node_id().param(param.param_type()));
        Ok(())
    }

    // https://webaudio.github.io/web-audio-api/#dom-audionode-disconnect
    fn Disconnect______(&self, param: &AudioParam<TH>, out: u32) -> ErrorResult {
        self.context
            .audio_context_impl()
            .disconnect_output_between_to(self.node_id().output(out),
                                          param.node_id().param(param.param_type()));
        Ok(())
    }

    // https://webaudio.github.io/web-audio-api/#dom-audionode-context
    fn Context(&self) -> DomRoot<BaseAudioContext<TH>> {
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

    // https://webaudio.github.io/web-audio-api/#dom-audionode-channelcount
    fn SetChannelCount(&self, value: u32) -> ErrorResult {
        match self.upcast::<EventTarget<TH>>().type_id() {
            EventTargetTypeId::AudioNode(AudioNodeTypeId::AudioDestinationNode) => {
                if self.context.is_offline() {
                    return Err(Error::InvalidState);
                } else if value < 1 || value > MAX_CHANNEL_COUNT {
                    return Err(Error::IndexSize);
                }
            },
            // XXX We do not support any of the other AudioNodes with
            // constraints yet. Add more cases here as we add support
            // for new AudioNodes.
            _ => (),
        };

        if value == 0 || value > MAX_CHANNEL_COUNT {
            return Err(Error::NotSupported);
        }

        self.channel_count.set(value);
        self.message(AudioNodeMessage::SetChannelCount(value as u8));
        Ok(())
    }

    // https://webaudio.github.io/web-audio-api/#dom-audionode-channelcountmode
    fn ChannelCountMode(&self) -> ChannelCountMode {
        self.channel_count_mode.get()
    }

    // https://webaudio.github.io/web-audio-api/#dom-audionode-channelcountmode
    fn SetChannelCountMode(&self, value: ChannelCountMode) -> ErrorResult {
        // Channel count mode has no effect for nodes with no inputs.
        if self.number_of_inputs == 0 {
            return Ok(());
        }

        match self.upcast::<EventTarget<TH>>().type_id() {
            EventTargetTypeId::AudioNode(AudioNodeTypeId::AudioDestinationNode) => {
                if self.context.is_offline() {
                    return Err(Error::InvalidState);
                }
            },
            // XXX We do not support any of the other AudioNodes with
            // constraints yet. Add more cases here as we add support
            // for new AudioNodes.
            _ => (),
        };

        self.channel_count_mode.set(value);
        self.message(AudioNodeMessage::SetChannelMode(value.into()));
        Ok(())
    }

    // https://webaudio.github.io/web-audio-api/#dom-audionode-channelinterpretation
    fn ChannelInterpretation(&self) -> ChannelInterpretation {
        self.channel_interpretation.get()
    }

    // https://webaudio.github.io/web-audio-api/#dom-audionode-channelinterpretation
    fn SetChannelInterpretation(&self, value: ChannelInterpretation) {
        // Channel interpretation mode has no effect for nodes with no inputs.
        if self.number_of_inputs == 0 {
            return;
        }

        self.channel_interpretation.set(value);
        self.message(AudioNodeMessage::SetChannelInterpretation(value.into()));
    }
}

impl From<ChannelCountMode> for ServoMediaChannelCountMode {
    fn from(mode: ChannelCountMode) -> Self {
        match mode {
            ChannelCountMode::Max => ServoMediaChannelCountMode::Max,
            ChannelCountMode::Clamped_max => ServoMediaChannelCountMode::ClampedMax,
            ChannelCountMode::Explicit => ServoMediaChannelCountMode::Explicit,
        }
    }
}

impl From<ChannelInterpretation> for ServoMediaChannelInterpretation {
    fn from(interpretation: ChannelInterpretation) -> Self {
        match interpretation {
            ChannelInterpretation::Discrete => ServoMediaChannelInterpretation::Discrete,
            ChannelInterpretation::Speakers => ServoMediaChannelInterpretation::Speakers,
        }
    }
}
