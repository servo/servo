/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::audionode::AudioNode;
use dom::audioparam::AudioParam;
use dom::baseaudiocontext::BaseAudioContext;
use dom::bindings::codegen::Bindings::AudioNodeBinding::{ChannelCountMode, ChannelInterpretation};
use dom::bindings::codegen::Bindings::AudioParamBinding::AutomationRate;
use dom::bindings::codegen::Bindings::BiquadFilterNodeBinding::{self, BiquadFilterNodeMethods};
use dom::bindings::codegen::Bindings::BiquadFilterNodeBinding::BiquadFilterOptions;
use dom::bindings::codegen::Bindings::BiquadFilterNodeBinding::BiquadFilterType;
use dom::bindings::error::Fallible;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::root::{Dom, DomRoot};
use dom::window::Window;
use dom_struct::dom_struct;
use servo_media::audio::biquad_filter_node::{BiquadFilterNodeOptions, FilterType};
use servo_media::audio::biquad_filter_node::BiquadFilterNodeMessage;
use servo_media::audio::node::{AudioNodeInit, AudioNodeMessage};
use servo_media::audio::param::ParamType;
use std::cell::Cell;
use std::f32;
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct BiquadFilterNode<TH: TypeHolderTrait> {
    node: AudioNode<TH>,
    gain: Dom<AudioParam<TH>>,
    frequency: Dom<AudioParam<TH>>,
    q: Dom<AudioParam<TH>>,
    detune: Dom<AudioParam<TH>>,
    filter: Cell<BiquadFilterType>,
}

impl<TH: TypeHolderTrait> BiquadFilterNode<TH> {
    #[allow(unrooted_must_root)]
    pub fn new_inherited(
        window: &Window<TH>,
        context: &BaseAudioContext<TH>,
        options: &BiquadFilterOptions,
    ) -> Fallible<BiquadFilterNode<TH>> {
        let node_options = options.parent
                                  .unwrap_or(2, ChannelCountMode::Max,
                                             ChannelInterpretation::Speakers);
        let filter = Cell::new(options.type_);
        let options = options.into();
        let node = AudioNode::new_inherited(
            AudioNodeInit::BiquadFilterNode(options),
            context,
            node_options,
            1, // inputs
            1, // outputs
        )?;
        let gain = AudioParam::new(
            window,
            context,
            node.node_id(),
            ParamType::Gain,
            AutomationRate::A_rate,
            options.gain,       // default value
            f32::MIN, // min value
            f32::MAX, // max value
        );
        let q = AudioParam::new(
            window,
            context,
            node.node_id(),
            ParamType::Q,
            AutomationRate::A_rate,
            options.q,       // default value
            f32::MIN, // min value
            f32::MAX, // max value
        );
        let frequency = AudioParam::new(
            window,
            context,
            node.node_id(),
            ParamType::Frequency,
            AutomationRate::A_rate,
            options.frequency,       // default value
            f32::MIN, // min value
            f32::MAX, // max value
        );
        let detune = AudioParam::new(
            window,
            context,
            node.node_id(),
            ParamType::Detune,
            AutomationRate::A_rate,
            options.detune,       // default value
            f32::MIN, // min value
            f32::MAX, // max value
        );
        Ok(BiquadFilterNode {
            node,
            filter,
            gain: Dom::from_ref(&gain),
            q: Dom::from_ref(&q),
            frequency: Dom::from_ref(&frequency),
            detune: Dom::from_ref(&detune),
        })
    }

    #[allow(unrooted_must_root)]
    pub fn new(
        window: &Window<TH>,
        context: &BaseAudioContext<TH>,
        options: &BiquadFilterOptions,
    ) -> Fallible<DomRoot<BiquadFilterNode<TH>>> {
        let node = BiquadFilterNode::new_inherited(window, context, options)?;
        Ok(reflect_dom_object(Box::new(node), window, BiquadFilterNodeBinding::Wrap))
    }

    pub fn Constructor(
        window: &Window<TH>,
        context: &BaseAudioContext<TH>,
        options: &BiquadFilterOptions,
    ) -> Fallible<DomRoot<BiquadFilterNode<TH>>> {
        BiquadFilterNode::new(window, context, options)
    }
}

impl<TH: TypeHolderTrait> BiquadFilterNodeMethods<TH> for BiquadFilterNode<TH> {
    // https://webaudio.github.io/web-audio-api/#dom-biquadfilternode-gain
    fn Gain(&self) -> DomRoot<AudioParam<TH>> {
        DomRoot::from_ref(&self.gain)
    }

    // https://webaudio.github.io/web-audio-api/#dom-biquadfilternode-q
    fn Q(&self) -> DomRoot<AudioParam<TH>> {
        DomRoot::from_ref(&self.q)
    }

    // https://webaudio.github.io/web-audio-api/#dom-biquadfilternode-detune
    fn Detune(&self) -> DomRoot<AudioParam<TH>> {
        DomRoot::from_ref(&self.detune)
    }

    // https://webaudio.github.io/web-audio-api/#dom-biquadfilternode-frequency
    fn Frequency(&self) -> DomRoot<AudioParam<TH>> {
        DomRoot::from_ref(&self.frequency)
    }

    // https://webaudio.github.io/web-audio-api/#dom-biquadfilternode-type
    fn Type(&self) -> BiquadFilterType {
        self.filter.get()
    }

    // https://webaudio.github.io/web-audio-api/#dom-biquadfilternode-type
    fn SetType(&self, filter: BiquadFilterType) {
        self.filter.set(filter);
        self.node
            .message(AudioNodeMessage::BiquadFilterNode(
                BiquadFilterNodeMessage::SetFilterType(filter.into()),
            ));
    }
}

impl<'a> From<&'a BiquadFilterOptions> for BiquadFilterNodeOptions {
    fn from(options: &'a BiquadFilterOptions) -> Self {
        Self {
            gain: *options.gain,
            q: *options.Q,
            frequency: *options.frequency,
            detune: *options.detune,
            filter: options.type_.into()
        }
    }
}

impl From<BiquadFilterType> for FilterType {
    fn from(filter: BiquadFilterType) -> FilterType {
        match filter {
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
