/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use js::rust::HandleObject;
use script_bindings::reflector::reflect_dom_object_with_proto;
use servo_media::audio::audio_node::AudioNodeInit;
use servo_media::audio::channel_node::ChannelNodeOptions;
use servo_media::audio::graph::NodeId;

use crate::conversions::Convert;
use crate::dom::audio::audionode::{AudioNode, AudioNodeOptionsHelper, MAX_CHANNEL_COUNT};
use crate::dom::audio::baseaudiocontext::BaseAudioContext;
use crate::dom::audionode::UnwrappedAudioNodeOptions;
use crate::dom::bindings::codegen::Bindings::AudioNodeBinding::{
    ChannelCountMode, ChannelInterpretation,
};
use crate::dom::bindings::codegen::Bindings::ChannelMergerNodeBinding::{
    ChannelMergerNodeMethods, ChannelMergerOptions,
};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::root::DomRoot;
use crate::dom::window::Window;

#[dom_struct]
pub(crate) struct ChannelMergerNode {
    node: AudioNode,
}

impl ChannelMergerNode {
    fn new_inherited(
        node_id: Option<NodeId>,
        context: &BaseAudioContext,
        node_options: UnwrappedAudioNodeOptions,
        number_of_inputs: u32,
    ) -> Self {
        ChannelMergerNode {
            node: AudioNode::new_inherited(
                node_id,
                context,
                node_options,
                number_of_inputs, // inputs
                1,                // outputs
            ),
        }
    }
}

impl ChannelMergerNodeMethods<crate::DomTypeHolder> for ChannelMergerNode {
    /// <https://webaudio.github.io/web-audio-api/#dom-channelmergernode-channelmergernode>
    fn Constructor(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        context: &BaseAudioContext,
        options: &ChannelMergerOptions,
    ) -> Fallible<DomRoot<ChannelMergerNode>> {
        let node_options = options.parent.unwrap_or(
            1,
            ChannelCountMode::Explicit,
            ChannelInterpretation::Speakers,
        );

        if AudioNode::channel_count_check(node_options.count) {
            return Err(Error::NotSupported(Some(String::from(
                "Channel count not supported",
            ))));
        }

        if node_options.count != 1 || node_options.mode != ChannelCountMode::Explicit {
            return Err(Error::InvalidState(None));
        }

        if options.numberOfInputs < 1 || options.numberOfInputs > MAX_CHANNEL_COUNT {
            return Err(Error::IndexSize(None));
        }

        let audio_node_init = AudioNodeInit::ChannelMergerNode(options.convert());
        let node_id =
            AudioNode::generate_audio_node_id(cx, &node_options, audio_node_init, context);

        Ok(reflect_dom_object_with_proto(
            cx,
            Box::new(ChannelMergerNode::new_inherited(
                node_id,
                context,
                node_options,
                options.numberOfInputs, // inputs
            )),
            window,
            proto,
        ))
    }
}

impl Convert<ChannelNodeOptions> for ChannelMergerOptions {
    fn convert(self) -> ChannelNodeOptions {
        ChannelNodeOptions {
            channels: self.numberOfInputs as u8,
        }
    }
}
