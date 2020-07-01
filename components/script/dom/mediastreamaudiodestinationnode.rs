/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::audiocontext::AudioContext;
use crate::dom::audionode::AudioNode;
use crate::dom::bindings::codegen::Bindings::AudioNodeBinding::AudioNodeOptions;
use crate::dom::bindings::codegen::Bindings::AudioNodeBinding::{
    ChannelCountMode, ChannelInterpretation,
};
use crate::dom::bindings::codegen::Bindings::MediaStreamAudioDestinationNodeBinding::MediaStreamAudioDestinationNodeMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::mediastream::MediaStream;
use crate::dom::window::Window;
use dom_struct::dom_struct;
use servo_media::audio::node::AudioNodeInit;
use servo_media::streams::MediaStreamType;
use servo_media::ServoMedia;

#[dom_struct]
pub struct MediaStreamAudioDestinationNode {
    node: AudioNode,
    stream: Dom<MediaStream>,
}

impl MediaStreamAudioDestinationNode {
    #[allow(unrooted_must_root)]
    pub fn new_inherited(
        context: &AudioContext,
        options: &AudioNodeOptions,
    ) -> Fallible<MediaStreamAudioDestinationNode> {
        let media = ServoMedia::get().unwrap();
        let (socket, id) = media.create_stream_and_socket(MediaStreamType::Audio);
        let stream = MediaStream::new_single(&context.global(), id, MediaStreamType::Audio);
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
        Ok(MediaStreamAudioDestinationNode {
            node,
            stream: Dom::from_ref(&stream),
        })
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

impl MediaStreamAudioDestinationNodeMethods for MediaStreamAudioDestinationNode {
    /// https://webaudio.github.io/web-audio-api/#dom-mediastreamaudiodestinationnode-stream
    fn Stream(&self) -> DomRoot<MediaStream> {
        DomRoot::from_ref(&self.stream)
    }
}
