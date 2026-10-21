/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use js::rust::HandleObject;
use script_bindings::reflector::reflect_dom_object_with_proto;
use servo_media::ServoMedia;
use servo_media::audio::audio_node::AudioNodeInit;
use servo_media::audio::graph::NodeId;
use servo_media::streams::MediaStreamType;

use crate::dom::audio::audiocontext::AudioContext;
use crate::dom::audio::audionode::{AudioNode, AudioNodeOptionsHelper};
use crate::dom::audio::baseaudiocontext::BaseAudioContext;
use crate::dom::audionode::UnwrappedAudioNodeOptions;
use crate::dom::bindings::codegen::Bindings::AudioNodeBinding::{
    AudioNodeOptions, ChannelCountMode, ChannelInterpretation,
};
use crate::dom::bindings::codegen::Bindings::MediaStreamAudioDestinationNodeBinding::MediaStreamAudioDestinationNodeMethods;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::mediastream::MediaStream;
use crate::dom::window::Window;

#[dom_struct]
pub(crate) struct MediaStreamAudioDestinationNode {
    node: AudioNode,
    stream: Dom<MediaStream>,
}

impl MediaStreamAudioDestinationNode {
    fn new_inherited(
        node_id: Option<NodeId>,
        context: &BaseAudioContext,
        node_options: UnwrappedAudioNodeOptions,
        stream: &MediaStream,
    ) -> Self {
        MediaStreamAudioDestinationNode {
            node: AudioNode::new_inherited(
                node_id,
                context,
                node_options,
                1, // inputs
                0, // outputs
            ),
            stream: Dom::from_ref(stream),
        }
    }
}

impl MediaStreamAudioDestinationNodeMethods<crate::DomTypeHolder>
    for MediaStreamAudioDestinationNode
{
    /// <https://webaudio.github.io/web-audio-api/#dom-mediastreamaudiodestinationnode-mediastreamaudiodestinationnode>
    fn Constructor(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        context: &AudioContext,
        options: &AudioNodeOptions,
    ) -> Fallible<DomRoot<MediaStreamAudioDestinationNode>> {
        let media = ServoMedia::get();
        let (socket, id) = media.create_stream_and_socket(MediaStreamType::Audio);
        let stream = MediaStream::new_single(cx, &context.global(), id, MediaStreamType::Audio);
        let node_options = options.unwrap_or(
            2,
            ChannelCountMode::Explicit,
            ChannelInterpretation::Speakers,
        );
        if AudioNode::channel_count_check(node_options.count) {
            return Err(Error::NotSupported(Some(String::from(
                "Channel count not supported",
            ))));
        }
        let audio_node_init = AudioNodeInit::MediaStreamDestinationNode(socket);
        let node_id =
            AudioNode::generate_audio_node_id(cx, &node_options, audio_node_init, context.upcast());
        Ok(reflect_dom_object_with_proto(
            cx,
            Box::new(MediaStreamAudioDestinationNode::new_inherited(
                node_id,
                context.upcast(),
                node_options,
                &stream,
            )),
            window,
            proto,
        ))
    }

    /// <https://webaudio.github.io/web-audio-api/#dom-mediastreamaudiodestinationnode-stream>
    fn Stream(&self) -> DomRoot<MediaStream> {
        DomRoot::from_ref(&self.stream)
    }
}
