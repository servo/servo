/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v.2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::audioscheduledsourcenode::AudioScheduledSourceNode;
use dom::baseaudiocontext::BaseAudioContext;
use dom::bindings::codegen::Bindings::AudioNodeBinding::AudioNodeOptions;
use dom::bindings::codegen::Bindings::AudioNodeBinding::{ChannelCountMode, ChannelInterpretation};
use dom::bindings::codegen::Bindings::OscillatorNodeBinding::{self, OscillatorOptions, OscillatorType};
use dom::bindings::error::Fallible;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::root::DomRoot;
use dom::window::Window;
use dom_struct::dom_struct;
use servo_media::audio::node::AudioNodeType;
use servo_media::audio::oscillator_node::OscillatorNodeOptions as ServoMediaOscillatorOptions;
use servo_media::audio::oscillator_node::OscillatorType as ServoMediaOscillatorType;

#[dom_struct]
pub struct OscillatorNode {
    node: AudioScheduledSourceNode,
    oscillator_type: OscillatorType,
    //    frequency: AudioParam,
    //    detune: AudioParam,
}

impl OscillatorNode {
    #[allow(unrooted_must_root)]
    #[allow(unsafe_code)]
    pub fn new_inherited(
        window: &Window,
        context: &BaseAudioContext,
        oscillator_options: &OscillatorOptions,
        ) -> OscillatorNode {
        let mut node_options = unsafe { AudioNodeOptions::empty(window.get_cx()) };
        node_options.channelCount = Some(2);
        node_options.channelCountMode = Some(ChannelCountMode::Max);
        node_options.channelInterpretation = Some(ChannelInterpretation::Speakers);
        OscillatorNode {
            node: AudioScheduledSourceNode::new_inherited(
                      AudioNodeType::OscillatorNode(oscillator_options.into()),
                      context,
                      &node_options,
                      0, /* inputs */
                      1, /* outputs */
                      ),
                      oscillator_type: oscillator_options.type_,
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(
        window: &Window,
        context: &BaseAudioContext,
        options: &OscillatorOptions,
        ) -> DomRoot<OscillatorNode> {
        let node = OscillatorNode::new_inherited(window, context, options);
        reflect_dom_object(Box::new(node), window, OscillatorNodeBinding::Wrap)
    }

    pub fn Constructor(
        window: &Window,
        context: &BaseAudioContext,
        options: &OscillatorOptions,
        ) -> Fallible<DomRoot<OscillatorNode>> {
        Ok(OscillatorNode::new(window, context, options))
    }
}

/*impl OscillatorNodeMethods for OscillatorNode {
  fn SetPeriodicWave(&self, periodic_wave: PeriodicWave) {
// XXX
}

fn Type(&self) -> OscillatorType {
self.oscillator_type
}

fn Frequency(&self) -> DomRoot<AudioParam> {
DomRoot::from_ref(&self.frequency)
}

fn Detune(&self) -> DomRoot<AudioParam> {
DomRoot::from_ref(&self.detune)
}
}*/

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
