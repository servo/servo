/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use js::rust::HandleObject;
use script_bindings::reflector::reflect_dom_object_with_proto;
use servo_media::audio::audio_node::AudioNodeInit;
use servo_media::audio::graph::NodeId;
use servo_media::streams::MediaStreamType;

use crate::dom::audio::audiocontext::AudioContext;
use crate::dom::audio::audionode::AudioNode;
use crate::dom::audio::baseaudiocontext::BaseAudioContext;
use crate::dom::audionode::UnwrappedAudioNodeOptions;
use crate::dom::bindings::codegen::Bindings::MediaStreamAudioSourceNodeBinding::{
    MediaStreamAudioSourceNodeMethods, MediaStreamAudioSourceOptions,
};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::mediastream::MediaStream;
use crate::dom::window::Window;

#[dom_struct]
pub(crate) struct MediaStreamAudioSourceNode {
    node: AudioNode,
    stream: Dom<MediaStream>,
}

impl MediaStreamAudioSourceNode {
    fn new_inherited(
        node_id: Option<NodeId>,
        context: &BaseAudioContext,
        node_options: UnwrappedAudioNodeOptions,
        stream: &MediaStream,
    ) -> Self {
        MediaStreamAudioSourceNode {
            node: AudioNode::new_inherited(
                node_id,
                context,
                node_options,
                0, // inputs
                1, // outputs
            ),
            stream: Dom::from_ref(stream),
        }
    }
}

impl MediaStreamAudioSourceNodeMethods<crate::DomTypeHolder> for MediaStreamAudioSourceNode {
    /// <https://webaudio.github.io/web-audio-api/#dom-mediastreamaudiosourcenode-mediastreamaudiosourcenode>
    fn Constructor(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        context: &AudioContext,
        options: &MediaStreamAudioSourceOptions,
    ) -> Fallible<DomRoot<MediaStreamAudioSourceNode>> {
        let node_options: UnwrappedAudioNodeOptions = Default::default();
        if AudioNode::channel_count_check(node_options.count) {
            return Err(Error::NotSupported(Some(String::from(
                "Channel count not supported",
            ))));
        }
        let track = options
            .mediaStream
            .get_tracks()
            .iter()
            .find(|t| t.ty() == MediaStreamType::Audio)
            .ok_or(Error::InvalidState(None))?
            .id();
        let audio_node_init = AudioNodeInit::MediaStreamSourceNode(track);
        let node_id =
            AudioNode::generate_audio_node_id(cx, &node_options, audio_node_init, context.upcast());
        Ok(reflect_dom_object_with_proto(
            cx,
            Box::new(MediaStreamAudioSourceNode::new_inherited(
                node_id,
                context.upcast(),
                node_options,
                &options.mediaStream,
            )),
            window,
            proto,
        ))
    }

    /// <https://webaudio.github.io/web-audio-api/#dom-MediaStreamAudioSourceNode-stream>
    fn MediaStream(&self) -> DomRoot<MediaStream> {
        DomRoot::from_ref(&self.stream)
    }
}
