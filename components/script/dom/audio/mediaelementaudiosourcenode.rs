/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::mpsc;

use dom_struct::dom_struct;
use js::context::JSContext;
use js::rust::HandleObject;
use script_bindings::reflector::reflect_dom_object_with_proto;
use servo_media::audio::audio_node::{AudioNodeInit, AudioNodeMessage};
use servo_media::audio::graph::NodeId;
use servo_media::audio::media_element_source_node::MediaElementSourceNodeMessage;

use crate::dom::audio::audiocontext::AudioContext;
use crate::dom::audio::audionode::AudioNode;
use crate::dom::audio::baseaudiocontext::BaseAudioContext;
use crate::dom::audionode::UnwrappedAudioNodeOptions;
use crate::dom::bindings::codegen::Bindings::MediaElementAudioSourceNodeBinding::{
    MediaElementAudioSourceNodeMethods, MediaElementAudioSourceOptions,
};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::html::htmlmediaelement::HTMLMediaElement;
use crate::dom::window::Window;

#[dom_struct]
pub(crate) struct MediaElementAudioSourceNode {
    node: AudioNode,
    media_element: Dom<HTMLMediaElement>,
}

impl MediaElementAudioSourceNode {
    fn new_inherited(
        node_id: Option<NodeId>,
        context: &BaseAudioContext,
        node_options: UnwrappedAudioNodeOptions,
        media_element: &HTMLMediaElement,
    ) -> Self {
        MediaElementAudioSourceNode {
            node: AudioNode::new_inherited(node_id, context, node_options, 0, 1),
            media_element: Dom::from_ref(media_element),
        }
    }
}

impl MediaElementAudioSourceNodeMethods<crate::DomTypeHolder> for MediaElementAudioSourceNode {
    /// <https://webaudio.github.io/web-audio-api/#dom-mediaelementaudiosourcenode-mediaelementaudiosourcenode>
    fn Constructor(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        context: &AudioContext,
        options: &MediaElementAudioSourceOptions,
    ) -> Fallible<DomRoot<MediaElementAudioSourceNode>> {
        let node_options: UnwrappedAudioNodeOptions = Default::default();
        if AudioNode::channel_count_check(node_options.count) {
            return Err(Error::NotSupported(Some(String::from(
                "Channel count not supported",
            ))));
        }
        let context = context.base();
        let node_id = AudioNode::generate_audio_node_id(
            cx,
            &node_options,
            AudioNodeInit::MediaElementSourceNode,
            &context,
        );

        let node = reflect_dom_object_with_proto(
            cx,
            Box::new(MediaElementAudioSourceNode::new_inherited(
                node_id,
                &context,
                node_options,
                &options.mediaElement,
            )),
            window,
            proto,
        );
        let (sender, receiver) = mpsc::channel();
        node.node.message(AudioNodeMessage::MediaElementSourceNode(
            MediaElementSourceNodeMessage::GetAudioRenderer(sender),
        ));
        let audio_renderer = receiver.recv();
        node.media_element
            .set_audio_renderer(cx, audio_renderer.ok());

        Ok(node)
    }

    /// <https://webaudio.github.io/web-audio-api/#dom-mediaelementaudiosourcenode-mediaelement>
    fn MediaElement(&self) -> DomRoot<HTMLMediaElement> {
        DomRoot::from_ref(&*self.media_element)
    }
}
