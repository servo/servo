/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use js::rust::HandleObject;
use script_bindings::reflector::reflect_dom_object_with_proto;
use servo_media::audio::audio_node::{AudioNodeInit, AudioNodeType};
use servo_media::audio::graph::NodeId;
use servo_media::audio::param::ParamType;
use servo_media::audio::stereo_panner::StereoPannerOptions as ServoMediaStereoPannerOptions;

use crate::conversions::Convert;
use crate::dom::audio::audionode::{AudioNode, AudioNodeOptionsHelper};
use crate::dom::audio::audioparam::AudioParam;
use crate::dom::audio::audioscheduledsourcenode::AudioScheduledSourceNode;
use crate::dom::audio::baseaudiocontext::BaseAudioContext;
use crate::dom::audionode::UnwrappedAudioNodeOptions;
use crate::dom::bindings::codegen::Bindings::AudioNodeBinding::{
    ChannelCountMode, ChannelInterpretation,
};
use crate::dom::bindings::codegen::Bindings::AudioParamBinding::AutomationRate;
use crate::dom::bindings::codegen::Bindings::StereoPannerNodeBinding::{
    StereoPannerNodeMethods, StereoPannerOptions,
};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::window::Window;

#[dom_struct]
pub(crate) struct StereoPannerNode {
    source_node: AudioScheduledSourceNode,
    pan: Dom<AudioParam>,
}

impl StereoPannerNode {
    fn new_inherited(
        node_id: Option<NodeId>,
        context: &BaseAudioContext,
        node_options: UnwrappedAudioNodeOptions,
        pan: &AudioParam,
    ) -> Self {
        StereoPannerNode {
            source_node: AudioScheduledSourceNode::new_inherited(
                node_id,
                context,
                node_options,
                1, /* inputs */
                1, /* outputs */
            ),
            pan: Dom::from_ref(pan),
        }
    }
}

impl StereoPannerNodeMethods<crate::DomTypeHolder> for StereoPannerNode {
    /// <https://webaudio.github.io/web-audio-api/#dom-stereopannernode-stereopannernode>
    fn Constructor(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        context: &BaseAudioContext,
        options: &StereoPannerOptions,
    ) -> Fallible<DomRoot<StereoPannerNode>> {
        let node_options = options.parent.unwrap_or(
            2,
            ChannelCountMode::Clamped_max,
            ChannelInterpretation::Speakers,
        );
        if AudioNode::channel_count_check(node_options.count) {
            return Err(Error::NotSupported(Some(String::from(
                "Channel count not supported",
            ))));
        }
        if node_options.mode == ChannelCountMode::Max {
            return Err(Error::NotSupported(None));
        }
        if node_options.count > 2 || node_options.count == 0 {
            return Err(Error::NotSupported(None));
        }
        let pan = *options.pan;
        let audio_node_init = AudioNodeInit::StereoPannerNode(options.convert());
        let node_id =
            AudioNode::generate_audio_node_id(cx, &node_options, audio_node_init, context);
        // <https://webaudio.github.io/web-audio-api/#dom-stereopannernode-pan>
        let pan = AudioParam::new(
            cx,
            window,
            context,
            node_id,
            AudioNodeType::StereoPannerNode,
            ParamType::Pan,
            AutomationRate::A_rate,
            pan,
            -1.,
            1.,
        );
        Ok(reflect_dom_object_with_proto(
            cx,
            Box::new(StereoPannerNode::new_inherited(
                node_id,
                context,
                node_options,
                &pan,
            )),
            window,
            proto,
        ))
    }

    /// <https://webaudio.github.io/web-audio-api/#dom-stereopannernode-pan>
    fn Pan(&self) -> DomRoot<AudioParam> {
        DomRoot::from_ref(&self.pan)
    }
}

impl Convert<ServoMediaStereoPannerOptions> for StereoPannerOptions {
    fn convert(self) -> ServoMediaStereoPannerOptions {
        ServoMediaStereoPannerOptions { pan: *self.pan }
    }
}
