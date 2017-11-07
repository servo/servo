/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::audionode::{AudioNode, MAX_CHANNEL_COUNT};
use dom::baseaudiocontext::BaseAudioContext;
use dom::bindings::codegen::Bindings::AudioDestinationNodeBinding;
use dom::bindings::codegen::Bindings::AudioDestinationNodeBinding::AudioDestinationNodeMethods;
use dom::bindings::codegen::Bindings::AudioNodeBinding::AudioNodeOptions;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::root::DomRoot;
use dom::globalscope::GlobalScope;
use dom_struct::dom_struct;

#[dom_struct]
pub struct AudioDestinationNode {
    node: AudioNode,
}

impl AudioDestinationNode {
    fn new_inherited(context: &BaseAudioContext,
                     options: &AudioNodeOptions) -> AudioDestinationNode {
        AudioDestinationNode {
            node: AudioNode::new_inherited(context, options, 1, 1),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(global: &GlobalScope,
               context: &BaseAudioContext,
               options: &AudioNodeOptions) -> DomRoot<AudioDestinationNode> {
        let node = AudioDestinationNode::new_inherited(context, options);
        reflect_dom_object(Box::new(node), global, AudioDestinationNodeBinding::Wrap)
    }
}

impl AudioDestinationNodeMethods for AudioDestinationNode {
    fn MaxChannelCount(&self) -> u32 {
        MAX_CHANNEL_COUNT
    }
}
