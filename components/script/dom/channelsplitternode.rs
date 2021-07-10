/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::audionode::{AudioNode, MAX_CHANNEL_COUNT};
use crate::dom::baseaudiocontext::BaseAudioContext;
use crate::dom::bindings::codegen::Bindings::AudioNodeBinding::{
    ChannelCountMode, ChannelInterpretation,
};
use crate::dom::bindings::codegen::Bindings::ChannelSplitterNodeBinding::ChannelSplitterOptions;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::DomRoot;
use crate::dom::window::Window;
use dom_struct::dom_struct;
use servo_media::audio::node::AudioNodeInit;

#[dom_struct]
pub struct ChannelSplitterNode {
    node: AudioNode,
}

impl ChannelSplitterNode {
    #[allow(unrooted_must_root)]
    pub fn new_inherited(
        _: &Window,
        context: &BaseAudioContext,
        options: &ChannelSplitterOptions,
    ) -> Fallible<ChannelSplitterNode> {
        if options.numberOfOutputs < 1 || options.numberOfOutputs > MAX_CHANNEL_COUNT {
            return Err(Error::IndexSize);
        }

        let node_options = options.parent.unwrap_or(
            options.numberOfOutputs,
            ChannelCountMode::Explicit,
            ChannelInterpretation::Discrete,
        );

        if node_options.count != options.numberOfOutputs ||
            node_options.mode != ChannelCountMode::Explicit ||
            node_options.interpretation != ChannelInterpretation::Discrete
        {
            return Err(Error::InvalidState);
        }

        let node = AudioNode::new_inherited(
            AudioNodeInit::ChannelSplitterNode,
            context,
            node_options,
            1,                       // inputs
            options.numberOfOutputs, // outputs
        )?;
        Ok(ChannelSplitterNode { node })
    }

    #[allow(unrooted_must_root)]
    pub fn new(
        window: &Window,
        context: &BaseAudioContext,
        options: &ChannelSplitterOptions,
    ) -> Fallible<DomRoot<ChannelSplitterNode>> {
        let node = ChannelSplitterNode::new_inherited(window, context, options)?;
        Ok(reflect_dom_object(Box::new(node), window))
    }

    #[allow(non_snake_case)]
    pub fn Constructor(
        window: &Window,
        context: &BaseAudioContext,
        options: &ChannelSplitterOptions,
    ) -> Fallible<DomRoot<ChannelSplitterNode>> {
        ChannelSplitterNode::new(window, context, options)
    }
}
