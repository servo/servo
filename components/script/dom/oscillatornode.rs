/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::audioparam::AudioParam;
use dom::audioscheduledsourcenode::AudioScheduledSourceNode;
use dom::baseaudiocontext::BaseAudioContext;
use dom::bindings::codegen::Bindings::AudioNodeBinding::{ChannelCountMode, ChannelInterpretation};
use dom::bindings::codegen::Bindings::AudioNodeBinding::AudioNodeOptions;
use dom::bindings::codegen::Bindings::AudioParamBinding::AutomationRate;
use dom::bindings::codegen::Bindings::OscillatorNodeBinding::{self, OscillatorOptions, OscillatorType};
use dom::bindings::codegen::Bindings::OscillatorNodeBinding::OscillatorNodeMethods;
use dom::bindings::error::Fallible;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::root::{Dom, DomRoot};
use dom::window::Window;
use dom_struct::dom_struct;
use servo_media::audio::node::AudioNodeInit;
use servo_media::audio::oscillator_node::OscillatorNodeOptions as ServoMediaOscillatorOptions;
use servo_media::audio::oscillator_node::OscillatorType as ServoMediaOscillatorType;
use servo_media::audio::param::ParamType;
use std::f32;
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct OscillatorNode<TH: TypeHolderTrait> {
    source_node: AudioScheduledSourceNode<TH>,
    oscillator_type: OscillatorType,
    frequency: Dom<AudioParam<TH>>,
    detune: Dom<AudioParam<TH>>,
}

impl<TH: TypeHolderTrait> OscillatorNode<TH> {
    #[allow(unrooted_must_root)]
    pub fn new_inherited(
        window: &Window<TH>,
        context: &BaseAudioContext<TH>,
        oscillator_options: &OscillatorOptions,
    ) -> OscillatorNode<TH> {
        let mut node_options = AudioNodeOptions::empty();
        node_options.channelCount = Some(2);
        node_options.channelCountMode = Some(ChannelCountMode::Max);
        node_options.channelInterpretation = Some(ChannelInterpretation::Speakers);
        let source_node = AudioScheduledSourceNode::new_inherited(
            AudioNodeInit::OscillatorNode(oscillator_options.into()),
            context,
            &node_options,
            0, /* inputs */
            1, /* outputs */
        );
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

        OscillatorNode {
            source_node,
            oscillator_type: oscillator_options.type_,
            frequency: Dom::from_ref(&frequency),
            detune: Dom::from_ref(&detune),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(
        window: &Window<TH>,
        context: &BaseAudioContext<TH>,
        options: &OscillatorOptions,
    ) -> DomRoot<OscillatorNode<TH>> {
        let node = OscillatorNode::new_inherited(window, context, options);
        reflect_dom_object(Box::new(node), window, OscillatorNodeBinding::Wrap)
    }

    pub fn Constructor(
        window: &Window<TH>,
        context: &BaseAudioContext<TH>,
        options: &OscillatorOptions,
    ) -> Fallible<DomRoot<OscillatorNode<TH>>> {
        Ok(OscillatorNode::new(window, context, options))
    }
}

impl<TH: TypeHolderTrait> OscillatorNodeMethods<TH> for OscillatorNode<TH> {
    // https://webaudio.github.io/web-audio-api/#dom-oscillatornode-frequency
    fn Frequency(&self) -> DomRoot<AudioParam<TH>> {
        DomRoot::from_ref(&self.frequency)
    }

    // https://webaudio.github.io/web-audio-api/#dom-oscillatornode-detune
    fn Detune(&self) -> DomRoot<AudioParam<TH>> {
        DomRoot::from_ref(&self.detune)
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
