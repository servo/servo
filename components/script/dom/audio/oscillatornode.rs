/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use js::context::JSContext;
use js::rust::HandleObject;
use script_bindings::reflector::reflect_dom_object_with_proto;
use servo_media::audio::audio_node::{AudioNodeInit, AudioNodeMessage, AudioNodeType};
use servo_media::audio::oscillator_node::{
    OscillatorNodeMessage, OscillatorNodeOptions as ServoMediaOscillatorOptions,
    OscillatorType as ServoMediaOscillatorType,
};
use servo_media::audio::param::ParamType;
use servo_media::audio::periodic_wave::PeriodicWave as ServoMediaPeriodicWave;

use crate::conversions::Convert;
use crate::dom::audio::audionode::AudioNodeOptionsHelper;
use crate::dom::audio::audioparam::AudioParam;
use crate::dom::audio::audioscheduledsourcenode::AudioScheduledSourceNode;
use crate::dom::audio::baseaudiocontext::BaseAudioContext;
use crate::dom::bindings::codegen::Bindings::AudioNodeBinding::{
    ChannelCountMode, ChannelInterpretation,
};
use crate::dom::bindings::codegen::Bindings::AudioParamBinding::AutomationRate;
use crate::dom::bindings::codegen::Bindings::OscillatorNodeBinding::{
    OscillatorNodeMethods, OscillatorOptions, OscillatorType,
};
use crate::dom::bindings::error::{Error, ErrorResult, Fallible};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::types::PeriodicWave;
use crate::dom::window::Window;

#[dom_struct]
pub(crate) struct OscillatorNode {
    source_node: AudioScheduledSourceNode,
    detune: Dom<AudioParam>,
    frequency: Dom<AudioParam>,
    oscillator_type: Cell<OscillatorType>,
}

impl OscillatorNode {
    #[cfg_attr(crown, expect(crown::unrooted_must_root))]
    pub(crate) fn new_inherited(
        cx: &mut JSContext,
        window: &Window,
        context: &BaseAudioContext,
        options: &OscillatorOptions,
    ) -> Fallible<OscillatorNode> {
        if matches!(options.type_, OscillatorType::Custom) && options.periodicWave.is_none() {
            return Err(Error::InvalidState(Some(String::from(
                "Can not set oscillator type to custom without providing a periodic wave",
            ))));
        }

        let oscillator_type = if options.periodicWave.is_some() {
            OscillatorType::Custom
        } else {
            options.type_
        };

        let node_options =
            options
                .parent
                .unwrap_or(2, ChannelCountMode::Max, ChannelInterpretation::Speakers);

        let maybe_periodic_wave = options
            .periodicWave
            .as_ref()
            .map(|x| (*x.as_traced()).convert());

        let options = ServoMediaOscillatorOptions {
            oscillator_type: convert_oscillator_options(oscillator_type, maybe_periodic_wave)?,
            freq: *options.frequency,
            detune: *options.detune,
        };
        let source_node = AudioScheduledSourceNode::new_inherited(
            cx,
            AudioNodeInit::OscillatorNode(options),
            context,
            node_options,
            0, /* inputs */
            1, /* outputs */
        )?;
        let node_id = source_node.node().node_id();
        let frequency = AudioParam::new(
            cx,
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
            cx,
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
            oscillator_type: Cell::new(oscillator_type),
            frequency: Dom::from_ref(&frequency),
            detune: Dom::from_ref(&detune),
        })
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        window: &Window,
        context: &BaseAudioContext,
        options: &OscillatorOptions,
    ) -> Fallible<DomRoot<OscillatorNode>> {
        Self::new_with_proto(cx, window, None, context, options)
    }

    #[cfg_attr(crown, expect(crown::unrooted_must_root))]
    fn new_with_proto(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        context: &BaseAudioContext,
        options: &OscillatorOptions,
    ) -> Fallible<DomRoot<OscillatorNode>> {
        let node = OscillatorNode::new_inherited(cx, window, context, options)?;
        Ok(reflect_dom_object_with_proto(
            cx,
            Box::new(node),
            window,
            proto,
        ))
    }
}

impl OscillatorNodeMethods<crate::DomTypeHolder> for OscillatorNode {
    /// <https://webaudio.github.io/web-audio-api/#dom-oscillatornode-oscillatornode>
    fn Constructor(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        context: &BaseAudioContext,
        options: &OscillatorOptions,
    ) -> Fallible<DomRoot<OscillatorNode>> {
        OscillatorNode::new_with_proto(cx, window, proto, context, options)
    }

    /// <https://webaudio.github.io/web-audio-api/#dom-oscillatornode-setperiodicwave>
    fn SetPeriodicWave(&self, periodic_wave: &PeriodicWave) -> ErrorResult {
        self.oscillator_type.set(OscillatorType::Custom);
        self.source_node
            .node()
            .message(AudioNodeMessage::OscillatorNode(
                OscillatorNodeMessage::SetPeriodicWave(periodic_wave.convert()),
            ));
        Ok(())
    }

    /// <https://webaudio.github.io/web-audio-api/#dom-oscillatornode-frequency>
    fn Frequency(&self) -> DomRoot<AudioParam> {
        DomRoot::from_ref(&self.frequency)
    }

    /// <https://webaudio.github.io/web-audio-api/#dom-oscillatornode-detune>
    fn Detune(&self) -> DomRoot<AudioParam> {
        DomRoot::from_ref(&self.detune)
    }

    /// <https://webaudio.github.io/web-audio-api/#dom-oscillatornode-type>
    fn Type(&self) -> OscillatorType {
        self.oscillator_type.get()
    }

    /// <https://webaudio.github.io/web-audio-api/#dom-oscillatornode-type>
    fn SetType(&self, type_: OscillatorType) -> ErrorResult {
        if type_ == OscillatorType::Custom {
            return Err(Error::InvalidState(None));
        }
        self.oscillator_type.set(type_);
        self.source_node
            .node()
            .message(AudioNodeMessage::OscillatorNode(
                OscillatorNodeMessage::SetOscillatorType(convert_oscillator_options(type_, None)?),
            ));
        Ok(())
    }
}

// Helper function because Convert trait is not sufficient to handle Custom variant
fn convert_oscillator_options(
    oscillator_type: OscillatorType,
    maybe_periodic_wave: Option<ServoMediaPeriodicWave>,
) -> Fallible<ServoMediaOscillatorType> {
    match oscillator_type {
        OscillatorType::Sine => Ok(ServoMediaOscillatorType::Sine),
        OscillatorType::Square => Ok(ServoMediaOscillatorType::Square),
        OscillatorType::Sawtooth => Ok(ServoMediaOscillatorType::Sawtooth),
        OscillatorType::Triangle => Ok(ServoMediaOscillatorType::Triangle),
        OscillatorType::Custom => {
            let Some(periodic_wave) = maybe_periodic_wave else {
                return Err(Error::InvalidState(Some(String::from(
                    "Can not have oscillator type custom without a periodic wave",
                ))));
            };
            Ok(ServoMediaOscillatorType::Custom(periodic_wave))
        },
    }
}
