/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::audioparam::AudioParam;
use crate::dom::audioscheduledsourcenode::AudioScheduledSourceNode;
use crate::dom::baseaudiocontext::BaseAudioContext;
use crate::dom::bindings::codegen::Bindings::AudioNodeBinding::{
    ChannelCountMode, ChannelInterpretation,
};
use crate::dom::bindings::codegen::Bindings::AudioParamBinding::AutomationRate;
use crate::dom::bindings::codegen::Bindings::StereoPannerNodeBinding::StereoPannerNodeMethods;
use crate::dom::bindings::codegen::Bindings::StereoPannerNodeBinding::{
    self, StereoPannerOptions,
};
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::window::Window;
use dom_struct::dom_struct;
use servo_media::audio::stereo_panner_node::StereoPannerOptions as ServoMediaStereoPannerOptions;
use servo_media::audio::node::AudioNodeInit;
use servo_media::audio::param::ParamType;
use std::f32;

#[dom_struct]
pub struct StereoPannerNode {
    source_node: AudioScheduledSourceNode,
    offset: Dom<AudioParam>,
}

impl StereoPannerNode {
    #[allow(unrooted_must_root)]
    pub fn new_inherited(
        window: &Window,
        context: &BaseAudioContext,
        options: &StereoPannerOptions,
    ) -> Fallible<StereoPannerNode> {
        let node_options =
            options
                .parent
                .unwrap_or(2, ChannelCountMode::Max, ChannelInterpretation::Speakers);
        let source_node = AudioScheduledSourceNode::new_inherited(
            AudioNodeInit::StereoPannerNode(options.into()),
            context,
            node_options,
            0, /* inputs */
            1, /* outputs */
        )?;
        let node_id = source_node.node().node_id();
        let offset = AudioParam::new(
            window,
            context,
            node_id,
            ParamType::Offset,
            AutomationRate::A_rate,
            1.,
            f32::MIN,
            f32::MAX,
        );

        Ok(StereoPannerNode {
            source_node,
            offset: Dom::from_ref(&offset),
        })
    }

    #[allow(unrooted_must_root)]
    pub fn new(
        window: &Window,
        context: &BaseAudioContext,
        options: &StereoPannerOptions,
    ) -> Fallible<DomRoot<StereoPannerNode>> {
        let node = StereoPannerNode::new_inherited(window, context, options)?;
        Ok(reflect_dom_object(
            Box::new(node),
            window,
            StereoPannerNodeBinding::Wrap,
        ))
    }

    pub fn Constructor(
        window: &Window,
        context: &BaseAudioContext,
        options: &StereoPannerOptions,
    ) -> Fallible<DomRoot<StereoPannerNode>> {
        StereoPannerNode::new(window, context, options)
    }
}

impl StereoPannerNodeMethods for StereoPannerNode {
    fn Offset(&self) -> DomRoot<AudioParam> {
        DomRoot::from_ref(&self.offset)
    }
}

impl<'a> From<&'a StereoPannerOptions> for ServoMediaStereoPannerOptions {
    fn from(options: &'a StereoPannerOptions) -> Self {
        Self {
            offset: *options.offset,
        }
    }
}
