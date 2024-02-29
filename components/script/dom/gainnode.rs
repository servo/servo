/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::f32;

use dom_struct::dom_struct;
use js::rust::HandleObject;
use servo_media::audio::gain_node::GainNodeOptions;
use servo_media::audio::node::{AudioNodeInit, AudioNodeType};
use servo_media::audio::param::ParamType;

use crate::dom::audionode::AudioNode;
use crate::dom::audioparam::AudioParam;
use crate::dom::baseaudiocontext::BaseAudioContext;
use crate::dom::bindings::codegen::Bindings::AudioNodeBinding::{
    ChannelCountMode, ChannelInterpretation,
};
use crate::dom::bindings::codegen::Bindings::AudioParamBinding::AutomationRate;
use crate::dom::bindings::codegen::Bindings::GainNodeBinding::{GainNodeMethods, GainOptions};
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::window::Window;

#[dom_struct]
pub struct GainNode {
    node: AudioNode,
    gain: Dom<AudioParam>,
}

impl GainNode {
    #[allow(crown::unrooted_must_root)]
    pub fn new_inherited(
        window: &Window,
        context: &BaseAudioContext,
        options: &GainOptions,
    ) -> Fallible<GainNode> {
        let node_options =
            options
                .parent
                .unwrap_or(2, ChannelCountMode::Max, ChannelInterpretation::Speakers);
        let node = AudioNode::new_inherited(
            AudioNodeInit::GainNode(options.into()),
            context,
            node_options,
            1, // inputs
            1, // outputs
        )?;
        let gain = AudioParam::new(
            window,
            context,
            node.node_id(),
            AudioNodeType::GainNode,
            ParamType::Gain,
            AutomationRate::A_rate,
            *options.gain, // default value
            f32::MIN,      // min value
            f32::MAX,      // max value
        );
        Ok(GainNode {
            node,
            gain: Dom::from_ref(&gain),
        })
    }

    pub fn new(
        window: &Window,
        context: &BaseAudioContext,
        options: &GainOptions,
    ) -> Fallible<DomRoot<GainNode>> {
        Self::new_with_proto(window, None, context, options)
    }

    #[allow(crown::unrooted_must_root)]
    fn new_with_proto(
        window: &Window,
        proto: Option<HandleObject>,
        context: &BaseAudioContext,
        options: &GainOptions,
    ) -> Fallible<DomRoot<GainNode>> {
        let node = GainNode::new_inherited(window, context, options)?;
        Ok(reflect_dom_object_with_proto(Box::new(node), window, proto))
    }

    #[allow(non_snake_case)]
    pub fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        context: &BaseAudioContext,
        options: &GainOptions,
    ) -> Fallible<DomRoot<GainNode>> {
        GainNode::new_with_proto(window, proto, context, options)
    }
}

impl GainNodeMethods for GainNode {
    // https://webaudio.github.io/web-audio-api/#dom-gainnode-gain
    fn Gain(&self) -> DomRoot<AudioParam> {
        DomRoot::from_ref(&self.gain)
    }
}

impl<'a> From<&'a GainOptions> for GainNodeOptions {
    fn from(options: &'a GainOptions) -> Self {
        Self {
            gain: *options.gain,
        }
    }
}
