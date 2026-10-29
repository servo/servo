/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use js::rust::HandleObject;
use script_bindings::reflector::reflect_dom_object_with_proto;
use servo_media::audio::audio_node::{AudioNodeInit, AudioNodeType};
use servo_media::audio::gain_node::GainNodeOptions;
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
use crate::dom::bindings::codegen::Bindings::GainNodeBinding::{GainNodeMethods, GainOptions};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::window::Window;

#[dom_struct]
pub(crate) struct GainNode {
    node: AudioNode,
    gain: Dom<AudioParam>,
}

impl GainNode {
    fn new_inherited(
        node_id: Option<NodeId>,
        context: &BaseAudioContext,
        node_options: UnwrappedAudioNodeOptions,
        gain: &AudioParam,
    ) -> Self {
        GainNode {
            node: AudioNode::new_inherited(
                node_id,
                context,
                node_options,
                1, // inputs
                1, // outputs
            ),
            gain: Dom::from_ref(gain),
        }
    }
}

impl GainNodeMethods<crate::DomTypeHolder> for GainNode {
    /// <https://webaudio.github.io/web-audio-api/#dom-gainnode-gainnode>
    fn Constructor(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        context: &BaseAudioContext,
        options: &GainOptions,
    ) -> Fallible<DomRoot<GainNode>> {
        let node_options =
            options
                .parent
                .unwrap_or(2, ChannelCountMode::Max, ChannelInterpretation::Speakers);
        if AudioNode::channel_count_check(node_options.count) {
            return Err(Error::NotSupported(Some(String::from(
                "Channel count not supported",
            ))));
        }

        let gain = *options.gain;
        let gain_init = AudioNodeInit::GainNode(options.convert());
        let node_id = AudioNode::generate_audio_node_id(cx, &node_options, gain_init, context);

        // <https://webaudio.github.io/web-audio-api/#dom-gainnode-gain>
        let gain = AudioParam::new(
            cx,
            window,
            context,
            node_id,
            AudioNodeType::GainNode,
            ParamType::Gain,
            AutomationRate::A_rate,
            gain,     // default value
            f32::MIN, // min value
            f32::MAX, // max value
        );
        Ok(reflect_dom_object_with_proto(
            cx,
            Box::new(GainNode::new_inherited(
                node_id,
                context,
                node_options,
                &gain,
            )),
            window,
            proto,
        ))
    }

    /// <https://webaudio.github.io/web-audio-api/#dom-gainnode-gain>
    fn Gain(&self) -> DomRoot<AudioParam> {
        DomRoot::from_ref(&self.gain)
    }
}

impl Convert<GainNodeOptions> for GainOptions {
    fn convert(self) -> GainNodeOptions {
        GainNodeOptions { gain: *self.gain }
    }
}
