/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::f32;

use dom_struct::dom_struct;
use js::rust::HandleObject;
use servo_media::audio::node::{AudioNodeInit, AudioNodeMessage, AudioNodeType};
use servo_media::audio::oscillator_node::{
    OscillatorNodeMessage, OscillatorNodeOptions as ServoMediaOscillatorOptions,
    OscillatorType as ServoMediaOscillatorType,
};
use servo_media::audio::param::ParamType;

use crate::conversions::Convert;
use crate::dom::audioparam::AudioParam;
use crate::dom::audioscheduledsourcenode::AudioScheduledSourceNode;
use crate::dom::baseaudiocontext::BaseAudioContext;
use crate::dom::bindings::codegen::Bindings::AudioNodeBinding::{
    ChannelCountMode, ChannelInterpretation,
};
use crate::dom::bindings::codegen::Bindings::AudioParamBinding::AutomationRate;
use crate::dom::bindings::codegen::Bindings::OscillatorNodeBinding::{
    OscillatorNodeMethods, OscillatorOptions, OscillatorType,
};
use crate::dom::bindings::error::{Error, ErrorResult, Fallible};
use crate::dom::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct OscillatorNode {
    source_node: AudioScheduledSourceNode,
    detune: Dom<AudioParam>,
    frequency: Dom<AudioParam>,
    oscillator_type: Cell<OscillatorType>,
}

impl OscillatorNode {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new_inherited(
        window: &Window,
        context: &BaseAudioContext,
        options: &OscillatorOptions,
    ) -> Fallible<OscillatorNode> {
        let node_options =
            options
                .parent
                .unwrap_or(2, ChannelCountMode::Max, ChannelInterpretation::Speakers);
        let source_node = AudioScheduledSourceNode::new_inherited(
            AudioNodeInit::OscillatorNode(options.convert()),
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
            AudioNodeType::OscillatorNode,
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
            AudioNodeType::OscillatorNode,
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

    pub(crate) fn new(
        window: &Window,
        context: &BaseAudioContext,
        options: &OscillatorOptions,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<OscillatorNode>> {
        Self::new_with_proto(window, None, context, options, can_gc)
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn new_with_proto(
        window: &Window,
        proto: Option<HandleObject>,
        context: &BaseAudioContext,
        options: &OscillatorOptions,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<OscillatorNode>> {
        let node = OscillatorNode::new_inherited(window, context, options)?;
        Ok(reflect_dom_object_with_proto(
            Box::new(node),
            window,
            proto,
            can_gc,
        ))
    }
}

impl OscillatorNodeMethods<crate::DomTypeHolder> for OscillatorNode {
    // https://webaudio.github.io/web-audio-api/#dom-oscillatornode-oscillatornode
    fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        context: &BaseAudioContext,
        options: &OscillatorOptions,
    ) -> Fallible<DomRoot<OscillatorNode>> {
        OscillatorNode::new_with_proto(window, proto, context, options, can_gc)
    }

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
                OscillatorNodeMessage::SetOscillatorType(type_.convert()),
            ));
        Ok(())
    }
}

impl Convert<ServoMediaOscillatorOptions> for &OscillatorOptions {
    fn convert(self) -> ServoMediaOscillatorOptions {
        ServoMediaOscillatorOptions {
            oscillator_type: self.type_.convert(),
            freq: *self.frequency,
            detune: *self.detune,
            periodic_wave_options: None, // XXX
        }
    }
}

impl Convert<ServoMediaOscillatorType> for OscillatorType {
    fn convert(self) -> ServoMediaOscillatorType {
        match self {
            OscillatorType::Sine => ServoMediaOscillatorType::Sine,
            OscillatorType::Square => ServoMediaOscillatorType::Square,
            OscillatorType::Sawtooth => ServoMediaOscillatorType::Sawtooth,
            OscillatorType::Triangle => ServoMediaOscillatorType::Triangle,
            OscillatorType::Custom => ServoMediaOscillatorType::Custom,
        }
    }
}
