/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::audionode::{AudioNode, MAX_CHANNEL_COUNT};
use dom::baseaudiocontext::BaseAudioContext;
use dom::bindings::codegen::Bindings::AudioDestinationNodeBinding::{self, AudioDestinationNodeMethods};
use dom::bindings::codegen::Bindings::AudioNodeBinding::AudioNodeOptions;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::root::DomRoot;
use dom::globalscope::GlobalScope;
use dom_struct::dom_struct;
use servo_media::audio::node::AudioNodeInit;
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct AudioDestinationNode<TH: TypeHolderTrait> {
    node: AudioNode<TH>,
}

impl<TH: TypeHolderTrait> AudioDestinationNode<TH> {
    fn new_inherited(
        context: &BaseAudioContext<TH>,
        options: &AudioNodeOptions,
    ) -> AudioDestinationNode<TH> {
        AudioDestinationNode {
            node: AudioNode::new_inherited(
                AudioNodeInit::DestinationNode,
                Some(context.destination_node()),
                context,
                options,
                1,
                1,
            ),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(
        global: &GlobalScope<TH>,
        context: &BaseAudioContext<TH>,
        options: &AudioNodeOptions,
    ) -> DomRoot<AudioDestinationNode<TH>> {
        let node = AudioDestinationNode::new_inherited(context, options);
        reflect_dom_object(Box::new(node), global, AudioDestinationNodeBinding::Wrap)
    }
}

impl<TH: TypeHolderTrait> AudioDestinationNodeMethods for AudioDestinationNode<TH> {
    // https://webaudio.github.io/web-audio-api/#dom-audiodestinationnode-maxchannelcount
    fn MaxChannelCount(&self) -> u32 {
        MAX_CHANNEL_COUNT
    }
}
