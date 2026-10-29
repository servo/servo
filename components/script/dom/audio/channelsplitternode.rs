/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use js::rust::HandleObject;
use script_bindings::reflector::reflect_dom_object_with_proto;
use servo_media::audio::audio_node::AudioNodeInit;
use servo_media::audio::graph::NodeId;

use crate::dom::audio::audionode::{AudioNode, AudioNodeOptionsHelper, MAX_CHANNEL_COUNT};
use crate::dom::audio::baseaudiocontext::BaseAudioContext;
use crate::dom::audionode::UnwrappedAudioNodeOptions;
use crate::dom::bindings::codegen::Bindings::AudioNodeBinding::{
    ChannelCountMode, ChannelInterpretation,
};
use crate::dom::bindings::codegen::Bindings::ChannelSplitterNodeBinding::{
    ChannelSplitterNodeMethods, ChannelSplitterOptions,
};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::root::DomRoot;
use crate::dom::window::Window;

#[dom_struct]
pub(crate) struct ChannelSplitterNode {
    node: AudioNode,
}

impl ChannelSplitterNode {
    fn new_inherited(
        node_id: Option<NodeId>,
        context: &BaseAudioContext,
        node_options: UnwrappedAudioNodeOptions,
        number_of_outputs: u32,
    ) -> Self {
        ChannelSplitterNode {
            node: AudioNode::new_inherited(
                node_id,
                context,
                node_options,
                1,                 // inputs
                number_of_outputs, // outputs
            ),
        }
    }
}

impl ChannelSplitterNodeMethods<crate::DomTypeHolder> for ChannelSplitterNode {
    /// <https://webaudio.github.io/web-audio-api/#dom-channelsplitternode-channelsplitternode>
    fn Constructor(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        context: &BaseAudioContext,
        options: &ChannelSplitterOptions,
    ) -> Fallible<DomRoot<ChannelSplitterNode>> {
        if options.numberOfOutputs < 1 || options.numberOfOutputs > MAX_CHANNEL_COUNT {
            return Err(Error::IndexSize(None));
        }

        let node_options = options.parent.unwrap_or(
            options.numberOfOutputs,
            ChannelCountMode::Explicit,
            ChannelInterpretation::Discrete,
        );

        if AudioNode::channel_count_check(node_options.count) {
            return Err(Error::NotSupported(Some(String::from(
                "Channel count not supported",
            ))));
        }

        if node_options.count != options.numberOfOutputs ||
            node_options.mode != ChannelCountMode::Explicit ||
            node_options.interpretation != ChannelInterpretation::Discrete
        {
            return Err(Error::InvalidState(None));
        }

        let audio_node_init = AudioNodeInit::ChannelSplitterNode;
        let node_id =
            AudioNode::generate_audio_node_id(cx, &node_options, audio_node_init, context);

        Ok(reflect_dom_object_with_proto(
            cx,
            Box::new(ChannelSplitterNode::new_inherited(
                node_id,
                context,
                node_options,
                options.numberOfOutputs, // outputs
            )),
            window,
            proto,
        ))
    }
}
