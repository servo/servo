/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;
use servo_media::audio::node::AudioNodeInit;

use crate::dom::audionode::{AudioNode, AudioNodeOptionsHelper, MAX_CHANNEL_COUNT};
use crate::dom::baseaudiocontext::BaseAudioContext;
use crate::dom::bindings::codegen::Bindings::AudioNodeBinding::{
    ChannelCountMode, ChannelInterpretation,
};
use crate::dom::bindings::codegen::Bindings::ChannelSplitterNodeBinding::{
    ChannelSplitterNodeMethods, ChannelSplitterOptions,
};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::bindings::root::DomRoot;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct ChannelSplitterNode {
    node: AudioNode,
}

impl ChannelSplitterNode {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new_inherited(
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

    pub(crate) fn new(
        window: &Window,
        context: &BaseAudioContext,
        options: &ChannelSplitterOptions,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<ChannelSplitterNode>> {
        Self::new_with_proto(window, None, context, options, can_gc)
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn new_with_proto(
        window: &Window,
        proto: Option<HandleObject>,
        context: &BaseAudioContext,
        options: &ChannelSplitterOptions,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<ChannelSplitterNode>> {
        let node = ChannelSplitterNode::new_inherited(window, context, options)?;
        Ok(reflect_dom_object_with_proto(
            Box::new(node),
            window,
            proto,
            can_gc,
        ))
    }
}

impl ChannelSplitterNodeMethods<crate::DomTypeHolder> for ChannelSplitterNode {
    /// <https://webaudio.github.io/web-audio-api/#dom-channelsplitternode-channelsplitternode>
    fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        context: &BaseAudioContext,
        options: &ChannelSplitterOptions,
    ) -> Fallible<DomRoot<ChannelSplitterNode>> {
        ChannelSplitterNode::new_with_proto(window, proto, context, options, can_gc)
    }
}
