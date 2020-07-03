/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::audiocontext::AudioContext;
use crate::dom::audionode::AudioNode;
use crate::dom::bindings::codegen::Bindings::MediaStreamAudioSourceNodeBinding::{
    MediaStreamAudioSourceNodeMethods, MediaStreamAudioSourceOptions,
};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::mediastream::MediaStream;
use crate::dom::window::Window;
use dom_struct::dom_struct;
use servo_media::audio::node::AudioNodeInit;
use servo_media::streams::MediaStreamType;

#[dom_struct]
pub struct MediaStreamAudioSourceNode {
    node: AudioNode,
    stream: Dom<MediaStream>,
}

impl MediaStreamAudioSourceNode {
    #[allow(unrooted_must_root)]
    pub fn new_inherited(
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
            &context.upcast(),
            Default::default(),
            0, // inputs
            1, // outputs
        )?;
        Ok(MediaStreamAudioSourceNode {
            node,
            stream: Dom::from_ref(&stream),
        })
    }

    #[allow(unrooted_must_root)]
    pub fn new(
        window: &Window,
        context: &AudioContext,
        stream: &MediaStream,
    ) -> Fallible<DomRoot<MediaStreamAudioSourceNode>> {
        let node = MediaStreamAudioSourceNode::new_inherited(context, stream)?;
        Ok(reflect_dom_object(Box::new(node), window))
    }

    #[allow(non_snake_case)]
    pub fn Constructor(
        window: &Window,
        context: &AudioContext,
        options: &MediaStreamAudioSourceOptions,
    ) -> Fallible<DomRoot<MediaStreamAudioSourceNode>> {
        MediaStreamAudioSourceNode::new(window, context, &options.mediaStream)
    }
}

impl MediaStreamAudioSourceNodeMethods for MediaStreamAudioSourceNode {
    /// https://webaudio.github.io/web-audio-api/#dom-MediaStreamAudioSourceNode-stream
    fn MediaStream(&self) -> DomRoot<MediaStream> {
        DomRoot::from_ref(&self.stream)
    }
}
