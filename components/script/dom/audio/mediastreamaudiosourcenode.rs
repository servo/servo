/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;
use servo_media::audio::node::AudioNodeInit;
use servo_media::streams::MediaStreamType;

use crate::dom::audio::audiocontext::AudioContext;
use crate::dom::audio::audionode::AudioNode;
use crate::dom::bindings::codegen::Bindings::MediaStreamAudioSourceNodeBinding::{
    MediaStreamAudioSourceNodeMethods, MediaStreamAudioSourceOptions,
};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::reflect_dom_object_with_proto_and_cx;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::mediastream::MediaStream;
use crate::dom::window::Window;

#[dom_struct]
pub(crate) struct MediaStreamAudioSourceNode {
    node: AudioNode,
    stream: Dom<MediaStream>,
}

impl MediaStreamAudioSourceNode {
    #[cfg_attr(crown, expect(crown::unrooted_must_root))]
    pub(crate) fn new_inherited(
        context: &AudioContext,
        stream: &MediaStream,
    ) -> Fallible<MediaStreamAudioSourceNode> {
        let track = stream
            .get_tracks()
            .iter()
            .find(|t| t.ty() == MediaStreamType::Audio)
            .ok_or(Error::InvalidState(None))?
            .id();
        let node = AudioNode::new_inherited(
            AudioNodeInit::MediaStreamSourceNode(track),
            context.upcast(),
            Default::default(),
            0, // inputs
            1, // outputs
        )?;
        Ok(MediaStreamAudioSourceNode {
            node,
            stream: Dom::from_ref(stream),
        })
    }

    pub(crate) fn new(
        cx: &mut js::context::JSContext,
        window: &Window,
        context: &AudioContext,
        stream: &MediaStream,
    ) -> Fallible<DomRoot<MediaStreamAudioSourceNode>> {
        Self::new_with_proto(cx, window, None, context, stream)
    }

    #[cfg_attr(crown, expect(crown::unrooted_must_root))]
    fn new_with_proto(
        cx: &mut js::context::JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        context: &AudioContext,
        stream: &MediaStream,
    ) -> Fallible<DomRoot<MediaStreamAudioSourceNode>> {
        let node = MediaStreamAudioSourceNode::new_inherited(context, stream)?;
        Ok(reflect_dom_object_with_proto_and_cx(
            Box::new(node),
            window,
            proto,
            cx,
        ))
    }
}

impl MediaStreamAudioSourceNodeMethods<crate::DomTypeHolder> for MediaStreamAudioSourceNode {
    /// <https://webaudio.github.io/web-audio-api/#dom-mediastreamaudiosourcenode-mediastreamaudiosourcenode>
    fn Constructor(
        cx: &mut js::context::JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        context: &AudioContext,
        options: &MediaStreamAudioSourceOptions,
    ) -> Fallible<DomRoot<MediaStreamAudioSourceNode>> {
        MediaStreamAudioSourceNode::new_with_proto(cx, window, proto, context, &options.mediaStream)
    }

    /// <https://webaudio.github.io/web-audio-api/#dom-MediaStreamAudioSourceNode-stream>
    fn MediaStream(&self) -> DomRoot<MediaStream> {
        DomRoot::from_ref(&self.stream)
    }
}
