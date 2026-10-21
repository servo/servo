/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use js::rust::HandleObject;
use script_bindings::reflector::reflect_dom_object_with_proto;
use servo_media::audio::audio_node::{AudioNodeInit, AudioNodeType};
use servo_media::audio::constant_source_node::ConstantSourceNodeOptions as ServoMediaConstantSourceOptions;
use servo_media::audio::graph::NodeId;
use servo_media::audio::param::ParamType;

use crate::conversions::Convert;
use crate::dom::audio::audionode::AudioNode;
use crate::dom::audio::audioparam::AudioParam;
use crate::dom::audio::audioscheduledsourcenode::AudioScheduledSourceNode;
use crate::dom::audio::baseaudiocontext::BaseAudioContext;
use crate::dom::audionode::UnwrappedAudioNodeOptions;
use crate::dom::bindings::codegen::Bindings::AudioParamBinding::AutomationRate;
use crate::dom::bindings::codegen::Bindings::ConstantSourceNodeBinding::{
    ConstantSourceNodeMethods, ConstantSourceOptions,
};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::window::Window;

#[dom_struct]
pub(crate) struct ConstantSourceNode {
    source_node: AudioScheduledSourceNode,
    offset: Dom<AudioParam>,
}

impl ConstantSourceNode {
    fn new_inherited(
        node_id: Option<NodeId>,
        context: &BaseAudioContext,
        node_options: UnwrappedAudioNodeOptions,
        offset: &AudioParam,
    ) -> Self {
        ConstantSourceNode {
            source_node: AudioScheduledSourceNode::new_inherited(
                node_id,
                context,
                node_options, /* 2, MAX, Speakers */
                0,            /* inputs */
                1,            /* outputs */
            ),
            offset: Dom::from_ref(offset),
        }
    }
}

impl ConstantSourceNodeMethods<crate::DomTypeHolder> for ConstantSourceNode {
    /// <https://webaudio.github.io/web-audio-api/#dom-constantsourcenode-constantsourcenode>
    fn Constructor(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        context: &BaseAudioContext,
        options: &ConstantSourceOptions,
    ) -> Fallible<DomRoot<ConstantSourceNode>> {
        let node_options: UnwrappedAudioNodeOptions = Default::default();
        if AudioNode::channel_count_check(node_options.count) {
            return Err(Error::NotSupported(Some(String::from(
                "Channel count not supported",
            ))));
        }
        let offset = *options.offset;
        let node_id = AudioNode::generate_audio_node_id(
            cx,
            &node_options,
            AudioNodeInit::ConstantSourceNode(options.convert()),
            context,
        );
        // <https://webaudio.github.io/web-audio-api/#dom-constantsourcenode-offset>
        let offset = AudioParam::new(
            cx,
            window,
            context,
            node_id,
            AudioNodeType::ConstantSourceNode,
            ParamType::Offset,
            AutomationRate::A_rate,
            offset,
            f32::MIN,
            f32::MAX,
        );

        Ok(reflect_dom_object_with_proto(
            cx,
            Box::new(ConstantSourceNode::new_inherited(
                node_id,
                context,
                node_options, /* 2, MAX, Speakers */
                &offset,
            )),
            window,
            proto,
        ))
    }

    /// <https://webaudio.github.io/web-audio-api/#dom-constantsourcenode-offset>
    fn Offset(&self) -> DomRoot<AudioParam> {
        DomRoot::from_ref(&self.offset)
    }
}

impl Convert<ServoMediaConstantSourceOptions> for ConstantSourceOptions {
    fn convert(self) -> ServoMediaConstantSourceOptions {
        ServoMediaConstantSourceOptions {
            offset: *self.offset,
        }
    }
}
