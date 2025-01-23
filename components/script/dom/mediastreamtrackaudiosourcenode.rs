/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;
use servo_media::audio::node::AudioNodeInit;

use crate::dom::audiocontext::AudioContext;
use crate::dom::audionode::AudioNode;
use crate::dom::bindings::codegen::Bindings::MediaStreamTrackAudioSourceNodeBinding::{
    MediaStreamTrackAudioSourceNodeMethods, MediaStreamTrackAudioSourceOptions,
};
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::mediastreamtrack::MediaStreamTrack;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct MediaStreamTrackAudioSourceNode {
    node: AudioNode,
    track: Dom<MediaStreamTrack>,
}

impl MediaStreamTrackAudioSourceNode {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new_inherited(
        context: &AudioContext,
        track: &MediaStreamTrack,
    ) -> Fallible<MediaStreamTrackAudioSourceNode> {
        let node = AudioNode::new_inherited(
            AudioNodeInit::MediaStreamSourceNode(track.id()),
            context.upcast(),
            Default::default(),
            0, // inputs
            1, // outputs
        )?;
        Ok(MediaStreamTrackAudioSourceNode {
            node,
            track: Dom::from_ref(track),
        })
    }

    pub(crate) fn new(
        window: &Window,
        context: &AudioContext,
        track: &MediaStreamTrack,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<MediaStreamTrackAudioSourceNode>> {
        Self::new_with_proto(window, None, context, track, can_gc)
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn new_with_proto(
        window: &Window,
        proto: Option<HandleObject>,
        context: &AudioContext,
        track: &MediaStreamTrack,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<MediaStreamTrackAudioSourceNode>> {
        let node = MediaStreamTrackAudioSourceNode::new_inherited(context, track)?;
        Ok(reflect_dom_object_with_proto(
            Box::new(node),
            window,
            proto,
            can_gc,
        ))
    }
}

impl MediaStreamTrackAudioSourceNodeMethods<crate::DomTypeHolder>
    for MediaStreamTrackAudioSourceNode
{
    /// <https://webaudio.github.io/web-audio-api/#dom-mediastreamtrackaudiosourcenode-mediastreamtrackaudiosourcenode>
    fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        context: &AudioContext,
        options: &MediaStreamTrackAudioSourceOptions,
    ) -> Fallible<DomRoot<MediaStreamTrackAudioSourceNode>> {
        MediaStreamTrackAudioSourceNode::new_with_proto(
            window,
            proto,
            context,
            &options.mediaStreamTrack,
            can_gc,
        )
    }
}
