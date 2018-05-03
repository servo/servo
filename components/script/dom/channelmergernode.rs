/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::audionode::{AudioNode, MAX_CHANNEL_COUNT};
use dom::baseaudiocontext::BaseAudioContext;
use dom::bindings::codegen::Bindings::AudioNodeBinding::{ChannelCountMode, ChannelInterpretation};
use dom::bindings::codegen::Bindings::ChannelMergerNodeBinding::{self, ChannelMergerOptions};
use dom::bindings::error::{Error, Fallible};
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::root::DomRoot;
use dom::window::Window;
use dom_struct::dom_struct;
use servo_media::audio::channel_node::ChannelNodeOptions;
use servo_media::audio::node::AudioNodeInit;
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct ChannelMergerNode<TH: TypeHolderTrait> {
    node: AudioNode<TH>,
}

impl<TH: TypeHolderTrait> ChannelMergerNode<TH> {
    #[allow(unrooted_must_root)]
    pub fn new_inherited(
        _: &Window<TH>,
        context: &BaseAudioContext<TH>,
        options: &ChannelMergerOptions,
    ) -> Fallible<ChannelMergerNode<TH>> {
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

    #[allow(unrooted_must_root)]
    pub fn new(
        window: &Window<TH>,
        context: &BaseAudioContext<TH>,
        options: &ChannelMergerOptions,
    ) -> Fallible<DomRoot<ChannelMergerNode<TH>>> {
        let node = ChannelMergerNode::new_inherited(window, context, options)?;
        Ok(reflect_dom_object(
            Box::new(node),
            window,
            ChannelMergerNodeBinding::Wrap,
        ))
    }

    pub fn Constructor(
        window: &Window<TH>,
        context: &BaseAudioContext<TH>,
        options: &ChannelMergerOptions,
    ) -> Fallible<DomRoot<ChannelMergerNode<TH>>> {
        ChannelMergerNode::new(window, context, options)
    }
}

impl<'a> From<&'a ChannelMergerOptions> for ChannelNodeOptions {
    fn from(options: &'a ChannelMergerOptions) -> Self {
        Self {
            channels: options.numberOfInputs as u8,
        }
    }
}
