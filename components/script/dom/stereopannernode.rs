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
use crate::dom::bindings::codegen::Bindings::StereoPannerNodeBinding::StereoPannerOptions;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::window::Window;
use dom_struct::dom_struct;
use servo_media::audio::node::AudioNodeInit;
use servo_media::audio::param::ParamType;
use servo_media::audio::stereo_panner::StereoPannerOptions as ServoMediaStereoPannerOptions;

#[dom_struct]
pub struct StereoPannerNode {
    source_node: AudioScheduledSourceNode,
    pan: Dom<AudioParam>,
}

impl StereoPannerNode {
    #[allow(unrooted_must_root)]
    pub fn new_inherited(
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
            ParamType::Pan,
            AutomationRate::A_rate,
            *options.pan,
            -1.,
            1.,
        );

        Ok(StereoPannerNode {
            source_node,
            pan: Dom::from_ref(&pan),
        })
    }

    #[allow(unrooted_must_root)]
    pub fn new(
        window: &Window,
        context: &BaseAudioContext,
        options: &StereoPannerOptions,
    ) -> Fallible<DomRoot<StereoPannerNode>> {
        let node = StereoPannerNode::new_inherited(window, context, options)?;
        Ok(reflect_dom_object(Box::new(node), window))
    }

    #[allow(non_snake_case)]
    pub fn Constructor(
        window: &Window,
        context: &BaseAudioContext,
        options: &StereoPannerOptions,
    ) -> Fallible<DomRoot<StereoPannerNode>> {
        StereoPannerNode::new(window, context, options)
    }
}

impl StereoPannerNodeMethods for StereoPannerNode {
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
