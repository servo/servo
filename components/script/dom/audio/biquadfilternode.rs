/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use js::context::JSContext;
use js::rust::HandleObject;
use script_bindings::codegen::GenericBindings::BaseAudioContextBinding::BaseAudioContextMethods;
use script_bindings::reflector::reflect_dom_object_with_proto;
use servo_media::audio::audio_node::{AudioNodeInit, AudioNodeMessage, AudioNodeType};
use servo_media::audio::biquad_filter_node::{
    BiquadFilterNodeMessage, BiquadFilterNodeOptions, FilterType,
};
use servo_media::audio::graph::NodeId;
use servo_media::audio::param::ParamType;

use crate::conversions::Convert;
use crate::dom::audio::audionode::{AudioNode, AudioNodeOptionsHelper};
use crate::dom::audio::audioparam::AudioParam;
use crate::dom::audio::baseaudiocontext::BaseAudioContext;
use crate::dom::audionode::UnwrappedAudioNodeOptions;
use crate::dom::bindings::codegen::Bindings::AudioNodeBinding::{
    ChannelCountMode, ChannelInterpretation,
};
use crate::dom::bindings::codegen::Bindings::AudioParamBinding::AutomationRate;
use crate::dom::bindings::codegen::Bindings::BiquadFilterNodeBinding::{
    BiquadFilterNodeMethods, BiquadFilterOptions, BiquadFilterType,
};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::window::Window;

#[dom_struct]
pub(crate) struct BiquadFilterNode {
    node: AudioNode,
    gain: Dom<AudioParam>,
    frequency: Dom<AudioParam>,
    q: Dom<AudioParam>,
    detune: Dom<AudioParam>,
    filter: Cell<BiquadFilterType>,
}

impl BiquadFilterNode {
    fn new_inherited(
        node_id: Option<NodeId>,
        context: &BaseAudioContext,
        node_options: UnwrappedAudioNodeOptions,
        filter: BiquadFilterType,
        audio_params: [&AudioParam; 4],
    ) -> Self {
        BiquadFilterNode {
            node: AudioNode::new_inherited(
                node_id,
                context,
                node_options,
                1, // inputs
                1, // outputs
            ),
            filter: Cell::new(filter),
            gain: Dom::from_ref(audio_params[0]),
            q: Dom::from_ref(audio_params[1]),
            frequency: Dom::from_ref(audio_params[2]),
            detune: Dom::from_ref(audio_params[3]),
        }
    }
}

impl BiquadFilterNodeMethods<crate::DomTypeHolder> for BiquadFilterNode {
    /// <https://webaudio.github.io/web-audio-api/#dom-biquadfilternode-biquadfilternode-context-options>
    fn Constructor(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        context: &BaseAudioContext,
        options: &BiquadFilterOptions,
    ) -> Fallible<DomRoot<BiquadFilterNode>> {
        let node_options =
            options
                .parent
                .unwrap_or(2, ChannelCountMode::Max, ChannelInterpretation::Speakers);
        if AudioNode::channel_count_check(node_options.count) {
            return Err(Error::NotSupported(Some(String::from(
                "Channel count not supported",
            ))));
        }
        let filter = options.type_;
        let options = options.convert();
        let audio_node_init = AudioNodeInit::BiquadFilterNode(options);
        let node_id =
            AudioNode::generate_audio_node_id(cx, &node_options, audio_node_init, context);
        // <https://webaudio.github.io/web-audio-api/#dom-biquadfilternode-gain>
        let gain = AudioParam::new(
            cx,
            window,
            context,
            node_id,
            AudioNodeType::BiquadFilterNode,
            ParamType::Gain,
            AutomationRate::A_rate,
            options.gain,           // default value
            f32::MIN,               // min value
            40. * f32::MAX.log10(), // max value
        );
        // <https://webaudio.github.io/web-audio-api/#dom-biquadfilternode-q>
        let q = AudioParam::new(
            cx,
            window,
            context,
            node_id,
            AudioNodeType::BiquadFilterNode,
            ParamType::Q,
            AutomationRate::A_rate,
            options.q, // default value
            f32::MIN,  // min value
            f32::MAX,  // max value
        );
        let nyquist = (*context.SampleRate()) / 2.;
        // <https://webaudio.github.io/web-audio-api/#dom-biquadfilternode-frequency>
        let frequency = AudioParam::new(
            cx,
            window,
            context,
            node_id,
            AudioNodeType::BiquadFilterNode,
            ParamType::Frequency,
            AutomationRate::A_rate,
            options.frequency, // default value
            0.,                // min value
            nyquist,           // max value
        );
        let detune_max = 1200. * f32::MAX.log2();
        // <https://webaudio.github.io/web-audio-api/#dom-biquadfilternode-detune>
        let detune = AudioParam::new(
            cx,
            window,
            context,
            node_id,
            AudioNodeType::BiquadFilterNode,
            ParamType::Detune,
            AutomationRate::A_rate,
            options.detune, // default value
            -detune_max,    // min value
            detune_max,     // max value
        );
        Ok(reflect_dom_object_with_proto(
            cx,
            Box::new(BiquadFilterNode::new_inherited(
                node_id,
                context,
                node_options,
                filter,
                [&gain, &q, &frequency, &detune],
            )),
            window,
            proto,
        ))
    }

    /// <https://webaudio.github.io/web-audio-api/#dom-biquadfilternode-gain>
    fn Gain(&self) -> DomRoot<AudioParam> {
        DomRoot::from_ref(&self.gain)
    }

    /// <https://webaudio.github.io/web-audio-api/#dom-biquadfilternode-q>
    fn Q(&self) -> DomRoot<AudioParam> {
        DomRoot::from_ref(&self.q)
    }

    /// <https://webaudio.github.io/web-audio-api/#dom-biquadfilternode-detune>
    fn Detune(&self) -> DomRoot<AudioParam> {
        DomRoot::from_ref(&self.detune)
    }

    /// <https://webaudio.github.io/web-audio-api/#dom-biquadfilternode-frequency>
    fn Frequency(&self) -> DomRoot<AudioParam> {
        DomRoot::from_ref(&self.frequency)
    }

    /// <https://webaudio.github.io/web-audio-api/#dom-biquadfilternode-type>
    fn Type(&self) -> BiquadFilterType {
        self.filter.get()
    }

    /// <https://webaudio.github.io/web-audio-api/#dom-biquadfilternode-type>
    fn SetType(&self, filter: BiquadFilterType) {
        self.filter.set(filter);
        self.node.message(AudioNodeMessage::BiquadFilterNode(
            BiquadFilterNodeMessage::SetFilterType(filter.convert()),
        ));
    }
}

impl Convert<BiquadFilterNodeOptions> for &BiquadFilterOptions {
    fn convert(self) -> BiquadFilterNodeOptions {
        BiquadFilterNodeOptions {
            gain: *self.gain,
            q: *self.Q,
            frequency: *self.frequency,
            detune: *self.detune,
            filter: self.type_.convert(),
        }
    }
}

impl Convert<FilterType> for BiquadFilterType {
    fn convert(self) -> FilterType {
        match self {
            BiquadFilterType::Lowpass => FilterType::LowPass,
            BiquadFilterType::Highpass => FilterType::HighPass,
            BiquadFilterType::Bandpass => FilterType::BandPass,
            BiquadFilterType::Lowshelf => FilterType::LowShelf,
            BiquadFilterType::Highshelf => FilterType::HighShelf,
            BiquadFilterType::Peaking => FilterType::Peaking,
            BiquadFilterType::Allpass => FilterType::AllPass,
            BiquadFilterType::Notch => FilterType::Notch,
        }
    }
}
