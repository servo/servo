/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;
use servo_media::audio::node::{AudioNodeInit, AudioNodeType};
use servo_media::audio::param::ParamType;
use servo_media::audio::stereo_panner::StereoPannerOptions as ServoMediaStereoPannerOptions;

use crate::dom::audioparam::AudioParam;
use crate::dom::audioscheduledsourcenode::AudioScheduledSourceNode;
use crate::dom::baseaudiocontext::BaseAudioContext;
use crate::dom::bindings::codegen::Bindings::AudioNodeBinding::{
    ChannelCountMode, ChannelInterpretation,
};
use crate::dom::bindings::codegen::Bindings::AudioParamBinding::AutomationRate;
use crate::dom::bindings::codegen::Bindings::StereoPannerNodeBinding::{
    StereoPannerNodeMethods, StereoPannerOptions,
};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct StereoPannerNode {
    source_node: AudioScheduledSourceNode,
    pan: Dom<AudioParam>,
}

impl StereoPannerNode {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new_inherited(
        window: &Window,
        context: &BaseAudioContext,
        options: &StereoPannerOptions,
    ) -> Fallible<StereoPannerNode> {
        let node_options = options.parent.unwrap_or(
            2,
            ChannelCountMode::Clamped_max,
            ChannelInterpretation::Speakers,
        );
        if node_options.mode == ChannelCountMode::Max {
            return Err(Error::NotSupported);
        }
        if node_options.count > 2 || node_options.count == 0 {
            return Err(Error::NotSupported);
        }
        let source_node = AudioScheduledSourceNode::new_inherited(
            AudioNodeInit::StereoPannerNode(options.into()),
            context,
            node_options,
            1, /* inputs */
            1, /* outputs */
        )?;
        let node_id = source_node.node().node_id();
        let pan = AudioParam::new(
            window,
            context,
            node_id,
            AudioNodeType::StereoPannerNode,
            ParamType::Pan,
            AutomationRate::A_rate,
            *options.pan,
            -1.,
            1.,
            CanGc::note(),
        );

        Ok(StereoPannerNode {
            source_node,
            pan: Dom::from_ref(&pan),
        })
    }

    pub(crate) fn new(
        window: &Window,
        context: &BaseAudioContext,
        options: &StereoPannerOptions,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<StereoPannerNode>> {
        Self::new_with_proto(window, None, context, options, can_gc)
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn new_with_proto(
        window: &Window,
        proto: Option<HandleObject>,
        context: &BaseAudioContext,
        options: &StereoPannerOptions,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<StereoPannerNode>> {
        let node = StereoPannerNode::new_inherited(window, context, options)?;
        Ok(reflect_dom_object_with_proto(
            Box::new(node),
            window,
            proto,
            can_gc,
        ))
    }
}

impl StereoPannerNodeMethods<crate::DomTypeHolder> for StereoPannerNode {
    // https://webaudio.github.io/web-audio-api/#dom-stereopannernode-stereopannernode
    fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        context: &BaseAudioContext,
        options: &StereoPannerOptions,
    ) -> Fallible<DomRoot<StereoPannerNode>> {
        StereoPannerNode::new_with_proto(window, proto, context, options, can_gc)
    }

    // https://webaudio.github.io/web-audio-api/#dom-stereopannernode-pan
    fn Pan(&self) -> DomRoot<AudioParam> {
        DomRoot::from_ref(&self.pan)
    }
}

impl<'a> From<&'a StereoPannerOptions> for ServoMediaStereoPannerOptions {
    fn from(options: &'a StereoPannerOptions) -> Self {
        Self { pan: *options.pan }
    }
}
