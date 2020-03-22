/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::audioparam::AudioParam;
use crate::dom::audioscheduledsourcenode::AudioScheduledSourceNode;
use crate::dom::baseaudiocontext::BaseAudioContext;
use crate::dom::bindings::codegen::Bindings::AudioNodeBinding::{
    ChannelCountMode, ChannelInterpretation,
};
use crate::dom::bindings::codegen::Bindings::AudioParamBinding::AutomationRate;
use crate::dom::bindings::codegen::Bindings::OscillatorNodeBinding::OscillatorNodeMethods;
use crate::dom::bindings::codegen::Bindings::OscillatorNodeBinding::{
    OscillatorOptions, OscillatorType,
};
use crate::dom::bindings::error::{Error, ErrorResult, Fallible};
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::window::Window;
use dom_struct::dom_struct;
use servo_media::audio::node::{AudioNodeInit, AudioNodeMessage};
use servo_media::audio::oscillator_node::OscillatorNodeMessage;
use servo_media::audio::oscillator_node::OscillatorNodeOptions as ServoMediaOscillatorOptions;
use servo_media::audio::oscillator_node::OscillatorType as ServoMediaOscillatorType;
use servo_media::audio::param::ParamType;
use std::cell::Cell;
use std::f32;

#[dom_struct]
pub struct OscillatorNode {
    source_node: AudioScheduledSourceNode,
    detune: Dom<AudioParam>,
    frequency: Dom<AudioParam>,
    oscillator_type: Cell<OscillatorType>,
}

impl OscillatorNode {
    #[allow(unrooted_must_root)]
    pub fn new_inherited(
        window: &Window,
        context: &BaseAudioContext,
        options: &OscillatorOptions,
    ) -> Fallible<OscillatorNode> {
        let node_options =
            options
                .parent
                .unwrap_or(2, ChannelCountMode::Max, ChannelInterpretation::Speakers);
        let source_node = AudioScheduledSourceNode::new_inherited(
            AudioNodeInit::OscillatorNode(options.into()),
            context,
            node_options,
            0, /* inputs */
            1, /* outputs */
        )?;
        let node_id = source_node.node().node_id();
        let frequency = AudioParam::new(
            window,
            context,
            node_id,
            ParamType::Frequency,
            AutomationRate::A_rate,
            440.,
            f32::MIN,
            f32::MAX,
        );
        let detune = AudioParam::new(
            window,
            context,
            node_id,
            ParamType::Detune,
            AutomationRate::A_rate,
            0.,
            -440. / 2.,
            440. / 2.,
        );
        Ok(OscillatorNode {
            source_node,
            oscillator_type: Cell::new(options.type_),
            frequency: Dom::from_ref(&frequency),
            detune: Dom::from_ref(&detune),
        })
    }

    #[allow(unrooted_must_root)]
    pub fn new(
        window: &Window,
        context: &BaseAudioContext,
        options: &OscillatorOptions,
    ) -> Fallible<DomRoot<OscillatorNode>> {
        let node = OscillatorNode::new_inherited(window, context, options)?;
        Ok(reflect_dom_object(Box::new(node), window))
    }

    #[allow(non_snake_case)]
    pub fn Constructor(
        window: &Window,
        context: &BaseAudioContext,
        options: &OscillatorOptions,
    ) -> Fallible<DomRoot<OscillatorNode>> {
        OscillatorNode::new(window, context, options)
    }
}

impl OscillatorNodeMethods for OscillatorNode {
    // https://webaudio.github.io/web-audio-api/#dom-oscillatornode-frequency
    fn Frequency(&self) -> DomRoot<AudioParam> {
        DomRoot::from_ref(&self.frequency)
    }

    // https://webaudio.github.io/web-audio-api/#dom-oscillatornode-detune
    fn Detune(&self) -> DomRoot<AudioParam> {
        DomRoot::from_ref(&self.detune)
    }

    // https://webaudio.github.io/web-audio-api/#dom-oscillatornode-type
    fn Type(&self) -> OscillatorType {
        self.oscillator_type.get()
    }

    // https://webaudio.github.io/web-audio-api/#dom-oscillatornode-type
    fn SetType(&self, type_: OscillatorType) -> ErrorResult {
        if type_ == OscillatorType::Custom {
            return Err(Error::InvalidState);
        }
        self.oscillator_type.set(type_);
        self.source_node
            .node()
            .message(AudioNodeMessage::OscillatorNode(
                OscillatorNodeMessage::SetOscillatorType(type_.into()),
            ));
        return Ok(());
    }
}

impl<'a> From<&'a OscillatorOptions> for ServoMediaOscillatorOptions {
    fn from(options: &'a OscillatorOptions) -> Self {
        Self {
            oscillator_type: options.type_.into(),
            freq: *options.frequency,
            detune: *options.detune,
            periodic_wave_options: None, // XXX
        }
    }
}

impl From<OscillatorType> for ServoMediaOscillatorType {
    fn from(oscillator_type: OscillatorType) -> Self {
        match oscillator_type {
            OscillatorType::Sine => ServoMediaOscillatorType::Sine,
            OscillatorType::Square => ServoMediaOscillatorType::Square,
            OscillatorType::Sawtooth => ServoMediaOscillatorType::Sawtooth,
            OscillatorType::Triangle => ServoMediaOscillatorType::Triangle,
            OscillatorType::Custom => ServoMediaOscillatorType::Custom,
        }
    }
}
