/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::reflector::reflect_dom_object_with_cx;

use crate::dom::audio::audionode::{AudioNode, AudioNodeOptionsHelper, MAX_CHANNEL_COUNT};
use crate::dom::audio::baseaudiocontext::BaseAudioContext;
use crate::dom::bindings::codegen::Bindings::AudioDestinationNodeBinding::AudioDestinationNodeMethods;
use crate::dom::bindings::codegen::Bindings::AudioNodeBinding::{
    AudioNodeOptions, ChannelCountMode, ChannelInterpretation,
};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;

#[dom_struct]
pub(crate) struct AudioDestinationNode {
    node: AudioNode,
}

impl AudioDestinationNode {
    fn new_inherited(
        context: &BaseAudioContext,
        options: &AudioNodeOptions,
    ) -> AudioDestinationNode {
        let node_options =
            options.unwrap_or(2, ChannelCountMode::Max, ChannelInterpretation::Speakers);
        AudioDestinationNode {
            node: AudioNode::new_inherited_for_id(
                Some(context.destination_node()),
                context,
                node_options,
                1,
                1,
            ),
        }
    }

    #[cfg_attr(crown, expect(crown::unrooted_must_root))]
    pub(crate) fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        context: &BaseAudioContext,
        options: &AudioNodeOptions,
    ) -> DomRoot<AudioDestinationNode> {
        let node = AudioDestinationNode::new_inherited(context, options);
        reflect_dom_object_with_cx(Box::new(node), global, cx)
    }
}

impl AudioDestinationNodeMethods<crate::DomTypeHolder> for AudioDestinationNode {
    /// <https://webaudio.github.io/web-audio-api/#dom-audiodestinationnode-maxchannelcount>
    fn MaxChannelCount(&self) -> u32 {
        MAX_CHANNEL_COUNT
    }
}
