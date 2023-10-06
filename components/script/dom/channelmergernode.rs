/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;
use servo_media::audio::channel_node::ChannelNodeOptions;
use servo_media::audio::node::AudioNodeInit;

use crate::dom::audionode::{AudioNode, MAX_CHANNEL_COUNT};
use crate::dom::baseaudiocontext::BaseAudioContext;
use crate::dom::bindings::codegen::Bindings::AudioNodeBinding::{
    ChannelCountMode, ChannelInterpretation,
};
use crate::dom::bindings::codegen::Bindings::ChannelMergerNodeBinding::ChannelMergerOptions;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::bindings::root::DomRoot;
use crate::dom::window::Window;

#[dom_struct]
pub struct ChannelMergerNode {
    node: AudioNode,
}

impl ChannelMergerNode {
    #[allow(crown::unrooted_must_root)]
    pub fn new_inherited(
        _: &Window,
        context: &BaseAudioContext,
        options: &ChannelMergerOptions,
    ) -> Fallible<ChannelMergerNode> {
        let node_options = options.parent.unwrap_or(
            1,
            ChannelCountMode::Explicit,
            ChannelInterpretation::Speakers,
        );

        if node_options.count != 1 || node_options.mode != ChannelCountMode::Explicit {
            return Err(Error::InvalidState);
        }

        if options.numberOfInputs < 1 || options.numberOfInputs > MAX_CHANNEL_COUNT {
            return Err(Error::IndexSize);
        }

        let node = AudioNode::new_inherited(
            AudioNodeInit::ChannelMergerNode(options.into()),
            context,
            node_options,
            options.numberOfInputs, // inputs
            1,                      // outputs
        )?;
        Ok(ChannelMergerNode { node })
    }

    pub fn new(
        window: &Window,
        context: &BaseAudioContext,
        options: &ChannelMergerOptions,
    ) -> Fallible<DomRoot<ChannelMergerNode>> {
        Self::new_with_proto(window, None, context, options)
    }

    #[allow(crown::unrooted_must_root)]
    fn new_with_proto(
        window: &Window,
        proto: Option<HandleObject>,
        context: &BaseAudioContext,
        options: &ChannelMergerOptions,
    ) -> Fallible<DomRoot<ChannelMergerNode>> {
        let node = ChannelMergerNode::new_inherited(window, context, options)?;
        Ok(reflect_dom_object_with_proto(Box::new(node), window, proto))
    }

    #[allow(non_snake_case)]
    pub fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        context: &BaseAudioContext,
        options: &ChannelMergerOptions,
    ) -> Fallible<DomRoot<ChannelMergerNode>> {
        ChannelMergerNode::new_with_proto(window, proto, context, options)
    }
}

impl<'a> From<&'a ChannelMergerOptions> for ChannelNodeOptions {
    fn from(options: &'a ChannelMergerOptions) -> Self {
        Self {
            channels: options.numberOfInputs as u8,
        }
    }
}
