/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;
use servo_media::audio::channel_node::ChannelNodeOptions;
use servo_media::audio::node::AudioNodeInit;

use crate::conversions::Convert;
use crate::dom::audionode::{AudioNode, AudioNodeOptionsHelper, MAX_CHANNEL_COUNT};
use crate::dom::baseaudiocontext::BaseAudioContext;
use crate::dom::bindings::codegen::Bindings::AudioNodeBinding::{
    ChannelCountMode, ChannelInterpretation,
};
use crate::dom::bindings::codegen::Bindings::ChannelMergerNodeBinding::{
    ChannelMergerNodeMethods, ChannelMergerOptions,
};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::bindings::root::DomRoot;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct ChannelMergerNode {
    node: AudioNode,
}

impl ChannelMergerNode {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new_inherited(
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

        let num_inputs = options.numberOfInputs;
        let node = AudioNode::new_inherited(
            AudioNodeInit::ChannelMergerNode(options.convert()),
            context,
            node_options,
            num_inputs, // inputs
            1,          // outputs
        )?;
        Ok(ChannelMergerNode { node })
    }

    pub(crate) fn new(
        window: &Window,
        context: &BaseAudioContext,
        options: &ChannelMergerOptions,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<ChannelMergerNode>> {
        Self::new_with_proto(window, None, context, options, can_gc)
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn new_with_proto(
        window: &Window,
        proto: Option<HandleObject>,
        context: &BaseAudioContext,
        options: &ChannelMergerOptions,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<ChannelMergerNode>> {
        let node = ChannelMergerNode::new_inherited(window, context, options)?;
        Ok(reflect_dom_object_with_proto(
            Box::new(node),
            window,
            proto,
            can_gc,
        ))
    }
}

impl ChannelMergerNodeMethods<crate::DomTypeHolder> for ChannelMergerNode {
    /// <https://webaudio.github.io/web-audio-api/#dom-channelmergernode-channelmergernode>
    fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        context: &BaseAudioContext,
        options: &ChannelMergerOptions,
    ) -> Fallible<DomRoot<ChannelMergerNode>> {
        ChannelMergerNode::new_with_proto(window, proto, context, options, can_gc)
    }
}

impl Convert<ChannelNodeOptions> for ChannelMergerOptions {
    fn convert(self) -> ChannelNodeOptions {
        ChannelNodeOptions {
            channels: self.numberOfInputs as u8,
        }
    }
}
