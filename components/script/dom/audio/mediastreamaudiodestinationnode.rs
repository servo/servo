/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;
use servo_media::ServoMedia;
use servo_media::audio::node::AudioNodeInit;
use servo_media::streams::MediaStreamType;

use crate::dom::audio::audiocontext::AudioContext;
use crate::dom::audio::audionode::{AudioNode, AudioNodeOptionsHelper};
use crate::dom::bindings::codegen::Bindings::AudioNodeBinding::{
    AudioNodeOptions, ChannelCountMode, ChannelInterpretation,
};
use crate::dom::bindings::codegen::Bindings::MediaStreamAudioDestinationNodeBinding::MediaStreamAudioDestinationNodeMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{DomGlobal, reflect_dom_object_with_proto_and_cx};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::mediastream::MediaStream;
use crate::dom::window::Window;

#[dom_struct]
pub(crate) struct MediaStreamAudioDestinationNode {
    node: AudioNode,
    stream: Dom<MediaStream>,
}

impl MediaStreamAudioDestinationNode {
    #[cfg_attr(crown, expect(crown::unrooted_must_root))]
    pub(crate) fn new_inherited(
        cx: &mut js::context::JSContext,
        context: &AudioContext,
        options: &AudioNodeOptions,
    ) -> Fallible<MediaStreamAudioDestinationNode> {
        let media = ServoMedia::get();
        let (socket, id) = media.create_stream_and_socket(MediaStreamType::Audio);
        let stream = MediaStream::new_single(cx, &context.global(), id, MediaStreamType::Audio);
        let node_options = options.unwrap_or(
            2,
            ChannelCountMode::Explicit,
            ChannelInterpretation::Speakers,
        );
        let node = AudioNode::new_inherited(
            AudioNodeInit::MediaStreamDestinationNode(socket),
            context.upcast(),
            node_options,
            1, // inputs
            0, // outputs
        )?;
        Ok(MediaStreamAudioDestinationNode {
            node,
            stream: Dom::from_ref(&stream),
        })
    }

    pub(crate) fn new(
        cx: &mut js::context::JSContext,
        window: &Window,
        context: &AudioContext,
        options: &AudioNodeOptions,
    ) -> Fallible<DomRoot<MediaStreamAudioDestinationNode>> {
        Self::new_with_proto(cx, window, None, context, options)
    }

    #[cfg_attr(crown, expect(crown::unrooted_must_root))]
    fn new_with_proto(
        cx: &mut js::context::JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        context: &AudioContext,
        options: &AudioNodeOptions,
    ) -> Fallible<DomRoot<MediaStreamAudioDestinationNode>> {
        let node = MediaStreamAudioDestinationNode::new_inherited(cx, context, options)?;
        Ok(reflect_dom_object_with_proto_and_cx(
            Box::new(node),
            window,
            proto,
            cx,
        ))
    }
}

impl MediaStreamAudioDestinationNodeMethods<crate::DomTypeHolder>
    for MediaStreamAudioDestinationNode
{
    /// <https://webaudio.github.io/web-audio-api/#dom-mediastreamaudiodestinationnode-mediastreamaudiodestinationnode>
    fn Constructor(
        cx: &mut js::context::JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        context: &AudioContext,
        options: &AudioNodeOptions,
    ) -> Fallible<DomRoot<MediaStreamAudioDestinationNode>> {
        MediaStreamAudioDestinationNode::new_with_proto(cx, window, proto, context, options)
    }

    /// <https://webaudio.github.io/web-audio-api/#dom-mediastreamaudiodestinationnode-stream>
    fn Stream(&self) -> DomRoot<MediaStream> {
        DomRoot::from_ref(&self.stream)
    }
}
