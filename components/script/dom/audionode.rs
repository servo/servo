/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use servo_media::audio::graph::NodeId;
use servo_media::audio::node::{
    AudioNodeInit, AudioNodeMessage, ChannelCountMode as ServoMediaChannelCountMode, ChannelInfo,
    ChannelInterpretation as ServoMediaChannelInterpretation,
};

use crate::dom::audioparam::AudioParam;
use crate::dom::baseaudiocontext::BaseAudioContext;
use crate::dom::bindings::codegen::Bindings::AudioNodeBinding::{
    AudioNodeMethods, AudioNodeOptions, ChannelCountMode, ChannelInterpretation,
};
use crate::dom::bindings::codegen::InheritTypes::{
    AudioNodeTypeId, AudioScheduledSourceNodeTypeId, EventTargetTypeId,
};
use crate::dom::bindings::error::{Error, ErrorResult, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::eventtarget::EventTarget;

// 32 is the minimum required by the spec for createBuffer() and the deprecated
// createScriptProcessor() and matches what is used by Blink and Gecko.
// The limit protects against large memory allocations.
pub const MAX_CHANNEL_COUNT: u32 = 32;

#[dom_struct]
pub struct AudioNode {
    eventtarget: EventTarget,
    #[ignore_malloc_size_of = "servo_media"]
    #[no_trace]
    node_id: NodeId,
    context: Dom<BaseAudioContext>,
    number_of_inputs: u32,
    number_of_outputs: u32,
    channel_count: Cell<u32>,
    channel_count_mode: Cell<ChannelCountMode>,
    channel_interpretation: Cell<ChannelInterpretation>,
}

impl AudioNode {
    pub fn new_inherited(
        node_type: AudioNodeInit,
        context: &BaseAudioContext,
        options: UnwrappedAudioNodeOptions,
        number_of_inputs: u32,
        number_of_outputs: u32,
    ) -> Fallible<AudioNode> {
        if options.count == 0 || options.count > MAX_CHANNEL_COUNT {
            return Err(Error::NotSupported);
        }
        let ch = ChannelInfo {
            count: options.count as u8,
            mode: options.mode.into(),
            interpretation: options.interpretation.into(),
        };
        let node_id = context
            .audio_context_impl()
            .lock()
            .unwrap()
            .create_node(node_type, ch);
        Ok(AudioNode::new_inherited_for_id(
            node_id,
            context,
            options,
            number_of_inputs,
            number_of_outputs,
        ))
    }

    pub fn new_inherited_for_id(
        node_id: NodeId,
        context: &BaseAudioContext,
        options: UnwrappedAudioNodeOptions,
        number_of_inputs: u32,
        number_of_outputs: u32,
    ) -> AudioNode {
        AudioNode {
            eventtarget: EventTarget::new_inherited(),
            node_id,
            context: Dom::from_ref(context),
            number_of_inputs,
            number_of_outputs,
            channel_count: Cell::new(options.count),
            channel_count_mode: Cell::new(options.mode),
            channel_interpretation: Cell::new(options.interpretation),
        }
    }

    pub fn message(&self, message: AudioNodeMessage) {
        self.context
            .audio_context_impl()
            .lock()
            .unwrap()
            .message_node(self.node_id, message);
    }

    pub fn node_id(&self) -> NodeId {
        self.node_id
    }
}

impl AudioNodeMethods for AudioNode {
    // https://webaudio.github.io/web-audio-api/#dom-audionode-connect
    fn Connect(
        &self,
        destination: &AudioNode,
        output: u32,
        input: u32,
    ) -> Fallible<DomRoot<AudioNode>> {
        if self.context != destination.context {
            return Err(Error::InvalidAccess);
        }

        if output >= self.NumberOfOutputs() || input >= destination.NumberOfInputs() {
            return Err(Error::IndexSize);
        }

        // servo-media takes care of ignoring duplicated connections.

        self.context
            .audio_context_impl()
            .lock()
            .unwrap()
            .connect_ports(
                self.node_id().output(output),
                destination.node_id().input(input),
            );

        Ok(DomRoot::from_ref(destination))
    }

    // https://webaudio.github.io/web-audio-api/#dom-audionode-connect-destinationparam-output
    fn Connect_(&self, dest: &AudioParam, output: u32) -> Fallible<()> {
        if self.context != dest.context() {
            return Err(Error::InvalidAccess);
        }

        if output >= self.NumberOfOutputs() {
            return Err(Error::IndexSize);
        }

        // servo-media takes care of ignoring duplicated connections.

        self.context
            .audio_context_impl()
            .lock()
            .unwrap()
            .connect_ports(
                self.node_id().output(output),
                dest.node_id().param(dest.param_type()),
            );

        Ok(())
    }

    // https://webaudio.github.io/web-audio-api/#dom-audionode-disconnect
    fn Disconnect(&self) -> ErrorResult {
        self.context
            .audio_context_impl()
            .lock()
            .unwrap()
            .disconnect_all_from(self.node_id());
        Ok(())
    }

    // https://webaudio.github.io/web-audio-api/#dom-audionode-disconnect-output
    fn Disconnect_(&self, out: u32) -> ErrorResult {
        self.context
            .audio_context_impl()
            .lock()
            .unwrap()
            .disconnect_output(self.node_id().output(out));
        Ok(())
    }

    // https://webaudio.github.io/web-audio-api/#dom-audionode-disconnect-destinationnode
    fn Disconnect__(&self, to: &AudioNode) -> ErrorResult {
        self.context
            .audio_context_impl()
            .lock()
            .unwrap()
            .disconnect_between(self.node_id(), to.node_id());
        Ok(())
    }

    // https://webaudio.github.io/web-audio-api/#dom-audionode-disconnect-destinationnode-output
    fn Disconnect___(&self, to: &AudioNode, out: u32) -> ErrorResult {
        self.context
            .audio_context_impl()
            .lock()
            .unwrap()
            .disconnect_output_between(self.node_id().output(out), to.node_id());
        Ok(())
    }

    // https://webaudio.github.io/web-audio-api/#dom-audionode-disconnect-destinationnode-output-input
    fn Disconnect____(&self, to: &AudioNode, out: u32, inp: u32) -> ErrorResult {
        self.context
            .audio_context_impl()
            .lock()
            .unwrap()
            .disconnect_output_between_to(self.node_id().output(out), to.node_id().input(inp));
        Ok(())
    }

    // https://webaudio.github.io/web-audio-api/#dom-audionode-disconnect
    fn Disconnect_____(&self, param: &AudioParam) -> ErrorResult {
        self.context
            .audio_context_impl()
            .lock()
            .unwrap()
            .disconnect_to(self.node_id(), param.node_id().param(param.param_type()));
        Ok(())
    }

    // https://webaudio.github.io/web-audio-api/#dom-audionode-disconnect
    fn Disconnect______(&self, param: &AudioParam, out: u32) -> ErrorResult {
        self.context
            .audio_context_impl()
            .lock()
            .unwrap()
            .disconnect_output_between_to(
                self.node_id().output(out),
                param.node_id().param(param.param_type()),
            );
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

    // https://webaudio.github.io/web-audio-api/#dom-audionode-channelcount
    fn SetChannelCount(&self, value: u32) -> ErrorResult {
        match self.upcast::<EventTarget>().type_id() {
            EventTargetTypeId::AudioNode(AudioNodeTypeId::AudioDestinationNode) => {
                if self.context.is_offline() {
                    return Err(Error::InvalidState);
                } else if !(1..=MAX_CHANNEL_COUNT).contains(&value) {
                    return Err(Error::IndexSize);
                }
            },
            EventTargetTypeId::AudioNode(AudioNodeTypeId::PannerNode) => {
                if value > 2 {
                    return Err(Error::NotSupported);
                }
            },
            EventTargetTypeId::AudioNode(AudioNodeTypeId::AudioScheduledSourceNode(
                AudioScheduledSourceNodeTypeId::StereoPannerNode,
            )) => {
                if value > 2 {
                    return Err(Error::NotSupported);
                }
            },
            EventTargetTypeId::AudioNode(AudioNodeTypeId::ChannelMergerNode) => {
                return Err(Error::InvalidState);
            },
            EventTargetTypeId::AudioNode(AudioNodeTypeId::ChannelSplitterNode) => {
                return Err(Error::InvalidState);
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

        match self.upcast::<EventTarget>().type_id() {
            EventTargetTypeId::AudioNode(AudioNodeTypeId::AudioDestinationNode) => {
                if self.context.is_offline() {
                    return Err(Error::InvalidState);
                }
            },
            EventTargetTypeId::AudioNode(AudioNodeTypeId::PannerNode) => {
                if value == ChannelCountMode::Max {
                    return Err(Error::NotSupported);
                }
            },
            EventTargetTypeId::AudioNode(AudioNodeTypeId::AudioScheduledSourceNode(
                AudioScheduledSourceNodeTypeId::StereoPannerNode,
            )) => {
                if value == ChannelCountMode::Max {
                    return Err(Error::NotSupported);
                }
            },
            EventTargetTypeId::AudioNode(AudioNodeTypeId::ChannelMergerNode) => {
                return Err(Error::InvalidState);
            },
            EventTargetTypeId::AudioNode(AudioNodeTypeId::ChannelSplitterNode) => {
                return Err(Error::InvalidState);
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
    fn SetChannelInterpretation(&self, value: ChannelInterpretation) -> ErrorResult {
        // Channel interpretation mode has no effect for nodes with no inputs.
        if self.number_of_inputs == 0 {
            return Ok(());
        }

        if let EventTargetTypeId::AudioNode(AudioNodeTypeId::ChannelSplitterNode) =
            self.upcast::<EventTarget>().type_id()
        {
            return Err(Error::InvalidState);
        };

        self.channel_interpretation.set(value);
        self.message(AudioNodeMessage::SetChannelInterpretation(value.into()));
        Ok(())
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

impl AudioNodeOptions {
    pub fn unwrap_or(
        &self,
        count: u32,
        mode: ChannelCountMode,
        interpretation: ChannelInterpretation,
    ) -> UnwrappedAudioNodeOptions {
        UnwrappedAudioNodeOptions {
            count: self.channelCount.unwrap_or(count),
            mode: self.channelCountMode.unwrap_or(mode),
            interpretation: self.channelInterpretation.unwrap_or(interpretation),
        }
    }
}

/// Each node has a set of defaults, so this lets us work with them
/// easily without having to deal with the Options
pub struct UnwrappedAudioNodeOptions {
    pub count: u32,
    pub mode: ChannelCountMode,
    pub interpretation: ChannelInterpretation,
}

impl Default for UnwrappedAudioNodeOptions {
    fn default() -> Self {
        UnwrappedAudioNodeOptions {
            count: 2,
            mode: ChannelCountMode::Max,
            interpretation: ChannelInterpretation::Speakers,
        }
    }
}
