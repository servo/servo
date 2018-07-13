/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v.2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::audionode::AudioNode;
use dom::audioparam::AudioParam;
use dom::baseaudiocontext::BaseAudioContext;
use dom::bindings::codegen::Bindings::AudioParamBinding::AutomationRate;
use dom::bindings::codegen::Bindings::AudioNodeBinding::AudioNodeOptions;
use dom::bindings::codegen::Bindings::AudioNodeBinding::{ChannelCountMode, ChannelInterpretation};
use dom::bindings::codegen::Bindings::GainNodeBinding::{self, GainNodeMethods, GainOptions};
use dom::bindings::error::Fallible;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::root::{Dom, DomRoot};
use dom::window::Window;
use dom_struct::dom_struct;
use servo_media::audio::gain_node::GainNodeOptions;
use servo_media::audio::node::AudioNodeInit;
use servo_media::audio::param::ParamType;
use std::f32;

#[dom_struct]
pub struct GainNode {
    node: AudioNode,
    gain: Dom<AudioParam>,
}

impl GainNode {
    #[allow(unrooted_must_root)]
    pub fn new_inherited(
        window: &Window,
        context: &BaseAudioContext,
        gain_options: &GainOptions,
        ) -> GainNode {
        let mut node_options = AudioNodeOptions::empty();
        node_options.channelCount = Some(2);
        node_options.channelCountMode = Some(ChannelCountMode::Max);
        node_options.channelInterpretation = Some(ChannelInterpretation::Speakers);
        let node = AudioNode::new_inherited(
            AudioNodeInit::GainNode(gain_options.into()),
            None,
            context,
            &node_options,
            1, // inputs
            1, // outputs
            );
        let gain = AudioParam::new(window,
                                   context,
                                   node.node_id(),
                                   ParamType::Gain,
                                   AutomationRate::A_rate,
                                   1., // default value
                                   f32::MIN, // min value
                                   f32::MAX, // max value
                                  );
        GainNode {
            node,
            gain: Dom::from_ref(&gain),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window,
               context: &BaseAudioContext,
               options: &GainOptions
              ) -> DomRoot<GainNode> {
        let node = GainNode::new_inherited(window, context, options);
        reflect_dom_object(Box::new(node), window, GainNodeBinding::Wrap)
    }

    pub fn Constructor(
        window: &Window,
        context: &BaseAudioContext,
        options: &GainOptions,
        ) -> Fallible<DomRoot<GainNode>> {
        Ok(GainNode::new(window, context, options))
    }
}

impl GainNodeMethods for GainNode {
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
