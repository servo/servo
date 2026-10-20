/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use js::rust::HandleObject;
use script_bindings::reflector::reflect_dom_object_with_proto;
use servo_media::audio::audio_node::AudioNodeInit;

use crate::dom::audio::audiocontext::AudioContext;
use crate::dom::audio::audionode::AudioNode;
use crate::dom::audionode::UnwrappedAudioNodeOptions;
use crate::dom::bindings::codegen::Bindings::MediaStreamTrackAudioSourceNodeBinding::{
    MediaStreamTrackAudioSourceNodeMethods, MediaStreamTrackAudioSourceOptions,
};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::mediastreamtrack::MediaStreamTrack;
use crate::dom::window::Window;

#[dom_struct]
pub(crate) struct MediaStreamTrackAudioSourceNode {
    node: AudioNode,
    track: Dom<MediaStreamTrack>,
}

impl MediaStreamTrackAudioSourceNodeMethods<crate::DomTypeHolder>
    for MediaStreamTrackAudioSourceNode
{
    /// <https://webaudio.github.io/web-audio-api/#dom-mediastreamtrackaudiosourcenode-mediastreamtrackaudiosourcenode>
    fn Constructor(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        context: &AudioContext,
        options: &MediaStreamTrackAudioSourceOptions,
    ) -> Fallible<DomRoot<MediaStreamTrackAudioSourceNode>> {
        let node_options: UnwrappedAudioNodeOptions = Default::default();
        if AudioNode::channel_count_check(node_options.count) {
            return Err(Error::NotSupported(Some(String::from(
                "Channel count not supported",
            ))));
        }
        let track = &(*options.mediaStreamTrack);
        let audio_node_init = AudioNodeInit::MediaStreamSourceNode(track.id());
        let node_id =
            AudioNode::generate_audio_node_id(&node_options, audio_node_init, context.upcast());
        AudioNode::inert_node_warning(cx, context.upcast(), node_id);
        let node = AudioNode::new_inherited(
            node_id,
            context.upcast(),
            node_options,
            0, // inputs
            1, // outputs
        );
        Ok(reflect_dom_object_with_proto(
            cx,
            Box::new(MediaStreamTrackAudioSourceNode {
                node,
                track: Dom::from_ref(track),
            }),
            window,
            proto,
        ))
    }
}
