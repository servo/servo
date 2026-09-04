/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use js::rust::HandleObject;
use script_bindings::codegen::GenericBindings::DelayNodeBinding::{DelayNodeMethods, DelayOptions};
use script_bindings::reflector::reflect_dom_object_with_proto;
use servo_media::audio::audio_node::{AudioNodeInit, AudioNodeType};
use servo_media::audio::delay_node::DelayNodeOptions;
use servo_media::audio::param::ParamType;

use crate::conversions::Convert;
use crate::dom::audio::audionode::AudioNodeOptionsHelper;
use crate::dom::audio::audioparam::AudioParam;
use crate::dom::audio::baseaudiocontext::BaseAudioContext;
use crate::dom::bindings::codegen::Bindings::AudioNodeBinding::{
    ChannelCountMode, ChannelInterpretation,
};
use crate::dom::bindings::codegen::Bindings::AudioParamBinding::AutomationRate;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::types::AudioNode;
use crate::dom::window::Window;

#[dom_struct]
pub(crate) struct DelayNode {
    source_node: AudioNode,
    /// <https://webaudio.github.io/web-audio-api/#dom-delaynode-delaytime>
    delay_time: Dom<AudioParam>,
}

impl DelayNodeMethods<crate::DomTypeHolder> for DelayNode {
    /// <https://webaudio.github.io/web-audio-api/#dom-delaynode-delaynode>
    #[cfg_attr(crown, expect(crown::unrooted_must_root))]
    fn Constructor(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        context: &BaseAudioContext,
        options: &DelayOptions,
    ) -> Fallible<DomRoot<DelayNode>> {
        let node_options =
            options
                .parent
                .unwrap_or(2, ChannelCountMode::Max, ChannelInterpretation::Speakers);
        let delay_time = *options.delayTime;
        let max_delay_time = *options.maxDelayTime;
        let delay_options = AudioNodeInit::DelayNode(options.convert());
        let source_node = AudioNode::new_inherited(cx, delay_options, context, node_options, 1, 1)?;
        let node_id = source_node.node_id();
        let delay_time = AudioParam::new(
            cx,
            window,
            context,
            node_id,
            AudioNodeType::OscillatorNode,
            ParamType::DelayTime,
            AutomationRate::A_rate,
            delay_time as f32,
            0.,
            max_delay_time as f32,
        );
        let node = DelayNode {
            source_node,
            delay_time: Dom::from_ref(&delay_time),
        };

        Ok(reflect_dom_object_with_proto(
            cx,
            Box::new(node),
            window,
            proto,
        ))
    }

    /// <https://webaudio.github.io/web-audio-api/#dom-delaynode-delaytime>
    fn DelayTime(&self) -> DomRoot<AudioParam> {
        DomRoot::from_ref(&self.delay_time)
    }
}

impl Convert<DelayNodeOptions> for DelayOptions {
    fn convert(self) -> DelayNodeOptions {
        DelayNodeOptions {
            max_delay_time: *self.maxDelayTime,
            delay_time: *self.delayTime,
        }
    }
}
