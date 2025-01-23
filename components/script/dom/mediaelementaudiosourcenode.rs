/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::mpsc;

use dom_struct::dom_struct;
use js::rust::HandleObject;
use servo_media::audio::media_element_source_node::MediaElementSourceNodeMessage;
use servo_media::audio::node::{AudioNodeInit, AudioNodeMessage};

use crate::dom::audiocontext::AudioContext;
use crate::dom::audionode::AudioNode;
use crate::dom::bindings::codegen::Bindings::MediaElementAudioSourceNodeBinding::{
    MediaElementAudioSourceNodeMethods, MediaElementAudioSourceOptions,
};
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::htmlmediaelement::HTMLMediaElement;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct MediaElementAudioSourceNode {
    node: AudioNode,
    media_element: Dom<HTMLMediaElement>,
}

impl MediaElementAudioSourceNode {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn new_inherited(
        context: &AudioContext,
        media_element: &HTMLMediaElement,
        can_gc: CanGc,
    ) -> Fallible<MediaElementAudioSourceNode> {
        let node = AudioNode::new_inherited(
            AudioNodeInit::MediaElementSourceNode,
            &context.base(),
            Default::default(),
            0,
            1,
        )?;
        let (sender, receiver) = mpsc::channel();
        node.message(AudioNodeMessage::MediaElementSourceNode(
            MediaElementSourceNodeMessage::GetAudioRenderer(sender),
        ));
        let audio_renderer = receiver.recv().unwrap();
        media_element.set_audio_renderer(audio_renderer, can_gc);
        let media_element = Dom::from_ref(media_element);
        Ok(MediaElementAudioSourceNode {
            node,
            media_element,
        })
    }

    pub(crate) fn new(
        window: &Window,
        context: &AudioContext,
        media_element: &HTMLMediaElement,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<MediaElementAudioSourceNode>> {
        Self::new_with_proto(window, None, context, media_element, can_gc)
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn new_with_proto(
        window: &Window,
        proto: Option<HandleObject>,
        context: &AudioContext,
        media_element: &HTMLMediaElement,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<MediaElementAudioSourceNode>> {
        let node = MediaElementAudioSourceNode::new_inherited(context, media_element, can_gc)?;
        Ok(reflect_dom_object_with_proto(
            Box::new(node),
            window,
            proto,
            can_gc,
        ))
    }
}

impl MediaElementAudioSourceNodeMethods<crate::DomTypeHolder> for MediaElementAudioSourceNode {
    /// <https://webaudio.github.io/web-audio-api/#dom-mediaelementaudiosourcenode-mediaelementaudiosourcenode>
    fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        context: &AudioContext,
        options: &MediaElementAudioSourceOptions,
    ) -> Fallible<DomRoot<MediaElementAudioSourceNode>> {
        MediaElementAudioSourceNode::new_with_proto(
            window,
            proto,
            context,
            &options.mediaElement,
            can_gc,
        )
    }

    /// <https://webaudio.github.io/web-audio-api/#dom-mediaelementaudiosourcenode-mediaelement>
    fn MediaElement(&self) -> DomRoot<HTMLMediaElement> {
        DomRoot::from_ref(&*self.media_element)
    }
}
