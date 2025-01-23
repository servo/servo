/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;
use servo_media::audio::node::AudioNodeInit;
use servo_media::streams::MediaStreamType;

use crate::dom::audiocontext::AudioContext;
use crate::dom::audionode::AudioNode;
use crate::dom::bindings::codegen::Bindings::MediaStreamAudioSourceNodeBinding::{
    MediaStreamAudioSourceNodeMethods, MediaStreamAudioSourceOptions,
};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::mediastream::MediaStream;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct MediaStreamAudioSourceNode {
    node: AudioNode,
    stream: Dom<MediaStream>,
}

impl MediaStreamAudioSourceNode {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new_inherited(
        context: &AudioContext,
        stream: &MediaStream,
    ) -> Fallible<MediaStreamAudioSourceNode> {
        let track = stream
            .get_tracks()
            .iter()
            .find(|t| t.ty() == MediaStreamType::Audio)
            .ok_or(Error::InvalidState)?
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
        window: &Window,
        context: &AudioContext,
        stream: &MediaStream,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<MediaStreamAudioSourceNode>> {
        Self::new_with_proto(window, None, context, stream, can_gc)
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn new_with_proto(
        window: &Window,
        proto: Option<HandleObject>,
        context: &AudioContext,
        stream: &MediaStream,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<MediaStreamAudioSourceNode>> {
        let node = MediaStreamAudioSourceNode::new_inherited(context, stream)?;
        Ok(reflect_dom_object_with_proto(
            Box::new(node),
            window,
            proto,
            can_gc,
        ))
    }
}

impl MediaStreamAudioSourceNodeMethods<crate::DomTypeHolder> for MediaStreamAudioSourceNode {
    /// <https://webaudio.github.io/web-audio-api/#dom-mediastreamaudiosourcenode-mediastreamaudiosourcenode>
    fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        context: &AudioContext,
        options: &MediaStreamAudioSourceOptions,
    ) -> Fallible<DomRoot<MediaStreamAudioSourceNode>> {
        MediaStreamAudioSourceNode::new_with_proto(
            window,
            proto,
            context,
            &options.mediaStream,
            can_gc,
        )
    }

    /// <https://webaudio.github.io/web-audio-api/#dom-MediaStreamAudioSourceNode-stream>
    fn MediaStream(&self) -> DomRoot<MediaStream> {
        DomRoot::from_ref(&self.stream)
    }
}
