/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::audiocontext::AudioContext;
use crate::dom::audionode::AudioNode;
use crate::dom::bindings::codegen::Bindings::AudioNodeBinding::AudioNodeOptions;
use crate::dom::bindings::codegen::Bindings::AudioNodeBinding::{
    ChannelCountMode, ChannelInterpretation,
};
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::DomRoot;
use crate::dom::window::Window;
use dom_struct::dom_struct;
use servo_media::audio::node::AudioNodeInit;
use servo_media::streams::MediaStreamType;
use servo_media::ServoMedia;

#[dom_struct]
pub struct MediaStreamAudioDestinationNode {
    node: AudioNode,
}

impl MediaStreamAudioDestinationNode {
    #[allow(unrooted_must_root)]
    pub fn new_inherited(
        context: &AudioContext,
        options: &AudioNodeOptions,
    ) -> Fallible<MediaStreamAudioDestinationNode> {
        let media = ServoMedia::get().unwrap();
        let (socket, _id) = media.create_stream_and_socket(MediaStreamType::Audio);
        let node_options = options.unwrap_or(
            2,
            ChannelCountMode::Explicit,
            ChannelInterpretation::Speakers,
        );
        let node = AudioNode::new_inherited(
            AudioNodeInit::MediaStreamDestinationNode(socket),
            &context.upcast(),
            node_options,
            1, // inputs
            0, // outputs
        )?;
        Ok(MediaStreamAudioDestinationNode { node })
    }

    #[allow(unrooted_must_root)]
    pub fn new(
        window: &Window,
        context: &AudioContext,
        options: &AudioNodeOptions,
    ) -> Fallible<DomRoot<MediaStreamAudioDestinationNode>> {
        let node = MediaStreamAudioDestinationNode::new_inherited(context, options)?;
        Ok(reflect_dom_object(Box::new(node), window))
    }

    #[allow(non_snake_case)]
    pub fn Constructor(
        window: &Window,
        context: &AudioContext,
        options: &AudioNodeOptions,
    ) -> Fallible<DomRoot<MediaStreamAudioDestinationNode>> {
        MediaStreamAudioDestinationNode::new(window, context, options)
    }
}
