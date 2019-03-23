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
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::window::Window;
use dom_struct::dom_struct;
use servo_media::audio::node::AudioNodeInit;
use servo_media::audio::constant_source_node::ConstantSourceNodeOptions;
use servo_media::audio::param::ParamType;
use std::f32;

#[dom_struct]
pub struct ConstantSourceNode {
    source_node: AudioScheduledSourceNode,
    offset: Dom<AudioParam>,
}

impl ConstantSourceNode {
    #[allow(unrooted_must_root)]
    pub fn new_inherited(
        window: &Window,
        context: &BaseAudioContext,
        options: &ConstantSourceOptions,
    ) -> Fallible<ConstantSourceNode> {
        let node_options =
            options
                .parent
                .unwrap_or(2, ChannelCountMode::Max, ChannelInterpretation::Speakers);
        let source_node = AudioScheduledSourceNode::new_inherited(
            AudioNodeInit::ConstantSourceNode(options.into()),
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

        Ok(ConstantSourceNode {
            source_node,
            offset: Dom::from_ref(&offset),
        })
    }

    #[allow(unrooted_must_root)]
    pub fn new(
        window: &Window,
        context: &BaseAudioContext,
        options: &ConstantSourceOptions,
    ) -> Fallible<DomRoot<ConstantSourceNode>> {
        let node = ConstantSourceNode::new_inherited(window, context, options)?;
        Ok(reflect_dom_object(
            Box::new(node),
            window,
            OscillatorNodeBinding::Wrap,
        ))
    }

    pub fn Constructor(
        window: &Window,
        context: &BaseAudioContext,
        options: &ConstantSourceOptions,
    ) -> Fallible<DomRoot<ConstantSourceNode>> {
        ConstantSourceNode::new(window, context, options)
    }
}