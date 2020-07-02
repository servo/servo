/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::audiocontext::AudioContext;
use crate::dom::audionode::AudioNode;
use crate::dom::bindings::codegen::Bindings::MediaStreamTrackAudioSourceNodeBinding::MediaStreamTrackAudioSourceOptions;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::mediastreamtrack::MediaStreamTrack;
use crate::dom::window::Window;
use dom_struct::dom_struct;
use servo_media::audio::node::AudioNodeInit;

#[dom_struct]
pub struct MediaStreamTrackAudioSourceNode {
    node: AudioNode,
    track: Dom<MediaStreamTrack>,
}

impl MediaStreamTrackAudioSourceNode {
    #[allow(unrooted_must_root)]
    pub fn new_inherited(
        context: &AudioContext,
        track: &MediaStreamTrack,
    ) -> Fallible<MediaStreamTrackAudioSourceNode> {
        let node = AudioNode::new_inherited(
            AudioNodeInit::MediaStreamSourceNode(track.id()),
            &context.upcast(),
            Default::default(),
            0, // inputs
            1, // outputs
        )?;
        Ok(MediaStreamTrackAudioSourceNode {
            node,
            track: Dom::from_ref(&track),
        })
    }

    #[allow(unrooted_must_root)]
    pub fn new(
        window: &Window,
        context: &AudioContext,
        track: &MediaStreamTrack,
    ) -> Fallible<DomRoot<MediaStreamTrackAudioSourceNode>> {
        let node = MediaStreamTrackAudioSourceNode::new_inherited(context, track)?;
        Ok(reflect_dom_object(Box::new(node), window))
    }

    #[allow(non_snake_case)]
    pub fn Constructor(
        window: &Window,
        context: &AudioContext,
        options: &MediaStreamTrackAudioSourceOptions,
    ) -> Fallible<DomRoot<MediaStreamTrackAudioSourceNode>> {
        MediaStreamTrackAudioSourceNode::new(window, context, &options.mediaStreamTrack)
    }
}
